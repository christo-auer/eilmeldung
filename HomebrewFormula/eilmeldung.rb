class Eilmeldung < Formula
  desc "a feature-rich TUI RSS reader based on the newsflash library"
  homepage "https://github.com/christo-auer/eilmeldung"
  version "0.1.0"
  
  depends_on "libxml2"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/christo-auer/eilmeldung/releases/download/0.1.0/eilmeldung-aarch64-apple-darwin-0.1.0.tar.gz"
    sha256 "fd19cff271298780831c229a34aab48b10eb3849d9e8ae0d0b707dd0beb46112"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/christo-auer/eilmeldung/releases/download/0.1.0/eilmeldung-x86_64-apple-darwin-0.1.0.tar.gz"
    sha256 "96613a98a4479ce45c1c9d5b15caea7006496ae9896191dd54763de2b8016e4d"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/christo-auer/eilmeldung/releases/download/0.1.0/eilmeldung-x86_64-unknown-linux-gnu-0.1.0.tar.gz"
    sha256 "0b1851cd62807ce9b53f09a9f1f738b55e45f43129a0177ae855339cc8e41e7f"
  end

  def install
    bin.install "eilmeldung"
  end
end
