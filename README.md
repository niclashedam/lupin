# 🕵️ Lupin

![CI](https://github.com/niclashedam/lupin/workflows/CI/badge.svg)
![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

A blazing-fast, lightweight steganography tool for concealing data inside PDF files. Lupin exploits the fact that PDF viewers ignore content after the `%%EOF` marker, allowing arbitrary payloads to be appended to a document without affecting how it is displayed by standard PDF readers.

Written in Rust for performance and safety, Lupin provides a simple command-line interface for embedding and extracting hidden files. It is cross-platform and works on Linux, macOS and Windows.

In an era when privacy is important, Lupin offers a discreet way to conceal sensitive information within ordinary documents. It is named after Arsène Lupin, the fictional gentleman thief — a nod to the project's goal of invisibly hiding data inside ordinary files.

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

# Extract without using Lupin (using standard tools)
tail -n 1 examples/out.pdf | cut -c 6- | base64 -d
```

While extraction can be done with standard tools, we recommend using Lupin to ensure compatibility.

## 🔧 How it works

Lupin exploits a feature of the PDF format: content after the `%%EOF` marker is ignored by PDF readers but remains part of the file. The tool implements two main operations:

1. **Embedding**: finds the last `%%EOF` marker in the PDF and appends base64-encoded payload data directly after it.
2. **Extraction**: locates the `%%EOF` marker and decodes everything after it back to the original binary data.

In short, Lupin uses the PDF as a container for hidden data while preserving the original document so it opens normally in any PDF viewer.

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

The project includes comprehensive tests to ensure reliability:

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test --verbose

# Run a single test
cargo test test_encode_and_extract_payload
```

The CI pipeline also performs integration tests by embedding and extracting data from the included `examples/cat.pdf` file.

## 📁 Project structure

```
lupin/
├── src/
│   ├── main.rs              # CLI interface and argument parsing
│   └── lib.rs               # Core library with modular components
├── examples/
│   ├── cat.pdf              # Sample PDF for testing
│   ├── out.pdf              # Sample output PDF after embedding message.txt
│   └── message.txt          # Sample payload file
└── .github/workflows/       # CI/CD pipelines
```

## 📚 Library usage

You can also use Lupin as a Rust library:

```rust
use lupin::operations::{embed, extract};
use std::path::Path;

fn main() -> std::io::Result<()> {
    // Embed a payload
    embed(
        Path::new("source.pdf"),
        Path::new("payload.bin"),
        Path::new("output.pdf"),
    )?;

    // Extract the payload
    extract(
        Path::new("output.pdf"),
        Path::new("recovered.bin"),
    )?;

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
