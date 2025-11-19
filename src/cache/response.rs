use crate::error::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// 캐시 엔트리
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// 생성된 명령어
    command: String,
    /// 캐시 생성 시간
    #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: chrono::DateTime<chrono::Utc>,
    /// 캐시 히트 횟수
    hit_count: u32,
}

/// Response Cache for AI-generated commands
///
/// 동일한 프롬프트에 대한 AI 응답을 캐싱하여 재사용합니다.
/// 프롬프트와 컨텍스트를 해싱하여 캐시 키로 사용합니다.
#[derive(Debug)]
pub struct ResponseCache {
    /// 캐시 저장소 (해시 -> 엔트리)
    cache: HashMap<String, CacheEntry>,
    /// Time To Live (캐시 유효 시간)
    ttl: Duration,
    /// 최대 엔트리 수
    max_entries: usize,
    /// 캐시 파일 경로
    cache_file: PathBuf,
}

impl ResponseCache {
    /// 새로운 ResponseCache 인스턴스 생성
    ///
    /// # Arguments
    /// * `ttl_seconds` - 캐시 유효 시간 (초)
    /// * `max_entries` - 최대 엔트리 수
    ///
    /// # Returns
    /// * `Result<ResponseCache>` - 캐시 인스턴스
    pub fn new(ttl_seconds: u64, max_entries: usize) -> Result<Self> {
        let cache_file = Self::get_cache_file_path()?;
        let mut cache = Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
            max_entries,
            cache_file,
        };

        // 디스크에서 캐시 로드 시도
        if let Err(e) = cache.load_from_disk() {
            eprintln!("Warning: Failed to load cache from disk: {}", e);
            // 로드 실패는 치명적이지 않으므로 계속 진행
        }

        Ok(cache)
    }

    /// 기본 설정으로 ResponseCache 생성
    ///
    /// Config 파일에서 설정을 로드하여 사용
    /// - TTL: config.cache_ttl_days (기본: 7일)
    /// - Max entries: config.cache_max_entries (기본: 1000)
    pub fn default_config() -> Result<Self> {
        // Config 로드 시도 (실패하면 기본값 사용)
        let config = crate::config::Config::load().unwrap_or_default();
        let ttl_seconds = config.cache_ttl_days * 86400; // days to seconds
        Self::new(ttl_seconds, config.cache_max_entries)
    }

    /// 캐시 파일 경로 반환
    fn get_cache_file_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            crate::error::AskAiError::ConfigError("Could not find home directory".to_string())
        })?;
        Ok(home.join(".askai-cache.json"))
    }

    /// 캐시 키 생성 (프롬프트 + 컨텍스트 해싱)
    ///
    /// # Arguments
    /// * `prompt` - 사용자 프롬프트
    /// * `context` - 실행 컨텍스트
    ///
    /// # Returns
    /// * `String` - SHA256 해시 (hex 문자열)
    fn cache_key(&self, prompt: &str, context: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        hasher.update(b"|"); // 구분자
        hasher.update(context.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 캐시에서 명령어 조회
    ///
    /// # Arguments
    /// * `prompt` - 사용자 프롬프트
    /// * `context` - 실행 컨텍스트
    ///
    /// # Returns
    /// * `Option<String>` - 캐시된 명령어 (있으면 Some, 없으면 None)
    pub fn get(&mut self, prompt: &str, context: &str) -> Option<String> {
        let key = self.cache_key(prompt, context);

        if let Some(entry) = self.cache.get_mut(&key) {
            // TTL 확인
            let now = chrono::Utc::now();
            let age = now.signed_duration_since(entry.timestamp);

            if age.num_seconds() < self.ttl.as_secs() as i64 {
                // 유효한 캐시
                entry.hit_count += 1;
                return Some(entry.command.clone());
            } else {
                // 만료된 캐시 제거
                self.cache.remove(&key);
            }
        }

        None
    }

    /// 캐시에 명령어 저장
    ///
    /// # Arguments
    /// * `prompt` - 사용자 프롬프트
    /// * `context` - 실행 컨텍스트
    /// * `command` - 생성된 명령어
    pub fn set(&mut self, prompt: &str, context: &str, command: String) {
        // 최대 엔트리 수 확인
        if self.cache.len() >= self.max_entries {
            self.evict_oldest();
        }

        let key = self.cache_key(prompt, context);
        self.cache.insert(
            key,
            CacheEntry {
                command,
                timestamp: chrono::Utc::now(),
                hit_count: 0,
            },
        );
    }

    /// 가장 오래된 캐시 엔트리 제거 (LRU 방식)
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(key, _)| key.clone())
        {
            self.cache.remove(&oldest_key);
        }
    }

    /// 디스크에서 캐시 로드
    pub fn load_from_disk(&mut self) -> Result<()> {
        if !self.cache_file.exists() {
            // 캐시 파일이 없으면 스킵
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.cache_file)?;
        self.cache = serde_json::from_str(&content)?;
        Ok(())
    }

    /// 디스크에 캐시 저장
    pub fn save_to_disk(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.cache)?;
        std::fs::write(&self.cache_file, json)?;
        Ok(())
    }

    /// 캐시 전체 삭제
    pub fn clear(&mut self) -> Result<()> {
        self.cache.clear();

        // 디스크 파일도 삭제
        if self.cache_file.exists() {
            std::fs::remove_file(&self.cache_file)?;
        }

        Ok(())
    }

    /// 캐시 통계 정보
    pub fn stats(&self) -> CacheStats {
        let total_hits: u32 = self.cache.values().map(|e| e.hit_count).sum();
        CacheStats {
            total_entries: self.cache.len(),
            total_hits,
            max_entries: self.max_entries,
            ttl_seconds: self.ttl.as_secs(),
        }
    }
}

