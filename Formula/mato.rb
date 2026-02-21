# Homebrew Formula for MATO
# This file should be placed in a homebrew-tap repository
# Repository structure: https://github.com/YOUR_USERNAME/homebrew-tap/Formula/mato.rb

class Mato < Formula
  desc "Multi-Agent Terminal Office - A daemon-based persistent terminal multiplexer"
  homepage "https://github.com/YOUR_USERNAME/mato"
  version "0.1.0"
  
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/YOUR_USERNAME/mato/releases/download/v#{version}/mato-macos-aarch64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    else
      url "https://github.com/YOUR_USERNAME/mato/releases/download/v#{version}/mato-macos-x86_64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/YOUR_USERNAME/mato/releases/download/v#{version}/mato-linux-aarch64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    else
      url "https://github.com/YOUR_USERNAME/mato/releases/download/v#{version}/mato-linux-x86_64.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256"
    end
  end

  def install
    bin.install "mato"
  end

  test do
    system "#{bin}/mato", "--version"
  end
end
