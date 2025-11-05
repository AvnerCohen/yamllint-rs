#[cfg(test)]
mod tests {
    use yamllint_rs::rules::braces::BracesRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_braces_real_file_line_8() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue at line 9:53
        let content = r#"ctd_filter_fields:
- {field_name: req_job_department, label: Department}
departments: []
"#;

        let rule = BracesRule::new();
        let issues = rule.check(content, "test.yaml");

        let braces_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("braces"))
            .collect();

        assert_eq!(
            braces_issues.len(),
            0,
            "Flow mapping on line 8 should not be flagged. Found issues: {:?}",
            braces_issues
        );
    }
}
