#[cfg(test)]
mod tests {
    use yamllint_rs::rules::brackets::BracketsRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_brackets_empty_brackets_line_71() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue
        // This is an empty bracket case - FlowSequenceStart immediately followed by FlowSequenceEnd
        let content = r#"      data_filters: []
"#;

        let rule = BracketsRule::new();
        let issues = rule.check(content, "test.yaml");

        let brackets_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("brackets"))
            .collect();

        assert_eq!(
            brackets_issues.len(),
            0,
            "Empty brackets [] should not be flagged. Found issues: {:?}",
            brackets_issues
        );
    }

    #[test]
    fn test_brackets_inline_flow_sequence_line_73() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue
        // This is an inline flow sequence with elements on the same line
        let content = r#"      exclude_phases: ["phase1", "phase2"]
"#;

        let rule = BracketsRule::new();
        let issues = rule.check(content, "test.yaml");

        let brackets_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("brackets"))
            .collect();

        assert_eq!(
            brackets_issues.len(),
            0,
            "Inline brackets with elements should not be flagged. Found issues: {:?}",
            brackets_issues
        );
    }

    #[test]
    fn test_brackets_inline_flow_sequence_line_300() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue
        // This is an inline flow sequence with a single element
        let content = r#"                value: [0]
"#;

        let rule = BracketsRule::new();
        let issues = rule.check(content, "test.yaml");

        let brackets_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("brackets"))
            .collect();

        assert_eq!(
            brackets_issues.len(),
            0,
            "Inline brackets with single element should not be flagged. Found issues: {:?}",
            brackets_issues
        );
    }

    #[test]
    fn test_brackets_inline_flow_sequence_line_301() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue
        // This is an inline flow sequence with multiple elements
        let content = r#"                key: [0, "item_id"]
"#;

        let rule = BracketsRule::new();
        let issues = rule.check(content, "test.yaml");

        let brackets_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("brackets"))
            .collect();

        assert_eq!(
            brackets_issues.len(),
            0,
            "Inline brackets with multiple elements should not be flagged. Found issues: {:?}",
            brackets_issues
        );
    }
}
