use crate::rules::base::BaseRule;
use crate::rules::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct CommentsConfig {
    pub min_spaces_from_content: usize,
}

impl Default for CommentsConfig {
    fn default() -> Self {
        Self {
            min_spaces_from_content: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommentsRule {
    base: BaseRule<CommentsConfig>,
}

impl CommentsRule {
    pub fn new() -> Self {
        Self {
            base: BaseRule::new(CommentsConfig::default()),
        }
    }

    pub fn with_config(config: CommentsConfig) -> Self {
        Self {
            base: BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &CommentsConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: CommentsConfig) {
        self.base.set_config(config);
    }

    pub fn get_severity(&self) -> Severity {
        self.base.get_severity(Severity::Warning)
    }

    pub fn set_severity(&mut self, severity: Severity) {
        self.base.set_severity(severity);
    }

    pub fn has_severity_override(&self) -> bool {
        self.base.has_severity_override()
    }

    pub fn create_issue(&self, line: usize, column: usize, message: String) -> LintIssue {
        LintIssue {
            line,
            column,
            message,
            severity: self.get_severity(),
        }
    }
}

impl Default for CommentsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for CommentsRule {
    fn rule_id(&self) -> &'static str {
        "comments"
    }

    fn rule_name(&self) -> &'static str {
        "Comments"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper spacing before comments."
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

    fn can_fix(&self) -> bool {
        true
    }

    fn check(&self, content: &str, file_path: &str) -> Vec<LintIssue> {
        self.check_impl(content, file_path)
    }
}

impl CommentsRule {
    pub fn check_impl(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            if let Some(comment_pos) = line.find('#') {
                let before_comment: String = line.chars().take(comment_pos).collect();
                if !before_comment.trim().is_empty() {
                    let spaces = before_comment
                        .chars()
                        .rev()
                        .take_while(|&c| c == ' ')
                        .count();
                    if spaces < self.config().min_spaces_from_content {
                        issues.push(self.create_issue(
                            line_num,
                            comment_pos + 1,
                            "too few spaces before comment".to_string(),
                        ));
                    }
                }
            }
        }

        issues
    }

    pub fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        let mut fixed_lines = Vec::new();
        let mut fixes_applied = 0;

        for line in content.lines() {
            if let Some(comment_pos) = line.find('#') {
                let before_comment: String = line.chars().take(comment_pos).collect();
                if !before_comment.trim().is_empty() {
                    let content_part = before_comment.trim_end();
                    let trailing_spaces = before_comment.len() - content_part.len();
                    if trailing_spaces < self.config().min_spaces_from_content {
                        let needed_spaces = self.config().min_spaces_from_content;
                        let _additional_spaces = needed_spaces - trailing_spaces;
                        let comment_part: String = line.chars().skip(comment_pos).collect();
                        let fixed_line = format!(
                            "{}{}{}",
                            content_part,
                            " ".repeat(needed_spaces),
                            comment_part
                        );
                        fixed_lines.push(fixed_line);
                        fixes_applied += 1;
                        continue;
                    }
                }
            }
            fixed_lines.push(line.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;
    use crate::Severity;

    #[test]
    fn test_comments_rule_default() {
        let rule = CommentsRule::new();
        assert_eq!(rule.rule_id(), "comments");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_comments_check_clean_lines() {
        let rule = CommentsRule::new();
        let content = "key: value  # comment\nanother: item";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_comments_check_insufficient_spacing() {
        let rule = CommentsRule::new();
        let content = "key: value # comment\nanother: item  # good comment";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line, 1);
        assert!(issues[0].message.contains("too few spaces before comment"));
    }

    #[test]
    fn test_comments_fix() {
        let rule = CommentsRule::new();
        let content = "key: value # comment\nanother: item";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.content.contains("key: value  # comment"));
    }

    #[test]
    fn test_comments_fix_no_changes() {
        let rule = CommentsRule::new();
        let content = "key: value  # comment\nanother: item";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
