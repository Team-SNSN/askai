use std::env;

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
