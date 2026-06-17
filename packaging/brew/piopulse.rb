class Piopulse < Formula
  desc "A high-concurrency ESP32 factory flashing tool designed for production lines."
  homepage "https://github.com/Wang-Yang-source/piopulse"
  version "0.2.2"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/Wang-Yang-source/piopulse/releases/download/v#{version}/piopulse-macos-aarch64.tar.gz"
      sha256 "INSERT_MACOS_AARCH64_SHA256_HERE"
    else
      url "https://github.com/Wang-Yang-source/piopulse/releases/download/v#{version}/piopulse-macos-x86_64.tar.gz"
      sha256 "INSERT_MACOS_X86_64_SHA256_HERE"
    end
  elsif OS.linux?
    url "https://github.com/Wang-Yang-source/piopulse/releases/download/v#{version}/piopulse-linux-x86_64.tar.gz"
    sha256 "INSERT_LINUX_X86_64_SHA256_HERE"
  end

  def install
    bin.install "piopulse"
  end

  test do
    # Verify binary is executable and outputs help / version
    assert_match "PioPulse", shell_output("#{bin}/piopulse --help")
  end
end
