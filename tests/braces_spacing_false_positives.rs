#[cfg(test)]
mod tests {
    use yamllint_rs::rules::braces::BracesRule;
    use yamllint_rs::rules::Rule;

    #[test]
    fn test_braces_flow_mapping_no_spaces() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports 1 issue
        // Flow mapping: {id: new, label: New, localized_label_translations_key: new}
        // Default config: min-spaces-inside: 0, max-spaces-inside: 0
        // This means 0 spaces after { and 0 spaces before }
        let content = r#"candidate_phases:
- {id: new, label: New, localized_label_translations_key: new}
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
            "Flow mapping with no spaces should not be flagged. Found issues: {:?}",
            braces_issues
        );
    }

    #[test]
    fn test_braces_flow_mapping_multiple_items() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports multiple issues
        // Multiple flow mappings with no spaces inside
        let content = r#"candidate_phases:
- {id: new, label: New, localized_label_translations_key: new}
- {id: Screening, label: Screening, localized_label_translations_key: screening}
- {id: hm_review, label: HM Review, localized_label_translations_key: hm_review}
- {id: interview, label: Interview, localized_label_translations_key: interview}
- {id: offer, label: Offer, localized_label_translations_key: offer}
- {id: post-Offer, label: Post-Offer, localized_label_translations_key: post_offer}
- {id: hire, label: Hire, localized_label_translations_key: hire}
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
            "Flow mappings with no spaces inside should not be flagged. Found issues: {:?}",
            braces_issues
        );
    }

    #[test]
    fn test_braces_flow_mapping_simple() {
        // Test case from real config file
        // yamllint reports 0 issues, but yamllint-rs reports issues
        let content = r#"ctd_filter_fields:
- {field_name: req_job_department, label: Department}
- {field_name: req_business_unit_group, label: Business Unit}
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
            "Simple flow mappings should not be flagged. Found issues: {:?}",
            braces_issues
        );
    }
}
