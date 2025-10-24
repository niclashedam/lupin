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

//! Error types for Lupin CLI operations

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Main error type for Lupin
#[derive(Error, Debug)]
pub enum LupinError {
    /// File I/O errors with rich context
    #[error("Failed to read source file '{path}'")]
    SourceFileRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to read payload file '{path}'")]
    PayloadFileRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to write output file '{path}'")]
    OutputFileWrite {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// I/O errors with explicit context for stdout operations
    #[error("Failed to write to stdout")]
    StdoutWrite {
        #[source]
        source: io::Error,
    },

    /// Engine detection and operation errors
    #[error("Engine detection failed: no suitable engine found for the input file format")]
    EngineDetection {
        #[source]
        source: io::Error,
    },

    #[error("Embedding operation failed")]
    EmbedFailed {
        #[source]
        source: io::Error,
    },

    #[error("Embedding operation failed. Is there already hidden data in the source file?")]
    EmbedCollision {
        #[source]
        source: io::Error,
    },

    #[error("Extraction operation failed")]
    ExtractFailed {
        #[source]
        source: io::Error,
    },

    /// PDF-specific errors
    #[error("Invalid PDF: no %%EOF marker found")]
    PdfNoEofMarker,

    #[error("No hidden data found in PDF")]
    PdfNoHiddenData,

    #[error("Corrupted hidden data in PDF")]
    PdfCorruptedData,

    /// PNG-specific errors
    #[error("Invalid PNG: no IDAT chunk found")]
    PngNoIdatChunk,

    #[error("No hidden data found in PNG")]
    PngNoHiddenData,

    #[error("Corrupted hidden data in PNG")]
    PngCorruptedData,

    /// Generic I/O error for cases where automatic conversion is desired
    #[error("I/O operation failed")]
    Io {
        #[from]
        source: io::Error,
    },
}

/// Convenient Result type alias
pub type Result<T> = std::result::Result<T, LupinError>;
