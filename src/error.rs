use thiserror::Error;

#[derive(Error, Debug)]
pub enum AskAiError {
    #[error("AI CLI 실행 실패: {0}")]
    AiCliError(String),

    #[error("위험한 명령어 감지: {0}")]
    DangerousCommand(String),

    #[error("명령어 실행 실패: {0}")]
    ExecutionError(String),

    #[error("사용자 취소")]
    UserCancelled,

    #[error("IO 에러: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON 직렬화/역직렬화 에러: {0}")]
    JsonError(#[from] serde_json::Error),

    // Phase 2 추가 에러 타입
    #[error("프로젝트 타입 감지 실패: {0}")]
    ProjectDetectionError(String),

    #[error("배치 실행 부분 실패: {success}/{total} 성공")]
    BatchPartialFailure {
        success: usize,
        total: usize,
        errors: Vec<String>,
    },

    #[error("설정 파일 오류: {0}")]
    ConfigError(String),

    #[error("병렬 실행 오류: {0}")]
    ParallelExecutionError(String),
}

pub type Result<T> = std::result::Result<T, AskAiError>;
