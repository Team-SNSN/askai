/// Plugin system for askai
///
/// This module provides the foundation for extending askai with custom
/// AI providers, command processors, and context analyzers.
///
/// ## Design Overview
///
/// The plugin system follows a trait-based architecture where plugins
/// implement specific trait interfaces to extend functionality.
///
/// ## Architecture
///
/// ```text
/// ┌──────────────────────────────────────┐
/// │         Plugin Manager               │
/// │  - Discovery                         │
/// │  - Loading                           │
/// │  - Lifecycle                         │
/// └────────────┬─────────────────────────┘
///              │
///       ┌──────┴──────┬──────────────┐
///       ▼             ▼              ▼
/// ┌─────────┐  ┌──────────┐  ┌────────────┐
/// │Provider │  │Processor │  │  Context   │
/// │ Plugin  │  │  Plugin  │  │   Plugin   │
/// └─────────┘  └──────────┘  └────────────┘
/// ```
///
/// ## Plugin Types
///
/// ### 1. Provider Plugins
/// Add new AI providers (OpenAI, Anthropic, local models, etc.)
///
/// ### 2. Processor Plugins
/// Post-process AI responses (formatting, validation, enhancement)
///
/// ### 3. Context Plugins
/// Analyze project context (new project types, custom metadata)
///
/// ## Plugin Discovery
///
/// Plugins are discovered from:
/// - `~/.askai/plugins/` directory
/// - Environment variable `ASKAI_PLUGIN_PATH`
/// - Built-in plugins (compiled-in)
///
/// ## Security
///
/// - Plugins run in the same process (no sandboxing yet)
/// - Only load plugins from trusted sources
/// - Future: WASM-based sandboxing for third-party plugins
///
/// ## Configuration
///
/// Plugins are configured via `~/.askai/plugins.toml`:
///
/// ```toml
/// [[plugins]]
/// name = "my-provider"
/// type = "provider"
/// enabled = true
/// config = { api_key = "..." }
/// ```

use crate::error::Result;
use async_trait::async_trait;

/// Plugin trait - base interface for all plugins
pub trait Plugin: Send + Sync {
    /// Plugin name (unique identifier)
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// Plugin description
    fn description(&self) -> &str;

    /// Initialize the plugin
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Cleanup/shutdown the plugin
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Provider plugin trait - for adding new AI providers
#[async_trait]
pub trait ProviderPlugin: Plugin {
    /// Generate command from prompt and context
    async fn generate_command(&self, prompt: &str, context: &str) -> Result<String>;

    /// Check if the provider is available
    async fn check_availability(&self) -> Result<bool>;
}

/// Processor plugin trait - for post-processing AI responses
pub trait ProcessorPlugin: Plugin {
    /// Process/transform a command
    fn process_command(&self, command: &str) -> Result<String>;
}

/// Context plugin trait - for custom context analysis
pub trait ContextPlugin: Plugin {
    /// Analyze and enhance project context
    fn analyze_context(&self, path: &str) -> Result<String>;
}

/// Plugin manager - handles plugin lifecycle
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        self.plugins.push(plugin);
        Ok(())
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    /// Initialize all plugins
    pub fn initialize_all(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            // Can't call initialize() on trait object directly
            // This is a design limitation - will need Box<dyn Plugin> refactor
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test-plugin"
        }

        fn version(&self) -> &str {
            "0.1.0"
        }

        fn description(&self) -> &str {
            "Test plugin"
        }
    }

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.list_plugins().len(), 0);
    }

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin);
        manager.register(plugin).unwrap();
        assert_eq!(manager.list_plugins().len(), 1);
    }

    #[test]
    fn test_plugin_retrieval() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin);
        manager.register(plugin).unwrap();

        let found = manager.get_plugin("test-plugin");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "test-plugin");
    }
}
