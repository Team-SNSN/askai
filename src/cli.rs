use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "askai")]
#[command(version)]
#[command(about = "AI-powered terminal automation", long_about = None)]
pub struct Cli {
    /// 자연어 프롬프트
    #[arg(required = true)]
    pub prompt: Vec<String>,

    /// AI 제공자 선택 (gemini, claude, codex). 미지정시 설정 파일의 default_provider 사용
    #[arg(short = 'p', long)]
    pub provider: Option<String>,

    /// 확인 없이 바로 실행 (위험)
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// 명령어만 출력하고 실행하지 않음
    #[arg(long)]
    pub dry_run: bool,

    /// 디버그 모드
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// 배치 모드: 여러 디렉토리에서 같은 명령어 실행 (Phase 3용)
    #[arg(long)]
    pub batch: bool,

    /// 대상 디렉토리 패턴 (예: "projects/*/") (Phase 3용)
    #[arg(long)]
    pub targets: Option<String>,

    /// 최대 병렬 실행 개수 (Phase 3용)
    #[arg(long)]
    pub max_parallel: Option<usize>,

    /// 프로젝트 타입 필터 (예: "git", "npm", "cargo") (Phase 3용)
    #[arg(long)]
    pub project_type: Option<String>,
}

impl Cli {
    pub fn prompt_text(&self) -> String {
        self.prompt.join(" ")
    }
}
