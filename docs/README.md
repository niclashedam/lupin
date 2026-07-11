# Lupin Documentation

This directory contains comprehensive documentation for Lupin, a steganography tool for concealing data inside ordinary files (PDF, PNG, and JPEG).

## 📖 Documentation Index

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

## 🚀 Quick Links

**New to Lupin?** Start with the [CLI Guide](cli.md) for immediate usage.

**Integrating into code?** Check the [Library Guide](library.md) for API examples.

**Contributing or extending?** Read the [Architecture](architecture.md) docs first.

## 💡 Tips

- The CLI and library share the same underlying engine system
- Code examples are not run automatically as part of CI, so verify against the current API if something looks off