# askai

간단한 CLI 도구로, 인자로 전달된 문자열을 그대로 한 줄로 출력합니다. `askai`라는 별칭(alias)을 걸어두고 로컬에서 빠르게 확인하는 용도로 쓸 수 있습니다.

## 요구 사항

- Rust 1.70 이상 (rustup으로 설치 권장)

## 설치

```bash
cd ~/VSCode/askai
rustup default stable     # 이미 rustup이 설치되어 있다면 생략 가능
cargo build
```

`cargo build`를 실행하면 `target/debug/askai` 바이너리가 생성됩니다.

## 사용법

```bash
./target/debug/askai hello world
# 출력: hello world
```

alias로 등록하면 더 편하게 사용할 수 있습니다.

```bash
function askai() {
  ~/VSCode/askai/target/debug/askai "$@"
}
```

또는 `~/.zshrc`에 함수를 추가한 뒤 셸을 새로 열거나 `source ~/.zshrc`로 반영하세요.

## 개발

코드를 수정한 뒤에는 다음 명령으로 다시 빌드하거나 실행합니다.

```bash
cargo build            # 바이너리만 갱신
cargo run -- foo bar   # 빌드 후 즉시 실행
```
