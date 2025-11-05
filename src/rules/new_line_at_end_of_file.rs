use crate::rules::base::BaseRule;
use crate::rules::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct NewLineAtEndOfFileConfig {
    pub require: bool,
}

impl Default for NewLineAtEndOfFileConfig {
    fn default() -> Self {
        Self { require: true }
    }
}

#[derive(Debug, Clone)]
pub struct NewLineAtEndOfFileRule {
    base: BaseRule<NewLineAtEndOfFileConfig>,
}

impl NewLineAtEndOfFileRule {
    pub fn new() -> Self {
        Self {
            base: BaseRule::new(NewLineAtEndOfFileConfig::default()),
        }
    }

    pub fn with_config(config: NewLineAtEndOfFileConfig) -> Self {
        Self {
            base: BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &NewLineAtEndOfFileConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: NewLineAtEndOfFileConfig) {
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

impl Default for NewLineAtEndOfFileRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for NewLineAtEndOfFileRule {
    fn rule_id(&self) -> &'static str {
        "new-line-at-end-of-file"
    }

    fn rule_name(&self) -> &'static str {
        "New Line at End of File"
    }

    fn rule_description(&self) -> &'static str {
        "Checks that files end with a newline character."
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

impl NewLineAtEndOfFileRule {
    pub fn check_impl(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if self.config().require && !content.is_empty() && !content.ends_with('\n') {
            let line_count = content.lines().count();
            let last_line = if content.ends_with('\r') {
                content.lines().last().unwrap_or("")
            } else {
                content.lines().last().unwrap_or("")
            };

            issues.push(self.create_issue(
                line_count,
                last_line.len() + 1,
                "no new line character at the end of file".to_string(),
            ));
        }

        issues
    }

    pub fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        if !self.config().require {
            return super::FixResult {
                content: content.to_string(),
                changed: false,
                fixes_applied: 0,
            };
        }

        let mut fixed_content = content.to_string();
        let mut fixes_applied = 0;

        if !content.is_empty() && !content.ends_with('\n') {
            fixed_content.push('\n');
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;
    use crate::Severity;

    #[test]
    fn test_new_line_at_end_of_file_rule_default() {
        let rule = NewLineAtEndOfFileRule::new();
        assert_eq!(rule.rule_id(), "new-line-at-end-of-file");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_new_line_at_end_of_file_check_clean_file() {
        let rule = NewLineAtEndOfFileRule::new();
        let content = "key: value\nanother: item\n";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_new_line_at_end_of_file_check_missing_newline() {
        let rule = NewLineAtEndOfFileRule::new();
        let content = "key: value\nanother: item";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0]
            .message
            .contains("no new line character at the end of file"));
    }

    #[test]
    fn test_new_line_at_end_of_file_check_empty_file() {
        let rule = NewLineAtEndOfFileRule::new();
        let content = "";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_new_line_at_end_of_file_fix() {
        let rule = NewLineAtEndOfFileRule::new();
        let content = "key: value\nanother: item";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.content.ends_with('\n'));
    }

    #[test]
    fn test_new_line_at_end_of_file_fix_no_changes() {
        let rule = NewLineAtEndOfFileRule::new();
        let content = "key: value\nanother: item\n";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }

    #[test]
    fn test_new_line_at_end_of_file_config_disabled() {
        let rule = NewLineAtEndOfFileRule::with_config(NewLineAtEndOfFileConfig { require: false });
        let content = "key: value\nanother: item";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }
}
