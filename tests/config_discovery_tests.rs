use std::fs;
use tempfile::TempDir;
use yamllint_rs::{discover_config_file_from_dir, load_config};

#[test]
fn test_discover_config_file_not_found() {
    // Create a temporary directory with no .yamllint file
    let temp_dir = TempDir::new().unwrap();

    // Should not find any config file
    let result = discover_config_file_from_dir(temp_dir.path().to_path_buf());
    assert!(
        result.is_none(),
        "Should not find config file in empty directory"
    );
}

#[test]
fn test_discover_config_file_in_current_directory() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();

    // Create a .yamllint file with our config format
    let config_content = r#"
rules:
  document-start:
    enabled: false
  truthy:
    enabled: false
  line-length:
    enabled: true
    settings:
      max_length: 2000
global:
  default_severity: Error
"#;
    fs::write(temp_dir.path().join(".yamllint"), config_content).unwrap();

    // Should find the config file
    let result = discover_config_file_from_dir(temp_dir.path().to_path_buf());
    assert!(
        result.is_some(),
        "Should find config file in current directory"
    );

    let config_path = result.unwrap();
    assert_eq!(config_path.file_name().unwrap(), ".yamllint");
    assert!(config_path.exists());
}

#[test]
fn test_discover_config_file_in_parent_directory() {
    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let parent_dir = temp_dir.path();
    let child_dir = parent_dir.join("child");
    fs::create_dir(&child_dir).unwrap();

    // Create .yamllint in parent directory
    let config_content = r#"
rules:
  document-start:
    enabled: false
global:
  default_severity: Error
"#;
    fs::write(parent_dir.join(".yamllint"), config_content).unwrap();

    // Should find the config file in parent directory
    let result = discover_config_file_from_dir(child_dir.to_path_buf());
    assert!(
        result.is_some(),
        "Should find config file in parent directory"
    );

    let config_path = result.unwrap();
    assert_eq!(config_path.file_name().unwrap(), ".yamllint");
    // Use canonicalize to handle path differences
    let canonical_parent = parent_dir.canonicalize().unwrap();
    let canonical_config_parent = config_path.parent().unwrap().canonicalize().unwrap();
    assert_eq!(canonical_config_parent, canonical_parent);
}

#[test]
fn test_discover_config_file_prefers_closer_directory() {
    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let parent_dir = temp_dir.path();
    let child_dir = parent_dir.join("child");
    fs::create_dir(&child_dir).unwrap();

    // Create .yamllint in both parent and child directories
    let parent_config = r#"
rules:
  document-start:
    enabled: false
global:
  default_severity: Error
"#;
    let child_config = r#"
rules:
  truthy:
    enabled: false
global:
  default_severity: Error
"#;
    fs::write(parent_dir.join(".yamllint"), parent_config).unwrap();
    fs::write(child_dir.join(".yamllint"), child_config).unwrap();

    // Should find the config file in child directory (closer)
    let result = discover_config_file_from_dir(child_dir.to_path_buf());
    assert!(
        result.is_some(),
        "Should find config file in child directory"
    );

    let config_path = result.unwrap();
    let canonical_child = child_dir.canonicalize().unwrap();
    let canonical_config_parent = config_path.parent().unwrap().canonicalize().unwrap();
    assert_eq!(canonical_config_parent, canonical_child);
}

#[test]
fn test_load_config_file() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();

    // Create a .yamllint file with our config format
    let config_content = r#"
rules:
  document-start:
    enabled: false
  truthy:
    enabled: false
  line-length:
    enabled: true
    settings:
      max_length: 2000
global:
  default_severity: Error
"#;
    let config_path = temp_dir.path().join(".yamllint");
    fs::write(&config_path, config_content).unwrap();

    // Load the config file
    let config = load_config(&config_path).unwrap();

    // Verify the configuration was loaded correctly
    assert!(
        !config.is_rule_enabled("document-start"),
        "document-start should be disabled"
    );
    assert!(
        !config.is_rule_enabled("truthy"),
        "truthy should be disabled"
    );

    // Check line-length rule configuration
    let line_length_config = config.get_rule_config("line-length");
    assert!(
        line_length_config.is_some(),
        "line-length rule should exist"
    );
}

#[test]
fn test_discover_config_file_stops_at_root() {
    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let parent_dir = temp_dir.path();
    let child_dir = parent_dir.join("child");
    let grandchild_dir = child_dir.join("grandchild");
    fs::create_dir_all(&grandchild_dir).unwrap();

    // Should not find any config file (none exists)
    let result = discover_config_file_from_dir(grandchild_dir.to_path_buf());
    assert!(
        result.is_none(),
        "Should not find config file when none exists"
    );
}

#[test]
fn test_discover_config_file_with_invalid_yaml() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();

    // Create an invalid .yamllint file
    let invalid_config = r#"
rules:
  document-start:
    enabled: false
  truthy:
    enabled: false
  line-length:
    enabled: true
    settings:
      max_length: 2000
global:
  default_severity: Error
invalid: yaml: content: [unclosed
"#;
    let config_path = temp_dir.path().join(".yamllint");
    fs::write(&config_path, invalid_config).unwrap();

    // Should find the config file but loading should fail
    let result = discover_config_file_from_dir(temp_dir.path().to_path_buf());
    assert!(result.is_some(), "Should find config file even if invalid");

    let config_path = result.unwrap();
    let load_result = load_config(&config_path);
    assert!(
        load_result.is_err(),
        "Should fail to load invalid YAML config"
    );
}
