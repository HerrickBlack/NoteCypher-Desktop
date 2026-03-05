# NoteCypher - Native Rust Desktop App

A native desktop PDF note optimizer built with Rust and Iced framework. This is a complete rewrite of the original React web app as a cross-platform native desktop application.

## Features

- **Multiple PDF Upload** - Select multiple PDF files using native file dialogs
- **Interactive Thumbnails** - Click to select/deselect individual pages or entire PDFs
- **Filters**:
  - 🌓 **Invert Colors** - Convert dark-themed slides to print-friendly white background
  - ✨ **Clear Background** - Remove gray tints, yellowing, or scanner noise
  - ⚫ **Grayscale** - Convert to pure grayscale for ink saving
- **Flexible Grid Layout** - Arrange 1, 2, 3, 4, or 6 slides per page
- **Custom Options** - Portrait/Landscape orientation, adjustable margins (0-5cm)
- **Dark Mode** - Toggle between light and dark themes, persists across sessions
- **100% Offline** - No internet connection required, all processing happens locally

## Technology Stack

- **GUI Framework**: [Iced](https://github.com/iced-rs/iced) - Cross-platform GUI library
- **PDF Processing**: [lopdf](https://github.com/J-F-Liu/lopdf) - PDF manipulation
- **Image Processing**: [image](https://github.com/image-rs/image) - Image processing crate
- **Async Runtime**: [Tokio](https://tokio.rs/) - Async runtime for background processing
- **File Dialogs**: [rfd](https://github.com/PolyMeilex/rfd) - Native file dialogs

## Building

### Prerequisites

- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs))
- For Windows: Visual Studio Build Tools with C++ support
- For Linux: Development libraries for X11, Wayland, or your display server

### Build Commands

```bash
# Navigate to the Rust project directory
cd rust

# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run in development mode
cargo run
```

## Project Structure

```
rust/
├── Cargo.toml              # Project dependencies and metadata
├── src/
│   ├── main.rs             # Application entry point and UI
│   ├── pdf_processor.rs    # PDF processing and export logic
│   └── theme_style.rs      # Custom theme styling
└── target/                 # Build output (generated)
```

## Cross-Platform Compilation

### Windows
```bash
cargo build --release
# Output: target/release/notecypher.exe
```

### Linux
```bash
cargo build --release
# Output: target/release/notecypher
```

### macOS
```bash
cargo build --release
# Output: target/release/notecypher
```

## Usage

1. **Launch the application**
   - Run `cargo run` in development or execute the compiled binary

2. **Upload PDFs**
   - Click the upload area or press `Ctrl+O` to select PDF files
   - Multiple files can be selected at once

3. **Select Pages**
   - Click individual page thumbnails to select/deselect
   - Click PDF headers to toggle all pages from that file
   - Use "Select All" / "Deselect All" buttons

4. **Apply Filters**
   - Enable Invert, Clear Background, or Grayscale as needed

5. **Configure Layout**
   - Choose slides per page (1, 2, 3, 4, or 6)
   - Select Portrait or Landscape orientation
   - Set margins (0-5 cm)

6. **Export**
   - Click "Download Optimized PDF"
   - Choose save location in the file dialog

## Keyboard Shortcuts

- `Ctrl+O` - Open file dialog to select PDFs

## Differences from Web Version

- **Native Performance**: Faster PDF processing using native code
- **Offline First**: No internet dependency, works completely offline
- **Native File System**: Direct file access without browser sandbox restrictions
- **System Theme Integration**: Follows system dark/light mode preferences
- **No CDN Dependencies**: All libraries compiled into the binary

## Development Status

This is a complete rewrite of the NoteCypher web application. The core features are implemented:

- ✅ PDF file loading
- ✅ Page thumbnail generation
- ✅ Page selection system
- ✅ Image filters (invert, clear background, grayscale)
- ✅ Grid layout (N-up)
- ✅ Orientation and margin settings
- ✅ PDF export
- ✅ Dark mode with persistence
- ✅ Cross-platform support

## License

MIT License - Same as the original web application

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## Acknowledgments

- Original NoteCypher web app built with React
- Iced framework and community
- All Rust crate authors whose libraries made this possible
