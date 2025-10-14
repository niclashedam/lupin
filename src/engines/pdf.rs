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

use crate::SteganographyEngine;
use base64::{engine::general_purpose, Engine as _};
use std::io;

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

    fn embed(&self, source_data: &[u8], payload: &[u8]) -> io::Result<Vec<u8>> {
        let eof_end = self.find_eof_end(source_data).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid PDF: no %%EOF marker found",
            )
        })?;

        let encoded_payload = general_purpose::STANDARD.encode(payload);
        let mut result = Vec::with_capacity(eof_end + encoded_payload.len());
        result.extend_from_slice(&source_data[..eof_end]);
        result.extend_from_slice(encoded_payload.as_bytes());
        Ok(result)
    }

    fn extract(&self, source_data: &[u8]) -> io::Result<Vec<u8>> {
        let eof_marker = b"%%EOF";
        let eof_pos = source_data
            .windows(eof_marker.len())
            .rposition(|w| w == eof_marker)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Invalid PDF: no %%EOF marker found",
                )
            })?;

        let payload_start = eof_pos + eof_marker.len();
        let payload = &source_data[payload_start..];

        // Skip whitespace after %%EOF
        let payload: Vec<u8> = payload
            .iter()
            .skip_while(|&&b| b.is_ascii_whitespace())
            .copied()
            .collect();

        if payload.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No hidden data found in PDF",
            ));
        }

        general_purpose::STANDARD
            .decode(&payload)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Corrupted hidden data"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let engine = PdfEngine::new();
        let pdf_data = b"%PDF-1.4\nSome content\n%%EOF";
        let payload = b"Hello, World!";

        let embedded = engine.embed(pdf_data, payload).unwrap();
        let extracted = engine.extract(&embedded).unwrap();

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_find_eof_end() {
        let engine = PdfEngine::new();

        // Test normal case
        let pdf_data = b"Content\n%%EOF\nExtra";
        assert_eq!(engine.find_eof_end(pdf_data), Some(13));

        // Test multiple EOF markers (should find last)
        let pdf_data = b"%%EOF\nContent\n%%EOF\n";
        assert_eq!(engine.find_eof_end(pdf_data), Some(19));

        // Test no EOF marker
        let pdf_data = b"No EOF marker here";
        assert_eq!(engine.find_eof_end(pdf_data), None);
    }

    #[test]
    fn test_invalid_pdf() {
        let engine = PdfEngine::new();
        let invalid_data = b"Not a PDF";

        let result = engine.extract(invalid_data);
        assert!(result.is_err());
    }
}
