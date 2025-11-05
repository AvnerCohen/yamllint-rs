use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct OctalValuesConfig {
    pub forbid_implicit_octal: bool,
    pub forbid_explicit_octal: bool,
}

#[derive(Debug, Clone)]
pub struct OctalValuesRule {
    config: OctalValuesConfig,
}

impl OctalValuesRule {
    pub fn new() -> Self {
        Self {
            config: OctalValuesConfig {
                forbid_implicit_octal: true,
                forbid_explicit_octal: true,
            },
        }
    }

    pub fn with_config(config: OctalValuesConfig) -> Self {
        Self { config }
    }

    fn is_forbidden_octal(&self, value: &str) -> Option<String> {
        let trimmed = value.trim();

        if self.config.forbid_implicit_octal && trimmed.starts_with('0') && trimmed.len() > 1 {
            let rest = &trimmed[1..];
            if rest.chars().all(|c| c.is_ascii_digit())
                && !trimmed.starts_with("0x")
                && !trimmed.starts_with("0b")
            {
                return Some("implicit octal".to_string());
            }
        }

        if self.config.forbid_explicit_octal && trimmed.starts_with("0o") {
            return Some("explicit octal".to_string());
        }

        None
    }
}

impl Rule for OctalValuesRule {
    fn rule_id(&self) -> &'static str {
        "octal-values"
    }

    fn rule_name(&self) -> &'static str {
        "Octal Values"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for forbidden octal values in YAML."
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

                if value_part.starts_with('"') || value_part.starts_with('\'') {
                    continue;
                }

                if let Some(forbidden_type) = self.is_forbidden_octal(value_part) {
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

impl Default for OctalValuesRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_octal_values_rule_default() {
        let rule = OctalValuesRule::new();
        assert_eq!(rule.rule_id(), "octal-values");
        assert_eq!(rule.default_severity(), Severity::Error);
        assert!(rule.is_enabled_by_default());
        assert!(!rule.can_fix());
    }

    #[test]
    fn test_octal_values_check_clean_values() {
        let rule = OctalValuesRule::new();
        let content = "normal: 42\nhex: 0x10\nbinary: 0b10\nquoted: '010'";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_octal_values_check_implicit_octal() {
        let rule = OctalValuesRule::new();
        let content = "octal: 010";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("forbidden implicit octal value"));
    }

    #[test]
    fn test_octal_values_check_explicit_octal() {
        let rule = OctalValuesRule::new();
        let content = "octal: 0o10";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("forbidden explicit octal value"));
    }

    #[test]
    fn test_octal_values_fix_no_changes() {
        let rule = OctalValuesRule::new();
        let content = "octal: 010";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
