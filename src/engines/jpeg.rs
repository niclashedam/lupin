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

//! JPEG steganography engine using APP13 application marker
//!
//! # How It Works
//!
//! This engine hides data by adding an APP13 application-specific marker to the JPEG file.
//! JPEG files are organized into segments/markers, and APP markers are specifically
//! designed for application-specific metadata that JPEG readers will safely ignore.
//!
//! ## Storage Format
//!
//! We add a custom APP13 segment containing only the Base64-encoded payload:
//!
//! ```text
//! [0xFF 0xED][2 bytes: Length][N bytes: Base64 Payload]
//! ```
//!
//! - `0xFF 0xED` - JPEG APP13 marker (application-specific data)
//! - Length (2 bytes) - Big-endian length of segment data (including length field itself)
//! - Payload - Base64-encoded data (no signature, just pure data)
//!
//! The APP13 segment is inserted after the SOI (Start of Image) marker and before
//! the actual image data, which is the standard location for application metadata.
//!
//! ## JPEG Segment Structure
//!
//! JPEG files consist of segments (also called markers):
//! - **Marker** (2 bytes): `0xFF` followed by marker type (e.g., `0xD8` for SOI)
//! - **Length** (2 bytes): Big-endian length including the length field itself (but not the marker)
//! - **Data** (N bytes): Segment-specific data
//!
//! Common markers:
//! - `0xFFD8` - SOI (Start of Image) - always first
//! - `0xFFE0-0xFFEF` - APP0-APP15 (application-specific data, like EXIF, XMP, etc.)
//! - `0xFFFE` - COM (comment)
//! - `0xFFDB` - DQT (quantization table)
//! - `0xFFC0` - SOF (start of frame)
//! - `0xFFDA` - SOS (start of scan) - image data follows
//! - `0xFFD9` - EOI (End of Image) - always last
//!
//! ## Why APP13 Marker?
//!
//! - **Designed for metadata** - APP markers are meant for application-specific data
//! - **Invisible to users** - Never displayed by image viewers, only in hex editors
//! - **Safe** - All JPEG readers skip unknown APP markers
//! - **Common** - EXIF (APP1), XMP (APP1), Adobe (APP14) use APP markers
//! - **Stealthy** - Looks like legitimate application metadata
//! - **Standard** - Part of JPEG specification (ISO/IEC 10918-1)
//!

use crate::error::{LupinError, Result};
use crate::SteganographyEngine;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use log::debug;

/// JPEG steganography engine
///
/// Uses APP13 application markers to hide data in JPEG files without modifying image data.
/// Data is Base64-encoded and stored directly in an APP13 segment.
/// This approach is stealthy - the data appears as legitimate application metadata and
/// is never shown to users (unlike comment segments which may be displayed).
///
/// See the module documentation for details on how data is stored.
pub struct JpegEngine;

impl JpegEngine {
    /// Creates a new JPEG engine
    pub fn new() -> Self {
        Self
    }

    /// JPEG Start of Image marker
    const SOI_MARKER: u16 = 0xFFD8;

    /// JPEG End of Image marker
    const EOI_MARKER: u16 = 0xFFD9;

    /// JPEG APP13 marker (application-specific data)
    const APP13_MARKER: u16 = 0xFFED;

    /// JPEG Start of Scan marker (image data follows)
    const SOS_MARKER: u16 = 0xFFDA;

    /// Reads a big-endian u16 from a slice
    fn read_u16_be(data: &[u8]) -> u16 {
        ((data[0] as u16) << 8) | (data[1] as u16)
    }

    /// Writes a big-endian u16 to a vector
    fn write_u16_be(value: u16) -> [u8; 2] {
        [(value >> 8) as u8, value as u8]
    }

    /// Finds the position right after the SOI marker where we can insert our COM segment
    fn find_insert_position(&self, jpeg_data: &[u8]) -> Result<usize> {
        if jpeg_data.len() < 2 {
            return Err(LupinError::JpegInvalidFormat {
                reason: "File too short".to_string(),
            });
        }

        // Check for SOI marker at start
        let soi = Self::read_u16_be(&jpeg_data[0..2]);
        if soi != Self::SOI_MARKER {
            return Err(LupinError::JpegInvalidFormat {
                reason: format!("Missing SOI marker, found 0x{:04X}", soi),
            });
        }

        // Insert after SOI (position 2)
        Ok(2)
    }

