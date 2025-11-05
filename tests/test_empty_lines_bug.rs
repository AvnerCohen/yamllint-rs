#[cfg(test)]
mod tests {
    use yamllint_rs::rules::empty_lines::EmptyLinesConfig;
    use yamllint_rs::rules::empty_lines::EmptyLinesRule;
    use yamllint_rs::rules::Rule;

    #[test]
    #[ignore]
    fn test_empty_lines_bug_actual_file_reports_wrong_count() {
        let config = EmptyLinesConfig {
            max: 2,
            max_start: 0,
            max_end: 0,
        };
        let rule = EmptyLinesRule::with_config(config);

        let content = "    Best,\n    {user_name}\n\n\nplural_send_applications_to_hm:\n";

        let issues = rule.check(content, "test.yaml");
        let empty_line_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("blank lines"))
            .collect();

        assert_eq!(empty_line_issues.len(), 0);
    }

    #[test]
    fn test_empty_lines_bug_is_last_blank_line_detection() {
        let config = EmptyLinesConfig {
            max: 2,
            max_start: 0,
            max_end: 0,
        };
        let rule = EmptyLinesRule::with_config(config);

        let content = "    Best,\n    {user_name}\n\n\nplural_send_applications_to_hm:\n";

        let issues = rule.check(content, "test.yaml");
        let empty_line_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("blank lines"))
            .collect();

        assert_eq!(empty_line_issues.len(), 0);
    }

    #[test]
    fn test_empty_lines_bug_reproduce_actual_file_case() {
        let config = EmptyLinesConfig {
            max: 2,
            max_start: 0,
            max_end: 0,
        };
        let rule = EmptyLinesRule::with_config(config);

        let mut content = String::new();
        for i in 1..=55 {
            content.push_str(&format!("line_{}: value_{}\n", i, i));
        }
        content.push_str("    Best,\n");
        content.push_str("    {user_name}\n");
        content.push_str("\n");
        content.push_str("\n");
        content.push_str("plural_send_applications_to_hm:\n");

        let issues = rule.check(&content, "test.yaml");
        let empty_line_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("blank lines"))
            .collect();

        assert_eq!(empty_line_issues.len(), 0);
    }

    #[test]
    fn test_empty_lines_bug_three_blank_lines_with_max_two() {
        let config = EmptyLinesConfig {
            max: 2,
            max_start: 0,
            max_end: 0,
        };
        let rule = EmptyLinesRule::with_config(config);

        let content = "text1\ntext2\n\n\n\ntext3\n";

        let issues = rule.check(content, "test.yaml");
        let empty_line_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("blank lines"))
            .collect();

        assert_eq!(empty_line_issues.len(), 1);

        if let Some(issue) = empty_line_issues.first() {
            assert!(issue.message.contains("3 > 2"));
        }
    }
}
