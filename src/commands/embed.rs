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

//! Embed command handler

use super::CommandHandler;
use crate::operations;
use crate::output::OutputFormatter;
use std::error::Error;
use std::path::PathBuf;

pub struct EmbedCommand {
    pub src: PathBuf,
    pub payload: PathBuf,
    pub output: PathBuf,
}

impl CommandHandler for EmbedCommand {
    fn execute(&self, formatter: &OutputFormatter) -> Result<(), Box<dyn Error>> {
        // Print file paths in verbose mode
        formatter.info(&format!(
            "Input files: {}, {}, {}",
            formatter.path(&self.src.display().to_string()),
            formatter.path(&self.payload.display().to_string()),
            formatter.path(&self.output.display().to_string())
        ));

        // Execute the embed operation
        operations::embed(&self.src, &self.payload, &self.output, formatter)?;

        // Print success message
        formatter.success("Successfully embedded payload into PDF.");

        Ok(())
    }
}
