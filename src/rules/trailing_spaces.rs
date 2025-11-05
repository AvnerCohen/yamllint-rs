use super::{base::utils, base::BaseRule, FixResult, Rule};
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct TrailingSpacesConfig {
    pub allow: bool,
}

impl Default for TrailingSpacesConfig {
    fn default() -> Self {
        Self { allow: false }
    }
}

#[derive(Debug, Clone)]
pub struct TrailingSpacesRule {
    base: BaseRule<TrailingSpacesConfig>,
}

impl TrailingSpacesRule {
    pub fn new() -> Self {
        Self {
            base: BaseRule::new(TrailingSpacesConfig::default()),
        }
    }

    pub fn with_config(config: TrailingSpacesConfig) -> Self {
        Self {
            base: BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &TrailingSpacesConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: TrailingSpacesConfig) {
        self.base.set_config(config);
    }

    pub fn get_severity(&self) -> Severity {
        self.base.get_severity(self.default_severity())
    }

    pub fn set_severity(&mut self, severity: Severity) {
        self.base.set_severity(severity);
    }

    pub fn has_severity_override(&self) -> bool {
        self.base.has_severity_override()
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    pub fn create_issue(&self, line: usize, column: usize, message: String) -> LintIssue {
        LintIssue {
            line,
            column,
            message,
            severity: self.get_severity(),
        }
    }

    pub fn check_impl(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if self.config().allow {
            return issues;
        }

        for (line_num, line) in content.lines().enumerate() {
            if utils::has_trailing_whitespace(line) {
                let trailing_count = utils::count_trailing_whitespace(line);
                issues.push(self.create_issue(
                    line_num + 1,
                    line.len() - trailing_count + 1,
                    format!(
                        "trailing spaces ({} trailing character{})",
                        trailing_count,
                        if trailing_count == 1 { "" } else { "s" }
                    ),
                ));
            }
        }

        issues
    }
}

impl Rule for TrailingSpacesRule {
    fn rule_id(&self) -> &'static str {
        "trailing-spaces"
    }

    fn rule_name(&self) -> &'static str {
        "Trailing Spaces"
    }

    fn rule_description(&self) -> &'static str {
        "Ensures that lines do not have trailing whitespace"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
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

    fn fix(&self, content: &str, _file_path: &str) -> FixResult {
        if self.config().allow {
            return FixResult {
                content: content.to_string(),
                changed: false,
                fixes_applied: 0,
            };
        }

        let mut fixed_lines = Vec::new();
        let mut fixes_applied = 0;

        for line in content.lines() {
            let trimmed = line.trim_end();
            if trimmed.len() != line.len() {
                fixes_applied += 1;
            }
            fixed_lines.push(trimmed.to_string());
        }

        let fixed_content =
            utils::join_lines_preserving_newlines(fixed_lines, content.ends_with('\n'));

        let changed = fixes_applied > 0;

        FixResult {
            content: fixed_content,
            changed,
            fixes_applied,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_spaces_rule_default() {
        let rule = TrailingSpacesRule::new();
        assert_eq!(rule.rule_id(), "trailing-spaces");
        assert!(!rule.config().allow);
        assert!(rule.can_fix());
    }

    #[test]
    fn test_trailing_spaces_check_clean_lines() {
        let rule = TrailingSpacesRule::new();
        let content = "clean line\nanother clean line\n";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_trailing_spaces_check_trailing_spaces() {
        let rule = TrailingSpacesRule::new();
        let content = "line with spaces   \nclean line\nline with tabs\t\t\n";
        let issues = rule.check(content, "test.yaml");

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line, 1);
        assert!(issues[0].message.contains("trailing spaces"));
        assert_eq!(issues[1].line, 3);
    }

    #[test]
    fn test_trailing_spaces_fix() {
        let rule = TrailingSpacesRule::new();
        let content = "line with spaces   \nclean line\nline with tabs\t\t\n";
        let result = rule.fix(content, "test.yaml");

        assert!(result.changed);
        assert_eq!(result.fixes_applied, 2);
        assert_eq!(
            result.content,
            "line with spaces\nclean line\nline with tabs\n"
        );
    }

    #[test]
    fn test_trailing_spaces_fix_no_changes() {
        let rule = TrailingSpacesRule::new();
        let content = "clean line\nanother clean line\n";
        let result = rule.fix(content, "test.yaml");

        assert!(!result.changed);
        assert_eq!(result.fixes_applied, 0);
        assert_eq!(result.content, content);
    }

    #[test]
    fn test_trailing_spaces_fix_after_document_start() {
        let rule = TrailingSpacesRule::new();
        let content_after_doc_start = "---\nkey1: value1   \nkey2: value2\t\t\nkey3: value3\n";
        let result = rule.fix(content_after_doc_start, "test.yaml");

        assert!(result.changed);
        assert_eq!(result.fixes_applied, 2);
        assert_eq!(
            result.content,
            "---\nkey1: value1\nkey2: value2\nkey3: value3\n"
        );
    }

    #[test]
    fn test_trailing_spaces_allow_config() {
        let config = TrailingSpacesConfig { allow: true };
        let rule = TrailingSpacesRule::with_config(config);
        let content = "line with spaces   \nclean line\n";
        let issues = rule.check(content, "test.yaml");

        assert!(issues.is_empty());

        let result = rule.fix(content, "test.yaml");
        assert!(!result.changed);
    }
}
