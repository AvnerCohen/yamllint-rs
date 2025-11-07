//! Integration tests for the rule system.

use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that line-length rule works correctly
#[test]
fn test_line_length_rule() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create test file with long lines
    let content = "key1: value1\n# This line is way too long and exceeds the maximum line length limit of 80 characters\nkey2: value2\n";
    fs::write(&test_file, content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("line too long"))
        .stdout(predicate::str::contains("> 80 characters"));
}

/// Test that trailing-spaces rule works correctly
#[test]
fn test_trailing_spaces_rule() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create test file with trailing spaces
    let content = "key1: value1   \nkey2: value2\t\t\nkey3: value3\n";
    fs::write(&test_file, content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("trailing spaces"))
        .stdout(predicate::str::contains("3 trailing characters"));
}

/// Test that both rules work together
#[test]
fn test_multiple_rules() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create test file with both issues
    let content = "key1: value1   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\nkey2: value2\t\t\n";
    fs::write(&test_file, content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("trailing spaces"))
        .stdout(predicate::str::contains("line too long"));
    // Note: Issue count may vary depending on enabled rules
    // .stdout(predicate::str::contains("Found"));
}

/// Test that clean files pass all rules
#[test]
fn test_clean_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create clean test file
    let content = "---\nkey1: value1\nkey2: value2\n# Short comment\n";
    fs::write(&test_file, content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    cmd.assert().success().stdout(predicate::str::is_empty());
}

/// Test that rules work with recursive directory processing
#[test]
fn test_rules_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    let file1 = temp_dir.path().join("file1.yaml");
    let file2 = sub_dir.join("file2.yaml");

    // Create files with different issues
    fs::write(&file1, "key1: value1   \n").unwrap(); // Trailing spaces
    fs::write(
        &file2,
        "# This line is way too long and exceeds the maximum line length limit of 80 characters\n",
    )
    .unwrap(); // Line length

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--recursive")
        .arg(temp_dir.path().to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("trailing spaces"))
        .stdout(predicate::str::contains("line too long"));
}

/// Test that verbose output shows processing information
#[test]
fn test_verbose_output() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    let content = "---\nkey1: value1\n";
    fs::write(&test_file, content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--verbose").arg(test_file.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing file:"))
        .stdout(predicate::str::contains("No issues found"));
}

/// Test that rules work with different file extensions
#[test]
fn test_different_extensions() {
    let temp_dir = TempDir::new().unwrap();
    let yaml_file = temp_dir.path().join("test.yaml");
    let yml_file = temp_dir.path().join("test.yml");

    // Create files with issues
    fs::write(&yaml_file, "key1: value1   \n").unwrap();
    fs::write(&yml_file, "key2: value2   \n").unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--recursive")
        .arg(temp_dir.path().to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("trailing spaces"))
        .stdout(predicate::str::contains("test.yaml"))
        .stdout(predicate::str::contains("test.yml"));
}

/// Test that rules handle empty files correctly
#[test]
fn test_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    fs::write(&test_file, "").unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    cmd.assert().success().stdout(predicate::str::is_empty());
}

/// Test that rules handle files with only whitespace
#[test]
fn test_whitespace_only_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    fs::write(&test_file, "   \n\t\t\n   \n").unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("trailing spaces"));
}

/// Test that severity configuration works with fix mode
#[test]
fn test_severity_with_fix_mode() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    // Create test file with trailing spaces (fixable) and long line (not fixable)
    let content = "key: value   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\n";
    fs::write(&test_file, content).unwrap();

    // Create configuration with different severities
    let config_content = r#"
global:
  enable_all_rules: true
rules:
  line-length:
    max: 80
severity:
  line-length: warning
  trailing-spaces: info
"#;
    fs::write(&config_file, config_content).unwrap();

    // Run with fix mode
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg("--fix")
        .arg(test_file.to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("non-fixable issues"));
}
