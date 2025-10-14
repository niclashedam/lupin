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

//! Help command handler

use super::CommandHandler;
use crate::output::OutputFormatter;
use crate::EngineRouter;
use std::error::Error;

pub struct HelpCommand {
    pub program_name: String,
}

impl CommandHandler for HelpCommand {
    fn execute(&self, formatter: &OutputFormatter) -> Result<(), Box<dyn Error>> {
        let supported = EngineRouter::new()
            .engines
            .iter()
            .map(|e| format!("{} ({})", e.format_ext(), e.format_name()))
            .collect::<Vec<_>>()
            .join(", ");

        println!(
            "{}\n\
             {1} [OPTIONS] <COMMAND> [ARGS...]\n\
             \n\
             {2}\n\
             {1} embed   <source.pdf> <payload.bin> <output.pdf>\n\
             {1} extract <pdf-with-payload.pdf> <output.bin>\n\
             \n\
             {3}\n\
             -h, --help        Show this help message\n\
             --no-colors       Disable colored output\n\
             -q, --quiet       Suppress all output except errors\n\
             -v, --verbose     Enable verbose output\n\
             --version         Show version information\n\
             \n\
             {4}\n\
             {supported}",
            formatter.header("Usage:"),
            self.program_name,
            formatter.header("Commands:"),
            formatter.header("Options:"),
            formatter.header("Supported formats:"),
        );
        Ok(())
    }
}
