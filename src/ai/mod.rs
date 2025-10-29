pub mod gemini;

use crate::error::Result;

pub trait AiProvider {
    async fn generate_command(&self, prompt: &str, context: &str) -> Result<String>;
}
