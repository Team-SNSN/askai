/// AI provider용 프롬프트 템플릿 생성기
///
/// 모든 provider가 일관된 프롬프트 형식을 사용하도록 중앙 집중화된 템플릿 관리를 제공합니다.
pub struct PromptTemplate;

impl PromptTemplate {
    /// bash 명령어 생성을 위한 기본 프롬프트 템플릿 생성
    ///
    /// # Arguments
    /// * `prompt` - 사용자 프롬프트 (자연어)
    /// * `context` - 실행 환경 컨텍스트 (현재 디렉토리, OS 등)
    /// * `provider_rules` - Provider별 추가 규칙 (옵션)
    ///
    /// # Returns
    /// * 완전한 프롬프트 문자열
    ///
    /// # Examples
    /// ```
    /// use askai::ai::prompt_template::PromptTemplate;
    ///
    /// let prompt = PromptTemplate::build_command_generation_prompt(
    ///     "파일 목록 보기",
    ///     "Current directory: /home/user",
    ///     None
    /// );
    /// assert!(prompt.contains("파일 목록 보기"));
    /// ```
    pub fn build_command_generation_prompt(
        prompt: &str,
        context: &str,
        provider_rules: Option<&str>,
    ) -> String {
        let mut template = format!(
            "You are a bash command generator. Convert natural language to a single bash command.\n\n\
             RULES:\n\
             - Output ONLY the bash command (no explanations, no markdown)\n\
             - Do NOT say \"I cannot\" or similar - just output the command\n\
             - Be precise and accurate\n"
        );

        // Provider별 추가 규칙이 있으면 추가
        if let Some(rules) = provider_rules {
            template.push_str(&format!("{}\n", rules));
        }

        template.push_str(&format!(
            "\nContext: {}\n\
             Request: {}\n\n\
             Examples:\n\
             \"파일 목록\" → ls -la\n\
             \"git 상태\" → git status\n\
             \"txt 파일 찾기\" → find . -name \"*.txt\"\n\
             \"현재 시간\" → date\n\n\
             Command:",
            context, prompt
        ));

        template
    }

    /// Gemini provider용 최적화된 프롬프트 생성
    pub fn for_gemini(prompt: &str, context: &str) -> String {
        Self::build_command_generation_prompt(
            prompt,
            context,
            None, // Gemini는 기본 규칙만 사용
        )
    }

    /// Claude provider용 최적화된 프롬프트 생성
    pub fn for_claude(prompt: &str, context: &str) -> String {
        Self::build_command_generation_prompt(
            prompt,
            context,
            None, // Claude도 기본 규칙만 사용
        )
    }

    /// Codex provider용 최적화된 프롬프트 생성
    pub fn for_codex(prompt: &str, context: &str) -> String {
        Self::build_command_generation_prompt(
            prompt,
            context,
            Some("- Be concise and use standard Unix commands"), // Codex 전용 규칙
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_template() {
        let template = PromptTemplate::build_command_generation_prompt(
            "파일 목록",
            "Current directory: /home",
            None,
        );

        assert!(template.contains("파일 목록"));
        assert!(template.contains("Current directory: /home"));
        assert!(template.contains("RULES:"));
        assert!(template.contains("Examples:"));
    }

    #[test]
    fn test_gemini_template() {
        let template = PromptTemplate::for_gemini("git 상태", "OS: linux");

        assert!(template.contains("git 상태"));
        assert!(template.contains("OS: linux"));
    }

    #[test]
    fn test_claude_template() {
        let template = PromptTemplate::for_claude("현재 시간", "Shell: bash");

        assert!(template.contains("현재 시간"));
        assert!(template.contains("Shell: bash"));
    }

    #[test]
    fn test_codex_template_with_custom_rules() {
        let template = PromptTemplate::for_codex("txt 파일 찾기", "Current directory: /var");

        assert!(template.contains("txt 파일 찾기"));
        assert!(template.contains("concise"));
        assert!(template.contains("standard Unix commands"));
    }

    #[test]
    fn test_template_with_provider_rules() {
        let template = PromptTemplate::build_command_generation_prompt(
            "test",
            "context",
            Some("- Custom rule 1\n- Custom rule 2"),
        );

        assert!(template.contains("Custom rule 1"));
        assert!(template.contains("Custom rule 2"));
    }
}
