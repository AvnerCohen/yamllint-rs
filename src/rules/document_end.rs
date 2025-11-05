use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct DocumentEndConfig {
    pub present: bool,
}

#[derive(Debug, Clone)]
pub struct DocumentEndRule {
    config: DocumentEndConfig,
}

impl DocumentEndRule {
    pub fn new() -> Self {
        Self {
            config: DocumentEndConfig { present: true },
        }
    }

    pub fn with_config(config: DocumentEndConfig) -> Self {
        Self { config }
    }
}

impl Rule for DocumentEndRule {
    fn rule_id(&self) -> &'static str {
        "document-end"
    }

    fn rule_name(&self) -> &'static str {
        "Document End"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for the presence of document end markers (...)."
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

        if content.trim().is_empty() {
            return issues;
        }

        let last_line = content.lines().last().unwrap_or("");
        let has_document_end = last_line.trim() == "...";

        if self.config.present && !has_document_end {
            let line_count = content.lines().count();
            issues.push(LintIssue {
                line: line_count,
                column: 1,
                message: "missing document end marker (...)".to_string(),
                severity: self.get_severity(),
            });
        } else if !self.config.present && has_document_end {
            let line_count = content.lines().count();
            issues.push(LintIssue {
                line: line_count,
                column: 1,
                message: "document end marker (...) should not be present".to_string(),
                severity: self.get_severity(),
            });
        }

        issues
    }

    fn can_fix(&self) -> bool {
        true
    }

    fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        if content.trim().is_empty() {
            return super::FixResult {
                content: content.to_string(),
                changed: false,
                fixes_applied: 0,
            };
        }

        let last_line = content.lines().last().unwrap_or("");
        let has_document_end = last_line.trim() == "...";

        let mut fixed_content = content.to_string();
        let mut fixes_applied = 0;

        if self.config.present && !has_document_end {
            if content.ends_with('\n') {
                fixed_content = format!("{}...\n", content.trim_end());
            } else {
                fixed_content = format!("{}\n...", content);
            }
            fixes_applied = 1;
        } else if !self.config.present && has_document_end {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() > 1 {
                fixed_content = lines[..lines.len() - 1].join("\n");
            } else {
                fixed_content = "".to_string();
            }
            fixes_applied = 1;
        }

        if content.ends_with('\n') && !fixed_content.ends_with('\n') {
            fixed_content.push('\n');
        }

        let changed = fixes_applied > 0;

        super::FixResult {
            content: fixed_content,
            changed,
            fixes_applied,
        }
    }
}

impl Default for DocumentEndRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_document_end_rule_default() {
        let rule = DocumentEndRule::new();
        assert_eq!(rule.rule_id(), "document-end");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_document_end_check_clean_with_marker() {
        let rule = DocumentEndRule::new();
        let content = "key: value\n...";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_document_end_check_missing_marker() {
        let rule = DocumentEndRule::new();
        let content = "key: value";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("missing document end marker"));
    }

    #[test]
    fn test_document_end_fix_add_marker() {
        let rule = DocumentEndRule::new();
        let content = "key: value";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.content.ends_with("..."));
    }

    #[test]
    fn test_document_end_fix_no_changes() {
        let rule = DocumentEndRule::new();
        let content = "key: value\n...";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
