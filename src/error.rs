use thiserror::Error;

#[derive(Error, Debug)]
pub enum AskAiError {
    #[error("Failed to execute AI CLI: {0}")]
    AiCliError(String),

    #[error("Dangerous command detected: {0}")]
    DangerousCommand(String),

    #[error("Command execution failed: {0}")]
    ExecutionError(String),

    #[error("User cancelled")]
    UserCancelled,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    // Phase 2 error types
    #[error("Project type detection failed: {0}")]
    #[allow(dead_code)]  // Reserved for future project detection errors
    ProjectDetectionError(String),

    #[error("Batch execution partially failed: {success}/{total} succeeded")]
    #[allow(dead_code)]  // Reserved for partial batch failures
    BatchPartialFailure {
        success: usize,
        total: usize,
        errors: Vec<String>,
    },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Parallel execution error: {0}")]
    #[allow(dead_code)]  // Reserved for parallel execution errors
    ParallelExecutionError(String),
}

pub type Result<T> = std::result::Result<T, AskAiError>;
