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

### JPEG Engine (`jpeg.rs`)

**Technique**: Signed APP13 application marker segments

- **How it works**: Stores the raw payload behind a `Lupin\0` signature in one or more APP13 (`0xFFED`) segments, inserted after the leading APPn segments (JFIF/EXIF) and before the first non-APP marker. The signature distinguishes Lupin's segments from foreign APP13 data (e.g. Adobe Photoshop/IPTC), so those are left untouched and never mistaken for hidden data. A single segment is capped at ~64 KB by its 16-bit length field; larger payloads are split across multiple consecutive APP13 segments and reassembled on extract.
- **Detection**: Looks for `\xFF\xD8\xFF` magic bytes (SOI followed by the start of the next marker) at the start of the file
- **Capacity**: Unlimited (payload is split across as many APP13 segments as needed)
  - File size increases by payload size plus a small per-segment header (marker + length + `Lupin\0` signature)
- **Visibility**: Image appears completely normal with zero visual artifacts
- **Format**: `[0xFF 0xED][2 bytes: Length][6 bytes: "Lupin\0"][N bytes: Raw Payload]`, repeated per segment for payloads over ~64 KB
- **Limitations**:
  - Easily detectable (visible in segment list and hex editor)
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
