use crate::{LintIssue, Severity};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BaseRule<T> {
    pub config: T,
    pub severity_override: Option<Severity>,
}

impl<T> BaseRule<T> {
    pub fn new(config: T) -> Self {
        Self {
            config,
            severity_override: None,
        }
    }

    pub fn with_config(config: T) -> Self {
        Self {
            config,
            severity_override: None,
        }
    }

    pub fn config(&self) -> &T {
        &self.config
    }

    pub fn set_config(&mut self, config: T) {
        self.config = config;
    }

    pub fn get_severity(&self, default_severity: Severity) -> Severity {
        self.severity_override.unwrap_or(default_severity)
    }

    pub fn set_severity(&mut self, severity: Severity) {
        self.severity_override = Some(severity);
    }

    pub fn has_severity_override(&self) -> bool {
        self.severity_override.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct BaseRuleWithRegex<T> {
    pub config: T,
    pub severity_override: Option<Severity>,
    pub compiled_patterns: HashMap<String, Regex>,
}

impl<T> BaseRuleWithRegex<T> {
    pub fn new(config: T) -> Self {
        Self {
            config,
            severity_override: None,
            compiled_patterns: HashMap::new(),
        }
    }

    pub fn with_config(config: T) -> Self {
        Self {
            config,
            severity_override: None,
            compiled_patterns: HashMap::new(),
        }
    }

    pub fn config(&self) -> &T {
        &self.config
    }

    pub fn set_config(&mut self, config: T) {
        self.config = config;
    }

    pub fn get_severity(&self, default_severity: Severity) -> Severity {
        self.severity_override.unwrap_or(default_severity)
    }

    pub fn set_severity(&mut self, severity: Severity) {
        self.severity_override = Some(severity);
    }

    pub fn has_severity_override(&self) -> bool {
        self.severity_override.is_some()
    }

    pub fn get_or_compile_pattern(&mut self, pattern: &str) -> Result<&Regex, regex::Error> {
        if !self.compiled_patterns.contains_key(pattern) {
            let regex = Regex::new(pattern)?;
            self.compiled_patterns.insert(pattern.to_string(), regex);
        }
        Ok(self.compiled_patterns.get(pattern).unwrap())
    }

    pub fn get_cached_pattern(&self, pattern: &str) -> &Regex {
        self.compiled_patterns
            .get(pattern)
            .expect("Pattern should be cached")
    }
}

pub trait LintIssueBuilder {
    fn create_issue(line: usize, column: usize, message: String, severity: Severity) -> LintIssue {
        LintIssue {
            line,
            column,
            message,
            severity,
        }
    }

    fn create_line_issue(
        line: usize,
        column: usize,
        message: String,
        severity: Severity,
    ) -> LintIssue {
        Self::create_issue(line, column, message, severity)
    }
}

impl<T> LintIssueBuilder for T {}

pub mod utils {
    pub fn is_empty_or_comment(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.is_empty() || trimmed.starts_with('#')
    }

    pub fn calculate_indentation(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }

    pub fn has_trailing_whitespace(line: &str) -> bool {
        line.ends_with(' ') || line.ends_with('\t')
    }

    pub fn count_trailing_whitespace(line: &str) -> usize {
        line.len() - line.trim_end().len()
    }

    pub fn join_lines_preserving_newlines(
        lines: Vec<String>,
        original_ends_with_newline: bool,
    ) -> String {
        if original_ends_with_newline {
            lines.join("\n") + "\n"
        } else {
            lines.join("\n")
        }
    }
}
