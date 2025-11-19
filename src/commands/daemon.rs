use crate::error::Result;
use crate::daemon::server::{DaemonServer, DaemonClient};
use crate::daemon::protocol::{DaemonRequest, DaemonResponse};
use crate::ui::create_spinner;
use crate::context;
use colored::*;

/// 데몬 서버 시작
pub async fn start_daemon() -> Result<()> {
    eprintln!("{} Starting daemon server...\n", "[>>]".cyan().bold());

    let server = DaemonServer::default_socket()?;

    // Provider pre-warming
    let spinner = create_spinner("Pre-warming providers...");
    server.prewarm_providers(&["gemini"]).await?;
    spinner.finish_and_clear();
    eprintln!("{} Provider pre-warming complete", "[v]".green());

    // 캐시 pre-warming
    let spinner = create_spinner("Pre-warming cache...");
    let ctx = context::get_current_context();
    let count = server.prewarm_cache(&ctx).await;
    spinner.finish_and_clear();
    eprintln!("{} Added {} commands to cache.", "[v]".green(), count);

    eprintln!();

    // 서버 실행 (blocking)
    server.start().await?;

    Ok(())
}

/// 데몬 서버 종료
pub async fn stop_daemon() -> Result<()> {
    eprintln!("{} Stopping daemon server...", "[STOP]".yellow());

    let client = DaemonClient::default_client()?;
    let request = DaemonRequest::Shutdown;

    match client.send_request(&request).await {
        Ok(_) => {
            eprintln!("{} Daemon server stopped.", "[OK]".green());
            Ok(())
        }
        Err(e) => {
            eprintln!("{} Failed to stop daemon server: {}", "[X]".red(), e);
            Err(e)
        }
    }
}

/// 데몬 서버 상태 확인
pub async fn check_daemon_status() -> Result<()> {
    if !DaemonClient::is_running().await {
        eprintln!("{} Daemon server is not running.", "[X]".red());
        eprintln!("\n{} To start the daemon server:", "[TIP]".cyan());
        eprintln!("  {}", "askai --daemon-start".yellow());
        return Ok(());
    }

    let client = DaemonClient::default_client()?;
    let request = DaemonRequest::Ping;

    match client.send_request(&request).await {
        Ok(response) => match response {
            DaemonResponse::Pong {
                uptime_seconds,
                session_count,
            } => {
                eprintln!("{} Daemon server is running.", "[OK]".green().bold());
                eprintln!("  [>] Uptime: {} seconds", uptime_seconds);
                eprintln!("  [PKG] Loaded providers: {}", session_count);
                Ok(())
            }
            _ => {
                eprintln!("{} Unexpected response", "[!]".yellow());
                Ok(())
            }
        },
        Err(e) => {
            eprintln!("{} Failed to check daemon status: {}", "[X]".red(), e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_daemon_module_exists() {
        // Placeholder test to establish test infrastructure for daemon commands
        // Integration tests would be more appropriate for daemon functionality
        assert!(true);
    }
}
