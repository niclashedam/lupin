# Library Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lupin = "0.2.1"  # Replace with actual version
```

## Quick Start

```rust
use lupin::operations::{embed, extract};

// Read source and payload
let source_data = std::fs::read("document.pdf")?;
let payload_data = std::fs::read("secret.txt")?;

// Embed with rich metadata
let (embedded_data, embed_result) = embed(&source_data, &payload_data)?;
println!("Embedded {} bytes using {} engine",
         embed_result.payload_size, embed_result.engine);

// Save result
std::fs::write("output.pdf", embedded_data)?;

// Extract later
let output_data = std::fs::read("output.pdf")?;
let (extracted_data, extract_result) = extract(&output_data)?;
std::fs::write("recovered.txt", extracted_data)?;
```

## Core Types

### Operations Module

```rust
use lupin::operations::{embed, extract, EmbedResult, ExtractResult};

// Vector-based operations
pub fn embed(source_data: &[u8], payload_data: &[u8]) -> Result<(Vec<u8>, EmbedResult)>
pub fn extract(source_data: &[u8]) -> Result<(Vec<u8>, ExtractResult)>
```

### Result Types

```rust
#[derive(Debug, Clone)]
pub struct EmbedResult {
    pub source_size: usize,    // Original file size
    pub payload_size: usize,   // Hidden data size
    pub output_size: usize,    // Final file size
    pub engine: String,        // Engine used (e.g., "PDF")
}

#[derive(Debug, Clone)]
pub struct ExtractResult {
    pub source_size: usize,    // Source file size
    pub payload_size: usize,   // Extracted data size
    pub engine: String,        // Engine used
}
```

## Benefits of Vector-based API

- **Performance**: No intermediate file I/O in library operations, making it adaptable to various data sources.
- **Flexibility**: Works with data from any source (files, network, memory, etc.).
- **Testability**: Easy to test with in-memory data.
- **Clean separation**: I/O concerns handled by your application.

## Advanced Examples

### Working with Different Data Sources

```rust
use lupin::operations::{embed, extract};

// From network or any byte source
let source_data = download_pdf_from_url("https://example.com/doc.pdf").await?;
let payload_data = b"secret message".to_vec();

// Embed without touching filesystem
let (result, metadata) = embed(&source_data, &payload_data)?;
println!("Output size: {} bytes (+{:.1}% increase)",
         metadata.output_size,
         (metadata.output_size as f64 / metadata.source_size as f64 - 1.0) * 100.0);

// Stream result anywhere
send_to_storage(&result).await?;
```

### Error Handling

```rust
use lupin::error::LupinError;
use lupin::operations::embed;

let source_data = std::fs::read("document.pdf")?;
let payload_data = std::fs::read("secret.txt")?;

match embed(&source_data, &payload_data) {
    Ok((embedded_data, metadata)) => {
        println!("Success! Used {} engine", metadata.engine);
        std::fs::write("output.pdf", embedded_data)?;
    }
    Err(LupinError::UnsupportedFormat) => {
        eprintln!("File format not supported");
    }
    Err(LupinError::PdfNoEofMarker) => {
        eprintln!("Invalid PDF file");
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

### Engine System Direct Access

For more control, you can use the engine system directly:

```rust
use lupin::{EngineRouter, SteganographyEngine};

let router = EngineRouter::new();
let data = std::fs::read("document.pdf")?;

// Auto-detect and get the appropriate engine
let engine = router.detect_engine(&data)?;
println!("Detected format: {}", engine.format_name());

// Use the engine directly
let payload = b"secret data";
let result = engine.embed(&data, payload)?;

// Save the embedded data
std::fs::write("embedded.pdf", result)?;
```

### Testing with In-Memory Data

```rust
#[cfg(test)]
mod tests {
    use lupin::operations::{embed, extract};

    #[test]
    fn test_round_trip() {
        // Create minimal PDF
        let pdf_data = b"%PDF-1.4\n%%EOF".to_vec();
        let payload = b"test payload";

        // Embed
        let (embedded, embed_result) = embed(&pdf_data, payload).unwrap();
        assert_eq!(embed_result.engine, "PDF");
        assert_eq!(embed_result.payload_size, payload.len());

        // Extract
        let (extracted, extract_result) = extract(&embedded).unwrap();
        assert_eq!(extracted, payload);
        assert_eq!(extract_result.payload_size, payload.len());
    }
}
```

## Error Types

The full list of error types can always be found in the `lupin::error` module.

```rust
use lupin::error::LupinError;

// Common error types you'll encounter:
LupinError::UnsupportedFormat          // File format not supported
LupinError::PdfNoEofMarker            // Invalid PDF (no %%EOF)
LupinError::PdfNoHiddenData           // No steganographic data found
LupinError::PdfCorruptedData          // Hidden data is corrupted
LupinError::SourceFileRead { path, source }     // CLI: Can't read source file
LupinError::PayloadFileRead { path, source }    // CLI: Can't read payload file
LupinError::OutputFileWrite { path, source }    // CLI: Can't write output file
```
