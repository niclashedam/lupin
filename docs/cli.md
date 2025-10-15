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

### Embed a payload
```bash
lupin embed source.pdf payload.txt output.pdf
```

### Extract hidden payload
```bash
lupin extract output.pdf payload.txt
```

# Extract to stdout (useful for piping)
```bash
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
lupin embed report.pdf vacation_photo.jpg boring_report.pdf
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

### Scripting Examples

**Batch processing:**
```bash
#!/bin/bash
for pdf in *.pdf; do
    if lupin --quiet extract "$pdf" "/tmp/extracted_$(basename "$pdf")"; then
        echo "Found hidden data in $pdf"
    fi
done
```

**Check if PDF contains hidden data:**
```bash
if lupin --quiet extract suspicious.pdf /dev/null 2>/dev/null; then
    echo "PDF contains hidden data"
else
    echo "PDF is clean"
fi
```

## Help and Version

```bash
lupin --help         # Show full help
lupin --version      # Show version information
lupin embed --help   # Show help for embed command
lupin extract --help # Show help for extract command
```

## Exit Codes

- `0`: Success
- `1`: Error (file not found, unsupported format, etc.)

## Error Handling

Lupin provides clear error messages:

```bash
# File not found
lupin embed nonexistent.pdf payload.txt output.pdf
# Error: SourceFileRead { path: "nonexistent.pdf", source: Os { code: 2, kind: NotFound, message: "No such file or directory" } }

# Unsupported format
lupin embed document.txt payload.txt output.txt
# Error: Io { source: Custom { kind: Unsupported, error: "Unsupported file format - no matching engine found" } }
```