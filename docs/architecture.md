# Architecture

## Overview

Lupin uses a **modular engine architecture** with **vector-based operations** that automatically detects file formats and applies the appropriate steganography technique:

## Engine System

The core of Lupin is the `SteganographyEngine` trait, which each file format engine implements. The `EngineRouter` manages multiple engines and selects the right one based on magic byte detection.

As such, the entire process works as follows:

1. **Auto-detection**: Lupin matches the file's magic bytes against known engines.
2. **Vector-based processing**: All operations work on byte vectors (`&[u8]`) for performance and flexibility.
3. **Embedding**: Each engine implements format-specific hiding strategies.
4. **Extraction**: Engines know how to recover hidden data from their format.

This design separates I/O operations (CLI layer) from steganography logic (library layer), making the code more testable and the library more flexible.

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ CLI Interface.  │───▶│ Operations       │───▶│ EngineRouter    │
│                 │    │                  │    │                 │
│ embed/extract   │    │ Vector-based     │    │ Auto-detection  │
│ + I/O handling  │    │ + metadata       │    │ Magic bytes     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                        │
                                                        ▼
                                               ┌─────────────────┐
                                               │ Format Engine   │
                                               │   (PDF, etc.)   │
                                               │ Format-specific │
                                               │ embed/extract   │
                                               └─────────────────┘
```

## Project Structure

```
lupin/
├── src/
│   ├── error.rs             # Structured error types using thiserror
│   ├── lib.rs               # Core library and engine router
│   ├── main.rs              # CLI interface with advanced logging
│   ├── operations.rs        # Vector-based embed/extract operations
│   └── engines/
│       ├── mod.rs           # Engine module declarations
│       ├── pdf.rs           # PDF steganography engine
│       ├── png.rs           # PNG steganography engine (ancillary chunks)
│       ├── jpeg.rs          # JPEG steganography engine (APP13 marker)
│       └── README.md        # Guide for adding new engines
├── examples/
│   ├── cat.pdf              # Sample PDF for testing
│   ├── out.pdf              # Sample output PDF after embedding message.txt
│   ├── cat.png              # Sample PNG for testing
│   ├── out.png              # Sample output PNG after embedding message.txt
│   ├── cat.jpg              # Sample JPEG for testing
│   ├── out.jpg              # Sample output JPEG after embedding message.txt
│   └── message.txt          # Sample payload file
├── docs/                    # Documentation
└── .github/workflows/       # CI/CD pipelines
```

## Key Architecture Features

- **Vector-based operations**: Library functions work with `&[u8]` for performance and flexibility.
- **Clean separation**: I/O operations handled in CLI, pure logic in library.
- **Structured errors**: Using `thiserror` for comprehensive error handling.
- **Advanced logging**: Multiple log levels with `simplelog` integration.

## Adding New File Format Support

The modular architecture makes it easy to add support for new file formats:

1. **Create an engine** in `src/engines/yourformat.rs`
2. **Implement the trait**:
   ```rust
   impl SteganographyEngine for YourFormatEngine {
       fn magic_bytes(&self) -> &[u8] { b"MAGIC" }
       fn format_name(&self) -> &str { "YourFormat" }
       fn format_ext(&self) -> &str { ".your" }
       fn embed(&self, source: &[u8], payload: &[u8]) -> Result<Vec<u8>> { ... }
       fn extract(&self, source: &[u8]) -> Result<Vec<u8>> { ... }
   }
   ```
3. **Register the engine** in `EngineRouter::new()` in `lib.rs`

The CLI and detection logic automatically work with new engines!

## Engines

See the [Engines Guide](docs/engines/README.md) for details on existing engines and how to add new ones.
