use crate::rules::*;
use crate::rules::{registry::RuleRegistry, Rule};

pub struct RuleFactory {
    registry: RuleRegistry,
}

impl RuleFactory {
    pub fn new() -> Self {
        Self {
            registry: RuleRegistry::new(),
        }
    }

    pub fn create_rule(&self, rule_id: &str) -> Option<Box<dyn Rule>> {
        match rule_id {
            "line-length" => Some(Box::new(LineLengthRule::new())),
            "trailing-spaces" => Some(Box::new(TrailingSpacesRule::new())),
            "comments" => Some(Box::new(CommentsRule::new())),
            "truthy" => Some(Box::new(TruthyRule::new())),
            "comments-indentation" => Some(Box::new(CommentsIndentationRule::new())),
            "new-line-at-end-of-file" => Some(Box::new(NewLineAtEndOfFileRule::new())),
            "braces" => Some(Box::new(BracesRule::new())),
            "brackets" => Some(Box::new(BracketsRule::new())),
            "colons" => Some(Box::new(ColonsRule::new())),
            "commas" => Some(Box::new(CommasRule::new())),
            "hyphens" => Some(Box::new(HyphensRule::new())),
            "quoted-strings" => Some(Box::new(QuotedStringsRule::new())),
            "indentation" => Some(Box::new(IndentationRule::new())),
            "document-start" => Some(Box::new(DocumentStartRule::new())),
            "document-end" => Some(Box::new(DocumentEndRule::new())),
            "empty-values" => Some(Box::new(EmptyValuesRule::new())),
            "float-values" => Some(Box::new(FloatValuesRule::new())),
            "octal-values" => Some(Box::new(OctalValuesRule::new())),
            "key-duplicates" => Some(Box::new(KeyDuplicatesRule::new())),
            "key-ordering" => Some(Box::new(KeyOrderingRule::new())),
            "empty-lines" => Some(Box::new(EmptyLinesRule::new())),
            "anchors" => Some(Box::new(AnchorsRule::new())),
            "new-lines" => Some(Box::new(NewLinesRule::new())),
            _ => None,
        }
    }

    pub fn create_default_rules(&self) -> Vec<Box<dyn Rule>> {
        let default_rule_ids = self.registry.get_default_enabled_rules();
        default_rule_ids
            .iter()
            .filter_map(|id| self.create_rule(id))
            .collect()
    }

    pub fn create_rules_by_ids(&self, rule_ids: &[String]) -> Vec<Box<dyn Rule>> {
        rule_ids
            .iter()
            .filter_map(|id| self.create_rule(id))
            .collect()
    }

    pub fn create_rules_by_ids_with_config(
        &self,
        rule_ids: &[String],
        config: &crate::config::Config,
    ) -> Vec<Box<dyn Rule>> {
        rule_ids
            .iter()
            .filter_map(|id| self.create_rule_with_config(id, config))
            .collect()
    }

    fn create_line_length_rule_with_config(&self, config: &crate::config::Config) -> Box<dyn Rule> {
        let mut rule = LineLengthRule::new();
        if let Some(line_config) =
            config.get_rule_settings::<crate::config::LineLengthConfig>("line-length")
        {
            rule.set_config(crate::rules::line_length::LineLengthConfig {
                max_length: line_config.max_length,
                allow_non_breakable_words: line_config.allow_non_breakable_words,
                allow_non_breakable_inline_mappings: line_config
                    .allow_non_breakable_inline_mappings,
            });
        }
        Box::new(rule)
    }

    fn create_indentation_rule_with_config(&self, config: &crate::config::Config) -> Box<dyn Rule> {
        let mut rule = IndentationRule::new();

        let indent_config = config
            .get_rule_settings::<crate::config::IndentationConfig>("indentation")
            .or_else(|| {
                config.rules.get("indentation").and_then(|rule_config| {
                    let mut spaces = None;
                    let mut indent_sequences = None;
                    let mut check_multi_line_strings = None;
                    let mut ignore = None;

                    if let Some(spaces_val) =
                        rule_config.other.get("spaces").and_then(|v| v.as_u64())
                    {
                        spaces = Some(spaces_val as usize);
                    }
                    if let Some(indent_val) = rule_config.other.get("indent-sequences") {
                        if let Some(bool_val) = indent_val.as_bool() {
                            indent_sequences = Some(bool_val);
                        }
                    }
                    if let Some(check_val) = rule_config
                        .other
                        .get("check-multi-line-strings")
                        .and_then(|v| v.as_bool())
                    {
                        check_multi_line_strings = Some(check_val);
                    }
                    if let Some(ignore_val) = rule_config.other.get("ignore") {
                        if let Some(s) = ignore_val.as_str() {
                            ignore = Some(s.to_string());
                        }
                    }

                    Some(crate::config::IndentationConfig {
                        spaces,
                        indent_sequences,
                        check_multi_line_strings,
                        ignore,
                    })
                })
            });

        if let Some(indent_config) = indent_config {
            rule.set_config(crate::rules::indentation::IndentationConfig {
                spaces: indent_config.spaces.unwrap_or(2),
                indent_sequences: indent_config.indent_sequences.unwrap_or(true),
                check_multi_line_strings: indent_config.check_multi_line_strings.unwrap_or(false),
                ignore_patterns: crate::rules::indentation::IndentationRule::parse_ignore_patterns(
                    indent_config.ignore,
                ),
            });
        }
        Box::new(rule)
    }

    pub fn create_rule_with_config(
        &self,
        rule_id: &str,
        config: &crate::config::Config,
    ) -> Option<Box<dyn Rule>> {
        match rule_id {
            "line-length" => Some(self.create_line_length_rule_with_config(config)),
            "indentation" => Some(self.create_indentation_rule_with_config(config)),
            "trailing-spaces" => {
                let mut rule = TrailingSpacesRule::new();
                let allow = config
                    .get_rule_settings::<crate::config::TrailingSpacesConfig>("trailing-spaces")
                    .map(|c| c.allow)
                    .unwrap_or(false);
                rule.set_config(crate::rules::trailing_spaces::TrailingSpacesConfig { allow });
                Some(Box::new(rule))
            }
            _ => self.create_rule(rule_id),
        }
    }

    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }
}

impl Default for RuleFactory {
    fn default() -> Self {
        Self::new()
    }
}
