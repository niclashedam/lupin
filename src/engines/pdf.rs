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

use crate::{
    error::{LupinError, Result},
    SteganographyEngine,
};
use base64::{engine::general_purpose, Engine as _};
use log::debug;

/// PDF steganography engine
///
/// PDFs end with %%EOF, but viewers ignore anything after that.
/// We append a base64-encoded payload after the EOF marker.
pub struct PdfEngine;

impl PdfEngine {
    pub fn new() -> Self {
        Self
    }

    /// Finds where the PDF actually ends (after the last %%EOF marker)
    fn find_eof_end(&self, pdf: &[u8]) -> Option<usize> {
        let eof_marker = b"%%EOF";
        pdf.windows(eof_marker.len())
            .rposition(|window| window == eof_marker)
            .map(|pos| pos + eof_marker.len())
    }
}

impl Default for PdfEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SteganographyEngine for PdfEngine {
    fn magic_bytes(&self) -> &[u8] {
        b"%PDF"
    }

    fn format_name(&self) -> &str {
        "PDF"
    }

    fn format_ext(&self) -> &str {
        ".pdf"
    }

    fn embed(&self, source_data: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
        let eof_end = self
            .find_eof_end(source_data)
            .ok_or(LupinError::PdfNoEofMarker)?;

        debug!("PDF: Found %%EOF at position {}", eof_end - 5);

        let encoded_payload = general_purpose::STANDARD.encode(payload);

        // Check if there's non-whitespace content after %%EOF (indicating existing hidden data)
        let content_after_eof = &source_data[eof_end..];
        let has_non_whitespace = content_after_eof.iter().any(|&b| !b.is_ascii_whitespace());

        if has_non_whitespace {
            // There's already non-whitespace data after %%EOF - likely hidden data
            return Err(LupinError::EmbedCollision {
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "PDF: Already contains some data after %%EOF",
                ),
            });
        }

