use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct FloatValuesConfig {
    pub forbid_nan: bool,
    pub forbid_inf: bool,
}

#[derive(Debug, Clone)]
pub struct FloatValuesRule {
    config: FloatValuesConfig,
}

impl FloatValuesRule {
    pub fn new() -> Self {
        Self {
            config: FloatValuesConfig {
                forbid_nan: true,
                forbid_inf: true,
            },
        }
    }

    pub fn with_config(config: FloatValuesConfig) -> Self {
        Self { config }
    }

    fn is_forbidden_float(&self, value: &str) -> Option<String> {
        let trimmed = value.trim();

        if self.config.forbid_nan && (trimmed == ".NaN" || trimmed == ".nan" || trimmed == ".NAN") {
            return Some("NaN".to_string());
        }

        if self.config.forbid_inf
            && (trimmed == ".inf"
                || trimmed == ".Inf"
                || trimmed == ".INF"
                || trimmed == "-.inf"
                || trimmed == "-.Inf"
                || trimmed == "-.INF")
        {
            return Some("infinity".to_string());
        }

        None
    }
}

impl Rule for FloatValuesRule {
    fn rule_id(&self) -> &'static str {
        "float-values"
    }

    fn rule_name(&self) -> &'static str {
        "Float Values"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for forbidden float values in YAML."
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
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

                if let Some(forbidden_type) = self.is_forbidden_float(value_part) {
                    issues.push(LintIssue {
                        line: line_num,
                        column: colon_pos + 2,
                        message: format!("forbidden {} value", forbidden_type),
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

impl Default for FloatValuesRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_float_values_rule_default() {
        let rule = FloatValuesRule::new();
        assert_eq!(rule.rule_id(), "float-values");
        assert_eq!(rule.default_severity(), Severity::Error);
        assert!(rule.is_enabled_by_default());
        assert!(!rule.can_fix());
    }

    #[test]
    fn test_float_values_check_clean_values() {
        let rule = FloatValuesRule::new();
        let content = "normal_float: 3.14\ninteger: 42\nstring: \"hello\"";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_float_values_check_forbidden_nan() {
        let rule = FloatValuesRule::new();
        let content = "nan_value: .NaN";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("forbidden NaN value"));
    }

    #[test]
    fn test_float_values_check_forbidden_inf() {
        let rule = FloatValuesRule::new();
        let content = "inf_value: .inf";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("forbidden infinity value"));
    }

    #[test]
    fn test_float_values_fix_no_changes() {
        let rule = FloatValuesRule::new();
        let content = "nan_value: .NaN";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
