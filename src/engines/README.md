# Engines

This directory contains steganography engines for different file formats.

## Current Engines

### PDF Engine (`pdf.rs`)

**Technique**: Appends data after the `%%EOF` marker

- **How it works**: PDF readers typically ignore data after the `%%EOF` marker, so we can safely append Base64-encoded payload data there
- **Detection**: Looks for `%PDF` magic bytes at the start of the file
- **Capacity**: Unlimited (appends to file)
  - File size increases by ~1.33× payload size (Base64 encoding)
- **Visibility**: Data is visible in a hex editor but ignored by PDF readers
- **Format**: `[Base64 Payload]` appended after `%%EOF`
- **Limitations**:
  - Easily detectable
  - Not truly "hidden" - just stored in out-of-bounds

### PNG Engine (`png.rs`)

**Technique**: Custom ancillary chunk steganography

- **How it works**: Adds a custom `lpNg` chunk to the PNG file before the IEND chunk. Payload is Base64-encoded before storage. PNG readers ignore unknown ancillary chunks, so the image displays normally.
- **Detection**: Looks for `\x89PNG\r\n\x1a\n` magic bytes at the start of the file
- **Capacity**: Unlimited (adds to file size, similar to PDF)
  - File size increases by ~1.33× payload size + 12 bytes (Base64 encoding + chunk overhead)
- **Visibility**: Image appears completely normal with zero visual artifacts
- **Format**: `[4 bytes: Length][4 bytes: "lpNg"][N bytes: Base64 Payload][4 bytes: CRC32]`
- **Limitations**:
  - Easily detectable (visible in chunk list and hex editor)
  - Not truly "hidden" - just stored in metadata

## Adding New Engines

1. Create a new file (e.g., `myformat.rs`)
2. Implement the `SteganographyEngine` trait:
   ```rust
   pub trait SteganographyEngine {
       fn magic_bytes(&self) -> &[u8];     // File format signature
       fn format_name(&self) -> &str;       // Human-readable name
       fn format_ext(&self) -> &str;        // File extension
       fn embed(&self, source_data: &[u8], payload: &[u8]) -> Result<Vec<u8>>;
       fn extract(&self, source_data: &[u8]) -> Result<Vec<u8>>;
   }
   ```
3. Add the engine to `mod.rs` exports
4. Register it in `lib.rs` EngineRouter::new()

The EngineRouter will automatically detect the file format based on magic bytes and route to the appropriate engine.
