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

// Lupin is a steganography tool for hiding data in PDF files
//
// PDFs end with %%EOF, but viewers ignore anything after that.
// We append a base64-encoded payload after the EOF marker.
// Most PDF readers will still open the file normally, but we can extract our data later.

use lupin::cli::{CliArgs, CliError, Command};
use lupin::commands::{
    embed::EmbedCommand, extract::ExtractCommand, help::HelpCommand, version::VersionCommand,
    CommandHandler,
};
use lupin::output::OutputFormatter;

fn main() {
    // Parse command line arguments
    let args = match CliArgs::parse() {
        Ok(args) => args,
        Err(error) => {
            handle_cli_error(error);
            std::process::exit(1);
        }
    };

    // Create output formatter
    let formatter = OutputFormatter::new(args.use_colors, args.quiet, args.verbose);

    // Print verbose startup messages
    formatter.info("Verbose mode enabled");

    // Create and execute the appropriate command
    let command: Box<dyn CommandHandler> = match args.command {
        Command::Help { program_name } => Box::new(HelpCommand { program_name }),
        Command::Version => Box::new(VersionCommand),
        Command::Embed {
            src,
            payload,
            output,
        } => {
            formatter.info("Running command: embed");
            Box::new(EmbedCommand {
                src,
                payload,
                output,
            })
        }
        Command::Extract { src, output } => {
            formatter.info("Running command: extract");
            Box::new(ExtractCommand { src, output })
        }
    };

    // Execute the command
    if command.execute(&formatter).is_err() {
        std::process::exit(1);
    }
}

fn handle_cli_error(error: CliError) {
    match error {
        CliError::MissingSubcommand { program_name } => {
            let formatter = OutputFormatter::new(true, false, false);
            formatter.error("Error: Missing subcommand.");
            let help_cmd = HelpCommand { program_name };
            let _ = help_cmd.execute(&formatter);
        }
        CliError::UnknownCommand {
            program_name,
            command,
        } => {
            let formatter = OutputFormatter::new(true, false, false);
            formatter.error(&format!("Error: unknown sub‑command '{}'.", command));
            let help_cmd = HelpCommand { program_name };
            let _ = help_cmd.execute(&formatter);
        }
        CliError::WrongArgumentCount {
            program_name,
            command,
            expected,
            got,
        } => {
            let formatter = OutputFormatter::new(true, false, false);
            formatter.error(&format!(
                "Error: {} expects {} arguments, got {}.",
                command, expected, got
            ));
            let help_cmd = HelpCommand { program_name };
            let _ = help_cmd.execute(&formatter);
        }
    }
}
