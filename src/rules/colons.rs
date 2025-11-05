use crate::{LintIssue, Severity};
use yaml_rust::scanner::{Scanner, Token, TokenType};

#[derive(Debug, Clone)]
pub struct ColonsConfig {
    pub max_spaces_before: i32,
    pub max_spaces_after: i32,
}

impl Default for ColonsConfig {
    fn default() -> Self {
        Self {
            max_spaces_before: 0,
            max_spaces_after: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColonsRule {
    base: crate::rules::base::BaseRule<ColonsConfig>,
}

impl ColonsRule {
    pub fn new() -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(ColonsConfig::default()),
        }
    }

    pub fn with_config(config: ColonsConfig) -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &ColonsConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: ColonsConfig) {
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
}

impl Default for ColonsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::rules::Rule for ColonsRule {
    fn rule_id(&self) -> &'static str {
        "colons"
    }

    fn rule_name(&self) -> &'static str {
        "Colons"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper spacing around colons."
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

    fn check_with_analysis(
        &self,
        content: &str,
        _file_path: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        self.check_impl_with_analysis(content, analysis)
    }
}

impl ColonsRule {
    fn check_with_tokens(
        &self,
        content: &str,
        tokens: &[Token],
        token_analysis: &crate::analysis::TokenAnalysis,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (i, token) in tokens.iter().enumerate() {
            let Token(marker, token_type) = token;
            let flow_depth = token_analysis.get_flow_depth(i);

            match token_type {
                TokenType::Value => {
                    if flow_depth > 0 {
                        continue;
                    }

                    let mut prev_idx = i.saturating_sub(1);
                    let mut prev_token = None;

                    while let Some(prev_token_val) = tokens.get(prev_idx) {
                        let Token(prev_marker, prev_token_type) = prev_token_val;
                        match prev_token_type {
                            TokenType::Scalar(_, _) => {
                                if prev_marker.line() == marker.line() {
                                    prev_token = Some(prev_token_val);
                                    break;
                                }
                                prev_idx = prev_idx.saturating_sub(1);
                            }
                            TokenType::Key
                            | TokenType::BlockMappingStart
                            | TokenType::BlockSequenceStart
                            | TokenType::FlowMappingStart
                            | TokenType::FlowSequenceStart
                            | TokenType::BlockEnd
                            | TokenType::FlowMappingEnd
                            | TokenType::FlowSequenceEnd
                            | TokenType::Value => {
                                prev_idx = prev_idx.saturating_sub(1);
                            }
                            _ => {
                                break;
                            }
                        }
                    }

                    if let Some(prev_token) = prev_token {
                        let Token(prev_marker, prev_token_type) = prev_token;

                        let is_quoted_key = if let TokenType::Scalar(_, _) = prev_token_type {
                            let prev_start = prev_marker.index();
                            if prev_start < content.len() {
                                if let Some(first_char) = content.chars().nth(prev_start) {
                                    first_char == '"' || first_char == '\''
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_quoted_key {
                            continue;
                        }

                        if !self.is_alias_value(prev_token_type, prev_marker, marker) {
                            if self.config().max_spaces_before >= 0 {
                                if let Some(_) = self.spaces_before(
                                    marker,
                                    prev_marker,
                                    prev_token_type,
                                    content,
                                    self.config().max_spaces_before as usize,
                                ) {
                                    issues.push(LintIssue {
                                        line: marker.line() + 1,
                                        column: marker.col() + 1,
                                        message: "too many spaces before colon".to_string(),
                                        severity: self.get_severity(),
                                    });
                                }
                            }

                            if self.config().max_spaces_after >= 0 {
                                if let Some(next_token) = tokens.get(i + 1) {
                                    let Token(next_marker, _) = next_token;
                                    if let Some(_) = self.spaces_after(
                                        marker,
                                        next_marker,
                                        content,
                                        self.config().max_spaces_after as usize,
                                    ) {
                                        issues.push(LintIssue {
                                            line: marker.line() + 1,
                                            column: marker.col() + 1,
                                            message: "too many spaces after colon".to_string(),
                                            severity: self.get_severity(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                TokenType::Key => {
                    if self.is_explicit_key(marker, content) {
                        if self.config().max_spaces_after >= 0 {
                            if let Some(next_token) = tokens.get(i + 1) {
                                let Token(next_marker, _) = next_token;
                                if let Some(_) = self.spaces_after(
                                    marker,
                                    next_marker,
                                    content,
                                    self.config().max_spaces_after as usize,
                                ) {
                                    issues.push(LintIssue {
                                        line: marker.line() + 1,
                                        column: marker.col() + 1,
                                        message: "too many spaces after question mark".to_string(),
                                        severity: self.get_severity(),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        issues
    }

    pub fn check_impl(&self, content: &str, _file_path: &str) -> Vec<LintIssue> {
        let scanner = Scanner::new(content.chars());
        let tokens: Vec<_> = scanner.collect();
        let token_analysis = crate::analysis::TokenAnalysis::analyze(content);
        self.check_with_tokens(content, &tokens, &token_analysis)
    }

    pub fn check_impl_with_analysis(
        &self,
        content: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        if let Some(token_analysis) = analysis.tokens() {
            self.check_with_tokens(content, &token_analysis.tokens, token_analysis)
        } else {
            self.check_impl(content, "")
        }
    }

    fn is_alias_value(
        &self,
        prev_token_type: &TokenType,
        prev_marker: &yaml_rust::scanner::Marker,
        marker: &yaml_rust::scanner::Marker,
    ) -> bool {
        matches!(prev_token_type, TokenType::Alias(_)) && marker.index() - prev_marker.index() == 1
    }

    fn is_explicit_key(&self, marker: &yaml_rust::scanner::Marker, content: &str) -> bool {
        marker.index() < content.len()
            && content
                .chars()
                .nth(marker.index())
                .map_or(false, |c| c == '?')
    }

    fn spaces_before(
        &self,
        token_marker: &yaml_rust::scanner::Marker,
        prev_marker: &yaml_rust::scanner::Marker,
        prev_token_type: &TokenType,
        content: &str,
        max_spaces: usize,
    ) -> Option<()> {
        if prev_marker.line() != token_marker.line() {
            return None;
        }

        let prev_start = prev_marker.index();
        let token_start = token_marker.index();

        if token_start <= prev_start {
            return None;
        }

        let spaces = if let TokenType::Scalar(_, scalar_value) = prev_token_type {
            let prev_end = if let Some(first_char) = content.chars().nth(prev_start) {
                if first_char == '"' || first_char == '\'' {
                    let quote_char = first_char;
                    let bytes = content.as_bytes();
                    let expected_end_min = prev_start + scalar_value.as_bytes().len();
                    let mut prev_end = prev_start + scalar_value.as_bytes().len() + 2;

                    let mut pos = expected_end_min.min(bytes.len().saturating_sub(1));
                    while pos < bytes.len() {
                        if bytes[pos] == quote_char as u8 {
                            let mut backslash_count = 0;
                            let mut check_pos = pos;
                            while check_pos > prev_start && bytes[check_pos - 1] == b'\\' {
                                backslash_count += 1;
                                check_pos -= 1;
                            }

                            if backslash_count % 2 == 0 {
                                prev_end = pos + 1;
                                break;
                            }
                        }
                        pos += 1;
                        if pos > prev_start + scalar_value.as_bytes().len() + 10 {
                            break;
                        }
                    }

                    prev_end
                } else {
                    prev_start + scalar_value.as_bytes().len()
                }
            } else {
                prev_start + scalar_value.as_bytes().len()
            };

            if token_start <= prev_end {
                return None;
            }

            if let Some(between_text) = content.get(prev_end..token_start) {
                if between_text.bytes().any(|b| b == b'\n') {
                    return None;
                }
                if between_text.is_empty() {
                    return None;
                }
            }

            let spacing = token_start.saturating_sub(prev_end);

            if spacing == 0 {
                return None;
            }

            spacing
        } else {
            if let Some(between_text) = content.get(prev_start..token_start) {
                if between_text.bytes().any(|b| b == b'\n') {
                    return None; // Tokens are on different lines, skip spacing check
                }

                // Work backwards from the end (colon position) to find where spaces start
                let mut trailing_spaces = 0;
                for byte in between_text.bytes().rev() {
                    if byte == b' ' {
                        trailing_spaces += 1;
                    } else {
                        break;
                    }
                }
                trailing_spaces
            } else {
                return None;
            }
        };

        if spaces > max_spaces {
            return Some(());
        }

        None
    }

    fn spaces_after(
        &self,
        token_marker: &yaml_rust::scanner::Marker,
        next_marker: &yaml_rust::scanner::Marker,
        content: &str,
        max_spaces: usize,
    ) -> Option<()> {
        if token_marker.line() != next_marker.line() {
            return None;
        }

        let token_end = token_marker.index() + 1;
        let next_start = next_marker.index();

        if next_start <= token_end {
            return None;
        }

        let spacing = next_start - token_end;

        if spacing > max_spaces {
            if let Some(between_text) = content.get(token_end..next_start) {
                let space_count = between_text.bytes().filter(|&b| b == b' ').count();
                if space_count > max_spaces {
                    return Some(());
                }
            }
        }

        None
    }

    pub fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        super::FixResult {
            content: content.to_string(),
            changed: false,
            fixes_applied: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;

    #[test]
    fn test_colons_rule_default() {
        let rule = ColonsRule::new();
        assert_eq!(rule.rule_id(), "colons");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_colons_check_clean_colons() {
        let rule = ColonsRule::new();
        let content = "key: value\nanother: test";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_colons_check_spaces_before() {
        let rule = ColonsRule::new();
        let content = "key : value\nanother : test";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("too many spaces before colon"));
        assert!(issues[1].message.contains("too many spaces before colon"));
    }

    #[test]
    fn test_colons_check_spaces_after() {
        let rule = ColonsRule::new();
        let content = "key:  value\nanother:  test";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("too many spaces after colon"));
        assert!(issues[1].message.contains("too many spaces after colon"));
    }

    #[test]
    fn test_colons_fix() {
        let rule = ColonsRule::new();
        let content = "key : value\nanother : test";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }

    #[test]
    fn test_colons_fix_no_changes() {
        let rule = ColonsRule::new();
        let content = "key: value\nanother: test";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }

    #[test]
    fn test_colons_should_ignore_colons_in_comments() {
        let rule = ColonsRule::new();
        let content = r#"key: value
# This is a comment with : colon
another: test"#;
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_colons_with_unicode_characters() {
        let rule = ColonsRule::new();
        let content = "key: value 态 with chinese\nanother: test é with accent";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_colons_with_unicode_content_should_work() {
        let rule = ColonsRule::new();

        let content =
            "key: value 态 with chinese\nanother: test é with accent\nfinal: test – with em-dash";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty(), "Unicode characters should work");

        let spacing_unicode = "key : value 态 with chinese\nanother: test é with accent";
        let spacing_issues = rule.check(spacing_unicode, "test.yaml");
        assert!(
            !spacing_issues.is_empty(),
            "Spacing issues with Unicode should be detected"
        );
    }

    #[test]
    fn test_colons_unicode_edge_cases() {
        let rule = ColonsRule::new();
        let content =
            "key: value 态 with 中文\nanother: test é with français\nfinal: test – with em-dash";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty(), "Complex Unicode sequences should work");

        let spacing_unicode = "key : value 态 with chinese\nanother: test é with accent";
        let spacing_issues = rule.check(spacing_unicode, "test.yaml");
        assert!(
            !spacing_issues.is_empty(),
            "Spacing issues with Unicode should be detected"
        );
    }

    #[test]
    fn test_debug_spacing_calculation() {
        let content = "key: value";
        let scanner = Scanner::new(content.chars());
        let tokens: Vec<_> = scanner.collect();

        println!("Content: '{}'", content);

        for (i, token) in tokens.iter().enumerate() {
            let Token(marker, token_type) = token;
            println!(
                "Token {}: {:?} at line {}, col {}, index {}",
                i,
                token_type,
                marker.line(),
                marker.col(),
                marker.index()
            );
        }

        let rule = ColonsRule::new();
        let issues = rule.check(content, "test.yaml");
        println!("Issues: {:?}", issues);
    }

    #[test]
    fn test_debug_spacing_with_violations() {
        let content = r#"key : value
another:  test
indented:
  subkey : subvalue"#;
        let scanner = Scanner::new(content.chars());
        let tokens: Vec<_> = scanner.collect();

        println!("Content: '{}'", content);

        for (i, token) in tokens.iter().enumerate() {
            let Token(marker, token_type) = token;
            println!(
                "Token {}: {:?} at line {}, col {}, index {}",
                i,
                token_type,
                marker.line(),
                marker.col(),
                marker.index()
            );
        }

        let rule = ColonsRule::new();
        let issues = rule.check(content, "test.yaml");
        println!("Issues: {:?}", issues);
    }

    #[test]
    fn test_debug_indented_case() {
        let content = r#"supported_countries:
  United States:
    code: US"#;
        let scanner = Scanner::new(content.chars());
        let tokens: Vec<_> = scanner.collect();

        println!("Content: '{}'", content);

        for (i, token) in tokens.iter().enumerate() {
            let Token(marker, token_type) = token;
            println!(
                "Token {}: {:?} at line {}, col {}, index {}",
                i,
                token_type,
                marker.line(),
                marker.col(),
                marker.index()
            );
        }

        let rule = ColonsRule::new();
        let issues = rule.check(content, "test.yaml");
        println!("Issues: {:?}", issues);
    }

    #[test]
    fn test_colons_multiple_keys_no_violations() {
        let rule = ColonsRule::new();
        let content = r#"some_key: value
another_key: another_value
third_key: third_value
fourth_key: fourth_value"#;

        let issues = rule.check(content, "test.yaml");

        assert!(
            issues.is_empty(),
            "No spacing violations should be detected. Current false positives: {:?}",
            issues
        );
    }

    #[test]
    fn test_colons_indented_keys_should_not_trigger_false_positive() {
        let rule = ColonsRule::new();
        let content = r#"supported_countries:
  United States:
    code: US
    is_enabled: true
    is_gradable: true"#;

        let issues = rule.check(content, "test.yaml");

        assert!(
            issues.is_empty(),
            "Indented keys should not trigger colons violations. Current false positives: {:?}",
            issues
        );
    }

    #[test]
    fn test_colons_actual_spacing_violations() {
        let rule = ColonsRule::new();
        let content = r#"key : value
another:  test
indented:
  subkey : subvalue"#;

        let issues = rule.check(content, "test.yaml");

        // Should detect spaces before colon in "key : value" and "subkey : subvalue"
        // Should detect spaces after colon in "another:  test"
        assert_eq!(
            issues.len(),
            3,
            "Should detect 3 spacing violations: {:?}",
            issues
        );

        let before_colon_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("before colon"))
            .collect();
        let after_colon_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("after colon"))
            .collect();

        assert_eq!(
            before_colon_issues.len(),
            2,
            "Should detect 2 'before colon' violations"
        );
        assert_eq!(
            after_colon_issues.len(),
            1,
            "Should detect 1 'after colon' violation"
        );
    }

    #[test]
    fn test_colons_no_false_positives_for_immediate_colons() {
        let rule = ColonsRule::new();
        let content = r#"    integration_id: '110960'
    title: Some title
    type: text
  - id: Question text
    integration_id: '127519'
    title: Another title
    type: text
  form_name: Form Name
  form_title: Form Title
  integration_id: '4993'"#;

        let issues = rule.check(content, "test.yaml");
        let before_colon_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("before colon"))
            .collect();

        assert!(
            before_colon_issues.is_empty(),
            "No 'before colon' issues should be reported for valid YAML. Found: {:?}",
            before_colon_issues
        );
    }

    #[test]
    fn test_colons_ensure_char_boundary_doesnt_cause_false_positives() {
        let rule = ColonsRule::new();
        let content = r#"key: value
another: test
indented_key: value
deeply_nested:
  nested_key: value"#;

        let issues = rule.check(content, "test.yaml");
        let before_colon_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("before colon"))
            .collect();

        assert!(
            before_colon_issues.is_empty(),
            "No false positives when colons immediately follow scalars. Found: {:?}",
            before_colon_issues
        );
    }

    #[test]
    fn test_colons_false_positive_with_full_context() {
        // This test reproduces the false positive by using the exact content from utah.yaml
        // lines 1-150, which includes the context needed to trigger the issue at line 147
        // yamllint reports 0 colons issues for this content
        use std::fs;

        let rule = ColonsRule::new();
        let test_file = "tests/test_colons_false_positive_input.yaml";

        // Load the actual problematic content
        let content = match fs::read_to_string(test_file) {
            Ok(content) => content,
            Err(_) => {
                // If file doesn't exist, skip this test
                eprintln!("Skipping test - test file not found: {}", test_file);
                return;
            }
        };

        let issues = rule.check(&content, test_file);

        // Filter for "before colon" issues on line 147
        let line_147_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.line == 147 && issue.message.contains("before colon"))
            .collect();

        // Print debug info
        if !line_147_issues.is_empty() {
            eprintln!("\n=== FALSE POSITIVE REPRODUCED ===");
            eprintln!("Line 147 content: 'shortcut_types:'");
            eprintln!("yamllint reports: 0 issues");
            eprintln!(
                "yamllint-rs reports: {} false positives",
                line_147_issues.len()
            );
            for issue in &line_147_issues {
                eprintln!("  {:?}", issue);
            }
            eprintln!("\nRoot cause:");
            eprintln!("  yamllint uses: prev.end_mark.pointer (END position)");
            eprintln!("  yamllint-rs uses: prev_marker.index() (START position)");
            eprintln!("  This causes spacing calculation to include token content");
        }

        // This test FAILS - it documents the false positive issue
        assert_eq!(
            line_147_issues.len(),
            0,
            "Line 147 has {} false positives. yamllint reports 0 issues. Issues: {:?}",
            line_147_issues.len(),
            line_147_issues
        );
    }

    #[test]
    fn test_colons_false_positive_test_colon_file() {
        // This test uses the exact test_colon.yml file that shows false positives
        // yamllint reports 0 colons issues for this file
        // but yamllint-rs currently reports false positives on lines 60 and 69
        use std::fs;

        let rule = ColonsRule::new();
        let test_file = "tests/test_colon_false_positives.yaml";

        // Load the problematic content
        let content = match fs::read_to_string(test_file) {
            Ok(content) => content,
            Err(_) => {
                eprintln!("Skipping test - test file not found: {}", test_file);
                return;
            }
        };

        let issues = rule.check(&content, test_file);

        // Filter for "before colon" issues - these are all false positives
        let false_positives: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("before colon"))
            .collect();

        // Print debug info if we find false positives
        if !false_positives.is_empty() {
            eprintln!("\n=== FALSE POSITIVES DETECTED ===");
            eprintln!("yamllint reports: 0 issues");
            eprintln!(
                "yamllint-rs reports: {} false positives",
                false_positives.len()
            );
            for issue in &false_positives {
                eprintln!(
                    "  Line {}: {} - {}",
                    issue.line, issue.column, issue.message
                );
            }
        }

        // This test FAILS - yamllint reports 0 issues for this file
        // It documents false positives on lines 60 and 69
        assert_eq!(
            false_positives.len(),
            0,
            "Found {} false positives. yamllint reports 0 issues. False positives: {:?}",
            false_positives.len(),
            false_positives
        );
    }

    #[test]
    fn test_colons_test_colon_simple3_file() {
        // Test case for spaces_after bug fix
        // yamllint reports 0 colons issues for this content
        // yamllint-rs previously reported false positives for spaces_after
        let rule = ColonsRule::new();

        let content = r#"key: value
another: test
nested:
  subkey: subvalue
  with: spaces
list:
  - item: one
  - item: two"#;

        let issues = rule.check(content, "test.yaml");

        // Filter for colons issues - these should all be false positives
        let colons_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("colon"))
            .collect();

        // yamllint reports 0 issues for this content
        assert_eq!(
            colons_issues.len(),
            0,
            "Found {} colons issues. yamllint reports 0 issues. Issues: {:?}",
            colons_issues.len(),
            colons_issues
        );
    }

    #[test]
    fn test_colons_flow_mapping_false_positive() {
        // Test case for flow mappings with quoted keys: "key": value
        // yamllint reports 0 colons issues for this content (correct behavior)
        // Spaces between quoted keys and colons are valid in flow mappings
        let rule = ColonsRule::new();

        let content = r#"test:
  mapping: {"title": "value", "another": "test"}
  nested: {"key": "val", "spaced": "item"}
"#;

        let issues = rule.check(content, "test.yaml");

        // Filter for colons issues - these should all be false positives
        let colons_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("colon"))
            .collect();

        // yamllint reports 0 issues for flow mappings like "key": value
        assert_eq!(
            colons_issues.len(),
            0,
            "Found {} colons issues in flow mapping. yamllint reports 0 issues. Issues: {:?}",
            colons_issues.len(),
            colons_issues
        );
    }

    #[test]
    fn test_colons_single_quoted_flow_mapping_false_positive() {
        // Test case for single-quoted flow mappings: {'key': value}
        // yamllint reports 0 colons issues for this content
        // yamllint-rs currently reports false positives due to single quotes
        let rule = ColonsRule::new();

        // Single-quoted flow mapping pattern from actual files
        let content = r#"performance_ranges: [{'min': 1, 'max': 5}, {'min': 50, 'max': 150}]"#;

        let issues = rule.check(content, "test.yaml");

        // Filter for colons issues - these should all be false positives
        let colons_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("colon"))
            .collect();

        // Print debug info if we find issues
        if !colons_issues.is_empty() {
            eprintln!("\n=== SINGLE-QUOTED FLOW MAPPING FALSE POSITIVES DETECTED ===");
            eprintln!("yamllint reports: 0 issues");
            eprintln!("yamllint-rs reports: {} colons issues", colons_issues.len());
            for issue in &colons_issues {
                eprintln!(
                    "  Line {}: {} - {}",
                    issue.line, issue.column, issue.message
                );
            }
        }

        // yamllint reports 0 issues for single-quoted flow mappings
        assert_eq!(colons_issues.len(), 0,
            "Found {} colons issues in single-quoted flow mapping. yamllint reports 0 issues. Issues: {:?}",
            colons_issues.len(), colons_issues);
    }

    #[test]
    fn test_colons_escape_sequences_in_quoted_keys() {
        // Test case for quoted keys containing escape sequences
        // yamllint reports 0 colons issues for this content
        // yamllint-rs previously reported false positives due to escape sequences
        // Issue: scalar_value.len() returns parsed length, but escape sequences
        // take more characters in source than in parsed form
        //
        // Edge cases covered:
        // 1. Hex escape sequences: \xF4, \xE7
        // 2. Standard escapes: \n, \t, \r
        // 3. Escaped quotes inside: \", \'
        // 4. Escaped backslashes: \\
        // 5. Complex combinations: \\\"
        // 6. Single quotes with escapes
        // 7. Empty quoted strings
        let rule = ColonsRule::new();

        let content = r#"countries:
  "C\xF4te d'Ivoire":
    code: CI
  "Cura\xE7ao":
    code: CW
  "Test\nEscaped":
    value: 123
  "Tab\tHere":
    value: 456
  "Carriage\rReturn":
    value: 789
  "He said \"Hello\"":
    response: "world"
  "Escaped\"Quote\"Inside":
    value: true
  "Path\\to\\file":
    exists: false
  "Complex\\\"String":
    valid: true
  'It\'s working':
    status: ok
  'Single\nLine':
    type: string
  "Simple": "no issue"
  "":
    empty: true
"#;

        let issues = rule.check(content, "test.yaml");

        // Filter for colons issues - these should all be false positives
        let colons_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("colon"))
            .collect();

        // yamllint reports 0 issues for quoted keys with escape sequences
        assert_eq!(colons_issues.len(), 0,
            "Found {} colons issues in quoted keys with escape sequences. yamllint reports 0 issues. Issues: {:?}",
            colons_issues.len(), colons_issues);
    }

    #[test]
    fn test_colons_skip_spacing_in_flow_mappings() {
        // Test that spacing checks are skipped for colons inside flow mappings
        // yamllint reports 0 colons issues for this content
        // yamllint-rs currently reports false positives (should be 0 after fix)
        let rule = ColonsRule::new();

        // Exact pattern from account_settings files that causes false positives
        // This line from account_settings/all_environments/utah/argentina/config.yml:50
        // causes 4 false positives: columns 34, 44, 56, 67
        let content = r#"    internal_fetch:
      is_internal_active: true
      min_performance_rate: 0.7
      performance_ranges: [{'min': 1, 'max': 5}, {'min': 50, 'max': 150}]
      max_distance: 999999999
"#;

        let issues = rule.check(content, "test.yaml");

        // Filter for colons issues - currently fails with false positives
        let colons_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("colon"))
            .collect();

        if !colons_issues.is_empty() {
            eprintln!("\n=== FLOW MAPPING SPACING FALSE POSITIVES DETECTED ===");
            eprintln!("yamllint reports: 0 issues");
            eprintln!("yamllint-rs reports: {} colons issues", colons_issues.len());
            for issue in &colons_issues {
                eprintln!(
                    "  Line {}: {} - {}",
                    issue.line, issue.column, issue.message
                );
            }
        }

        // yamllint reports 0 issues - it doesn't check spacing in flow mappings
        assert_eq!(colons_issues.len(), 0,
            "Found {} colons issues in flow mappings. yamllint reports 0 issues (doesn't check flow mappings). Issues: {:?}",
            colons_issues.len(), colons_issues);
    }

    #[test]
    fn test_colons_skip_spacing_in_flow_sequences() {
        // Test that spacing checks are skipped for colons inside flow sequences
        // Flow sequences can contain mappings with colons
        let rule = ColonsRule::new();

        let content = r#"test:
  # Flow sequence containing mappings
  items: [{name: item1, value: 1}, {name: item2, value: 2}]
  
  # Flow sequence with quoted keys
  quoted: [{"key": "val"}, {"other": "test"}]
"#;

        let issues = rule.check(content, "test.yaml");

        // Filter for colons issues - should be 0
        let colons_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message.contains("colon"))
            .collect();

        if !colons_issues.is_empty() {
            eprintln!("\n=== FLOW SEQUENCE SPACING FALSE POSITIVES DETECTED ===");
            eprintln!("yamllint reports: 0 issues");
            eprintln!("yamllint-rs reports: {} colons issues", colons_issues.len());
            for issue in &colons_issues {
                eprintln!(
                    "  Line {}: {} - {}",
                    issue.line, issue.column, issue.message
                );
            }
        }

        assert_eq!(
            colons_issues.len(),
            0,
            "Found {} colons issues in flow sequences. yamllint reports 0 issues. Issues: {:?}",
            colons_issues.len(),
            colons_issues
        );
    }
}
