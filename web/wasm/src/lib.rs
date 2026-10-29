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

//! WebAssembly bindings for the website. Not published: it lives outside the `lupin` crate so the
//! published crate does not need a `cdylib` target.

use lupin::error::LupinError;
use lupin::operations;
use lupin::EmbedMode;
use wasm_bindgen::prelude::*;

/// Embeds `payload` in `source` using capacity mode
#[wasm_bindgen]
pub fn embed(source: &[u8], payload: &[u8]) -> Result<Vec<u8>, JsError> {
    operations::embed(source, payload, EmbedMode::Capacity)
        .map(|(data, _)| data)
        .map_err(|e| JsError::new(&e.to_string()))
}

/// Extracts the hidden payload from `source`, or `undefined` if there is none
#[wasm_bindgen]
pub fn extract(source: &[u8]) -> Result<Option<Vec<u8>>, JsError> {
    match operations::extract(source) {
        Ok((data, _)) => Ok(Some(data)),
        Err(
            LupinError::PdfNoHiddenData
            | LupinError::PngNoHiddenData
            | LupinError::JpegNoHiddenData
            | LupinError::MkvNoHiddenData,
        ) => Ok(None),
        Err(e) => Err(JsError::new(&e.to_string())),
    }
}
