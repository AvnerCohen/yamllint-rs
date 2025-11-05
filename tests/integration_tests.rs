use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_help_output() {
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A YAML linter written in Rust"));
}

#[test]
fn test_version_output() {
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("yamllint-rs"));
}

#[test]
fn test_no_args_shows_hello_world() {
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Hello from yamllint-rs! 🦀"));
}

#[test]
fn test_config_flag_lowercase() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    let test_content = "key: value   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\n";
    fs::write(&test_file, test_content).unwrap();

    let config_content = r#"
global:
  default_severity: Error
rules:
  line-length:
    enabled: true
    severity: Error
    settings:
      max_length: 100
"#;
    fs::write(&config_file, config_content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("-c")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("error"));
}

#[test]
fn test_config_flag_uppercase() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    let test_content = "key: value   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\n";
    fs::write(&test_file, test_content).unwrap();

    let config_content = r#"
global:
  default_severity: Error
rules:
  line-length:
    enabled: true
    severity: Error
    settings:
      max_length: 100
"#;
    fs::write(&config_file, config_content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("-C")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("error"));
}

#[test]
fn test_config_respects_disabled_rule() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    let test_content = "---\n  value: 123\n";
    fs::write(&test_file, test_content).unwrap();

    let config_content = r#"
global:
  default_severity: Error
rules:
  document-start:
    enabled: false
  indentation:
    enabled: true
"#;
    fs::write(&config_file, config_content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("-C")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    assert!(
        !stdout.contains("document-start"),
        "document-start should be disabled"
    );
}

#[test]
fn test_config_with_settings() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    let test_content = "key: this_is_a_very_long_line_that_exceeds_maximum_length\n";
    fs::write(&test_file, test_content).unwrap();

    let config_content = r#"
global:
  default_severity: Error
rules:
  line-length:
    enabled: true
    severity: Error
    settings:
      max_length: 20
      allow_non_breakable_words: false
"#;
    fs::write(&config_file, config_content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("-c")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    assert!(
        stdout.contains("line-length"),
        "line-length rule should report issue"
    );
}

#[test]
fn test_config_with_recursive_flag() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    let test_content = "key: value\n";
    fs::write(&test_file, test_content).unwrap();

    let config_content = r#"
global:
  default_severity: Error
rules:
  document-start:
    enabled: false
"#;
    fs::write(&config_file, config_content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("-c")
        .arg(config_file.to_str().unwrap())
        .arg("-r")
        .arg(test_dir.to_str().unwrap());

    cmd.assert().success();
}

#[test]
fn test_indentation_ignore_pattern_not_implemented() {
    // This test demonstrates that ignore patterns are NOT implemented
    // The config specifies: ignore: account_settings/
    // But yamllint-rs still reports indentation errors in those files
    let temp_dir = TempDir::new().unwrap();
    let account_settings_dir = temp_dir.path().join("account_settings");
    fs::create_dir_all(&account_settings_dir).unwrap();

    let test_file = account_settings_dir.join("config.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    // Create a file with indentation issue (should be ignored per config)
    let test_content = "cell_id: '0000'\nwd_tenants:\n- airliquidehr\n";
    fs::write(&test_file, test_content).unwrap();

    // Config with ignore pattern for indentation rule
    let config_content = r#"
global:
  default_severity: Error
rules:
  indentation:
    enabled: true
    indent-sequences: whatever
    ignore: |
      account_settings/
"#;
    fs::write(&config_file, config_content).unwrap();

    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("-c")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // BUG: When ignore patterns work correctly, this file should be ignored
    // and NO errors should be reported. However, yamllint-rs currently
    // doesn't implement ignore patterns, so it still reports the error.
    println!("Output: {}", stdout);

    // This test FAILS because ignore patterns are not implemented
    // Expected: No errors (file should be ignored)
    // Actual: Reports "wrong indentation" error
    assert!(!stdout.contains("wrong indentation"), 
            "BUG: File in account_settings/ should be ignored. yamllint-rs currently does NOT respect ignore patterns");
    println!("SUCCESS: Ignore patterns are working correctly!");
}
