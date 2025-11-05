use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct KeyOrderingConfig {
    pub require_alphabetical: bool,
}

#[derive(Debug, Clone)]
pub struct KeyOrderingRule {
    config: KeyOrderingConfig,
}

impl KeyOrderingRule {
    pub fn new() -> Self {
        Self {
            config: KeyOrderingConfig {
                require_alphabetical: true,
            },
        }
    }

    pub fn with_config(config: KeyOrderingConfig) -> Self {
        Self { config }
    }

    fn extract_keys(&self, content: &str) -> Vec<(usize, String)> {
        let mut keys = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            if line.trim().starts_with('#') || line.trim().is_empty() {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let key_part = line[..colon_pos].trim();
                if !key_part.is_empty() {
                    keys.push((line_num, key_part.to_string()));
                }
            }
        }

        keys
    }

    fn check_alphabetical_order(&self, keys: &[(usize, String)]) -> Vec<usize> {
        let mut violations = Vec::new();

        for i in 1..keys.len() {
            if keys[i].1 < keys[i - 1].1 {
                violations.push(keys[i].0);
            }
        }

        violations
    }
}

impl Rule for KeyOrderingRule {
    fn rule_id(&self) -> &'static str {
        "key-ordering"
    }

    fn rule_name(&self) -> &'static str {
        "Key Ordering"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper key ordering in YAML mappings."
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

        if self.config.require_alphabetical {
            let keys = self.extract_keys(content);
            let violations = self.check_alphabetical_order(&keys);

            for line_num in violations {
                issues.push(LintIssue {
                    line: line_num,
                    column: 1,
                    message: "keys not in alphabetical order".to_string(),
                    severity: self.get_severity(),
                });
            }
        }

        issues
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        let mut fixes_applied = 0;

        let keys = self.extract_keys(content);
        if keys.is_empty() {
            return super::FixResult {
                content: content.to_string(),
                changed: false,
                fixes_applied: 0,
            };
        }

        let mut sorted_keys = keys.clone();
        sorted_keys.sort_by(|a, b| a.1.cmp(&b.1));

        let needs_reordering = keys.iter().zip(sorted_keys.iter()).any(|(a, b)| a.1 != b.1);

        if needs_reordering {
            let mut new_lines = Vec::new();
            for (_line_num, line) in content.lines().enumerate() {
                if let Some(colon_pos) = line.find(':') {
                    let key_part = line[..colon_pos].trim();
                    if !key_part.is_empty() && !line.trim().starts_with('#') {
                        if let Some((_, sorted_key)) =
                            sorted_keys.iter().find(|(_, k)| k == key_part)
                        {
                            let new_line = line.replace(key_part, sorted_key);
                            new_lines.push(new_line);
                            fixes_applied += 1;
                        } else {
                            new_lines.push(line.to_string());
                        }
                    } else {
                        new_lines.push(line.to_string());
                    }
                } else {
                    new_lines.push(line.to_string());
                }
            }

            let fixed_content = if content.ends_with('\n') {
                new_lines.join("\n") + "\n"
            } else {
                new_lines.join("\n")
            };

            let changed = fixes_applied > 0;

            super::FixResult {
                content: fixed_content,
                changed,
                fixes_applied,
            }
        } else {
            super::FixResult {
                content: content.to_string(),
                changed: false,
                fixes_applied: 0,
            }
        }
    }
}

impl Default for KeyOrderingRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_key_ordering_rule_default() {
        let rule = KeyOrderingRule::new();
        assert_eq!(rule.rule_id(), "key-ordering");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_key_ordering_check_clean_ordering() {
        let rule = KeyOrderingRule::new();
        let content = "apple: red\nbanana: yellow\ncherry: red";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_key_ordering_check_bad_ordering() {
        let rule = KeyOrderingRule::new();
        let content = "cherry: red\napple: red\nbanana: yellow";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("keys not in alphabetical order"));
    }

    #[test]
    fn test_key_ordering_fix() {
        let rule = KeyOrderingRule::new();
        let content = "cherry: red\napple: red\nbanana: yellow";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert!(fix_result.fixes_applied > 0);
    }

    #[test]
    fn test_key_ordering_fix_no_changes() {
        let rule = KeyOrderingRule::new();
        let content = "apple: red\nbanana: yellow\ncherry: red";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
