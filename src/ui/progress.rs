use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

/// 스피너 스타일 (AI 명령어 생성 중)
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

/// 프로그레스 바 스타일 (배치 작업용)
pub fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░ "),
    );
    pb
}

/// 멀티 프로그레스 바 (여러 작업 동시 표시)
pub struct MultiProgressDisplay {
    multi: MultiProgress,
}

impl MultiProgressDisplay {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
        }
    }

    /// 새 프로그레스 바 추가
    pub fn add_bar(&self, total: u64) -> ProgressBar {
        let pb = create_progress_bar(total);
        self.multi.add(pb)
    }

    /// 새 스피너 추가
    pub fn add_spinner(&self, message: &str) -> ProgressBar {
        let spinner = create_spinner(message);
        self.multi.add(spinner)
    }

    /// 완료 메시지와 함께 스피너 종료
    pub fn finish_spinner(&self, spinner: &ProgressBar, message: &str) {
        spinner.finish_with_message(format!("{} {}", "✓".green(), message));
    }

    /// 에러 메시지와 함께 스피너 종료
    pub fn fail_spinner(&self, spinner: &ProgressBar, message: &str) {
        spinner.finish_with_message(format!("{} {}", "✗".red(), message));
    }
}

impl Default for MultiProgressDisplay {
    fn default() -> Self {
        Self::new()
    }
}

/// 배치 작업용 프로그레스 디스플레이
pub struct BatchProgressDisplay {
    multi: MultiProgress,
    main_bar: ProgressBar,
}

impl BatchProgressDisplay {
    /// 새 배치 프로그레스 생성
    pub fn new(total: usize, title: &str) -> Self {
        let multi = MultiProgress::new();
        let main_bar = ProgressBar::new(total as u64);

        main_bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{spinner:.green}} {} [{{bar:40.cyan/blue}}] {{pos}}/{{len}} ({{percent}}%) {{msg}}", title))
                .unwrap()
                .progress_chars("█▓▒░ "),
        );

        let main_bar = multi.add(main_bar);

        Self { multi, main_bar }
    }

    /// 개별 작업 스피너 추가
    pub fn add_task(&self, name: &str) -> ProgressBar {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template(&format!("  {{spinner:.cyan}} {}: {{msg}}", name))
                .unwrap(),
        );
        spinner.enable_steady_tick(Duration::from_millis(80));
        self.multi.add(spinner)
    }

    /// 작업 성공 완료
    pub fn finish_task(&self, spinner: &ProgressBar, duration_ms: u128) {
        spinner.finish_with_message(format!("{} ({}ms)", "완료".green(), duration_ms));
        self.main_bar.inc(1);
    }

    /// 작업 실패
    pub fn fail_task(&self, spinner: &ProgressBar, error: &str) {
        spinner.finish_with_message(format!("{} {}", "실패".red(), error.dimmed()));
        self.main_bar.inc(1);
    }

    /// 전체 완료
    pub fn finish(&self, success: usize, total: usize) {
        self.main_bar.finish_with_message(
            format!(
                "{} 완료 (성공: {}, 실패: {})",
                "✓".green().bold(),
                success.to_string().green(),
                (total - success).to_string().red()
            )
        );
    }
}

/// 간단한 스피너 헬퍼 함수
pub fn with_spinner<F, T>(message: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let spinner = create_spinner(message);
    let result = f();
    spinner.finish_and_clear();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_spinner() {
        let spinner = create_spinner("테스트 중...");
        assert!(!spinner.is_finished());
        spinner.finish_and_clear();
    }

    #[test]
    fn test_create_progress_bar() {
        let pb = create_progress_bar(10);
        assert_eq!(pb.length(), Some(10));
        pb.finish_and_clear();
    }

    #[test]
    fn test_multi_progress_display() {
        let display = MultiProgressDisplay::new();
        let spinner = display.add_spinner("테스트");
        display.finish_spinner(&spinner, "완료");
    }
}
