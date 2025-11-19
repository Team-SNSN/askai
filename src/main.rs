use clap::Parser;
use colored::*;

mod cli;
mod error;
mod ai;
mod executor;
mod ui;
mod config;
mod context;
mod cache;
mod daemon;

use cli::Cli;
use error::Result;
use ai::{factory::ProviderFactory, history::{CommandHistory, HistoryStore}};
use executor::{CommandValidator, CommandRunner};
use ui::{ConfirmPrompt, create_spinner};
use chrono::Utc;
use config::Config;
use cache::ResponseCache;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// 전역 Response Cache (프로그램 전체에서 재사용)
static RESPONSE_CACHE: Lazy<Mutex<ResponseCache>> = Lazy::new(|| {
    Mutex::new(
        ResponseCache::default_config()
            .expect("Failed to initialize response cache")
    )
});

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // --clear-cache 옵션 처리 (우선 처리)
    if cli.clear_cache {
        let mut cache = RESPONSE_CACHE.lock().unwrap();
        cache.clear()?;
        eprintln!("{} Cache cleared.", "[OK]".green());
        return Ok(());
    }

    // --prewarm-cache 옵션 처리
    if cli.prewarm_cache {
        let ctx = context::get_current_context();
        let mut cache = RESPONSE_CACHE.lock().unwrap();
        let count = cache.prewarm(&ctx);
        cache.save_to_disk()?;
        eprintln!("{} Added {} frequently used commands to cache.", "[OK]".green(), count);
        eprintln!("{} Run this command at terminal startup for faster responses:", "[TIP]".cyan());
        eprintln!("  {}", "echo 'askai --prewarm-cache &' >> ~/.zshrc".yellow());
        return Ok(());
    }

    // --daemon-start 옵션 처리 (데몬 서버 시작)
    if cli.daemon_start {
        return start_daemon().await;
    }

    // --daemon-stop 옵션 처리 (데몬 서버 종료)
    if cli.daemon_stop {
        return stop_daemon().await;
    }

    // --daemon-status 옵션 처리 (데몬 서버 상태 확인)
    if cli.daemon_status {
        return check_daemon_status().await;
    }

    // 설정 파일 로드 (없으면 기본값 사용)
    let config = Config::load().unwrap_or_default();

    // Provider 결정: CLI 옵션 > 설정 파일 > 기본값
    let provider_name = cli.provider.as_deref().unwrap_or(&config.default_provider);

    if cli.debug {
        eprintln!("{} {:?}", "DEBUG:".yellow(), cli);
    }

    // --batch 모드 처리
    if cli.batch {
        return execute_batch_mode(&cli, &config).await;
    }

    // 1. 프롬프트 출력
    if !cli.quiet {
        eprintln!("{} {}", "[?] Prompt:".cyan(), cli.prompt_text());
    }

    // 2. 컨텍스트 수집 (RAG: 관련 히스토리 포함)
    let ctx = context::get_context_with_history(&cli.prompt_text());
    if cli.debug && !cli.quiet {
        eprintln!("{} {}", "DEBUG Context:".yellow(), ctx);
    }

    // 3. AI provider 선택 및 명령어 생성
    if cli.debug && !cli.quiet {
        eprintln!("{} {}", "DEBUG Provider:".yellow(), provider_name);
    }

    // 3-1. Daemon 모드 또는 일반 모드로 명령어 생성
    let command = if cli.daemon {
        // Daemon 모드: 데몬 서버에 요청
        use daemon::protocol::{DaemonRequest, DaemonResponse};
        use daemon::server::DaemonClient;

        if !DaemonClient::is_running().await {
            if !cli.quiet {
                eprintln!("{} Daemon server is not running.", "[!]".yellow());
                eprintln!("{} To run in daemon mode, first start the daemon server:", "[TIP]".cyan());
                eprintln!("  {}", "askai --daemon-start".yellow());
                eprintln!("\n{} Continuing in normal mode...", "[i]".cyan());
            }
            String::new() // 일반 모드로 fallback
        } else {
            let client = DaemonClient::default_client()?;
            let request = DaemonRequest::GenerateCommand {
                prompt: cli.prompt_text(),
                context: ctx.clone(),
                provider: provider_name.to_string(),
            };

            match client.send_request(&request).await {
                Ok(DaemonResponse::Success { command, from_cache }) => {
                    if !cli.quiet {
                        if from_cache {
                            eprintln!("{} Instant response from daemon cache!", "[*]".green().bold());
                        } else {
                            eprintln!("{} Daemon generated the command.", "[AI]".cyan());
                        }
                    }
                    // 명령어를 얻었으므로 이 블록의 결과로 사용
                    command
                }
                Ok(DaemonResponse::Error { message }) => {
                    if !cli.quiet {
                        eprintln!("{} Daemon error: {}", "[X]".red(), message);
                        eprintln!("{} Continuing in normal mode...", "[i]".cyan());
                    }
                    // 일반 모드로 fallback
                    String::new() // 임시값, 아래에서 덮어씀
                }
                Err(e) => {
                    if !cli.quiet {
                        eprintln!("{} Daemon communication error: {}", "[X]".red(), e);
                        eprintln!("{} Continuing in normal mode...", "[i]".cyan());
                    }
                    String::new() // fallback
                }
                _ => {
                    if !cli.quiet {
                        eprintln!("{} Unexpected response", "[!]".yellow());
                    }
                    String::new()
                }
            }
        }
    } else {
        String::new() // 일반 모드
    };

    // Daemon 모드에서 명령어를 얻지 못했거나 일반 모드인 경우
    let command = if !cli.daemon || command.is_empty() {
        let provider = ProviderFactory::create(provider_name)?;

        // 캐시 확인 (--no-cache 플래그가 없으면)
        if !cli.no_cache {
            let mut cache = RESPONSE_CACHE.lock().unwrap();
            if let Some(cached_command) = cache.get(&cli.prompt_text(), &ctx) {
                if !cli.quiet {
                    eprintln!("{} Instant response from cache! (Skipping AI call)", "[*]".green().bold());
                }
                cached_command
            } else {
                drop(cache); // lock 해제

                let spinner = if !cli.quiet {
                    create_spinner(&format!(
                        "Generating command with {} provider...",
                        provider.name()
                    ))
                } else {
                    create_spinner("")  // quiet 모드에서는 빈 spinner
                };

                let generated_command = provider.generate_command(&cli.prompt_text(), &ctx).await?;

                spinner.finish_and_clear();
                if !cli.quiet {
                    eprintln!("{} Command generation complete!", "[v]".green());
                }

                // 캐시에 저장
                let mut cache = RESPONSE_CACHE.lock().unwrap();
                cache.set(&cli.prompt_text(), &ctx, generated_command.clone());

                generated_command
            }
        } else {
            // --no-cache: 캐시 무시하고 바로 생성
            let spinner = if !cli.quiet {
                create_spinner(&format!(
                    "{} provider로 명령어 생성 중...",
                    provider.name()
                ))
            } else {
                create_spinner("")  // quiet 모드에서는 빈 spinner
            };

            let generated_command = provider.generate_command(&cli.prompt_text(), &ctx).await?;

            spinner.finish_and_clear();
            if !cli.quiet {
                eprintln!("{} 명령어 생성 완료!", "[v]".green());
            }

            generated_command
        }
    } else {
        command // daemon에서 얻은 명령어 사용
    };

    // 4. 안전성 검사
    let validator = CommandValidator::new();
    let danger_level = validator.validate(&command)?;

    // 5. 사용자 확인 (--yes 플래그가 없으면)
    if !cli.yes && !cli.quiet {
        let prompt = ConfirmPrompt::new();
        if !prompt.confirm_execution(&command, danger_level)? {
            eprintln!("{}", "[X] User cancelled.".yellow());
            std::process::exit(1);  // 사용자 취소는 exit code 1로 종료
        }
    } else if !cli.yes && cli.quiet {
        // quiet 모드에서는 자동으로 yes로 처리 (위험할 수 있음)
        // 또는 에러를 반환할 수도 있음
        // 여기서는 안전을 위해 에러 반환
        eprintln!("{}", "[X] --yes flag required in quiet mode.".red());
        std::process::exit(1);  // 에러는 exit code 1로 종료
    }

    // 6. 명령어를 stdout에 출력 (stderr에는 아무것도 출력하지 않음)
    // 사용자는 eval $(askai "프롬프트")로 실행하거나 shell function으로 감싸서 사용
    println!("{}", command);

    // 7. 히스토리 저장 (RAG)
    let store = HistoryStore::new();
    let history_entry = CommandHistory {
        prompt: cli.prompt_text(),
        command: command.clone(),
        timestamp: Utc::now(),
        executed: true,
        provider: provider_name.to_string(),
    };

    if let Err(e) = store.add(history_entry) {
        if cli.debug {
            eprintln!("{} Failed to save history: {}", "DEBUG:".yellow(), e);
        }
        // 히스토리 저장 실패는 치명적이지 않으므로 계속 진행
    }

    // 캐시를 디스크에 저장
    if let Err(e) = RESPONSE_CACHE.lock().unwrap().save_to_disk() {
        if cli.debug {
            eprintln!("{} Failed to save cache: {}", "DEBUG:".yellow(), e);
        }
    }

    Ok(())
}

