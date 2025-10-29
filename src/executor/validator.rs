use crate::error::{AskAiError, Result};

pub struct CommandValidator;

impl CommandValidator {
    const DANGEROUS_PATTERNS: &'static [&'static str] = &[
        "rm -rf /",
        "rm -rf /*",
        "dd if=/dev/zero",
        "mkfs",
        "> /dev/sd",
        "mv /* ",
        ":(){ :|:& };:", // Fork bomb
    ];

    const DANGEROUS_KEYWORDS: &'static [&'static str] =
        &["rm -rf", "mkfs", "dd if=/dev/zero", "format"];

    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, command: &str) -> Result<DangerLevel> {
        // 절대 금지 패턴
        for pattern in Self::DANGEROUS_PATTERNS {
            if command.contains(pattern) {
                return Err(AskAiError::DangerousCommand(format!(
                    "위험한 패턴 감지: {}",
                    pattern
                )));
            }
        }

        // 위험 키워드 확인
        let danger_level = if Self::DANGEROUS_KEYWORDS
            .iter()
            .any(|k| command.contains(k))
        {
            DangerLevel::High
        } else if command.contains("sudo") {
            DangerLevel::Medium
        } else {
            DangerLevel::Low
        };

        Ok(danger_level)
    }
}

#[derive(Debug, PartialEq)]
pub enum DangerLevel {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_command() {
        let validator = CommandValidator::new();

        // 위험한 명령어
        assert!(validator.validate("rm -rf /").is_err());
        assert!(validator.validate("dd if=/dev/zero of=/dev/sda").is_err());

        // 안전한 명령어
        assert!(validator.validate("ls -la").is_ok());
        assert!(validator.validate("git status").is_ok());
    }

    #[test]
    fn test_danger_levels() {
        let validator = CommandValidator::new();

        assert_eq!(validator.validate("ls").unwrap(), DangerLevel::Low);

        assert_eq!(
            validator.validate("sudo apt update").unwrap(),
            DangerLevel::Medium
        );
    }
}
