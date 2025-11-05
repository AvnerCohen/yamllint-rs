# yamllint-rs

A YAML linter written in Rust, inspired by the Python [yamllint](https://github.com/adrienverge/yamllint) package.

## Credits

This project is a Rust implementation of [yamllint](https://github.com/adrienverge/yamllint) by [Adrien Vergé](https://github.com/adrienverge). The original yamllint serves as the reference implementation for all rules, configurations, and behavior. This Rust version aims to provide the same functionality with improved performance through parallel processing and native compilation.

## Features

- **Fast YAML linting** powered by Rust with parallel processing
- **23 configurable rules** covering formatting, content, and YAML-specific features
- **Automatic fixes** for fixable issues with `--fix` flag
- **Configuration support** with automatic discovery of `.yamllint` files
- **Compatible** with original yamllint configuration format
- **Git-aware** processing that respects `.gitignore` files
- **Colored output** with automatic detection of terminal capabilities
- **Command-line interface** with flexible options
- **Library API** for integration into Rust projects

## Installation

### From Source

```bash
git clone https://github.com/AvnerCohen/yamllint-rs
cd yamllint-rs
cargo build --release
```

### Using Cargo

```bash
cargo install yamllint-rs
```

## Usage

### Command Line

```bash
# Lint a single file
yamllint-rs file.yaml

# Lint multiple files (processed in parallel)
yamllint-rs file1.yaml file2.yaml file3.yaml

# Recursive directory processing
yamllint-rs --recursive directory/

# Verbose output
yamllint-rs --verbose file.yaml

# Use custom config file
yamllint-rs --config .yamllint.yaml file.yaml

# Automatically fix fixable issues
yamllint-rs --fix file.yaml

# Set output format (standard, colored, or auto)
yamllint-rs --format colored file.yaml

# Combine options
yamllint-rs -r --verbose --fix directory/
```

### Command-Line Options

- `files` - YAML file(s) to lint (positional arguments)
- `-r, --recursive` - Process directories recursively
- `-v, --verbose` - Enable verbose output
- `-c, --config <path>` - Path to configuration file
- `--fix` - Automatically fix fixable issues
- `-f, --format <format>` - Output format: `standard`, `colored`, or `auto` (default: `auto`)

### Configuration

yamllint-rs automatically discovers configuration files by searching for `.yamllint` in the current directory and parent directories. You can also specify a custom path with `--config`.

```bash
# Automatic discovery (searches for .yamllint in current and parent dirs)
yamllint-rs file.yaml

# Explicit config file
yamllint-rs --config custom-config.yaml file.yaml
```

The tool supports both the original yamllint configuration format and the native format. See the [Rules.md](Rules.md) file for detailed rule documentation.

Example `.yamllint` configuration:

```yaml
rules:
  line-length:
    max: 120
  indentation:
    spaces: 4
    ignore: |
      *.template.yaml
      generated/
  truthy:
    allowed-values: ['true', 'false', 'yes', 'no']
```

### Library API

```rust
use yamllint_rs::{FileProcessor, ProcessingOptions, OutputFormat};

// Process a single file
let options = ProcessingOptions {
    recursive: false,
    verbose: false,
    output_format: OutputFormat::Colored,
};

let processor = FileProcessor::with_default_rules(options);
let result = processor.process_file("path/to/file.yaml")?;

for (issue, rule_name) in result.issues {
    println!("{}:{} - {} ({})", 
        issue.line, 
        issue.column, 
        issue.message, 
        rule_name
    );
}

// Process directory recursively
let processor = FileProcessor::with_default_rules(options);
processor.process_directory("directory/")?;

// Use custom configuration
use yamllint_rs::{load_config, discover_config_file};

if let Some(config_path) = discover_config_file() {
    let config = load_config(config_path)?;
    let processor = FileProcessor::with_config(options, config);
    processor.process_file("file.yaml")?;
}

// Fix mode
let processor = FileProcessor::with_fix_mode(options);
processor.process_file("file.yaml")?;
```

## Development

```bash
# Build the project
cargo build

# Build optimized release binary
cargo build --release

# Run tests
cargo test

# Run with debug output
cargo run -- --verbose file.yaml

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Run benchmarks
cargo bench
```

## Features in Detail

### Parallel Processing

yamllint-rs processes multiple files in parallel for better performance. Files are automatically processed in parallel when:
- Multiple files are specified on the command line
- Recursive directory processing finds multiple YAML files

### Automatic Fixes

Many rules support automatic fixes. When using `--fix`, the tool will:
1. Apply all fixable corrections
2. Write the fixed content back to the file
3. Report remaining non-fixable issues

### Git Integration

When processing directories recursively, yamllint-rs respects `.gitignore` files using the `ignore` crate, automatically skipping files that would be ignored by Git.

### Output Formats

- **auto** (default): Automatically detects terminal capabilities and NO_COLOR environment variable
- **colored**: Always use colored output with ANSI codes
- **standard**: Plain text output without colors

## Supported Rules

yamllint-rs supports all 23 rules from the original yamllint. See [Rules.md](Rules.md) for complete documentation.

### Enabled by Default
- braces, brackets, colons, commas, hyphens
- line-length, indentation, trailing-spaces
- comments, comments-indentation
- document-start
- empty-lines, new-lines
- new-line-at-end-of-file
- key-duplicates, anchors
- truthy

### Disabled by Default
- document-end
- quoted-strings
- empty-values
- float-values
- octal-values
- key-ordering

## License

MIT
