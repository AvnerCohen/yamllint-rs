use yamllint_rs::{FileProcessor, ProcessingOptions};

#[test]
fn test_complex_list_structure() {
    let yaml_content = r#"- name: first_item
  value: 1
  description: First item in the list
- name: second_item
  value: 2
  description: Second item in the list"#;

    let options = ProcessingOptions::default();
    let processor = FileProcessor::with_default_rules(options);

    // Create a temporary file for testing
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", yaml_content).unwrap();

    let result = processor.process_file(file.path()).unwrap();

    let duplicate_key_errors: Vec<_> = result
        .issues
        .iter()
        .filter(|(issue, _)| issue.message.contains("duplication of key"))
        .collect();

    assert_eq!(duplicate_key_errors.len(), 0);

    let _indentation_errors: Vec<_> = result
        .issues
        .iter()
        .filter(|(issue, _)| issue.message.contains("wrong indentation"))
        .collect();
}

#[test]
fn test_document_start_warning() {
    let yaml_content = r#"key: value
another_key: another_value"#;

    let options = ProcessingOptions::default();
    let processor = FileProcessor::with_default_rules(options);

    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", yaml_content).unwrap();

    let result = processor.process_file(file.path()).unwrap();

    let document_start_warnings: Vec<_> = result
        .issues
        .iter()
        .filter(|(issue, _)| issue.message.contains("missing document start"))
        .collect();

    assert!(document_start_warnings.len() > 0);
}

#[test]
fn test_line_length_errors() {
    let yaml_content = r#"key: this_is_a_very_long_line_that_exceeds_eighty_characters_and_should_trigger_a_line_length_error
another_key: value"#;

    let options = ProcessingOptions::default();
    let processor = FileProcessor::with_default_rules(options);

    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", yaml_content).unwrap();

    let result = processor.process_file(file.path()).unwrap();

    let line_length_errors: Vec<_> = result
        .issues
        .iter()
        .filter(|(issue, _)| issue.message.contains("line too long"))
        .collect();

    assert!(line_length_errors.len() > 0);
}
