//! Single-pass content analysis system.

use std::collections::HashMap;
use yaml_rust::scanner::{Scanner, Token, TokenType};

/// Information about a single line
#[derive(Debug, Clone)]
pub struct LineInfo {
    /// Line number (1-based)
    pub line_number: usize,
    /// Line length
    pub length: usize,
    /// Whether the line is empty
    pub is_empty: bool,
    /// Whether the line is a comment
    pub is_comment: bool,
    /// Whether the line has trailing whitespace
    pub has_trailing_whitespace: bool,
    /// Number of trailing whitespace characters
    pub trailing_whitespace_count: usize,
    /// Indentation level (number of leading spaces/tabs)
    pub indentation: usize,
    /// Whether the line starts with a hyphen (list item)
    pub is_list_item: bool,
    /// Whether the line contains a colon (key-value pair)
    pub has_colon: bool,
    /// Whether the line contains quotes
    pub has_quotes: bool,
    /// Whether the line contains braces
    pub has_braces: bool,
    /// Whether the line contains brackets
    pub has_brackets: bool,
}

#[derive(Debug, Clone)]
pub struct TokenAnalysis {
    pub tokens: Vec<Token>,
    pub flow_depths: Vec<usize>,
    pub token_to_line: Vec<usize>,
}

impl TokenAnalysis {
    pub fn analyze(content: &str) -> Self {
        let scanner = Scanner::new(content.chars());
        let tokens: Vec<_> = scanner.collect();

        let mut flow_depths = Vec::with_capacity(tokens.len());
        let mut token_to_line = Vec::with_capacity(tokens.len());
        let mut current_flow_depth = 0;

        for token in &tokens {
            let Token(marker, token_type) = token;
            token_to_line.push(marker.line() + 1);

            match token_type {
                TokenType::FlowMappingStart | TokenType::FlowSequenceStart => {
                    current_flow_depth += 1;
                    flow_depths.push(current_flow_depth);
                }
                TokenType::FlowMappingEnd | TokenType::FlowSequenceEnd => {
                    flow_depths.push(current_flow_depth);
                    if current_flow_depth > 0 {
                        current_flow_depth -= 1;
                    }
                }
                _ => {
                    flow_depths.push(current_flow_depth);
                }
            }
        }

        Self {
            tokens,
            flow_depths,
            token_to_line,
        }
    }

