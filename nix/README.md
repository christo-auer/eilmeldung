# Nix Installation Guide

This directory contains Nix configuration files for building and installing eilmeldung.

## Quick Start

### Try Without Installing

```bash
nix run gitlab:christo-auer/eilmeldung
```

### Install System-Wide

```bash
nix profile install gitlab:christo-auer/eilmeldung
```

### Development

```bash
nix develop
cargo build
cargo run
```

## Home Manager Integration

eilmeldung provides a Home Manager module for declarative configuration.

### Setup

Add eilmeldung as an input to your Home Manager flake:

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    eilmeldung.url = "gitlab:christo-auer/eilmeldung";
  };

  outputs = { nixpkgs, home-manager, eilmeldung, ... }: {
    homeConfigurations."youruser" = home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      
      modules = [
        eilmeldung.homeManagerModules.default
        ./home.nix
      ];
    };
  };
}
```

### Usage

Then in your `home.nix`:

```nix
{
  programs.eilmeldung = {
    enable = true;
    
    settings = {
      # All settings from config.toml can be configured here
      refresh_fps = 60;
      article_scope = "unread";
      
      theme = {
        color_palette = {
          background = "#1e1e2e";
          foreground = "#cdd6f4";
          accent_primary = "#f5c2e7";
        };
      };
      
      input_config = {
        mappings = {
          "q" = "quit";
          "j" = "down";
          "k" = "up";
        };
      };
    };
  };
}
```

See [home-manager-example.nix](./home-manager-example.nix) for a complete example.

## File Structure

- **`package.nix`** - Build recipe for the eilmeldung binary
- **`shell.nix`** - Development environment with Rust toolchain
- **`home-manager-module.nix`** - Home Manager module definition
- **`home-manager-example.nix`** - Complete usage example

## Configuration

When using the Home Manager module, your configuration is:

- **Generated from**: `programs.eilmeldung.settings` in Nix
- **Written to**: `~/.config/eilmeldung/config.toml`
- **Format**: TOML (automatically converted from Nix)

All configuration options from the main README are supported.

## Alternative Installation Methods

### Just the Package (Manual Config)

If you prefer to manage `config.toml` manually:

```nix
{
  home.packages = [ eilmeldung.packages.${pkgs.system}.default ];
  
  # Optional: manage config file separately
  xdg.configFile."eilmeldung/config.toml".source = ./my-config.toml;
}
```

### Overlays

To override the package version:

```nix
{
  programs.eilmeldung = {
    enable = true;
    package = eilmeldung.packages.${pkgs.system}.default.overrideAttrs (old: {
      # Your overrides here
    });
  };
}
```
