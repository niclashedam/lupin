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

//! Extract command handler

use super::CommandHandler;
use crate::operations;
use crate::output::OutputFormatter;
use std::error::Error;
use std::path::PathBuf;

pub struct ExtractCommand {
    pub src: PathBuf,
    pub output: PathBuf,
}

impl CommandHandler for ExtractCommand {
    fn execute(&self, formatter: &OutputFormatter, verbose: bool) -> Result<(), Box<dyn Error>> {
        // Print file paths in verbose mode
        formatter.verbose_println(
            verbose,
            &format!(
                "Source PDF: {}",
                formatter.path(&self.src.display().to_string())
            ),
        );
        formatter.verbose_println(
            verbose,
            &format!(
                "Output file: {}",
                formatter.path(&self.output.display().to_string())
            ),
        );

        // Execute the extract operation
        operations::extract(&self.src, &self.output, formatter)?;

        // Print success message in verbose mode
        formatter.verbose_println(
            verbose,
            &formatter
                .success("Successfully extracted payload from PDF")
                .to_string(),
        );

        Ok(())
    }
}
