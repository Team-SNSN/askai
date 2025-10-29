use clap::Parser;
use colored::*;

mod cli;
mod error;
mod ai;
mod executor;
mod ui;

use cli::Cli;
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.debug {
        println!("{} {:?}", "DEBUG:".yellow(), cli);
    }

    println!("{} {}", "🔍 프롬프트:".cyan(), cli.prompt_text());

    // TODO: AI 명령어 생성
    // TODO: 안전성 검사
    // TODO: 실행

    Ok(())
}
