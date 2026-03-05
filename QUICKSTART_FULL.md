# Quick Start Guide - NoteCypher Desktop App

## Running the Application

### Option 1: Quick Run (Development)
```bash
cd rust
cargo run
```

### Option 2: Build and Run (Faster)
```bash
cd rust
cargo build --release
./target/release/notecypher    # On Windows: target\release\notecypher.exe
```

### Option 3: Use Build Script (Windows)
```bash
# From the main project directory
build_rust.bat
```

## First Time Build

The first build will take longer (5-15 minutes) as it compiles all dependencies. Subsequent builds will be much faster.

### Expected First Build Time
- **Windows**: 10-15 minutes
- **Linux**: 8-12 minutes  
- **macOS**: 10-15 minutes

### Subsequent Build Time
- **Debug**: 30 seconds - 2 minutes
- **Release**: 1-3 minutes

## Troubleshooting

### "Rust not found"
Install Rust from [rustup.rs](https://rustup.rs)

### Build fails on Linux
Install required development libraries:
```bash
# Ubuntu/Debian
sudo apt-get install pkg-config libx11-dev libasound2-dev libudev-dev

# Fedora
sudo dnf install libX11-devel alsa-lib-devel systemd-devel
```

### Build fails on Windows
Make sure you have Visual Studio Build Tools with C++ support installed.

### Application won't start
Try running from command line to see error messages:
```bash
cd rust
cargo run
```

## System Requirements

- **OS**: Windows 10+, macOS 10.15+, or Linux (any modern distro)
- **RAM**: 2GB minimum, 4GB recommended
- **Disk**: 200MB for the application
- **Display**: 1280x720 minimum resolution

## Features Overview

1. **Upload PDFs**: Click the upload area or press `Ctrl+O`
2. **Select Pages**: Click thumbnails to choose pages
3. **Apply Filters**: Invert, Clear Background, Grayscale
4. **Set Layout**: 1-6 slides per page
5. **Export**: Download your optimized PDF

## Keyboard Shortcuts

- `Ctrl+O` - Open file dialog
- `Ctrl+Q` - Quit application (Linux/Windows)
- `Cmd+Q` - Quit application (macOS)

## Getting Help

- Check the main [README.md](../README.md)
- View the full [Documentation](README.md)
- Report issues on GitHub
