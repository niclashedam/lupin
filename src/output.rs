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

use colored::*;

/// Helper struct to handle colored output consistently
pub struct OutputFormatter {
    use_colors: bool,
    quiet: bool,
    verbose: bool,
}

impl OutputFormatter {
    pub fn new(use_colors: bool, quiet: bool, verbose: bool) -> Self {
        Self {
            use_colors,
            quiet,
            verbose,
        }
    }

    /// Print info messages (only in verbose mode, unless quiet)
    pub fn info(&self, message: &str) {
        if self.verbose && !self.quiet {
            println!("{}", message);
        }
    }

    /// Print success messages (unless quiet mode)
    pub fn success(&self, message: &str) {
        if !self.quiet {
            let colored_msg = if self.use_colors {
                message.bright_green().bold()
            } else {
                message.normal()
            };
            println!("{}", colored_msg);
        }
    }

    /// Print error messages (always, not affected by quiet mode)
    pub fn error(&self, message: &str) {
        let colored_msg = if self.use_colors {
            message.bright_red().bold()
        } else {
            message.normal()
        };
        eprintln!("{}", colored_msg);
    }

    /// Print regular output messages (suppressed in quiet mode)
    pub fn println(&self, message: &str) {
        if !self.quiet {
            println!("{}", message);
        }
    }

    /// Helper methods for colored strings (for cases where you need the ColoredString)
    pub fn header(&self, text: &str) -> ColoredString {
        if self.use_colors {
            text.bright_cyan().bold()
        } else {
            text.normal()
        }
    }

    pub fn path(&self, text: &str) -> ColoredString {
        if self.use_colors {
            text.bright_cyan()
        } else {
            text.normal()
        }
    }

    pub fn command(&self, text: &str) -> ColoredString {
        if self.use_colors {
            text.bright_white().bold()
        } else {
            text.normal()
        }
    }

    pub fn size(&self, size: usize) -> ColoredString {
        // print the size in a human-readable format
        let text = if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.2} KiB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.2} MiB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GiB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        };

        if self.use_colors {
            text.bright_magenta()
        } else {
            text.normal()
        }
    }
}
