use crate::error::{AskAiError, Result};
use tokio::process::Command;

pub struct GeminiProvider;

impl GeminiProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_command(&self, prompt: &str, context: &str) -> Result<String> {
        // Gemini CLI가 설치되어 있는지 확인
        self.check_installation().await?;

        // 최적화된 프롬프트 생성 (속도와 품질 균형)
        let full_prompt = format!(
            "You are a bash command generator. Convert natural language to a single bash command.\n\n\
             RULES:\n\
             - Output ONLY the bash command (no explanations, no markdown)\n\
             - Do NOT say \"I cannot\" or similar - just output the command\n\n\
             Context: {}\n\
             Request: {}\n\n\
             Examples:\n\
             \"파일 목록\" → ls -la\n\
             \"git 상태\" → git status\n\
             \"txt 파일 찾기\" → find . -name \"*.txt\"\n\
             \"현재 시간\" → date\n\n\
             Command:",
            context, prompt
        );

        // Gemini CLI 호출 (기본 모델 사용)
        let output = Command::new("gemini")
            .arg(&full_prompt)
            .output()
            .await
            .map_err(|e| AskAiError::AiCliError(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AskAiError::AiCliError(error.to_string()));
        }

        let raw_output = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        // 후처리: AI 응답을 정제하여 실제 명령어만 추출
        let command = self.postprocess_response(&raw_output)?;

        Ok(command)
    }

    /// AI 응답을 후처리하여 실제 bash 명령어만 추출
    fn postprocess_response(&self, raw: &str) -> Result<String> {
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
                        "AI가 명령어 대신 설명을 반환했습니다: {}\n\
                         프롬프트를 더 명확하게 작성하거나 다시 시도해주세요.",
                        command
                    )
                ));
            }
        }

        // 2. 마크다운 코드 블록 제거
        // ```bash\ncommand\n``` → command
        // ```\ncommand\n``` → command
        if command.contains("```") {
            // ```bash 또는 ``` 로 시작하는 코드 블록 추출
            let re = regex::Regex::new(r"```(?:bash|sh)?\n(.*?)\n```").unwrap();
            if let Some(caps) = re.captures(&command) {
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
                "AI가 빈 명령어를 반환했습니다. 다시 시도해주세요.".to_string()
            ));
        }

        Ok(command)
    }

    async fn check_installation(&self) -> Result<()> {
        let output = Command::new("which")
            .arg("gemini")
            .output()
            .await
            .map_err(|e| AskAiError::AiCliError(e.to_string()))?;

        if !output.status.success() {
            return Err(AskAiError::AiCliError(
                "Gemini CLI가 설치되어 있지 않습니다.\n\
                 설치 방법: npm install -g @google/generative-ai-cli"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postprocess_clean_command() {
        let provider = GeminiProvider::new();
        let result = provider.postprocess_response("ls -la").unwrap();
        assert_eq!(result, "ls -la");
    }

    #[test]
    fn test_postprocess_markdown_code_block() {
        let provider = GeminiProvider::new();

        // ```bash\ncommand\n```
        let result = provider.postprocess_response("```bash\ngit status\n```").unwrap();
        assert_eq!(result, "git status");

        // ```\ncommand\n```
        let result = provider.postprocess_response("```\ndate\n```").unwrap();
        assert_eq!(result, "date");
    }

    #[test]
    fn test_postprocess_prefix_removal() {
        let provider = GeminiProvider::new();

        let result = provider.postprocess_response("Here is the command: ls -la").unwrap();
        assert_eq!(result, "ls -la");

        let result = provider.postprocess_response("You can use: git status").unwrap();
        assert_eq!(result, "git status");
    }

    #[test]
    fn test_postprocess_multiline_response() {
        let provider = GeminiProvider::new();

        // 첫 번째 줄만 추출
        let result = provider.postprocess_response("ls -la\nThis lists all files").unwrap();
        assert_eq!(result, "ls -la");

        // 설명이 첫 줄인 경우 두 번째 줄 추출
        let result = provider.postprocess_response("This is a command to list files:\nls -la").unwrap();
        assert_eq!(result, "ls -la");
    }

    #[test]
    fn test_postprocess_invalid_response() {
        let provider = GeminiProvider::new();

        // "I am unable to" 패턴 감지
        let result = provider.postprocess_response(
            "I am unable to execute shell commands. I will try to find a solution."
        );
        assert!(result.is_err());

        // "I cannot" 패턴 감지
        let result = provider.postprocess_response("I cannot run this command");
        assert!(result.is_err());
    }

    #[test]
    fn test_postprocess_empty_response() {
        let provider = GeminiProvider::new();

        let result = provider.postprocess_response("");
        assert!(result.is_err());

        let result = provider.postprocess_response("   \n  \n  ");
        assert!(result.is_err());
    }

    #[test]
    fn test_postprocess_complex_case() {
        let provider = GeminiProvider::new();

        // 마크다운 + 프리픽스 조합
        let result = provider.postprocess_response(
            "Here is the command:\n```bash\nfind . -name \"*.txt\"\n```"
        );
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.contains("find"));
        assert!(cmd.contains("*.txt"));
    }
}
