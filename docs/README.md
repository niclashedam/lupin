# Lupin Documentation

This directory documents Lupin, a steganography tool for concealing data inside ordinary files (PDF, PNG, JPEG, and MKV).

## Documentation Index

### [CLI Guide](cli.md)
Complete guide for using Lupin as a command-line tool:
- Installation instructions
- Basic usage (embed/extract)
- Logging and verbosity control
- Advanced examples and scripting
- Error handling

### [Library Guide](library.md) 
Complete guide for using Lupin as a Rust library:
- Cargo.toml setup
- Core API and types
- Vector-based operations
- Error handling
- Integration examples
- Performance considerations

### [Architecture](architecture.md)
Technical documentation about Lupin's design:
- Modular engine system
- Vector-based operations
- Project structure
- Adding new file formats
- How the PDF engine works

## Quick Links

If you're new to Lupin, start with the [CLI Guide](cli.md). To integrate it into your own code, see the [Library Guide](library.md). If you're contributing or adding a new file format, read [Architecture](architecture.md) first.

## Tips

- The CLI and library share the same underlying engine system
- Code examples are not run automatically as part of CI, so verify against the current API if something looks off