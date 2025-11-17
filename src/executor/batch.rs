use crate::error::{AskAiError, Result};
use crate::executor::planner::{ExecutionPlan, Task};
use crate::executor::runner::CommandRunner;
use colored::*;
use std::time::Instant;

/// 작업 실행 결과
#[derive(Debug, Clone)]
pub struct TaskResult {
    /// 작업 ID
    pub task_id: usize,
    /// 작업 설명
    pub description: String,
    /// 성공 여부
    pub success: bool,
    /// 출력 (성공 시)
    pub output: Option<String>,
    /// 에러 메시지 (실패 시)
    pub error: Option<String>,
    /// 실행 시간 (밀리초)
    pub duration_ms: u128,
}

impl TaskResult {
    pub fn success(task: &Task, output: String, duration_ms: u128) -> Self {
        Self {
            task_id: task.id,
            description: task.description.clone(),
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
        }
    }

    pub fn failure(task: &Task, error: String, duration_ms: u128) -> Self {
        Self {
            task_id: task.id,
            description: task.description.clone(),
            success: false,
            output: None,
            error: Some(error),
            duration_ms,
        }
    }
}

/// 배치 실행 결과
#[derive(Debug)]
pub struct BatchResult {
    /// 총 작업 수
    pub total: usize,
    /// 성공한 작업 수
    pub success_count: usize,
    /// 실패한 작업 수
    pub failure_count: usize,
    /// 개별 작업 결과
    pub task_results: Vec<TaskResult>,
    /// 전체 실행 시간 (밀리초)
    pub total_duration_ms: u128,
}

impl BatchResult {
    /// 모든 작업이 성공했는지 확인
    pub fn all_succeeded(&self) -> bool {
        self.failure_count == 0
    }

    /// 실패한 작업들의 설명 반환
    pub fn failed_tasks(&self) -> Vec<&TaskResult> {
        self.task_results.iter().filter(|r| !r.success).collect()
    }

    /// 성공률 계산
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.success_count as f64 / self.total as f64) * 100.0
    }
}

/// 배치 실행기
pub struct BatchExecutor {
    /// 최대 병렬 실행 개수
    max_parallel: usize,
    /// 실행기
    runner: CommandRunner,
}

impl BatchExecutor {
    pub fn new(max_parallel: usize) -> Self {
        Self {
            max_parallel,
            runner: CommandRunner::new(),
        }
    }

    /// 실행 계획에 따라 배치 실행
    pub async fn execute(&self, plan: &ExecutionPlan) -> BatchResult {
        let start_time = Instant::now();
        let total = plan.task_count();

        println!(
            "{} {} 개의 작업 실행 중...",
            "🚀".cyan(),
            total.to_string().bold()
        );

        let mut all_results = Vec::new();

        if plan.can_parallelize && self.max_parallel > 1 {
            // 병렬 실행
            let groups = plan.get_parallel_groups();

            for (group_idx, group) in groups.iter().enumerate() {
                println!(
                    "\n{} 그룹 {} ({} 작업)",
                    "📦".cyan(),
                    group_idx + 1,
                    group.len()
                );

                let group_results = self.execute_parallel(group).await;
                all_results.extend(group_results);
            }
        } else {
            // 순차 실행
            for task in &plan.tasks {
                let result = self.execute_task(task).await;
                all_results.push(result);
            }
        }

        let success_count = all_results.iter().filter(|r| r.success).count();
        let failure_count = all_results.iter().filter(|r| !r.success).count();

        let total_duration = start_time.elapsed().as_millis();

        BatchResult {
            total,
            success_count,
            failure_count,
            task_results: all_results,
            total_duration_ms: total_duration,
        }
    }

    /// 단일 작업 실행
    async fn execute_task(&self, task: &Task) -> TaskResult {
        let start_time = Instant::now();

        println!(
            "  {} {}",
            "▶️".cyan(),
            task.description.dimmed()
        );

        // working_dir이 있으면 cd를 포함한 명령어 생성
        let command = if let Some(dir) = &task.working_dir {
            format!("cd {} && {}", dir, task.command)
        } else {
            task.command.clone()
        };

        match self.runner.execute(&command).await {
            Ok(output) => {
                let duration = start_time.elapsed().as_millis();
                println!(
                    "    {} {} ({}ms)",
                    "✓".green(),
                    task.description,
                    duration
                );
                TaskResult::success(task, output, duration)
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis();
                println!(
                    "    {} {} - {}",
                    "✗".red(),
                    task.description,
                    e.to_string().red()
                );
                TaskResult::failure(task, e.to_string(), duration)
            }
        }
    }

    /// 여러 작업 병렬 실행
    async fn execute_parallel(&self, tasks: &[&Task]) -> Vec<TaskResult> {
        // 간단한 병렬 실행 (실제로는 순차 실행, Phase 3에서 tokio::spawn 사용)
        let mut results = Vec::new();

        for task in tasks {
            let result = self.execute_task(task).await;
            results.push(result);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::planner::ExecutionPlanner;

    #[test]
    fn test_task_result_success() {
        let task = Task::new(0, "ls".to_string());
        let result = TaskResult::success(&task, "file1\nfile2".to_string(), 100);

        assert!(result.success);
        assert!(result.output.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_task_result_failure() {
        let task = Task::new(0, "invalid".to_string());
        let result = TaskResult::failure(&task, "command not found".to_string(), 50);

        assert!(!result.success);
        assert!(result.output.is_none());
        assert!(result.error.is_some());
        assert_eq!(result.duration_ms, 50);
    }

    #[test]
    fn test_batch_result_all_succeeded() {
        let task = Task::new(0, "test".to_string());
        let results = vec![
            TaskResult::success(&task, "ok".to_string(), 100),
            TaskResult::success(&task, "ok".to_string(), 200),
        ];

        let batch = BatchResult {
            total: 2,
            success_count: 2,
            failure_count: 0,
            task_results: results,
            total_duration_ms: 300,
        };

        assert!(batch.all_succeeded());
        assert_eq!(batch.success_rate(), 100.0);
    }

    #[test]
    fn test_batch_result_partial_failure() {
        let task = Task::new(0, "test".to_string());
        let results = vec![
            TaskResult::success(&task, "ok".to_string(), 100),
            TaskResult::failure(&task, "error".to_string(), 50),
        ];

        let batch = BatchResult {
            total: 2,
            success_count: 1,
            failure_count: 1,
            task_results: results,
            total_duration_ms: 150,
        };

        assert!(!batch.all_succeeded());
        assert_eq!(batch.success_rate(), 50.0);
        assert_eq!(batch.failed_tasks().len(), 1);
    }

    #[tokio::test]
    async fn test_batch_executor_single_task() {
        let plan = ExecutionPlanner::create_single("echo 'test'".to_string());
        let executor = BatchExecutor::new(1);

        let result = executor.execute(&plan).await;

        assert_eq!(result.total, 1);
        assert!(result.all_succeeded());
    }

    #[tokio::test]
    async fn test_batch_executor_multiple_tasks() {
        let commands = vec!["echo 'a'".to_string(), "echo 'b'".to_string()];
        let plan = ExecutionPlanner::create_parallel(commands);
        let executor = BatchExecutor::new(2);

        let result = executor.execute(&plan).await;

        assert_eq!(result.total, 2);
    }
}
