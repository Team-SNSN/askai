use crate::context::{ProjectDetector, ProjectInfo, ProjectType};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 프로젝트 스캔 결과
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub projects: Vec<ProjectInfo>,
    pub total_scanned: usize,
}

/// 프로젝트 스캐너 - 디렉토리 트리를 탐색하여 프로젝트 발견
pub struct ProjectScanner {
    max_depth: usize,
    follow_links: bool,
}

impl ProjectScanner {
    /// 새로운 스캐너 생성
    ///
    /// # Arguments
    /// * `max_depth` - 탐색할 최대 깊이 (0 = 무제한)
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            follow_links: false,
        }
    }

    /// 기본 설정으로 스캐너 생성 (최대 깊이 3)
    pub fn default() -> Self {
        Self::new(3)
    }

    /// 단일 디렉토리에서 모든 서브프로젝트 찾기
    ///
    /// # Arguments
    /// * `root` - 탐색 시작 디렉토리
    ///
    /// # Returns
    /// * `ScanResult` - 발견된 프로젝트 목록
    pub fn scan(&self, root: &Path) -> ScanResult {
        let mut projects = Vec::new();
        let mut total_scanned = 0;

        let walker = if self.max_depth > 0 {
            WalkDir::new(root)
                .max_depth(self.max_depth)
                .follow_links(self.follow_links)
        } else {
            WalkDir::new(root).follow_links(self.follow_links)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            // 디렉토리만 검사
            if !path.is_dir() {
                continue;
            }

            total_scanned += 1;

            // 프로젝트 감지
            let project_info = ProjectDetector::detect(path);

            // Unknown이 아닌 프로젝트만 수집
            if project_info.primary_type() != &ProjectType::Unknown {
                projects.push(project_info);
            }
        }

        ScanResult {
            projects,
            total_scanned,
        }
    }

    /// 여러 디렉토리에서 프로젝트 찾기
    pub fn scan_multiple(&self, roots: &[PathBuf]) -> ScanResult {
        let mut all_projects = Vec::new();
        let mut total_scanned = 0;

        for root in roots {
            let result = self.scan(root);
            all_projects.extend(result.projects);
            total_scanned += result.total_scanned;
        }

        ScanResult {
            projects: all_projects,
            total_scanned,
        }
    }

    /// 특정 타입의 프로젝트만 필터링
    pub fn scan_by_type(&self, root: &Path, project_type: ProjectType) -> ScanResult {
        let mut result = self.scan(root);

        result.projects.retain(|p| p.primary_type() == &project_type);

        result
    }

    /// 패턴 매칭으로 프로젝트 찾기
    ///
    /// # Arguments
    /// * `pattern` - 예: "project-*", "*/backend"
    pub fn scan_pattern(&self, pattern: &str) -> ScanResult {
        let glob_results = glob::glob(pattern).unwrap_or_else(|_| {
            // Fallback: 현재 디렉토리
            glob::glob(".").unwrap()
        });

        let paths: Vec<PathBuf> = glob_results.filter_map(|p| p.ok()).collect();

        self.scan_multiple(&paths)
    }
}

impl Default for ProjectScanner {
    fn default() -> Self {
        Self::new(3)  // Default max depth of 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_scan_current_directory() {
        let scanner = ProjectScanner::default();
        let current_dir = env::current_dir().unwrap();
        let result = scanner.scan(&current_dir);

        // 현재 디렉토리는 Rust 프로젝트
        assert!(result.total_scanned > 0);
        assert!(!result.projects.is_empty());
    }

    #[test]
    fn test_scanner_max_depth() {
        let scanner = ProjectScanner::new(1); // 깊이 1만
        let current_dir = env::current_dir().unwrap();
        let result = scanner.scan(&current_dir);

        // 최소 1개 이상 스캔됨
        assert!(result.total_scanned >= 1);
    }

    #[test]
    fn test_scan_by_type() {
        let scanner = ProjectScanner::default();
        let current_dir = env::current_dir().unwrap();
        let result = scanner.scan_by_type(&current_dir, ProjectType::Rust);

        // 현재 프로젝트는 Rust
        assert!(!result.projects.is_empty());
    }
}
