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

// Enigma is a steganography tool for hiding data in PDF files
//
// PDFs end with %%EOF, but viewers ignore anything after that.
// We append a base64-encoded payload after the EOF marker.
// Most PDF readers will still open the file normally, but we can extract our data later.

use enigma::operations::{embed, extract};
use std::env;
use std::io;
use std::path::Path;

// Show usage info when user input is incorrect
fn print_usage(program_name: &str) {
    eprintln!(
        "Usage:\n\
         {0} embed   <source.pdf> <payload.bin> <output.pdf>\n\
         {0} extract <pdf-with-payload.pdf> <output.bin>\n",
        program_name
    );
}

fn main() -> io::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Need at least a subcommand (e.g. embed)
    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    // call the appropriate operation based on subcommand
    match args[1].as_str() {
        "embed" => {
            // embed needs three parameters:
            // source.pdf      -> The PDF file to hide data in
            // payload.bin     -> The file to embed
            // output.pdf      -> The resulting PDF with embedded data

            if args.len() != 5 {
                eprintln!("Error: embed expects three arguments.");
                print_usage(&args[0]);
                std::process::exit(1);
            }
            let (src_pdf, payload, out_pdf) = (
                Path::new(&args[2]),
                Path::new(&args[3]),
                Path::new(&args[4]),
            );
            embed(src_pdf, payload, out_pdf)?;
        }
        "extract" => {
            // extract needs two parameters:
            // source.pdf         -> The PDF file with hidden data
            // output.bin         -> The file to write the extracted data to

            if args.len() != 4 {
                eprintln!("Error: extract expects two arguments.");
                print_usage(&args[0]);
                std::process::exit(1);
            }
            let (src_pdf, out_bin) = (Path::new(&args[2]), Path::new(&args[3]));
            extract(src_pdf, out_bin)?;
        }
        _ => {
            // User typed something we don't recognize
            eprintln!("Error: unknown sub‑command '{}'.", args[1]);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}
