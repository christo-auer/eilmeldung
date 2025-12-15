# Example: How to use eilmeldung in Home Manager
#
# Add this flake as an input to your Home Manager configuration:

# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    eilmeldung.url = "https://github.com/christo-auer/eilmeldung";
  };

  outputs = { nixpkgs, home-manager, eilmeldung, ... }: {
    homeConfigurations."youruser" = home-manager.lib.homeManagerConfiguration {
      # Apply the overlay so pkgs.eilmeldung is available
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ eilmeldung.overlays.default ];
      };
      
      modules = [
        # Import the eilmeldung Home Manager module
        eilmeldung.homeManagerModules.default
        
        # Your home configuration
        ./home.nix
      ];
    };
  };
}

# home.nix
{ config, pkgs, ... }:

{
  # Method 1: Use the Home Manager module (Recommended)
  programs.eilmeldung = {
    enable = true;
    
    # The package comes from pkgs.eilmeldung (via overlay)
    # Optional: Override if needed
    # package = pkgs.eilmeldung.override { ... };
    
    # Optional: Configure via Nix instead of manually editing config.toml
    settings = {
      refresh_fps = 60;
      article_scope = "unread";
      
      # Icons (require Nerd Fonts)
      read_icon = "󰄬";
      unread_icon = "󰄱";
      marked_icon = "";
      unmarked_icon = "";
      
      # Theme configuration
      theme = {
        color_palette = {
          background = "#1e1e2e";
          foreground = "#cdd6f4";
          accent_primary = "#f5c2e7";
          accent_secondary = "#89b4fa";
          accent_tertiary = "#94e2d5";
        };
      };
      
      # Keybindings
      input_config = {
        scroll_amount = 10;
        timeout_millis = 5000;
        mappings = {
          "q" = "quit";
          "j" = "down";
          "k" = "up";
          "g g" = "gotofirst";
          "G" = "gotolast";
          "o" = ["open" "read" "nextunread"];
        };
      };
      
      # Feed list configuration
      feed_list = [
        "query: \"Today Unread\" today unread"
        "query: \"Today Marked\" today marked"
        "feeds"
        "* categories"
        "tags"
      ];
    };
  };

  # Method 2: Just install the package without the module
  # (if you prefer to manage config.toml manually)
  # home.packages = [ eilmeldung.packages.${pkgs.system}.default ];
  
  # Method 3: Install and provide custom config file
  # home.packages = [ eilmeldung.packages.${pkgs.system}.default ];
  # xdg.configFile."eilmeldung/config.toml".source = ./eilmeldung-config.toml;
}