/// 배치 모드 실행: 여러 프로젝트에 대해 같은 명령어를 병렬 실행
async fn execute_batch_mode(cli: &Cli, config: &Config) -> Result<()> {
    use context::{ProjectScanner, ScanResult};
    use executor::{planner::{ExecutionPlan, Task}, batch::BatchExecutor};
    use std::env;

    eprintln!("{} Running in batch mode...", "[>>]".cyan().bold());

    // 1. 프로젝트 탐색
    let scanner = if let Some(max_depth) = cli.max_parallel {
        ProjectScanner::new(max_depth)
    } else {
        ProjectScanner::default()
    };

    let current_dir = env::current_dir()?;
    let scan_result: ScanResult = scanner.scan(&current_dir);

    if scan_result.projects.is_empty() {
        eprintln!("{} No projects found.", "[X]".red());
        return Ok(());
    }

    eprintln!(
        "{} Found {} projects.",
        "[PKG]".cyan(),
        scan_result.projects.len().to_string().bold()
    );

    for (idx, project) in scan_result.projects.iter().enumerate() {
        eprintln!(
            "  {}. {} ({})",
            idx + 1,
            project.root_dir.display().to_string().dimmed(),
            project.primary_type().as_str().yellow()
        );
    }

    // 2. Provider 선택
    let provider_name = cli.provider.as_deref().unwrap_or(&config.default_provider);
    let provider = ProviderFactory::create(provider_name)?;

    eprintln!(
        "\n{} Generating commands for each project using {} provider...",
        "[AI]".cyan(),
        provider.name()
    );

    // 3. 각 프로젝트에 대해 명령어 생성 (캐시 활용)
    let mut tasks = Vec::new();

    for (idx, project) in scan_result.projects.iter().enumerate() {
        let project_context = project.to_context_string();

        // 캐시 확인
        let command = if !cli.no_cache {
            let mut cache = RESPONSE_CACHE.lock().unwrap();
            if let Some(cached_command) = cache.get(&cli.prompt_text(), &project_context) {
                eprintln!(
                    "  {} {} - [*] Cache hit",
                    "[v]".green(),
                    project.root_dir.file_name().unwrap().to_str().unwrap()
                );
                cached_command
            } else {
                drop(cache);

                let generated_command = provider
                    .generate_command(&cli.prompt_text(), &project_context)
                    .await?;

                // 캐시 저장
                let mut cache = RESPONSE_CACHE.lock().unwrap();
                cache.set(&cli.prompt_text(), &project_context, generated_command.clone());

                eprintln!(
                    "  {} {} - {}",
                    "[v]".green(),
                    project.root_dir.file_name().unwrap().to_str().unwrap(),
                    generated_command.dimmed()
                );

                generated_command
            }
        } else {
            let generated_command = provider
                .generate_command(&cli.prompt_text(), &project_context)
                .await?;

            eprintln!(
                "  {} {} - {}",
                "[v]".green(),
                project.root_dir.file_name().unwrap().to_str().unwrap(),
                generated_command.dimmed()
            );

            generated_command
        };

        // Task 생성
        let task = Task::new(idx, command)
            .with_dir(project.root_dir.display().to_string())
            .with_description(format!(
                "{}: {}",
                project.root_dir.file_name().unwrap().to_str().unwrap(),
                cli.prompt_text()
            ));

        tasks.push(task);
    }

    // 4. 실행 계획 생성
    let mut plan = ExecutionPlan::new(tasks);
    plan.can_parallelize = true;

    // 5. 사용자 확인 (--yes 플래그가 없으면)
    if !cli.yes && !cli.dry_run {
        eprintln!("\n{} Execute the following tasks?", "[?]".cyan());
        eprintln!("  - {} projects", plan.task_count());
        eprintln!("  - Parallel execution: {}", if plan.can_parallelize { "Yes" } else { "No" });

        let prompt = ConfirmPrompt::new();
        // 간단히 첫 번째 명령어로 확인
        if !plan.tasks.is_empty() {
            if !prompt.confirm_execution(&plan.tasks[0].command, executor::DangerLevel::Low)? {
                eprintln!("{}", "[X] User cancelled.".yellow());
                std::process::exit(1);  // 사용자 취소는 exit code 1로 종료
            }
        }
    } else if cli.dry_run {
        eprintln!("\n{} Output commands only (will not execute).", "[i]".cyan());
        return Ok(());
    }

    // 6. 병렬 실행
    let max_parallel = cli.max_parallel.unwrap_or(4);
    let executor = BatchExecutor::new(max_parallel);

    eprintln!("\n{} Starting parallel execution...", "[*]".cyan().bold());
    let batch_result = executor.execute(&plan).await;

    // 7. 결과 출력
    eprintln!("\n{} Batch execution complete!", "[OK]".green().bold());
    eprintln!("  - Total tasks: {}", batch_result.total);
    eprintln!("  - Success: {}", batch_result.success_count.to_string().green());
    eprintln!("  - Failed: {}", batch_result.failure_count.to_string().red());
    eprintln!(
        "  - Success rate: {:.1}%",
        batch_result.success_rate()
    );
    eprintln!("  - Execution time: {}ms", batch_result.total_duration_ms);

    if !batch_result.failed_tasks().is_empty() {
        eprintln!("\n{} Failed tasks:", "[X]".red());
        for failed in batch_result.failed_tasks() {
            eprintln!(
                "  - {}: {}",
                failed.description,
                failed.error.as_ref().unwrap().red()
            );
        }
    }

    // 8. 캐시 저장
    if let Err(e) = RESPONSE_CACHE.lock().unwrap().save_to_disk() {
        if cli.debug {
            eprintln!("{} Failed to save cache: {}", "DEBUG:".yellow(), e);
        }
    }

    Ok(())
}

/// 데몬 서버 시작
async fn start_daemon() -> Result<()> {
    use daemon::server::DaemonServer;

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
async fn stop_daemon() -> Result<()> {
    use daemon::protocol::DaemonRequest;
    use daemon::server::DaemonClient;

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
async fn check_daemon_status() -> Result<()> {
    use daemon::protocol::DaemonRequest;
    use daemon::protocol::DaemonResponse;
    use daemon::server::DaemonClient;

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
