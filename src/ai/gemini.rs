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

        // 프롬프트 생성
        let full_prompt = format!(
            "Context: {}\n\n\
             User request: {}\n\n\
             Generate a single bash command to accomplish this task. \
             Output ONLY the command, nothing else. No explanation, no markdown, just the raw command.",
            context, prompt
        );

        // Gemini CLI 호출
        let output = Command::new("gemini")
            .arg(&full_prompt)
            .output()
            .await
            .map_err(|e| AskAiError::AiCliError(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AskAiError::AiCliError(error.to_string()));
        }

        let command = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

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
