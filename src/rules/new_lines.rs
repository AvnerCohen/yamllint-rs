use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct NewLinesConfig {
    pub line_type: String,
}

#[derive(Debug, Clone)]
pub struct NewLinesRule {
    config: NewLinesConfig,
}

impl NewLinesRule {
    pub fn new() -> Self {
        Self {
            config: NewLinesConfig {
                line_type: "unix".to_string(),
            },
        }
    }

    pub fn with_config(config: NewLinesConfig) -> Self {
        Self { config }
    }

    fn check_newline_type(&self, content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if content.is_empty() {
            return issues;
        }

        let _expected_newline = match self.config.line_type.as_str() {
            "unix" => "\n",
            "dos" => "\r\n",
            "mac" => "\r",
            _ => "\n",
        };

        let has_unix = content.contains('\n');
        let has_dos = content.contains("\r\n");
        let has_mac = content.contains('\r') && !content.contains("\r\n");

        let mut found_types = Vec::new();
        if has_unix {
            found_types.push("unix");
        }
        if has_dos {
            found_types.push("dos");
        }
        if has_mac {
            found_types.push("mac");
        }

        if found_types.len() > 1 {
            issues.push(LintIssue {
                line: 1,
                column: 1,
                message: format!("mixed line endings found: {}", found_types.join(", ")),
                severity: self.get_severity(),
            });
        } else if !found_types.is_empty() && found_types[0] != self.config.line_type {
            issues.push(LintIssue {
                line: 1,
                column: 1,
                message: format!(
                    "wrong line ending type: expected {}, found {}",
                    self.config.line_type, found_types[0]
                ),
                severity: self.get_severity(),
            });
        }

        issues
    }
}

impl Rule for NewLinesRule {
    fn rule_id(&self) -> &'static str {
        "new-lines"
    }

    fn rule_name(&self) -> &'static str {
        "New Lines"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper newline character usage in YAML."
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
        self.check_newline_type(content)
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        if content.is_empty() {
            return super::FixResult {
                content: content.to_string(),
                changed: false,
                fixes_applied: 0,
            };
        }

        let target_newline = match self.config.line_type.as_str() {
            "unix" => "\n",
            "dos" => "\r\n",
            "mac" => "\r",
            _ => "\n",
        };

        let mut fixed_content = content.to_string();
        let mut fixes_applied = 0;

        let needs_conversion = if target_newline == "\n" {
            content.contains("\r\n") || content.contains('\r')
        } else {
            !content.ends_with(target_newline) || content.contains("\r\n") || content.contains('\r')
        };

        if needs_conversion {
            fixed_content = fixed_content.replace("\r\n", "\n").replace("\r", "\n");

            if target_newline != "\n" {
                fixed_content = fixed_content.replace("\n", target_newline);
            }
            fixes_applied = 1;
        }

        let changed = fixes_applied > 0;

        super::FixResult {
            content: fixed_content,
            changed,
            fixes_applied,
        }
    }
}

impl Default for NewLinesRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_new_lines_rule_default() {
        let rule = NewLinesRule::new();
        assert_eq!(rule.rule_id(), "new-lines");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_new_lines_check_clean_unix() {
        let rule = NewLinesRule::new();
        let content = "key: value\n";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_new_lines_check_wrong_type() {
        let rule = NewLinesRule::new();
        let content = "key: value\r\n";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("mixed line endings"));
    }

    #[test]
    fn test_new_lines_fix() {
        let rule = NewLinesRule::new();
        let content = "key: value\r\n";
        let fix_result = rule.fix(content, "test.yaml");
        println!("Fix result: {:?}", fix_result);
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.content.ends_with('\n'));
    }

    #[test]
    fn test_new_lines_fix_no_changes() {
        let rule = NewLinesRule::new();
        let content = "key: value\n";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
