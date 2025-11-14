use clap::Parser;
use colored::*;

mod cli;
mod error;
mod ai;
mod executor;
mod ui;

use cli::Cli;
use error::Result;
use ai::{context, factory::ProviderFactory};
use executor::{CommandValidator, CommandRunner};
use ui::ConfirmPrompt;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.debug {
        println!("{} {:?}", "DEBUG:".yellow(), cli);
    }

    // 1. 프롬프트 출력
    println!("{} {}", "🔍 프롬프트:".cyan(), cli.prompt_text());

    // 2. 컨텍스트 수집
    let ctx = context::get_current_context();
    if cli.debug {
        println!("{} {}", "DEBUG Context:".yellow(), ctx);
    }

    // 3. AI provider 선택 및 명령어 생성
    if cli.debug {
        println!("{} {}", "DEBUG Provider:".yellow(), cli.provider);
    }

    let provider = ProviderFactory::create(&cli.provider)?;

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
        return Ok(());
    } else {
        // --yes 플래그: 명령어 출력만 하고 바로 실행
        println!("\n{}", "📋 생성된 명령어:".cyan().bold());
        println!("  {}", command.green());
        println!("{}", "\n⚡ 자동 승인 모드로 실행합니다...".yellow());
    }

    // 7. 명령어 실행
    let runner = CommandRunner::new();
    runner.execute(&command).await?;

    println!("\n{}", "✅ 완료!".green().bold());

    Ok(())
}
