use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct EmptyValuesConfig {
    pub forbid_empty: bool,
}

#[derive(Debug, Clone)]
pub struct EmptyValuesRule {
    config: EmptyValuesConfig,
}

impl EmptyValuesRule {
    pub fn new() -> Self {
        Self {
            config: EmptyValuesConfig { forbid_empty: true },
        }
    }

    pub fn with_config(config: EmptyValuesConfig) -> Self {
        Self { config }
    }

    fn is_empty_value(&self, value: &str) -> bool {
        let trimmed = value.trim();
        trimmed.is_empty() || trimmed == "null" || trimmed == "~" || trimmed == "\"\""
    }
}

impl Rule for EmptyValuesRule {
    fn rule_id(&self) -> &'static str {
        "empty-values"
    }

    fn rule_name(&self) -> &'static str {
        "Empty Values"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for empty values in YAML."
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn get_severity(&self) -> Severity {
        self.default_severity()
    }

    fn set_severity(&mut self, _severity: Severity) {}

    fn has_severity_override(&self) -> bool {
        false
    }

    fn check(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            if line.trim().starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let value_part = line[colon_pos + 1..].trim();

                if self.config.forbid_empty && self.is_empty_value(value_part) {
                    issues.push(LintIssue {
                        line: line_num,
                        column: colon_pos + 2,
                        message: "empty value not allowed".to_string(),
                        severity: self.get_severity(),
                    });
                }
            }
        }

        issues
    }

    fn can_fix(&self) -> bool {
        false
    }

    fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        super::FixResult {
            content: content.to_string(),
            changed: false,
            fixes_applied: 0,
        }
    }
}

impl Default for EmptyValuesRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_empty_values_rule_default() {
        let rule = EmptyValuesRule::new();
        assert_eq!(rule.rule_id(), "empty-values");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(!rule.can_fix());
    }

    #[test]
    fn test_empty_values_check_clean_values() {
        let rule = EmptyValuesRule::new();
        let content = "key1: value1\nkey2: \"not empty\"\nkey3: 42";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_empty_values_check_empty_values() {
        let rule = EmptyValuesRule::new();
        let content = "key1: \nkey2: \"\"\nkey3: null";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 3);
        assert!(issues[0].message.contains("empty value not allowed"));
        assert!(issues[1].message.contains("empty value not allowed"));
        assert!(issues[2].message.contains("empty value not allowed"));
    }

    #[test]
    fn test_empty_values_fix_no_changes() {
        let rule = EmptyValuesRule::new();
        let content = "key1: \nkey2: \"\"\nkey3: null";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
