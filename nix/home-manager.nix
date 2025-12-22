{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.programs.nlauncher;
  settingsFormat = pkgs.formats.json {};
in
{
  options.programs.nlauncher = {
    enable = mkEnableOption "nlauncher application launcher";

    package = mkOption {
      type = types.package;
      default = pkgs.nlauncher;
      description = "The nlauncher package to use";
    };

    clipboardDaemon = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable clipboard history daemon";
      };
    };

    settings = {
      vaultPath = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "/home/user/vault.kdbx";
        description = "Path to KeePass vault file";
      };
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.configFile."nlauncher/settings.json" = mkIf (cfg.settings.vaultPath != null) {
      source = settingsFormat.generate "nlauncher-settings" {
        vault_path = cfg.settings.vaultPath;
      };
    };

    systemd.user.services.clipboard-daemon = mkIf cfg.clipboardDaemon.enable {
      Unit = {
        Description = "Clipboard history daemon for nlauncher";
        After = [ "graphical-session.target" ];
      };

      Service = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/clipboard-daemon";
        Restart = "on-failure";
        RestartSec = 5;
      };

      Install = {
        WantedBy = [ "default.target" ];
      };
    };
  };
}
