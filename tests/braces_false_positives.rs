#[cfg(test)]
mod tests {
    use yamllint_rs::rules::braces::BracesRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_braces_should_not_flag_braces_in_scalar_text() {
        // Braces in scalar text (like {field_name}) should NOT be flagged
        // because PyYAML doesn't tokenize them as FlowMappingStart/FlowMappingEnd
        let content =
            r#"    Example text with placeholders {field_name} ({field_id}) at {field_location}."#;

        let rule = BracesRule::new();
        let issues = rule.check(content, "test.yaml");

        // yamllint reports 0 issues for this line
        // Our implementation should match this behavior
        let braces_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("braces"))
            .collect();

        assert_eq!(
            braces_issues.len(),
            0,
            "Braces in scalar text should not be flagged. Found issues: {:?}",
            braces_issues
        );
    }
}
