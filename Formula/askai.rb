class Askai < Formula
  desc "AI-powered terminal automation tool"
  homepage "https://github.com/Team-SNSN/askai"
  url "https://github.com/Team-SNSN/askai/archive/v0.1.0.tar.gz"
  sha256 "a39171b1ce688cde73fea1fedaa7f33dff18bd485ef367f7cd4aeb51af92f590"
  license "MIT"
  version "0.1.0"

  depends_on "rust" => :build

  def install
    # Rust 바이너리 빌드
    system "cargo", "build", "--release", "--locked"

    # 바이너리를 askai-bin으로 설치
    bin.install "target/release/askai" => "askai-bin"

    # Wrapper 스크립트 생성 및 설치
    (bin/"askai").write wrapper_script
  end

  def wrapper_script
    <<~EOS
      #!/bin/bash
      # askai wrapper - Homebrew 버전
      # 이 스크립트는 명령어를 현재 쉘에서 실행할 수 있게 합니다

      ASKAI_BIN="#{opt_bin}/askai-bin"

      # 특별한 옵션들은 바이너리로 직접 전달
      case "$1" in
          --help|--version|--clear-cache|--prewarm-cache|--daemon-*|--batch|-d|--debug)
              exec "$ASKAI_BIN" "$@"
              ;;
      esac

      # 일반 명령어 생성 및 실행
      if [ $# -eq 0 ]; then
          echo "사용법: askai \\"자연어 명령어\\"" >&2
          echo "예시: askai \\"현재 시간\\"" >&2
          exit 1
      fi

      # 명령어 생성
      cmd=$("$ASKAI_BIN" --quiet --yes "$@" 2>/dev/null)

      if [ $? -eq 0 ] && [ -n "$cmd" ]; then
          # 명령어 실행 (eval 사용)
          eval "$cmd"
      else
          # 에러 발생시 일반 모드로 실행
          exec "$ASKAI_BIN" "$@"
      fi
    EOS
  end

  def caveats
    <<~EOS
      🎉 askai가 설치되었습니다!

      이제 eval 없이 직접 사용할 수 있습니다:
        askai "현재 시간"
        askai "src 디렉토리로 이동"
        askai "모든 파일 목록"

      💡 cd 같은 쉘 내장 명령어도 정상 작동합니다!

      처음 사용시 Gemini API 키 설정이 필요합니다:
        export GEMINI_API_KEY="your-api-key"

      Get your API key from: https://makersuite.google.com/app/apikey
    EOS
  end

  test do
    assert_match "askai", shell_output("#{bin}/askai --version")
    # Basic functionality test
    system "#{bin}/askai", "--help"
  end
end