/// 캐시 통계 정보
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u32,
    pub max_entries: usize,
    pub ttl_seconds: u64,
}

impl Drop for ResponseCache {
    /// 프로그램 종료 시 자동으로 디스크에 저장
    fn drop(&mut self) {
        if let Err(e) = self.save_to_disk() {
            eprintln!("Warning: Failed to save cache to disk: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let cache = ResponseCache::new(3600, 100).unwrap();
        let key1 = cache.cache_key("test prompt", "test context");
        let key2 = cache.cache_key("test prompt", "test context");
        let key3 = cache.cache_key("different prompt", "test context");

        assert_eq!(key1, key2); // 동일한 입력은 동일한 키
        assert_ne!(key1, key3); // 다른 입력은 다른 키
    }

    #[test]
    fn test_cache_set_and_get() {
        let mut cache = ResponseCache::new(3600, 100).unwrap();

        // 캐시에 저장
        cache.set("list files", "OS: macOS", "ls -la".to_string());

        // 캐시에서 조회
        let result = cache.get("list files", "OS: macOS");
        assert_eq!(result, Some("ls -la".to_string()));

        // 다른 프롬프트는 캐시 미스
        let result = cache.get("different prompt", "OS: macOS");
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_hit_count() {
        let mut cache = ResponseCache::new(3600, 100).unwrap();

        cache.set("test", "context", "command".to_string());

        // 여러 번 조회
        cache.get("test", "context");
        cache.get("test", "context");
        cache.get("test", "context");

        let stats = cache.stats();
        assert_eq!(stats.total_hits, 3);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = ResponseCache::new(3600, 2).unwrap(); // 최대 2개

        cache.set("prompt1", "ctx", "cmd1".to_string());
        cache.set("prompt2", "ctx", "cmd2".to_string());

        // 3번째 추가하면 가장 오래된 것 제거
        std::thread::sleep(std::time::Duration::from_millis(10));
        cache.set("prompt3", "ctx", "cmd3".to_string());

        assert_eq!(cache.cache.len(), 2);
        assert_eq!(cache.get("prompt1", "ctx"), None); // 제거됨
        assert_eq!(cache.get("prompt3", "ctx"), Some("cmd3".to_string()));
    }
}
