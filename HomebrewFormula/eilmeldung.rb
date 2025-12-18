class Eilmeldung < Formula
  desc "a feature-rich TUI RSS reader based on the newsflash library"
  homepage "https://github.com/christo-auer/eilmeldung"
  url "https://github.com/christo-auer/eilmeldung/archive/refs/tags/0.2.1.tar.gz"
  sha256 "172dbbf468f2d83896081a22c4d65964cd0fa3dfda7926bfbd6da8b7d4d91d75"
  license "GPL-3.0"
  head "https://github.com/christo-auer/eilmeldung.git", branch: "main"
  version "0.2.1"

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
end
