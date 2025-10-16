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

use clap::{Parser, Subcommand, ValueEnum};
use log::{debug, error, info, warn};
use lupin::error::{LupinError, Result};
use lupin::operations;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

/// Log level for controlling output verbosity
#[derive(Debug, Clone, ValueEnum)]
enum LogLevel {
    /// Only show errors
    Error,
    /// Show warnings and errors
    Warn,
    /// Show informational messages, warnings, and errors (default)
    Info,
    /// Show debug information and all other messages
    Debug,
}

/// A blazing-fast steganography tool for concealing data inside PDF files
#[derive(Parser, Debug)]
#[command(name = "lupin")]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct CliArgs {
    /// Set log level explicitly
    #[arg(long, value_enum)]
    log_level: Option<LogLevel>,

    /// Enable verbose output (shorthand for --log-level debug)
    #[arg(short, long)]
    verbose: bool,

    /// Suppress normal output (shorthand for --log-level error)
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Command,
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

/// Initialize logging based on CLI flags
fn init_logging(log_level: Option<LogLevel>, verbose: bool, quiet: bool) {
    let level = if let Some(ref level) = log_level {
        match level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
        }
    } else if quiet {
        log::LevelFilter::Error
    } else if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    TermLogger::init(
        level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .ok(); // Ignore error if logger is already initialized

    // Warn if both explicit log-level and shorthand flags are used
    if log_level.is_some() && (verbose || quiet) {
        warn!("Explicit --log-level overrides --verbose and --quiet flags");
    }
}
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

/// Handle embed command
fn handle_embed(src: PathBuf, payload: PathBuf, output: PathBuf) -> Result<()> {
    debug!("Running command: embed");
    debug!(
        "Source: {}, Payload: {}, Output: {}",
        src.display(),
        payload.display(),
        output.display()
    );

    // Read files
    let source_data = fs::read(&src).map_err(|e| LupinError::SourceFileRead {
        path: src,
        source: e,
    })?;
    let payload_data = fs::read(&payload).map_err(|e| LupinError::PayloadFileRead {
        path: payload,
        source: e,
    })?;

    // Process
    let (embedded_data, result) = operations::embed(&source_data, &payload_data)?;

    // Write output
    fs::write(&output, &embedded_data).map_err(|e| LupinError::OutputFileWrite {
        path: output.clone(),
        source: e,
    })?;

    // Display results
    debug!("Using {} engine", result.engine);
    info!(
        "Embedded payload into {} source → {} output (+{:.0}%)",
        format_size(result.source_size),
        format_size(result.output_size),
        ((result.output_size as f64 / result.source_size as f64 - 1.0) * 100.0).round()
    );

    Ok(())
}

/// Handle extract command
fn handle_extract(src: PathBuf, output: PathBuf) -> Result<()> {
    debug!("Running command: extract");
    debug!("Source: {}, Output: {}", src.display(), output.display());

    // Read file
    let source_data = fs::read(&src).map_err(|e| LupinError::SourceFileRead {
        path: src,
        source: e,
    })?;

    // Process
    let (payload_data, result) = operations::extract(&source_data)?;

    // Write output
    let written_to_stdout = output.as_os_str() == "-";
    if written_to_stdout {
        io::stdout()
            .write_all(&payload_data)
            .map_err(|e| LupinError::StdoutWrite { source: e })?;
    } else {
        fs::write(&output, &payload_data).map_err(|e| LupinError::OutputFileWrite {
            path: output,
            source: e,
        })?;
    }

    // Display results
    debug!("Using {} engine", result.engine);
    if written_to_stdout {
        debug!("Extracted {} to stdout", format_size(result.payload_size));
    } else {
        debug!("Extracted {} from source", format_size(result.payload_size));
    }

    info!("Successfully extracted payload from PDF.");
    Ok(())
}

fn main() -> ExitCode {
    let args = CliArgs::parse();

    // Initialize logging based on verbosity flags
    init_logging(args.log_level, args.verbose, args.quiet);

    debug!("Verbose mode enabled");

    // Execute command and handle errors with pretty printing
    let result = match args.command {
        Command::Embed {
            src,
            payload,
            output,
        } => handle_embed(src, payload, output),
        Command::Extract { src, output } => handle_extract(src, output),
    };

    // Handle errors with pretty printing using the log system
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // print the user-friendly error message (from thiserror Display)
            error!("{}", error);

            // Log detailed debug information including source chain
            error!("{:?}", error);
            ExitCode::FAILURE
        }
    }
}
