# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **MKV steganography engine** - Adds Matroska (`.mkv`) and WebM support, detected via the EBML header magic bytes. Stores the payload behind a `Lupin\0` signature in an EBML `Void` element appended as the last child inside the Segment. `Void` is the spec's reserved/padding primitive that every conformant reader skips, so media streams and playback are untouched. Because Matroska seek indexes (`SeekHead`/`Cues`) are Segment-relative, appending after all existing children preserves every offset; the Segment size field is rewritten for known-size Segments and left alone for unknown-size ones. New `LupinError::MkvInvalidFormat` and `LupinError::MkvNoHiddenData` variants.
- **Embed mode selector (`--capacity` / `--stealth`)** - `lupin embed` and `operations::embed()` now take an `EmbedMode` that chooses the embedding strategy. `Capacity` (the default) is the existing behavior: unlimited payload size, easily detected by a `strings`/hex-dump pass. `Stealth` is reserved for a future low-detectability strategy; no engine implements it yet, so requesting it returns the new `LupinError::StealthNotSupported { format }` rather than silently falling back to capacity. `EmbedMode` is `#[non_exhaustive]`, so further modes can be added later without a breaking change.

### Changed

- **BREAKING: `embed` now takes an `EmbedMode` argument.** `operations::embed(source, payload)` becomes `operations::embed(source, payload, mode)`, and `SteganographyEngine::embed` gains the same parameter. Pass `EmbedMode::Capacity` to preserve the previous behavior. `operations::extract()` and `SteganographyEngine::extract` are unchanged and detect the payload automatically without being told the mode.

### Supported Formats

- **MKV** - Appends the raw payload behind a `Lupin\0` signature in an EBML `Void` element inside the Segment (unlimited capacity, playback unaffected, somewhat easily detectable); also handles WebM

## [1.1.0] - 2026-07-11

### Added

- **JPEG steganography engine** - Stores the payload behind a `Lupin\0` signature in signed APP13 application markers, inserted after the leading APPn segments (JFIF ordering preserved). Payloads larger than a single ~64 KB segment are split across multiple consecutive APP13 segments for unlimited capacity. Foreign APP13 segments (e.g. Adobe Photoshop / IPTC) are left untouched and never mistaken for hidden data.

### Changed

- `LupinError` is now marked `#[non_exhaustive]`, so future variants can be added without a major version bump. This release still adds `EmptyPayload` (see Fixed below); pin to `1.1` if you match on `LupinError` exhaustively without a wildcard arm.
- Updated dependencies.

### Fixed

- **Empty payloads now rejected at embed** - Embedding an empty payload into a PDF previously produced a file byte-identical to the source, which then reported "no hidden data" on extract, inconsistent with PNG and JPEG. All engines now reject empty payloads with `LupinError::EmptyPayload`.
- **PNG double-embed now rejected** - Embedding into a PNG that already carries a Lupin chunk appended a second `lpNg` chunk, whose payload was silently unrecoverable since extraction only reads the first chunk. This now returns `LupinError::EmbedCollision`, matching the PDF and JPEG engines.
- **Extract success message named the wrong engine** - The message reported "from PDF" regardless of the actual source format; it now reflects the engine used.
- **PNG `format_ext` now includes the leading dot** (`.png`), matching the PDF (`.pdf`) and JPEG (`.jpg`) engines.

### Supported Formats

- **JPEG** - Inserts the raw payload behind a `Lupin\0` signature in one or more APP13 markers (unlimited capacity, zero visual artifacts, somewhat easily detectable)

[1.1.0]: https://github.com/niclashedam/lupin/releases/tag/v1.1.0

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
