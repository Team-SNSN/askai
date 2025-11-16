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
}

pub type Result<T> = std::result::Result<T, AskAiError>;
