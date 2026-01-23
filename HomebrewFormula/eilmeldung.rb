class Eilmeldung < Formula
  desc "a feature-rich TUI RSS reader based on the newsflash library"
  homepage "https://github.com/christo-auer/eilmeldung"
  url "https://github.com/christo-auer/eilmeldung/archive/refs/tags/0.8.0.tar.gz"
  sha256 "f5a27f13e6d40eb1b1a2e5ed4e665bfa1ea2187e7b3b823f3cbe1a5e16f9c22e"
  license "GPL-3.0"
  head "https://github.com/christo-auer/eilmeldung.git", branch: "main"
  version "0.8.0"

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
