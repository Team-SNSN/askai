use crate::daemon::protocol::{DaemonRequest, DaemonResponse};
use crate::daemon::session::SessionPool;
use crate::error::{AskAiError, Result};
use colored::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;

/// 데몬 서버
pub struct DaemonServer {
    /// Unix socket 경로
    socket_path: PathBuf,
    /// Session pool
    session_pool: Arc<SessionPool>,
    /// 서버 시작 시간
    start_time: Instant,
    /// 실행 중 플래그
    running: Arc<RwLock<bool>>,
}

impl DaemonServer {
    /// 새 데몬 서버 생성
    pub fn new(socket_path: PathBuf) -> Result<Self> {
        let session_pool = SessionPool::new()?;

        Ok(Self {
            socket_path,
            session_pool: Arc::new(session_pool),
            start_time: Instant::now(),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// 기본 socket 경로로 생성
    pub fn default_socket() -> Result<Self> {
        let socket_path = Self::get_socket_path()?;
        Self::new(socket_path)
    }

    /// 기본 socket 경로 가져오기 (~/.askai-daemon.sock)
    pub fn get_socket_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| AskAiError::ConfigError(
            "Could not find home directory".to_string()
        ))?;

        Ok(home.join(".askai-daemon.sock"))
    }

    /// PID 파일 경로 가져오기 (~/.askai-daemon.pid)
    pub fn get_pid_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| AskAiError::ConfigError(
            "Could not find home directory".to_string()
        ))?;

        Ok(home.join(".askai-daemon.pid"))
    }

    /// 데몬 서버 시작
    pub async fn start(&self) -> Result<()> {
        // 기존 socket 파일 삭제
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).map_err(|e| AskAiError::ConfigError(
                format!("Failed to remove existing socket: {}", e)
            ))?;
        }

        // Unix socket listener 생성
        let listener = UnixListener::bind(&self.socket_path).map_err(|e| {
            AskAiError::ConfigError(
                format!("Failed to bind socket: {}", e)
            )
        })?;

        println!(
            "{} 데몬 서버가 시작되었습니다.",
            "✅".green().bold()
        );
        println!("  Socket: {}", self.socket_path.display());

        // PID 파일 작성
        self.write_pid_file()?;

        // 실행 중 플래그 설정
        *self.running.write().await = true;

        // 클라이언트 연결 수락
        loop {
            // 종료 신호 확인
            if !*self.running.read().await {
                break;
            }

            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let session_pool = Arc::clone(&self.session_pool);
                    let start_time = self.start_time;
                    let running = Arc::clone(&self.running);

                    // 각 연결을 별도 태스크로 처리
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, session_pool, start_time, running).await {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }

        println!("{} 데몬 서버가 종료되었습니다.", "👋".cyan());

        // 정리
        self.cleanup()?;

        Ok(())
    }

    /// 클라이언트 연결 처리
    async fn handle_client(
        stream: UnixStream,
        session_pool: Arc<SessionPool>,
        start_time: Instant,
        running: Arc<RwLock<bool>>,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        // 한 줄 읽기 (JSON)
        reader.read_line(&mut line).await.map_err(|e| {
            AskAiError::ConfigError(
                format!("Failed to read request: {}", e)
            )
        })?;

        // 요청 파싱
        let request = DaemonRequest::from_json(&line).map_err(|e| AskAiError::ConfigError(
            format!("Failed to parse request: {}", e)
        ))?;

        // 요청 처리
        let response = match request {
            DaemonRequest::GenerateCommand {
                prompt,
                context,
                provider,
            } => {
                match session_pool.generate_command(&prompt, &context, &provider).await {
                    Ok((command, from_cache)) => DaemonResponse::Success { command, from_cache },
                    Err(e) => DaemonResponse::Error {
                        message: e.to_string(),
                    },
                }
            }
            DaemonRequest::Ping => {
                let uptime = start_time.elapsed().as_secs();
                let session_count = session_pool.provider_count().await;
                DaemonResponse::Pong {
                    uptime_seconds: uptime,
                    session_count,
                }
            }
            DaemonRequest::Shutdown => {
                // 종료 신호 설정
                *running.write().await = false;
                DaemonResponse::ShuttingDown
            }
        };

        // 응답 전송
        let response_json = response.to_json().map_err(|e| AskAiError::ConfigError(
            format!("Failed to serialize response: {}", e)
        ))?;

        writer
            .write_all(response_json.as_bytes())
            .await
            .map_err(|e| AskAiError::ConfigError(
                format!("Failed to write response: {}", e)
            ))?;
        writer.write_all(b"\n").await.map_err(|e| {
            AskAiError::ConfigError(
                format!("Failed to write newline: {}", e)
            )
        })?;

        Ok(())
    }

    /// PID 파일 작성
    fn write_pid_file(&self) -> Result<()> {
        let pid_path = Self::get_pid_path()?;
        let pid = std::process::id();

        std::fs::write(&pid_path, pid.to_string()).map_err(|e| AskAiError::ConfigError(
            format!("Failed to write PID file: {}", e)
        ))?;

        Ok(())
    }

    /// 정리 (socket 파일, PID 파일 삭제)
    fn cleanup(&self) -> Result<()> {
        // Socket 파일 삭제
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).map_err(|e| AskAiError::ConfigError(
                format!("Failed to remove socket: {}", e)
            ))?;
        }

        // PID 파일 삭제
        let pid_path = Self::get_pid_path()?;
        if pid_path.exists() {
            std::fs::remove_file(&pid_path).map_err(|e| AskAiError::ConfigError(
                format!("Failed to remove PID file: {}", e)
            ))?;
        }

        Ok(())
    }

    /// Provider pre-warming
    pub async fn prewarm_providers(&self, providers: &[&str]) -> Result<()> {
        for provider_name in providers {
            if let Err(e) = self.session_pool.prewarm_provider(provider_name).await {
                eprintln!(
                    "{} Provider '{}' pre-warming failed: {}",
                    "⚠️".yellow(),
                    provider_name,
                    e
                );
            } else {
                println!(
                    "  {} Provider '{}' pre-warmed",
                    "✓".green(),
                    provider_name
                );
            }
        }
        Ok(())
    }

    /// 캐시 pre-warming
    pub async fn prewarm_cache(&self, context: &str) -> usize {
        self.session_pool.prewarm_cache(context).await
    }
}

