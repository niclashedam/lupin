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
    pub payload_size: usize,
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
        payload_size: payload_data.len(),
        output_size: embedded_data.len(),
        engine: engine.format_name().to_string(),
    };

    Ok((embedded_data, result))
}

/// Extracts hidden data from source data using the appropriate engine
/// Returns the extracted payload and operation metadata
pub fn extract(source_data: &[u8]) -> Result<(Vec<u8>, ExtractResult)> {
    // Determine the correct engine based on magic bytes
    let router = EngineRouter::new();
    let engine = router.detect_engine(source_data)?;

    // Extract the payload data using the detected engine
    let payload_data = engine.extract(source_data)?;

    // Create the result metadata
    let result = ExtractResult {
        source_size: source_data.len(),
        payload_size: payload_data.len(),
        engine: engine.format_name().to_string(),
    };

    Ok((payload_data, result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LupinError;

    fn create_valid_pdf() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n2 0 obj\n<<\n/Type /Pages\n/Kids [3 0 R]\n/Count 1\n>>\nendobj\n3 0 obj\n<<\n/Type /Page\n/Parent 2 0 R\n/MediaBox [0 0 612 792]\n>>\nendobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000074 00000 n \n0000000120 00000 n \ntrailer\n<<\n/Size 4\n/Root 1 0 R\n>>\nstartxref\n174\n%%EOF".to_vec()
    }

    #[test]
    fn test_embed_operation() {
        let pdf_data = create_valid_pdf();
        let payload = b"secret message";

        let result = embed(&pdf_data, payload);
        assert!(result.is_ok());

        let (embedded_data, embed_result) = result.unwrap();

        // Check that the embedded data is larger than original
        assert!(embedded_data.len() > pdf_data.len());

        // Check that result contains correct metadata
        assert_eq!(embed_result.source_size, pdf_data.len());
        assert_eq!(embed_result.payload_size, payload.len());
        assert_eq!(embed_result.output_size, embedded_data.len());
        assert_eq!(embed_result.engine, "PDF");
    }

    #[test]
    fn test_extract_operation() {
        let pdf_data = create_valid_pdf();
        let payload = b"test payload for extraction";

        // First embed the payload
        let (embedded_data, _) = embed(&pdf_data, payload).unwrap();

        // Then extract it
        let result = extract(&embedded_data);
        assert!(result.is_ok());

        let (extracted_payload, extract_result) = result.unwrap();

        // Check that the extracted payload matches original
        assert_eq!(extracted_payload, payload);

        // Check that result contains correct metadata
        assert_eq!(extract_result.source_size, embedded_data.len());
        assert_eq!(extract_result.payload_size, payload.len());
        assert_eq!(extract_result.engine, "PDF");
    }

    #[test]
    fn test_round_trip() {
        let pdf_data = create_valid_pdf();
        let original_payload = b"round trip test with special chars: !@#$%^&*()";

        // Embed
        let (embedded_data, embed_result) = embed(&pdf_data, original_payload).unwrap();

        // Extract
        let (extracted_payload, extract_result) = extract(&embedded_data).unwrap();

        // Verify round trip
        assert_eq!(extracted_payload, original_payload);
        assert_eq!(embed_result.payload_size, extract_result.payload_size);
        assert_eq!(embed_result.engine, extract_result.engine);
    }

    #[test]
    fn test_embed_invalid_source() {
        let invalid_data = b"not a PDF file";
        let payload = b"test";

        let result = embed(invalid_data, payload);
        assert!(result.is_err());

        match result.unwrap_err() {
            LupinError::Io { .. } => {} // Expected - engine detection returns Io error for unsupported format
            _ => panic!("Expected Io error for unsupported format"),
        }
    }

    #[test]
    fn test_extract_no_hidden_data() {
        let pdf_data = create_valid_pdf(); // Plain PDF without hidden data

        let result = extract(&pdf_data);
        assert!(result.is_err());

        match result.unwrap_err() {
            LupinError::PdfNoHiddenData => {} // Expected - directly from PDF engine
            _ => panic!("Expected PdfNoHiddenData error"),
        }
    }

    #[test]
    fn test_empty_payload() {
        let pdf_data = create_valid_pdf();
        let empty_payload = b"";

        // Should be able to embed empty payload
        let (embedded_data, embed_result) = embed(&pdf_data, empty_payload).unwrap();
        assert_eq!(embed_result.payload_size, 0);

        // However, extracting empty payload should fail because the PDF engine
        // treats empty base64 data as "no hidden data"
        let result = extract(&embedded_data);
        assert!(result.is_err());

        match result.unwrap_err() {
            LupinError::PdfNoHiddenData => {} // Expected for empty payload - directly from PDF engine
            _ => panic!("Expected PdfNoHiddenData error for empty payload"),
        }
    }

    #[test]
    fn test_large_payload() {
        let pdf_data = create_valid_pdf();
        let large_payload = vec![b'X'; 10000]; // 10KB payload

        let (embedded_data, embed_result) = embed(&pdf_data, &large_payload).unwrap();
        assert_eq!(embed_result.payload_size, 10000);

        let (extracted_payload, extract_result) = extract(&embedded_data).unwrap();
        assert_eq!(extracted_payload, large_payload);
        assert_eq!(extract_result.payload_size, 10000);
    }
}
