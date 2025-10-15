# 🕵️ Lupin

![CI](https://github.com/niclashedam/lupin/actions/workflows/ci.yml/badge.svg?branch=master)
![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![Crates.io](https://img.shields.io/crates/v/lupin.svg)

A blazing-fast, lightweight steganography tool for concealing data inside normal-looking files. Lupin can be used as a CLI tool for quick operations or as a Rust library for integration into your applications.

In an era of increasing digital surveillance and diminishing privacy, steganography offers an extra layer of protection by hiding the very existence of sensitive communications. Unlike encryption, which makes data unreadable but obvious, steganography makes data invisible.

Lupin makes this powerful technique accessible through clean, modern tooling.

Lupin is named after [Arsène Lupin](https://en.wikipedia.org/wiki/Ars%C3%A8ne_Lupin), the fictional gentleman thief due to the art of hiding data in plain sight.

## 🚀 Quick Start

### 📱 CLI Tool

```bash
# Install
cargo install lupin

# Or build from source: git clone && cargo build --release

# Hide data
lupin embed document.pdf secret.txt output.pdf

# Extract data  
lupin extract output.pdf recovered.txt

# More options
lupin --help
```

More info in the [CLI Guide](docs/cli.md).

### 📚 Rust Library

```toml
# Cargo.toml
[dependencies]
lupin = "0.2.1"
```

```rust
use lupin::operations::{embed, extract};

// Read files
let source = std::fs::read("document.pdf")?;
let payload = std::fs::read("secret.txt")?;

// Embed with metadata
let (embedded_data, metadata) = embed(&source, &payload)?;
println!("Used {} engine", metadata.engine);

// Extract
let (extracted, info) = extract(&embedded_data)?;
```

More info in the [Library Guide](docs/library.md).

## 📚 Documentation

- **[CLI Guide](docs/cli.md)** - Command-line usage, logging, examples
- **[Library Guide](docs/library.md)** - Rust API, integration examples  
- **[Architecture](docs/architecture.md)** - How it works, adding new formats

## 🤝 Contributing

Contributions welcome! Please read the [architecture docs](docs/architecture.md) to understand the codebase structure.

## 📜 License

Apache License 2.0 — see [LICENSE](LICENSE) for details.

---

**Disclaimer**: For educational and legitimate use only. Users are responsible for complying with applicable laws.