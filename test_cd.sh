#!/bin/bash

echo "=== cd 명령어 테스트 ==="
echo "현재 디렉토리: $(pwd)"

# ai wrapper 사용해서 cd 명령어 실행
source <(echo 'cd $(./target/debug/askai --quiet --yes "src 디렉토리로 이동")')

echo "이동 후 디렉토리: $(pwd)"

if [[ $(pwd) == */src ]]; then
    echo "✅ cd 명령어 성공!"
else
    echo "❌ cd 명령어 실패!"
fi