use serde::{Deserialize, Serialize};

/// 데몬 서버로 전송하는 요청
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonRequest {
    /// 명령어 생성 요청
    GenerateCommand {
        prompt: String,
        context: String,
        provider: String,
    },
    /// 데몬 상태 확인
    Ping,
    /// 데몬 종료
    Shutdown,
}

/// 데몬 서버의 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum DaemonResponse {
    /// 성공
    Success {
        command: String,
        /// 캐시에서 가져왔는지 여부
        from_cache: bool,
    },
    /// Ping 응답
    Pong {
        uptime_seconds: u64,
        session_count: usize,
    },
    /// 에러
    Error { message: String },
    /// 종료 확인
    ShuttingDown,
}

impl DaemonRequest {
    /// JSON으로 직렬화
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// JSON에서 역직렬화
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl DaemonResponse {
    /// JSON으로 직렬화
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// JSON에서 역직렬화
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = DaemonRequest::GenerateCommand {
            prompt: "현재 시간".to_string(),
            context: "test context".to_string(),
            provider: "gemini".to_string(),
        };

        let json = request.to_json().unwrap();
        let deserialized = DaemonRequest::from_json(&json).unwrap();

        match deserialized {
            DaemonRequest::GenerateCommand { prompt, .. } => {
                assert_eq!(prompt, "현재 시간");
            }
            _ => panic!("Wrong request type"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = DaemonResponse::Success {
            command: "date".to_string(),
            from_cache: true,
        };

        let json = response.to_json().unwrap();
        let deserialized = DaemonResponse::from_json(&json).unwrap();

        match deserialized {
            DaemonResponse::Success { command, from_cache } => {
                assert_eq!(command, "date");
                assert!(from_cache);
            }
            _ => panic!("Wrong response type"),
        }
    }
}
