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

//! Simple file I/O operations

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

/// Read an entire file into a byte vector
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    fs::read(path)
}

/// Write data to a file, creating it if it doesn't exist
pub fn write_file<P: AsRef<Path>>(path: P, data: &[u8]) -> io::Result<()> {
    File::create(path)?.write_all(data)
}
