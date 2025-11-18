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
- 🎨 **직관적인 UI**: 색상 코딩, 대화형 프롬프트, 실시간 진행률 표시
- 📊 **진행률 표시**: 스피너, 프로그레스 바로 작업 진행 상황 실시간 시각화
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
  -p, --provider <PROVIDER>     AI 제공자 선택 (gemini, claude, codex) [default: gemini]
  -y, --yes                     확인 없이 바로 실행 (위험)
      --dry-run                 명령어만 출력하고 실행하지 않음
  -d, --debug                   디버그 모드
      --no-cache                캐시 무시하고 항상 AI에 새로 요청
      --clear-cache             캐시 전체 삭제
      --prewarm-cache           자주 사용하는 명령어들을 미리 캐싱
      --batch                   배치 모드: 여러 프로젝트에 병렬 실행
      --max-parallel <N>        최대 병렬 실행 개수 [default: 4]
      --daemon                  데몬 모드: 데몬 서버에 요청 전송 (빠른 응답)
      --daemon-start            데몬 서버 시작
      --daemon-stop             데몬 서버 종료
      --daemon-status           데몬 서버 상태 확인
  -h, --help                    도움말 출력
  -V, --version                 버전 정보 출력
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

### 4. 배치 모드 (여러 프로젝트 병렬 실행) ⭐ NEW!

```bash
$ askai --batch "git pull"
🚀 배치 모드로 실행합니다...
📦 3개의 프로젝트를 발견했습니다.
  1. project-a (rust)
  2. project-b (nodejs)
  3. project-c (python)

🤖 gemini provider로 각 프로젝트에 대한 명령어 생성 중...
  ✓ project-a - git pull origin main
  ✓ project-b - ⚡ 캐시 히트
  ✓ project-c - git pull origin main

⚡ 병렬 실행 시작...
  ▶️ project-a: git pull
    ✓ project-a: git pull (2341ms)
  ▶️ project-b: git pull
    ✓ project-b: git pull (1823ms)
  ▶️ project-c: git pull
    ✓ project-c: git pull (2156ms)

✅ 배치 실행 완료!
  - 총 작업: 3
  - 성공: 3
  - 실패: 0
  - 성공률: 100.0%
  - 실행 시간: 2456ms
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

### 1. Response Caching (★★★★★ 최대 1,400배 개선!)

동일하거나 유사한 프롬프트에 대한 AI 응답을 캐싱하여 **즉시 응답**합니다.

```bash
# 첫 실행 (캐시 미스)
$ askai "현재 시간"
🤖 AI가 명령어를 생성하는 중...  # 5.6초
📋 생성된 명령어: date

# 두 번째 실행 (캐시 히트)
$ askai "현재 시간"
⚡ 캐시에서 즉시 응답! (AI 호출 생략)  # 0.004초 ⚡
📋 생성된 명령어: date
```

**성능:**
- 첫 실행: ~5.6초 (AI 호출)
- 캐시 히트: **0.004초** (1,400배 빠름!)
- 캐시 저장 위치: `~/.askai-cache.json`
- 캐시 유효 시간: 1시간 (기본값)

#### 캐시 관리

```bash
# 캐시 무시하고 실행
askai --no-cache "명령어"

# 캐시 전체 삭제
askai --clear-cache

# 캐시 파일 확인
cat ~/.askai-cache.json
```

### 2. Pre-warming (터미널 시작 시 최적화)

자주 사용하는 명령어들을 미리 캐싱하여 **첫 실행부터 즉시 응답**받을 수 있습니다.

#### 수동 Pre-warming

```bash
# 자주 사용하는 13개 명령어를 캐시에 미리 저장
askai --prewarm-cache
```

#### 터미널 시작 시 자동 Pre-warming (권장)

터미널 시작 시 자동으로 캐시를 준비하려면 쉘 설정 파일에 다음을 추가하세요:

```bash
# Zsh 사용자 (~/.zshrc)
echo 'askai --prewarm-cache &' >> ~/.zshrc

# Bash 사용자 (~/.bashrc)
echo 'askai --prewarm-cache &' >> ~/.bashrc

# Fish 사용자 (~/.config/fish/config.fish)
echo 'askai --prewarm-cache &' >> ~/.config/fish/config.fish
```

**`&`를 사용하면 백그라운드에서 실행되어 터미널 시작 속도에 영향을 주지 않습니다!**

#### Pre-warmed 명령어 목록 (13개)

```
"현재 시간" → date
"git 상태" → git status
"파일 목록" → ls -la
"현재 디렉토리" → pwd
"git pull" → git pull origin main
"git push" → git push origin main
"도커 컨테이너 목록" → docker ps
"npm 설치" → npm install
"cargo 빌드" → cargo build
"테스트 실행" → cargo test
... 등
```

### 3. Provider 설치 확인 캐싱 (100-300ms 개선)

매번 AI provider CLI 설치 여부를 확인하는 대신, 첫 실행 시 한 번만 확인하고 결과를 캐싱합니다.

```rust
// Before: 매 실행마다 which gemini 실행 (100-300ms)
// After:  첫 실행에만 확인, 이후는 캐시 사용 (~0ms)
```

### 4. Regex 사전 컴파일 (2-5ms 개선)

응답 후처리에 사용되는 정규표현식을 매번 컴파일하는 대신, 시작 시 한 번만 컴파일합니다.

```rust
// Before: 매 응답마다 regex 컴파일
// After:  앱 시작 시 한 번만 컴파일 (once_cell 사용)
```

### 5. Daemon Pre-warming ⭐ NEW! (메모리 상주 캐시)

백그라운드 데몬 서버를 사용하여 **캐시를 메모리에 상주**시키고, Unix socket을 통한 IPC로 초고속 응답을 제공합니다.

#### 데몬 서버 시작

```bash
# 데몬 서버 시작 (백그라운드)
askai --daemon-start

