use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// askai 사용자 설정
///
/// 설정 파일은 ~/.askai/config.toml에 저장됩니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 기본 AI provider (gemini, claude, codex)
    #[serde(default = "default_provider")]
    pub default_provider: String,

    /// 안전한 명령어 자동 승인 여부
    #[serde(default = "default_auto_approve")]
    pub auto_approve_safe_commands: bool,

    /// 히스토리 파일 경로
    #[serde(default = "default_history_path")]
    pub history_path: String,

    /// RAG 기능 활성화 여부
    #[serde(default = "default_enable_rag")]
    pub enable_rag: bool,

    /// 최대 병렬 작업 개수 (Phase 3용)
    #[serde(default = "default_max_parallel")]
    pub max_parallel_jobs: usize,
}

fn default_provider() -> String {
    "gemini".to_string()
}

fn default_auto_approve() -> bool {
    false
}

fn default_history_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.askai_history.json", home)
}

fn default_enable_rag() -> bool {
    true
}

fn default_max_parallel() -> usize {
    4
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_provider: default_provider(),
            auto_approve_safe_commands: default_auto_approve(),
            history_path: default_history_path(),
            enable_rag: default_enable_rag(),
            max_parallel_jobs: default_max_parallel(),
        }
    }
}

impl Config {
    /// 설정 파일 경로 가져오기
    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".askai").join("config.toml")
    }

    /// 설정 디렉토리 경로
    fn config_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".askai")
    }

    /// 설정 파일에서 로드 (없으면 기본값 사용)
    ///
    /// # Examples
    /// ```
    /// use askai::config::Config;
    ///
    /// let config = Config::load().unwrap();
    /// assert_eq!(config.default_provider, "gemini");
    /// ```
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        // 설정 파일이 없으면 기본값 반환
        if !config_path.exists() {
            return Ok(Self::default());
        }

        // 설정 파일 읽기
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| crate::error::AskAiError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e)
            ))?;

        Ok(config)
    }

    /// 설정을 파일에 저장
    ///
    /// # Examples
    /// ```no_run
    /// use askai::config::Config;
    ///
    /// let mut config = Config::default();
    /// config.default_provider = "claude".to_string();
    /// config.save().unwrap();
    /// ```
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir();
        let config_path = Self::config_path();

        // 디렉토리가 없으면 생성
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // TOML로 직렬화
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| crate::error::AskAiError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e)
            ))?;

        // 파일에 쓰기
        fs::write(&config_path, toml_string)?;

        Ok(())
    }

    /// 설정 파일 초기화 (기본값으로)
    pub fn init() -> Result<()> {
        let config = Self::default();
        config.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_provider, "gemini");
        assert_eq!(config.auto_approve_safe_commands, false);
        assert_eq!(config.enable_rag, true);
        assert_eq!(config.max_parallel_jobs, 4);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_string = toml::to_string(&config).unwrap();

        assert!(toml_string.contains("default_provider"));
        assert!(toml_string.contains("gemini"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            default_provider = "claude"
            auto_approve_safe_commands = true
            enable_rag = false
            max_parallel_jobs = 8
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default_provider, "claude");
        assert_eq!(config.auto_approve_safe_commands, true);
        assert_eq!(config.enable_rag, false);
        assert_eq!(config.max_parallel_jobs, 8);
    }
}
