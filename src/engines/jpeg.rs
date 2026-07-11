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
//! We add a custom APP13 segment prefixed with a Lupin signature so it can be
//! reliably distinguished from foreign APP13 segments (e.g. Adobe Photoshop /
//! IPTC metadata, which also live in APP13):
//!
//! ```text
//! [0xFF 0xED][2 bytes: Length][6 bytes: "Lupin\0"][N bytes: Raw Payload]
//! ```
//!
//! - `0xFF 0xED` - JPEG APP13 marker (application-specific data)
//! - Length (2 bytes) - Big-endian length of segment data (including length field itself)
//! - Signature - `Lupin\0`, identifies the segment as ours
//! - Payload - Raw payload bytes (APP segment data is length-delimited, so no
//!   encoding or byte-stuffing is required)
//!
//! A single APP13 segment's length field is a 16-bit value, capping one segment
//! at ~64 KB. To support payloads of any size, larger payloads are split across
//! several consecutive APP13 segments (each carrying the signature); on extract
//! every Lupin segment is found in file order and their chunks are concatenated.
//! This is the same technique JPEG itself uses for large ICC profiles.
//!
//! The APP13 segment(s) are inserted after the leading APPn segments (after the
//! JFIF/EXIF headers and before the first non-APP marker such as DQT/SOF), which
//! keeps the file conformant with the JFIF ordering requirement.
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
use log::debug;

