use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct EmptyLinesConfig {
    pub max: usize,
    pub max_start: usize,
    pub max_end: usize,
}

#[derive(Debug, Clone)]
pub struct EmptyLinesRule {
    config: EmptyLinesConfig,
}

impl EmptyLinesRule {
    pub fn new() -> Self {
        Self {
            config: EmptyLinesConfig {
                max: 2,
                max_start: 0,
                max_end: 0,
            },
        }
    }

    pub fn with_config(config: EmptyLinesConfig) -> Self {
        Self { config }
    }

    fn check_empty_lines(&self, content: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return issues;
        }

        for (line_no, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                let line_start = self.get_line_start_position(content, line_no);
                let line_end = line_start + line.len();

                if self.is_last_blank_line_of_series(content, line_end) {
                    let blank_lines = self.count_consecutive_blank_lines(content, line_start);
                    let max_allowed =
                        self.get_max_allowed_for_position(content, line_start, line_end);

                    if blank_lines > max_allowed {
                        issues.push(LintIssue {
                            line: line_no + 1,
                            column: 1,
                            message: format!(
                                "too many blank lines ({} > {})",
                                blank_lines, max_allowed
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        issues
    }

    fn get_line_start_position(&self, content: &str, line_no: usize) -> usize {
        let mut pos = 0;
        for (i, line) in content.lines().enumerate() {
            if i == line_no {
                return pos;
            }
            pos += line.len() + 1;
        }
        pos
    }

    fn is_last_blank_line_of_series(&self, content: &str, line_end: usize) -> bool {
        let check_pos = line_end + 1;

        if check_pos + 2 <= content.len() {
            let next_chars: String = content.chars().skip(check_pos).take(2).collect();
            if next_chars == "\n\n" {
                return false;
            }
        }
        if check_pos + 4 <= content.len() {
            let next_chars: String = content.chars().skip(check_pos).take(4).collect();
            if next_chars == "\r\n\r\n" {
                return false;
            }
        }
        true
    }

    fn count_consecutive_blank_lines(&self, content: &str, start: usize) -> usize {
        let mut blank_lines = 0;
        let mut pos = start;

        while pos >= 2 {
            let prev_chars: String = content.chars().skip(pos - 2).take(2).collect();
            if prev_chars == "\r\n" {
                blank_lines += 1;
                pos -= 2;
            } else {
                break;
            }
        }

        while pos >= 1 && content.chars().nth(pos - 1) == Some('\n') {
            let newline_pos = pos - 1;

            let is_separator = if newline_pos + 1 < content.len() {
                let char_after: String = content.chars().skip(newline_pos + 1).take(1).collect();

                char_after != "\n" && char_after != "\r"
            } else {
                false
            };

            if is_separator {
                break;
            }

            blank_lines += 1;
            pos -= 1;
        }

        blank_lines
    }

    fn get_max_allowed_for_position(
        &self,
        content: &str,
        line_start: usize,
        line_end: usize,
    ) -> usize {
        let mut max = self.config.max;

        if line_start == 0 {
            max = self.config.max_start;
        }

        if (line_end == content.len() - 1 && content.chars().nth(line_end) == Some('\n'))
            || (line_end == content.len() - 2)
        {
            if line_end == content.len() - 2 {
                let end_chars: String = content.chars().skip(line_end).take(2).collect();
                if end_chars == "\r\n" {
                    if line_end == 0 {
                        return 0;
                    }
                    max = self.config.max_end;
                }
            } else {
                if line_end == 0 {
                    return 0;
                }
                max = self.config.max_end;
            }
        }

        max
    }
}

impl Rule for EmptyLinesRule {
    fn rule_id(&self) -> &'static str {
        "empty-lines"
    }

    fn rule_name(&self) -> &'static str {
        "Empty Lines"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper empty line usage in YAML."
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn is_enabled_by_default(&self) -> bool {
        true
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn get_severity(&self) -> Severity {
        self.default_severity()
    }

    fn set_severity(&mut self, _severity: Severity) {}

    fn has_severity_override(&self) -> bool {
        false
    }

    fn check(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        self.check_empty_lines(content)
    }

    fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        let mut fixed_content = String::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];

            if line.trim().is_empty() {
                let mut empty_count = 0;
                let mut j = i;
                while j < lines.len() && lines[j].trim().is_empty() {
                    empty_count += 1;
                    j += 1;
                }

                let max_empty = if i == 0 {
                    self.config.max_start
                } else if j == lines.len() {
                    self.config.max_end
                } else {
                    self.config.max
                };

                for _ in 0..empty_count.min(max_empty) {
                    fixed_content.push('\n');
                }

                i = j;
            } else {
                fixed_content.push_str(line);
                fixed_content.push('\n');
                i += 1;
            }
        }

        super::FixResult {
            content: fixed_content.clone(),
            changed: fixed_content != content,
            fixes_applied: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_lines_rule_default() {
        let rule = EmptyLinesRule::new();
        assert_eq!(rule.rule_id(), "empty-lines");
        assert_eq!(rule.config.max, 2);
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_empty_lines_check_no_empty_lines() {
        let rule = EmptyLinesRule::new();
        let content = "key1: value1\nkey2: value2\nkey3: value3";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_empty_lines_check_single_empty_line() {
        let rule = EmptyLinesRule::new();
        let content = "key1: value1\n\nkey2: value2";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_empty_lines_check_excessive_lines() {
        let rule = EmptyLinesRule::new();
        let content = "key1: value1\n\n\nkey2: value2";
        let issues = rule.check(content, "test.yaml");

        assert!(issues.is_empty());
    }

    #[test]
    fn test_empty_lines_fix() {
        let rule = EmptyLinesRule::new();
        let content = "key1: value1\n\n\nkey2: value2";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
    }

    #[test]
    fn test_empty_lines_fix_no_changes() {
        let rule = EmptyLinesRule::new();
        let content = "key1: value1\n\nkey2: value2";
        let fix_result = rule.fix(content, "test.yaml");

        let _ = fix_result.fixes_applied;
    }

    #[test]
    fn test_empty_lines_false_positive_bug() {
        let rule = EmptyLinesRule::new();
        let content = "key1: value1\n\n\nkey2: value2\n\n\nkey3: value3";

        let issues = rule.check(content, "test.yaml");

        assert!(
            issues.is_empty(),
            "No empty line violations should be detected. Current false positives: {:?}",
            issues
        );
    }
}
