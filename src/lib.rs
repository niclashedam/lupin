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

// Module declarations
pub mod engines;
pub mod error;
pub mod operations;

use crate::engines::{PdfEngine, PngEngine};
use crate::error::Result;
use std::io;

/// Trait for steganography engines that can embed and extract hidden data
pub trait SteganographyEngine {
    /// Returns the magic bytes that identify this file format
    fn magic_bytes(&self) -> &[u8];

    /// Returns a human-readable name for this file format
    fn format_name(&self) -> &str;

    /// Returns a human-readable extension for this file format
    fn format_ext(&self) -> &str;

    /// Embeds payload data into the source file data
    fn embed(&self, source_data: &[u8], payload: &[u8]) -> Result<Vec<u8>>;

    /// Extracts hidden payload from the file data
    fn extract(&self, source_data: &[u8]) -> Result<Vec<u8>>;
}

/// File format detector that routes to appropriate engines
#[derive(Default)]
pub struct EngineRouter {
    pub engines: Vec<Box<dyn SteganographyEngine>>,
}

impl EngineRouter {
    /// Creates a new router with all available engines
    pub fn new() -> Self {
        Self {
            engines: vec![Box::new(PdfEngine::new()), Box::new(PngEngine::new())],
        }
    }

    /// Detects the appropriate engine for the given data
    pub fn detect_engine(&self, data: &[u8]) -> Result<&dyn SteganographyEngine> {
        for engine in &self.engines {
            if data.starts_with(engine.magic_bytes()) {
                return Ok(engine.as_ref());
            }
        }

        Err(crate::error::LupinError::EngineDetection {
            source: io::Error::new(
                io::ErrorKind::Unsupported,
                "Unsupported file format - no matching engine found",
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_pdf() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n>>\nendobj\nxref\n0 1\n0000000000 65535 f\ntrailer\n<<\n/Size 1\n/Root 1 0 R\n>>\nstartxref\n73\n%%EOF".to_vec()
    }

    fn create_minimal_png() -> Vec<u8> {
        let mut png = Vec::new();
        png.extend_from_slice(b"\x89PNG\r\n\x1a\n"); // PNG signature
        png.extend_from_slice(&[0, 0, 0, 13]); // IHDR length
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&[0; 13]); // IHDR data
        png.extend_from_slice(&[0; 4]); // CRC
        png
    }

    fn create_unsupported_format() -> Vec<u8> {
        b"RIFF....WEBP".to_vec() // WebP format
    }

    #[test]
    fn test_detect_engine_pdf() {
        // Arrange
        let router = EngineRouter::new();
        let pdf_data = create_minimal_pdf();

        // Act
        let result = router.detect_engine(&pdf_data);

        // Assert
        assert!(result.is_ok());

        let engine = result.unwrap();
        assert_eq!(engine.format_name(), "PDF");
    }

    #[test]
    fn test_detect_engine_png() {
        // Arrange
        let router = EngineRouter::new();
        let png_data = create_minimal_png();

        // Act
        let result = router.detect_engine(&png_data);

        // Assert
        assert!(result.is_ok());

        let engine = result.unwrap();
        assert_eq!(engine.format_name(), "PNG");
    }

    #[test]
    fn test_detect_engine_unsupported() {
        // Arrange
        let router = EngineRouter::new();
        let unsupported_data = create_unsupported_format();

        // Act
        let result = router.detect_engine(&unsupported_data);

        // Assert
        assert!(result.is_err()); // Should return error for unsupported format

        if let Err(error) = result {
            match error {
                crate::error::LupinError::EngineDetection { .. } => (), // Should return specific engine detection error
                other => panic!("Expected EngineDetection error, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_detect_engine_empty_data() {
        // Arrange
        let router = EngineRouter::new();
        let empty_data = Vec::new();

        // Act
        let result = router.detect_engine(&empty_data);

        // Assert
        assert!(result.is_err()); // Should return error for empty data

        if let Err(error) = result {
            match error {
                crate::error::LupinError::EngineDetection { .. } => (), // Should return specific engine detection error
                other => panic!("Expected EngineDetection error, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_detect_engine_partial_magic_bytes() {
        // Arrange
        let router = EngineRouter::new();
        let partial_pdf = b"%PD".to_vec(); // Only partial PDF magic bytes

        // Act
        let result = router.detect_engine(&partial_pdf);

        // Assert
        assert!(result.is_err()); // Should return error for partial magic bytes

        if let Err(error) = result {
            match error {
                crate::error::LupinError::EngineDetection { .. } => (), // Should return specific engine detection error
                other => panic!("Expected EngineDetection error, got {:?}", other),
            }
        }
    }
}
