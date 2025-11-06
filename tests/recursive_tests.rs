//! Integration tests for recursive directory processing.

use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test that recursive processing finds and processes YAML files
#[test]
fn test_recursive_processing() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    let file1 = temp_dir.path().join("file1.yaml");
    let file2 = sub_dir.join("file2.yaml");

    fs::write(&file1, "key1: value1   \n").unwrap();
    fs::write(&file2, "key2: value2\t\t\n").unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--recursive")
        .arg("--verbose")
        .arg(temp_dir.path().to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing directory"));
}

/// Test that directories are automatically processed recursively without -r flag
#[test]
fn test_auto_recursive_directory() {
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    let file1 = temp_dir.path().join("file1.yaml");
    let file2 = sub_dir.join("file2.yaml");

    fs::write(&file1, "key1: value1   \n").unwrap();
    fs::write(&file2, "key2: value2\t\t\n").unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--verbose")
        .arg(temp_dir.path().to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing directory"));
}

/// Test that recursive processing works with --fix
#[test]
fn test_recursive_fix() {
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
    cmd.arg("--recursive")
        .arg("--fix")
        .arg("--verbose")
        .arg(temp_dir.path().to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Processing directory"))
        .stdout(predicate::str::contains("Fixed 2 issues"));

    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();
    assert_eq!(content1, "---\nkey1: value1\n");
    assert_eq!(content2, "---\nkey2: value2\n");
}
