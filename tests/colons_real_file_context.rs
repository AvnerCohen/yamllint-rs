#[cfg(test)]
mod tests {
    use yamllint_rs::rules::colons::ColonsRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_colons_line_1610_with_context() {
        // Test case from real config file with actual context
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 1610:12
        let content = r#"      fields:
        '1': not_supported
        Long Key Name with Spaces and Parentheses (Includes Subitems): not_supported
        Country: not_supported
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
            "Line 1610 with context should not be flagged. Found issues: {:?}",
            colons_issues
        );
    }

    #[test]
    fn test_colons_line_5157_with_context() {
        // Test case from real config file with actual context
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 5157:18
        // Line 5157 has no colon - it's just "- ''"
        let content = r#"          - eq:
              '1':
              - ''
          - has_intersection:
              Field:
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
            "Line 5157 with context should not be flagged (no colon present). Found issues: {:?}",
            colons_issues
        );
    }
}