    /// Finds an existing Lupin COM segment in the JPEG data
    fn find_lupin_com_segment(&self, jpeg_data: &[u8]) -> Option<(usize, usize)> {
        let mut pos = 2; // Skip SOI marker

        while pos + 4 < jpeg_data.len() {
            // Check if this is a marker (0xFF followed by non-0x00)
            if jpeg_data[pos] != 0xFF {
                break; // Not a marker, we've hit image data
            }

            let marker = Self::read_u16_be(&jpeg_data[pos..pos + 2]);

            // If we hit SOS or EOI, we've gone past the header
            if marker == Self::SOS_MARKER || marker == Self::EOI_MARKER {
                break;
            }

            // Markers without length fields
            if marker == Self::SOI_MARKER || (0xFFD0..=0xFFD7).contains(&marker) {
                pos += 2;
                continue;
            }

            // Read segment length
            if pos + 4 > jpeg_data.len() {
                break;
            }

            let length = Self::read_u16_be(&jpeg_data[pos + 2..pos + 4]) as usize;

            // Check if this is an APP13 segment (we assume any APP13 is ours)
            if marker == Self::APP13_MARKER {
                // Found it! Return start and end positions
                let segment_end = pos + 2 + length;
                return Some((pos, segment_end));
            }

            // Move to next segment
            pos += 2 + length;
        }

        None
    }
}

impl Default for JpegEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SteganographyEngine for JpegEngine {
    fn magic_bytes(&self) -> &[u8] {
        b"\xFF\xD8\xFF" // JPEG SOI + start of next marker
    }

    fn format_name(&self) -> &str {
        "JPEG"
    }

    fn format_ext(&self) -> &str {
        ".jpg"
    }

    fn embed(&self, source_data: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
        // Check if there's already a Lupin APP13 segment
        if let Some((start, end)) = self.find_lupin_com_segment(source_data) {
            debug!(
                "JPEG: Found existing Lupin APP13 segment at {}-{}",
                start, end
            );
            return Err(LupinError::EmbedCollision {
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "JPEG already contains a Lupin APP13 segment",
                ),
            });
        }

        // Find where to insert our APP13 segment (right after SOI)
        let insert_pos = self.find_insert_position(source_data)?;

        debug!("JPEG: Inserting APP13 segment at position {}", insert_pos);

        // Encode payload
        let encoded_payload = BASE64.encode(payload);
        let payload_bytes = encoded_payload.as_bytes();

        // Calculate segment length: length field (2) + payload (no signature)
        let segment_data_length = 2 + payload_bytes.len();

        if segment_data_length > 0xFFFF {
            return Err(LupinError::JpegPayloadTooLarge {
                max_size: 0xFFFF - 2,
                actual_size: payload_bytes.len(),
            });
        }

        // Build the APP13 segment
        let mut app13_segment = Vec::new();
        app13_segment.extend_from_slice(&Self::write_u16_be(Self::APP13_MARKER)); // APP13 marker
        app13_segment.extend_from_slice(&Self::write_u16_be(segment_data_length as u16)); // Length
        app13_segment.extend_from_slice(payload_bytes); // Payload (no signature)

        // Build result: [original up to insert_pos] + [APP13 segment] + [rest of original]
        let mut result = Vec::with_capacity(source_data.len() + app13_segment.len());
        result.extend_from_slice(&source_data[..insert_pos]);
        result.extend_from_slice(&app13_segment);
        result.extend_from_slice(&source_data[insert_pos..]);

