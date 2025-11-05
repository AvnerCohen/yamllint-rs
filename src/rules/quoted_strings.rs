use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct QuotedStringsConfig {
    pub required: String,
    pub quote_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QuotedStringsRule {
    config: QuotedStringsConfig,
}

impl QuotedStringsRule {
    pub fn new() -> Self {
        Self {
            config: QuotedStringsConfig {
                required: "only-when-needed".to_string(),
                quote_type: None,
            },
        }
    }

    pub fn with_config(config: QuotedStringsConfig) -> Self {
        Self { config }
    }

    fn needs_quoting(&self, value: &str) -> bool {
        if value.parse::<i64>().is_ok() || value.parse::<f64>().is_ok() {
            return true;
        }

        if matches!(
            value.to_lowercase().as_str(),
            "true" | "false" | "yes" | "no" | "on" | "off"
        ) {
            return true;
        }

        if value.starts_with('#') || value.starts_with('[') || value.starts_with('{') {
            return true;
        }

        if value.contains(':') || value.contains('|') || value.contains('>') {
            return true;
        }

        false
    }

    fn is_properly_quoted(&self, value: &str) -> bool {
        if value.starts_with('"') && value.ends_with('"') {
            return true;
        }
        if value.starts_with('\'') && value.ends_with('\'') {
            return true;
        }
        false
    }

    fn has_correct_quote_type(&self, value: &str) -> bool {
        if let Some(quote_type) = &self.config.quote_type {
            match quote_type.as_str() {
                "single" => value.starts_with('\'') && value.ends_with('\''),
                "double" => value.starts_with('"') && value.ends_with('"'),
                _ => true,
            }
        } else {
            true
        }
    }
}

impl Rule for QuotedStringsRule {
    fn rule_id(&self) -> &'static str {
        "quoted-strings"
    }

    fn rule_name(&self) -> &'static str {
        "Quoted Strings"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper string quoting in YAML."
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

                if value_part.is_empty() {
                    continue;
                }

                match self.config.required.as_str() {
                    "true" => {
                        if !self.is_properly_quoted(value_part) {
                            issues.push(LintIssue {
                                line: line_num,
                                column: colon_pos + 2,
                                message: "string value must be quoted".to_string(),
                                severity: self.get_severity(),
                            });
                        } else if !self.has_correct_quote_type(value_part) {
                            issues.push(LintIssue {
                                line: line_num,
                                column: colon_pos + 2,
                                message: format!(
                                    "string must use {} quotes",
                                    self.config.quote_type.as_ref().unwrap()
                                ),
                                severity: self.get_severity(),
                            });
                        }
                    }
                    "only-when-needed" => {
                        if self.needs_quoting(value_part) && !self.is_properly_quoted(value_part) {
                            issues.push(LintIssue {
                                line: line_num,
                                column: colon_pos + 2,
                                message: "string value must be quoted".to_string(),
                                severity: self.get_severity(),
                            });
                        } else if self.is_properly_quoted(value_part)
                            && !self.needs_quoting(value_part)
                        {
                            issues.push(LintIssue {
                                line: line_num,
                                column: colon_pos + 2,
                                message: "string value should not be quoted".to_string(),
                                severity: self.get_severity(),
                            });
                        } else if self.is_properly_quoted(value_part)
                            && !self.has_correct_quote_type(value_part)
                        {
                            issues.push(LintIssue {
                                line: line_num,
                                column: colon_pos + 2,
                                message: format!(
                                    "string must use {} quotes",
                                    self.config.quote_type.as_ref().unwrap()
                                ),
                                severity: self.get_severity(),
                            });
                        }
                    }
                    _ => {
                        if self.is_properly_quoted(value_part) {
                            issues.push(LintIssue {
                                line: line_num,
                                column: colon_pos + 2,
                                message: "string value should not be quoted".to_string(),
                                severity: self.get_severity(),
                            });
                        }
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

        for line in content.lines() {
            let mut fixed_line = line.to_string();

            if line.trim().starts_with('#') || line.trim().is_empty() {
                fixed_lines.push(fixed_line);
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let value_part = line[colon_pos + 1..].trim();

                if value_part.is_empty() {
                    fixed_lines.push(fixed_line);
                    continue;
                }

                let quote_char = match self.config.quote_type.as_ref() {
                    Some(quote_type) => match quote_type.as_str() {
                        "single" => "'",
                        "double" => "\"",
                        _ => "\"",
                    },
                    None => "\"",
                };

                match self.config.required.as_str() {
                    "true" => {
                        if !self.is_properly_quoted(value_part) {
                            let unquoted_value = value_part.trim_matches('"').trim_matches('\'');
                            let new_value =
                                format!("{}{}{}", quote_char, unquoted_value, quote_char);
                            fixed_line = format!("{}: {}", &line[..colon_pos], new_value);
                            fixes_applied += 1;
                        } else if !self.has_correct_quote_type(value_part) {
                            let unquoted_value = value_part.trim_matches('"').trim_matches('\'');
                            let new_value =
                                format!("{}{}{}", quote_char, unquoted_value, quote_char);
                            fixed_line = format!("{}: {}", &line[..colon_pos], new_value);
                            fixes_applied += 1;
                        }
                    }
                    "only-when-needed" => {
                        if self.needs_quoting(value_part) && !self.is_properly_quoted(value_part) {
                            let new_value = format!("{}{}{}", quote_char, value_part, quote_char);
                            fixed_line = format!("{}: {}", &line[..colon_pos], new_value);
                            fixes_applied += 1;
                        } else if self.is_properly_quoted(value_part)
                            && !self.needs_quoting(value_part)
                        {
                            let unquoted_value = value_part.trim_matches('"').trim_matches('\'');
                            fixed_line = format!("{}: {}", &line[..colon_pos], unquoted_value);
                            fixes_applied += 1;
                        } else if self.is_properly_quoted(value_part)
                            && !self.has_correct_quote_type(value_part)
                        {
                            let unquoted_value = value_part.trim_matches('"').trim_matches('\'');
                            let new_value =
                                format!("{}{}{}", quote_char, unquoted_value, quote_char);
                            fixed_line = format!("{}: {}", &line[..colon_pos], new_value);
                            fixes_applied += 1;
                        }
                    }
                    _ => {
                        if self.is_properly_quoted(value_part) {
                            let unquoted_value = value_part.trim_matches('"').trim_matches('\'');
                            fixed_line = format!("{}: {}", &line[..colon_pos], unquoted_value);
                            fixes_applied += 1;
                        }
                    }
                }
            }

            fixed_lines.push(fixed_line);
        }

        let fixed_content = if content.ends_with('\n') {
            fixed_lines.join("\n") + "\n"
        } else {
            fixed_lines.join("\n")
        };

        let changed = fixes_applied > 0;

        super::FixResult {
            content: fixed_content,
            changed,
            fixes_applied,
        }
    }
}

impl Default for QuotedStringsRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_quoted_strings_rule_default() {
        let rule = QuotedStringsRule::new();
        assert_eq!(rule.rule_id(), "quoted-strings");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_quoted_strings_check_clean_strings() {
        let rule = QuotedStringsRule::new();
        let content = "foo: bar\nnormal: value\nanother: text";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_quoted_strings_check_needs_quoting() {
        let rule = QuotedStringsRule::new();
        let content = "not_number: 123\nnot_boolean: true";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("string value must be quoted"));
        assert!(issues[1].message.contains("string value must be quoted"));
    }

    #[test]
    fn test_quoted_strings_fix() {
        let rule = QuotedStringsRule::new();
        let content = "not_number: 123\nnot_boolean: true";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 2);
        assert!(fix_result.content.contains("not_number: \"123\""));
        assert!(fix_result.content.contains("not_boolean: \"true\""));
    }

    #[test]
    fn test_quoted_strings_fix_no_changes() {
        let rule = QuotedStringsRule::new();
        let content = "foo: bar\nnormal: value\nanother: text";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
