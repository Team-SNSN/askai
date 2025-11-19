use crate::error::{AskAiError, Result};
use crate::executor::validator::DangerLevel;
use colored::*;
use dialoguer::Confirm;

pub struct ConfirmPrompt;

impl ConfirmPrompt {
    pub fn new() -> Self {
        Self
    }

    pub fn confirm_execution(&self, command: &str, danger_level: DangerLevel) -> Result<bool> {
        // Output to stderr to avoid passing to eval
        eprintln!("\n{}", "[>] Generated command:".cyan().bold());

        let colored_command = match danger_level {
            DangerLevel::Low => command.green(),
            DangerLevel::Medium => command.yellow(),
            DangerLevel::High => command.red().bold(),
        };

        eprintln!("  {}", colored_command);

        let (danger_msg, danger_level_str) = match danger_level {
            DangerLevel::Low => ("Low".green(), "[*] Safe"),
            DangerLevel::Medium => ("Medium".yellow(), "[!] Caution"),
            DangerLevel::High => ("High".red().bold(), "[!!!] Very dangerous"),
        };

        eprintln!("\n{} {} - {}", "Risk level:".bold(), danger_msg, danger_level_str);

        let result = Confirm::new()
            .with_prompt("Execute this command?")
            .default(false)
            .interact()
            .map_err(|_| AskAiError::UserCancelled)?;

        Ok(result)
    }
}
