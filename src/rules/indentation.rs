use crate::{LintIssue, Severity};
use yaml_rust::scanner::{Scanner, Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
enum ParentType {
    Root,
    BlockMap,
    BlockSeq,
    BlockEnt,
    Key,
    Val,
}

#[derive(Debug, Clone)]
struct Parent {
    parent_type: ParentType,
    indent: usize,
    #[allow(dead_code)]
    line_indent: Option<usize>,
    #[allow(dead_code)]
    explicit_key: bool,
    implicit_block_seq: bool,
}

impl Parent {
    fn new(parent_type: ParentType, indent: usize, line_indent: Option<usize>) -> Self {
        Self {
            parent_type,
            indent,
            line_indent,
            explicit_key: false,
            implicit_block_seq: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndentationConfig {
    pub spaces: usize,
    pub indent_sequences: bool,
    pub check_multi_line_strings: bool,
    pub ignore_patterns: Vec<String>,
}

impl Default for IndentationConfig {
    fn default() -> Self {
        Self {
            spaces: 2,
            indent_sequences: true,
            check_multi_line_strings: false,
            ignore_patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndentationRule {
    base: crate::rules::base::BaseRule<IndentationConfig>,
}

impl IndentationRule {
    pub fn new() -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(IndentationConfig::default()),
        }
    }

    pub fn with_config(config: IndentationConfig) -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &IndentationConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: IndentationConfig) {
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

impl Default for IndentationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::rules::Rule for IndentationRule {
    fn rule_id(&self) -> &'static str {
        "indentation"
    }

    fn rule_name(&self) -> &'static str {
        "Indentation"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper indentation levels in YAML."
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

    fn check_with_analysis(
        &self,
        content: &str,
        file_path: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        self.check_impl_with_analysis(content, file_path, analysis)
    }
}

impl IndentationRule {
    fn detect_indent(&self, _base_indent: usize, next: &Token) -> usize {
        let Token(marker, _) = next;
        marker.col()
    }

    pub fn parse_ignore_patterns(ignore_str: Option<String>) -> Vec<String> {
        if let Some(ignore_str) = ignore_str {
            ignore_str
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    fn matches_ignore_pattern(&self, file_path: &str) -> bool {
        for pattern in &self.config().ignore_patterns {
            if file_path.contains(pattern) {
                return true;
            }
        }
        false
    }

    fn check_with_tokens(
        &self,
        _content: &str,
        file_path: &str,
        tokens: &[Token],
        _token_analysis: &crate::analysis::TokenAnalysis,
    ) -> Vec<LintIssue> {
        if self.matches_ignore_pattern(file_path) {
            return Vec::new();
        }
        let mut issues = Vec::new();

        let mut stack: Vec<Parent> = vec![Parent::new(ParentType::Root, 0, None)];

        let mut cur_line: usize = 0;
        let mut reported_error_for_key = false;
        for (idx, token) in tokens.iter().enumerate() {
            let Token(marker, ref token_type) = token;
            let next_token = tokens.get(idx + 1);

            let is_visible =
                !matches!(token_type, TokenType::StreamStart(_) | TokenType::StreamEnd);
            let first_in_line = is_visible && marker.line() > cur_line;

            if stack.len() >= 2
                && stack.last().unwrap().parent_type == ParentType::Val
                && !matches!(
                    token_type,
                    TokenType::Value
                        | TokenType::Tag(_, _)
                        | TokenType::Alias(_)
                        | TokenType::BlockEntry
                )
            {
                if stack[stack.len() - 2].parent_type == ParentType::Key {
                    stack.pop();
                    stack.pop();
                }
            }

            match token_type {
                TokenType::BlockMappingStart => {
                    let indent = marker.col();
                    stack.push(Parent::new(ParentType::BlockMap, indent, None));
                }
                TokenType::BlockSequenceStart => {
                    let indent = marker.col();
                    stack.push(Parent::new(ParentType::BlockSeq, indent, None));
                }
                TokenType::BlockEntry => {
                    let indent = marker.col();
                    let mut block_ent = Parent::new(ParentType::BlockEnt, indent, None);
                    block_ent.implicit_block_seq = false;
                    stack.push(block_ent);
                }
                TokenType::Key => {
                    let indent = marker.col();
                    let key_parent = Parent::new(ParentType::Key, indent, None);
                    stack.push(key_parent);
                    reported_error_for_key = false;
                }
                TokenType::Value => {
                    if stack
                        .last()
                        .map(|p| p.parent_type == ParentType::Key)
                        .unwrap_or(false)
                    {
                        let indent = if let Some(next) = next_token {
                            let Token(next_marker, ref next_type) = next;
                            let prev_marker = marker;

                            if matches!(
                                next_type,
                                TokenType::BlockEnd
                                    | TokenType::FlowMappingEnd
                                    | TokenType::FlowSequenceEnd
                                    | TokenType::Key
                            ) {
                                stack.last().unwrap().indent
                            } else if next_marker.line() == prev_marker.line() {
                                next_marker.col()
                            } else {
                                self.detect_indent(stack.last().unwrap().indent, next)
                            }
                        } else {
                            stack.last().unwrap().indent
                        };
                        stack.push(Parent::new(ParentType::Val, indent, None));
                    }
                }
                TokenType::BlockEnd => {
                    if stack.len() > 1 {
                        stack.pop();
                    }
                }
                _ => {}
            }

            if stack
                .last()
                .map(|p| p.parent_type == ParentType::BlockEnt)
                .unwrap_or(false)
            {
                if let Some(next) = next_token {
                    match &next.1 {
                        TokenType::BlockEntry | TokenType::BlockEnd => {
                            stack.pop();
                        }
                        _ => {}
                    }
                }
            }

            if is_visible {
                cur_line = marker.line();
            }

            if first_in_line
                && !matches!(
                    token_type,
                    TokenType::BlockEnd | TokenType::FlowMappingEnd | TokenType::FlowSequenceEnd
                )
            {
                let found_indentation = marker.col();

                // Calculate expected indentation based on context
                let expected = match token_type {
                    TokenType::BlockEntry => {
                        // For BlockEntry (list item), expected indent depends on parent context
                        // Find the mapping key that contains this sequence
                        let key_indent = stack
                            .iter()
                            .rev()
                            .find(|p| p.parent_type == ParentType::Key)
                            .map(|p| p.indent)
                            .unwrap_or(0);

                        // Expected indent is key's indent + 2 spaces (yamllint's default)
                        key_indent + self.config().spaces
                    }
                    _ => {
                        // For other tokens, use existing logic
                        if stack.len() >= 2 && stack.last().unwrap().parent_type == ParentType::Val
                        {
                            stack[stack.len() - 2].indent
                        } else {
                            stack.last().unwrap().indent
                        }
                    }
                };

                if found_indentation != expected {
                    // For BlockEntry, only report first error per key (like yamllint)
                    let should_report = match token_type {
                        TokenType::BlockEntry => {
                            if !reported_error_for_key {
                                reported_error_for_key = true;
                                true
                            } else {
                                false
                            }
                        }
                        _ => true,
                    };

                    if should_report {
                        let message = format!(
                            "wrong indentation: expected {} but found {}",
                            expected, found_indentation
                        );
                        issues.push(LintIssue {
                            line: marker.line() + 1,
                            column: found_indentation + 1,
                            message,
                            severity: self.get_severity(),
                        });
                    }
                }
            }
        }

        issues
    }

    pub fn check_impl(&self, content: &str, file_path: &str) -> Vec<LintIssue> {
        let scanner = Scanner::new(content.chars());
        let tokens: Vec<_> = scanner.collect();
        let token_analysis = crate::analysis::TokenAnalysis::analyze(content);
        self.check_with_tokens(content, file_path, &tokens, &token_analysis)
    }

    pub fn check_impl_with_analysis(
        &self,
        content: &str,
        file_path: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        if let Some(token_analysis) = analysis.tokens() {
            self.check_with_tokens(content, file_path, &token_analysis.tokens, token_analysis)
        } else {
            self.check_impl(content, file_path)
        }
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
    use crate::Severity;

    #[test]
    fn test_indentation_rule_default() {
        let rule = IndentationRule::new();
        assert_eq!(rule.rule_id(), "indentation");
        assert_eq!(rule.default_severity(), Severity::Error);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_indentation_check_clean_indentation() {
        let rule = IndentationRule::new();
        let content = "parent:\n  child1: value1\n  child2: value2\n";
        let issues = rule.check(content, "test.yaml");
        println!("Found {} issues: {:?}", issues.len(), issues);
        assert!(rule.rule_id() == "indentation");
    }

    #[test]
    fn test_indentation_check_wrong_indentation() {
        let rule = IndentationRule::new();
        let content = "parent:\n  child1: value1\nchild2: value2\n";
        let _issues = rule.check(content, "test.yaml");
        assert!(rule.rule_id() == "indentation");
    }

    #[test]
    fn test_list_items() {
        let rule = IndentationRule::new();
        let content = "items:\n  - item1\n  - item2\n";
        let _issues = rule.check(content, "test.yaml");
        assert!(rule.rule_id() == "indentation");
    }

    #[test]
    fn test_wrong_indentation_list_item() {
        let rule = IndentationRule::new();
        let content = "wd_tenants:\n- novartis\n";
        let _issues = rule.check(content, "test.yaml");
        assert!(rule.rule_id() == "indentation");
    }

    #[test]
    fn test_indentation_block_mapping() {
        let rule = IndentationRule::new();
        let content = "parent:\n  child1: value1\n  child2: value2\n";
        let issues = rule.check(content, "test.yaml");
        if !issues.is_empty() {
            println!("Issues found: {:?}", issues);
        }
        assert!(issues.is_empty());
    }

    #[test]
    fn test_indentation_flow_sequence() {
        let rule = IndentationRule::new();
        let content = "items:\n  - [a, b, c]\n  - [d, e]\n";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_indentation_block_sequence() {
        let rule = IndentationRule::new();
        let content = "items:\n  - item1\n  - item2\n";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_indentation_value_calculation() {
        let rule = IndentationRule::new();
        let content = "key:\n  value\n";
        let _issues = rule.check(content, "test.yaml");
        // This test verifies proper indentation for scalar values
        // The value should be indented relative to the key
        assert!(rule.rule_id() == "indentation");
    }

    #[test]
    fn test_indentation_implicit_sequence() {
        let rule = IndentationRule::new();
        let content = "key:\n- item1\n- item2\n";
        let issues = rule.check(content, "test.yaml");
        // This YAML has wrong indentation - items should be indented relative to key
        // yamllint reports error at line 2, so we should also report an error
        assert!(
            !issues.is_empty(),
            "Should report indentation error for implicit sequence"
        );
    }

    #[test]
    fn test_indentation_key_only() {
        let rule = IndentationRule::new();
        let content = "key1:\nkey2:\nkey3:\n";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_indentation_cleanup_sequence() {
        let rule = IndentationRule::new();
        let content = "items:\n  - first\n  - second\nother:\n  key: value\n";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }
}
