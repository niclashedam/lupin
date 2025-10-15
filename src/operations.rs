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

use crate::error::{LupinError, Result};
use crate::file::{read_file, write_file};
use crate::EngineRouter;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// File with path and size
#[derive(Debug, Clone)]
pub struct File {
    pub path: PathBuf,
    pub size: usize,
}

/// Result of an embed operation
#[derive(Debug, Clone)]
pub struct EmbedResult {
    pub src: File,
    pub payload: File,
    pub output: File,
    pub engine: String,
}

/// Result of an extract operation
#[derive(Debug, Clone)]
pub struct ExtractResult {
    pub src: File,
    pub output: File,
    pub engine: String,
    pub written_to_stdout: bool,
}

/// Embeds payload data inside a file using the appropriate engine
pub fn embed(src_file: &Path, payload: &Path, out_file: &Path) -> Result<EmbedResult> {
    let router = EngineRouter::new();

    let source_bytes = read_file(src_file).map_err(|e| LupinError::SourceFileRead {
        path: src_file.to_path_buf(),
        source: e,
    })?;

    let payload_bytes = read_file(payload).map_err(|e| LupinError::PayloadFileRead {
        path: payload.to_path_buf(),
        source: e,
    })?;

    let engine = router
        .detect_engine(&source_bytes)
        .map_err(|e| LupinError::EngineDetection { source: e })?;

    let embedded_bytes = engine
        .embed(&source_bytes, &payload_bytes)
        .map_err(|e| LupinError::EmbedFailed { source: e })?;

    write_file(out_file, &embedded_bytes).map_err(|e| LupinError::OutputFileWrite {
        path: out_file.to_path_buf(),
        source: e,
    })?;

    Ok(EmbedResult {
        src: File {
            path: src_file.to_path_buf(),
            size: source_bytes.len(),
        },
        payload: File {
            path: payload.to_path_buf(),
            size: payload_bytes.len(),
        },
        output: File {
            path: out_file.to_path_buf(),
            size: embedded_bytes.len(),
        },
        engine: engine.format_name().to_string(),
    })
}

/// Extracts hidden data from a file using the appropriate engine
pub fn extract(src_file: &Path, out_file: &Path) -> Result<ExtractResult> {
    let router = EngineRouter::new();

    let data = read_file(src_file).map_err(|e| LupinError::SourceFileRead {
        path: src_file.to_path_buf(),
        source: e,
    })?;

    let engine = router
        .detect_engine(&data)
        .map_err(|e| LupinError::EngineDetection { source: e })?;

    let payload = engine
        .extract(&data)
        .map_err(|e| LupinError::ExtractFailed { source: e })?;

    let payload_size = payload.len();
    let written_to_stdout = out_file.as_os_str() == "-";

    // Special case: "-" means write to stdout
    if written_to_stdout {
        io::stdout()
            .write_all(&payload)
            .map_err(|e| LupinError::StdoutWrite { source: e })?;
    } else {
        write_file(out_file, &payload).map_err(|e| LupinError::OutputFileWrite {
            path: out_file.to_path_buf(),
            source: e,
        })?;
    }

    Ok(ExtractResult {
        src: File {
            path: src_file.to_path_buf(),
            size: data.len(),
        },
        output: File {
            path: out_file.to_path_buf(),
            size: payload_size,
        },
        engine: engine.format_name().to_string(),
        written_to_stdout,
    })
}
