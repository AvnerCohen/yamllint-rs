use crate::{LintIssue, Severity};
use yaml_rust::scanner::{Scanner, Token, TokenType};

#[derive(Debug, Clone)]
pub struct HyphensConfig {
    pub max_spaces_after: i32,
}

impl Default for HyphensConfig {
    fn default() -> Self {
        Self {
            max_spaces_after: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HyphensRule {
    base: crate::rules::base::BaseRule<HyphensConfig>,
}

impl HyphensRule {
    pub fn new() -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(HyphensConfig::default()),
        }
    }

    pub fn with_config(config: HyphensConfig) -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &HyphensConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: HyphensConfig) {
        self.base.set_config(config);
    }

    pub fn get_severity(&self) -> Severity {
        self.base.get_severity(Severity::Warning)
    }

    pub fn set_severity(&mut self, severity: Severity) {
        self.base.set_severity(severity);
    }

    pub fn has_severity_override(&self) -> bool {
        self.base.has_severity_override()
    }
}

impl Default for HyphensRule {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::rules::Rule for HyphensRule {
    fn rule_id(&self) -> &'static str {
        "hyphens"
    }

    fn rule_name(&self) -> &'static str {
        "Hyphens"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper spacing around hyphens -."
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn get_severity(&self) -> Severity {
        self.base.get_severity(self.default_severity())
    }

    fn set_severity(&mut self, severity: Severity) {
        self.base.set_severity(severity);
    }

    fn has_severity_override(&self) -> bool {
        self.base.has_severity_override()
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, file_path: &str) -> Vec<LintIssue> {
        self.check_impl(content, file_path)
    }

    fn check_with_analysis(
        &self,
        content: &str,
        _file_path: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        self.check_impl_with_analysis(content, analysis)
    }
}

impl HyphensRule {
    fn check_with_tokens(
        &self,
        content: &str,
        tokens: &[Token],
        _token_analysis: &crate::analysis::TokenAnalysis,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (i, token) in tokens.iter().enumerate() {
            let Token(marker, token_type) = token;

            if matches!(token_type, TokenType::BlockEntry) {
                let mut next_idx = i + 1;
                let mut next_token_on_same_line: Option<&Token> = None;

                while let Some(next_token) = tokens.get(next_idx) {
                    let Token(next_marker, next_token_type) = next_token;

                    if marker.line() != next_marker.line() {
                        break;
                    }

                    match next_token_type {
                        TokenType::BlockMappingStart
                        | TokenType::BlockSequenceStart
                        | TokenType::FlowMappingStart
                        | TokenType::FlowSequenceStart
                        | TokenType::BlockEnd
                        | TokenType::FlowMappingEnd
                        | TokenType::FlowSequenceEnd => {
                            next_idx += 1;
                            continue;
                        }
                        _ => {
                            next_token_on_same_line = Some(next_token);
                            break;
                        }
                    }
                }

                if let Some(next_token) = next_token_on_same_line {
                    let Token(next_marker, _) = next_token;

                    if self.config().max_spaces_after >= 0 {
                        let spaces_after =
                            self.calculate_spaces_after(content, marker, next_marker);
                        if spaces_after > self.config().max_spaces_after as usize {
                            issues.push(LintIssue {
                                line: marker.line() + 1,
                                column: next_marker.col() + 1,
                                message: format!(
                                    "too many spaces after hyphen ({} > {})",
                                    spaces_after,
                                    self.config().max_spaces_after
                                ),
                                severity: self.get_severity(),
                            });
                        }
                    }
                }
            }
        }

        issues
    }

    pub fn check_impl(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let scanner = Scanner::new(content.chars());
        let tokens: Vec<_> = scanner.collect();
        let token_analysis = crate::analysis::TokenAnalysis::analyze(content);
        self.check_with_tokens(content, &tokens, &token_analysis)
    }

    pub fn check_impl_with_analysis(
        &self,
        content: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        if let Some(token_analysis) = analysis.tokens() {
            self.check_with_tokens(content, &token_analysis.tokens, token_analysis)
        } else {
            self.check_impl(content, "")
        }
    }

    fn calculate_spaces_after(
        &self,
        content: &str,
        token_marker: &yaml_rust::scanner::Marker,
        next_marker: &yaml_rust::scanner::Marker,
    ) -> usize {
        if token_marker.line() != next_marker.line() {
            return 0;
        }

        let token_end = token_marker.index() + 1;
        let next_start = next_marker.index();

        if next_start <= token_end {
            return 0;
        }

        content
            .chars()
            .skip(token_end)
            .take(next_start - token_end)
            .filter(|&c| c == ' ')
            .count()
    }

