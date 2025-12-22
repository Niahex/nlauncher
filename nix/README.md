# nlauncher NixOS Configuration

## Usage

Add to your home-manager configuration:

```nix
{
  inputs.nlauncher.url = "github:yourusername/nlauncher";
  
  # In your home-manager configuration:
  imports = [ inputs.nlauncher.homeManagerModules.default ];
  
  programs.nlauncher = {
    enable = true;
    settings = {
      vaultPath = "/home/user/vault.kdbx";
    };
  };
}
```

This will:
- Install nlauncher
- Create `~/.config/nlauncher/settings.json` with your vault path