/// 데몬 클라이언트 (서버에 요청 보내기)
pub struct DaemonClient {
    socket_path: PathBuf,
}

impl DaemonClient {
    /// 새 클라이언트 생성
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// 기본 클라이언트 생성
    pub fn default_client() -> Result<Self> {
        let socket_path = DaemonServer::get_socket_path()?;
        Ok(Self::new(socket_path))
    }

    /// 데몬이 실행 중인지 확인
    pub async fn is_running() -> bool {
        let socket_path = match DaemonServer::get_socket_path() {
            Ok(path) => path,
            Err(_) => return false,
        };

        socket_path.exists()
    }

    /// 요청 전송
    pub async fn send_request(&self, request: &DaemonRequest) -> Result<DaemonResponse> {
        // Socket 연결
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| AskAiError::ConfigError(
                format!("Failed to connect to daemon: {}", e)
            ))?;

        // 요청 전송
        let request_json = request.to_json().map_err(|e| AskAiError::ConfigError(
            format!("Failed to serialize request: {}", e)
        ))?;

        stream
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| AskAiError::ConfigError(
                format!("Failed to write request: {}", e)
            ))?;
        stream.write_all(b"\n").await.map_err(|e| {
            AskAiError::ConfigError(
                format!("Failed to write newline: {}", e)
            )
        })?;

        // 응답 읽기
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.map_err(|e| {
            AskAiError::ConfigError(
                format!("Failed to read response: {}", e)
            )
        })?;

        // 응답 파싱
        let response =
            DaemonResponse::from_json(&line).map_err(|e| AskAiError::ConfigError(
                format!("Failed to parse response: {}", e)
            ))?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path() {
        let path = DaemonServer::get_socket_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains(".askai-daemon.sock"));
    }

    #[test]
    fn test_pid_path() {
        let path = DaemonServer::get_pid_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains(".askai-daemon.pid"));
    }
}
