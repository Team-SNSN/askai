#!/bin/bash

# askai를 eval로 실행하는 shell function 시뮬레이션
function ai() {
    local cmd=$(./target/debug/askai --yes "$@" 2>&1 >/dev/tty)
    if [ -n "$cmd" ]; then
        echo "실행할 명령어: $cmd"
        eval "$cmd"
    fi
}

# 테스트 1: 현재 디렉토리 확인
echo "=== 테스트 1: 현재 디렉토리 ==="
pwd

# 테스트 2: cd 명령어 테스트 (askai로 명령어 생성 후 eval)
echo -e "\n=== 테스트 2: cd 명령어 ==="
cmd=$(./target/debug/askai --yes "src 디렉토리로 이동" 2>/dev/null)
echo "생성된 명령어: $cmd"
eval "$cmd"
echo "이동 후 디렉토리: $(pwd)"

# 원래 위치로 복귀
cd /Users/ys/Code/askai

echo -e "\n✅ 모든 테스트 완료!"
