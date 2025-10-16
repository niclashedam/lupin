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

//! High-level operations for embedding and extracting steganographic data

use crate::error::Result;
use crate::EngineRouter;

/// Result of an embed operation
#[derive(Debug, Clone)]
pub struct EmbedResult {
    pub source_size: usize,
    pub output_size: usize,
    pub engine: String,
}

/// Result of an extract operation  
#[derive(Debug, Clone)]
pub struct ExtractResult {
    pub source_size: usize,
    pub payload_size: usize,
    pub engine: String,
}

/// Embeds payload data inside source data using the appropriate engine
/// Returns the embedded data and operation metadata
pub fn embed(source_data: &[u8], payload_data: &[u8]) -> Result<(Vec<u8>, EmbedResult)> {
    // Determine the correct engine based on magic bytes
    let router = EngineRouter::new();
    let engine = router.detect_engine(source_data)?;

    // Embed the payload data using the detected engine
    let embedded_data = engine.embed(source_data, payload_data)?;

    // Create the result metadata
    let result = EmbedResult {
        source_size: source_data.len(),
        output_size: embedded_data.len(),
        engine: engine.format_name().to_string(),
    };

    Ok((embedded_data, result))
}

/// Extracts hidden data from source data using the appropriate engine
/// Returns the extracted payload and operation metadata
pub fn extract(source_data: &[u8]) -> Result<(Vec<u8>, ExtractResult)> {
    let router = EngineRouter::new();
    let engine = router.detect_engine(source_data)?;
    let payload = engine.extract(source_data)?;

    let result = ExtractResult {
        engine: engine.format_name().to_string(),
        payload_size: payload.len(),
        source_size: source_data.len(),
    };

    Ok((payload, result))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_pdf() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n>>\nendobj\nxref\n0 1\n0000000000 65535 f\ntrailer\n<<\n/Size 1\n/Root 1 0 R\n>>\nstartxref\n73\n%%EOF".to_vec()
    }

    #[test]
    fn test_embed() {
        // Arrange
        let source = create_minimal_pdf();
        let payload = b"test message";

        // Act
        let result = embed(&source, payload);

        // Assert
        assert!(result.is_ok()); // Embed operation should succeed

        let (embedded_data, metadata) = result.unwrap();

        // Verify the embedded data is valid
        assert!(embedded_data.len() > source.len()); // Should be larger than original
        assert!(embedded_data.starts_with(b"%PDF")); // Should preserve PDF format

        // Verify the metadata is correct
        assert_eq!(metadata.engine, "PDF"); // Should use PDF engine
        assert_eq!(metadata.source_size, 125); // Known size of minimal PDF
        assert_eq!(metadata.output_size, 141); // Length of the PDF plus "test message" base64 encoded
    }

    #[test]
    fn test_extract() {
        // Arrange
        let source = create_minimal_pdf();
        let original_payload = b"secret data";
        let (embedded_data, _) = embed(&source, original_payload).unwrap();

        // Act
        let result = extract(&embedded_data);

        // Assert
        assert!(result.is_ok()); // Extract operation should succeed

        let (extracted_payload, metadata) = result.unwrap();

        // Verify the extracted payload is correct
        assert_eq!(extracted_payload, original_payload); // Should match original exactly

        // Verify the metadata is correct
        assert_eq!(metadata.engine, "PDF"); // Should use PDF engine
        assert_eq!(metadata.source_size, embedded_data.len()); // Should match input size
        assert_eq!(metadata.payload_size, 11); // Length of "secret data"
    }
}