/// JPEG steganography engine
///
/// Uses APP13 application markers to hide data in JPEG files without modifying image data.
/// The payload is stored raw behind a Lupin signature in an APP13 segment.
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

    /// Signature prefixing the payload inside our APP13 segment.
    ///
    /// APP13 is also used by third-party tools (notably Adobe Photoshop / IPTC),
    /// so this signature is required to tell a Lupin segment apart from a foreign
    /// one when finding, extracting or checking for collisions.
    const LUPIN_SIGNATURE: &'static [u8] = b"Lupin\0";

    /// Reads a big-endian u16 from a slice
    fn read_u16_be(data: &[u8]) -> u16 {
        ((data[0] as u16) << 8) | (data[1] as u16)
    }

    /// Writes a big-endian u16 to a vector
    fn write_u16_be(value: u16) -> [u8; 2] {
        [(value >> 8) as u8, value as u8]
    }

    /// Finds the position where we can insert our APP13 segment.
    ///
    /// The segment is inserted after any leading APPn segments (JFIF/EXIF/etc.)
    /// and before the first non-APP marker, so the JFIF requirement that APP0
    /// immediately follows SOI is preserved.
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

        // Walk past the leading APPn segments (0xFFE0..=0xFFEF).
        let mut pos = 2;
        while pos + 4 <= jpeg_data.len() {
            if jpeg_data[pos] != 0xFF {
                break;
            }

            let marker = Self::read_u16_be(&jpeg_data[pos..pos + 2]);
            if !(0xFFE0..=0xFFEF).contains(&marker) {
                break; // Insert before the first non-APP marker (DQT/SOF/SOS/...)
            }

            let length = Self::read_u16_be(&jpeg_data[pos + 2..pos + 4]) as usize;
            if length < 2 || pos + 2 + length > jpeg_data.len() {
                break; // Malformed/truncated segment; insert here rather than run past EOF
            }

            pos += 2 + length;
        }

        Ok(pos)
    }

    /// Finds every Lupin APP13 segment in the JPEG data, in file order.
    ///
    /// A payload larger than a single APP13 segment (~64 KB) is split across
    /// several consecutive APP13 segments on embed; each carries the Lupin
    /// signature. This returns the `(chunk_start, chunk_end)` byte range of the
    /// payload chunk *inside* each of our segments (i.e. after the marker, length
    /// field and signature). Concatenating those ranges in order reconstructs the
    /// original payload.
    ///
    /// Foreign APP13 segments (those without the Lupin signature) are skipped.
    /// Truncated or malformed segments terminate the scan without panicking.
    fn find_lupin_segments(&self, jpeg_data: &[u8]) -> Vec<(usize, usize)> {
        let mut chunks = Vec::new();
        let mut pos = 2; // Skip SOI marker

        while pos + 4 <= jpeg_data.len() {
            // Check if this is a marker (0xFF followed by a marker code)
            if jpeg_data[pos] != 0xFF {
                break; // Not a marker, we've hit image data
            }

            let marker = Self::read_u16_be(&jpeg_data[pos..pos + 2]);

            // If we hit SOS or EOI, we've gone past the header
            if marker == Self::SOS_MARKER || marker == Self::EOI_MARKER {
                break;
            }

            // Standalone markers without length fields
            if marker == Self::SOI_MARKER || (0xFFD0..=0xFFD7).contains(&marker) {
                pos += 2;
                continue;
            }

            let length = Self::read_u16_be(&jpeg_data[pos + 2..pos + 4]) as usize;

            // The length field counts itself, so it must be at least 2. Anything
            // shorter, or a segment that runs past the end of the buffer, is
            // corrupt: stop scanning instead of computing an out-of-range end.
            if length < 2 || pos + 2 + length > jpeg_data.len() {
                break;
            }

            let segment_end = pos + 2 + length;

            // Only an APP13 segment carrying our signature is ours; foreign
            // APP13 segments (e.g. Photoshop) are skipped over.
            if marker == Self::APP13_MARKER {
                let data = &jpeg_data[pos + 4..segment_end];
                if data.starts_with(Self::LUPIN_SIGNATURE) {
                    let chunk_start = pos + 4 + Self::LUPIN_SIGNATURE.len();
                    chunks.push((chunk_start, segment_end));
                }
            }

            // Move to next segment
            pos = segment_end;
        }

        chunks
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
        // Reject empty payloads so the embed contract is uniform across engines.
        if payload.is_empty() {
            return Err(LupinError::EmptyPayload);
        }

        // Check if there's already a Lupin APP13 segment
        if let Some(&(start, end)) = self.find_lupin_segments(source_data).first() {
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

        // Find where to insert our APP13 segment(s) (after the leading APPn segments)
        let insert_pos = self.find_insert_position(source_data)?;

        // A single APP segment's data (length field + signature + chunk) must fit
        // in a u16 length field, so cap each chunk accordingly. Larger payloads
        // are split across several consecutive APP13 segments for unlimited
        // capacity.
        let max_chunk = 0xFFFF - 2 - Self::LUPIN_SIGNATURE.len();

        // Build the APP13 segment(s). An empty payload still emits one (empty)
        // segment so extraction can find it.
        let mut segments = Vec::new();
        let mut chunks = payload.chunks(max_chunk);
        loop {
            let chunk: &[u8] = chunks.next().unwrap_or(&[]);
            let segment_data_length = 2 + Self::LUPIN_SIGNATURE.len() + chunk.len();
            segments.extend_from_slice(&Self::write_u16_be(Self::APP13_MARKER)); // APP13 marker
            segments.extend_from_slice(&Self::write_u16_be(segment_data_length as u16)); // Length
            segments.extend_from_slice(Self::LUPIN_SIGNATURE); // Lupin signature
            segments.extend_from_slice(chunk); // Raw payload chunk

            // Stop once the payload is exhausted (the empty-payload case emits
            // exactly one segment via the initial `unwrap_or(&[])`).
            if chunk.len() < max_chunk {
                break;
            }
        }

        debug!(
            "JPEG: Inserting {} bytes of APP13 segment(s) at position {}",
            segments.len(),
            insert_pos
        );

        // Build result: [original up to insert_pos] + [APP13 segment(s)] + [rest of original]
        let mut result = Vec::with_capacity(source_data.len() + segments.len());
        result.extend_from_slice(&source_data[..insert_pos]);
        result.extend_from_slice(&segments);
        result.extend_from_slice(&source_data[insert_pos..]);

        Ok(result)
    }

    fn extract(&self, source_data: &[u8]) -> Result<Vec<u8>> {
        // Find every Lupin APP13 segment and concatenate their chunks in order.
        let chunks = self.find_lupin_segments(source_data);
        if chunks.is_empty() {
            return Err(LupinError::JpegNoHiddenData);
        }

        debug!("JPEG: Found {} Lupin APP13 segment(s)", chunks.len());

        let mut payload = Vec::new();
        for (start, end) in chunks {
            payload.extend_from_slice(&source_data[start..end]);
        }

        Ok(payload)
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
    fn test_empty_payload_rejected() {
        let engine = JpegEngine::new();
        let payload = b"";

        // Empty payloads are rejected for a uniform embed contract across engines.
        assert!(matches!(
            engine.embed(MINIMAL_JPEG, payload),
            Err(LupinError::EmptyPayload)
        ));
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
    fn test_multi_segment_payload() {
        let engine = JpegEngine::new();
        // Well over a single APP13 segment's ~64 KB capacity, forcing the
        // payload to be split across multiple chained APP13 segments.
        let payload: Vec<u8> = (0..250_000).map(|i| (i % 251) as u8).collect();

        let embedded = engine.embed(MINIMAL_JPEG, &payload).unwrap();

        // It must actually have produced more than one Lupin segment.
        assert!(
            engine.find_lupin_segments(&embedded).len() > 1,
            "large payload should span multiple APP13 segments"
        );

        let extracted = engine.extract(&embedded).unwrap();
        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_payload_at_segment_boundary() {
        let engine = JpegEngine::new();
        // Exactly the max chunk size, then one byte over, to exercise the split
        // boundary logic.
        let max_chunk = 0xFFFF - 2 - JpegEngine::LUPIN_SIGNATURE.len();
        for len in [max_chunk - 1, max_chunk, max_chunk + 1] {
            let payload = vec![7u8; len];
            let embedded = engine.embed(MINIMAL_JPEG, &payload).unwrap();
            let extracted = engine.extract(&embedded).unwrap();
            assert_eq!(extracted, payload, "round trip failed for len {}", len);
        }
    }

    #[test]
    fn test_unicode_payload() {
        let engine = JpegEngine::new();
        let payload = "Hello, 世界! 🕵️ Lupin steganography".as_bytes();

        let embedded = engine.embed(MINIMAL_JPEG, payload).unwrap();
        let extracted = engine.extract(&embedded).unwrap();
        assert_eq!(extracted, payload);
    }

    /// Builds a JPEG containing a foreign (non-Lupin) APP13 segment, as produced
    /// by tools like Adobe Photoshop.
    fn jpeg_with_foreign_app13() -> Vec<u8> {
        let foreign_data = b"Photoshop 3.0\x008BIM"; // Typical Photoshop IRB prefix
        let length = (2 + foreign_data.len()) as u16;
        let mut jpeg = vec![0xFF, 0xD8]; // SOI
        jpeg.extend_from_slice(&[0xFF, 0xED]); // APP13 marker
        jpeg.extend_from_slice(&[(length >> 8) as u8, length as u8]); // Length
        jpeg.extend_from_slice(foreign_data);
        jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI
        jpeg
    }

    #[test]
    fn test_foreign_app13_not_treated_as_lupin() {
        let engine = JpegEngine::new();
        let jpeg = jpeg_with_foreign_app13();

        // Extraction must report "no hidden data", not decode the foreign segment.
        assert!(matches!(
            engine.extract(&jpeg),
            Err(LupinError::JpegNoHiddenData)
        ));

        // Embedding must succeed (no collision with the foreign APP13)...
        let embedded = engine.embed(&jpeg, b"real secret").unwrap();
        // ...and round-trip our own payload while leaving the foreign segment intact.
        assert_eq!(engine.extract(&embedded).unwrap(), b"real secret");
    }

    #[test]
    fn test_extract_coexists_with_foreign_app13() {
        let engine = JpegEngine::new();
        let jpeg = jpeg_with_foreign_app13();
        let embedded = engine.embed(&jpeg, b"payload").unwrap();

        // The foreign Photoshop marker must still be present in the output.
        assert!(embedded.windows(9).any(|w| w == b"Photoshop"));
        assert_eq!(engine.extract(&embedded).unwrap(), b"payload");
    }

    #[test]
    fn test_truncated_segment_does_not_panic() {
        let engine = JpegEngine::new();
        // APP13 marker with a length claiming far more data than is present.
        let jpeg = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xED, // APP13 marker
            0xFF, 0xFF, // Length = 65535, but no data follows
        ];

        // Must return an error, not panic on an out-of-range slice.
        assert!(matches!(
            engine.extract(&jpeg),
            Err(LupinError::JpegNoHiddenData)
        ));
    }

    #[test]
    fn test_undersized_length_does_not_panic() {
        let engine = JpegEngine::new();
        // APP13 marker with a bogus length of 0 (a valid length must be >= 2).
        let jpeg = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xED, // APP13 marker
            0x00, 0x00, // Length = 0 (malformed)
            0xFF, 0xD9, // EOI
        ];

        assert!(matches!(
            engine.extract(&jpeg),
            Err(LupinError::JpegNoHiddenData)
        ));
    }

    #[test]
    fn test_insert_preserves_jfif_ordering() {
        let engine = JpegEngine::new();
        let embedded = engine.embed(MINIMAL_JPEG, b"hi").unwrap();

        // APP0/JFIF must still immediately follow SOI; our APP13 goes after it.
        assert_eq!(&embedded[0..2], &[0xFF, 0xD8]); // SOI
        assert_eq!(&embedded[2..4], &[0xFF, 0xE0]); // APP0 (JFIF) unchanged
    }
}
