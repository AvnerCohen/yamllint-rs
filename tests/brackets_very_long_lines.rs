#[cfg(test)]
mod tests {
    use yamllint_rs::rules::brackets::BracketsRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_brackets_very_long_line_with_many_items() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports issues at very high column numbers
        // This is a very long line with many quoted values in a flow sequence
        // The line has ~8526 characters, and errors are reported at columns 2001, 8526, etc.
        // This suggests column calculation is wrong for very long lines
        let content = r#"organization: ['ORG-001','ORG-002','ORG-003','ORG-004','ORG-005','ORG-006','ORG-007','ORG-008','ORG-009','ORG-010','ORG-011','ORG-012','ORG-013','ORG-014','ORG-015','ORG-016','ORG-017','ORG-018','ORG-019','ORG-020','ORG-021','ORG-022','ORG-023','ORG-024','ORG-025','ORG-026','ORG-027','ORG-028','ORG-029','ORG-030','ORG-031','ORG-032','ORG-033','ORG-034','ORG-035','ORG-036','ORG-037','ORG-038','ORG-039','ORG-040','ORG-041','ORG-042','ORG-043','ORG-044','ORG-045','ORG-046','ORG-047','ORG-048','ORG-049','ORG-050','ORG-051','ORG-052','ORG-053','ORG-054','ORG-055','ORG-056','ORG-057','ORG-058','ORG-059','ORG-060','ORG-061','ORG-062','ORG-063','ORG-064','ORG-065','ORG-066','ORG-067','ORG-068','ORG-069','ORG-070','ORG-071','ORG-072','ORG-073','ORG-074','ORG-075',]
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
            "Very long line with many items should not be flagged. Found issues: {:?}",
            brackets_issues
        );
    }
}
