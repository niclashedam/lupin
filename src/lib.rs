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

use std::io;

// Module declarations
pub mod cli;
pub mod commands;
pub mod engines;
pub mod file;
pub mod operations;
pub mod output;

/// Trait for steganography engines that can embed and extract data from specific file formats
pub trait SteganographyEngine {
    /// Returns the magic bytes that identify this file format
    fn magic_bytes(&self) -> &[u8];

    /// Returns a human-readable name for this file format
    fn format_name(&self) -> &str;

    /// Returns a human-readable extension for this file format
    fn format_ext(&self) -> &str;

    /// Embeds payload data into the source file data
    fn embed(&self, source_data: &[u8], payload: &[u8]) -> io::Result<Vec<u8>>;

    /// Extracts hidden payload from the file data
    fn extract(&self, source_data: &[u8]) -> io::Result<Vec<u8>>;
}

/// File format detector that routes to appropriate engines
pub struct EngineRouter {
    pub engines: Vec<Box<dyn SteganographyEngine>>,
}

impl EngineRouter {
    /// Creates a new router with all available engines
    pub fn new() -> Self {
        Self {
            engines: vec![Box::new(engines::PdfEngine::new())],
        }
    }

    /// Detects the file format and returns the appropriate engine
    pub fn detect_engine(&self, data: &[u8]) -> io::Result<&dyn SteganographyEngine> {
        for engine in &self.engines {
            let magic = engine.magic_bytes();
            if data.len() >= magic.len() && &data[..magic.len()] == magic {
                return Ok(engine.as_ref());
            }
        }

        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unsupported file format.",
        ))
    }
}

impl Default for EngineRouter {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests to make sure we don't break anything
#[cfg(test)]
mod tests {
    use crate::engines::PdfEngine;
    use crate::{EngineRouter, SteganographyEngine};

    #[test]
    fn test_pdf_engine_magic_bytes() {
        let engine = PdfEngine::new();
        assert_eq!(engine.magic_bytes(), b"%PDF");
        assert_eq!(engine.format_name(), "PDF");
    }

    #[test]
    fn test_engine_router_detection() {
        let router = EngineRouter::new();

        // Test PDF detection
        let pdf_data = b"%PDF-1.4\nSome PDF content";
        let engine = router.detect_engine(pdf_data).unwrap();
        assert_eq!(engine.format_name(), "PDF");

        // Test unsupported format
        let unknown_data = b"Unknown file format";
        let result = router.detect_engine(unknown_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_router_supported_formats() {
        let router = EngineRouter::new();
        // Test that we have at least PDF support
        let pdf_data = b"%PDF-1.4\nSome PDF content";
        let result = router.detect_engine(pdf_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().format_name(), "PDF");
    }
}
