# 🕵️ Lupin

![CI](https://github.com/niclashedam/lupin/actions/workflows/ci.yml/badge.svg?branch=master)
![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

A blazing-fast, lightweight steganography tool with modular engine support for concealing data inside various file formats. Lupin currently supports PDF files and is designed for easy extensibility to other formats.

Written in Rust for performance and safety, Lupin provides a simple command-line interface for embedding and extracting hidden files. It is cross-platform and works on Linux, macOS and Windows.

Named after [Arsène Lupin](https://en.wikipedia.org/wiki/Ars%C3%A8ne_Lupin), the fictional gentleman thief; a nod to the project's goal of invisibly hiding data inside ordinary files.

## 🚀 Quick Start

### Installation

You can build Lupin from source:

```bash
git clone https://github.com/niclashedam/lupin.git
cd lupin
cargo build --release
```

Alternatively, download the latest binary for your platform from the [releases page](https://github.com/niclashedam/lupin/releases).

### Basic usage

```bash
# Hide a secret file inside a PDF
lupin embed document.pdf secret.txt output.pdf

# Extract the hidden file
lupin extract output.pdf recovered_secret.txt

# Extract to stdout (useful for piping)
lupin extract output.pdf -
```

Lupin does not use proprietary formats or secret techniques, so it is possible to extract the hidden data using standard command-line tools. However, it should be noted that some formats may have specific requirements or nuances that make direct extraction more complex. Due to this, we recommend using Lupin to ensure compatibility.

```bash
# Extract from a PDF without using Lupin (using standard tools)
tail -n 1 examples/out.pdf | cut -c 6- | base64 -d
```

## 🔧 How it works

Lupin uses a **modular engine architecture** that automatically detects file formats and applies the appropriate steganography technique:

### Engine System

1. **Auto-detection**: Lupin reads magic bytes to identify the file format
2. **Engine routing**: Routes to the appropriate steganography engine
3. **Embedding**: Each engine implements format-specific hiding strategies
4. **Extraction**: Engines know how to recover hidden data from their format

This design makes adding new file formats straightforward while keeping the CLI interface simple.

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   CLI Interface │───▶│  EngineRouter    │───▶│  Format Engine  │
│                 │    │                  │    │   (PDF, etc.)   │
│ embed/extract   │    │ Auto-detection   │    │ Format-specific │
│    commands     │    │ Magic bytes      │    │ embed/extract   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

The modular design allows developers to easily add new file format support by implementing the `SteganographyEngine` trait.

## 💡 Usage examples

### Hide a text message

```bash
echo "Meet me at the park at 5 pm" > secret.txt
lupin embed document.pdf secret.txt innocent_looking.pdf
```

### Hide an image

```bash
lupin embed report.pdf vacation_photo.jpg boring_report.pdf
```

### Hide an archive

```bash
# Create a zip archive of a folder
zip -r secrets.zip confidential_folder/

# Embed the archive into a PDF
lupin embed presentation.pdf secrets.zip presentation_with_secrets.pdf
```

### Extract and view

```bash
# Extract to a file
lupin extract presentation_with_secrets.pdf extracted_secrets.zip

# Extract and pipe to another command
lupin extract presentation_with_secrets.pdf - | unzip -
```

## 🏗️ Building from source

### Prerequisites

- Rust 1.70 or later
- Cargo (included with Rust)

### Build commands

```bash
# Development build
cargo build

# Optimised release build
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run the linter
cargo clippy
```

## 🧪 Testing

The project includes focused tests for reliability and maintainability:

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test --verbose

# Run PDF engine tests specifically
cargo test engines::pdf

# Test the core router functionality
cargo test test_engine_router
```

## 📁 Project structure

```
lupin/
├── src/
│   ├── error.rs             # Error types
│   ├── file.rs              # Simple file I/O operations
│   ├── lib.rs               # Core traits and engine router
│   ├── main.rs              # CLI interface and argument parsing
│   ├── operations.rs        # High-level embed/extract functions
│   └── engines/
│       ├── mod.rs           # Engine module declarations
│       ├── pdf.rs           # PDF steganography engine
│       └── README.md        # Guide for adding new engines
├── examples/
│   ├── cat.pdf              # Sample PDF for testing
│   ├── out.pdf              # Sample output PDF after embedding message.txt
│   └── message.txt          # Sample payload file
└── .github/workflows/       # CI/CD pipelines
```

### Adding New File Format Support

The modular architecture makes it easy to add support for new file formats:

1. **Create an engine** in `src/engines/yourformat.rs`
2. **Implement the trait**:
   ```rust
   impl SteganographyEngine for YourFormatEngine {
       fn magic_bytes(&self) -> &[u8] { b"MAGIC" }
       fn format_name(&self) -> &str { "YourFormat" }
       fn embed(&self, source: &[u8], payload: &[u8]) -> io::Result<Vec<u8>> { ... }
       fn extract(&self, source: &[u8]) -> io::Result<Vec<u8>> { ... }
   }
   ```
3. **Register the engine** in `EngineRouter::new()` in `lib.rs`

The CLI and detection logic automatically work with new engines!

## 📚 Library usage

You can use Lupin as a Rust library with the clean, simple API:

```rust
use lupin::operations::{embed, extract};
use std::path::Path;

fn main() -> std::io::Result<()> {
    // Embed a payload (auto-detects file format)
    embed(
        Path::new("source.pdf"),    // Source file
        Path::new("payload.bin"),   // Data to hide
        Path::new("output.pdf"),    // Output file
    )?;

    // Extract the payload (auto-detects file format)
    extract(
        Path::new("output.pdf"),     // File with hidden data
        Path::new("recovered.bin"),  // Extracted output
    )?;

    Ok(())
}
```

The library automatically detects file formats and uses the appropriate engine, so your code works with any supported format without changes.

### Advanced Usage

For more control, you can use the engine system directly:

```rust
use lupin::{EngineRouter, SteganographyEngine};

fn main() -> std::io::Result<()> {
    let router = EngineRouter::new();
    let data = std::fs::read("document.pdf")?;

    // Auto-detect and get the appropriate engine
    let engine = router.detect_engine(&data)?;
    println!("Detected format: {}", engine.format_name());

    // Use the engine directly
    let payload = b"secret data";
    let result = engine.embed(&data, payload)?;

    Ok(())
}
```

## 🤝 Contributing

Contributions are welcome. To contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Commit your changes (`git commit -m "Add your feature"`)
4. Push to the branch (`git push origin feature/your-feature`)
5. Open a pull request

### Development guidelines

- Follow Rust formatting conventions (`cargo fmt`)
- Ensure all tests pass (`cargo test`)
- Add tests for new functionality
- Update documentation as appropriate

## 📜 License

This project is licensed under the Apache License 2.0 — see the [LICENSE](LICENSE) file for details.

---

**Disclaimer**: This tool is intended for educational and legitimate use only. Users are responsible for complying with applicable laws and regulations relating to data hiding and steganography.
