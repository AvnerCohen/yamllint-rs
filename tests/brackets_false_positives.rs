#[cfg(test)]
mod tests {
    use yamllint_rs::rules::brackets::BracketsRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_brackets_spacing_calculation_matches_yamllint() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue
        // The issue is that yamllint uses spaces_after/spaces_before (token-based)
        // while our implementation counts total leading + trailing spaces
        let content = r#"      items_to_exclude:
        [
          Item1,
          Item2,
          Item3,
          Item4,
        ]
"#;

        let rule = BracketsRule::new();
        let issues = rule.check(content, "test.yaml");

        // yamllint reports 0 issues for this content
        // Our implementation should match this behavior
        let brackets_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("brackets"))
            .collect();

        assert_eq!(
            brackets_issues.len(),
            0,
            "Brackets spacing calculation should match yamllint. Found issues: {:?}",
            brackets_issues
        );
    }
}
