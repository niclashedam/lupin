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
}

impl OutputFormatter {
    pub fn new(use_colors: bool, quiet: bool) -> Self {
        Self { use_colors, quiet }
    }

    pub fn header(&self, text: &str) -> ColoredString {
        if self.use_colors {
            text.bright_cyan().bold()
        } else {
            text.normal()
        }
    }

    pub fn error(&self, text: &str) -> ColoredString {
        if self.use_colors {
            text.bright_red().bold()
        } else {
            text.normal()
        }
    }

    pub fn success(&self, text: &str) -> ColoredString {
        if self.use_colors {
            text.bright_green().bold()
        } else {
            text.normal()
        }
    }

    pub fn info(&self, text: &str) -> ColoredString {
        if self.use_colors {
            text.bright_blue()
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

    /// Print verbose messages only if verbose is enabled and not quiet
    pub fn verbose_println(&self, verbose: bool, message: &str) {
        if verbose && !self.quiet {
            eprintln!("{}", message);
        }
    }

    /// Always print errors (not affected by quiet mode)
    pub fn error_println(&self, message: &str) {
        eprintln!("{}", message);
    }

    /// Print regular output messages (suppressed in quiet mode)
    pub fn println(&self, message: &str) {
        if !self.quiet {
            println!("{}", message);
        }
    }
}
