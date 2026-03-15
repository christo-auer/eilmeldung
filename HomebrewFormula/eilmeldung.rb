class Eilmeldung < Formula
  desc "a feature-rich TUI RSS reader based on the newsflash library"
  homepage "https://github.com/christo-auer/eilmeldung"
  url "https://github.com/christo-auer/eilmeldung/archive/refs/tags/1.2.0.tar.gz"
  sha256 "814e95e7e67cd2da8b4a969deb576eea117aeaf494e4bf8bcf1797a35d0b949e"
  license "GPL-3.0"
  head "https://github.com/christo-auer/eilmeldung.git", branch: "main"
  version "1.2.0"

  depends_on "pkg-config" => :build
  depends_on "rust" => :build

  depends_on "libxml2"
  depends_on "openssl@3"
  depends_on "sqlite"

  on_linux do
    depends_on "llvm" => :build
  end
  
  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "eilmeldung #{version}", shell_output("#{bin}/eilmeldung --version").strip
  end


end
