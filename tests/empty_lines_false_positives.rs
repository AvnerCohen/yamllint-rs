#[cfg(test)]
mod tests {
    use yamllint_rs::rules::empty_lines::EmptyLinesRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_empty_lines_email_template() {
        // Test case from real email template file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue
        // Multiple blank lines in email template
        // Default max is 2, so yamllint-rs flags it, but yamllint doesn't
        let content = r#"    Please share your comments on this.

    Thank you very much!

    Regards,
"#;

        let rule = EmptyLinesRule::new();
        let issues = rule.check(content, "test.yaml");

        let empty_lines_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("blank lines"))
            .collect();

        assert_eq!(
            empty_lines_issues.len(),
            0,
            "Blank lines in email template should not be flagged. Found issues: {:?}",
            empty_lines_issues
        );
    }
}
