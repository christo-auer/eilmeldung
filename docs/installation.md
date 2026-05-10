# Installation

Follow any of the installation methods below, then run *eilmeldung*. It will guide you through the setup process.

---

## Table of Contents

- [Important: Nerd Fonts](#important-nerd-fonts)
- [Manual Installation](#manual-installation)
- [Via Homebrew](#via-homebrew)
- [Via AUR (Arch)](#via-aur-arch)
- [Via Cargo](#via-cargo)
  - [Building on Windows](#building-on-windows)
- [Nix Flake and Home Manager](#nix-flake-and-home-manager)
- [Void Linux](#void-linux)
- [Windows via Scoop](#windows-via-scoop)
- [NetBSD](#netbsd)

---

## Important: Nerd Fonts

You need a [Nerd Font](https://github.com/ryanoasis/nerd-fonts) compatible font/terminal for icons to display correctly!

---

## Manual Installation

Go to [Latest Releases](https://github.com/christo-auer/eilmeldung/releases/latest), pick the according archive, extract and execute the `eilmeldung` binary.

---

## Via Homebrew

To install via [homebrew](https://brew.sh), tap this repository and install *eilmeldung*:

```bash
brew tap christo-auer/eilmeldung https://github.com/christo-auer/eilmeldung
brew install eilmeldung
```

---

## Via AUR (Arch)

There are three AUR packages

- `eilmeldung` compiles the latest release
- `eilmeldung-git` the `HEAD` of `main`.
- `eilmeldung-bin` installs the statically linked binaries

Use `paru` or `yay` to install.

---

## Via Cargo

In order to compile `eilmeldung` from source, you need `cargo` with a `rust` compiler with at least edition 2024 (e.g., use `rustup` and `rustup default stable`) and some build deps:

| Distribution | Dependencies (Build and Runtime)                                                           |
| ---          | ---                                                                                        |
| Ubuntu       | `# apt install rustup build-essential perl libssl-dev pkg-config libxml2-dev clang libsqlite3-dev`<br>install stable rust toolchain as your user: `rustup default stable` |
| Fedora       | `# dnf install cargo rust perl libxml2-devel clang sqlite-devel openssl-devel`<br> If the compiler complains about missing `perl` packages, just install them manually (thanks to @austingarrigus):<br> `dnf install  perl-FindBin perl-IPC-Cmd perl-File-Compare perl-Time-Piece`|
| Arch         | `# pacman -S cargo base-devel clang perl libxml2 openssl libsixel sqlite3`                             |

```bash
cargo install eilmeldung --locked
```
To compile the latest unreleased version (`HEAD` in `main`):
```bash
cargo install --locked --git https://github.com/christo-auer/eilmeldung
```

### Building on Windows

Windows requires a few extra steps because `libxml2` must be provided as a static library via [vcpkg](https://github.com/microsoft/vcpkg), and OpenSSL is compiled from source (requiring Perl).

**Step 1 — Install the Rust toolchain**

```pwsh
winget install Rustlang.Rustup
rustup default stable
rustup target add x86_64-pc-windows-msvc
```

**Step 2 — Install Perl**

Perl is required to compile OpenSSL from source. Install via scoop or [Strawberry Perl](https://strawberryperl.com):

```pwsh
scoop install perl
```

**Step 3 — Build**

The helper script handles everything else automatically (cloning vcpkg, installing libxml2, setting all required environment variables):

```pwsh
.\scripts\build-windows.ps1
```

vcpkg is installed to `$env:LOCALAPPDATA\vcpkg` by default. If Perl or vcpkg are in non-default locations, pass them explicitly:

```pwsh
.\scripts\build-windows.ps1 -PerlPath "C:\Users\you\scoop\apps\perl\current\perl\bin\perl.exe" -VcpkgRoot "D:\my-vcpkg"
```

The binary will be at `target\x86_64-pc-windows-msvc\release\eilmeldung.exe`.

> **Note:** The first build downloads and compiles vcpkg dependencies and OpenSSL from source — expect 20–30 minutes. Subsequent builds are much faster.

---

## Nix Flake and Home Manager

There are two packages, `eilmeldung` (latest release) and `eilmeldung-git` (`HEAD` of `main`).
Add *eilmeldung* to your inputs, apply `eilmeldung.overlays.default` overlay to `pkgs`. If you want Home Manager integration, add Home Manager module `eilmeldung.homeManager.default`.

Here is an example:

```nix
{
  inputs = {
    eilmeldung.url = "github:christo-auer/eilmeldung";
  };

  outputs = { nixpkgs, home-manager, eilmeldung, ... }: {
    homeConfigurations."..." = home-manager.lib.homeManagerConfiguration {
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ eilmeldung.overlays.default ];
      };

      modules = [
        # ...
        eilmeldung.homeManager.default
      ];
    };
  };
}
```

There are two packages: `eilmeldung` (latest release) and `eilmeldung-git` for `HEAD` of `main`:

```nix
home.packages = [ eilmeldung.packages.x86_64-linux.eilmeldung ];
# or for HEAD of main
home.packages = [ eilmeldung.packages.x86_64-linux.eilmeldung-git ];
```

Home Manager configuration works by defining the settings from the configuration file:

```nix
programs.eilmeldung = {
  enable = true;
  # for HEAD of main
  #package = eilmeldung.packages.x86_64-linux.eilmeldung-git;

  settings = {
    refresh_fps = 60;
    article_scope = "unread";


    theme = {
      color_palette = {
        background = "#1e1e2e";
        # ...
      };
    };

    input_config.mappings = {
        "q" = ["quit"];
        "j" = ["down"];
        "k" = ["up"];
        "g g" = ["gotofirst"];
        "G" = ["gotolast"];
        "o" = ["open" "read" "nextunread"];
    };

    feed_list = [
      "query: \"Today Unread\" today unread"
      "query: \"Today Marked\" today marked"
      "feeds"
      "* categories"
      "tags"
    ];
  };
};
```
## Void Linux

Via an unoffical repository:

```bash
echo "repository=https://raw.githubusercontent.com/Event-Horizon-VL/blackhole-vl/repository-x86_64" | sudo tee /etc/xbps.d/20-repository-extra.conf && sudo xbps-install -S eilmeldung
```

---

## Windows via Scoop

Install [scoop](https://scoop.sh/) and then

```
  scoop bucket add eilmeldung https://github.com/christo-auer/eilmeldung
  scoop install eilmeldung
```

Recommended terminal is [Windows Terminal](https://github.com/microsoft/terminal) with a NerdFont-patched font.

---

## NetBSD

Thanks @0323pin

```bash
pkgin install eilmeldung
```

## Next Steps

After installation, see the [Getting Started Guide](getting-started.md) to set up and configure eilmeldung.
