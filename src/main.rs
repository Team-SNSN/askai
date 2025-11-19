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
mod commands;

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
        return commands::start_daemon().await;
    }

    // --daemon-stop 옵션 처리 (데몬 서버 종료)
    if cli.daemon_stop {
        return commands::stop_daemon().await;
    }

    // --daemon-status 옵션 처리 (데몬 서버 상태 확인)
    if cli.daemon_status {
        return commands::check_daemon_status().await;
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
        return commands::execute_batch_mode(&cli, &config, &RESPONSE_CACHE).await;
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
