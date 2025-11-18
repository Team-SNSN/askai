use crate::cache::ResponseCache;
use crate::error::Result;

/// 자주 사용하는 명령어 패턴들을 미리 캐싱
///
/// 이 목록은 사용자가 커스터마이징 가능
pub const COMMON_PROMPTS: &[(&str, &str)] = &[
    ("현재 시간", "date"),
    ("현재 시간 출력", "date"),
    ("git 상태", "git status"),
    ("git 상태 보기", "git status"),
    ("파일 목록", "ls -la"),
    ("파일 목록 보기", "ls -la"),
    ("현재 디렉토리", "pwd"),
    ("git pull", "git pull origin main"),
    ("git push", "git push origin main"),
    ("도커 컨테이너 목록", "docker ps"),
    ("npm 설치", "npm install"),
    ("cargo 빌드", "cargo build"),
    ("테스트 실행", "cargo test"),
];

impl ResponseCache {
    /// 자주 사용하는 명령어들을 미리 캐싱
    ///
    /// # Arguments
    /// * `context` - 실행 컨텍스트 (현재 환경)
    ///
    /// # Returns
    /// * `usize` - 캐싱된 명령어 개수
    pub fn prewarm(&mut self, context: &str) -> usize {
        let mut count = 0;

        for (prompt, command) in COMMON_PROMPTS {
            // 이미 캐시에 있으면 스킵
            if self.get(prompt, context).is_none() {
                self.set(prompt, context, command.to_string());
                count += 1;
            }
        }

        count
    }

    /// 사용자 정의 프롬프트 목록으로 pre-warming
    ///
    /// # Arguments
    /// * `prompts` - (프롬프트, 명령어) 쌍의 목록
    /// * `context` - 실행 컨텍스트
    pub fn prewarm_custom(&mut self, prompts: &[(&str, &str)], context: &str) -> usize {
        let mut count = 0;

        for (prompt, command) in prompts {
            if self.get(prompt, context).is_none() {
                self.set(prompt, context, command.to_string());
                count += 1;
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prewarm() {
        let mut cache = ResponseCache::new(3600, 100).unwrap();
        let context = "OS: macOS";

        let count = cache.prewarm(context);

        assert!(count > 0);
        assert_eq!(cache.get("현재 시간", context), Some("date".to_string()));
        assert_eq!(cache.get("git 상태", context), Some("git status".to_string()));
    }

    #[test]
    fn test_prewarm_skip_existing() {
        let mut cache = ResponseCache::new(3600, 100).unwrap();
        let context = "OS: macOS";

        // 첫 번째 prewarm
        let count1 = cache.prewarm(context);

        // 두 번째 prewarm (이미 있으므로 0개 추가)
        let count2 = cache.prewarm(context);

        assert!(count1 > 0);
        assert_eq!(count2, 0);
    }
}
