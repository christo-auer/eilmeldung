class Eilmeldung < Formula
  desc "a feature-rich TUI RSS reader based on the newsflash library"
  homepage "https://github.com/christo-auer/eilmeldung"
  version "0.1.0"
  
  depends_on "libxml2"
  
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/christo-auer/eilmeldung/releases/download/0.1.0/eilmeldung-aarch64-apple-darwin-0.1.0.tar.gz"

    sha256 "8a1e54883035c97a1f6555d07317c84536dcc13e4e1cd75c0b3da013a2d918f6"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/christo-auer/eilmeldung/releases/download/0.1.0/eilmeldung-x86_64-apple-darwin-0.1.0.tar.gz"

    sha256 "91da374b4a11da9d813ce9bac52322a4581b83be1f533f1d0726c3ddef7163c7"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/christo-auer/eilmeldung/releases/download/0.1.0/eilmeldung-x86_64-unknown-linux-gnu-0.1.0.tar.gz"

    sha256 "02eb8b9e345c24c2af6c73a83042982f3397902d81d6f3ca17883a0f292dd82a"
  end

  def install
    bin.install "eilmeldung"
  end
end
