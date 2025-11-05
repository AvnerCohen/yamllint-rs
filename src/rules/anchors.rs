use crate::{LintIssue, Severity};
use std::collections::HashMap;
use yaml_rust::scanner::{Scanner, Token, TokenType};

#[derive(Debug, Clone)]
pub struct AnchorsConfig {
    pub forbid_undeclared_aliases: bool,
    pub forbid_duplicated_anchors: bool,
    pub forbid_unused_anchors: bool,
}

impl Default for AnchorsConfig {
    fn default() -> Self {
        Self {
            forbid_undeclared_aliases: true,
            forbid_duplicated_anchors: false,
            forbid_unused_anchors: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnchorsRule {
    base: crate::rules::base::BaseRule<AnchorsConfig>,
}

impl AnchorsRule {
    pub fn new() -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(AnchorsConfig::default()),
        }
    }

    pub fn with_config(config: AnchorsConfig) -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &AnchorsConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: AnchorsConfig) {
        self.base.set_config(config);
    }

    pub fn get_severity(&self) -> Severity {
        self.base.get_severity(Severity::Error)
    }

    pub fn set_severity(&mut self, severity: Severity) {
        self.base.set_severity(severity);
    }

    pub fn has_severity_override(&self) -> bool {
        self.base.has_severity_override()
    }
}

impl Default for AnchorsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::rules::Rule for AnchorsRule {
    fn rule_id(&self) -> &'static str {
        "anchors"
    }

    fn rule_name(&self) -> &'static str {
        "Anchors"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper YAML anchor and alias usage."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
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
        false
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

impl AnchorsRule {
    fn check_with_tokens(
        &self,
        tokens: &[Token],
        _token_analysis: &crate::analysis::TokenAnalysis,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        let mut anchors: HashMap<String, AnchorInfo> = HashMap::new();

        for token in tokens {
            let Token(marker, token_type) = token;

            if matches!(
                token_type,
                TokenType::StreamStart(_) | TokenType::DocumentStart | TokenType::DocumentEnd
            ) {
                anchors.clear();
                continue;
            }

            if let TokenType::Anchor(anchor_name) = token_type {
                if self.config().forbid_duplicated_anchors && anchors.contains_key(anchor_name) {
                    issues.push(LintIssue {
                        line: marker.line() + 1,
                        column: marker.col() + 1,
                        message: format!("found duplicated anchor \"{}\"", anchor_name),
                        severity: self.get_severity(),
                    });
                }

                anchors.insert(
                    anchor_name.clone(),
                    AnchorInfo {
                        line: marker.line(),
                        column: marker.col(),
                        used: false,
                    },
                );
            }

            if let TokenType::Alias(alias_name) = token_type {
                if self.config().forbid_undeclared_aliases && !anchors.contains_key(alias_name) {
                    issues.push(LintIssue {
                        line: marker.line() + 1,
                        column: marker.col() + 1,
                        message: format!("found undeclared alias \"{}\"", alias_name),
                        severity: self.get_severity(),
                    });
                }

                if let Some(anchor_info) = anchors.get_mut(alias_name) {
                    anchor_info.used = true;
                }
            }
        }

        if self.config().forbid_unused_anchors {
            for (anchor_name, anchor_info) in &anchors {
                if !anchor_info.used {
                    issues.push(LintIssue {
                        line: anchor_info.line + 1,
                        column: anchor_info.column + 1,
                        message: format!("found unused anchor \"{}\"", anchor_name),
                        severity: self.get_severity(),
                    });
                }
            }
        }

        issues
    }

    pub fn check_impl(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let scanner = Scanner::new(content.chars());
        let tokens: Vec<_> = scanner.collect();
        let token_analysis = crate::analysis::TokenAnalysis::analyze(content);
        self.check_with_tokens(&tokens, &token_analysis)
    }

    pub fn check_impl_with_analysis(
        &self,
        content: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        if let Some(token_analysis) = analysis.tokens() {
            self.check_with_tokens(&token_analysis.tokens, token_analysis)
        } else {
            self.check_impl(content, "")
        }
    }
}

#[derive(Debug, Clone)]
struct AnchorInfo {
    line: usize,
    column: usize,
    used: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;

    #[test]
    fn test_anchors_rule_default() {
        let rule = AnchorsRule::new();
        assert_eq!(rule.rule_id(), "anchors");
        assert_eq!(rule.default_severity(), Severity::Error);
        assert!(rule.is_enabled_by_default());
        assert!(!rule.can_fix());
    }

    #[test]
    fn test_anchors_check_clean_anchors() {
        let rule = AnchorsRule::new();
        let content = "- &anchor\n  foo: bar\n- *anchor";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_anchors_check_undeclared_alias() {
        let rule = AnchorsRule::new();
        let content = "- &anchor\n  foo: bar\n- *unknown";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("undeclared alias"));
    }

    #[test]
    fn test_anchors_check_multiline_string_with_asterisk() {
        let rule = AnchorsRule::new();
        let content = r#"test:
  query: |-
    select * from table
    where id = *some_value
    and name = 'test'
"#;
        let issues = rule.check(content, "test.yaml");
        // Should NOT report errors for * characters in multiline strings
        assert!(
            issues.is_empty(),
            "Expected no issues for * in multiline strings, got: {:?}",
            issues
        );
    }

    #[test]
    fn test_anchors_check_sql_content() {
        let rule = AnchorsRule::new();
        let content = r#"source_by_hired_rate_hbar:
  queries:
    chart: |-
      select
          case when source_bucket is not null then source_bucket else 'No Source' end as source_bucket
        , round(100*cast(sum(case when status_category = 'hired' then 1 else 0 end) as float)/count(*)) as value
        , count(*) as total
        , sum(case when status_category = 'hired' then 1 else 0 end) as portion
      from source_analysis
"#;
        let issues = rule.check(content, "test.yaml");
        // Should NOT report errors for * characters in SQL within multiline strings
        assert!(
            issues.is_empty(),
            "Expected no issues for * in SQL multiline strings, got: {:?}",
            issues
        );
    }

    #[test]
    fn test_anchors_check_duplicated_anchors() {
        let mut rule = AnchorsRule::new();
        rule.set_config(AnchorsConfig {
            forbid_undeclared_aliases: false,
            forbid_duplicated_anchors: true,
            forbid_unused_anchors: false,
        });

        let content = "- &anchor Foo Bar\n- &anchor [item 1, item 2]";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("duplicated anchor"));
    }

    #[test]
    fn test_anchors_check_unused_anchors() {
        let mut rule = AnchorsRule::new();
        rule.set_config(AnchorsConfig {
            forbid_undeclared_aliases: false,
            forbid_duplicated_anchors: false,
            forbid_unused_anchors: true,
        });

        let content = "- &anchor\n  foo: bar\n- items:\n  - item1\n  - item2";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("unused anchor"));
    }

    #[test]
    fn test_anchors_fix_no_changes() {
        let rule = AnchorsRule::new();
        let content = "- &anchor\n  foo: bar\n- *unknown";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
