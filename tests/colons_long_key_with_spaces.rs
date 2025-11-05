#[cfg(test)]
mod tests {
    use yamllint_rs::rules::colons::ColonsRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_colons_long_key_with_spaces_and_parentheses() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 1610:12
        // Key: "Long Key Name with Spaces and Parentheses (Includes Subitems)"
        // This key contains spaces and parentheses, which is valid YAML
        let content = r#"        Long Key Name with Spaces and Parentheses (Includes Subitems): not_supported
"#;

        let rule = ColonsRule::new();
        let issues = rule.check(content, "test.yaml");

        let colons_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("colon"))
            .collect();

        assert_eq!(
            colons_issues.len(),
            0,
            "Long key with spaces and parentheses should not be flagged. Found issues: {:?}",
            colons_issues
        );
    }

    #[test]
    fn test_colons_empty_string_in_list() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 5157:18
        // Line: "              - ''"
        // This is an empty string in a list - there's no colon on this line!
        let content = r#"              - ''
"#;

        let rule = ColonsRule::new();
        let issues = rule.check(content, "test.yaml");

        let colons_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("colon"))
            .collect();

        assert_eq!(
            colons_issues.len(),
            0,
            "Empty string in list should not be flagged (no colon present). Found issues: {:?}",
            colons_issues
        );
    }
}
