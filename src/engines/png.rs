// Copyright 2025 Niclas Hedam
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! PNG steganography engine using custom ancillary chunks
//!
//! # How It Works
//!
//! This engine hides data by adding a custom ancillary chunk to the PNG file.
//! PNG files are organized into chunks, and the specification allows for custom chunks
//! that will be ignored by standard PNG readers if they don't recognize them.
//!
//! ## Storage Format
//!
//! We add a custom chunk called `lpNg` (Lupin PNG) to the PNG file:
//!
//! ```text
//! [4 bytes: Length][4 bytes: "lpNg"][N bytes: Base64 Payload][4 bytes: CRC32]
//! ```
//!
//! The payload is Base64-encoded before storage to ensure it only contains printable
//! ASCII characters, avoiding any potential issues with binary data in the chunk.
//!
//! The chunk is inserted before the IEND (end) chunk, which is the standard location
//! for ancillary chunks that don't affect image rendering.
//!
//! ## PNG Chunk Structure
//!
//! Each PNG chunk has this format:
//! - **Length** (4 bytes): Big-endian unsigned integer of data length
//! - **Type** (4 bytes): ASCII chunk type code (e.g., "IHDR", "IDAT", "lpNg")
//! - **Data** (N bytes): The actual chunk data
//! - **CRC** (4 bytes): CRC-32 checksum of type and data fields
//!
//! ## Chunk Naming Convention
//!
//! Our chunk type `lpNg` follows PNG naming rules:
//! - Lowercase 'l' = safe to copy (ancillary chunk)
//! - Lowercase 'p' = private chunk (i.e. non standardised)
//! - Uppercase 'N' = to make it a valid chunk type
//! - Lowercase 'g' = safe to copy (i.e. does not affect rendering if unknown)
//! - This makes it `lpNg` which PNG readers will safely ignore
//!

use crate::error::{LupinError, Result};
use crate::SteganographyEngine;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// PNG steganography engine
///
/// Uses custom ancillary chunks to hide data in PNG files without modifying image data.
/// Data is Base64-encoded and stored in a `lpNg` chunk that standard PNG readers will safely ignore.
///
/// See the module documentation for details on how data is stored and limitations.
pub struct PngEngine;

impl PngEngine {
    /// Creates a new PNG engine
    pub fn new() -> Self {
        Self
    }

    /// Custom chunk type for steganography data
    const LUPIN_CHUNK_TYPE: &'static [u8] = b"lpNg";

    /// CRC-32 initial value (all bits set)
    const CRC32_INIT: u32 = 0xFFFFFFFF;

    /// CRC-32 polynomial (reversed, for bitwise operations)
    /// This is the standard CRC-32 polynomial used by PNG (and ZIP, Ethernet, etc.)
    const CRC32_POLYNOMIAL: u32 = 0xEDB88320;

    /// CRC-32 final XOR value (inverts all bits)
    const CRC32_FINAL_XOR: u32 = 0xFFFFFFFF;

