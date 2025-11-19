use crate::error::{AskAiError, Result};
use crate::ai::{AiProvider, factory::ProviderFactory, response_processor::ResponseProcessor, prompt_template::PromptTemplate};
use async_trait::async_trait;
use tokio::process::Command;

pub struct CodexProvider;

impl CodexProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AiProvider for CodexProvider {
    fn name(&self) -> &str {
        "codex"
    }

    fn cli_command(&self) -> &str {
        "codex"
    }

    async fn check_installation(&self) -> Result<()> {
        // 캐싱된 설치 확인 사용 (성능 최적화)
        ProviderFactory::check_installation(
            self.cli_command(),
            "Codex CLI is not installed.\n\
             Installation: npm install -g openai-codex-cli"
        ).await
    }

    async fn generate_command(&self, prompt: &str, context: &str) -> Result<String> {
        // Codex CLI가 설치되어 있는지 확인 (캐싱됨)
        self.check_installation().await?;

        // 공통 프롬프트 템플릿 사용 (Codex 전용 규칙 포함)
        let full_prompt = PromptTemplate::for_codex(prompt, context);

        // Codex CLI 호출
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

        // 후처리: AI 응답을 정제하여 실제 명령어만 추출 (공통 ResponseProcessor 사용)
        let command = ResponseProcessor::process(&raw_output)?;

        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = CodexProvider::new();
        assert_eq!(provider.name(), "codex");
        assert_eq!(provider.cli_command(), "codex");
    }
}
