{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.eilmeldung;
  settingsFormat = pkgs.formats.toml { };
  configFile = settingsFormat.generate "config.toml" cfg.settings;
in {
  meta.maintainers = [ "christo-auer" ];

  options.programs.eilmeldung = {
    enable = mkEnableOption "eilmeldung, a feature-rich TUI RSS reader";

    package = mkOption {
      type = types.package;
      default = pkgs.eilmeldung;
      defaultText = literalExpression "pkgs.eilmeldung";
      description = "The eilmeldung package to use.";
    };

    settings = mkOption {
      type = settingsFormat.type;
      default = { };
      example = literalExpression ''
        {
          refresh_fps = 60;
          article_scope = "unread";
          read_icon = "󰄬";
          unread_icon = "󰄱";
          
          theme = {
            color_palette = {
              background = "#1e1e2e";
              foreground = "#cdd6f4";
              accent_primary = "#f5c2e7";
            };
          };
          
          input_config = {
            scroll_amount = 10;
            mappings = {
              "q" = "quit";
              "j" = "down";
              "k" = "up";
            };
          };
        }
      '';
      description = ''
        Configuration written to {file}`$XDG_CONFIG_HOME/eilmeldung/config.toml`.
        
        See <https://github.com/christo-auer/eilmeldung#configuration-options>
        for the full list of options.
      '';
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile."eilmeldung/config.toml" = mkIf (cfg.settings != { }) {
      source = configFile;
    };
  };
}
