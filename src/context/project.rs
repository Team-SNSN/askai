use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 프로젝트 타입
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Git 저장소
    Git,
    /// Rust 프로젝트 (Cargo.toml)
    Rust,
    /// Node.js 프로젝트 (package.json)
    NodeJs,
    /// Python 프로젝트 (requirements.txt, pyproject.toml 등)
    Python,
    /// Go 프로젝트 (go.mod)
    Go,
    /// Java 프로젝트 (pom.xml, build.gradle)
    Java,
    /// 알 수 없음
    Unknown,
}

impl ProjectType {
    /// ProjectType을 문자열로 변환
    pub fn as_str(&self) -> &str {
        match self {
            ProjectType::Git => "git",
            ProjectType::Rust => "rust",
            ProjectType::NodeJs => "nodejs",
            ProjectType::Python => "python",
            ProjectType::Go => "go",
            ProjectType::Java => "java",
            ProjectType::Unknown => "unknown",
        }
    }

    /// 문자열에서 ProjectType 파싱
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "git" => ProjectType::Git,
            "rust" | "cargo" => ProjectType::Rust,
            "nodejs" | "node" | "npm" | "yarn" => ProjectType::NodeJs,
            "python" | "py" | "pip" => ProjectType::Python,
            "go" | "golang" => ProjectType::Go,
            "java" | "maven" | "gradle" => ProjectType::Java,
            _ => ProjectType::Unknown,
        }
    }
}

/// 프로젝트 정보
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// 프로젝트 루트 디렉토리
    pub root_dir: PathBuf,

    /// 프로젝트 타입들 (복수 가능, 예: Git + Rust)
    pub types: Vec<ProjectType>,

    /// 프로젝트 이름 (디렉토리 이름)
    pub name: String,

    /// Git 브랜치 (Git 프로젝트인 경우)
    pub git_branch: Option<String>,

    /// 추가 메타데이터 (package.json의 version 등)
    pub metadata: std::collections::HashMap<String, String>,
}

impl ProjectInfo {
    /// 새로운 ProjectInfo 생성
    pub fn new(root_dir: PathBuf) -> Self {
        let name = root_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            root_dir,
            types: vec![ProjectType::Unknown],
            name,
            git_branch: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 특정 프로젝트 타입을 포함하는지 확인
    pub fn has_type(&self, project_type: &ProjectType) -> bool {
        self.types.contains(project_type)
    }

    /// 주요 프로젝트 타입 반환 (Git 제외)
    pub fn primary_type(&self) -> &ProjectType {
        self.types
            .iter()
            .find(|t| **t != ProjectType::Git && **t != ProjectType::Unknown)
            .unwrap_or(&ProjectType::Unknown)
    }

    /// 프로젝트 정보를 컨텍스트 문자열로 변환
    pub fn to_context_string(&self) -> String {
        let mut ctx = format!("Project: {}\n", self.name);
        ctx.push_str(&format!("Project Type: {}\n", self.primary_type().as_str()));

        if let Some(branch) = &self.git_branch {
            ctx.push_str(&format!("Git Branch: {}\n", branch));
        }

        if !self.metadata.is_empty() {
            ctx.push_str("Metadata:\n");
            for (key, value) in &self.metadata {
                ctx.push_str(&format!("  {}: {}\n", key, value));
            }
        }

        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_as_str() {
        assert_eq!(ProjectType::Git.as_str(), "git");
        assert_eq!(ProjectType::Rust.as_str(), "rust");
        assert_eq!(ProjectType::NodeJs.as_str(), "nodejs");
    }

    #[test]
    fn test_project_type_from_str() {
        assert_eq!(ProjectType::from_str("git"), ProjectType::Git);
        assert_eq!(ProjectType::from_str("rust"), ProjectType::Rust);
        assert_eq!(ProjectType::from_str("cargo"), ProjectType::Rust);
        assert_eq!(ProjectType::from_str("npm"), ProjectType::NodeJs);
        assert_eq!(ProjectType::from_str("unknown"), ProjectType::Unknown);
    }

    #[test]
    fn test_project_info_new() {
        let path = PathBuf::from("/home/user/my-project");
        let info = ProjectInfo::new(path.clone());

        assert_eq!(info.name, "my-project");
        assert_eq!(info.root_dir, path);
        assert_eq!(info.types, vec![ProjectType::Unknown]);
    }

    #[test]
    fn test_project_info_has_type() {
        let mut info = ProjectInfo::new(PathBuf::from("/tmp/test"));
        info.types = vec![ProjectType::Git, ProjectType::Rust];

        assert!(info.has_type(&ProjectType::Git));
        assert!(info.has_type(&ProjectType::Rust));
        assert!(!info.has_type(&ProjectType::NodeJs));
    }

    #[test]
    fn test_project_info_primary_type() {
        let mut info = ProjectInfo::new(PathBuf::from("/tmp/test"));
        info.types = vec![ProjectType::Git, ProjectType::Rust];

        assert_eq!(info.primary_type(), &ProjectType::Rust);
    }
}
