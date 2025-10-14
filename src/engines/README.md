# Engines

This directory contains steganography engines for different file formats.

## Current Engines

- **PDF** (`pdf.rs`) - Hides data after the `%%EOF` marker

## Adding New Engines

1. Create a new file (e.g., `png.rs`)
2. Implement the `SteganographyEngine` trait
3. Add the engine to `mod.rs` 
4. Register it in `lib.rs` EngineRouter

Each engine defines magic bytes for file detection and implements embed/extract logic.