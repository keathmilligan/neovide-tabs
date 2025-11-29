# neovide-tabs

A lightweight tabbed wrapper application for [Neovide](https://neovide.dev) that manages multiple Neovide instances within a single window interface.

## Overview

neovide-tabs provides a native tabbed interface for Neovide by embedding multiple frameless Neovide windows into a single wrapper application. Each tab runs an independent Neovide instance, allowing you to manage multiple editing sessions seamlessly.

## Features

- **Tabbed Interface**: Manage multiple Neovide instances in a single window
- **Seamless Integration**: Uses Neovide's `--frame none` option for clean embedding
- **Automatic Sizing**: Each Neovide instance automatically fills the wrapper's client area
- **Simple Workflow**: New tabs launch Neovide in the current working directory

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
1. Open with a single tab containing a Neovide instance
2. Launch Neovide with frameless window mode (`--frame none`)
3. Set the working directory to the current directory
4. Automatically size and position the embedded window

### Creating New Tabs

- Click the "New Tab" button or use the keyboard shortcut (TBD)
- Each new tab launches a fresh Neovide instance in the current working directory

### Closing Tabs

- Click the close button on a tab or use the keyboard shortcut (TBD)
- The corresponding Neovide instance will be gracefully terminated

## Configuration

_Configuration options are planned for future releases._

## Roadmap

- [x] Project setup and architecture
- [ ] Basic window wrapper with tab bar
- [ ] Neovide process spawning with `--frame none`
- [ ] Window embedding and sizing
- [ ] Tab creation and management
- [ ] Graceful process lifecycle handling
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

This project is in early development. The MVP focuses on basic tab management with automatic window embedding. Future versions will add configuration options and enhanced tab behavior.
