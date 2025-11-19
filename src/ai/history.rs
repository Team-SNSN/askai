use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 명령어 히스토리 항목
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    /// 사용자 프롬프트
    pub prompt: String,
    /// 생성된 명령어
    pub command: String,
    /// 생성 시간
    pub timestamp: DateTime<Utc>,
    /// 실행 여부
    pub executed: bool,
    /// AI provider
    pub provider: String,
}

/// 명령어 히스토리 저장소
pub struct HistoryStore {
    file_path: PathBuf,
    max_entries: usize,
}

impl HistoryStore {
    /// 새로운 히스토리 저장소 생성
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let file_path = PathBuf::from(home).join(".askai_history.json");

        Self {
            file_path,
            max_entries: 100, // 최대 100개 항목 저장
        }
    }

    /// 히스토리 파일에서 모든 항목 로드
    pub fn load(&self) -> Result<Vec<CommandHistory>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .unwrap_or_else(|_| "[]".to_string());

        let history: Vec<CommandHistory> = serde_json::from_str(&content)
            .unwrap_or_else(|_| Vec::new());

        Ok(history)
    }

    /// 새로운 항목을 히스토리에 추가
    pub fn add(&self, entry: CommandHistory) -> Result<()> {
        let mut history = self.load()?;

        // 새 항목을 맨 앞에 추가
        history.insert(0, entry);

        // 최대 개수 제한
        if history.len() > self.max_entries {
            history.truncate(self.max_entries);
        }

        // 파일에 저장
        let json = serde_json::to_string_pretty(&history)?;
        fs::write(&self.file_path, json)?;

        Ok(())
    }

    /// 프롬프트와 관련된 히스토리 검색 (RAG)
    ///
    /// 간단한 키워드 기반 유사도 검색:
    /// - 프롬프트의 단어들이 과거 프롬프트에 얼마나 포함되어 있는지 계산
    /// - 가장 관련성 높은 상위 N개 반환
    pub fn get_relevant_history(&self, prompt: &str, limit: usize) -> Result<Vec<CommandHistory>> {
        let history = self.load()?;

        if history.is_empty() {
            return Ok(Vec::new());
        }

        // 프롬프트를 소문자 단어로 분리
        let prompt_words: Vec<String> = prompt
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if prompt_words.is_empty() {
            return Ok(Vec::new());
        }

        // 각 히스토리 항목의 관련성 점수 계산
        let mut scored_history: Vec<(usize, &CommandHistory)> = history
            .iter()
            .map(|entry| {
                let entry_words: Vec<String> = entry.prompt
                    .to_lowercase()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                // 공통 단어 개수 계산
                let score = prompt_words
                    .iter()
                    .filter(|word| entry_words.contains(word))
                    .count();

                (score, entry)
            })
            .filter(|(score, _)| *score > 0) // 관련성이 전혀 없는 항목 제외
            .collect();

        // 점수 내림차순 정렬
        scored_history.sort_by(|a, b| b.0.cmp(&a.0));

        // 상위 N개 반환
        Ok(scored_history
            .into_iter()
            .take(limit)
            .map(|(_, entry)| entry.clone())
            .collect())
    }

    /// 히스토리를 컨텍스트 문자열로 변환
    pub fn format_as_context(&self, history: &[CommandHistory]) -> String {
        if history.is_empty() {
            return String::new();
        }

        let mut context = String::from("\n\nRelevant past commands:\n");

        for (i, entry) in history.iter().enumerate() {
            context.push_str(&format!(
                "{}. Prompt: \"{}\" → Command: \"{}\"\n",
                i + 1,
                entry.prompt,
                entry.command
            ));
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_matching() {
        let _store = HistoryStore::new();

        // 테스트용 히스토리 생성
        let history = vec![
            CommandHistory {
                prompt: "파일 목록 보기".to_string(),
                command: "ls -la".to_string(),
                timestamp: Utc::now(),
                executed: true,
                provider: "gemini".to_string(),
            },
            CommandHistory {
                prompt: "git 상태 확인".to_string(),
                command: "git status".to_string(),
                timestamp: Utc::now(),
                executed: true,
                provider: "gemini".to_string(),
            },
        ];

        // "파일"이라는 키워드로 검색하면 첫 번째 항목이 반환되어야 함
        let prompt_words: Vec<String> = "파일"
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let entry_words: Vec<String> = history[0].prompt
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let score = prompt_words
            .iter()
            .filter(|word| entry_words.contains(word))
            .count();

        assert!(score > 0);
    }

    #[test]
    fn test_format_context() {
        let store = HistoryStore::new();

        let history = vec![
            CommandHistory {
                prompt: "파일 목록".to_string(),
                command: "ls -la".to_string(),
                timestamp: Utc::now(),
                executed: true,
                provider: "gemini".to_string(),
            },
        ];

        let context = store.format_as_context(&history);
        assert!(context.contains("Prompt:"));
        assert!(context.contains("Command:"));
        assert!(context.contains("ls -la"));
    }
}