    /// Calculates CRC-32 checksum for PNG chunk
    ///
    /// PNG uses CRC-32 (ISO 3309) for chunk integrity.
    /// The CRC is calculated over the chunk type and data fields.
    ///
    /// # Algorithm
    /// Uses the standard CRC-32 algorithm with:
    /// - Initial value: 0xFFFFFFFF
    /// - Polynomial: 0xEDB88320 (reversed)
    /// - Final XOR: 0xFFFFFFFF
    fn calculate_crc(chunk_type: &[u8], data: &[u8]) -> u32 {
        let mut crc = Self::CRC32_INIT;

        // Process chunk type
        for &byte in chunk_type {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ Self::CRC32_POLYNOMIAL;
                } else {
                    crc >>= 1;
                }
            }
        }

        // Process data
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ Self::CRC32_POLYNOMIAL;
                } else {
                    crc >>= 1;
                }
            }
        }

        crc ^ Self::CRC32_FINAL_XOR
    }

    /// Finds the position of the IEND chunk (end of PNG)
    ///
    /// We need to insert our custom chunk before IEND.
    fn find_iend_position(data: &[u8]) -> Result<usize> {
        let mut pos = 8; // Skip PNG signature

        while pos + 8 <= data.len() {
            let chunk_length =
                u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                    as usize;
            let chunk_type = &data[pos + 4..pos + 8];

            if chunk_type == b"IEND" {
                return Ok(pos); // Return position of IEND chunk
            }

            // Move to next chunk: 4 (length) + 4 (type) + data + 4 (CRC)
            pos += 4 + 4 + chunk_length + 4;
        }

        Err(LupinError::PngNoIdatChunk) // Reusing this error for "invalid PNG"
    }

    /// Creates a PNG chunk with the given type and data
    fn create_chunk(chunk_type: &[u8], data: &[u8]) -> Vec<u8> {
        let mut chunk = Vec::new();

        // Length (4 bytes, big-endian)
        chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());

        // Type (4 bytes)
        chunk.extend_from_slice(chunk_type);

        // Data (N bytes)
        chunk.extend_from_slice(data);

        // CRC (4 bytes)
        let crc = Self::calculate_crc(chunk_type, data);
        chunk.extend_from_slice(&crc.to_be_bytes());

        chunk
    }

    /// Extracts data from a custom chunk if it exists
    fn extract_custom_chunk(data: &[u8], chunk_type: &[u8]) -> Result<Vec<u8>> {
        let mut pos = 8; // Skip PNG signature

        while pos + 8 <= data.len() {
            let chunk_length =
                u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                    as usize;
            let current_chunk_type = &data[pos + 4..pos + 8];

            if current_chunk_type == chunk_type {
                // Found our chunk, extract the data
                let data_start = pos + 8;
                let data_end = data_start + chunk_length;

                if data_end + 4 > data.len() {
                    return Err(LupinError::PngNoHiddenData);
                }

                // Verify CRC
                let chunk_data = &data[data_start..data_end];
                let stored_crc = u32::from_be_bytes([
                    data[data_end],
                    data[data_end + 1],
                    data[data_end + 2],
                    data[data_end + 3],
                ]);
                let calculated_crc = Self::calculate_crc(chunk_type, chunk_data);

                if stored_crc != calculated_crc {
                    return Err(LupinError::PngCorruptedData);
                }

                return Ok(chunk_data.to_vec());
            }

            // Move to next chunk
            pos += 4 + 4 + chunk_length + 4;

            // Stop at IEND
            if current_chunk_type == b"IEND" {
                break;
            }
        }

        Err(LupinError::PngNoHiddenData)
    }
}

impl Default for PngEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SteganographyEngine for PngEngine {
    fn magic_bytes(&self) -> &[u8] {
        b"\x89PNG\r\n\x1a\n"
    }

    fn format_name(&self) -> &str {
        "PNG"
    }

    fn format_ext(&self) -> &str {
        "png"
    }

    fn embed(&self, source_data: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
        // Find where to insert our custom chunk (before IEND)
        let iend_pos = Self::find_iend_position(source_data)?;

        // Encode payload as Base64 to avoid any binary issues in the chunk
        let encoded_payload = BASE64.encode(payload);

        // Create our custom steganography chunk with Base64-encoded data
        let steg_chunk = Self::create_chunk(Self::LUPIN_CHUNK_TYPE, encoded_payload.as_bytes());

        // Build the output: original data up to IEND + our chunk + IEND chunk
        let mut output = Vec::with_capacity(source_data.len() + steg_chunk.len());
        output.extend_from_slice(&source_data[..iend_pos]);
        output.extend_from_slice(&steg_chunk);
        output.extend_from_slice(&source_data[iend_pos..]);

        Ok(output)
    }

