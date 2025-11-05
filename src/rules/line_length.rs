use crate::rules::base::BaseRule;
use crate::rules::Rule;
use crate::{create_issue, LintIssue, Severity};
use yaml_rust::scanner::{Scanner, Token, TokenType};

#[derive(Debug, Clone)]
pub struct LineLengthConfig {
    pub max_length: usize,
    pub allow_non_breakable_words: bool,
    pub allow_non_breakable_inline_mappings: bool,
}

impl Default for LineLengthConfig {
    fn default() -> Self {
        Self {
            max_length: 80,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineLengthRule {
    base: BaseRule<LineLengthConfig>,
}

impl LineLengthRule {
    pub fn new() -> Self {
        Self {
            base: BaseRule::new(LineLengthConfig::default()),
        }
    }

    pub fn with_config(config: LineLengthConfig) -> Self {
        Self {
            base: BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &LineLengthConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: LineLengthConfig) {
        self.base.set_config(config);
    }

    pub fn get_severity(&self) -> Severity {
        self.base.get_severity(Severity::Error)
    }

    pub fn set_severity(&mut self, severity: Severity) {
        self.base.set_severity(severity);
    }

    pub fn has_severity_override(&self) -> bool {
        self.base.has_severity_override()
    }
}

impl Default for LineLengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for LineLengthRule {
    fn rule_id(&self) -> &'static str {
        "line-length"
    }

    fn rule_name(&self) -> &'static str {
        "Line Length"
    }

    fn rule_description(&self) -> &'static str {
        "Ensures that lines do not exceed the maximum allowed length"
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
        false
    }

    fn check(&self, content: &str, file_path: &str) -> Vec<LintIssue> {
        self.check_impl(content, file_path)
    }
}

impl LineLengthRule {
    pub fn check_impl(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_length = line.len();
            if line_length > self.config().max_length {
                if self.config().allow_non_breakable_words && self.has_non_breakable_content(line) {
                    continue;
                }

                if self.config().allow_non_breakable_inline_mappings
                    && self.check_inline_mapping(line)
                {
                    continue;
                }

                issues.push(create_issue!(
                    line_num + 1,
                    self.config().max_length + 1,
                    format!(
                        "line too long ({} > {} characters)",
                        line_length,
                        self.config().max_length
                    ),
                    self.get_severity()
                ));
            }
        }

        issues
    }

    fn has_non_breakable_content(&self, line: &str) -> bool {
        let mut start = 0;
        while start < line.len() && line.chars().nth(start) == Some(' ') {
            start += 1;
        }

        if start == line.len() {
            return false;
        }

        if line.chars().nth(start) == Some('#') {
            while start < line.len() && line.chars().nth(start) == Some('#') {
                start += 1;
            }
            if start < line.len() {
                start += 1;
            }
        } else if line.chars().nth(start) == Some('-') {
            start += 2;
        }

        if start >= line.len() {
            return false;
        }

        !line[start..].contains(' ')
    }

    fn check_inline_mapping(&self, line: &str) -> bool {
        let scanner = Scanner::new(line.chars());
        let tokens: Vec<_> = scanner.collect();

        let mut found_block_mapping_start = false;
        let mut found_value = false;
        let mut scalar_column: Option<usize> = None;

        for token in &tokens {
            let Token(marker, token_type) = token;

            match token_type {
                TokenType::BlockMappingStart => {
                    found_block_mapping_start = true;
                }
                TokenType::Value => {
                    if found_block_mapping_start {
                        found_value = true;
                    }
                }
                TokenType::Scalar(_, _) => {
                    if found_block_mapping_start && found_value {
                        scalar_column = Some(marker.col());
                        break;
                    }
                }
                _ => {}
            }
        }

        if let Some(col) = scalar_column {
            let value_start = line
                .char_indices()
                .nth(col)
                .map(|(idx, _)| idx)
                .unwrap_or(line.len());

            let value_content = &line[value_start..];

            !value_content.contains(' ')
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;

    #[test]
    fn test_line_length_rule_default() {
        let rule = LineLengthRule::new();
        assert_eq!(rule.rule_id(), "line-length");
        assert_eq!(rule.config().max_length, 80);
    }

    #[test]
    fn test_line_length_rule_custom_config() {
        let config = LineLengthConfig {
            max_length: 100,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        };
        let rule = LineLengthRule::with_config(config);
        assert_eq!(rule.config().max_length, 100);
    }

    #[test]
    fn test_line_length_check_short_lines() {
        let rule = LineLengthRule::new();
        let content = "short line\nanother short line\n";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_line_length_check_long_lines() {
        let config = LineLengthConfig {
            max_length: 10,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        };
        let rule = LineLengthRule::with_config(config);
        let content = "short line\nthis is a very long line that exceeds the limit\nshort";
        let issues = rule.check(content, "test.yaml");

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].line, 2);
        assert_eq!(issues[0].column, 11);
        assert!(issues[0].message.contains("line too long"));
    }

    #[test]
    fn test_line_length_check_multiple_violations() {
        let config = LineLengthConfig {
            max_length: 5,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        };
        let rule = LineLengthRule::with_config(config);
        let content = "short\nthis is too long\nshort\nanother very long line here";
        let issues = rule.check(content, "test.yaml");

        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line, 2);
        assert_eq!(issues[0].column, 6);
        assert_eq!(issues[1].line, 4);
        assert_eq!(issues[1].column, 6);
    }

