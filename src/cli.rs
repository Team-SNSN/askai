use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "askai")]
#[command(about = "AI-powered terminal automation", long_about = None)]
pub struct Cli {
    /// 자연어 프롬프트
    #[arg(required = true)]
    pub prompt: Vec<String>,

    /// AI 제공자 선택 (gemini, claude)
    #[arg(short = 'p', long, default_value = "gemini")]
    pub provider: String,

    /// 확인 없이 바로 실행 (위험)
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// 명령어만 출력하고 실행하지 않음
    #[arg(long)]
    pub dry_run: bool,

    /// 디버그 모드
    #[arg(short = 'd', long)]
    pub debug: bool,
}

impl Cli {
    pub fn prompt_text(&self) -> String {
        self.prompt.join(" ")
    }
}
