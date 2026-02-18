# Installation

Follow any of the installation methods below, then run *eilmeldung*. It will guide you through the setup process.

---

## Table of Contents

- [Important: Nerd Fonts](#important-nerd-fonts)
- [Via Homebrew](#via-homebrew)
- [Via AUR (Arch)](#via-aur-arch)
- [Via Cargo](#via-cargo)
- [Nix Flake and Home Manager](#nix-flake-and-home-manager)

---

## Important: Nerd Fonts

You need a [Nerd Font](https://github.com/ryanoasis/nerd-fonts) compatible font/terminal for icons to display correctly!

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
| Fedora       | `# dnf install cargo rust perl libxml2-devel openssl-devel clang sqlite-devel`                           |
| Arch         | `# pacman -S cargo base-devel clang perl libxml2 openssl libsixel sqlite3`                             |

To compile the latest unreleased version (`HEAD` in `main`):
```bash
cargo install --locked --git https://github.com/christo-auer/eilmeldung
```
and for the latest tag:
```bash
cargo install --locked --git https://github.com/christo-auer/eilmeldung/tree/0.9.4
```

---

## Nix Flake and Home Manager

Add *eilmeldung* to your inputs, apply `eilmeldung.overlays.default` overlay to `pkgs`. If you want Home Manager integration, add Home Manager module `eilmeldung.homeManager.default`. Here is an example:

```nix
{
  inputs = {
    // ...
    eilmeldung.url = "github:christo-auer/eilmeldung";
  };

  outputs = { nixpkgs, home-manager, eilmeldung, ... }: {
    homeConfigurations."..." = home-manager.lib.homeManagerConfiguration {
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ eilmeldung.overlays.default ];
      };
      
      modules = [
        // ...
        eilmeldung.homeManager.default
      ];
    };
  };
}
```

Home Manager configuration works by defining the settings from the configuration file:

```nix
programs.eilmeldung = {
  enable = true;

  settings = {
    refresh_fps = 60;
    article_scope = "unread";


    theme = {
      color_palette = {
        background = "#1e1e2e";
        // ...
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

---

## Next Steps

After installation, see the [Getting Started Guide](getting-started.md) to set up and configure eilmeldung.
