//! Integration tests for the --fix functionality.

use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_fix_trailing_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create test file with trailing spaces
    let content = "key1: value1   \nkey2: value2\t\t\nkey3: value3\n";
    fs::write(&test_file, content).unwrap();

    // Run with --fix
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--fix").arg(test_file.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Fixed"))
        .stdout(predicate::str::contains("fixable"));

    // Verify the file was actually fixed
    let fixed_content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(
        fixed_content,
        "---\nkey1: value1\nkey2: value2\nkey3: value3\n"
    );

    // Run again to verify no remaining issues (all were fixed)
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    // Note: indentation rule is disabled by default to avoid false positives
    // The output may contain warnings but should not contain errors
    cmd.assert().success().stderr(predicate::str::is_empty()); // Check stderr, not stdout
}

/// Test that non-fixable issues are reported but not fixed
#[test]
fn test_fix_non_fixable_issues() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create test file with line length issues (non-fixable)
    let content = "key1: value1\n# This line is way too long and exceeds the maximum line length limit of 80 characters\nkey2: value2\n";
    fs::write(&test_file, content).unwrap();

    // Run with --fix
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--fix").arg(test_file.to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("Fixed"))
        .stdout(predicate::str::contains("remaining"));

    // Verify the file was NOT changed (line length issues can't be auto-fixed)
    let unchanged_content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(unchanged_content, "---\nkey1: value1\n# This line is way too long and exceeds the maximum line length limit of 80 characters\nkey2: value2\n");
}

/// Test mixed fixable and non-fixable issues
#[test]
fn test_fix_mixed_issues() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create test file with both trailing spaces (fixable) and line length (non-fixable)
    let content = "key1: value1   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\nkey2: value2\t\t\nkey3: value3\n";
    fs::write(&test_file, content).unwrap();

    // Verify file has issues before fix
    let content_before = fs::read_to_string(&test_file).unwrap();
    assert!(content_before.contains("   ")); // Trailing spaces
    assert!(content_before.contains("\t\t")); // Trailing tabs
    assert!(content_before.contains("way too long")); // Long line

    // Run with --fix
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--fix").arg(test_file.to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("Fixed"))
        .stdout(predicate::str::contains("fixable"));

    // Verify trailing spaces were fixed but line length issue remains
    let fixed_content = fs::read_to_string(&test_file).unwrap();
    assert!(fixed_content.contains("key1: value1\n")); // Trailing spaces removed
    assert!(fixed_content.contains("key2: value2\n")); // Trailing tabs removed
    assert!(fixed_content.contains("way too long")); // Line length issue still there
}

/// Test that --fix works with recursive directory processing
#[test]
fn test_fix_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    let file1 = temp_dir.path().join("file1.yaml");
    let file2 = sub_dir.join("file2.yaml");

    fs::write(&file1, "key1: value1   \n").unwrap();
    fs::write(&file2, "key2: value2\t\t\n").unwrap();

    let content1_before = fs::read_to_string(&file1).unwrap();
    let content2_before = fs::read_to_string(&file2).unwrap();
    assert!(content1_before.contains("   "));
    assert!(content2_before.contains("\t\t"));

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--fix")
        .arg("--recursive")
        .arg("--verbose")
        .arg(temp_dir.path().to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing directory"))
        .stdout(predicate::str::contains("Fixed"));

    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();
    assert_eq!(content1, "---\nkey1: value1\n");
    assert_eq!(content2, "---\nkey2: value2\n");
}

/// Test that --fix works with verbose output
#[test]
fn test_fix_verbose() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    let content = "key1: value1   \nkey2: value2\n";
    fs::write(&test_file, content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--fix")
        .arg("--verbose")
        .arg(test_file.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing file:"))
        .stdout(predicate::str::contains("Fixed"));
}

/// Test that files with no issues are not modified
#[test]
fn test_fix_clean_file() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    let content = "---\nkey1: value1\nkey2: value2\n";
    fs::write(&test_file, content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--fix")
        .arg("--verbose")
        .arg(test_file.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing file:"));

    let modified_content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(modified_content, "---\nkey1: value1\nkey2: value2\n");
}

#[test]
fn test_fix_help() {
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--fix"))
        .stdout(predicate::str::contains("Automatically fix fixable issues"));
}

#[test]
fn test_rule_fix_capabilities() {
    // Test that trailing-spaces rule can fix
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    let content = "key1: value1   \n";
    fs::write(&test_file, content).unwrap();

    // Run without --fix to see the issue
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    cmd.assert()
        .code(1)
        .stdout(predicate::str::contains("trailing spaces"));

    // Run with --fix to fix it
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--fix").arg(test_file.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Fixed"));
}
