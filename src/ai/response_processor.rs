use crate::error::{AskAiError, Result};
use once_cell::sync::Lazy;
use regex::Regex;

/// 사전 컴파일된 정규표현식 (성능 최적화)
static CODE_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"```(?:bash|sh)?\n(.*?)\n```").unwrap()
});

/// AI provider 응답을 후처리하여 실제 bash 명령어만 추출하는 공통 모듈
pub struct ResponseProcessor;

impl ResponseProcessor {
    /// AI 응답을 후처리하여 실제 bash 명령어만 추출
    ///
    /// # Arguments
    /// * `raw` - AI로부터 받은 원본 응답 문자열
    ///
    /// # Returns
    /// * `Result<String>` - 정제된 bash 명령어 또는 에러
    ///
    /// # Examples
    /// ```
    /// use askai::ai::response_processor::ResponseProcessor;
    ///
    /// let result = ResponseProcessor::process("```bash\nls -la\n```");
    /// assert_eq!(result.unwrap(), "ls -la");
    /// ```
    pub fn process(raw: &str) -> Result<String> {
        let mut command = raw.to_string();

        // 1. 메타 응답 감지 (에러 처리)
        let invalid_patterns = [
            "I am unable to",
            "I cannot",
            "I can't",
            "I will try to find",
            "I'm sorry",
            "As an AI",
            "I don't have the ability",
        ];

        for pattern in &invalid_patterns {
            if command.to_lowercase().contains(&pattern.to_lowercase()) {
                return Err(AskAiError::AiCliError(
                    format!(
                        "AI returned an explanation instead of a command: {}\n\
                         Please try again with a clearer prompt.",
                        command
                    )
                ));
            }
        }

        // 2. 마크다운 코드 블록 제거
        // ```bash\ncommand\n``` → command
        // ```\ncommand\n``` → command
        if command.contains("```") {
            // 사전 컴파일된 정규표현식 사용 (성능 최적화)
            if let Some(caps) = CODE_BLOCK_REGEX.captures(&command) {
                command = caps.get(1).map_or("", |m| m.as_str()).to_string();
            } else {
                // 단순히 ``` 제거
                command = command.replace("```bash", "")
                    .replace("```sh", "")
                    .replace("```", "")
                    .trim()
                    .to_string();
            }
        }

        // 3. 설명 프리픽스 제거
        let prefixes = [
            "Here is the command:",
            "The command is:",
            "You can use:",
            "Try this:",
            "Run this:",
            "Execute:",
            "Command:",
        ];

        for prefix in &prefixes {
            if command.to_lowercase().starts_with(&prefix.to_lowercase()) {
                command = command[prefix.len()..].trim().to_string();
            }
        }

        // 4. 여러 줄인 경우 첫 번째 유효한 명령어만 추출
        let lines: Vec<&str> = command.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.len() > 1 {
            // 첫 번째 줄이 설명이고 두 번째 줄이 명령어인 경우 감지
            if lines[0].ends_with(':') || lines[0].len() > 50 {
                command = lines.get(1).unwrap_or(&lines[0]).to_string();
            } else {
                command = lines[0].to_string();
            }
        }

        // 5. 최종 trim
        command = command.trim().to_string();

        // 6. 빈 명령어 체크
        if command.is_empty() {
            return Err(AskAiError::AiCliError(
                "AI returned an empty command. Please try again.".to_string()
            ));
        }

        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_command() {
        let result = ResponseProcessor::process("ls -la").unwrap();
        assert_eq!(result, "ls -la");
    }

    #[test]
    fn test_markdown_code_block() {
        // ```bash\ncommand\n```
        let result = ResponseProcessor::process("```bash\ngit status\n```").unwrap();
        assert_eq!(result, "git status");

        // ```\ncommand\n```
        let result = ResponseProcessor::process("```\ndate\n```").unwrap();
        assert_eq!(result, "date");
    }

    #[test]
    fn test_prefix_removal() {
        let result = ResponseProcessor::process("Here is the command: ls -la").unwrap();
        assert_eq!(result, "ls -la");

        let result = ResponseProcessor::process("You can use: git status").unwrap();
        assert_eq!(result, "git status");
    }

    #[test]
    fn test_multiline_response() {
        // 첫 번째 줄만 추출
        let result = ResponseProcessor::process("ls -la\nThis lists all files").unwrap();
        assert_eq!(result, "ls -la");

        // 설명이 첫 줄인 경우 두 번째 줄 추출
        let result = ResponseProcessor::process("This is a command to list files:\nls -la").unwrap();
        assert_eq!(result, "ls -la");
    }

    #[test]
    fn test_invalid_response() {
        // "I am unable to" 패턴 감지
        let result = ResponseProcessor::process(
            "I am unable to execute shell commands. I will try to find a solution."
        );
        assert!(result.is_err());

        // "I cannot" 패턴 감지
        let result = ResponseProcessor::process("I cannot run this command");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_response() {
        let result = ResponseProcessor::process("");
        assert!(result.is_err());

        let result = ResponseProcessor::process("   \n  \n  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_case() {
        // 마크다운 + 프리픽스 조합
        let result = ResponseProcessor::process(
            "Here is the command:\n```bash\nfind . -name \"*.txt\"\n```"
        );
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.contains("find"));
        assert!(cmd.contains("*.txt"));
    }
}
