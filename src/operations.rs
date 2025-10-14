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

use crate::file::{read_file, write_file};
use crate::output::OutputFormatter;
use crate::EngineRouter;
use std::io::{self, Write};
use std::path::Path;

/// Embeds payload data inside a file using the appropriate engine
pub fn embed(
    src_file: &Path,
    payload: &Path,
    out_file: &Path,
    formatter: &OutputFormatter,
) -> io::Result<()> {
    let router = EngineRouter::new();
    let source_bytes = match read_file(src_file) {
        Ok(bytes) => bytes,
        Err(e) => {
            formatter.error(&format!(
                "Failed to read source file '{}': {}",
                src_file.display(),
                e
            ));
            return Err(e);
        }
    };

    let payload_bytes = match read_file(payload) {
        Ok(bytes) => bytes,
        Err(e) => {
            formatter.error(&format!(
                "Failed to read payload file '{}': {}",
                payload.display(),
                e
            ));
            return Err(e);
        }
    };

    let engine = match router.detect_engine(&source_bytes) {
        Ok(engine) => engine,
        Err(e) => {
            formatter.error(&format!("Engine detection failed: {}", e));
            return Err(e);
        }
    };

    let output_data = match engine.embed(&source_bytes, &payload_bytes) {
        Ok(data) => data,
        Err(e) => {
            formatter.error(&format!("Embedding failed: {}", e));
            return Err(e);
        }
    };

    if let Err(e) = write_file(out_file, &output_data) {
        formatter.error(&format!(
            "Failed to write output file '{}': {}",
            out_file.display(),
            e
        ));
        return Err(e);
    }

    formatter.info(&format!(
        "Embedded {} bytes into {} ({}). Increased by {}% from {} to {} bytes.",
        formatter.size(payload_bytes.len()),
        formatter.path(&src_file.display().to_string()),
        engine.format_name(),
        (output_data.len() as f64 / source_bytes.len() as f64 * 100.0 - 100.0).round(),
        formatter.size(source_bytes.len()),
        formatter.size(output_data.len())
    ));
    Ok(())
}

/// Extracts hidden data from a file using the appropriate engine
pub fn extract(src_file: &Path, out_file: &Path, formatter: &OutputFormatter) -> io::Result<()> {
    let router = EngineRouter::new();
    let data = match read_file(src_file) {
        Ok(data) => data,
        Err(e) => {
            formatter.error(&format!(
                "Failed to read source file '{}': {}",
                src_file.display(),
                e
            ));
            return Err(e);
        }
    };

    let engine = match router.detect_engine(&data) {
        Ok(engine) => engine,
        Err(e) => {
            formatter.error(&format!("Engine detection failed: {}", e));
            return Err(e);
        }
    };

    let payload = match engine.extract(&data) {
        Ok(payload) => payload,
        Err(e) => {
            formatter.error(&format!("Extraction failed: {}", e));
            return Err(e);
        }
    };

    // Special case: "-" means write to stdout
    if out_file.as_os_str() == "-" {
        if let Err(e) = io::stdout().write_all(&payload) {
            formatter.error(&format!("Failed to write to stdout: {}", e));
            return Err(e);
        }
    } else {
        if let Err(e) = write_file(out_file, &payload) {
            formatter.error(&format!(
                "Failed to write output file '{}': {}",
                out_file.display(),
                e
            ));
            return Err(e);
        }
        formatter.info(&format!(
            "Extracted {} bytes from {} ({}).",
            formatter.size(payload.len()),
            formatter.path(&src_file.display().to_string()),
            engine.format_name(),
        ));
    }
    Ok(())
}
