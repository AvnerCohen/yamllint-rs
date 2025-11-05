use super::{
    base::{utils, BaseRuleWithRegex, LintIssueBuilder},
    Rule,
};
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct TruthyConfig {
    pub allowed_values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TruthyRule {
    base: BaseRuleWithRegex<TruthyConfig>,
}

impl TruthyRule {
    pub fn new() -> Self {
        Self {
            base: BaseRuleWithRegex::new(TruthyConfig {
                allowed_values: vec!["false".to_string(), "true".to_string()],
            }),
        }
    }

    pub fn with_config(config: TruthyConfig) -> Self {
        Self {
            base: BaseRuleWithRegex::new(config),
        }
    }

    pub fn config(&self) -> &TruthyConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: TruthyConfig) {
        self.base.set_config(config);
    }
}

impl Rule for TruthyRule {
    fn rule_id(&self) -> &'static str {
        "truthy"
    }

    fn rule_name(&self) -> &'static str {
        "Truthy"
    }

    fn rule_description(&self) -> &'static str {
        "Checks that truthy values are properly formatted."
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

    fn check(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            let words = line.split_whitespace();
            for word in words {
                let trimmed = word.trim_end_matches(',');
                if self.is_truthy_value(trimmed)
                    && !self
                        .base
                        .config()
                        .allowed_values
                        .contains(&trimmed.to_string())
                {
                    if let Some(pos) = line.find(trimmed) {
                        issues.push(Self::create_issue(
                            line_num,
                            pos + 1,
                            format!(
                                "truthy value should be one of [{}]",
                                self.base.config().allowed_values.join(", ")
                            ),
                            self.get_severity(),
                        ));
                    }
                }
            }
        }

        issues
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        let mut fixed_lines = Vec::new();
        let mut fixes_applied = 0;
        let mut base = self.base.clone();

        for line in content.lines() {
            let mut fixed_line = line.to_string();

            for word in line.split_whitespace() {
                let trimmed = word.trim_end_matches(',');
                if self.is_truthy_value(trimmed)
                    && !self
                        .base
                        .config()
                        .allowed_values
                        .contains(&trimmed.to_string())
                {
                    let replacement = self.get_replacement(trimmed);
                    if let Some(replacement) = replacement {
                        if word == trimmed {
                            let pattern = format!(r"\b{}\b", regex::escape(trimmed));
                            if let Ok(regex) = base.get_or_compile_pattern(&pattern) {
                                if regex.is_match(&fixed_line) {
                                    fixed_line = regex
                                        .replace_all(&fixed_line, replacement.as_str())
                                        .to_string();
                                    fixes_applied += 1;
                                }
                            }
                        } else if word == format!("{},", trimmed) {
                            let pattern = format!(r"\b{},", regex::escape(trimmed));
                            if let Ok(regex) = base.get_or_compile_pattern(&pattern) {
                                if regex.is_match(&fixed_line) {
                                    fixed_line = regex
                                        .replace_all(&fixed_line, &format!("{},", replacement))
                                        .to_string();
                                    fixes_applied += 1;
                                }
                            }
                        }
                    }
                }
            }

            fixed_lines.push(fixed_line);
        }

        let fixed_content =
            utils::join_lines_preserving_newlines(fixed_lines, content.ends_with('\n'));

        let changed = fixes_applied > 0;

        super::FixResult {
            content: fixed_content,
            changed,
            fixes_applied,
        }
    }
}

impl TruthyRule {
    fn is_truthy_value(&self, value: &str) -> bool {
        matches!(
            value.to_lowercase().as_str(),
            "yes"
                | "no"
                | "on"
                | "off"
                | "y"
                | "n"
                | "true"
                | "false"
                | "1"
                | "0"
                | "enable"
                | "disable"
                | "enabled"
                | "disabled"
        )
    }

    fn get_replacement(&self, value: &str) -> Option<String> {
        match value.to_lowercase().as_str() {
            "yes" | "y" | "on" | "1" | "enable" | "enabled" => Some("true".to_string()),
            "no" | "n" | "off" | "0" | "disable" | "disabled" => Some("false".to_string()),
            _ => None,
        }
    }
}

impl Default for TruthyRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_truthy_rule_default() {
        let rule = TruthyRule::new();
        assert_eq!(rule.rule_id(), "truthy");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_truthy_check_clean_values() {
        let rule = TruthyRule::new();
        let content = "key: true\nanother: false";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_truthy_check_invalid_values() {
        let rule = TruthyRule::new();
        let content = "key: yes\nanother: no\nvalue: on";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 3);
        assert!(issues[0].message.contains("truthy value should be one of"));
    }

    #[test]
    fn test_truthy_fix() {
        let rule = TruthyRule::new();
        let content = "key: yes\nanother: no";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 2);
        assert!(fix_result.content.contains("key: true"));
        assert!(fix_result.content.contains("another: false"));
    }

    #[test]
    fn test_truthy_fix_no_changes() {
        let rule = TruthyRule::new();
        let content = "key: true\nanother: false";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