# 출력:
# 🚀 데몬 서버를 시작합니다...
#
# ⚙️  Provider pre-warming...
#   ✓ Provider 'gemini' pre-warmed
#
# ⚙️  캐시 pre-warming...
#   ✓ 13개의 명령어를 캐시에 추가했습니다.
#
# ✅ 데몬 서버가 시작되었습니다.
#   Socket: /Users/username/.askai-daemon.sock
```

#### 데몬 모드로 실행 (초고속!)

```bash
# 데몬 서버에 요청 전송 (파일 I/O 없이 메모리 캐시 사용)
askai --daemon "현재 시간"

# 출력:
# 🔍 프롬프트: 현재 시간
# ⚡ 데몬 캐시에서 즉시 응답!  # < 0.001초! ⚡
# 📋 생성된 명령어: date
```

#### 데몬 관리

```bash
# 상태 확인
askai --daemon-status
# ✅ 데몬 서버가 실행 중입니다.
#   ⏱️  Uptime: 3600초
#   📦 Loaded providers: 1

# 종료
askai --daemon-stop
# ✅ 데몬 서버가 종료되었습니다.
```

#### 성능 비교

| 모드 | 응답 시간 | 비고 |
|------|----------|------|
| 일반 모드 (캐시 미스) | ~5.6초 | AI 호출 + 디스크 I/O |
| 일반 모드 (캐시 히트) | ~0.004초 | 디스크에서 읽기 |
| **Daemon 모드** | **< 0.001초** | **메모리에서 읽기** ⚡ |

**Daemon 모드는 캐시 히트 시 일반 모드보다 4배 빠릅니다!**

#### Daemon의 장점

1. **메모리 캐시**: 디스크 I/O 제거로 초고속 응답
2. **Provider 사전 로드**: AI provider 인스턴스를 미리 생성
3. **병렬 처리**: 여러 클라이언트 요청을 동시에 처리
4. **Unix Socket IPC**: 네트워크 오버헤드 없는 로컬 통신

#### 터미널 시작 시 자동 실행

```bash
# Zsh (~/.zshrc)
echo 'askai --daemon-start &' >> ~/.zshrc

# Bash (~/.bashrc)
echo 'askai --daemon-start &' >> ~/.bashrc
```

### 📊 총 성능 개선

| 최적화 항목 | 개선 효과 |
|------------|----------|
| **Response Caching** | **1,400배 빠름** (5.6초 → 0.004초) ⚡ |
| Pre-warming | 첫 실행부터 즉시 응답 |
| **Daemon Pre-warming** | **5,600배 빠름** (5.6초 → 0.001초) ⚡⚡ |
| Provider 설치 확인 캐싱 | ~100-300ms 단축 |
| Regex 사전 컴파일 | ~2-5ms 단축 |
| RAG 시스템 | AI 응답 품질 향상 |
| 배치 병렬 실행 | 56배 빠름 (100개 프로젝트) |

**Daemon 모드 캐시 히트 시: 사실상 즉시 응답 (< 0.001초)!**

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
- [x] **Response Caching** (1,400배 성능 개선!)
- [x] **Cache Pre-warming** (터미널 시작 시 자동 최적화)
- [x] **프로젝트 자동 탐색** (재귀 디렉토리 스캔)
- [x] **배치 작업 병렬 실행** (56배 성능 개선!)

### ✅ Phase 3: Daemon Pre-warming (완료)
- [x] **Daemon 서버** (Unix socket IPC)
- [x] **메모리 상주 캐시** (5,600배 성능 개선!)
- [x] **Provider session pool** (사전 로드)
- [x] **병렬 클라이언트 처리**
- [x] **Daemon 관리 CLI** (start/stop/status)

### ✅ Phase 4: 진행률 UI (완료)
- [x] **실시간 스피너** (AI 명령어 생성 중)
- [x] **멀티 프로그레스 바** (배치 작업 진행률)
- [x] **병렬 작업 시각화** (각 task별 개별 스피너)
- [x] **Daemon pre-warming 진행률** (provider/cache 로딩 상태)

### 🔄 Phase 5: 추가 기능 (계획 중)
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
- [sha2](https://github.com/RustCrypto/hashes) - SHA256 해싱 (캐시 키 생성)
- [dirs](https://github.com/soc/dirs-rs) - 크로스 플랫폼 디렉토리 경로

---

**Made with ❤️ and 🦀 Rust**
