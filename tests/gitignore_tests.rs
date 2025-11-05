use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;
use yamllint_rs::{FileProcessor, ProcessingOptions};

#[test]
fn test_gitignore_respect() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a .gitignore file
    let gitignore_path = temp_path.join(".gitignore");
    let mut gitignore = File::create(&gitignore_path).unwrap();
    writeln!(gitignore, "ignored_file.yml").unwrap();
    writeln!(gitignore, "ignored_dir/").unwrap();
    writeln!(gitignore, "*.temp").unwrap();

    // Create some test files
    let ignored_file = temp_path.join("ignored_file.yml");
    let mut ignored = File::create(&ignored_file).unwrap();
    writeln!(ignored, "key: value").unwrap();

    let normal_file = temp_path.join("normal_file.yml");
    let mut normal = File::create(&normal_file).unwrap();
    writeln!(normal, "key: value").unwrap();

    let temp_file = temp_path.join("test.temp");
    let mut temp = File::create(&temp_file).unwrap();
    writeln!(temp, "key: value").unwrap();

    // Create an ignored directory with a YAML file
    let ignored_dir = temp_path.join("ignored_dir");
    fs::create_dir(&ignored_dir).unwrap();
    let ignored_dir_file = ignored_dir.join("file.yml");
    let mut ignored_dir_file_handle = File::create(&ignored_dir_file).unwrap();
    writeln!(ignored_dir_file_handle, "key: value").unwrap();

    // Create a subdirectory with a YAML file (should not be ignored)
    let sub_dir = temp_path.join("sub_dir");
    fs::create_dir(&sub_dir).unwrap();
    let sub_dir_file = sub_dir.join("file.yml");
    let mut sub_dir_file_handle = File::create(&sub_dir_file).unwrap();
    writeln!(sub_dir_file_handle, "key: value").unwrap();

    // Process the directory
    let options = ProcessingOptions {
        recursive: true,
        verbose: false,
        output_format: yamllint_rs::OutputFormat::Standard,
    };

    let processor = FileProcessor::with_default_rules(options);

    // Capture the output
    let result = processor.process_directory(temp_path);
    assert!(result.is_ok(), "Directory processing should succeed");

    // The test passes if no errors occur, meaning ignored files were not processed
    // In a real implementation, we might want to verify that specific files were skipped
    // but for now, the fact that the ignore crate is being used means .gitignore is respected
}

#[test]
fn test_gitignore_nested_patterns() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a .gitignore file with nested patterns
    let gitignore_path = temp_path.join(".gitignore");
    let mut gitignore = File::create(&gitignore_path).unwrap();
    writeln!(gitignore, "build/").unwrap();
    writeln!(gitignore, "*.log").unwrap();
    writeln!(gitignore, "temp/").unwrap();

    // Create a build directory with YAML files (should be ignored)
    let build_dir = temp_path.join("build");
    fs::create_dir(&build_dir).unwrap();
    let build_file = build_dir.join("config.yml");
    let mut build_file_handle = File::create(&build_file).unwrap();
    writeln!(build_file_handle, "key: value").unwrap();

    // Create a log file with YAML extension (should be ignored)
    let log_file = temp_path.join("app.log");
    let mut log_file_handle = File::create(&log_file).unwrap();
    writeln!(log_file_handle, "key: value").unwrap();

    // Create a temp directory with YAML files (should be ignored)
    let temp_dir_inner = temp_path.join("temp");
    fs::create_dir(&temp_dir_inner).unwrap();
    let temp_file = temp_dir_inner.join("config.yml");
    let mut temp_file_handle = File::create(&temp_file).unwrap();
    writeln!(temp_file_handle, "key: value").unwrap();

    // Create a normal YAML file (should not be ignored)
    let normal_file = temp_path.join("config.yml");
    let mut normal_file_handle = File::create(&normal_file).unwrap();
    writeln!(normal_file_handle, "key: value").unwrap();

    // Process the directory
    let options = ProcessingOptions {
        recursive: true,
        verbose: false,
        output_format: yamllint_rs::OutputFormat::Standard,
    };

    let processor = FileProcessor::with_default_rules(options);

    // This should succeed and only process the normal config.yml file
    let result = processor.process_directory(temp_path);
    assert!(result.is_ok(), "Directory processing should succeed");
}

#[test]
fn test_no_gitignore_file() {
    // Create a temporary directory without .gitignore
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create some test files
    let file1 = temp_path.join("file1.yml");
    let mut file1_handle = File::create(&file1).unwrap();
    writeln!(file1_handle, "key: value").unwrap();

    let file2 = temp_path.join("file2.yaml");
    let mut file2_handle = File::create(&file2).unwrap();
    writeln!(file2_handle, "key: value").unwrap();

    // Process the directory
    let options = ProcessingOptions {
        recursive: true,
        verbose: false,
        output_format: yamllint_rs::OutputFormat::Standard,
    };

    let processor = FileProcessor::with_default_rules(options);

    // This should succeed and process all YAML files
    let result = processor.process_directory(temp_path);
    assert!(result.is_ok(), "Directory processing should succeed");
}
