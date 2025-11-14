use crate::error::{AskAiError, Result};
use crate::ai::{AiProvider, gemini::GeminiProvider, claude::ClaudeProvider, codex::CodexProvider};
use std::sync::Arc;

/// Provider factory for creating AI providers based on the provider name
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider instance based on the provider name
    ///
    /// # Arguments
    /// * `provider_name` - The name of the provider (gemini, claude, codex)
    ///
    /// # Returns
    /// * `Result<Arc<dyn AiProvider>>` - Boxed provider instance
    ///
    /// # Errors
    /// * Returns `AskAiError::AiCliError` if the provider name is unknown
    pub fn create(provider_name: &str) -> Result<Arc<dyn AiProvider>> {
        match provider_name.to_lowercase().as_str() {
            "gemini" => Ok(Arc::new(GeminiProvider::new())),
            "claude" => Ok(Arc::new(ClaudeProvider::new())),
            "codex" => Ok(Arc::new(CodexProvider::new())),
            _ => Err(AskAiError::AiCliError(
                format!(
                    "알 수 없는 AI provider: {}\n\
                     지원되는 provider: gemini, claude, codex",
                    provider_name
                )
            )),
        }
    }

    /// Get a list of all supported provider names
    #[allow(dead_code)]
    pub fn supported_providers() -> Vec<&'static str> {
        vec!["gemini", "claude", "codex"]
    }

    /// Check if a provider name is supported
    #[allow(dead_code)]
    pub fn is_supported(provider_name: &str) -> bool {
        Self::supported_providers().contains(&provider_name.to_lowercase().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_gemini_provider() {
        let provider = ProviderFactory::create("gemini");
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "gemini");
    }

    #[test]
    fn test_create_claude_provider() {
        let provider = ProviderFactory::create("claude");
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "claude");
    }

    #[test]
    fn test_create_codex_provider() {
        let provider = ProviderFactory::create("codex");
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "codex");
    }

    #[test]
    fn test_create_unknown_provider() {
        let provider = ProviderFactory::create("unknown");
        assert!(provider.is_err());
    }

    #[test]
    fn test_case_insensitive() {
        let provider1 = ProviderFactory::create("GEMINI");
        assert!(provider1.is_ok());

        let provider2 = ProviderFactory::create("GeMiNi");
        assert!(provider2.is_ok());
    }

    #[test]
    fn test_supported_providers() {
        let providers = ProviderFactory::supported_providers();
        assert_eq!(providers.len(), 3);
        assert!(providers.contains(&"gemini"));
        assert!(providers.contains(&"claude"));
        assert!(providers.contains(&"codex"));
    }

    #[test]
    fn test_is_supported() {
        assert!(ProviderFactory::is_supported("gemini"));
        assert!(ProviderFactory::is_supported("claude"));
        assert!(ProviderFactory::is_supported("codex"));
        assert!(!ProviderFactory::is_supported("unknown"));
    }
}