    pub fn get_tokens_for_line(&self, line_number: usize) -> Vec<(usize, &Token)> {
        self.tokens
            .iter()
            .enumerate()
            .filter(|(idx, _)| {
                if let Some(&line) = self.token_to_line.get(*idx) {
                    line == line_number
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_flow_depth(&self, token_idx: usize) -> usize {
        self.flow_depths.get(token_idx).copied().unwrap_or(0)
    }

    pub fn is_in_flow(&self, token_idx: usize) -> bool {
        self.get_flow_depth(token_idx) > 0
    }
}

#[derive(Debug, Clone)]
pub struct ContentAnalysis {
    pub lines: Vec<LineInfo>,
    pub line_count: usize,
    pub ends_with_newline: bool,
    pub starts_with_document_marker: bool,
    pub ends_with_document_marker: bool,
    pub truthy_values: HashMap<usize, Vec<String>>,
    pub duplicate_keys: HashMap<usize, Vec<String>>,
    pub empty_values: HashMap<usize, Vec<String>>,
    pub tokens: Option<TokenAnalysis>,
}

impl ContentAnalysis {
    pub fn analyze(content: &str) -> Self {
        Self::analyze_with_tokens(content, true)
    }

    pub fn analyze_with_tokens(content: &str, include_tokens: bool) -> Self {
        let mut lines = Vec::new();
        let mut truthy_values = HashMap::new();
        let mut duplicate_keys = HashMap::new();
        let mut empty_values = HashMap::new();

        let mut structure = YamlStructure::new();
        let mut current_contexts: Vec<usize> = Vec::new();

        let mut line_number = 1;

        let tokens = if include_tokens {
            Some(TokenAnalysis::analyze(content))
        } else {
            None
        };

        for line in content.lines() {
            let trimmed = line.trim();
            let indentation = line.len() - line.trim_start().len();

            let line_info = Self::analyze_line(line_number, line);

            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                current_contexts.retain(|&context_idx| {
                    if context_idx < structure.contexts.len() {
                        let context = &structure.contexts[context_idx];
                        if indentation >= context.indentation {
                            true
                        } else {
                            structure.contexts[context_idx].end_line = Some(line_number - 1);
                            false
                        }
                    } else {
                        false
                    }
                });

                if trimmed.starts_with('-') {
                    let new_context = MappingContext::new(line_number, indentation);
                    structure.contexts.push(new_context);
                    let context_idx = structure.contexts.len() - 1;
                    current_contexts.push(context_idx);
                } else if line.contains(':') {
                    if let Some(key) = Self::extract_key(line) {
                        let context_idx = Self::get_or_create_context_for_indentation(
                            &mut structure,
                            &mut current_contexts,
                            indentation,
                            line_number,
                        );

                        if let Some(prev_line) =
                            structure.contexts[context_idx].get_duplicate_key(&key, line_number)
                        {
                            if prev_line != line_number {
                                duplicate_keys
                                    .entry(prev_line)
                                    .or_insert_with(Vec::new)
                                    .push(key.clone());
                                duplicate_keys
                                    .entry(line_number)
                                    .or_insert_with(Vec::new)
                                    .push(key.clone());
                            }
                        }

                        structure.contexts[context_idx].add_key(key, line_number);
                    }
                }
            }

            let mut line_truthy_values = Vec::new();
            for word in line.split_whitespace() {
                let trimmed = word.trim_end_matches(',');
                if Self::is_truthy_value(trimmed) {
                    line_truthy_values.push(trimmed.to_string());
                }
            }
            if !line_truthy_values.is_empty() {
                truthy_values.insert(line_number, line_truthy_values);
            }

            if line_info.has_colon {
                if let Some(value) = Self::extract_value(line) {
                    if value.trim().is_empty() {
                        empty_values
                            .entry(line_number)
                            .or_insert_with(Vec::new)
                            .push(value);
                    }
                }
            }

            lines.push(line_info);
            line_number += 1;
        }

        for context_idx in current_contexts {
            if context_idx < structure.contexts.len() {
                structure.contexts[context_idx].end_line = Some(line_number - 1);
            }
        }

        let line_count = lines.len();
        let ends_with_newline = content.ends_with('\n');
        let starts_with_document_marker = content.starts_with("---");
        let ends_with_document_marker = content.ends_with("...");

        Self {
            lines,
            line_count,
            ends_with_newline,
            starts_with_document_marker,
            ends_with_document_marker,
            truthy_values,
            duplicate_keys,
            empty_values,
            tokens,
        }
    }

    pub fn tokens(&self) -> Option<&TokenAnalysis> {
        self.tokens.as_ref()
    }

    fn analyze_line(line_number: usize, line: &str) -> LineInfo {
        let length = line.len();
        let trimmed = line.trim();
        let is_empty = trimmed.is_empty();
        let is_comment = trimmed.starts_with('#');
        let has_trailing_whitespace = line.ends_with(' ') || line.ends_with('\t');
        let trailing_whitespace_count = if has_trailing_whitespace {
            line.len() - line.trim_end().len()
        } else {
            0
        };

        // Calculate indentation
        let indentation = line.len() - line.trim_start().len();

        // Check for various patterns
        let is_list_item = line.trim_start().starts_with('-');
        let has_colon = line.contains(':');
        let has_quotes = line.contains('"') || line.contains('\'');
        let has_braces = line.contains('{') || line.contains('}');
        let has_brackets = line.contains('[') || line.contains(']');

        LineInfo {
            line_number,
            length,
            is_empty,
            is_comment,
            has_trailing_whitespace,
            trailing_whitespace_count,
            indentation,
            is_list_item,
            has_colon,
            has_quotes,
            has_braces,
            has_brackets,
        }
    }

    /// Check if a value is truthy
    fn is_truthy_value(value: &str) -> bool {
        matches!(
            value.to_lowercase().as_str(),
            "yes"
                | "no"
                | "on"
                | "off"
                | "y"
                | "n"
                | "true"
                | "false"
                | "1"
                | "0"
                | "enable"
                | "disable"
                | "enabled"
                | "disabled"
        )
    }

    /// Extract key from a key-value line
    fn extract_key(line: &str) -> Option<String> {
        if let Some(colon_pos) = line.find(':') {
            Some(line[..colon_pos].trim().to_string())
        } else {
            None
        }
    }

    /// Extract value from a key-value line
    fn extract_value(line: &str) -> Option<String> {
        if let Some(colon_pos) = line.find(':') {
            Some(line[colon_pos + 1..].trim().to_string())
        } else {
            None
        }
    }

    /// Get line information by line number
    pub fn get_line(&self, line_number: usize) -> Option<&LineInfo> {
        if line_number > 0 && line_number <= self.lines.len() {
            Some(&self.lines[line_number - 1])
        } else {
            None
        }
    }

    /// Get all lines that exceed a certain length
    pub fn get_long_lines(&self, max_length: usize) -> Vec<&LineInfo> {
        self.lines
            .iter()
            .filter(|line| line.length > max_length)
            .collect()
    }

    /// Get all lines with trailing whitespace
    pub fn get_lines_with_trailing_whitespace(&self) -> Vec<&LineInfo> {
        self.lines
            .iter()
            .filter(|line| line.has_trailing_whitespace)
            .collect()
    }

    /// Get all empty lines
    pub fn get_empty_lines(&self) -> Vec<&LineInfo> {
        self.lines.iter().filter(|line| line.is_empty).collect()
    }

    /// Get all comment lines
    pub fn get_comment_lines(&self) -> Vec<&LineInfo> {
        self.lines.iter().filter(|line| line.is_comment).collect()
    }

    /// Get all list item lines
    pub fn get_list_item_lines(&self) -> Vec<&LineInfo> {
        self.lines.iter().filter(|line| line.is_list_item).collect()
    }

    /// Get all key-value lines
    pub fn get_key_value_lines(&self) -> Vec<&LineInfo> {
        self.lines.iter().filter(|line| line.has_colon).collect()
    }
    /// Get or create a context for the given indentation level
    fn get_or_create_context_for_indentation(
        structure: &mut YamlStructure,
        current_contexts: &mut Vec<usize>,
        indentation: usize,
        line_number: usize,
    ) -> usize {
        // Find existing context at this exact indentation level
        for &context_idx in current_contexts.iter().rev() {
            if context_idx < structure.contexts.len() {
                let context = &structure.contexts[context_idx];
                if context.indentation == indentation && context.is_active() {
                    return context_idx;
                }
            }
        }

        // Create new context for this indentation level
        let new_context = MappingContext::new(line_number, indentation);
        structure.contexts.push(new_context);
        let context_idx = structure.contexts.len() - 1;

        // Add to current contexts
        current_contexts.push(context_idx);

        context_idx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_analysis_basic() {
        let content = "key1: value1\nkey2: value2\nkey3: value3";
        let analysis = ContentAnalysis::analyze(content);

        assert_eq!(analysis.line_count, 3);
        assert_eq!(analysis.lines.len(), 3);
        assert!(analysis.duplicate_keys.is_empty());
        assert!(analysis.truthy_values.is_empty());
        assert!(analysis.empty_values.is_empty());
    }

    #[test]
    fn test_content_analysis_duplicate_keys_same_context() {
        let content = "name: John\nage: 30\nname: Jane";
        let analysis = ContentAnalysis::analyze(content);

        assert_eq!(analysis.duplicate_keys.len(), 2);
        assert!(analysis.duplicate_keys.contains_key(&1)); // First 'name'
        assert!(analysis.duplicate_keys.contains_key(&3)); // Second 'name'
    }

    #[test]
    fn test_content_analysis_duplicate_keys_different_contexts() {
        let content = r#"step_code: first
transitions:
- step_code: second
  fields:
    step_code: third"#;
        let analysis = ContentAnalysis::analyze(content);

        // Should NOT detect duplicates across different contexts
        assert!(
            analysis.duplicate_keys.is_empty(),
            "Should not detect duplicates across different contexts. Found: {:?}",
            analysis.duplicate_keys
        );
    }

    #[test]
    fn test_content_analysis_nested_mappings() {
        let content = r#"name: John
address:
  street: Main St
  city: New York
contact:
  street: Broadway
  city: Boston"#;
        let analysis = ContentAnalysis::analyze(content);

        // Should NOT detect duplicates across different nested mappings
        assert!(
            analysis.duplicate_keys.is_empty(),
            "Should not detect duplicates across different nested mappings. Found: {:?}",
            analysis.duplicate_keys
        );
    }

    #[test]
    fn test_content_analysis_list_items() {
        let content = r#"- name: item1
  value: 100
- name: item2
  value: 200"#;
        let analysis = ContentAnalysis::analyze(content);

        // Should NOT detect duplicates across different list items
        assert!(
            analysis.duplicate_keys.is_empty(),
            "Should not detect duplicates across different list items. Found: {:?}",
            analysis.duplicate_keys
        );
    }

    #[test]
    fn test_content_analysis_complex_structure() {
        // Test the exact structure that was failing before
        let content = r#"- hrm_phase_id: hm_manager_review
  is_disposition_step: false
  phase_id: review
  shortcut_types:
  - send_to_hm
  source_system: workday
  stage_name: Review
  status_stage: pending
  step_code: JOB_APPLICATION_DEFAULT_DEFINITION_STEP_B__ACTION
  step_name: Review
  transitions:
  - conditions:
      conditions:
        or:
        - not_has_intersection:
            CF - LRV - Current User's Org Roles:
            - Primary Recruiter
            - Recruiter (Local)
            - Recruiter (Supervisory)
        - has_intersection:
            CF - LRV - Current User's Org Roles:
            - Manager
            Current User:
            - ISU_HiredScore
      fields:
        CF - LRV - Current User's Org Roles: not_supported
        Current User: not_supported
    step_code: JOB_APPLICATION_DEFAULT_DEFINITION_STEP_A_NEW_ACTION"#;
        let analysis = ContentAnalysis::analyze(content);

        // Should NOT detect duplicates in complex nested structure
        assert!(
            analysis.duplicate_keys.is_empty(),
            "Should not detect duplicates in complex nested structure. Found: {:?}",
            analysis.duplicate_keys
        );
    }

    #[test]
    fn test_content_analysis_truthy_values() {
        let content = "enabled: yes\ndisabled: no\nflag: true\nvalue: 1";
        let analysis = ContentAnalysis::analyze(content);

        assert_eq!(analysis.truthy_values.len(), 4);
        assert!(analysis.truthy_values.contains_key(&1)); // 'yes'
        assert!(analysis.truthy_values.contains_key(&2)); // 'no'
        assert!(analysis.truthy_values.contains_key(&3)); // 'true'
        assert!(analysis.truthy_values.contains_key(&4)); // '1'
    }

    #[test]
    fn test_content_analysis_empty_values() {
        let content = "key1: \nkey2: value\nkey3:   \nkey4: another";
        let analysis = ContentAnalysis::analyze(content);

        assert_eq!(analysis.empty_values.len(), 2);
        assert!(analysis.empty_values.contains_key(&1)); // Empty value
        assert!(analysis.empty_values.contains_key(&3)); // Whitespace-only value
    }

    #[test]
    fn test_content_analysis_line_info() {
        let content = "  - key: value  \n# comment\n\nkey2: value2";
        let analysis = ContentAnalysis::analyze(content);

        assert_eq!(analysis.lines.len(), 4);

        // Line 1: list item with indentation and trailing whitespace
        let line1 = &analysis.lines[0];
        assert_eq!(line1.line_number, 1);
        assert_eq!(line1.indentation, 2);
        assert!(line1.is_list_item);
        assert!(line1.has_colon);
        assert!(line1.has_trailing_whitespace);

        // Line 2: comment
        let line2 = &analysis.lines[1];
        assert_eq!(line2.line_number, 2);
        assert!(line2.is_comment);
        assert!(!line2.has_colon);

        // Line 3: empty line
        let line3 = &analysis.lines[2];
        assert_eq!(line3.line_number, 3);
        assert!(line3.is_empty);

        // Line 4: key-value pair
        let line4 = &analysis.lines[3];
        assert_eq!(line4.line_number, 4);
        assert!(line4.has_colon);
        assert!(!line4.is_list_item);
    }

    #[test]
    fn test_content_analysis_document_markers() {
        let content = "---\nkey: value\n...";
        let analysis = ContentAnalysis::analyze(content);

        assert!(analysis.starts_with_document_marker);
        assert!(analysis.ends_with_document_marker);
    }

    #[test]
    fn test_content_analysis_newline_handling() {
        let content_with_newline = "key: value\n";
        let content_without_newline = "key: value";

        let analysis_with = ContentAnalysis::analyze(content_with_newline);
        let analysis_without = ContentAnalysis::analyze(content_without_newline);

        assert!(analysis_with.ends_with_newline);
        assert!(!analysis_without.ends_with_newline);
    }
}

/// Represents the YAML structure for context-aware duplicate key detection
#[derive(Debug)]
struct YamlStructure {
    contexts: Vec<MappingContext>,
}

impl YamlStructure {
    fn new() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }
}

/// Represents a mapping context for duplicate key detection
#[derive(Debug, Clone, Default)]
struct MappingContext {
    #[allow(dead_code)] // May be used in future features
    start_line: usize,
    end_line: Option<usize>,
    indentation: usize,
    keys: HashMap<String, usize>,
    active: bool,
}

impl MappingContext {
    fn new(start_line: usize, indentation: usize) -> Self {
        Self {
            start_line,
            end_line: None,
            indentation,
            keys: HashMap::new(),
            active: true,
        }
    }

    fn add_key(&mut self, key: String, line_number: usize) {
        self.keys.insert(key, line_number);
    }

    fn get_duplicate_key(&self, key: &str, _line_number: usize) -> Option<usize> {
        self.keys.get(key).copied()
    }

    fn is_active(&self) -> bool {
        self.active
    }
}
