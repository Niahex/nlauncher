# nlauncher

A fast, modern application launcher built with GPUI and Rust, featuring a beautiful Nord-themed UI.

## Features

### 🚀 Core Functionality
- **Fast Application Search**: Fuzzy search powered by [nucleo](https://github.com/helix-editor/nucleo) for instant results
- **Calculator Integration**: Built-in calculator using [numbat](https://numbat.dev/) - prefix with `=` for calculations
- **Process Management**: View and kill running processes with `ps` command
- **Clipboard History**: Access clipboard history with `clip` command (requires daemon)
- **Password Manager**: KeePassXC integration with `pass` command via Secret Service
- **Smart Caching**: Application list cached for faster startup times

### 🎨 Modern UI
- **Nord Dark Theme**: Beautiful color palette with centralized theme system
- **Command Highlighting**: Visual feedback with color-coded command prefixes
  - `ps` - Red background for process commands
  - `pass` - Yellow background for password manager
  - `=` - Green background for calculator
  - `clip` - Blue background for clipboard
- **Transparent Window**: Clean, modern appearance with transparency support
- **Smooth Animations**: Hover effects and smooth transitions
- **Keyboard Navigation**: Full keyboard control with arrow keys and Enter

### 🖥️ Wayland Support
- Native Wayland support via GPUI
- Layer shell integration for proper window management
- Exclusive keyboard focus when active

## Installation

### Using Nix (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/nlauncher.git
cd nlauncher

# Enter development shell
nix develop

# Build and run
cargo run --release
```

### Home Manager Module

Add to your Home Manager configuration:

```nix
{
  programs.nlauncher = {
    enable = true;
    
    # Optional: Enable clipboard history daemon
    clipboardDaemon.enable = true;
    
    # Optional: Configure KeePassXC vault path
    settings.vaultPath = "/path/to/vault.kdbx";
  };
}
```

### Manual Build

Requirements:
- Rust 1.88.0+
- Wayland development libraries
- Vulkan support

```bash
cargo build --release
./target/release/nlauncher
```

## Usage

### Launching Applications
Simply start typing the name of an application. The fuzzy search will find matches instantly.

### Calculator (`=`)
Type `=` followed by mathematical expressions:
- `= 2+2` → `4`
- `= 5*3` → `15`
- `= 100 km to miles` → Unit conversions

### Process Management (`ps`)
- `ps` → List all running processes
- `psfirefox` → Search and kill processes by name (no space needed)

### Password Manager (`pass`)
Requires KeePassXC with Secret Service integration enabled:
- `pass<password>` → Unlock vault
- `pass<search>` → Search entries
- `Ctrl+C` → Copy password
- `Ctrl+B` → Copy username
- `Ctrl+T` → Copy TOTP (if available)

### Clipboard History (`clip`)
Requires clipboard daemon to be running:
- `clip` → Show clipboard history
- `clip<search>` → Filter clipboard entries
- `Enter` → Restore selected entry

### Keyboard Shortcuts
- `↑/↓` - Navigate through results
- `Enter` - Launch selected application or execute command
- `Backspace` - Delete characters
- `Escape` - Close launcher

## Architecture

### Core Components

- **main.rs**: Application entry point and UI rendering
- **applications.rs**: Desktop entry parsing and caching
- **fuzzy.rs**: Fuzzy search implementation with nucleo
- **calculator.rs**: Mathematical expression evaluation
- **process.rs**: Process listing and management
- **clipboard.rs**: Clipboard history management
- **vault.rs**: KeePassXC integration via Secret Service
- **theme.rs**: Nord color palette and theme system

### Technologies

- **GPUI**: GPU-accelerated UI framework from Zed
- **Nucleo**: Fast fuzzy matching algorithm
- **Numbat**: Scientific calculator engine
- **freedesktop-desktop-entry**: .desktop file parsing
- **freedesktop-icons**: Icon lookup
- **secret-service**: KeePassXC integration

## Performance Optimizations

- **Task Cancellation**: Previous searches are cancelled when typing fast
- **Centralized Theme**: All colors managed through a single theme system
- **Efficient Rendering**: Only visible items are rendered (manual virtualization)
- **Smart Caching**: Application list cached to disk for instant startup

## Color Palette (Nord)

- **Polar Night**: `#2e3440` (background), `#3b4252`, `#434c5e`, `#4c566a`
- **Snow Storm**: `#d8dee9`, `#e5e9f0`, `#eceff4` (text)
- **Frost**: `#88c0d0` (accents), `#81a1c1`, `#5e81ac`, `#8fbcbb`
- **Aurora**: `#bf616a` (red), `#d08770` (orange), `#ebcb8b` (yellow), `#a3be8c` (green)

## Development

```bash
# Enter development environment
nix develop

# Run in debug mode
cargo run

# Build release
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt
```

## Daemons

### Clipboard Daemon

The clipboard daemon monitors clipboard changes and stores history:

```bash
# Run manually
cargo run --bin clipboard-daemon

# Or via systemd (with Home Manager)
systemctl --user start clipboard-daemon
```

History is stored in `$XDG_CACHE_HOME/nlauncher/clipboard_history.txt` (max 100 entries).

## License

MIT License - See LICENSE file for details

## Credits

- Built with [GPUI](https://github.com/zed-industries/zed)
- Inspired by [Loungy](https://github.com/MatthiasGrandl/Loungy)
- Color scheme: [Nord](https://www.nordtheme.com/)
