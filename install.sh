#!/bin/bash

# askai 설치 스크립트
# 이 스크립트는 askai를 빌드하고 쉘 통합을 자동으로 설정합니다

set -e

echo "🚀 askai 설치를 시작합니다..."

# 1. Rust/Cargo 확인
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo가 설치되어 있지 않습니다."
    echo "   https://rustup.rs 에서 Rust를 먼저 설치해주세요."
    exit 1
fi

# 2. Release 빌드
echo "📦 askai를 빌드합니다..."
cargo build --release

# 3. 바이너리 설치
INSTALL_PATH="$HOME/.local/bin"
mkdir -p "$INSTALL_PATH"

echo "📂 바이너리를 $INSTALL_PATH에 설치합니다..."
cp target/release/askai "$INSTALL_PATH/askai-bin"
chmod +x "$INSTALL_PATH/askai-bin"

# 4. Shell 감지
if [ -n "$ZSH_VERSION" ]; then
    SHELL_TYPE="zsh"
    SHELL_RC="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_TYPE="bash"
    SHELL_RC="$HOME/.bashrc"
else
    SHELL_TYPE="unknown"
    SHELL_RC="$HOME/.profile"
fi

echo "🐚 Shell 타입: $SHELL_TYPE"

# 5. Shell function 생성
SHELL_FUNCTION='
# askai - AI-powered terminal automation
# 이 함수는 askai가 현재 쉘 세션에서 명령어를 실행할 수 있게 합니다
askai() {
    local ASKAI_BIN="$HOME/.local/bin/askai-bin"

    # 특별한 옵션들은 바이너리로 직접 전달
    if [[ "$1" == "--help" ]] || [[ "$1" == "--version" ]] || \
       [[ "$1" == "--clear-cache" ]] || [[ "$1" == "--prewarm-cache" ]] || \
       [[ "$1" == "--daemon-start" ]] || [[ "$1" == "--daemon-stop" ]] || \
       [[ "$1" == "--daemon-status" ]] || [[ "$1" == "--batch" ]]; then
        "$ASKAI_BIN" "$@"
        return $?
    fi

    # 일반 명령어 생성 및 실행
    local cmd=$("$ASKAI_BIN" --quiet --yes "$@" 2>/dev/null)

    if [ $? -eq 0 ] && [ -n "$cmd" ]; then
        # 생성된 명령어를 stderr에 표시 (선택적)
        echo "🤖 실행: $cmd" >&2

        # 명령어 실행
        eval "$cmd"
    else
        # 에러가 발생한 경우 전체 출력 보여주기
        "$ASKAI_BIN" "$@"
    fi
}

# PATH에 추가 (필요한 경우)
export PATH="$HOME/.local/bin:$PATH"
'

# 6. Shell RC 파일 업데이트
echo "📝 $SHELL_RC 파일을 업데이트합니다..."

# 기존 설정 제거 (있는 경우)
if grep -q "# askai - AI-powered terminal automation" "$SHELL_RC" 2>/dev/null; then
    echo "   기존 askai 설정을 제거합니다..."
    # 임시 파일에 기존 설정 제거한 내용 저장
    awk '/# askai - AI-powered terminal automation/,/^$/{next}1' "$SHELL_RC" > "$SHELL_RC.tmp"
    mv "$SHELL_RC.tmp" "$SHELL_RC"
fi

# 새 설정 추가
echo "$SHELL_FUNCTION" >> "$SHELL_RC"

echo "✅ 설치가 완료되었습니다!"
echo ""
echo "🎉 다음 명령어를 실행하여 설정을 적용하세요:"
echo "   source $SHELL_RC"
echo ""
echo "📖 사용 예시:"
echo "   askai \"현재 시간\""
echo "   askai \"src 디렉토리로 이동\""
echo "   askai \"모든 git 프로젝트 pull\""
echo ""
echo "💡 팁: cd 같은 쉘 내장 명령어도 정상 작동합니다!"