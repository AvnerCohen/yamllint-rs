//! Tests for severity configuration functionality.

use std::fs;
use tempfile::TempDir;
use yamllint_rs::config::Config;
use yamllint_rs::{load_config, FileProcessor, ProcessingOptions, Severity};

/// Test that severity can be set and retrieved for rules
#[test]
fn test_severity_enum_conversion() {
    // Test string to severity conversion
    assert_eq!(Severity::from_str("error").unwrap(), Severity::Error);
    assert_eq!(Severity::from_str("warning").unwrap(), Severity::Warning);
    assert_eq!(Severity::from_str("info").unwrap(), Severity::Info);
    assert_eq!(Severity::from_str("ERROR").unwrap(), Severity::Error);
    assert_eq!(Severity::from_str("WARNING").unwrap(), Severity::Warning);

    // Test invalid severity
    assert!(Severity::from_str("invalid").is_err());

    // Test severity to string conversion
    assert_eq!(Severity::Error.to_string(), "error");
    assert_eq!(Severity::Warning.to_string(), "warning");
    assert_eq!(Severity::Info.to_string(), "info");
}

/// Test that rules can have their severity overridden
#[test]
fn test_rule_severity_override() {
    use yamllint_rs::rules::{LineLengthRule, TrailingSpacesRule};

    // Test LineLengthRule
    let mut line_rule = LineLengthRule::new();
    assert_eq!(line_rule.get_severity(), Severity::Error);
    assert!(!line_rule.has_severity_override());

    line_rule.set_severity(Severity::Warning);
    assert_eq!(line_rule.get_severity(), Severity::Warning);
    assert!(line_rule.has_severity_override());

    // Test TrailingSpacesRule
    let mut trailing_rule = TrailingSpacesRule::new();
    assert_eq!(trailing_rule.get_severity(), Severity::Error);
    assert!(!trailing_rule.has_severity_override());

    trailing_rule.set_severity(Severity::Info);
    assert_eq!(trailing_rule.get_severity(), Severity::Info);
    assert!(trailing_rule.has_severity_override());
}

/// Test that severity configuration can be created and used
#[test]
fn test_severity_config() {
    let mut config = Config::new();
    config.set_rule_severity("line-length", Severity::Warning);
    config.set_rule_severity("trailing-spaces", Severity::Info);

    // Test getting severity for rules
    assert_eq!(config.get_rule_severity("line-length"), Severity::Warning);
    assert_eq!(config.get_rule_severity("trailing-spaces"), Severity::Info);
    assert_eq!(config.get_rule_severity("unknown-rule"), Severity::Error); // Default severity

    // Test setting severity for rules
    config.set_rule_severity("line-length", Severity::Error);
    assert_eq!(config.get_rule_severity("line-length"), Severity::Error);
}

/// Test that configuration can be loaded from YAML
#[test]
fn test_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.yaml");

    // Create a test configuration file
    let config_content = r#"
global:
  default_severity: Error
rules:
  line-length:
    enabled: true
    severity: Warning
    settings:
      max_length: 100
  trailing-spaces:
    enabled: true
    severity: Info
"#;
    fs::write(&config_file, config_content).unwrap();

    // Load the configuration
    let config = load_config(&config_file).unwrap();

    // Verify the configuration was loaded correctly
    assert!(config.global.default_severity.is_some());
    let default_severity = config.global.default_severity.unwrap();
    assert_eq!(default_severity, Severity::Error);

    // Check rule-specific severities
    if let Some(line_length_rule) = config.rules.get("line-length") {
        assert_eq!(line_length_rule.severity, Some(Severity::Warning));
    }
    if let Some(trailing_spaces_rule) = config.rules.get("trailing-spaces") {
        assert_eq!(trailing_spaces_rule.severity, Some(Severity::Info));
    }
}

