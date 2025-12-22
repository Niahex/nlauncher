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
  };
}
