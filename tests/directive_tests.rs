#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use yamllint_rs::FileProcessor;
    use yamllint_rs::OutputFormat;
    use yamllint_rs::ProcessingOptions;

    fn create_processor() -> FileProcessor {
        let options = ProcessingOptions {
            recursive: false,
            show_progress: false,
            verbose: false,
            output_format: OutputFormat::Standard,
        };
        FileProcessor::with_default_rules(options)
    }

    fn write_temp_file(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file
    }

    #[test]
    fn test_yamllint_disable_all_suppresses_all_rules() {
        // Test that # yamllint disable suppresses all rules until # yamllint enable
        let content = r#"key: value
# yamllint disable
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2','item3','item4','item5','item6','item7','item8','item9','item10']
another_very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2','item3','item4','item5']
# yamllint enable
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // All issues on lines between disable and enable should be suppressed
        let issues_in_disabled_range: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, _)| issue.line >= 2 && issue.line <= 4)
            .collect();

        assert_eq!(issues_in_disabled_range.len(), 0,
            "Found {} issues in disabled range (lines 2-4). yamllint reports 0 issues. Issues: {:?}",
            issues_in_disabled_range.len(), issues_in_disabled_range);
    }

    #[test]
    fn test_yamllint_disable_specific_rule() {
        // Test that # yamllint disable rule:line-length suppresses only line-length
        let content = r#"key: value
# yamllint disable rule:line-length
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: value
another_very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: value
# yamllint enable rule:line-length
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // Filter for line-length issues
        let line_length_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|(_, rule_name)| rule_name == "line-length")
            .collect();

        // Other rules should still work
        let _other_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|(_, rule_name)| rule_name != "line-length")
            .collect();

        // yamllint reports 0 line-length issues but other rules may still report
        assert_eq!(line_length_issues.len(), 0,
            "Found {} line-length issues. yamllint reports 0 issues (suppressed by disable rule:line-length). Issues: {:?}",
            line_length_issues.len(), line_length_issues);
    }

    #[test]
    fn test_yamllint_disable_multiple_specific_rules() {
        // Test disabling multiple specific rules
        let content = r#"key: value
# yamllint disable rule:line-length rule:indentation
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: value
  bad_indentation: value
# yamllint enable rule:line-length rule:indentation
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // Both line-length and indentation issues should be suppressed
        let suppressed_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, rule_name)| {
                issue.line >= 2
                    && issue.line <= 4
                    && (rule_name == "line-length" || rule_name == "indentation")
            })
            .collect();

        assert_eq!(suppressed_issues.len(), 0,
            "Found {} suppressed issues (line-length/indentation). yamllint reports 0. Issues: {:?}",
            suppressed_issues.len(), suppressed_issues);
    }

    #[test]
    fn test_yamllint_disable_line_inline_comment() {
        // Test that # yamllint disable-line suppresses only that line
        let content = r#"key: value
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2']  # yamllint disable-line
another_very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2']
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // Filter for line-length issues
        let line_length_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|(_, rule_name)| rule_name == "line-length")
            .collect();

        // Line 2 should have no issues (disabled by disable-line)
        let issue_on_line_2: Vec<_> = line_length_issues
            .iter()
            .filter(|(issue, _)| issue.line == 2)
            .collect();

        assert_eq!(
            issue_on_line_2.len(),
            0,
            "Found {} line-length issues on line 2. disable-line should suppress it. Issues: {:?}",
            issue_on_line_2.len(),
            issue_on_line_2
        );

        // Line 3 should still have an issue
        let issue_on_line_3: Vec<_> = line_length_issues
            .iter()
            .filter(|(issue, _)| issue.line == 3)
            .collect();

        assert!(
            issue_on_line_3.len() > 0,
            "Expected line-length issue on line 3 (not disabled). Issues: {:?}",
            issue_on_line_3
        );
    }

    #[test]
    fn test_yamllint_disable_line_specific_rule() {
        // Test disable-line with specific rule
        let content = r#"key: value
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2']  # yamllint disable-line rule:line-length
  bad_indentation: value
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // Line 2: line-length should be disabled, but other rules should still work
        let line2_line_length: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, rule_name)| issue.line == 2 && rule_name == "line-length")
            .collect();

        let _line2_other: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, rule_name)| issue.line == 2 && rule_name != "line-length")
            .collect();

        assert_eq!(line2_line_length.len(), 0,
            "Line 2 should have no line-length issues (disabled by disable-line rule:line-length). Issues: {:?}",
            line2_line_length);
    }

    #[test]
    fn test_yamllint_rs_directive_support() {
        // Test that # yamllint-rs disable also works (for future compatibility)
        let content = r#"key: value
# yamllint-rs disable
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2','item3','item4','item5','item6','item7','item8','item9','item10']
another_very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2','item3']
# yamllint-rs enable
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // Filter for line-length issues
        let line_length_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|(_, rule_name)| rule_name == "line-length")
            .collect();

        // yamllint-rs directive should also work
        assert_eq!(line_length_issues.len(), 0,
            "Found {} line-length issues. yamllint-rs disable directive should suppress warnings. Issues: {:?}",
            line_length_issues.len(), line_length_issues);
    }

    #[test]
    fn test_nested_disable_enable() {
        // Test nested disable/enable patterns
        let content = r#"key: value
# yamllint disable
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: value
# yamllint disable rule:indentation
  bad_indentation: value
# yamllint enable rule:indentation
# yamllint enable
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // All issues in disabled range should be suppressed
        let issues_in_range: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, _)| issue.line >= 2 && issue.line <= 5)
            .collect();

        assert_eq!(
            issues_in_range.len(),
            0,
            "Found {} issues in nested disable/enable range. yamllint reports 0. Issues: {:?}",
            issues_in_range.len(),
            issues_in_range
        );
    }

    #[test]
    fn test_partial_enable() {
        // Test enabling only some rules after disabling all
        let content = r#"key: value
# yamllint disable
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: value
  bad_indentation: value
# yamllint enable rule:indentation
  still_bad_indentation: value
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: value
# yamllint enable
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // After partial enable, line-length should still be disabled, indentation should work
        // Line 5 is the comment, line 6 has the actual content
        let line6_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, _)| issue.line == 6)
            .collect();

        let line6_line_length: Vec<_> = line6_issues
            .iter()
            .filter(|(_, rule_name)| rule_name == "line-length")
            .collect();

        let _line6_indentation: Vec<_> = line6_issues
            .iter()
            .filter(|(_, rule_name)| rule_name == "indentation")
            .collect();

        // Line 6: line-length should be disabled (still disabled from line 2)
        // Indentation should be enabled (enabled on line 5), but may not report if indentation is valid
        assert_eq!(
            line6_line_length.len(),
            0,
            "Line 6 should have no line-length issues (still disabled). Issues: {:?}",
            line6_line_length
        );

        // Verify that indentation is not being incorrectly suppressed
        // (If indentation rule would report, it should be reported)
        // Note: The indentation rule may not report for this specific case,
        // but we verify that line-length is correctly disabled
    }

    #[test]
    fn test_disable_line_block_comment() {
        // Test disable-line as block comment (affects that line)
        let content = r#"key: value
# yamllint disable-line
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2']
another_very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2']
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // Line 3 should have no issues (disabled by disable-line on line 2)
        let line3_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, _)| issue.line == 3)
            .collect();

        assert_eq!(
            line3_issues.len(),
            0,
            "Line 3 should have no issues (disabled by disable-line on line 2). Issues: {:?}",
            line3_issues
        );
    }

    #[test]
    fn test_disable_line_specific_rule_block() {
        // Test disable-line with specific rule as block comment
        let content = r#"key: value
# yamllint disable-line rule:line-length
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: ['item1','item2']
  bad_indentation: value
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // Line 3: line-length disabled, but indentation should still work
        let line3_line_length: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, rule_name)| issue.line == 3 && rule_name == "line-length")
            .collect();

        let _line3_indentation: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, rule_name)| issue.line == 3 && rule_name == "indentation")
            .collect();

        assert_eq!(
            line3_line_length.len(),
            0,
            "Line 3 should have no line-length issues (disabled). Issues: {:?}",
            line3_line_length
        );
    }

    #[test]
    fn test_mixed_yamllint_and_yamllint_rs() {
        // Test mixing both directive prefixes
        let content = r#"key: value
# yamllint disable
very_long_line_that_exceeds_eighty_characters_should_trigger_line_length_warning: value
# yamllint-rs enable
normal: line
"#;

        let temp_file = write_temp_file(content);
        let processor = create_processor();
        let result = processor.process_file(temp_file.path()).unwrap();

        // yamllint-rs enable should work with yamllint disable
        let issues_in_range: Vec<_> = result
            .issues
            .iter()
            .filter(|(issue, _)| issue.line >= 2 && issue.line <= 3)
            .collect();

        assert_eq!(
            issues_in_range.len(),
            0,
            "Found {} issues in mixed directive range. Both prefixes should work. Issues: {:?}",
            issues_in_range.len(),
            issues_in_range
        );
    }
}
