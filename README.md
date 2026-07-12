# 🕵️ Lupin

![CI](https://github.com/niclashedam/lupin/actions/workflows/ci.yml/badge.svg?branch=master)
![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-red.svg)
![Crates.io](https://img.shields.io/crates/v/lupin.svg)

A blazing-fast, lightweight steganography tool for concealing data inside normal-looking files. Lupin can be used as a CLI tool for quick operations or as a Rust library for integration into your applications.

Steganography hides the existence of data rather than just its contents. Where encryption makes a message unreadable, steganography makes it invisible: the file still looks and behaves like an ordinary PDF, PNG, or JPEG.

Lupin is named after [Arsène Lupin](https://en.wikipedia.org/wiki/Ars%C3%A8ne_Lupin), the fictional gentleman thief, for the same reason: hiding something in plain sight.

## Supported Formats

- **PDF**: Appends data after the `%%EOF` marker (unlimited capacity, easily detectable)
- **PNG**: Custom ancillary chunks (unlimited capacity, zero visual artifacts, somewhat easily detectable)
- **JPEG**: Signed APP13 application markers, split across segments as needed (unlimited capacity, zero visual artifacts, somewhat easily detectable)

All three engines currently optimize for **capacity**: unlimited size, but easy to spot with `strings`. The CLI and API carry an `--capacity` / `--stealth` selector for a future low-detectability strategy; no engine implements `--stealth` yet, so requesting it returns a clear error. See the [CLI](docs/cli.md) and [library](docs/library.md) guides.

## Quick Start

### CLI Tool

```bash
# Install
cargo install lupin

# Or build from source: git clone && cargo build --release

# Hide data in PDF
lupin embed document.pdf secret.txt output.pdf

# Extract data
lupin extract output.pdf recovered.txt

# More options
lupin --help
```

More info in the [CLI Guide](docs/cli.md).

### Rust Library

```toml
# Cargo.toml
[dependencies]
lupin = "1.0"
```

```rust
use lupin::operations::{embed, extract};
use lupin::EmbedMode;

// Read files
let source = std::fs::read("document.pdf")?;
let payload = std::fs::read("secret.txt")?;

// Embed with metadata
let (embedded_data, metadata) = embed(&source, &payload, EmbedMode::Capacity)?;
println!("Used {} engine", metadata.engine);

// Extract (mode is autodetected)
let (extracted, info) = extract(&embedded_data)?;
```

More info in the [Library Guide](docs/library.md).

## Documentation

- **[CLI Guide](docs/cli.md)** - Command-line usage, logging, examples
- **[Library Guide](docs/library.md)** - Rust API, integration examples
- **[Architecture](docs/architecture.md)** - How it works, adding new formats

## Contributing

Contributions welcome! Please read the [architecture docs](docs/architecture.md) to understand the codebase structure.

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.

---

**Disclaimer**: For educational and legitimate use only. Users are responsible for complying with applicable laws.
