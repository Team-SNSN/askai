use crate::error::{AskAiError, Result};
use colored::*;
use tokio::process::Command;

pub struct CommandRunner {
    dry_run: bool,
}

impl CommandRunner {
    pub fn new() -> Self {
        Self { dry_run: false }
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub async fn execute(&self, command: &str) -> Result<String> {
        if self.dry_run {
            eprintln!("{} {}", "[DRY-RUN]".yellow().bold(), command);
            return Ok(format!("[DRY-RUN] {}", command));
        }

        println!("{} {}", "[>] Executing:".cyan(), command);

        let output = Command::new("bash")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| AskAiError::ExecutionError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            eprintln!("{} {}", "[X] Error:".red(), stderr);
            return Err(AskAiError::ExecutionError(stderr));
        }

        if !stdout.is_empty() {
            println!("{}", stdout);
        }

        Ok(stdout)
    }
}

impl Default for CommandRunner {
    fn default() -> Self {
        Self::new()
    }
}
