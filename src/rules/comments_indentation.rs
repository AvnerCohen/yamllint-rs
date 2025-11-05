use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct CommentsIndentationConfig {
    pub indent: usize,
}

#[derive(Debug, Clone)]
pub struct CommentsIndentationRule {
    config: CommentsIndentationConfig,
    severity_override: Option<crate::Severity>,
}

impl CommentsIndentationRule {
    pub fn new() -> Self {
        Self {
            config: CommentsIndentationConfig { indent: 2 },
            severity_override: None,
        }
    }

    pub fn with_config(config: CommentsIndentationConfig) -> Self {
        Self {
            config,
            severity_override: None,
        }
    }
}

impl Rule for CommentsIndentationRule {
    fn rule_id(&self) -> &'static str {
        "comments-indentation"
    }

    fn rule_name(&self) -> &'static str {
        "Comments Indentation"
    }

    fn rule_description(&self) -> &'static str {
        "Checks that comments are properly indented."
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn get_severity(&self) -> Severity {
        self.severity_override
            .unwrap_or_else(|| self.default_severity())
    }

    fn set_severity(&mut self, severity: Severity) {
        self.severity_override = Some(severity);
    }

    fn has_severity_override(&self) -> bool {
        self.severity_override.is_some()
    }

    fn check(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            if line.trim().is_empty() {
                continue;
            }

            if let Some(comment_pos) = line.find('#') {
                let before_comment: String = line.chars().take(comment_pos).collect();
                if before_comment.trim().is_empty() {
                    let current_indent = before_comment.len();
                    if current_indent % self.config.indent != 0 {
                        issues.push(LintIssue {
                            line: line_num,
                            column: 1,
                            message: format!(
                                "comment not indented like content (expected {} spaces, found {})",
                                (current_indent / self.config.indent + 1) * self.config.indent,
                                current_indent
                            ),
                            severity: self.get_severity(),
                        });
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

            if line.trim().is_empty() {
                fixed_lines.push(fixed_line);
                continue;
            }

            if let Some(comment_pos) = line.find('#') {
                let before_comment: String = line.chars().take(comment_pos).collect();
                if before_comment.trim().is_empty() {
                    let current_indent = before_comment.len();
                    if current_indent % self.config.indent != 0 {
                        let expected_indent =
                            ((current_indent / self.config.indent) + 1) * self.config.indent;
                        let spaces = " ".repeat(expected_indent);

                        let comment_part: String = line.chars().skip(comment_pos).collect();
                        fixed_line = format!("{}{}", spaces, comment_part);
                        fixes_applied += 1;
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

impl Default for CommentsIndentationRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_comments_indentation_rule_default() {
        let rule = CommentsIndentationRule::new();
        assert_eq!(rule.rule_id(), "comments-indentation");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_comments_indentation_check_clean_lines() {
        let rule = CommentsIndentationRule::new();
        let content = "key: value\n  # comment\n  another: item";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_comments_indentation_check_bad_indentation() {
        let rule = CommentsIndentationRule::new();
        let content = "key: value\n # comment\n  another: item";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0]
            .message
            .contains("comment not indented like content"));
    }

    #[test]
    fn test_comments_indentation_fix() {
        let rule = CommentsIndentationRule::new();
        let content = "key: value\n # comment\n  another: item";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.content.contains("  # comment"));
    }

    #[test]
    fn test_comments_indentation_fix_no_changes() {
        let rule = CommentsIndentationRule::new();
        let content = "key: value\n  # comment\n  another: item";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