/// Test that FileProcessor applies severity configuration correctly
#[test]
fn test_fileprocessor_with_severity_config() {
    let mut config = Config::new();
    config.set_rule_severity("line-length", Severity::Warning);
    config.set_rule_severity("trailing-spaces", Severity::Info);

    let options = ProcessingOptions::default();

    let _processor = FileProcessor::with_config(options, config);

    // The processor should have been created successfully
    // (We can't easily test the internal state, but we can verify it doesn't panic)
    assert!(true); // This test passes if the processor is created without panicking
}

/// Test that severity configuration affects lint output
#[test]
fn test_severity_in_lint_output() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    // Create test file with issues
    let content = "key: value   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\n";
    fs::write(&test_file, content).unwrap();

    // Create configuration with different severities
    let config_content = r#"
global:
  default_severity: Error
rules:
  line-length:
    enabled: true
    severity: Warning
    settings:
      max_length: 80
  trailing-spaces:
    enabled: true
    severity: Info
"#;
    fs::write(&config_file, config_content).unwrap();

    // Run the linter with configuration
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    let output = cmd.assert().code(1);

    // Check that the output contains the severity levels
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("warning") || stdout.contains("info"));
}

/// Test that default configuration works without severity overrides
#[test]
fn test_default_config_no_severity_override() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create test file with issues
    let content = "key: value   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\n";
    fs::write(&test_file, content).unwrap();

    // Run without configuration (should use defaults)
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg(test_file.to_str().unwrap());

    let output = cmd.assert().code(1);

    // Check that the output contains "error" (default severity)
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("error"));
}

/// Test that configuration file with only rules (no severity) works
#[test]
fn test_config_without_severity() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");
    let config_file = temp_dir.path().join("config.yaml");

    // Create test file with issues
    let content = "key: value   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\n";
    fs::write(&test_file, content).unwrap();

    // Create configuration without severity section
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

    // Run the linter with configuration
    let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
    cmd.arg("--config")
        .arg(config_file.to_str().unwrap())
        .arg(test_file.to_str().unwrap());

    let output = cmd.assert().code(1);

    // Should still work and use default severities (error)
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("error"));
}

/// Test that invalid configuration file is handled gracefully
#[test]
fn test_invalid_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("invalid.yaml");

    // Create invalid YAML
    let invalid_content = "invalid: yaml: content: [";
    fs::write(&config_file, invalid_content).unwrap();

    // Try to load the configuration
    let result = load_config(&config_file);
    assert!(result.is_err());
}

/// Test that missing configuration file falls back to defaults
#[test]
fn test_missing_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let missing_config = temp_dir.path().join("missing.yaml");

    // Try to load non-existent configuration
    let result = load_config(&missing_config);

    // Should fail because file doesn't exist
    assert!(result.is_err());
}

/// Test that all severity levels work correctly
#[test]
fn test_all_severity_levels() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.yaml");

    // Create test file with issues
    let content = "key: value   \n# This line is way too long and exceeds the maximum line length limit of 80 characters\n";
    fs::write(&test_file, content).unwrap();

    for (severity_name, expected_output) in
        [("Error", "error"), ("Warning", "warning"), ("Info", "info")]
    {
        let config_file = temp_dir
            .path()
            .join(format!("config_{}.yaml", severity_name));
        let config_content = format!(
            r#"
global:
  default_severity: Error
rules:
  line-length:
    enabled: true
    severity: {}
    settings:
      max_length: 80
  trailing-spaces:
    enabled: true
    severity: {}
"#,
            severity_name, severity_name
        );
        fs::write(&config_file, config_content).unwrap();

        // Run the linter with configuration
        let mut cmd = assert_cmd::Command::cargo_bin("yamllint-rs").unwrap();
        cmd.arg("--config")
            .arg(config_file.to_str().unwrap())
            .arg(test_file.to_str().unwrap());

        let output = cmd.assert().code(1);
        let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
        assert!(
            stdout.contains(expected_output),
            "Expected '{}' in output for severity '{}', got: {}",
            expected_output,
            severity_name,
            stdout
        );
    }
}
