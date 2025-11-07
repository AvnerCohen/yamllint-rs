use clap::Parser;
use rayon::prelude::*;
use std::path::Path;
use std::process;
use yamllint_rs::{discover_config_file, load_config, FileProcessor, ProcessingOptions};

#[derive(Parser)]
#[command(name = "yamllint-rs")]
#[command(about = "A YAML linter written in Rust")]
#[command(version)]
struct Cli {
    /// YAML file(s) to lint
    files: Vec<String>,

    /// Recursive directory processing
    #[arg(short, long)]
    recursive: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Configuration file path (alias for --config, -c)
    #[arg(short = 'C', long, hide = true)]
    config_upper: Option<String>,

    /// Automatically fix fixable issues
    #[arg(long)]
    fix: bool,

    /// Output format (standard, colored)
    #[arg(short, long, default_value = "auto")]
    format: String,

    /// Disable progress updates
    #[arg(long)]
    no_progress: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.files.is_empty() {
        println!("Hello from yamllint-rs! 🦀");
        println!("Usage: yamllint-rs <file1> [file2] ...");
        println!("       yamllint-rs <directory>");
        return Ok(());
    }

    let options = ProcessingOptions {
        recursive: cli.recursive,
        verbose: cli.verbose,
        output_format: yamllint_rs::detect_output_format(&cli.format),
        show_progress: !cli.no_progress,
    };

    let config_path = cli.config.as_deref().or(cli.config_upper.as_deref());
    let processor = if let Some(config_path) = config_path {
        if cli.verbose {
            println!("Loading config from: {}", config_path);
        }
        let config = load_config(config_path)?;
        if cli.fix {
            FileProcessor::with_config_and_fix_mode(options.clone(), config)
        } else {
            FileProcessor::with_config(options.clone(), config)
        }
    } else if let Some(config_path) = discover_config_file() {
        if cli.verbose {
            println!("Found config file: {}", config_path.display());
        }
        let config = load_config(config_path)?;
        if cli.fix {
            FileProcessor::with_config_and_fix_mode(options.clone(), config)
        } else {
            FileProcessor::with_config(options.clone(), config)
        }
    } else {
        if cli.fix {
            FileProcessor::with_fix_mode(options.clone())
        } else {
            FileProcessor::with_default_rules(options.clone())
        }
    };

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for path_str in &cli.files {
        let path = Path::new(path_str);
        if cli.recursive || path.is_dir() {
            directories.push(path_str);
        } else {
            files.push(path_str);
        }
    }

    let mut total_issues = 0;

    if !directories.is_empty() {
        for path in directories {
            total_issues += processor.process_directory(path)?;
        }
    }

    if !files.is_empty() {
        if files.len() > 1 {
            if cli.verbose {
                println!("Processing {} files in parallel...", files.len());
            }
            let results: Result<Vec<_>, _> = files
                .par_iter()
                .map(|file| processor.process_file(file))
                .collect();
            for result in results? {
                total_issues += result.issues.len();
            }
        } else {
            let result = processor.process_file(&files[0])?;
            total_issues += result.issues.len();
        }
    }

    if total_issues > 0 {
        process::exit(1);
    }

    Ok(())
}