    fn extract(&self, source_data: &[u8]) -> Result<Vec<u8>> {
        // Extract Base64-encoded data from our custom chunk
        let encoded_data = Self::extract_custom_chunk(source_data, Self::LUPIN_CHUNK_TYPE)?;

        // Decode from Base64
        BASE64
            .decode(&encoded_data)
            .map_err(|_| LupinError::PngCorruptedData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a minimal valid PNG file for testing
    fn create_minimal_png() -> Vec<u8> {
        let mut png = Vec::new();

        // PNG signature
        png.extend_from_slice(b"\x89PNG\r\n\x1a\n");

        // IHDR chunk (13 bytes data)
        png.extend_from_slice(&13u32.to_be_bytes()); // Length
        png.extend_from_slice(b"IHDR"); // Type
        png.extend_from_slice(&10u32.to_be_bytes()); // Width: 10
        png.extend_from_slice(&10u32.to_be_bytes()); // Height: 10
        png.push(8); // Bit depth
        png.push(2); // Color type: RGB
        png.push(0); // Compression
        png.push(0); // Filter
        png.push(0); // Interlace
        png.extend_from_slice(&[0x9a, 0x76, 0x82, 0x70]); // CRC (dummy)

        // IDAT chunk with pixel data (100 bytes for 10x10 RGB = 300 bytes needed, simplified)
        let pixel_data = vec![0u8; 500]; // Plenty of space for testing
        png.extend_from_slice(&(pixel_data.len() as u32).to_be_bytes()); // Length
        png.extend_from_slice(b"IDAT"); // Type
        png.extend_from_slice(&pixel_data);
        png.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // CRC (dummy)

        // IEND chunk
        png.extend_from_slice(&0u32.to_be_bytes()); // Length
        png.extend_from_slice(b"IEND"); // Type
        png.extend_from_slice(&[0xae, 0x42, 0x60, 0x82]); // CRC

        png
    }

    #[test]
    fn test_magic_bytes() {
        // Arrange
        let engine = PngEngine::new();

        // Act
        let magic = engine.magic_bytes();

        // Assert
        assert_eq!(magic, b"\x89PNG\r\n\x1a\n");
    }

    #[test]
    fn test_format_name() {
        // Arrange
        let engine = PngEngine::new();

        // Act
        let name = engine.format_name();

        // Assert
        assert_eq!(name, "PNG");
    }

    #[test]
    fn test_format_ext() {
        // Arrange
        let engine = PngEngine::new();

        // Act
        let ext = engine.format_ext();

        // Assert
        assert_eq!(ext, "png");
    }

    #[test]
    fn test_find_iend_position() {
        // Arrange
        let png_data = create_minimal_png();

        // Act
        let result = PngEngine::find_iend_position(&png_data);

        // Assert
        assert!(result.is_ok());
        let iend_pos = result.unwrap();
        // IEND should be near the end
        assert!(iend_pos > 8, "IEND position should be after PNG signature");
        // Verify it's actually at IEND
        assert_eq!(&png_data[iend_pos + 4..iend_pos + 8], b"IEND");
    }

    #[test]
    fn test_find_iend_no_iend() {
        // Arrange - PNG with only signature and IHDR, no IEND
        let mut png = Vec::new();
        png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&[0u8; 13]); // IHDR data
        png.extend_from_slice(&[0u8; 4]); // CRC

        // Act
        let result = PngEngine::find_iend_position(&png);

        // Assert
        assert!(result.is_err());
        match result {
            Err(LupinError::PngNoIdatChunk) => (), // Reusing this error
            other => panic!("Expected error, got {:?}", other),
        }
    }

    #[test]
    fn test_embed_success() {
        // Arrange
        let engine = PngEngine::new();
        let source = create_minimal_png();
        let payload = b"Hello, PNG steganography!";
        let expected_size = source.len() + BASE64.encode(payload).len() + 12; // payload + 12 byte chunk header

        // Act
        let result = engine.embed(&source, payload);

        // Assert
        assert!(result.is_ok());
        let embedded = result.unwrap();

        assert!(
            expected_size == embedded.len(),
            "Expect output to grow by payload size plus chunk overhead"
        );
        assert!(embedded.starts_with(b"\x89PNG\r\n\x1a\n")); // Still valid PNG
    }

    #[test]
    fn test_embed_and_extract_round_trip() {
        // Arrange
        let engine = PngEngine::new();
        let source = create_minimal_png();
        let payload = b"Secret message hidden in PNG!";

        // Act - Embed
        let embedded = engine
            .embed(&source, payload)
            .expect("Embed should succeed");

        // Act - Extract
        let extracted = engine.extract(&embedded).expect("Extract should succeed");

        // Assert
        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_extract_no_hidden_data() {
        // Arrange
        let engine = PngEngine::new();
        let source = create_minimal_png(); // Fresh PNG with no custom chunk

        // Act
        let result = engine.extract(&source);

        // Assert - Should not find our custom chunk
        assert!(result.is_err(), "Should fail to find hidden data");
        match result {
            Err(LupinError::PngNoHiddenData) => (),
            other => panic!("Expected PngNoHiddenData error, got {:?}", other),
        }
    }

    #[test]
    fn test_round_trip_with_binary_data() {
        // Arrange
        let engine = PngEngine::new();
        let source = create_minimal_png();
        let payload: Vec<u8> = (0..=255).cycle().take(100).collect(); // Binary data

        // Act
        let embedded = engine
            .embed(&source, &payload)
            .expect("Embed should succeed");
        let extracted = engine.extract(&embedded).expect("Extract should succeed");

        // Assert
        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_round_trip_with_empty_payload() {
        // Arrange
        let engine = PngEngine::new();
        let source = create_minimal_png();
        let payload = b"";

        // Act
        let result = engine.embed(&source, payload);

        // Assert - empty payload should succeed in embedding
        assert!(result.is_ok(), "Empty payload should be embeddable");

        if let Ok(embedded) = result {
            let extracted = engine.extract(&embedded).expect("Extract should succeed");
            assert_eq!(extracted, payload, "Extracted empty payload should match");
        }
    }

    #[test]
    fn test_crc_calculation() {
        // Arrange & Act
        let crc = PngEngine::calculate_crc(b"IEND", &[]);

        // Assert - Known CRC for IEND chunk with no data
        assert_eq!(crc, 0xae426082);
    }
}
