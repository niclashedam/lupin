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

use clap::{Parser, Subcommand};
use lupin::error::Result;
use lupin::operations::{self, EmbedResult, ExtractResult};
use std::path::PathBuf;

/// A blazing-fast steganography tool for concealing data inside PDF files
#[derive(Parser, Debug)]
#[command(name = "lupin")]
#[command(version, about, long_about = None)]
struct CliArgs {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Suppress output (quiet mode)
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Command {
    /// Embed payload data into a file
    Embed {
        /// Source file to embed data into
        src: PathBuf,
        /// Payload file to embed
        payload: PathBuf,
        /// Output file path
        output: PathBuf,
    },
    /// Extract hidden data from a file
    Extract {
        /// Source file to extract from
        src: PathBuf,
        /// Output file path (use "-" for stdout)
        output: PathBuf,
    },
}

/// Print verbose messages (only in verbose mode, unless quiet)
fn print_verbose(message: &str, verbose: bool, quiet: bool) {
    if verbose && !quiet {
        println!("{}", message);
    }
}

/// Print success messages (unless quiet mode)
fn print_success(message: &str, quiet: bool) {
    if !quiet {
        println!("{}", message);
    }
}

/// Format file size in human-readable format
fn format_size(size: usize) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.2} KiB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.2} MiB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GiB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Handle embed result output
fn handle_embed_result(result: &EmbedResult, verbose: bool, quiet: bool) {
    print_verbose(&format!("Using {} engine", result.engine), verbose, quiet);

    print_verbose(
        &format!(
            "Embedded {} ({}) into {} ({}) -> {} ({}), {}% increase",
            result.payload.path.display(),
            format_size(result.payload.size),
            result.src.path.display(),
            format_size(result.src.size),
            result.output.path.display(),
            format_size(result.output.size),
            ((result.output.size as f64 / result.src.size as f64 - 1.0) * 100.0).round()
        ),
        verbose,
        quiet,
    );

    print_success("Successfully embedded payload into PDF.", quiet);
}

/// Handle extract result output
fn handle_extract_result(result: &ExtractResult, verbose: bool, quiet: bool) {
    print_verbose("Detecting appropriate engine...", verbose, quiet);
    print_verbose(&format!("Using {} engine", result.engine), verbose, quiet);

    if result.written_to_stdout {
        print_verbose(
            &format!("Extracted {} to stdout", format_size(result.output.size)),
            verbose,
            quiet,
        );
    } else {
        print_verbose(
            &format!(
                "Extracted {} from {} to {}",
                format_size(result.output.size),
                result.src.path.display(),
                result.output.path.display(),
            ),
            verbose,
            quiet,
        );
    }

    print_success("Successfully extracted payload from PDF.", quiet);
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = CliArgs::parse();

    // If no command is provided, clap will handle help/error automatically
    let command = match args.command {
        Some(cmd) => cmd,
        None => {
            // This shouldn't happen with clap, but just in case
            return Ok(());
        }
    };

    // Print verbose startup messages
    print_verbose("Verbose mode enabled", args.verbose, args.quiet);

    // Execute the appropriate command directly
    match command {
        Command::Embed {
            src,
            payload,
            output,
        } => {
            print_verbose("Running command: embed", args.verbose, args.quiet);

            // Print file paths in verbose mode
            print_verbose(
                &format!(
                    "Input files: {}, {}, {}",
                    src.display(),
                    payload.display(),
                    output.display()
                ),
                args.verbose,
                args.quiet,
            );

            // Execute the embed operation and handle result
            let result = operations::embed(&src, &payload, &output)?;
            handle_embed_result(&result, args.verbose, args.quiet);
        }
        Command::Extract { src, output } => {
            print_verbose("Running command: extract", args.verbose, args.quiet);

            // Print file paths in verbose mode
            print_verbose(
                &format!("Input files: {}, {}", src.display(), output.display()),
                args.verbose,
                args.quiet,
            );

            // Execute the extract operation and handle result
            let result = operations::extract(&src, &output)?;
            handle_extract_result(&result, args.verbose, args.quiet);
        }
    }

    Ok(())
}
