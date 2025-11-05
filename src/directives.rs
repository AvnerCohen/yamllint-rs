//! Directive parsing for in-file rule control.

use crate::LintIssue;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};

lazy_static! {
    static ref DISABLE_PATTERN: Regex =
        Regex::new(r"^# (yamllint|yamllint-rs) disable( rule:\S+)*\s*$").unwrap();
    static ref ENABLE_PATTERN: Regex =
        Regex::new(r"^# (yamllint|yamllint-rs) enable( rule:\S+)*\s*$").unwrap();
    static ref DISABLE_LINE_PATTERN: Regex =
        Regex::new(r"^# (yamllint|yamllint-rs) disable-line( rule:\S+)*\s*$").unwrap();
}

pub struct DirectiveState {
    // Global state: disabled rules persist until explicitly enabled
    // Maps line number to set of disabled rules starting from that line
    global_disabled_from_line: HashMap<usize, HashSet<String>>,

    // Global state: enabled rules starting from a line
    // Maps line number to set of enabled rules starting from that line
    global_enabled_from_line: HashMap<usize, HashSet<String>>,

    // Per-line state: disabled rules for specific lines
    line_disabled: HashMap<usize, HashSet<String>>,

    // All available rules (for validation)
    all_rules: HashSet<String>,
}

impl DirectiveState {
    pub fn new(all_rules: HashSet<String>) -> Self {
        Self {
            global_disabled_from_line: HashMap::new(),
            global_enabled_from_line: HashMap::new(),
            line_disabled: HashMap::new(),
            all_rules,
        }
    }

