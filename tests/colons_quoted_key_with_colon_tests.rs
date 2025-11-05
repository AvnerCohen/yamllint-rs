use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Repro: single-quoted key containing a colon should NOT trigger colons spacing issues
// yamllint (python) reports 0 issues for this pattern.
// Current yamllint-rs incorrectly reports "too many spaces before colon".
#[test]
fn colons_should_not_flag_quoted_key_with_internal_colon() {
    // Use test data with quoted keys containing colons
    let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("aden_colons_false_positives.yaml");

    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");

    // Disable unrelated rules - we only want to check colons rule
    let config_content = r#"
rules:
  document-start: disable
  indentation: disable
  line-length: disable
  trailing-spaces: disable
  truthy: disable
  empty-lines: disable
"#;
    fs::write(&config_file, config_content).unwrap();

    let mut cmd = Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    // yamllint reports 0 colons issues for this file.
    // This test will FAIL until we fix the spacing calculation bug.
    cmd.assert().success().stdout(predicate::str::is_empty());
}

#[test]
fn colons_should_not_flag_components_after_quoted_key() {
    let test_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("remaining_colons_false_positives.yaml");

    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");

    let config_content = r#"
rules:
  document-start: disable
  indentation: disable
  line-length: disable
  trailing-spaces: disable
  truthy: disable
  empty-lines: disable
"#;
    fs::write(&config_file, config_content).unwrap();

    let mut cmd = Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    cmd.assert().success().stdout(predicate::str::is_empty());
}