        Ok(result)
    }

    fn extract(&self, source_data: &[u8]) -> Result<Vec<u8>> {
        // Find the Lupin APP13 segment
        let (segment_start, segment_end) = self
            .find_lupin_com_segment(source_data)
            .ok_or(LupinError::JpegNoHiddenData)?;

        debug!(
            "JPEG: Found Lupin APP13 segment at {}-{}",
            segment_start, segment_end
        );

        // Extract the payload (skip marker and length - 4 bytes total)
        let payload_start = segment_start + 4;
        let encoded_payload = &source_data[payload_start..segment_end];

        // Decode from Base64
        let decoded =
            BASE64
                .decode(encoded_payload)
                .map_err(|e| LupinError::JpegExtractionFailed {
                    source: Box::new(e),
                })?;

        Ok(decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid JPEG (1x1 pixel, red)
    const MINIMAL_JPEG: &[u8] = &[
        0xFF, 0xD8, // SOI
        0xFF, 0xE0, // APP0 marker
        0x00, 0x10, // Length
        0x4A, 0x46, 0x49, 0x46, 0x00, // "JFIF\0"
        0x01, 0x01, // Version
        0x00, // Density units
        0x00, 0x01, 0x00, 0x01, // X/Y density
        0x00, 0x00, // Thumbnail size
        0xFF, 0xDB, // DQT marker
        0x00, 0x43, // Length
        0x00, // Table ID
        // 64 quantization values (simplified)
        16, 11, 10, 16, 24, 40, 51, 61, 12, 12, 14, 19, 26, 58, 60, 55, 14, 13, 16, 24, 40, 57, 69,
        56, 14, 17, 22, 29, 51, 87, 80, 62, 18, 22, 37, 56, 68, 109, 103, 77, 24, 35, 55, 64, 81,
        104, 113, 92, 49, 64, 78, 87, 103, 121, 120, 101, 72, 92, 95, 98, 112, 100, 103, 99, 0xFF,
        0xC0, // SOF0 marker
        0x00, 0x0B, // Length
        0x08, // Precision
        0x00, 0x01, 0x00, 0x01, // Height x Width
        0x01, // Number of components
        0x01, 0x11, 0x00, // Component info
        0xFF, 0xDA, // SOS marker
        0x00, 0x08, // Length
        0x01, // Number of components
        0x01, 0x00, // Component selector
        0x00, 0x3F, 0x00, // Spectral selection
        0xFF, 0xD9, // EOI
    ];

    #[test]
    fn test_jpeg_magic_bytes() {
        let engine = JpegEngine::new();
        assert_eq!(engine.magic_bytes(), b"\xFF\xD8\xFF");
    }

    #[test]
    fn test_jpeg_format_info() {
        let engine = JpegEngine::new();
        assert_eq!(engine.format_name(), "JPEG");
        assert_eq!(engine.format_ext(), ".jpg");
    }

    #[test]
    fn test_embed_and_extract() {
        let engine = JpegEngine::new();
        let payload = b"Secret message hidden in JPEG!";

        // Embed
        let embedded = engine.embed(MINIMAL_JPEG, payload).unwrap();

        // Should be larger
        assert!(embedded.len() > MINIMAL_JPEG.len());

        // Should still start with JPEG magic bytes
        assert_eq!(&embedded[0..2], &[0xFF, 0xD8]);

        // Extract
        let extracted = engine.extract(&embedded).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_embed_collision() {
        let engine = JpegEngine::new();
        let payload1 = b"First payload";
        let payload2 = b"Second payload";

        // First embed
        let embedded_once = engine.embed(MINIMAL_JPEG, payload1).unwrap();

        // Second embed should fail
        let result = engine.embed(&embedded_once, payload2);
        assert!(matches!(result, Err(LupinError::EmbedCollision { .. })));
    }

    #[test]
    fn test_extract_without_data() {
        let engine = JpegEngine::new();

        // Try to extract from clean JPEG
        let result = engine.extract(MINIMAL_JPEG);
        assert!(matches!(result, Err(LupinError::JpegNoHiddenData)));
    }

    #[test]
    fn test_invalid_jpeg() {
        let engine = JpegEngine::new();
        let not_jpeg = b"This is not a JPEG file";

        let result = engine.embed(not_jpeg, b"payload");
        assert!(matches!(result, Err(LupinError::JpegInvalidFormat { .. })));
    }

    #[test]
    fn test_empty_payload() {
        let engine = JpegEngine::new();
        let payload = b"";

        let embedded = engine.embed(MINIMAL_JPEG, payload).unwrap();
        let extracted = engine.extract(&embedded).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_large_payload() {
        let engine = JpegEngine::new();
        let payload = vec![42u8; 10000]; // 10KB payload

        let embedded = engine.embed(MINIMAL_JPEG, &payload).unwrap();
        let extracted = engine.extract(&embedded).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_unicode_payload() {
        let engine = JpegEngine::new();
        let payload = "Hello, 世界! 🕵️ Lupin steganography".as_bytes();

        let embedded = engine.embed(MINIMAL_JPEG, payload).unwrap();
        let extracted = engine.extract(&embedded).unwrap();
        assert_eq!(extracted, payload);
    }
}
