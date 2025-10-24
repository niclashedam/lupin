# CLI Guide

## Installation

### Build from Source
```bash
# Install
cargo install lupin

# Or build from source: git clone && cargo build --release
```

### Download Binary
Download pre-built binaries from the [releases page](https://github.com/niclashedam/lupin/releases).

## Basic Usage

Lupin automatically detects the file format and uses the appropriate steganography engine.

### Embed a payload

**PDF files** (appends data after `%%EOF` marker):
```bash
lupin embed source.pdf payload.txt output.pdf
```

**PNG files** (uses LSB steganography in pixel data):
```bash
lupin embed photo.png message.txt stego_photo.png
```

### Extract hidden payload
```bash
# Extract to a file
lupin extract output.pdf payload.txt
lupin extract stego_photo.png message.txt

# Extract to stdout (useful for piping)
lupin extract output.pdf -
```

## Logging Control

Lupin provides flexible logging and output control:

### Log Levels
```bash
lupin --log-level debug embed source.pdf payload.txt output.pdf    # Detailed debug info
lupin --log-level info embed source.pdf payload.txt output.pdf     # Normal operation info  
lupin --log-level warn embed source.pdf payload.txt output.pdf     # Warnings only
lupin --log-level error embed source.pdf payload.txt output.pdf    # Errors only
```

### Shorthand Options
```bash
lupin --verbose embed source.pdf payload.txt output.pdf            # Same as --log-level debug
lupin --quiet embed source.pdf payload.txt output.pdf              # Same as --log-level error
```

**Note**: Explicit `--log-level` takes precedence over `--verbose`/`--quiet` flags. If you use both, you'll see a warning.

### Example Output

**Verbose mode:**
```bash
lupin --verbose embed document.pdf secret.txt output.pdf
# Output:
# [DEBUG] Verbose mode enabled
# [DEBUG] Running command: embed  
# [DEBUG] Source: document.pdf, Payload: secret.txt, Output: output.pdf
# [DEBUG] Using PDF engine
# [INFO] Embedded 1.2 KiB payload into 234.5 KiB source → 235.8 KiB output (+1%)
```

**Normal mode:**
```bash
lupin embed document.pdf secret.txt output.pdf
# Output:
# [INFO] Embedded 1.2 KiB payload into 234.5 KiB source → 235.8 KiB output (+1%)
```

**Quiet mode:**
```bash
lupin --quiet embed document.pdf secret.txt output.pdf
# Output: (none, unless there's an error)
```

## Advanced Examples

### Hide Different File Types

**Text message:**
```bash
echo "Meet me at the park at 5 pm" > secret.txt
lupin embed document.pdf secret.txt innocent_looking.pdf
```

**Image:**
```bash
# Hide an image in a PDF
lupin embed report.pdf vacation_photo.jpg boring_report.pdf

# Or hide data in an image itself
lupin embed cover.png hidden.jpg stego_cover.png
```

**Archive:**
```bash
# Create a zip archive
zip -r secrets.zip confidential_folder/

# Embed the archive
lupin embed presentation.pdf secrets.zip presentation_with_secrets.pdf
```

### Extract and Process

**Extract to file:**
```bash
lupin extract presentation_with_secrets.pdf extracted_secrets.zip
```

**Extract and pipe to another command:**
```bash
lupin extract presentation_with_secrets.pdf - | unzip -
lupin extract hidden_data.pdf - | file -
```