    /// Parse all directives from content and build state
    /// In yamllint, directives are processed line-by-line:
    /// - Block comment on line N → affects line N+1 and onwards (disabled_for_next_line)
    /// - Inline comment on line N → affects line N (disabled_for_line)
    pub fn parse_from_content(&mut self, content: &str) {
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1; // 1-indexed

            // Check if line is a block comment (starts with #)
            let trimmed = line.trim();
            let is_block_comment = trimmed.starts_with('#');

            // Check for inline comment (contains # not in quotes)
            let inline_comment = Self::extract_inline_comment(line);

            // Process block comment first (if it's a directive line)
            if is_block_comment {
                // Block comment on line N affects line N+1 and onwards
                self.process_comment(line_num, trimmed, false);
            }

            // Process inline comment (if present and not already processed as block)
            if let Some(comment_part) = inline_comment {
                if !is_block_comment {
                    // Only process if it wasn't already processed as a block comment
                    // Inline comment on line N affects line N
                    self.process_comment(line_num, comment_part, true);
                }
            }
        }
    }

    /// Extract inline comment from a line (everything after #)
    fn extract_inline_comment(line: &str) -> Option<&str> {
        // Simple approach: find first # that's not in quotes
        let mut in_single_quotes = false;
        let mut in_double_quotes = false;
        let mut escape_next = false;

        for (i, ch) in line.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => escape_next = true,
                '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
                '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
                '#' if !in_single_quotes && !in_double_quotes => {
                    return Some(&line[i..]);
                }
                _ => {}
            }
        }

        None
    }

    /// Process a single comment (matches yamllint's process_comment logic)
    fn process_comment(&mut self, line_num: usize, comment: &str, is_inline: bool) {
        let comment = comment.trim();

        // Match disable pattern
        if DISABLE_PATTERN.is_match(comment) {
            let rules = self.parse_rule_list(comment, "disable");
            if is_inline {
                // Inline comment → disable for this line only (like disable-line)
                self.apply_line_disable(line_num, rules);
            } else {
                // Block comment → disable globally starting from this line
                // In yamllint, block comments set disabled_for_next_line, but
                // when the comment line itself is processed, it's also suppressed
                // So we disable from this line (inclusive)
                self.apply_global_disable(line_num, rules);
            }
        }
        // Match enable pattern
        else if ENABLE_PATTERN.is_match(comment) {
            let rules = self.parse_rule_list(comment, "enable");
            // Enable only works globally (not line-specific)
            // Block comment on line N affects line N and onwards
            self.apply_global_enable(line_num, rules);
        }
        // Match disable-line pattern
        else if DISABLE_LINE_PATTERN.is_match(comment) {
            let rules = self.parse_rule_list(comment, "disable-line");
            // disable-line always affects the line it's on
            // For block comments, it affects the next line (line_num + 1)
            // For inline comments, it affects the current line
            let target_line = if is_inline { line_num } else { line_num + 1 };
            self.apply_line_disable(target_line, rules);
        }
    }

    /// Parse rule list from directive (matches yamllint's parsing logic exactly)
    /// "# yamllint disable rule:line-length rule:indentation"
    /// Returns: ["line-length", "indentation"]
    fn parse_rule_list(&self, comment: &str, action: &str) -> Vec<String> {
        // Find the prefix position (after "# yamllint " or "# yamllint-rs ")
        let prefix_patterns = ["# yamllint ", "# yamllint-rs "];
        let mut prefix_pos = 0;

        for prefix in &prefix_patterns {
            if let Some(pos) = comment.find(prefix) {
                prefix_pos = pos + prefix.len();
                break;
            }
        }

        // Find action position
        if let Some(action_pos) = comment[prefix_pos..].find(action) {
            let after_action = &comment[prefix_pos + action_pos + action.len()..];
            let after_action = after_action.trim();

            // Split by space, extract rule IDs from "rule:ID" items
            // This matches yamllint's logic: items = comment[18:].rstrip().split(' ')
            // For "# yamllint disable rule:line-length", after_action is "rule:line-length"
            // For "# yamllint disable", after_action is ""
            let items: Vec<&str> = after_action.split(' ').collect();

            // Extract rules: rules = [item[5:] for item in items][1:]
            // In yamllint, items[0] is the action word itself (empty after the action),
            // so we skip it. But in our case, after_action doesn't include the action,
            // so items[0] is the first rule token.
            // Actually, if after_action is "rule:line-length rule:indentation",
            // items = ["rule:line-length", "rule:indentation"]
            // We should NOT skip the first item - we should process all items
            items
                .iter()
                .filter_map(|item| {
                    if item.starts_with("rule:") {
                        Some(item[5..].to_string()) // Skip "rule:"
                    } else if !item.is_empty() {
                        // Handle case where action might be followed by non-rule text
                        None
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Apply global disable starting from a line
    fn apply_global_disable(&mut self, line_num: usize, rules: Vec<String>) {
        let disabled_rules: HashSet<String> = if rules.is_empty() {
            // Disable all rules
            self.all_rules.clone()
        } else {
            // Disable specific rules
            rules
                .into_iter()
                .filter(|rule| self.all_rules.contains(rule))
                .collect()
        };

        // Track that these rules are disabled starting from this line
        self.global_disabled_from_line
            .insert(line_num, disabled_rules);
    }

    /// Apply global enable starting from a line
    fn apply_global_enable(&mut self, line_num: usize, rules: Vec<String>) {
        let enabled_rules: HashSet<String> = if rules.is_empty() {
            // Enable all rules
            self.all_rules.clone()
        } else {
            // Enable specific rules
            rules
                .into_iter()
                .filter(|rule| self.all_rules.contains(rule))
                .collect()
        };

        // Track that these rules are enabled starting from this line
        self.global_enabled_from_line
            .insert(line_num, enabled_rules);
    }

    /// Apply line-specific disable
    fn apply_line_disable(&mut self, line_num: usize, rules: Vec<String>) {
        let line_set = self
            .line_disabled
            .entry(line_num)
            .or_insert_with(HashSet::new);

        if rules.is_empty() {
            // Disable all rules for this line
            *line_set = self.all_rules.clone();
        } else {
            // Disable specific rules for this line
            for rule in rules {
                if self.all_rules.contains(&rule) {
                    line_set.insert(rule);
                }
            }
        }
    }

    /// Check if rule is disabled for a line (matches yamllint's is_disabled_by_directive)
    pub fn is_rule_disabled(&self, line_num: usize, rule_id: &str) -> bool {
        // Check line-specific first (like yamllint's disabled_for_line)
        if let Some(line_rules) = self.line_disabled.get(&line_num) {
            if line_rules.contains(rule_id) {
                return true;
            }
        }

        // Check global (like yamllint's disabled)
        // Find the most recent disable directive before or on this line
        let mut most_recent_disable_line: Option<usize> = None;
        for (&disable_line, disabled_rules) in &self.global_disabled_from_line {
            if disable_line <= line_num {
                // Check if this disable affects this rule
                // If disabled_rules is empty (disable all), or rule_id is in the set
                let rule_is_disabled =
                    disabled_rules.is_empty() || disabled_rules.contains(rule_id);

                if rule_is_disabled {
                    if most_recent_disable_line.is_none()
                        || disable_line > most_recent_disable_line.unwrap()
                    {
                        most_recent_disable_line = Some(disable_line);
                    }
                }
            }
        }

        // Check for enable directives - if there's an enable for this rule on or before this line,
        // it overrides any earlier disables
        let mut most_recent_enable_line: Option<usize> = None;
        for (&enable_line, enabled_rules) in &self.global_enabled_from_line {
            if enable_line <= line_num {
                // Check if this enable affects this rule
                let rule_is_enabled = enabled_rules.is_empty() || enabled_rules.contains(rule_id);

                if rule_is_enabled {
                    if most_recent_enable_line.is_none()
                        || enable_line > most_recent_enable_line.unwrap()
                    {
                        most_recent_enable_line = Some(enable_line);
                    }
                }
            }
        }

        // If there's an enable, check if there's a disable after it
        if let Some(enable_line) = most_recent_enable_line {
            // Check for disable directives after the enable
            for (&disable_line, disabled_rules) in &self.global_disabled_from_line {
                if disable_line > enable_line && disable_line <= line_num {
                    let rule_is_disabled =
                        disabled_rules.is_empty() || disabled_rules.contains(rule_id);
                    if rule_is_disabled {
                        // There's a disable after the enable, so rule is disabled
                        return true;
                    }
                }
            }
            // No disable after enable, so rule is enabled
            return false;
        }

        // If there's a disable (and no enable), rule is disabled
        if most_recent_disable_line.is_some() {
            return true;
        }

        false
    }

    /// Filter issues based on directives
    pub fn filter_issues(&self, issues: Vec<(LintIssue, String)>) -> Vec<(LintIssue, String)> {
        issues
            .into_iter()
            .filter(|(issue, rule_id)| !self.is_rule_disabled(issue.line, rule_id))
            .collect()
    }
}
