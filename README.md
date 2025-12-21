# nlauncher

A fast, modern application launcher built with GPUI and Rust, featuring a beautiful Nord-themed UI with blur effects.

## Features

### 🚀 Core Functionality
- **Fast Application Search**: Fuzzy search powered by [nucleo](https://github.com/helix-editor/nucleo) for instant results
- **Calculator Integration**: Built-in calculator using [numbat](https://numbat.dev/) for quick calculations
- **Process Management**: View and kill running processes with `ps` and `kill` commands
- **Smart Caching**: Application list cached for faster startup times

### 🎨 Modern UI
- **Nord Dark Theme**: Beautiful color palette with Polar Night backgrounds and Frost accents
- **Blur Effects**: Transparent window with blur support (compositor-dependent)
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

### Calculator
Type mathematical expressions directly:
- `2+2` → `4`
- `5*3` → `15`
- `100 km to miles` → Unit conversions

### Process Management
- `ps` → List all running processes
- `kill firefox` → Search and kill processes by name

### Keyboard Shortcuts
- `↑/↓` - Navigate through results
- `Enter` - Launch selected application
- `Backspace` - Delete characters
- `Escape` - Close launcher

## Configuration

### Hyprland Blur
To enable blur effects in Hyprland, add to `~/.config/hypr/hyprland.conf`:

```conf
layerrule = blur, nlauncher
```

## Architecture

### Core Components

- **main.rs**: Application entry point and UI rendering
- **applications.rs**: Desktop entry parsing and caching
- **fuzzy.rs**: Fuzzy search implementation with nucleo
- **calculator.rs**: Mathematical expression evaluation
- **process.rs**: Process listing and management
- **theme.rs**: Nord color palette definitions

### Technologies

- **GPUI**: GPU-accelerated UI framework from Zed
- **Nucleo**: Fast fuzzy matching algorithm
- **Numbat**: Scientific calculator engine
- **freedesktop-desktop-entry**: .desktop file parsing
- **freedesktop-icons**: Icon lookup

## Color Palette (Nord)

- **Polar Night**: `#2e3440` (background), `#3b4252`, `#434c5e`, `#4c566a`
- **Snow Storm**: `#d8dee9`, `#e5e9f0`, `#eceff4` (text)
- **Frost**: `#88c0d0` (accents), `#81a1c1`, `#5e81ac`, `#8fbcbb`

## Sub-projects

This repository contains several related projects:

- **nwidgets**: GPUI-based system widgets (notifications, audio, network)
- **ndownloader**: Download manager with GPUI interface
- **Loungy**: Additional launcher components
- **zed**: Zed editor integration

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
```

## License

MIT License - See LICENSE file for details

## Credits

- Built with [GPUI](https://github.com/zed-industries/zed)
- Inspired by [Loungy](https://github.com/MatthiasGrandl/Loungy)
- Color scheme: [Nord](https://www.nordtheme.com/)
