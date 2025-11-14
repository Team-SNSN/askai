pub mod context;
pub mod gemini;
pub mod claude;
pub mod codex;
pub mod factory;

use crate::error::Result;
use async_trait::async_trait;

/// AI provider trait for extensible CLI integration
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Provider name (e.g., "gemini", "claude", "codex")
    fn name(&self) -> &str;

    /// CLI command name (e.g., "gemini", "claude", "codex")
    fn cli_command(&self) -> &str;

    /// Check if the CLI is installed on the system
    async fn check_installation(&self) -> Result<()>;

    /// Generate a bash command from natural language prompt
    async fn generate_command(&self, prompt: &str, context: &str) -> Result<String>;
}
