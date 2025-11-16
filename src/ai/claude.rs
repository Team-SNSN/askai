use crate::error::{AskAiError, Result};
use crate::ai::{AiProvider, factory::ProviderFactory};
use async_trait::async_trait;
use tokio::process::Command;
use once_cell::sync::Lazy;
use regex::Regex;

/// 사전 컴파일된 정규표현식 (성능 최적화)
static CODE_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"```(?:bash|sh)?\n(.*?)\n```").unwrap()
});

pub struct ClaudeProvider;

impl ClaudeProvider {
    pub fn new() -> Self {
        Self
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
        if command.contains("```") {
            // 사전 컴파일된 정규표현식 사용 (성능 최적화)
            if let Some(caps) = CODE_BLOCK_REGEX.captures(&command) {
                command = caps.get(1).map_or("", |m| m.as_str()).to_string();
            } else {
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
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    fn cli_command(&self) -> &str {
        "claude"
    }

    async fn check_installation(&self) -> Result<()> {
        // 캐싱된 설치 확인 사용 (성능 최적화)
        ProviderFactory::check_installation(
            self.cli_command(),
            "Claude CLI가 설치되어 있지 않습니다.\n\
             설치 방법: npm install -g @anthropics/claude-cli"
        ).await
    }

    async fn generate_command(&self, prompt: &str, context: &str) -> Result<String> {
        // Claude CLI가 설치되어 있는지 확인 (캐싱됨)
        self.check_installation().await?;

        // Claude용 최적화된 프롬프트 생성
        let full_prompt = format!(
            "You are a bash command generator. Convert natural language to a single bash command.\n\n\
             RULES:\n\
             - Output ONLY the bash command (no explanations, no markdown)\n\
             - Do NOT say \"I cannot\" or similar - just output the command\n\
             - Be precise and accurate\n\n\
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

        // Claude CLI 호출
        let output = Command::new(self.cli_command())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postprocess_clean_command() {
        let provider = ClaudeProvider::new();
        let result = provider.postprocess_response("ls -la").unwrap();
        assert_eq!(result, "ls -la");
    }

    #[test]
    fn test_postprocess_markdown_code_block() {
        let provider = ClaudeProvider::new();

        let result = provider.postprocess_response("```bash\ngit status\n```").unwrap();
        assert_eq!(result, "git status");

        let result = provider.postprocess_response("```\ndate\n```").unwrap();
        assert_eq!(result, "date");
    }
}
