# neovide-tabs

A lightweight wrapper application for [Neovide](https://neovide.dev) that embeds Neovide instances within a host window (Windows only).

## Overview

neovide-tabs provides a native wrapper window for Neovide by embedding a frameless Neovide window into a host application. The Neovide window automatically fills the wrapper's client area and maintains focus synchronization.

## Features

- **Window Embedding**: Embeds Neovide with `--frame none` for seamless integration
- **Automatic Sizing**: Neovide window fills the wrapper's client area and resizes dynamically
- **Focus Synchronization**: Wrapper window activation automatically focuses the embedded Neovide
- **Graceful Lifecycle**: Clean process management with proper termination on close
- **Neovide Detection**: Validates Neovide installation at startup with helpful error messages
- **Debug Utilities**: `list-windows` command for troubleshooting window detection

## Prerequisites

- [Neovide](https://neovide.dev) must be installed and available in your system PATH
- Rust toolchain (2024 edition or later)

### Installing Neovide

**Windows:**
```bash
# Using winget
winget install Neovide.Neovide

# Using Scoop
scoop install neovide

# Or download from releases
# https://github.com/neovide/neovide/releases
```

**Linux:**
```bash
# Using cargo
cargo install neovide

# Or use your distribution's package manager
```

**macOS:**
```bash
# Using Homebrew
brew install neovide

# Using cargo
cargo install neovide
```

## Getting Started

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/neovide-tabs.git
cd neovide-tabs

# Build the project
cargo build --release

# Run the application
cargo run --release
```

### Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Check code with clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## Usage

### Basic Usage

Launch neovide-tabs from your desired working directory:

```bash
neovide-tabs
```

The application will:
1. Open a wrapper window (1024x768, minimum 800x600)
2. Launch Neovide with frameless window mode (`--frame none`)
3. Position and resize the Neovide window to fill the wrapper's client area
4. Automatically bring Neovide to foreground when the wrapper is activated

### Debug Commands

```bash
# List all windows matching a search term (default: "neovide")
neovide-tabs list-windows [search-term]

# Show help
neovide-tabs help
```

### Closing

- Close the wrapper window normally (Alt+F4, close button, etc.)
- The embedded Neovide process will be gracefully terminated

## Architecture

The application consists of three main modules:

- **main.rs**: Entry point with CLI argument handling and startup validation
- **window.rs**: Win32 window management, message loop, and state handling
- **process.rs**: Neovide process spawning, window discovery, and positioning

## Limitations

- **Windows only**: Currently only supports Windows (uses Win32 API directly)
- **Single instance**: Manages one Neovide instance per wrapper window
- **No configuration**: All settings are currently hardcoded

## Roadmap

- [x] Project setup and architecture
- [x] Basic window wrapper
- [x] Neovide process spawning with `--frame none`
- [x] Window embedding and sizing
- [x] Graceful process lifecycle handling
- [x] Focus synchronization
- [ ] Tab bar UI for multiple instances
- [ ] Tab creation and management
- [ ] Keyboard shortcuts
- [ ] Configurable tab behavior (custom working directories)
- [ ] Persistent tab sessions
- [ ] Cross-platform support (Linux, macOS)

## Contributing

Contributions are welcome! Please read the development guidelines in `AGENTS.md` and `openspec/AGENTS.md` before submitting pull requests.

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Pass all clippy checks (`cargo clippy -- -D warnings`)
- Write tests for new functionality
- Update documentation as needed

## License

[License TBD]

## Acknowledgments

- [Neovide](https://neovide.dev) - The excellent Neovim GUI that this project wraps
- [Neovim](https://neovim.io) - The extensible text editor

## Project Status

**Current Version:** 0.1.0 (Initial Development)

This project is in early development. The current implementation provides a working single-Neovide-instance wrapper with automatic window embedding. The next milestone is implementing the tab bar UI to support multiple Neovide instances.
