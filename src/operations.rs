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
use crate::EngineRouter;
use std::io::{self, Write};
use std::path::Path;

/// Embeds payload data inside a file using the appropriate engine
pub fn embed(src_file: &Path, payload: &Path, out_file: &Path) -> io::Result<()> {
    let router = EngineRouter::new();
    let source_bytes = read_file(src_file)?;
    let payload_bytes = read_file(payload)?;

    let engine = router.detect_engine(&source_bytes)?;
    let output_data = engine.embed(&source_bytes, &payload_bytes)?;

    write_file(out_file, &output_data)?;

    println!(
        "✅ Embedded {} bytes into {} ({}), saved as {}",
        payload_bytes.len(),
        src_file.display(),
        engine.format_name(),
        out_file.display()
    );
    Ok(())
}

/// Extracts hidden data from a file using the appropriate engine
pub fn extract(src_file: &Path, out_file: &Path) -> io::Result<()> {
    let router = EngineRouter::new();
    let data = read_file(src_file)?;

    let engine = router.detect_engine(&data)?;
    let payload = engine.extract(&data)?;

    // Special case: "-" means write to stdout
    if out_file.as_os_str() == "-" {
        io::stdout().write_all(&payload)?;
    } else {
        write_file(out_file, &payload)?;
        println!(
            "✅ Extracted {} bytes from {} ({}), saved as {}",
            payload.len(),
            src_file.display(),
            engine.format_name(),
            out_file.display()
        );
    }
    Ok(())
}