    #[test]
    fn test_line_length_allow_non_breakable_words() {
        let config = LineLengthConfig {
            max_length: 20,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        };
        let rule = LineLengthRule::with_config(config);

        // This should NOT be flagged (non-breakable word without spaces)
        let content = "http://localhost/very/very/very/very/long/url";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty(), "Non-breakable words should be allowed");
    }

    #[test]
    fn test_line_length_disallow_non_breakable_words() {
        let config = LineLengthConfig {
            max_length: 20,
            allow_non_breakable_words: false,
            allow_non_breakable_inline_mappings: false,
        };
        let rule = LineLengthRule::with_config(config);

        // This SHOULD be flagged when non-breakable words are disabled
        let content = "key: http://localhost/very/very/very/very/long/url";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("line too long"));
    }

    #[test]
    fn test_line_length_breakable_content_still_flagged() {
        let config = LineLengthConfig {
            max_length: 20,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        };
        let rule = LineLengthRule::with_config(config);

        // This SHOULD be flagged (breakable content with spaces)
        let content = "key: this is a very long line with spaces";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("line too long"));
    }

    #[test]
    fn test_line_length_comment_lines_not_affected() {
        let config = LineLengthConfig {
            max_length: 10,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        };
        let rule = LineLengthRule::with_config(config);

        // Comments should still be flagged even with non-breakable words enabled
        let content = "# this is a very long comment that exceeds the limit";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("line too long"));
    }

    #[test]
    fn test_line_length_underscore_values_still_flagged() {
        let config = LineLengthConfig {
            max_length: 20,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        };
        let rule = LineLengthRule::with_config(config);

        // Values with underscores should still be flagged (not truly non-breakable)
        let content = "key: this_is_a_very_long_line_with_underscores";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("line too long"));
    }

    #[test]
    fn test_line_length_false_positive_bug() {
        let rule = LineLengthRule::new();
        let content = "this_is_a_very_long_line_that_exceeds_the_maximum_length_limit_and_should_be_detected_as_a_violation_but_the_original_yamllint_might_not_detect_it_due_to_different_parsing_logic";

        let issues = rule.check(content, "test.yaml");

        assert!(
            issues.is_empty(),
            "No line length violations should be detected. Current false positives: {:?}",
            issues
        );
    }

    #[test]
    fn test_line_length_allow_inline_mappings() {
        // Test that allow-non-breakable-inline-mappings allows inline mappings
        // with non-breakable values (long URLs, IDs, etc.)
        // yamllint reports 0 line-length issues for this content
        // yamllint-rs should also report 0 (after fix)
        let mut config = LineLengthConfig::default();
        config.max_length = 80;
        config.allow_non_breakable_words = true;
        config.allow_non_breakable_inline_mappings = true;

        let rule = LineLengthRule::with_config(config);

        // Inline mapping with long URL (no spaces in value) - should be allowed
        let content = r#"short: line
link: '{client_base_url}/#/pendingActions?phase=considered&queryId=CrmmH9CvmiiTbtgfQb6tR0uI4hNio5m1EDsxYSIzr3AKTDmQxXRYbhu31IfGpkxvR%252FSBCFlgVbOZ1oW3Z7ylqt5BRxKFGImiZ5sranhW4yM8QVUTyErDLFlq1gixrudftgo97yI81t%252BUup7kRenZ39DlzLHkZFMSQfgFETHtejSon4hLDHcu%252BVXSr3sTApj0I6Mss8%252BudX2Uf4ZdnaDOqwLc05gu09dzT%252FeDo8BOnmntafigB64salJPpVxW4k7iwhwvvofDY4DnuKstbZztfK60UFioI7Yf2lrlEO%252FZGeUZSM%252BOUbI0egEzomnwbw0CPIqC0VMuiLIlW9udXFKd7aMwWKYQVSnXRqaBbdkxspNkiVles2bum7Kwk0gY6BxbFuG%252FcndibiZyeZ5i2o8MrZ4wTD4jMFmc5OOYZu6DPHxMIIqbWaxK%252FdkYTYb57thvutjT9T9%252BceggbcKzUjv2gXa9Clg4IK9dkYzSr5d1RDZsoyrgnPCbfFTomn6%252BjRiIs173VSLyzznB8lNpfEXAwwZ0aDeA2qcjd4ICxzt8iOkr%252BdxHAn58rz2yLriV1XF7EhL7tefmBxYZAtFS3VXG16x9rOwIueozK1zwq9iwwv%252BztMVwQ7AV%252FfLJDqG4W%252B2iU%252ByEyvbVBOUMyKQ66gaeSzam3ByHu7jpYK8KebT%252Fi6B25W4PB4ZHOOGe5CD5s04ACwVT8pt%252Bw84l3SlU8Ragjuc28%252B6oOkhxR01yzfdTNuo%252BovD3LImhEb7FXLE2FdiUdAT0Ye1r29pS5MxIa2oY6emmEsvf0wMY0WlAfDfD1%252FDX5ZKA34rNu6GLJ8bbkfJxj0aSJKZWjqOl8Ud%252FSGN8KzyCG4twem%252FBVIwSxcZpqfLZ3qnWrmgvpVDoZyOjmsXJ2WLqYm57HQjMDsdIaK3mekhFoVzD68F7iw7JMCpv%252FaRqYw2iyKB%252Bu2cOGEN1bHe7go7293kx6sjvJmDrXQoTP6AMGToHc4Cw2N%252FXs8MKKu6GFtv4U3k0dW4UGM02iu67rFn0ZHz4aty92friH6HUntBe5V2PHxczEEcEGgiKITWLXIpw1bcPHvoFB4NJlzlK9ULF%252BcfUukIkk%252Fbwl3%252B1dMVbsuwL6CVO6vGWZsWs5dMy3FeJjpcVUBIfiumwtWxQGLNpM9bGWu8ZErFZFKc6c1AgsjD3VcgFJQUHTHvTwvqdcPGDYnfrWEKzOMMG8%252FLtmVtKxdeYVtcIS5aoUTcbBhV9jKj4HDnDApCX%252F%252Fi25Uf21yD%252B%252BsCu5N4NIdxH%252Fhq7cI1kAiZ0SHxxLrJ%252BSMN8Aa7pUCa82Hi6rGrjsuhV0ZD0PC6Iis1QcSciKhBf2vahMbRDjku1cYJXmeySQZXh%252FuR1tUk%252FniSfDwc0MoWCF1dNTRINCbBLioImw9o5g4ALLKlfACUK4mVSNNJerIYYh4vK5PCsk1KEG0%252B%252F7wejbjfhbpo3lNPUKY6z22NcI98gSWG%252F9dlDAotryg6kjmBYBrc65KFWq4Izx2gOtlynQ5dkGPPXYgQed4Pz1P48PjEdxLbyAvOzgO7BjoDYL9E3rYhVWhdFEns%252FvdCeVZ55nMHqDDD1z%252FIKSff4dDpcz4Rs6W3TWDxq5gBoewRcW0edoebsF5jDOfnUzFr0eOvhLStZ9mhdRFO2da%252FfROzqWwY6NYV%252FqWGyoKoV%252BwDhS%252FY7oxvXcg5%252Fw270JWKz%252F4pVBKulMe7K1EyBy2hCR5Q4%252FgqdKx%252FEaMJPlcerecp0AYyeHX0Y923hbxw3Lm0ZG3O21FS%252BGEOaOGTyodDdBhu47yhtsjFFdcbBNlpL00oS%252B6zfx5nSnRSG4TL5dVt90mWINOLDcTuyLNa8Rm0l7mGVakTbe85VfTxAshTofV64dLstf9RmijwUuw1bnNgCtMygh8k%253D'
normal: short line
"#;

        let issues = rule.check(content, "test.yaml");

        // Filter for line-length issues
        let length_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("line too long"))
            .collect();

        if !length_issues.is_empty() {
            eprintln!("\n=== LINE LENGTH ISSUES ON INLINE MAPPINGS DETECTED ===");
            eprintln!(
                "yamllint reports: 0 issues (allows inline mappings with non-breakable values)"
            );
            eprintln!(
                "yamllint-rs reports: {} line-length issues",
                length_issues.len()
            );
            for issue in &length_issues {
                eprintln!(
                    "  Line {}: {} - {}",
                    issue.line, issue.column, issue.message
                );
            }
        }

        // yamllint reports 0 issues - inline mappings with non-breakable values are allowed
        // This test should FAIL until we implement allow-non-breakable-inline-mappings
        assert_eq!(length_issues.len(), 0,
            "Found {} line-length issues in inline mappings. yamllint reports 0 issues (allows with allow-non-breakable-inline-mappings). Issues: {:?}",
            length_issues.len(), length_issues);
    }
}
