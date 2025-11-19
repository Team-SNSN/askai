use crate::cli::Cli;
use crate::config::Config;
use crate::error::Result;
use crate::ai::factory::ProviderFactory;
use crate::context::{ProjectScanner, ScanResult, ProjectType};
use crate::executor::{planner::{ExecutionPlan, Task}, batch::BatchExecutor, DangerLevel};
use crate::ui::ConfirmPrompt;
use colored::*;
use std::env;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::cache::ResponseCache;

// RESPONSE_CACHE는 main.rs에서 정의되어 있으므로 외부 참조
extern "Rust" {
    static RESPONSE_CACHE: Lazy<Mutex<ResponseCache>>;
}

/// 배치 모드 실행: 여러 프로젝트에 대해 같은 명령어를 병렬 실행
pub async fn execute_batch_mode(cli: &Cli, config: &Config, response_cache: &Lazy<Mutex<ResponseCache>>) -> Result<()> {
    eprintln!("{} Running in batch mode...", "[>>]".cyan().bold());

    // 1. 프로젝트 탐색
    let scanner = if let Some(max_depth) = cli.max_parallel {
        ProjectScanner::new(max_depth)
    } else {
        ProjectScanner::default()
    };

    let current_dir = env::current_dir()?;
    let mut scan_result: ScanResult = scanner.scan(&current_dir);

    // 프로젝트 타입 필터링 (--project-type 옵션이 있으면)
    if let Some(project_type_str) = &cli.project_type {
        let project_type = ProjectType::from_str(project_type_str);
        let original_count = scan_result.projects.len();

        scan_result.projects.retain(|p| p.has_type(&project_type));

        if scan_result.projects.is_empty() {
            eprintln!("{} No {} projects found (scanned {} total).",
                "[X]".red(),
                project_type.as_str().yellow(),
                original_count
            );
            return Ok(());
        }

        eprintln!(
            "{} Filtered to {} {} projects (from {} total).",
            "[FILTER]".cyan(),
            scan_result.projects.len().to_string().bold(),
            project_type.as_str().yellow(),
            original_count
        );
    }

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
            let mut cache = response_cache.lock().unwrap();
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
                let mut cache = response_cache.lock().unwrap();
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
            if !prompt.confirm_execution(&plan.tasks[0].command, DangerLevel::Low)? {
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
    let executor = BatchExecutor::new(max_parallel).with_dry_run(cli.dry_run);

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
    if let Err(e) = response_cache.lock().unwrap().save_to_disk() {
        if cli.debug {
            eprintln!("{} Failed to save cache: {}", "DEBUG:".yellow(), e);
        }
    }

    Ok(())
}
