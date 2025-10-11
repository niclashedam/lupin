pub mod file_operations {
    use std::fs::{self, File};
    use std::io::{self, Write};
    use std::path::Path;

    // Simple file I/O wrappers - nothing fancy here
    pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    pub fn write_file<P: AsRef<Path>>(path: P, data: &[u8]) -> io::Result<()> {
        File::create(path)?.write_all(data)
    }
}

pub mod pdf_parser {
    // Finds where the PDF actually ends
    // Caution: PDFs can have multiple %%EOF markers, so we want the rightmost one
    pub fn find_eof_end(pdf: &[u8]) -> Option<usize> {
        let eof_marker = b"%%EOF";
        pdf.windows(eof_marker.len())
            .rposition(|window| window == eof_marker)
            .map(|pos| pos + eof_marker.len())
    }
}

pub mod payload_operations {
    use base64::{Engine as _, engine::general_purpose};
    use std::io;

    // Takes a PDF and payload, returns the combined data with payload embedded
    // Structure: [PDF content up to %%EOF][base64(payload)]
    pub fn encode_payload(pdf_data: &[u8], eof_end: usize, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(pdf_data.len() + payload.len() + 64);
        out.extend_from_slice(&pdf_data[..eof_end]);
        out.extend_from_slice(general_purpose::STANDARD.encode(payload).as_bytes());
        out
    }

    // Extracts and decodes the hidden payload from a PDF with embedded data
    // Returns the raw binary payload, not the base64 version
    pub fn extract_payload(data: &[u8]) -> io::Result<Vec<u8>> {
        // Find the last %%EOF marker in the file
        let eof_marker = b"%%EOF";
        let eof_pos = data
            .windows(eof_marker.len())
            .rposition(|w| w == eof_marker)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "%%EOF marker not found – this doesn't appear to be a valid PDF file.",
                )
            })?;

        // Payload starts right after the %%EOF marker
        let payload_start = eof_pos + eof_marker.len();
        let payload = &data[payload_start..];

        // Skip any whitespace that might be after %%EOF
        let payload = payload
            .iter()
            .skip_while(|&&b| b == b'\n' || b == b'\r' || b == b' ' || b == b'\t')
            .copied()
            .collect::<Vec<u8>>();

        // If there's no payload after EOF, that's an error
        if payload.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No payload found after %%EOF marker – this PDF doesn't contain hidden data.",
            ));
        }

        // Decode the base64 payload back to original binary data
        general_purpose::STANDARD.decode(&payload).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to decode base64 payload. The data might be corrupted.",
            )
        })
    }
}

pub mod operations {
    use crate::file_operations::{read_file, write_file};
    use crate::payload_operations::{encode_payload, extract_payload};
    use crate::pdf_parser::find_eof_end;
    use std::io::{self, Write};
    use std::path::Path;

    // Main embedding function - hides payload data inside a PDF file
    // Takes source PDF + payload file, outputs a new PDF with hidden data
    pub fn embed(src_pdf: &Path, payload: &Path, out_pdf: &Path) -> io::Result<()> {
        let pdf_bytes = read_file(src_pdf)?;

        // Make sure this is actually a PDF with a proper EOF marker
        let eof_end = find_eof_end(&pdf_bytes).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Could not locate a '%%EOF' marker in the source PDF. Are you sure this is a valid PDF file?",
            )
        })?;

        let payload_bytes = read_file(payload)?;
        let output_data = encode_payload(&pdf_bytes, eof_end, &payload_bytes);

        write_file(out_pdf, &output_data)?;

        // Let the user know we succeeded
        println!(
            "✅ Embedded {} bytes into '{}', saved as '{}'.",
            payload_bytes.len(),
            src_pdf.display(),
            out_pdf.display()
        );
        Ok(())
    }

    // Main extraction function - pulls hidden data out of a PDF
    // Can write to a file or stdout (if output path is "-")
    pub fn extract(src_pdf: &Path, out_bin: &Path) -> io::Result<()> {
        let data = read_file(src_pdf)?;
        let decoded_payload = extract_payload(&data)?;

        // Special case: "-" means write to stdout for piping
        if out_bin.as_os_str() == "-" {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(&decoded_payload)?;
        } else {
            // Regular file output
            write_file(out_bin, &decoded_payload)?;
            println!(
                "✅ Extracted {} bytes from '{}', saved as '{}'.",
                decoded_payload.len(),
                src_pdf.display(),
                out_bin.display()
            );
        }
        Ok(())
    }
}

// Unit tests to make sure we don't break anything
#[cfg(test)]
mod tests {
    use crate::payload_operations::{encode_payload, extract_payload};
    use crate::pdf_parser::find_eof_end;

    #[test]
    fn test_find_eof_end() {
        let pdf_data = b"Some PDF content\n%%EOF\nExtra data";
        let result = find_eof_end(pdf_data);
        assert_eq!(result, Some(22)); // Position after %%EOF
    }

    #[test]
    fn test_find_eof_end_multiple() {
        let pdf_data = b"%%EOF\nSome content\n%%EOF\n";
        let result = find_eof_end(pdf_data);
        assert_eq!(result, Some(24)); // Should find the last one
    }

    #[test]
    fn test_find_eof_end_not_found() {
        let pdf_data = b"Some PDF content without EOF marker";
        let result = find_eof_end(pdf_data);
        assert_eq!(result, None); // Should return None for invalid PDFs
    }

    #[test]
    fn test_encode_and_extract_payload() {
        let pdf_data = b"PDF content\n%%EOF";
        let eof_end = 17; // Position after %%EOF (13 + 5 - 1)
        let payload = b"Hello, World!";

        // Test round-trip: encode then extract
        let encoded = encode_payload(pdf_data, eof_end, payload);
        let extracted = extract_payload(&encoded).unwrap();

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_extract_payload_missing_eof() {
        let data = b"PDF content without EOF marker";
        let result = extract_payload(data);
        assert!(result.is_err()); // Should fail gracefully
    }

    #[test]
    fn test_extract_payload_invalid_base64() {
        let data = b"PDF content\n%%EOFInvalidBase64!!!";

        let result = extract_payload(data);
        assert!(result.is_err()); // Should detect corrupted data
    }
}
