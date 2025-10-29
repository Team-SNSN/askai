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
        println!("\n{}", "📋 생성된 명령어:".cyan().bold());

        let colored_command = match danger_level {
            DangerLevel::Low => command.green(),
            DangerLevel::Medium => command.yellow(),
            DangerLevel::High => command.red().bold(),
        };

        println!("  {}", colored_command);

        let danger_msg = match danger_level {
            DangerLevel::Low => "",
            DangerLevel::Medium => " ⚠️  (주의 필요)",
            DangerLevel::High => " 🚨 (매우 위험)",
        };

        println!("\n{}{}", "위험도:".bold(), danger_msg);

        let result = Confirm::new()
            .with_prompt("이 명령어를 실행하시겠습니까?")
            .default(false)
            .interact()
            .map_err(|_| AskAiError::UserCancelled)?;

        Ok(result)
    }
}
