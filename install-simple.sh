#!/bin/bash

# askai 간단 설치 스크립트
# 바이너리를 askai-bin으로, wrapper를 askai로 설치

set -e

echo "🚀 askai 설치를 시작합니다..."

# 1. Release 빌드
echo "📦 askai를 빌드합니다..."
cargo build --release

# 2. 설치 경로 설정
INSTALL_PATH="/usr/local/bin"

# 권한 확인
if [ ! -w "$INSTALL_PATH" ]; then
    echo "⚠️  $INSTALL_PATH에 쓰기 권한이 없습니다. sudo로 실행합니다."
    SUDO="sudo"
else
    SUDO=""
fi

# 3. 바이너리 설치 (askai-bin으로)
echo "📂 바이너리를 설치합니다..."
$SUDO cp target/release/askai "$INSTALL_PATH/askai-bin"
$SUDO chmod +x "$INSTALL_PATH/askai-bin"

# 4. Wrapper 스크립트 생성
cat > /tmp/askai-wrapper << 'EOF'
#!/bin/bash
# askai wrapper - 명령어를 현재 쉘에서 실행

ASKAI_BIN="askai-bin"

# 특별한 옵션들은 바이너리로 직접 전달
case "$1" in
    --help|--version|--clear-cache|--prewarm-cache|--daemon-*|--batch|-d|--debug)
        exec "$ASKAI_BIN" "$@"
        ;;
esac

# 일반 명령어 생성 및 실행
cmd=$("$ASKAI_BIN" --quiet --yes "$@" 2>/dev/null)

if [ $? -eq 0 ] && [ -n "$cmd" ]; then
    # 명령어 실행
    eval "$cmd"
else
    # 에러 발생시 일반 모드로 실행
    exec "$ASKAI_BIN" "$@"
fi
EOF

# 5. Wrapper 설치
echo "📝 Wrapper를 설치합니다..."
$SUDO mv /tmp/askai-wrapper "$INSTALL_PATH/askai"
$SUDO chmod +x "$INSTALL_PATH/askai"

echo ""
echo "✅ 설치가 완료되었습니다!"
echo ""
echo "🎉 이제 다음과 같이 사용할 수 있습니다:"
echo "   askai \"현재 시간\""
echo "   askai \"src 디렉토리로 이동\""
echo "   askai \"모든 파일 목록 보기\""
echo ""
echo "💡 cd 같은 쉘 내장 명령어도 정상 작동합니다!"
echo ""
echo "📖 도움말: askai --help"