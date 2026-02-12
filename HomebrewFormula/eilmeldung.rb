class Eilmeldung < Formula
  desc "a feature-rich TUI RSS reader based on the newsflash library"
  homepage "https://github.com/christo-auer/eilmeldung"
  url "https://github.com/christo-auer/eilmeldung/archive/refs/tags/0.9.3.tar.gz"
  sha256 "5334aea574e4df38c197340d5106cef2f1e5af3d977a232aae62db9eb21f9b8f"
  license "GPL-3.0"
  head "https://github.com/christo-auer/eilmeldung.git", branch: "main"
  version "0.9.3"

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
