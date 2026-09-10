class PtdCli < Formula
  desc "CLI for PT-Depiler browser extension via Native Messaging"
  homepage "https://github.com/pt-plugins/ptd-cli"
  version "0.1.4"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/pt-plugins/ptd-cli/releases/download/v0.1.4/ptd-cli-v0.1.4-aarch64-apple-darwin.tar.gz"
      sha256 "f09ca2ca95e90e0404b24af0c20d5aecb358a586d84d5c28fdaec7031278615e"
    else
      url "https://github.com/pt-plugins/ptd-cli/releases/download/v0.1.4/ptd-cli-v0.1.4-x86_64-apple-darwin.tar.gz"
      sha256 "f348c342ac8a1752e75b8c15a63071c47b29e943b465916c4487259c1ed43a37"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/pt-plugins/ptd-cli/releases/download/v0.1.4/ptd-cli-v0.1.4-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "e5d982f42c14fd8bb98c68df2cf62a53430e492a9f4e3c69f2443736723c54c6"
    else
      url "https://github.com/pt-plugins/ptd-cli/releases/download/v0.1.4/ptd-cli-v0.1.4-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "47df68f1d3a940263758990767b0a025e571daefa60bf9e24439b75c1fa15089"
    end
  end

  def install
    libexec.install "ptd", "ptd-host"
    (bin/"ptd").write_env_script libexec/"ptd",
      PTD_NATIVE_HOST_PATH: opt_libexec/"ptd-host"
  end

  test do
    assert_match "ptd", shell_output("#{bin}/ptd --help")
  end
end
