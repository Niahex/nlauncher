# Improvements TODO

## 🐛 Bug Fixes

### High Priority
- [ ] **Fix vault overflow**: Vault entries overflow beyond the container, need proper scrolling/clipping
- [ ] **Verify clipboard daemon**: Test that clipboard daemon works correctly with the main app
- [ ] **Fix KeePassXC integration**: Vault functionality is currently broken, needs debugging

### Medium Priority
- [ ] **Consistent item heights**: All result items should have the same height (applications, processes, vault entries, clipboard)

## ✨ Features

### UI/UX Improvements
- [ ] **Replace text commands with icons**: Use icons instead of text for `ps`, `pass`, `=`, `clip` commands
- [ ] **Add status bar**: Bottom bar showing available actions and keyboard shortcuts
  - Show current mode (search/process/vault/clipboard/calculator)
  - Display available shortcuts (↑↓ navigate, Enter select, Esc close, etc.)
  - Show item count (e.g., "5 results")

### Configuration
- [ ] **Configurable theming via Nix**: Allow users to customize colors through Home Manager
  ```nix
  programs.nlauncher = {
    theme = {
      background = "#2e3440";
      accent = "#88c0d0";
      # ... other colors
    };
  };
  ```

### New Commands
- [ ] **System commands (`sys`)**: Power management and system control
  - `sys shutdown` - Shutdown immediately
  - `sys shutdown 10m` - Shutdown in 10 minutes
  - `sys shutdown 30s` - Shutdown in 30 seconds
  - `sys shutdown 1h` - Shutdown in 1 hour
  - `sys reboot` - Reboot system
  - `sys sleep` - Suspend to RAM
  - `sys hibernate` - Hibernate to disk
  - `sys lock` - Lock screen
  - `sys logout` - Logout current session

## 🎨 Design
- [ ] Improve visual hierarchy
- [ ] Add subtle animations for state changes

## 🔧 Technical Debt
- [ ] Add proper error handling for all commands
- [ ] Add logging system
- [ ] Write unit tests for core functionality
- [ ] Add integration tests
- [ ] Cachix https://app.cachix.org/cache/nlauncher#pull

## 📝 Documentation
- [ ] Add screenshots and video/gif to README
- [ ] Create user guide
- [ ] Document all keyboard shortcuts
- [ ] Add troubleshooting section
