pub mod project;
pub mod detector;
pub mod scanner;

use std::env;
use crate::ai::history::HistoryStore;
pub use project::{ProjectInfo, ProjectType};
pub use detector::ProjectDetector;
pub use scanner::{ProjectScanner, ScanResult};

/// 현재 실행 환경의 컨텍스트 정보를 수집
pub fn get_current_context() -> String {
    let cwd = env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let shell = env::var("SHELL").unwrap_or_else(|_| "bash".to_string());

    format!(
        "Current directory: {}\n\
         Shell: {}\n\
         OS: {}",
        cwd,
        shell,
        std::env::consts::OS
    )
}

/// RAG를 사용하여 관련 히스토리를 포함한 향상된 컨텍스트 생성
///
/// # Arguments
/// * `prompt` - 사용자 프롬프트
///
/// # Returns
/// * 기본 컨텍스트 + 관련 과거 명령어 히스토리
pub fn get_context_with_history(prompt: &str) -> String {
    let base_context = get_current_context();

    // 히스토리 로드 및 관련 항목 검색
    let store = HistoryStore::new();
    let relevant_history = store
        .get_relevant_history(prompt, 3) // 최대 3개의 관련 항목
        .unwrap_or_else(|_| Vec::new());

    // 관련 히스토리가 있으면 컨텍스트에 추가
    if !relevant_history.is_empty() {
        let history_context = store.format_as_context(&relevant_history);
        format!("{}{}", base_context, history_context)
    } else {
        base_context
    }
}

/// 프로젝트 정보를 포함한 향상된 컨텍스트 생성 (Phase 3용)
///
/// # Arguments
/// * `prompt` - 사용자 프롬프트
///
/// # Returns
/// * 기본 컨텍스트 + 프로젝트 정보 + 히스토리
pub fn get_context_with_project(prompt: &str) -> String {
    let mut context = get_current_context();

    // 현재 디렉토리의 프로젝트 정보 감지
    if let Ok(current_dir) = env::current_dir() {
        let project_info = ProjectDetector::detect(&current_dir);

        // Unknown이 아니면 프로젝트 정보 추가
        if project_info.primary_type() != &ProjectType::Unknown {
            context.push('\n');
            context.push_str(&project_info.to_context_string());
        }
    }

    // 히스토리도 추가
    let store = HistoryStore::new();
    let relevant_history = store
        .get_relevant_history(prompt, 3)
        .unwrap_or_else(|_| Vec::new());

    if !relevant_history.is_empty() {
        let history_context = store.format_as_context(&relevant_history);
        context.push_str(&history_context);
    }

    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_context() {
        let context = get_current_context();
        assert!(context.contains("Current directory:"));
        assert!(context.contains("Shell:"));
        assert!(context.contains("OS:"));
    }

    #[test]
    fn test_get_context_with_project() {
        let context = get_context_with_project("test command");

        assert!(context.contains("Current directory:"));

        // 현재 프로젝트는 Rust 프로젝트이므로 프로젝트 정보가 포함되어야 함
        assert!(context.contains("Project:") || context.contains("rust"));
    }
}
