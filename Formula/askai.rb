class Askai < Formula
  desc "AI-powered terminal automation tool"
  homepage "https://github.com/Team-SNSN/askai"
  url "https://github.com/Team-SNSN/askai/archive/v0.1.0.tar.gz"
  sha256 "22aebfb0e5012ffbdce3e432de46eabd3bab4efaeeada69cb68bbfc968059c7c"
  license "MIT"
  version "0.1.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      askai requires Gemini CLI to be installed and configured.

      Install Gemini CLI:
        npm install -g @google/generative-ai-cli

      Configure API key:
        gemini config set apiKey YOUR_API_KEY

      Get your API key from: https://makersuite.google.com/app/apikey
    EOS
  end

  test do
    assert_match "askai", shell_output("#{bin}/askai --version")
    # Basic functionality test
    system "#{bin}/askai", "--help"
  end
end