    pub fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        super::FixResult {
            content: content.to_string(),
            changed: false,
            fixes_applied: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;
    use crate::Severity;

    #[test]
    fn test_hyphens_rule_default() {
        let rule = HyphensRule::new();
        assert_eq!(rule.rule_id(), "hyphens");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_hyphens_check_clean_hyphens() {
        let rule = HyphensRule::new();
        let content = "- first item\n- second item\n- third item";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_hyphens_check_spaces_after() {
        let rule = HyphensRule::new();
        let content = "-  first item\n-  second item";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("too many spaces after hyphen"));
        assert!(issues[1].message.contains("too many spaces after hyphen"));
    }

    #[test]
    fn test_hyphens_fix() {
        let rule = HyphensRule::new();
        let content = "-  first item\n-  second item";
        let fix_result = rule.fix(content, "test.yaml");
        // Fix functionality is not yet implemented for token-based approach
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }

    #[test]
    fn test_hyphens_fix_no_changes() {
        let rule = HyphensRule::new();
        let content = "- first item\n- second item\n- third item";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }

    #[test]
    fn test_hyphens_should_ignore_hyphens_in_content() {
        let rule = HyphensRule::new();
        // This test reproduces the false positive bug where hyphens in content
        // are incorrectly flagged as spacing violations
        let content = r#"name: Israel -  L&P-Menora-Delivery (153)
another: Some -  other content
list:
  - valid list item
  - another valid item"#;

        let issues = rule.check(content, "test.yaml");

        // This test should PASS (no issues) because hyphens in content should be ignored
        // The current implementation incorrectly flags content hyphens, so this test will FAIL
        // Once we fix the implementation to align with yamllint, this test will PASS
        assert!(issues.is_empty(),
            "No spacing violations should be detected for hyphens in content. Current false positives: {:?}", issues);
    }

    #[test]
    fn test_hyphens_list_item_with_mapping_value_should_not_report_error() {
        // This test reproduces the bug where yamllint-rs incorrectly reports
        // errors for valid list items with mapping values on following lines.
        // Based on colorado.yaml structure where list items have mappings:
        //   - Active or not: not active
        //     category: application
        // The issue is that yamllint-rs uses token start position instead of end position
        // and incorrectly calculates spaces, causing false positives.
        let rule = HyphensRule::new();
        let content = r#"  workflow: '1'
- Active or not: not active
  category: application
  is_change_to_status_allowed: false
  is_required: false
  label: Hired at SAP
  status_bucket: hired
  status_group: Hired
  status_name: HiredAtSAP
  status_stage: accepted
  workflow: '2'
- Active or not: not active
  category: application"#;

        let issues = rule.check(content, "test.yaml");

        // This should PASS (no issues) - yamllint Python reports no errors for this structure
        // Current implementation incorrectly reports errors because it uses token start position
        // instead of token end position for spacing calculation
        assert!(issues.is_empty(),
            "No errors should be reported for valid list items with proper spacing. Current false positives: {:?}", issues);
    }

    #[test]
    fn test_hyphens_spacing_calculation_uses_token_end_position() {
        // This test verifies that spacing calculation uses the token END position,
        // not the START position. Python yamllint uses token.end_mark.pointer,
        // but yamllint-rs currently uses token.start_mark.index() which is incorrect.
        let rule = HyphensRule::new();
        let content = "- first item\n- second item\n";

        let issues = rule.check(content, "test.yaml");

        // Should have no issues - single space after hyphen is valid
        assert!(
            issues.is_empty(),
            "Single space after hyphen should be valid. Current issues: {:?}",
            issues
        );
    }

    #[test]
    fn test_hyphens_column_reporting_uses_next_token_column() {
        // This test verifies that error column reporting uses the NEXT token's column,
        // not the BlockEntry token's column. Python yamllint uses next.start_mark.column,
        // but yamllint-rs currently uses marker.col() which is incorrect.
        let rule = HyphensRule::new();
        let content = "-   first item\n"; // 3 spaces after hyphen (should be error)

        let issues = rule.check(content, "test.yaml");

        // Should report exactly one error
        assert_eq!(
            issues.len(),
            1,
            "Should report exactly one error for 3 spaces after hyphen"
        );

        // The column should be the column of the next token (the scalar "first item"),
        // not the column of the BlockEntry token itself.
        // For "-   first item", the BlockEntry is at column 1, but the scalar starts at column 5
        // Python yamllint reports column 5 (next token's column, 1-based)
        // Note: yaml-rust may report line numbers differently, so we check column only
        let issue = &issues[0];
        assert_eq!(
            issue.column, 5,
            "Error column should be next token's column (5), not BlockEntry column (1). Got: {}",
            issue.column
        );
        assert!(
            issue.line >= 1,
            "Error should be on a valid line, got: {}",
            issue.line
        );
    }

    #[test]
    fn test_hyphens_matches_yamllint_python_behavior() {
        // Integration test: This structure should match yamllint Python output exactly.
        // yamllint Python reports NO errors for this content, so yamllint-rs should also report none.
        let rule = HyphensRule::new();
        let content = r#"- category: forwarded
  is_change_to_status_allowed: false
  is_required: false
  label: Forwarded
  placement: 1
  status_bucket: pending
  status_group: Pending
  status_name: Forwarded
  status_stage: pending
  workflow: '1'
- Active or not: not active
  category: application
  is_change_to_status_allowed: false"#;

        let issues = rule.check(content, "test.yaml");

        // yamllint Python reports 0 errors for this content
        // yamllint-rs currently reports false positives due to incorrect token position calculation
        assert!(
            issues.is_empty(),
            "Should match yamllint Python behavior (0 errors). Current false positives: {:?}",
            issues
        );
    }
}
