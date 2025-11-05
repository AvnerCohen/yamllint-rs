use crate::{LintIssue, Severity};
use yaml_rust::scanner::{Scanner, Token, TokenType};

#[derive(Debug, Clone, PartialEq)]
enum ParentType {
    Map,
    Seq,
}

#[derive(Debug, Clone)]
struct Parent {
    parent_type: ParentType,
    keys: Vec<String>,
}

impl Parent {
    fn new(parent_type: ParentType) -> Self {
        Self {
            parent_type,
            keys: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyDuplicatesConfig {
    pub forbid_duplicated_merge_keys: bool,
}

impl Default for KeyDuplicatesConfig {
    fn default() -> Self {
        Self {
            forbid_duplicated_merge_keys: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyDuplicatesRule {
    base: crate::rules::base::BaseRule<KeyDuplicatesConfig>,
}

impl KeyDuplicatesRule {
    pub fn new() -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(KeyDuplicatesConfig::default()),
        }
    }

    pub fn with_config(config: KeyDuplicatesConfig) -> Self {
        Self {
            base: crate::rules::base::BaseRule::new(config),
        }
    }

    pub fn config(&self) -> &KeyDuplicatesConfig {
        self.base.config()
    }

    pub fn set_config(&mut self, config: KeyDuplicatesConfig) {
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

impl Default for KeyDuplicatesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::rules::Rule for KeyDuplicatesRule {
    fn rule_id(&self) -> &'static str {
        "key-duplicates"
    }

    fn rule_name(&self) -> &'static str {
        "Key Duplicates"
    }

    fn rule_description(&self) -> &'static str {
        "Checks for duplicate keys in YAML mappings."
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

    fn check_with_analysis(
        &self,
        content: &str,
        _file_path: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        self.check_impl_with_analysis(content, analysis)
    }
}

impl KeyDuplicatesRule {
    fn check_with_tokens(
        &self,
        tokens: &[Token],
        _token_analysis: &crate::analysis::TokenAnalysis,
    ) -> Vec<LintIssue> {
        let mut issues = Vec::new();
        let mut stack: Vec<Parent> = Vec::new();

        for (i, token) in tokens.iter().enumerate() {
            let Token(marker, token_type) = token;

            match token_type {
                TokenType::BlockMappingStart | TokenType::FlowMappingStart => {
                    stack.push(Parent::new(ParentType::Map));
                }
                TokenType::BlockSequenceStart | TokenType::FlowSequenceStart => {
                    stack.push(Parent::new(ParentType::Seq));
                }
                TokenType::BlockEnd | TokenType::FlowMappingEnd | TokenType::FlowSequenceEnd => {
                    if !stack.is_empty() {
                        stack.pop();
                    }
                }
                TokenType::Key => {
                    if let Some(next_token) = tokens.get(i + 1) {
                        let Token(_, next_token_type) = next_token;
                        if let TokenType::Scalar(_scalar_type, key_value) = next_token_type {
                            if !stack.is_empty()
                                && stack.last().unwrap().parent_type == ParentType::Map
                            {
                                let current_parent = stack.last_mut().unwrap();

                                if current_parent.keys.contains(key_value) {
                                    if key_value != "<<"
                                        || self.config().forbid_duplicated_merge_keys
                                    {
                                        issues.push(LintIssue {
                                            line: marker.line() + 1,
                                            column: marker.col() + 1,
                                            message: format!(
                                                "duplication of key \"{}\" in mapping",
                                                key_value
                                            ),
                                            severity: self.get_severity(),
                                        });
                                    }
                                } else {
                                    current_parent.keys.push(key_value.clone());
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
        self.check_with_tokens(&tokens, &token_analysis)
    }

    pub fn check_impl_with_analysis(
        &self,
        content: &str,
        analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        if let Some(token_analysis) = analysis.tokens() {
            self.check_with_tokens(&token_analysis.tokens, token_analysis)
        } else {
            self.check_impl(content, "")
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

    #[test]
    fn test_key_duplicates_rule_default() {
        let rule = KeyDuplicatesRule::new();
        assert_eq!(rule.rule_id(), "key-duplicates");
        assert_eq!(rule.default_severity(), Severity::Error);
        assert!(rule.is_enabled_by_default());
        assert!(!rule.can_fix());
    }

    #[test]
    fn test_key_duplicates_check_no_duplicates() {
        let rule = KeyDuplicatesRule::new();
        let content = "key1: value1\nkey2: value2\nkey3: value3";
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_key_duplicates_check_with_duplicates() {
        let rule = KeyDuplicatesRule::new();
        let content = "key1: value1\nkey2: value2\nkey1: value3";
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("duplication of key \"key1\""));
    }

    #[test]
    fn test_key_duplicates_check_nested_duplicates() {
        let rule = KeyDuplicatesRule::new();
        let content = r#"parent:
  key1: value1
  key2: value2
  key1: value3"#;
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("duplication of key \"key1\""));
    }

    #[test]
    fn test_key_duplicates_check_different_levels() {
        let rule = KeyDuplicatesRule::new();
        let content = r#"level1:
  key1: value1
level2:
  key1: value2"#;
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_key_duplicates_check_merge_keys() {
        let rule = KeyDuplicatesRule::new();
        let content = r#"anchor1: &anchor1
  key1: value1
anchor2: &anchor2
  key2: value2
merged:
  <<: *anchor1
  <<: *anchor2"#;
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_key_duplicates_check_merge_keys_forbidden() {
        let mut config = KeyDuplicatesConfig::default();
        config.forbid_duplicated_merge_keys = true;
        let rule = KeyDuplicatesRule::with_config(config);

        let content = r#"anchor1: &anchor1
  key1: value1
anchor2: &anchor2
  key2: value2
merged:
  <<: *anchor1
  <<: *anchor2"#;
        let issues = rule.check(content, "test.yaml");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("duplication of key \"<<\""));
    }

    #[test]
    fn test_key_duplicates_check_list_structure() {
        let rule = KeyDuplicatesRule::new();
        let content = r#"- name: item1
  value: 1
- name: item2
  value: 2"#;
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_key_duplicates_check_complex_nested() {
        let rule = KeyDuplicatesRule::new();
        let content = r#"outer:
  inner1:
    key1: value1
    key2: value2
  inner2:
    key1: value3
    key3: value4"#;
        let issues = rule.check(content, "test.yaml");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_key_duplicates_fix_no_changes() {
        let rule = KeyDuplicatesRule::new();
        let content = "key1: value1\nkey2: value2\nkey1: value3";
        let fix_result = rule.fix(content, "test.yaml");
        assert!(!fix_result.changed);
        assert_eq!(fix_result.fixes_applied, 0);
    }

    #[test]
    fn test_key_duplicates_false_positive_bug() {
        let rule = KeyDuplicatesRule::new();
        let content = r#"ProfileInformation,Requests,Request,Result,VerticalScreen:AdjudicationResult:
  containing_entity: ExtensionResult
  custom: true
  isInKey: false
  type: String
ProfileInformation,Requests,Request,Result,VerticalScreen:DetailsUrl:
  containing_entity: ExtensionResult
  custom: true
  isInKey: false
  type: String
ProfileInformation,Requests,Request,Result,VerticalScreen:Discrepancies:
  containing_entity: ExtensionResult
  custom: true
  isInKey: false
  type: Long"#;

        let issues = rule.check(content, "test.yaml");

        assert!(
            issues.is_empty(),
            "No key duplication violations should be detected. Current false positives: {:?}",
            issues
        );
    }
}
