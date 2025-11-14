use std::env;
use super::history::HistoryStore;

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
}
