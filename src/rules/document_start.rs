use super::Rule;
use crate::{LintIssue, Severity};

#[derive(Debug, Clone)]
pub struct DocumentStartConfig {
    pub present: bool,
}

#[derive(Debug, Clone)]
pub struct DocumentStartRule {
    config: DocumentStartConfig,
}

impl DocumentStartRule {
    pub fn new() -> Self {
        Self {
            config: DocumentStartConfig { present: true },
        }
    }

    pub fn with_config(config: DocumentStartConfig) -> Self {
        Self { config }
    }
}

impl Rule for DocumentStartRule {
    fn rule_id(&self) -> &'static str {
        "document-start"
    }

    fn rule_name(&self) -> &'static str {
        "Document Start"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for the presence of document start markers (---)."
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

        let first_line = content.lines().next().unwrap_or("");
        let has_document_start = first_line.trim() == "---";

        if self.config.present && !has_document_start {
            issues.push(LintIssue {
                line: 1,
                column: 1,
                message: "missing document start \"---\"".to_string(),
                severity: self.get_severity(),
            });
        } else if !self.config.present && has_document_start {
            issues.push(LintIssue {
                line: 1,
                column: 1,
                message: "document start marker (---) should not be present".to_string(),
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

        let first_line = content.lines().next().unwrap_or("");
        let has_document_start = first_line.trim() == "---";

        let mut fixed_content = content.to_string();
        let mut fixes_applied = 0;

        if self.config.present && !has_document_start {
            if content.ends_with('\n') {
                fixed_content = format!("---\n{}", content);
            } else {
                fixed_content = format!("---\n{}\n", content);
            }
            fixes_applied = 1;
        } else if !self.config.present && has_document_start {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() > 1 {
                fixed_content = lines[1..].join("\n");
            } else {
                fixed_content = "".to_string();
            }
            fixes_applied = 1;
        }

        if content.ends_with('\n') && !fixed_content.ends_with('\n') {
            fixed_content.push('\n');
        } else if !content.ends_with('\n') && fixed_content.ends_with('\n') && !self.config.present
        {
            fixed_content.pop();
        }

        let changed = fixes_applied > 0;

        super::FixResult {
            content: fixed_content,
            changed,
            fixes_applied,
        }
    }
}

impl Default for DocumentStartRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;

    #[test]
    fn test_document_start_rule_default() {
        let rule = DocumentStartRule::new();
        assert_eq!(rule.rule_id(), "document-start");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_document_start_check_clean_with_marker() {
        let rule = DocumentStartRule::new();
        let content = "---\nkey: value";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_document_start_check_missing_marker() {
        let rule = DocumentStartRule::new();
        let content = "key: value";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("missing document start"));
    }

    #[test]
    fn test_document_start_fix_add_marker() {
        let rule = DocumentStartRule::new();
        let content = "key: value";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.content.starts_with("---\n"));
    }

    #[test]
    fn test_document_start_fix_no_changes() {
        let rule = DocumentStartRule::new();
        let content = "---\nkey: value";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
