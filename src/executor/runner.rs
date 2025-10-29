use crate::error::{AskAiError, Result};
use colored::*;
use tokio::process::Command;

pub struct CommandRunner;

impl CommandRunner {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, command: &str) -> Result<String> {
        println!("{} {}", "▶️  실행 중:".cyan(), command);

        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| AskAiError::ExecutionError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            eprintln!("{} {}", "❌ 에러:".red(), stderr);
            return Err(AskAiError::ExecutionError(stderr));
        }

        if !stdout.is_empty() {
            println!("{}", stdout);
        }

        Ok(stdout)
    }
}
