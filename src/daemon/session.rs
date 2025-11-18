use crate::ai::factory::ProviderFactory;
use crate::ai::AiProvider;
use crate::cache::ResponseCache;
use crate::error::{AskAiError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// AI Provider session pool
///
/// 각 provider의 인스턴스를 미리 생성해두고 재사용합니다.
/// 실제 프로세스 풀링은 대부분의 AI CLI가 지원하지 않으므로,
/// 주로 Provider 인스턴스와 캐시를 메모리에 유지하는 역할을 합니다.
pub struct SessionPool {
    /// Provider 인스턴스 맵 (provider_name -> Provider)
    providers: Arc<RwLock<HashMap<String, Arc<dyn AiProvider>>>>,
    /// 메모리 캐시 (디스크 I/O 제거)
    cache: Arc<RwLock<ResponseCache>>,
}

impl SessionPool {
    /// 새 세션 풀 생성
    pub fn new() -> Result<Self> {
        let cache = ResponseCache::default_config()?;

        Ok(Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(cache)),
        })
    }

    /// Provider 미리 로드 (pre-warming)
    pub async fn prewarm_provider(&self, provider_name: &str) -> Result<()> {
        let mut providers = self.providers.write().await;

        if !providers.contains_key(provider_name) {
            let provider = ProviderFactory::create(provider_name)?;
            providers.insert(provider_name.to_string(), provider);
        }

        Ok(())
    }

    /// 명령어 생성 (캐시 우선, 캐시 미스 시 AI 호출)
    pub async fn generate_command(
        &self,
        prompt: &str,
        context: &str,
        provider_name: &str,
    ) -> Result<(String, bool)> {
        // 1. 캐시 확인
        {
            let mut cache = self.cache.write().await;
            if let Some(cached_command) = cache.get(prompt, context) {
                return Ok((cached_command, true)); // from_cache = true
            }
        }

        // 2. 캐시 미스: AI 호출
        let command = {
            let mut providers = self.providers.write().await;

            // Provider가 없으면 생성
            if !providers.contains_key(provider_name) {
                let provider = ProviderFactory::create(provider_name)?;
                providers.insert(provider_name.to_string(), provider);
            }

            let provider = providers
                .get(provider_name)
                .ok_or_else(|| AskAiError::AiCliError(
                    format!("Provider '{}' not found", provider_name)
                ))?;

            provider.generate_command(prompt, context).await?
        };

        // 3. 캐시에 저장
        {
            let mut cache = self.cache.write().await;
            cache.set(prompt, context, command.clone());
        }

        Ok((command, false)) // from_cache = false
    }

    /// 캐시 pre-warming
    pub async fn prewarm_cache(&self, context: &str) -> usize {
        let mut cache = self.cache.write().await;
        cache.prewarm(context)
    }

    /// 캐시 클리어
    pub async fn clear_cache(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.clear()
    }

    /// 캐시를 디스크에 저장
    pub async fn save_cache(&self) -> Result<()> {
        let cache = self.cache.read().await;
        cache.save_to_disk()
    }

    /// 현재 로드된 provider 수
    pub async fn provider_count(&self) -> usize {
        let providers = self.providers.read().await;
        providers.len()
    }
}

impl Default for SessionPool {
    fn default() -> Self {
        Self::new().expect("Failed to create session pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_pool_creation() {
        let pool = SessionPool::new();
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_prewarm_provider() {
        let pool = SessionPool::new().unwrap();
        let result = pool.prewarm_provider("gemini").await;

        // Gemini CLI가 설치되어 있지 않으면 에러가 발생할 수 있음
        // 테스트 환경에서는 성공/실패 모두 허용
        match result {
            Ok(_) => {
                assert_eq!(pool.provider_count().await, 1);
            }
            Err(_) => {
                // Gemini CLI가 없는 환경
                assert_eq!(pool.provider_count().await, 0);
            }
        }
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let pool = SessionPool::new().unwrap();

        // Pre-warming
        let count = pool.prewarm_cache("test context").await;
        assert!(count > 0);

        // Clear
        let result = pool.clear_cache().await;
        assert!(result.is_ok());
    }
}
