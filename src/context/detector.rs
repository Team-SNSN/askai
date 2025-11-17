use super::project::{ProjectInfo, ProjectType};
use std::path::Path;

/// 프로젝트 타입 감지기
pub struct ProjectDetector;

impl ProjectDetector {
    /// 디렉토리에서 프로젝트 정보 감지
    ///
    /// # Arguments
    /// * `path` - 분석할 디렉토리 경로
    ///
    /// # Returns
    /// * `ProjectInfo` - 감지된 프로젝트 정보
    pub fn detect(path: &Path) -> ProjectInfo {
        let mut info = ProjectInfo::new(path.to_path_buf());
        let mut detected_types = Vec::new();

        // Git 저장소 확인
        if path.join(".git").exists() {
            detected_types.push(ProjectType::Git);

            // Git 브랜치 감지 (간단한 구현)
            if let Ok(head) = std::fs::read_to_string(path.join(".git/HEAD")) {
                if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
                    info.git_branch = Some(branch.trim().to_string());
                }
            }
        }

        // Rust 프로젝트 확인
        if path.join("Cargo.toml").exists() {
            detected_types.push(ProjectType::Rust);

            // Cargo.toml에서 메타데이터 추출
            if let Ok(content) = std::fs::read_to_string(path.join("Cargo.toml")) {
                // 간단한 파싱 (name, version)
                for line in content.lines() {
                    if let Some(name) = Self::extract_toml_value(line, "name") {
                        info.metadata.insert("package_name".to_string(), name);
                    }
                    if let Some(version) = Self::extract_toml_value(line, "version") {
                        info.metadata.insert("version".to_string(), version);
                    }
                }
            }
        }

        // Node.js 프로젝트 확인
        if path.join("package.json").exists() {
            detected_types.push(ProjectType::NodeJs);

            // package.json에서 메타데이터 추출
            if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                        info.metadata.insert("package_name".to_string(), name.to_string());
                    }
                    if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                        info.metadata.insert("version".to_string(), version.to_string());
                    }
                }
            }
        }

        // Python 프로젝트 확인
        if path.join("requirements.txt").exists()
            || path.join("pyproject.toml").exists()
            || path.join("setup.py").exists()
        {
            detected_types.push(ProjectType::Python);
        }

        // Go 프로젝트 확인
        if path.join("go.mod").exists() {
            detected_types.push(ProjectType::Go);
        }

        // Java 프로젝트 확인
        if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            detected_types.push(ProjectType::Java);
        }

        // 감지된 타입이 없으면 Unknown
        if detected_types.is_empty() {
            detected_types.push(ProjectType::Unknown);
        }

        info.types = detected_types;
        info
    }

    /// TOML 파일에서 키=값 추출 (간단한 파서)
    fn extract_toml_value(line: &str, key: &str) -> Option<String> {
        let line = line.trim();
        if line.starts_with(key) && line.contains('=') {
            let value = line.split('=').nth(1)?.trim();
            // 따옴표 제거
            let value = value.trim_matches('"').trim_matches('\'');
            return Some(value.to_string());
        }
        None
    }

    /// 경로가 특정 프로젝트 타입인지 확인
    pub fn is_project_type(path: &Path, project_type: &ProjectType) -> bool {
        let info = Self::detect(path);
        info.has_type(project_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_extract_toml_value() {
        assert_eq!(
            ProjectDetector::extract_toml_value("name = \"askai\"", "name"),
            Some("askai".to_string())
        );

        assert_eq!(
            ProjectDetector::extract_toml_value("version = '0.1.0'", "version"),
            Some("0.1.0".to_string())
        );

        assert_eq!(
            ProjectDetector::extract_toml_value("other = value", "name"),
            None
        );
    }

    #[test]
    fn test_detect_current_project() {
        // 현재 프로젝트 디렉토리 테스트 (askai는 Rust + Git 프로젝트)
        let current_dir = std::env::current_dir().unwrap();
        let info = ProjectDetector::detect(&current_dir);

        // askai 프로젝트는 Rust 프로젝트
        assert!(info.has_type(&ProjectType::Rust));

        // Git 저장소이기도 함
        assert!(info.has_type(&ProjectType::Git));

        // primary type은 Rust여야 함
        assert_eq!(info.primary_type(), &ProjectType::Rust);
    }

    #[test]
    fn test_detect_unknown_project() {
        // 임시 디렉토리는 아무 타입도 없음
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("askai_test_unknown");

        // 디렉토리 생성
        let _ = fs::create_dir_all(&test_dir);

        let info = ProjectDetector::detect(&test_dir);

        // Unknown 타입이어야 함
        assert_eq!(info.types, vec![ProjectType::Unknown]);

        // 정리
        let _ = fs::remove_dir_all(&test_dir);
    }
}
