//! Output formatting for lint issues.

use crate::{LintIssue, OutputFormat, Severity};

/// Formatter trait for output formatting
pub trait Formatter: Send + Sync {
    /// Format a single issue
    fn format_issue(&self, issue: &LintIssue, rule_name: &str) -> String;

    /// Format a filename
    fn format_filename(&self, filename: &str) -> String;
}

/// Standard (non-colored) formatter
pub struct StandardFormatter;

impl Formatter for StandardFormatter {
    fn format_issue(&self, issue: &LintIssue, rule_name: &str) -> String {
        let level = match issue.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };

        let location = format!("  {}:{}", issue.line, issue.column);
        let padding1 = " ".repeat((12 - location.len()).max(0));
        let with_severity = format!("{}{}{}", location, padding1, level);
        let padding2 = " ".repeat((21 - with_severity.len()).max(0));
        let rule_name_formatted = rule_name.replace("_", "-");
        format!(
            "{}{}{}  ({})\n",
            with_severity, padding2, issue.message, rule_name_formatted
        )
    }

    fn format_filename(&self, filename: &str) -> String {
        filename.to_string()
    }
}

/// Colored formatter
pub struct ColoredFormatter;

impl Formatter for ColoredFormatter {
    fn format_issue(&self, issue: &LintIssue, rule_name: &str) -> String {
        let level = match issue.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };

        let location_str = format!("{}:{}", issue.line, issue.column);
        let location = format!("\x1B[2m{}\x1B[0m", location_str);
        let padding1 = " ".repeat((11 - location_str.len()).max(0));
        let severity_colored = match issue.severity {
            Severity::Error => format!("\x1B[31m{}\x1B[0m", level),
            Severity::Warning => format!("\x1B[33m{}\x1B[0m", level),
            Severity::Info => level.to_string(),
        };
        let with_severity = format!("{}{}{}", location, padding1, severity_colored);
        let with_severity_plain = format!("{}{}{}", location_str, padding1, level);
        let padding2 = " ".repeat((38 - with_severity_plain.len()).max(0));
        let rule_name_formatted = rule_name.replace("_", "-");
        let dim_rule_name = format!("\x1B[2m({})\x1B[0m", rule_name_formatted);
        format!(
            "{}{}{}  {}\n",
            with_severity, padding2, issue.message, dim_rule_name
        )
    }

    fn format_filename(&self, filename: &str) -> String {
        format!("\x1B[4m{}\x1B[0m", filename)
    }
}

/// Create a formatter based on the output format
pub fn create_formatter(format: OutputFormat) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Standard => Box::new(StandardFormatter),
        OutputFormat::Colored => Box::new(ColoredFormatter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_formatter() {
        let formatter = StandardFormatter;
        let issue = LintIssue {
            line: 5,
            column: 10,
            message: "test message".to_string(),
            severity: Severity::Error,
        };

        let formatted = formatter.format_issue(&issue, "test-rule");
        assert!(formatted.contains("5:10"));
        assert!(formatted.contains("error"));
        assert!(formatted.contains("test message"));
        assert!(formatted.contains("test-rule"));

        let filename_formatted = formatter.format_filename("test.yaml");
        assert_eq!(filename_formatted, "test.yaml");
    }

    #[test]
    fn test_colored_formatter() {
        let formatter = ColoredFormatter;
        let issue = LintIssue {
            line: 5,
            column: 10,
            message: "test message".to_string(),
            severity: Severity::Error,
        };

        let formatted = formatter.format_issue(&issue, "test-rule");
        assert!(formatted.contains("5:10"));
        assert!(formatted.contains("error"));
        assert!(formatted.contains("test message"));
        assert!(formatted.contains("test-rule"));
        // Should contain ANSI color codes
        assert!(formatted.contains("\x1B"));

        let filename_formatted = formatter.format_filename("test.yaml");
        assert!(filename_formatted.contains("\x1B[4m"));
        assert!(filename_formatted.contains("test.yaml"));
    }

    #[test]
    fn test_create_formatter() {
        let standard = create_formatter(OutputFormat::Standard);
        assert!(standard.format_filename("test.yaml") == "test.yaml");

        let colored = create_formatter(OutputFormat::Colored);
        assert!(colored.format_filename("test.yaml").contains("\x1B"));
    }
}
