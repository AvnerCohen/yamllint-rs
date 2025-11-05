#[cfg(test)]
mod tests {
    use yamllint_rs::rules::brackets::BracketsRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_brackets_nested_braces_inside_brackets() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 124:48
        // Flow sequence with nested flow mapping: [{'min': 1, 'max': 5}]
        // When checking spacing before ], the previous token is FlowMappingEnd }
        let content = r#"ranges: [{'min': 1, 'max': 5}]
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
            "Nested braces inside brackets should not be flagged. Found issues: {:?}",
            brackets_issues
        );
    }

    #[test]
    fn test_brackets_nested_brackets() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 35:52
        // Nested flow sequences: [["field.path.to.value", "alias"]]
        // When checking spacing before outer ], the previous token is FlowSequenceEnd ]
        let content = r#"filter_fields: [["field.path.to.value", "alias"]]
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
            "Nested brackets should not be flagged. Found issues: {:?}",
            brackets_issues
        );
    }

    #[test]
    fn test_brackets_nested_brackets_simple() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 50:39
        // Nested flow sequences: [["option1", 1.0]]
        let content = r#"options_probs: [["option1", 1.0]]
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
            "Nested brackets should not be flagged. Found issues: {:?}",
            brackets_issues
        );
    }

    #[test]
    fn test_brackets_nested_braces_multiple_items() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 79:73
        // Flow sequence with multiple nested flow mappings: [{'min': 1, 'max': 5}, {'min': 50, 'max': 150}]
        // When checking spacing before ], the previous token is FlowMappingEnd }
        let content = r#"ranges: [{'min': 1, 'max': 5}, {'min': 50, 'max': 150}]
"#;

        let rule = BracketsRule::new();
        let issues = rule.check(content, "test.yaml");

        let brackets_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("brackets"))
            .collect();

        assert_eq!(brackets_issues.len(), 0,
            "Nested braces inside brackets with multiple items should not be flagged. Found issues: {:?}", brackets_issues);
    }
}
