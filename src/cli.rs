use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "askai")]
#[command(version)]
#[command(about = "AI-powered terminal automation", long_about = None)]
pub struct Cli {
    /// 자연어 프롬프트
    #[arg(required_unless_present_any = ["clear_cache", "prewarm_cache", "daemon_start", "daemon_stop", "daemon_status"])]
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

    /// 캐시 무시하고 항상 AI에 새로 요청
    #[arg(long)]
    pub no_cache: bool,

    /// 캐시 전체 삭제
    #[arg(long)]
    pub clear_cache: bool,

    /// 자주 사용하는 명령어들을 미리 캐싱 (터미널 시작 시 권장)
    #[arg(long)]
    pub prewarm_cache: bool,

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

    /// 데몬 모드: 데몬 서버에 요청 전송 (빠른 응답)
    #[arg(long)]
    pub daemon: bool,

    /// 데몬 서버 시작
    #[arg(long)]
    pub daemon_start: bool,

    /// 데몬 서버 종료
    #[arg(long)]
    pub daemon_stop: bool,

    /// 데몬 서버 상태 확인
    #[arg(long)]
    pub daemon_status: bool,
}

impl Cli {
    pub fn prompt_text(&self) -> String {
        self.prompt.join(" ")
    }
}
