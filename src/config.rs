//! Configuration system for all rules.

use crate::Severity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Rule-specific configurations
    pub rules: HashMap<String, RuleConfig>,
    /// Global settings
    pub global: GlobalConfig,
}

/// Global configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Default severity for rules
    pub default_severity: Option<Severity>,
    /// Whether to enable all rules by default
    pub enable_all_rules: Option<bool>,
    /// Whether to enable fix mode by default
    pub enable_fix_mode: Option<bool>,
}

/// Configuration for individual rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Whether this rule is enabled
    pub enabled: Option<bool>,
    /// Severity override for this rule
    pub severity: Option<Severity>,
    /// Rule-specific settings
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
    /// Rule-specific settings (deprecated, use other)
    pub settings: Option<serde_json::Value>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: None,
            severity: None,
            other: serde_json::Map::new(),
            settings: None,
        }
    }
}

/// Rule-specific configuration structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineLengthConfig {
    pub max_length: usize,
    /// Allow non-breakable words (without spaces) to overflow the limit
    pub allow_non_breakable_words: bool,
    /// Allow non-breakable inline mappings (key: value where value has no spaces)
    #[serde(default)]
    pub allow_non_breakable_inline_mappings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndentationConfig {
    pub spaces: Option<usize>,
    pub indent_sequences: Option<bool>,
    pub check_multi_line_strings: Option<bool>,
    pub ignore: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsConfig {
    pub min_spaces_from_content: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthyConfig {
    pub allowed_values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailingSpacesConfig {
    pub allow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStartConfig {
    pub present: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEndConfig {
    pub present: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyLinesConfig {
    pub max: Option<usize>,
    pub max_start: Option<usize>,
    pub max_end: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyOrderingConfig {
    pub order: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorsConfig {
    pub max_length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLinesConfig {
    pub type_: Option<String>, // "unix" or "dos"
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        let mut config = Self {
            rules: HashMap::new(),
            global: GlobalConfig {
                default_severity: Some(Severity::Error),
                enable_all_rules: Some(true),
                enable_fix_mode: Some(false),
            },
        };

        // Set up default rule configurations
        config.setup_default_rules();
        config
    }

    /// Set up default rule configurations
    fn setup_default_rules(&mut self) {
        // Line length rule
        self.rules.insert(
            "line-length".to_string(),
            RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Error),
                settings: Some(
                    serde_json::to_value(LineLengthConfig {
                        max_length: 80,
                        allow_non_breakable_words: true,
                        allow_non_breakable_inline_mappings: false,
                    })
                    .unwrap(),
                ),
                ..Default::default()
            },
        );

        // Trailing spaces rule
        self.rules.insert(
            "trailing-spaces".to_string(),
            RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Error),
                settings: Some(
                    serde_json::to_value(TrailingSpacesConfig { allow: false }).unwrap(),
                ),
                ..Default::default()
            },
        );

        // Comments rule
        self.rules.insert(
            "comments".to_string(),
            RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Warning),
                settings: Some(
                    serde_json::to_value(CommentsConfig {
                        min_spaces_from_content: Some(2),
                    })
                    .unwrap(),
                ),
                ..Default::default()
            },
        );

        // Truthy rule
        self.rules.insert(
            "truthy".to_string(),
            RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Warning), // Changed from Error to Warning to match yamllint default
                settings: Some(
                    serde_json::to_value(TruthyConfig {
                        allowed_values: vec!["false".to_string(), "true".to_string()],
                    })
                    .unwrap(),
                ),
                ..Default::default()
            },
        );

        // Comments indentation rule
        self.rules.insert(
            "comments-indentation".to_string(),
            RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Warning),
                settings: None,
                ..Default::default()
            },
        );

        // New line at end of file rule
        self.rules.insert(
            "new-line-at-end-of-file".to_string(),
            RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Error),
                settings: None,
                ..Default::default()
            },
        );

        // Document start rule
        self.rules.insert(
            "document-start".to_string(),
            RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Warning),
                settings: Some(
                    serde_json::to_value(DocumentStartConfig {
                        present: Some(true),
                    })
                    .unwrap(),
                ),
                ..Default::default()
            },
        );

        // Document end rule
        self.rules.insert(
            "document-end".to_string(),
            RuleConfig {
                enabled: Some(false), // Changed from true to false to match yamllint default
                severity: Some(Severity::Warning),
                settings: Some(
                    serde_json::to_value(DocumentEndConfig {
                        present: Some(true),
                    })
                    .unwrap(),
                ),
                ..Default::default()
            },
        );

        // Indentation rule with default settings
        self.rules.insert(
            "indentation".to_string(),
            RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Error),
                settings: Some(
                    serde_json::to_value(IndentationConfig {
                        spaces: Some(2),
                        indent_sequences: Some(true),
                        check_multi_line_strings: Some(false),
                        ignore: None,
                    })
                    .unwrap(),
                ),
                ..Default::default()
            },
        );

        // Rules that are enabled by default in yamllint
        let enabled_rules = vec![
            "braces",
            "brackets",
            "colons",
            "hyphens",
            "key-duplicates",
            "empty-lines",
            "anchors",
            "new-lines",
        ];

        for rule_id in enabled_rules {
            self.rules.insert(
                rule_id.to_string(),
                RuleConfig {
                    enabled: Some(true),
                    severity: Some(Severity::Error),
                    settings: None,
                    ..Default::default()
                },
            );
        }

        // Rules that are disabled by default in yamllint
        let disabled_rules = vec![
            "quoted-strings",
            "empty-values",
            "float-values",
            "octal-values",
            "key-ordering",
        ];

        for rule_id in disabled_rules {
            self.rules.insert(
                rule_id.to_string(),
                RuleConfig {
                    enabled: Some(false), // Disabled to match yamllint default
                    severity: Some(Severity::Error),
                    settings: None,
                    ..Default::default()
                },
            );
        }
    }

    /// Get configuration for a specific rule
    pub fn get_rule_config(&self, rule_id: &str) -> Option<&RuleConfig> {
        self.rules.get(rule_id)
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        self.rules
            .get(rule_id)
            .and_then(|config| config.enabled)
            .unwrap_or(self.global.enable_all_rules.unwrap_or(true))
    }

    /// Get severity for a rule
    pub fn get_rule_severity(&self, rule_id: &str) -> Severity {
        self.rules
            .get(rule_id)
            .and_then(|config| config.severity)
            .unwrap_or(self.global.default_severity.unwrap_or(Severity::Error))
    }

    /// Get rule-specific settings
    pub fn get_rule_settings<T>(&self, rule_id: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.rules
            .get(rule_id)
            .and_then(|config| config.settings.as_ref())
            .and_then(|settings| serde_json::from_value(settings.clone()).ok())
    }

    /// Enable or disable a rule
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) {
        self.rules
            .entry(rule_id.to_string())
            .or_insert_with(|| RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Error),
                settings: None,
                ..Default::default()
            })
            .enabled = Some(enabled);
    }

    /// Set severity for a rule
    pub fn set_rule_severity(&mut self, rule_id: &str, severity: Severity) {
        self.rules
            .entry(rule_id.to_string())
            .or_insert_with(|| RuleConfig {
                enabled: Some(true),
                severity: Some(Severity::Error),
                settings: None,
                ..Default::default()
            })
            .severity = Some(severity);
    }

    /// Get all enabled rule IDs
    pub fn get_enabled_rules(&self) -> Vec<String> {
        self.rules
            .iter()
            .filter(|(_, config)| config.enabled.unwrap_or(true))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all disabled rule IDs
    pub fn get_disabled_rules(&self) -> Vec<String> {
        self.rules
            .iter()
            .filter(|(_, config)| config.enabled.unwrap_or(true) == false)
            .map(|(id, _)| id.clone())
            .collect()
    }
}
