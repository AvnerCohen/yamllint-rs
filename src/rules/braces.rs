use crate::{LintIssue, Severity};
use yaml_rust::scanner::{Scanner, Token, TokenType};

#[derive(Debug, Clone)]
pub struct BracesConfig {
    pub forbid: ForbidSetting,
    pub min_spaces_inside: i32,
    pub max_spaces_inside: i32,
    pub min_spaces_inside_empty: i32,
    pub max_spaces_inside_empty: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForbidSetting {
    False,
    True,
    NonEmpty,
}

impl Default for ForbidSetting {
    fn default() -> Self {
        ForbidSetting::False
    }
}

impl Default for BracesConfig {
    fn default() -> Self {
        Self {
            forbid: ForbidSetting::False,
            min_spaces_inside: 0,
            max_spaces_inside: 0,
            min_spaces_inside_empty: -1,
            max_spaces_inside_empty: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BracesRule {
    base: crate::rules::base::BaseRule<BracesConfig>,
}

impl BracesRule {
    pub fn new() -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(BracesConfig::default()),
        }
    }

    pub fn with_config(config: BracesConfig) -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &BracesConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: BracesConfig) {
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

impl Default for BracesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::rules::Rule for BracesRule {
    fn rule_id(&self) -> &'static str {
        "braces"
    }

    fn rule_name(&self) -> &'static str {
        "Braces"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for proper spacing inside braces {}."
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

impl BracesRule {
    fn spaces_after(
        &self,
        token_marker: &yaml_rust::scanner::Marker,
        next_marker: &yaml_rust::scanner::Marker,
        _content: &str,
        min: i32,
        max: i32,
        min_desc: &str,
        max_desc: &str,
    ) -> Option<LintIssue> {
        if token_marker.line() != next_marker.line() {
            return None;
        }

        let token_end = token_marker.index() + 1;
        let next_start = next_marker.index();

        if next_start <= token_end {
            return None;
        }

        let spaces = next_start - token_end;

        if max != -1 && spaces > max as usize {
            return Some(LintIssue {
                line: token_marker.line() + 1,
                column: next_marker.col() + 1,
                message: max_desc.to_string(),
                severity: self.get_severity(),
            });
        }

        if min != -1 && spaces < min as usize {
            return Some(LintIssue {
                line: token_marker.line() + 1,
                column: next_marker.col() + 1,
                message: min_desc.to_string(),
                severity: self.get_severity(),
            });
        }

        None
    }

    fn spaces_before(
        &self,
        token_marker: &yaml_rust::scanner::Marker,
        prev_marker: &yaml_rust::scanner::Marker,
        prev_token_type: &TokenType,
        content: &str,
        min: i32,
        max: i32,
        min_desc: &str,
        max_desc: &str,
    ) -> Option<LintIssue> {
        if prev_marker.line() != token_marker.line() {
            return None;
        }

        let prev_start = prev_marker.index();
        let token_start = token_marker.index();

        let prev_end = match prev_token_type {
            TokenType::Scalar(_, scalar_value) => {
                if let Some(first_char) = content.chars().nth(prev_start) {
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
                }
            }
            TokenType::FlowMappingEnd | TokenType::FlowSequenceEnd => prev_start + 1,
            TokenType::FlowEntry => prev_start + 1,
            _ => prev_start,
        };

        if token_start <= prev_end {
            return None;
        }

        if prev_end > 0 {
            if let Some(prev_char) = content.as_bytes().get(prev_end - 1) {
                if *prev_char == b'\n' {
                    return None;
                }
            }
        }

        let spaces = token_start - prev_end;

        if max != -1 && spaces > max as usize {
            return Some(LintIssue {
                line: token_marker.line() + 1,
                column: token_marker.col() + 1,
                message: max_desc.to_string(),
                severity: self.get_severity(),
            });
        }

        if min != -1 && spaces < min as usize {
            return Some(LintIssue {
                line: token_marker.line() + 1,
                column: token_marker.col() + 1,
                message: min_desc.to_string(),
                severity: self.get_severity(),
            });
        }

        None
    }

    fn check_with_tokens(
        &self,
        content: &str,
        tokens: &[Token],
        _token_analysis: &crate::analysis::TokenAnalysis,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (i, token) in tokens.iter().enumerate() {
            let Token(marker, token_type) = token;

            let prev_token = if i > 0 { tokens.get(i - 1) } else { None };
            let next_token = tokens.get(i + 1);

            match token_type {
                TokenType::FlowMappingStart => {
                    if self.config().forbid == ForbidSetting::True {
                        issues.push(LintIssue {
                            line: marker.line() + 1,
                            column: marker.col() + 1,
                            message: "forbidden flow mapping".to_string(),
                            severity: self.get_severity(),
                        });
                    } else if let Some(next) = next_token {
                        let Token(next_marker, next_token_type) = next;
                        if matches!(next_token_type, TokenType::FlowMappingEnd) {
                            let min = if self.config().min_spaces_inside_empty != -1 {
                                self.config().min_spaces_inside_empty
                            } else {
                                self.config().min_spaces_inside
                            };
                            let max = if self.config().max_spaces_inside_empty != -1 {
                                self.config().max_spaces_inside_empty
                            } else {
                                self.config().max_spaces_inside
                            };

                            if let Some(issue) = self.spaces_after(
                                marker,
                                next_marker,
                                content,
                                min,
                                max,
                                "too few spaces inside empty braces",
                                "too many spaces inside empty braces",
                            ) {
                                issues.push(issue);
                            }
                        } else {
                            if self.config().forbid == ForbidSetting::NonEmpty {
                                issues.push(LintIssue {
                                    line: marker.line() + 1,
                                    column: marker.col() + 1,
                                    message: "forbidden flow mapping".to_string(),
                                    severity: self.get_severity(),
                                });
                            } else {
                                if let Some(issue) = self.spaces_after(
                                    marker,
                                    next_marker,
                                    content,
                                    self.config().min_spaces_inside,
                                    self.config().max_spaces_inside,
                                    "too few spaces inside braces",
                                    "too many spaces inside braces",
                                ) {
                                    issues.push(issue);
                                }
                            }
                        }
                    }
                }
                TokenType::FlowMappingEnd => {
                    if let Some(prev) = prev_token {
                        let Token(prev_marker, prev_token_type) = prev;
                        if !matches!(prev_token_type, TokenType::FlowMappingStart) {
                            if let Some(issue) = self.spaces_before(
                                marker,
                                prev_marker,
                                prev_token_type,
                                content,
                                self.config().min_spaces_inside,
                                self.config().max_spaces_inside,
                                "too few spaces inside braces",
                                "too many spaces inside braces",
                            ) {
                                issues.push(issue);
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

    pub fn fix(&self, content: &str, _file_path: &str) -> super::FixResult {
        let mut fixed_lines = Vec::new();
        let mut fixes_applied = 0;

        for line in content.lines() {
            let mut fixed_line = line.to_string();

            if let Some(brace_start) = line.find('{') {
                if let Some(brace_end_offset) = line[brace_start..].find('}') {
                    let brace_end = brace_start + brace_end_offset;
                    let inside_braces: String = line
                        .chars()
                        .skip(brace_start + 1)
                        .take(brace_end - brace_start - 1)
                        .collect();
                    let is_empty = inside_braces.trim().is_empty();

                    if is_empty {
                        if self.config().min_spaces_inside_empty >= 0
                            || self.config().max_spaces_inside_empty >= 0
                        {
                            let current_spaces = inside_braces.len();
                            let target_spaces = if self.config().min_spaces_inside_empty >= 0 {
                                self.config().min_spaces_inside_empty as usize
                            } else {
                                0
                            };

                            if current_spaces != target_spaces {
                                let new_inside = " ".repeat(target_spaces);
                                let new_braces = format!("{{{}}}", new_inside);
                                let before_braces: String =
                                    line.chars().take(brace_start).collect();
                                let after_braces: String =
                                    line.chars().skip(brace_end + 1).collect();
                                fixed_line = before_braces + &new_braces + &after_braces;
                                fixes_applied += 1;
                            }
                        }
                    } else if self.config().forbid == ForbidSetting::False {
                        let content_part = inside_braces.trim();
                        let target_spaces = if self.config().min_spaces_inside >= 0 {
                            self.config().min_spaces_inside as usize
                        } else {
                            0
                        };

                        let leading_spaces = inside_braces.len() - inside_braces.trim_start().len();
                        let trailing_spaces = inside_braces.len() - inside_braces.trim_end().len();
                        let current_total_spaces = leading_spaces + trailing_spaces;

                        if current_total_spaces != target_spaces {
                            let new_inside = if target_spaces > 0 {
                                format!(" {} ", content_part)
                            } else {
                                content_part.to_string()
                            };

                            let new_braces = format!("{{{}}}", new_inside);
                            let before_braces: String = line.chars().take(brace_start).collect();
                            let after_braces: String = line.chars().skip(brace_end + 1).collect();
                            fixed_line = before_braces + &new_braces + &after_braces;
                            fixes_applied += 1;
                        }
                    }
                }
            }

            fixed_lines.push(fixed_line);
        }

        let fixed_content = if content.ends_with('\n') {
            fixed_lines.join("\n") + "\n"
        } else {
            fixed_lines.join("\n")
        };

        let changed = fixes_applied > 0;

        super::FixResult {
            content: fixed_content,
            changed,
            fixes_applied,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Rule;
    use crate::Severity;

    #[test]
    fn test_braces_rule_default() {
        let rule = BracesRule::new();
        assert_eq!(rule.rule_id(), "braces");
        assert_eq!(rule.default_severity(), Severity::Warning);
        assert!(rule.is_enabled_by_default());
        assert!(rule.can_fix());
    }

    #[test]
    fn test_braces_check_clean_braces() {
        let rule = BracesRule::new();
        let content = "key: {value1, value2}\nempty: {}";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_braces_check_spaces_when_forbidden() {
        let rule = BracesRule::new();
        let content = "key: { value1, value2 }";
        let issues = rule.check(content, "test.yaml");
        // yamllint reports 2 issues: one for spaces after { and one for spaces before }
        assert_eq!(issues.len(), 2);
        assert!(issues
            .iter()
            .any(|issue| issue.message.contains("too many spaces inside braces")));
    }

    #[test]
    fn test_braces_fix() {
        let rule = BracesRule::new();
        let content = "key: { value1, value2 }";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 1);
        assert!(fix_result.content.contains("key: {value1, value2}"));
    }

    #[test]
    fn test_braces_fix_no_changes() {
        let rule = BracesRule::new();
        let content = "key: {value1, value2}\nempty: {}";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }
}
