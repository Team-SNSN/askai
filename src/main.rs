use clap::Parser;
use colored::*;

mod cli;
mod error;
mod ai;
mod executor;
mod ui;
mod config;
mod context;

use cli::Cli;
use error::Result;
use ai::{factory::ProviderFactory, history::{CommandHistory, HistoryStore}};
use executor::{CommandValidator, CommandRunner};
use ui::ConfirmPrompt;
use chrono::Utc;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 설정 파일 로드 (없으면 기본값 사용)
    let config = Config::load().unwrap_or_default();

    // Provider 결정: CLI 옵션 > 설정 파일 > 기본값
    let provider_name = cli.provider.as_deref().unwrap_or(&config.default_provider);

    if cli.debug {
        println!("{} {:?}", "DEBUG:".yellow(), cli);
    }

    // 1. 프롬프트 출력
    println!("{} {}", "🔍 프롬프트:".cyan(), cli.prompt_text());

    // 2. 컨텍스트 수집 (RAG: 관련 히스토리 포함)
    let ctx = context::get_context_with_history(&cli.prompt_text());
    if cli.debug {
        println!("{} {}", "DEBUG Context:".yellow(), ctx);
    }

    // 3. AI provider 선택 및 명령어 생성
    if cli.debug {
        println!("{} {}", "DEBUG Provider:".yellow(), provider_name);
    }

    let provider = ProviderFactory::create(provider_name)?;

    println!("{} {} provider를 사용하여 명령어를 생성하는 중...",
             "🤖".cyan(),
             provider.name());

    let command = provider.generate_command(&cli.prompt_text(), &ctx).await?;

    // 4. 안전성 검사
    let validator = CommandValidator::new();
    let danger_level = validator.validate(&command)?;

    // 5. 사용자 확인 (--yes 플래그가 없으면)
    if !cli.yes && !cli.dry_run {
        let prompt = ConfirmPrompt::new();
        if !prompt.confirm_execution(&command, danger_level)? {
            println!("{}", "❌ 사용자가 취소했습니다.".yellow());
            return Ok(());
        }
    } else if cli.dry_run {
        // dry-run 모드: 명령어만 출력
        println!("\n{}", "📋 생성된 명령어:".cyan().bold());
        println!("  {}", command.green());
        println!("\n{} 명령어만 출력합니다 (실행하지 않음).", "ℹ️".cyan());

        // dry-run도 히스토리에 저장 (실행하지 않음으로 표시)
        let store = HistoryStore::new();
        let history_entry = CommandHistory {
            prompt: cli.prompt_text(),
            command: command.clone(),
            timestamp: Utc::now(),
            executed: false,
            provider: provider_name.to_string(),
        };
        let _ = store.add(history_entry); // 실패해도 무시

        return Ok(());
    } else {
        // --yes 플래그: 명령어 출력만 하고 바로 실행
        println!("\n{}", "📋 생성된 명령어:".cyan().bold());
        println!("  {}", command.green());
        println!("{}", "\n⚡ 자동 승인 모드로 실행합니다...".yellow());
    }

    // 7. 명령어 실행
    let runner = CommandRunner::new();
    let execution_result = runner.execute(&command).await;

    // 8. 히스토리 저장 (RAG)
    let store = HistoryStore::new();
    let history_entry = CommandHistory {
        prompt: cli.prompt_text(),
        command: command.clone(),
        timestamp: Utc::now(),
        executed: execution_result.is_ok(),
        provider: provider_name.to_string(),
    };

    if let Err(e) = store.add(history_entry) {
        if cli.debug {
            println!("{} 히스토리 저장 실패: {}", "DEBUG:".yellow(), e);
        }
        // 히스토리 저장 실패는 치명적이지 않으므로 계속 진행
    }

    // 실행 결과 확인
    execution_result?;

    println!("\n{}", "✅ 완료!".green().bold());

    Ok(())
}
