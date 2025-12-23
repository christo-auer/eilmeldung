class Eilmeldung < Formula
  desc "a feature-rich TUI RSS reader based on the newsflash library"
  homepage "https://github.com/christo-auer/eilmeldung"
  url "https://github.com/christo-auer/eilmeldung/archive/refs/tags/0.4.6.tar.gz"
  sha256 "328b067b5cb145c20060566394558af69ee51b75e3f42b979b806dfb8e305f9e"
  license "GPL-3.0"
  head "https://github.com/christo-auer/eilmeldung.git", branch: "main"
  version "0.4.6"

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
