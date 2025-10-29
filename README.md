# askai - AI CLI Wrapper

> 자연어로 터미널 명령어를 생성하고 실행하는 도구

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)]()

## ✨ 특징

- 🤖 **자연어 명령어 생성**: 한국어/영어 프롬프트를 bash 명령어로 자동 변환
- 🔒 **안전성 보장**: 위험한 명령어 자동 차단 및 사용자 확인
- ⚡ **빠른 실행**: Rust로 작성되어 빠른 성능
- 🎨 **직관적인 UI**: 색상 코딩과 대화형 프롬프트
- 🛠️ **다양한 옵션**: dry-run, 자동 승인, 디버그 모드 지원

## 📋 목차

- [설치](#-설치)
- [사용법](#-사용법)
- [예시](#-예시)
- [필수 요구사항](#-필수-요구사항)
- [개발](#-개발)
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
  -p, --provider <PROVIDER>  AI 제공자 선택 (gemini, claude) [default: gemini]
  -y, --yes                  확인 없이 바로 실행 (위험)
      --dry-run              명령어만 출력하고 실행하지 않음
  -d, --debug                디버그 모드
  -h, --help                 도움말 출력
  -V, --version              버전 정보 출력
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

## ⚙️ 필수 요구사항

### Rust

- Rust 1.70 이상 (rustup으로 설치 권장)

### Gemini CLI

`askai`는 Gemini CLI를 사용하여 명령어를 생성합니다. 아래 명령어로 설치하세요:

```bash
npm install -g @google/generative-ai-cli
```

설치 후 Gemini API 키를 설정해야 합니다:

```bash
gemini config set apiKey YOUR_API_KEY
```

API 키는 [Google AI Studio](https://makersuite.google.com/app/apikey)에서 발급받을 수 있습니다.

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

### 🔄 Phase 2: 핵심 기능 (계획 중)
- [ ] 프로젝트 자동 탐색 및 인식
- [ ] 배치 작업 지원
- [ ] 병렬 실행
- [ ] 명령어 히스토리 관리

### 🔮 Phase 3: 고급 기능 (계획 중)
- [ ] Claude Code 통합
- [ ] 컨텍스트 학습
- [ ] 롤백 기능

## 📄 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다.

## 🙏 감사의 말

- [Gemini CLI](https://github.com/google/generative-ai-cli) - AI 명령어 생성
- [clap](https://github.com/clap-rs/clap) - CLI 프레임워크
- [tokio](https://github.com/tokio-rs/tokio) - 비동기 런타임

---

**Made with ❤️ and 🦀 Rust**
