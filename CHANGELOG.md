# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-10-24

### Added

- **PNG steganography engine** - Uses custom ancillary chunks (`lpNg`) with Base64 encoding
- **Flexible logging** - Support for `--verbose`, `--quiet`, and `--log-level` flags
- **Comprehensive error handling** - Rich error types with context for debugging
- **Full test suite** - 35 tests covering PDF, PNG, operations, and engine detection
- **Complete documentation** - README, CLI guide, library guide, and architecture docs

### Supported Formats

- **PDF** - Appends Base64-encoded data after `%%EOF` marker (unlimited capacity, easily detectable)
- **PNG** - Inserts Base64-encoded data in custom `lpNg` ancillary chunk before IEND (unlimited capacity, zero visual artifacts, somewhat easily detectable)

### Technical Details

- Rust 2021 edition
- Minimum Rust version: 1.70
- Dependencies: clap 4.5, base64 0.22, thiserror 2, log 0.4, simplelog 0.12
- Cross-platform: Linux, macOS, Windows (x86_64 and ARM64)
- Licensed under Apache 2.0

### API Stability

This is the first stable release. The public API is now considered stable:

- `operations::embed()` and `operations::extract()` functions
- `SteganographyEngine` trait for custom engines
- `EngineRouter` for format detection
- `LupinError` enum and `Result` type alias
- CLI commands and arguments

All public APIs will follow semantic versioning going forward.

[1.0.0]: https://github.com/niclashedam/lupin/releases/tag/v1.0.0

## [0.2.1] - 2025-10-13

### Added

- **Lower minimum Rust version** to 1.48 for broader compatibility

### Supported Formats

- **PDF** - Appends Base64-encoded data after `%%EOF` marker (unlimited capacity, easily detectable)

### Technical Details

- Rust 2018 edition
- Minimum Rust version: 1.48
- Dependencies: base64 0.22
- Cross-platform: Linux, macOS, Windows (x86_64 and ARM64)
- Licensed under Apache 2.0

### API Stability

Unstable.

[0.2.1]: https://github.com/niclashedam/lupin/releases/tag/v0.2.1

## [0.2.0] - 2025-10-12

### Added

- **PDF steganography engine** - Embeds data after the `%%EOF` marker with Base64 encoding
- **Automatic format detection** - Detects file type via magic bytes and routes to appropriate engine
- **CLI tool** with embed and extract commands
- **Stdout extraction** - Use `-` as output path to pipe extracted data
- **Library API** - Simple `embed()` and `extract()` functions that work on byte vectors
- **Result metadata** - `EmbedResult` and `ExtractResult` provide operation information
- **CI/CD workflows** - Automated testing, building, and release process

### Supported Formats

- **PDF** - Appends Base64-encoded data after `%%EOF` marker (unlimited capacity, easily detectable)

### Technical Details

- Rust 2024 edition
- Minimum Rust version: 1.70
- Dependencies: base64 0.21
- Cross-platform: Linux, macOS, Windows (x86_64 and ARM64)
- Licensed under Apache 2.0

### API Stability

Unstable.

[0.2.0]: https://github.com/niclashedam/lupin/releases/tag/v0.2.0