        let mut result = Vec::with_capacity(eof_end + encoded_payload.len());
        result.extend_from_slice(&source_data[..eof_end]);
        result.extend_from_slice(encoded_payload.as_bytes());
        Ok(result)
    }

    fn extract(&self, source_data: &[u8]) -> Result<Vec<u8>> {
        let eof_marker = b"%%EOF";
        let eof_pos = source_data
            .windows(eof_marker.len())
            .rposition(|w| w == eof_marker)
            .ok_or(LupinError::PdfNoEofMarker)?;

        debug!("PDF: Found %%EOF at position {}", eof_pos);

        let payload_start = eof_pos + eof_marker.len();
        let payload = &source_data[payload_start..];

        // Skip whitespace after %%EOF
        let payload: Vec<u8> = payload
            .iter()
            .skip_while(|&&b| b.is_ascii_whitespace())
            .copied()
            .collect();

        if payload.is_empty() {
            return Err(LupinError::PdfNoHiddenData);
        }

        general_purpose::STANDARD
            .decode(&payload)
            .map_err(|_| LupinError::PdfCorruptedData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_pdf() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n>>\nendobj\nxref\n0 1\n0000000000 65535 f\ntrailer\n<<\n/Size 1\n/Root 1 0 R\n>>\nstartxref\n73\n%%EOF".to_vec()
    }

    fn create_invalid_pdf_no_eof() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n>>\nendobj".to_vec()
    }

    #[test]
    fn test_magic_bytes() {
        // Arrange
        let engine = PdfEngine::new();

        // Act & Assert
        assert_eq!(engine.magic_bytes(), b"%PDF"); // Magic bytes should match PDF file format signature
    }

    #[test]
    fn test_format_name() {
        // Arrange
        let engine = PdfEngine::new();

        // Act & Assert
        assert_eq!(engine.format_name(), "PDF"); // Format name should be human-readable identifier
    }

    #[test]
    fn test_format_ext() {
        // Arrange
        let engine = PdfEngine::new();

        // Act & Assert
        assert_eq!(engine.format_ext(), ".pdf"); // File extension should include the dot
    }

    #[test]
    fn test_find_eof_end() {
        // Arrange
        let engine = PdfEngine::new();
        let pdf = create_minimal_pdf();

        // Act
        let eof_end = engine.find_eof_end(&pdf);

        // Assert
        assert!(eof_end.is_some()); // Should find %%EOF marker in valid PDF
        let pos = eof_end.unwrap();
        assert_eq!(pos, 125); // Position should be immediately after %%EOF in minimal PDF (120 + 5 = 125)
        assert_eq!(&pdf[120..125], b"%%EOF"); // Should find the actual %%EOF marker at position 120
    }

    #[test]
    fn test_find_eof_end_multiple_eof() {
        // Arrange
        let engine = PdfEngine::new();
        let mut pdf = create_minimal_pdf();
        pdf.extend_from_slice(b"\n%%EOF\nfake_data");

        // Act
        let eof_end = engine.find_eof_end(&pdf);

        // Assert
        assert!(eof_end.is_some()); // Should find the last %%EOF marker when multiple exist
        let pos = eof_end.unwrap();
        assert_eq!(pos, 131); // Should point to end of second %%EOF (125 + 6 = 131)
        assert_eq!(&pdf[126..131], b"%%EOF"); // Should find the second %%EOF marker
        assert_eq!(&pdf[131..], b"\nfake_data"); // Content after second %%EOF should remain
    }

    #[test]
    fn test_find_eof_end_no_eof() {
        // Arrange
        let engine = PdfEngine::new();
        let pdf = create_invalid_pdf_no_eof();

        // Act
        let eof_end = engine.find_eof_end(&pdf);

        // Assert
        assert!(eof_end.is_none()); // Should return None when no %%EOF marker is found
    }

    #[test]
    fn test_embed_success() {
        // Arrange
        let engine = PdfEngine::new();
        let pdf = create_minimal_pdf();
        let payload = b"secret message";

        // Act
        let result = engine.embed(&pdf, payload);

        // Assert
        assert!(result.is_ok()); // Embed operation should succeed with valid PDF

        let embedded = result.unwrap();
        assert_eq!(embedded.len(), 145); // Original PDF (125 bytes) + base64 encoded payload (20 bytes) = 145
        assert!(embedded.starts_with(b"%PDF")); // Should preserve PDF magic bytes at start
        assert!(embedded.ends_with(b"c2VjcmV0IG1lc3NhZ2U=")); // Should end with base64 of "secret message"
    }

    #[test]
    fn test_embed_no_eof_marker() {
        // Arrange
        let engine = PdfEngine::new();
        let invalid_pdf = create_invalid_pdf_no_eof();
        let payload = b"secret message";

        // Act
        let result = engine.embed(&invalid_pdf, payload);

        // Assert
        assert!(result.is_err()); // Should fail when PDF has no %%EOF marker

        match result.unwrap_err() {
            LupinError::PdfNoEofMarker => (), // Should return specific error for missing %%EOF
            other => panic!("Expected PdfNoEofMarker, got {:?}", other),
        }
    }

    #[test]
    fn test_embed_empty_payload() {
        // Arrange
        let engine = PdfEngine::new();
        let pdf = create_minimal_pdf();
        let payload = b"";

        // Act
        let result = engine.embed(&pdf, payload);

        // Assert
        assert!(result.is_ok()); // Empty payload should still embed successfully

        let embedded = result.unwrap();
        assert_eq!(embedded.len(), 125); // Original PDF (125 bytes) + base64 of empty string (0 bytes) = 125
    }

    #[test]
    fn test_embed_into_already_embedded_pdf() {
        // Arrange
        let engine = PdfEngine::new();
        let mut pdf = create_minimal_pdf();
        pdf.extend_from_slice(b"c2VjcmV0IG1lc3NhZ2U="); // base64 of "secret message"

        // Act
        let result = engine.embed(&pdf, "more secret".as_bytes());

        // Assert
        assert!(result.is_err()); // Should fail when base64 data is corrupted

        match result.unwrap_err() {
            LupinError::EmbedCollision { .. } => (), // Should return specific error for embed collision
            other => panic!("Expected EmbedCollision, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_success() {
        // Arrange
        let engine = PdfEngine::new();
        let mut pdf = create_minimal_pdf();
        pdf.extend_from_slice(b"c2VjcmV0IG1lc3NhZ2U="); // base64 of "secret message"

        // Act
        let result = engine.extract(&pdf);

        // Assert
        assert!(result.is_ok()); // Extract should succeed with valid embedded data

        let extracted_payload = result.unwrap();
        assert_eq!(extracted_payload, b"secret message"); // Should extract exact original payload
    }

    #[test]
    fn test_extract_no_eof_marker() {
        // Arrange
        let engine = PdfEngine::new();
        let invalid_pdf = create_invalid_pdf_no_eof();

        // Act
        let result = engine.extract(&invalid_pdf);

        // Assert
        assert!(result.is_err()); // Should fail when PDF has no %%EOF marker

        match result.unwrap_err() {
            LupinError::PdfNoEofMarker => (), // Should return specific error for missing %%EOF
            other => panic!("Expected PdfNoEofMarker, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_no_hidden_data() {
        // Arrange
        let engine = PdfEngine::new();
        let clean_pdf = create_minimal_pdf();

        // Act
        let result = engine.extract(&clean_pdf);

        // Assert
        assert!(result.is_err()); // Should fail when no data exists after %%EOF

        match result.unwrap_err() {
            LupinError::PdfNoHiddenData => (), // Should return specific error for no hidden data
            other => panic!("Expected PdfNoHiddenData, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_corrupted_data() {
        // Arrange
        let engine = PdfEngine::new();
        let mut pdf = create_minimal_pdf();

        // Add invalid base64 data after %%EOF
        pdf.extend_from_slice(b"invalid@base64!");

        // Act
        let result = engine.extract(&pdf);

        // Assert
        assert!(result.is_err()); // Should fail when base64 data is corrupted

        match result.unwrap_err() {
            LupinError::PdfCorruptedData => (), // Should return specific error for corrupted base64
            other => panic!("Expected PdfCorruptedData, got {:?}", other),
        }
    }

    #[test]
    fn test_extract_with_whitespace() {
        // Arrange
        let engine = PdfEngine::new();
        let pdf = create_minimal_pdf();

        // Create embedded data with whitespace manually
        let mut embedded = Vec::new();
        embedded.extend_from_slice(&pdf);
        embedded.extend_from_slice(b"  \n\t"); // Add whitespace after %%EOF
        embedded.extend_from_slice(b"dGVzdCB3aXRoIHNwYWNlcw=="); // base64 of "test with spaces"

        // Act
        let result = engine.extract(&embedded);

        // Assert
        assert!(result.is_ok()); // Should succeed despite whitespace after %%EOF

        let extracted_payload = result.unwrap();
        assert_eq!(extracted_payload, b"test with spaces"); // Should extract correct payload ignoring whitespace
    }

    #[test]
    fn test_round_trip_with_binary_data() {
        // Arrange
        let engine = PdfEngine::new();
        let pdf = create_minimal_pdf();

        // Act
        let binary_payload = b"\x00\x01\x02\xff";
        let embedded2 = engine.embed(&pdf, binary_payload).unwrap();
        let extracted2 = engine.extract(&embedded2).unwrap();

        // Assert
        assert_eq!(extracted2, b"\x00\x01\x02\xff"); // Binary data should round-trip correctly
    }

    #[test]
    fn test_round_trip_with_unicode_data() {
        // Arrange
        let engine = PdfEngine::new();
        let pdf = create_minimal_pdf();

        // Act
        let unicode_payload = "unicode: 🕵️ αβγ δεζ".as_bytes();
        let embedded3 = engine.embed(&pdf, unicode_payload).unwrap();
        let extracted3 = engine.extract(&embedded3).unwrap();

        // Assert
        assert_eq!(extracted3, "unicode: 🕵️ αβγ δεζ".as_bytes()); // Unicode should round-trip correctly
    }
}
