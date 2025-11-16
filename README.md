# askai - AI CLI Wrapper

> 자연어로 터미널 명령어를 생성하고 실행하는 도구

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

## ✨ 특징

- 🤖 **자연어 명령어 생성**: 한국어/영어 프롬프트를 bash 명령어로 자동 변환
- 🔄 **멀티 AI Provider 지원**: Gemini, Claude, Codex 등 여러 AI CLI 지원
- 🧠 **RAG 시스템**: 과거 명령어 히스토리를 학습하여 더 정확한 제안
- 🔒 **안전성 보장**: 위험한 명령어 자동 차단 및 사용자 확인
- ⚡ **빠른 실행**: Rust로 작성 + 캐싱 최적화로 초고속 성능
- 🎨 **직관적인 UI**: 색상 코딩과 대화형 프롬프트
- 🛠️ **다양한 옵션**: dry-run, 자동 승인, 디버그 모드 지원
- 🔌 **확장 가능한 구조**: 새로운 AI provider를 쉽게 추가 가능

## 📋 목차

- [설치](#-설치)
- [사용법](#-사용법)
- [예시](#-예시)
- [RAG 시스템](#-rag-시스템)
- [성능 최적화](#-성능-최적화)
- [필수 요구사항](#-필수-요구사항)
- [안전 기능](#-안전-기능)
- [개발](#-개발)
- [로드맵](#️-로드맵)
- [라이선스](#-라이선스)

## 🚀 설치

### 소스에서 빌드

```bash
# 저장소 클론
git clone https://github.com/Team-SNSN/askai.git
cd askai

# 빌드
cargo build --release

# 설치 (선택사항)
cargo install --path .
```

## 📖 사용법

### 기본 사용

```bash
askai "자연어 명령어"
```

### 옵션

```
Usage: askai [OPTIONS] <PROMPT>...

Arguments:
  <PROMPT>...  자연어 프롬프트

Options:
  -p, --provider <PROVIDER>  AI 제공자 선택 (gemini, claude, codex) [default: gemini]
  -y, --yes                  확인 없이 바로 실행 (위험)
      --dry-run              명령어만 출력하고 실행하지 않음
  -d, --debug                디버그 모드
  -h, --help                 도움말 출력
  -V, --version              버전 정보 출력
```

### Provider 선택

다양한 AI provider를 선택하여 사용할 수 있습니다:

```bash
# Gemini 사용 (기본값)
askai "파일 목록"

# Claude 사용
askai -p claude "파일 목록"

# Codex 사용
askai -p codex "파일 목록"
```

## 💡 예시

### 1. 기본 명령어 생성

```bash
$ askai "현재 디렉토리의 파일 목록"
🔍 프롬프트: 현재 디렉토리의 파일 목록
🤖 AI가 명령어를 생성하는 중...

📋 생성된 명령어:
  ls -la

위험도:

▶️  이 명령어를 실행하시겠습니까? (y/n) y
▶️  실행 중: ls -la
...

✅ 완료!
```

### 2. Dry-run 모드 (명령어만 확인)

```bash
$ askai "모든 txt 파일 삭제" --dry-run
🔍 프롬프트: 모든 txt 파일 삭제
🤖 AI가 명령어를 생성하는 중...

📋 생성된 명령어:
  rm *.txt

ℹ️ 명령어만 출력합니다 (실행하지 않음).
```

### 3. 자동 승인 모드

```bash
$ askai "현재 시간" --yes
🔍 프롬프트: 현재 시간
🤖 AI가 명령어를 생성하는 중...

📋 생성된 명령어:
  date

⚡ 자동 승인 모드로 실행합니다...
▶️  실행 중: date
2025년 10월 29일 수요일 13시 50분 51초 KST

✅ 완료!
```

## 🧠 RAG 시스템

`askai`는 **RAG (Retrieval-Augmented Generation)** 시스템을 탑재하여 과거 명령어 히스토리를 학습합니다.

### 작동 방식

1. **히스토리 저장**: 모든 명령어가 `~/.askai_history.json`에 자동 저장
2. **관련 검색**: 새 프롬프트와 유사한 과거 명령어를 키워드 기반으로 검색
3. **컨텍스트 강화**: 관련 히스토리를 AI에게 제공하여 더 정확한 명령어 생성

### 예시

```bash
# 첫 번째 실행
$ askai "파일 목록"
📋 생성된 명령어: ls -la

# 두 번째 실행 (유사한 프롬프트)
$ askai "모든 파일 보기"
# RAG 시스템이 과거의 "ls -la" 명령어를 참조
📋 생성된 명령어: ls -la
```

### 저장되는 정보

- 사용자 프롬프트
- 생성된 명령어
- 실행 시간
- 실행 여부
- 사용된 AI provider

### 히스토리 관리

히스토리 파일은 최대 100개 항목을 저장하며, 자동으로 오래된 항목을 삭제합니다.

```bash
# 히스토리 파일 확인
cat ~/.askai_history.json

# 히스토리 초기화 (필요시)
rm ~/.askai_history.json
```

## ⚡ 성능 최적화

### 1. Provider 설치 확인 캐싱 (100-300ms 개선)

매번 AI provider CLI 설치 여부를 확인하는 대신, 첫 실행 시 한 번만 확인하고 결과를 캐싱합니다.

```rust
// Before: 매 실행마다 which gemini 실행 (100-300ms)
// After:  첫 실행에만 확인, 이후는 캐시 사용 (~0ms)
```

### 2. Regex 사전 컴파일 (2-5ms 개선)

응답 후처리에 사용되는 정규표현식을 매번 컴파일하는 대신, 시작 시 한 번만 컴파일합니다.

```rust
// Before: 매 응답마다 regex 컴파일
// After:  앱 시작 시 한 번만 컴파일 (once_cell 사용)
```

### 3. 총 성능 개선

- **설치 확인 캐싱**: ~100-300ms 단축
- **Regex 사전 컴파일**: ~2-5ms 단축
- **RAG 시스템**: AI 응답 품질 향상으로 재시도 횟수 감소

**총 오버헤드 감소: 약 100-310ms/실행**

## ⚙️ 필수 요구사항

### Rust

- Rust 1.70 이상 (rustup으로 설치 권장)

### AI Provider CLI 설치

`askai`는 사용자 환경에 설치된 AI CLI를 활용합니다. 사용하려는 provider에 맞춰 CLI를 설치하세요:

#### Gemini CLI (기본값)

```bash
npm install -g @google/generative-ai-cli
```

설치 후 API 키 설정:

```bash
gemini config set apiKey YOUR_API_KEY
```

API 키는 [Google AI Studio](https://makersuite.google.com/app/apikey)에서 발급받을 수 있습니다.

#### Claude CLI

```bash
npm install -g @anthropics/claude-cli
```

설치 후 해당 CLI의 설정 방법에 따라 API 키를 설정하세요.

#### Codex CLI

```bash
npm install -g openai-codex-cli
```

설치 후 해당 CLI의 설정 방법에 따라 API 키를 설정하세요.

**참고**: 최소 하나 이상의 AI provider CLI가 설치되어 있어야 합니다.

## 🔒 안전 기능

### 위험 명령어 차단

다음과 같은 위험한 명령어는 자동으로 차단됩니다:

- `rm -rf /` - 루트 디렉토리 삭제
- `dd if=/dev/zero` - 디스크 덮어쓰기
- `mkfs` - 파일시스템 포맷
- `:(){ :|:& };:` - Fork bomb

### 위험도 표시

- 🟢 **Low**: 일반 명령어 (녹색)
- 🟡 **Medium**: `sudo` 포함 명령어 (노란색)
- 🔴 **High**: 위험 키워드 포함 (빨간색)

## 🛠️ 개발

### 빌드

```bash
# 개발 빌드
cargo build

# 릴리스 빌드
cargo build --release

# 실행
cargo run -- "현재 시간"
```

### 테스트

```bash
# 모든 테스트 실행
cargo test

# 통합 테스트만
cargo test --test integration_test
```

### 코드 품질

```bash
# 포맷팅
cargo fmt

# 린팅
cargo clippy
```

## 🗺️ 로드맵

### ✅ Phase 1: MVP (완료)
- [x] 기본 CLI 인터페이스
- [x] Gemini CLI 통합
- [x] 단일 명령어 생성 및 실행
- [x] 기본 안전성 검사
- [x] 테스트 작성
- [x] 멀티 AI Provider 지원 (Gemini, Claude, Codex)
- [x] 확장 가능한 Provider 아키텍처

### ✅ Phase 2: 성능 최적화 & 스마트 기능 (완료)
- [x] 명령어 히스토리 관리 (RAG 시스템)
- [x] Provider 설치 확인 캐싱
- [x] Regex 사전 컴파일 최적화
- [x] 컨텍스트 학습 (RAG 기반)

### 🔄 Phase 3: 고급 기능 (계획 중)
- [ ] 프로젝트 자동 탐색 및 인식
- [ ] 배치 작업 지원
- [ ] 병렬 실행
- [ ] 추가 AI Provider 지원 (GPT-4, etc.)
- [ ] 롤백 기능
- [ ] 플러그인 시스템

## 📄 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다.

## 🙏 감사의 말

- [Gemini CLI](https://github.com/google/generative-ai-cli) - AI 명령어 생성
- [clap](https://github.com/clap-rs/clap) - CLI 프레임워크
- [tokio](https://github.com/tokio-rs/tokio) - 비동기 런타임
- [once_cell](https://github.com/matklad/once_cell) - 지연 초기화 및 캐싱
- [chrono](https://github.com/chronotope/chrono) - 시간 처리
- [serde](https://github.com/serde-rs/serde) - 직렬화/역직렬화

---

**Made with ❤️ and 🦀 Rust**
