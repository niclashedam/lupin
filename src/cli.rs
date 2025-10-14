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

//! Command-line interface parsing and argument handling

use std::env;
use std::path::PathBuf;

/// Parsed command line arguments
#[derive(Debug)]
pub struct CliArgs {
    pub verbose: bool,
    pub quiet: bool,
    pub use_colors: bool,
    pub command: Command,
}

/// Available commands
#[derive(Debug)]
pub enum Command {
    Help {
        program_name: String,
    },
    Version,
    Embed {
        src: PathBuf,
        payload: PathBuf,
        output: PathBuf,
    },
    Extract {
        src: PathBuf,
        output: PathBuf,
    },
}

/// Error that can occur during CLI parsing
#[derive(Debug)]
pub enum CliError {
    MissingSubcommand {
        program_name: String,
    },
    UnknownCommand {
        program_name: String,
        command: String,
    },
    WrongArgumentCount {
        program_name: String,
        command: String,
        expected: usize,
        got: usize,
    },
}

impl CliArgs {
    /// Parse command line arguments
    pub fn parse() -> Result<Self, CliError> {
        let args: Vec<String> = env::args().collect();
        let program_name = args[0].clone();

        // Parse flags first
        let mut verbose = false;
        let mut quiet = false;
        let mut use_colors = true;
        let mut show_help = false;
        let mut show_version = false;
        let mut filtered_args = Vec::new();

        // First pass: collect all flags and non-flag arguments
        for arg in args.iter() {
            match arg.as_str() {
                "--version" => show_version = true,
                "-v" | "--verbose" => verbose = true,
                "-q" | "--quiet" => quiet = true,
                "--no-colors" => {
                    use_colors = false;
                    colored::control::set_override(false);
                }
                "-h" | "--help" => show_help = true,
                _ => filtered_args.push(arg.clone()),
            }
        }

        // Handle early exit commands
        if show_version {
            return Ok(CliArgs {
                verbose,
                quiet,
                use_colors,
                command: Command::Version,
            });
        }

        if show_help {
            return Ok(CliArgs {
                verbose,
                quiet,
                use_colors,
                command: Command::Help { program_name },
            });
        }

        // Need at least a subcommand
        if filtered_args.len() < 2 {
            return Err(CliError::MissingSubcommand { program_name });
        }

        // Parse the command
        let command = match filtered_args[1].as_str() {
            "embed" => {
                if filtered_args.len() != 5 {
                    return Err(CliError::WrongArgumentCount {
                        program_name,
                        command: "embed".to_string(),
                        expected: 3,
                        got: filtered_args.len() - 2,
                    });
                }
                Command::Embed {
                    src: PathBuf::from(&filtered_args[2]),
                    payload: PathBuf::from(&filtered_args[3]),
                    output: PathBuf::from(&filtered_args[4]),
                }
            }
            "extract" => {
                if filtered_args.len() != 4 {
                    return Err(CliError::WrongArgumentCount {
                        program_name,
                        command: "extract".to_string(),
                        expected: 2,
                        got: filtered_args.len() - 2,
                    });
                }
                Command::Extract {
                    src: PathBuf::from(&filtered_args[2]),
                    output: PathBuf::from(&filtered_args[3]),
                }
            }
            unknown => {
                return Err(CliError::UnknownCommand {
                    program_name,
                    command: unknown.to_string(),
                });
            }
        };

        Ok(CliArgs {
            verbose,
            quiet,
            use_colors,
            command,
        })
    }
}
