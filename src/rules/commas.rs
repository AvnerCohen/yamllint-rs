use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct CommasConfig {
    pub max_spaces_before: i32,
    pub min_spaces_after: i32,
    pub max_spaces_after: i32,
}

#[derive(Debug, Clone)]
pub struct CommasRule {
    config: CommasConfig,
}

impl CommasRule {
    pub fn new() -> Self {
        Self {
            config: CommasConfig {
                max_spaces_before: 0,
                min_spaces_after: 1,
                max_spaces_after: 1,
            },
        }
    }

    pub fn with_config(config: CommasConfig) -> Self {
        Self { config }
    }
}

impl Rule for CommasRule {
    fn rule_id(&self) -> &'static str {
        "commas"
    }

    fn rule_name(&self) -> &'static str {
        "Commas"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper spacing around commas ,."
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

            for (char_pos, ch) in line.char_indices() {
                if ch == ',' {
                    if self.config.max_spaces_before >= 0 {
                        let before_comma: String = line.chars().take(char_pos).collect();
                        let trailing_spaces = before_comma.len() - before_comma.trim_end().len();
                        if trailing_spaces > self.config.max_spaces_before as usize {
                            issues.push(LintIssue {
                                line: line_num,
                                column: char_pos + 1,
                                message: format!(
                                    "too many spaces before comma ({} > {})",
                                    trailing_spaces, self.config.max_spaces_before
                                ),
                                severity: self.get_severity(),
                            });
                        }
                    }

                    if self.config.min_spaces_after >= 0 || self.config.max_spaces_after >= 0 {
                        let after_comma: String = line.chars().skip(char_pos + 1).collect();
                        let leading_spaces = after_comma.len() - after_comma.trim_start().len();

                        if self.config.min_spaces_after >= 0
                            && leading_spaces < self.config.min_spaces_after as usize
                        {
                            issues.push(LintIssue {
                                line: line_num,
                                column: char_pos + 1,
                                message: format!(
                                    "too few spaces after comma ({} < {})",
                                    leading_spaces, self.config.min_spaces_after
                                ),
                                severity: self.get_severity(),
                            });
                        }

                        if self.config.max_spaces_after >= 0
                            && leading_spaces > self.config.max_spaces_after as usize
                        {
                            issues.push(LintIssue {
                                line: line_num,
                                column: char_pos + 1,
                                message: format!(
                                    "too many spaces after comma ({} > {})",
                                    leading_spaces, self.config.max_spaces_after
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

    fn can_fix(&self) -> bool {
        true
    }

    fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        let mut fixed_lines = Vec::new();
        let mut fixes_applied = 0;

        for line in content.lines() {
            let mut fixed_line = line.to_string();

            for (char_pos, ch) in line.char_indices() {
                if ch == ',' {
                    let before_comma: String = line.chars().take(char_pos).collect();
                    let after_comma: String = line.chars().skip(char_pos + 1).collect();

                    let before_trimmed = before_comma.trim_end();
                    let before_spaces = before_comma.len() - before_trimmed.len();
                    let target_before_spaces = if self.config.max_spaces_before >= 0 {
                        self.config.max_spaces_before as usize
                    } else {
                        before_spaces
                    };

                    let after_trimmed = after_comma.trim_start();
                    let after_spaces = after_comma.len() - after_trimmed.len();
                    let target_after_spaces = if self.config.min_spaces_after >= 0 {
                        self.config.min_spaces_after as usize
                    } else if self.config.max_spaces_after >= 0 {
                        self.config.max_spaces_after as usize
                    } else {
                        after_spaces
                    };

                    if before_spaces != target_before_spaces || after_spaces != target_after_spaces
                    {
                        let new_before =
                            format!("{}{}", before_trimmed, " ".repeat(target_before_spaces));
                        let new_after =
                            format!("{}{}", " ".repeat(target_after_spaces), after_trimmed);

                        let after_comma_part: String = line.chars().skip(char_pos + 1).collect();
                        fixed_line = format!("{},{}{}", new_before, new_after, after_comma_part);
                        fixes_applied += 1;
                        break;
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

impl Default for CommasRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_commas_rule_default() {
        let rule = CommasRule::new();
        assert_eq!(rule.rule_id(), "commas");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_commas_check_clean_commas() {
        let rule = CommasRule::new();
        let content = "list: [a, b, c]\nmap: {key1: value1, key2: value2}";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_commas_check_spaces_before() {
        let rule = CommasRule::new();
        let content = "list: [a, b , c]";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("too many spaces before comma"));
    }

    #[test]
    fn test_commas_check_spaces_after() {
        let rule = CommasRule::new();
        let content = "list: [a, b,  c]";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("too many spaces after comma"));
    }

    #[test]
    fn test_commas_fix() {
        let rule = CommasRule::new();
        let content = "list: [a, b ,  c]";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.content.contains("list: [a, b, c]"));
    }

    #[test]
    fn test_commas_fix_no_changes() {
        let rule = CommasRule::new();
        let content = "list: [a, b, c]\nmap: {key1: value1, key2: value2}";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
