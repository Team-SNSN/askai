use std::collections::{HashMap, HashSet};

/// 실행 작업 단위
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Task {
    /// 작업 ID
    pub id: usize,
    /// 실행할 명령어
    pub command: String,
    /// 작업 디렉토리 (None이면 현재 디렉토리)
    pub working_dir: Option<String>,
    /// 작업 설명
    pub description: String,
}

impl Task {
    pub fn new(id: usize, command: String) -> Self {
        Self {
            id,
            command: command.clone(),
            working_dir: None,
            description: format!("Task {}", id),
        }
    }

    pub fn with_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }
}

/// 실행 계획
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// 실행할 작업 목록
    pub tasks: Vec<Task>,
    /// 병렬 실행 가능 여부
    pub can_parallelize: bool,
    /// 작업 간 의존성 (task_id -> 의존하는 task_id들)
    pub dependencies: HashMap<usize, Vec<usize>>,
}

impl ExecutionPlan {
    /// 새로운 실행 계획 생성
    pub fn new(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            can_parallelize: true,
            dependencies: HashMap::new(),
        }
    }

    /// 작업 추가
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// 의존성 추가 (task_id는 depends_on_ids에 의존)
    pub fn add_dependency(&mut self, task_id: usize, depends_on: usize) {
        self.dependencies
            .entry(task_id)
            .or_insert_with(Vec::new)
            .push(depends_on);
    }

    /// 병렬 실행 불가 설정
    pub fn disable_parallelization(&mut self) {
        self.can_parallelize = false;
    }

    /// 병렬 실행 가능한 작업 그룹 반환
    ///
    /// 의존성을 고려하여 동시에 실행할 수 있는 작업들을 그룹으로 반환
    pub fn get_parallel_groups(&self) -> Vec<Vec<&Task>> {
        if !self.can_parallelize || self.tasks.is_empty() {
            return vec![self.tasks.iter().collect()];
        }

        let mut groups: Vec<Vec<&Task>> = Vec::new();
        let mut completed: HashSet<usize> = HashSet::new();

        while completed.len() < self.tasks.len() {
            let mut current_group: Vec<&Task> = Vec::new();

            for task in &self.tasks {
                // 이미 완료된 작업은 스킵
                if completed.contains(&task.id) {
                    continue;
                }

                // 의존성이 모두 완료되었는지 확인
                let deps = self.dependencies.get(&task.id);
                let can_run = match deps {
                    None => true, // 의존성 없음
                    Some(dep_ids) => dep_ids.iter().all(|id| completed.contains(id)),
                };

                if can_run {
                    current_group.push(task);
                }
            }

            // 현재 그룹의 작업들을 완료 목록에 추가
            for task in &current_group {
                completed.insert(task.id);
            }

            if !current_group.is_empty() {
                groups.push(current_group);
            } else {
                // 무한 루프 방지 (순환 의존성)
                break;
            }
        }

        groups
    }

    /// 전체 작업 수
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

/// 실행 계획 생성기
pub struct ExecutionPlanner;

impl ExecutionPlanner {
    /// 단일 명령어를 위한 계획 생성
    pub fn create_single(command: String) -> ExecutionPlan {
        let task = Task::new(0, command);
        ExecutionPlan::new(vec![task])
    }

    /// 여러 명령어를 위한 계획 생성 (병렬 실행 가능)
    pub fn create_parallel(commands: Vec<String>) -> ExecutionPlan {
        let tasks: Vec<Task> = commands
            .into_iter()
            .enumerate()
            .map(|(id, cmd)| Task::new(id, cmd))
            .collect();

        ExecutionPlan::new(tasks)
    }

    /// 순차 실행 계획 생성
    pub fn create_sequential(commands: Vec<String>) -> ExecutionPlan {
        let mut plan = Self::create_parallel(commands);
        plan.disable_parallelization();
        plan
    }

    /// 디렉토리별 배치 실행 계획 생성
    pub fn create_batch(dirs: Vec<String>, command: String) -> ExecutionPlan {
        let tasks: Vec<Task> = dirs
            .into_iter()
            .enumerate()
            .map(|(id, dir)| {
                let dir_name = std::path::Path::new(&dir)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&dir);

                Task::new(id, command.clone())
                    .with_dir(dir.clone())
                    .with_description(format!("{} in {}", command, dir_name))
            })
            .collect();

        ExecutionPlan::new(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(0, "ls -la".to_string());
        assert_eq!(task.id, 0);
        assert_eq!(task.command, "ls -la");
        assert_eq!(task.working_dir, None);
    }

    #[test]
    fn test_task_with_dir() {
        let task = Task::new(0, "git status".to_string())
            .with_dir("/home/user/project".to_string());

        assert_eq!(task.working_dir, Some("/home/user/project".to_string()));
    }

    #[test]
    fn test_execution_plan_add_task() {
        let mut plan = ExecutionPlan::new(vec![]);
        plan.add_task(Task::new(0, "task1".to_string()));
        plan.add_task(Task::new(1, "task2".to_string()));

        assert_eq!(plan.task_count(), 2);
    }

    #[test]
    fn test_parallel_groups_no_dependencies() {
        let tasks = vec![
            Task::new(0, "task0".to_string()),
            Task::new(1, "task1".to_string()),
            Task::new(2, "task2".to_string()),
        ];

        let plan = ExecutionPlan::new(tasks);
        let groups = plan.get_parallel_groups();

        // 의존성이 없으면 모두 첫 번째 그룹
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }

    #[test]
    fn test_parallel_groups_with_dependencies() {
        let tasks = vec![
            Task::new(0, "task0".to_string()),
            Task::new(1, "task1".to_string()),
            Task::new(2, "task2".to_string()),
        ];

        let mut plan = ExecutionPlan::new(tasks);

        // task1은 task0에 의존, task2는 task1에 의존
        plan.add_dependency(1, 0);
        plan.add_dependency(2, 1);

        let groups = plan.get_parallel_groups();

        // 3개의 순차 그룹이어야 함
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].len(), 1); // task0
        assert_eq!(groups[1].len(), 1); // task1
        assert_eq!(groups[2].len(), 1); // task2
    }

    #[test]
    fn test_create_single() {
        let plan = ExecutionPlanner::create_single("ls -la".to_string());

        assert_eq!(plan.task_count(), 1);
        assert_eq!(plan.tasks[0].command, "ls -la");
    }

    #[test]
    fn test_create_parallel() {
        let commands = vec![
            "git status".to_string(),
            "npm test".to_string(),
            "cargo build".to_string(),
        ];

        let plan = ExecutionPlanner::create_parallel(commands);

        assert_eq!(plan.task_count(), 3);
        assert!(plan.can_parallelize);
    }

    #[test]
    fn test_create_sequential() {
        let commands = vec!["step1".to_string(), "step2".to_string()];
        let plan = ExecutionPlanner::create_sequential(commands);

        assert_eq!(plan.task_count(), 2);
        assert!(!plan.can_parallelize);
    }

    #[test]
    fn test_create_batch() {
        let dirs = vec!["/project1".to_string(), "/project2".to_string()];
        let plan = ExecutionPlanner::create_batch(dirs, "git pull".to_string());

        assert_eq!(plan.task_count(), 2);
        assert_eq!(plan.tasks[0].command, "git pull");
        assert_eq!(plan.tasks[0].working_dir, Some("/project1".to_string()));
        assert_eq!(plan.tasks[1].working_dir, Some("/project2".to_string()));
    }
}
