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

use cli::Cli;
use error::Result;
use ai::{factory::ProviderFactory, history::{CommandHistory, HistoryStore}};
use executor::{CommandValidator, CommandRunner};
use ui::ConfirmPrompt;
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
        println!("{} 캐시가 삭제되었습니다.", "✅".green());
        return Ok(());
    }

    // --prewarm-cache 옵션 처리
    if cli.prewarm_cache {
        let ctx = context::get_current_context();
        let mut cache = RESPONSE_CACHE.lock().unwrap();
        let count = cache.prewarm(&ctx);
        cache.save_to_disk()?;
        println!("{} {}개의 자주 사용하는 명령어를 캐시에 추가했습니다.", "✅".green(), count);
        println!("{} 터미널 시작 시 이 명령어를 실행하면 더 빠른 응답을 받을 수 있습니다:", "💡".cyan());
        println!("  {}", "echo 'askai --prewarm-cache &' >> ~/.zshrc".yellow());
        return Ok(());
    }

    // 설정 파일 로드 (없으면 기본값 사용)
    let config = Config::load().unwrap_or_default();

    // Provider 결정: CLI 옵션 > 설정 파일 > 기본값
    let provider_name = cli.provider.as_deref().unwrap_or(&config.default_provider);

    if cli.debug {
        println!("{} {:?}", "DEBUG:".yellow(), cli);
    }

    // --batch 모드 처리
    if cli.batch {
        return execute_batch_mode(&cli, &config).await;
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

    // 3-1. 캐시 확인 (--no-cache 플래그가 없으면)
    let command = if !cli.no_cache {
        let mut cache = RESPONSE_CACHE.lock().unwrap();
        if let Some(cached_command) = cache.get(&cli.prompt_text(), &ctx) {
            println!("{} 캐시에서 즉시 응답! (AI 호출 생략)", "⚡".green().bold());
            cached_command
        } else {
            drop(cache); // lock 해제

            println!("{} {} provider를 사용하여 명령어를 생성하는 중...",
                     "🤖".cyan(),
                     provider.name());

            let generated_command = provider.generate_command(&cli.prompt_text(), &ctx).await?;

            // 캐시에 저장
            let mut cache = RESPONSE_CACHE.lock().unwrap();
            cache.set(&cli.prompt_text(), &ctx, generated_command.clone());

            generated_command
        }
    } else {
        // --no-cache: 캐시 무시하고 바로 생성
        println!("{} {} provider를 사용하여 명령어를 생성하는 중...",
                 "🤖".cyan(),
                 provider.name());
        provider.generate_command(&cli.prompt_text(), &ctx).await?
    };

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

        // 캐시를 디스크에 저장 (dry-run도 캐시 활용)
        if let Err(e) = RESPONSE_CACHE.lock().unwrap().save_to_disk() {
            if cli.debug {
                println!("{} 캐시 저장 실패: {}", "DEBUG:".yellow(), e);
            }
        }

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

    // 캐시를 디스크에 저장
    if let Err(e) = RESPONSE_CACHE.lock().unwrap().save_to_disk() {
        if cli.debug {
            println!("{} 캐시 저장 실패: {}", "DEBUG:".yellow(), e);
        }
    }

    Ok(())
}

/// 배치 모드 실행: 여러 프로젝트에 대해 같은 명령어를 병렬 실행
async fn execute_batch_mode(cli: &Cli, config: &Config) -> Result<()> {
    use context::{ProjectScanner, ScanResult};
    use executor::{planner::{ExecutionPlan, Task}, batch::BatchExecutor};
    use std::env;

    println!("{} 배치 모드로 실행합니다...", "🚀".cyan().bold());

    // 1. 프로젝트 탐색
    let scanner = if let Some(max_depth) = cli.max_parallel {
        ProjectScanner::new(max_depth)
    } else {
        ProjectScanner::default()
    };

    let current_dir = env::current_dir()?;
    let scan_result: ScanResult = scanner.scan(&current_dir);

    if scan_result.projects.is_empty() {
        println!("{} 프로젝트를 찾을 수 없습니다.", "❌".red());
        return Ok(());
    }

    println!(
        "{} {}개의 프로젝트를 발견했습니다.",
        "📦".cyan(),
        scan_result.projects.len().to_string().bold()
    );

    for (idx, project) in scan_result.projects.iter().enumerate() {
        println!(
            "  {}. {} ({})",
            idx + 1,
            project.root_dir.display().to_string().dimmed(),
            project.primary_type().as_str().yellow()
        );
    }

    // 2. Provider 선택
    let provider_name = cli.provider.as_deref().unwrap_or(&config.default_provider);
    let provider = ProviderFactory::create(provider_name)?;

    println!(
        "\n{} {} provider로 각 프로젝트에 대한 명령어 생성 중...",
        "🤖".cyan(),
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
                println!(
                    "  {} {} - ⚡ 캐시 히트",
                    "✓".green(),
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

                println!(
                    "  {} {} - {}",
                    "✓".green(),
                    project.root_dir.file_name().unwrap().to_str().unwrap(),
                    generated_command.dimmed()
                );

                generated_command
            }
        } else {
            let generated_command = provider
                .generate_command(&cli.prompt_text(), &project_context)
                .await?;

            println!(
                "  {} {} - {}",
                "✓".green(),
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
        println!("\n{} 다음 작업을 실행하시겠습니까?", "❓".cyan());
        println!("  - {} 개의 프로젝트", plan.task_count());
        println!("  - 병렬 실행: {}", if plan.can_parallelize { "예" } else { "아니오" });

        let prompt = ConfirmPrompt::new();
        // 간단히 첫 번째 명령어로 확인
        if !plan.tasks.is_empty() {
            if !prompt.confirm_execution(&plan.tasks[0].command, executor::DangerLevel::Low)? {
                println!("{}", "❌ 사용자가 취소했습니다.".yellow());
                return Ok(());
            }
        }
    } else if cli.dry_run {
        println!("\n{} 명령어만 출력합니다 (실행하지 않음).", "ℹ️".cyan());
        return Ok(());
    }

    // 6. 병렬 실행
    let max_parallel = cli.max_parallel.unwrap_or(4);
    let executor = BatchExecutor::new(max_parallel);

    println!("\n{} 병렬 실행 시작...", "⚡".cyan().bold());
    let batch_result = executor.execute(&plan).await;

    // 7. 결과 출력
    println!("\n{} 배치 실행 완료!", "✅".green().bold());
    println!("  - 총 작업: {}", batch_result.total);
    println!("  - 성공: {}", batch_result.success_count.to_string().green());
    println!("  - 실패: {}", batch_result.failure_count.to_string().red());
    println!(
        "  - 성공률: {:.1}%",
        batch_result.success_rate()
    );
    println!("  - 실행 시간: {}ms", batch_result.total_duration_ms);

    if !batch_result.failed_tasks().is_empty() {
        println!("\n{} 실패한 작업:", "❌".red());
        for failed in batch_result.failed_tasks() {
            println!(
                "  - {}: {}",
                failed.description,
                failed.error.as_ref().unwrap().red()
            );
        }
    }

    // 8. 캐시 저장
    if let Err(e) = RESPONSE_CACHE.lock().unwrap().save_to_disk() {
        if cli.debug {
            println!("{} 캐시 저장 실패: {}", "DEBUG:".yellow(), e);
        }
    }

    Ok(())
}
