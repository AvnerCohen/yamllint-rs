use crate::Severity;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RuleMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub default_severity: Severity,
    pub can_fix: bool,
    pub enabled_by_default: bool,
    pub fix_order: Option<usize>,
    pub dependencies: Vec<&'static str>,
}

pub struct RuleRegistry {
    metadata: HashMap<String, RuleMetadata>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            metadata: HashMap::new(),
        };

        registry.register_all_rules();
        registry
    }

    fn register_all_rules(&mut self) {
        self.register_rule(RuleMetadata {
            id: "line-length",
            name: "Line Length",
            description: "Ensures that lines do not exceed the maximum allowed length",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "trailing-spaces",
            name: "Trailing Spaces",
            description: "Ensures that lines do not have trailing whitespace",
            default_severity: Severity::Error,
            can_fix: true,
            enabled_by_default: true,
            fix_order: Some(10),
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "comments",
            name: "Comments",
            description: "Checks comment formatting",
            default_severity: Severity::Warning,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "truthy",
            name: "Truthy",
            description: "Checks that truthy values are properly formatted",
            default_severity: Severity::Warning,
            can_fix: true,
            enabled_by_default: true,
            fix_order: Some(10),
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "comments-indentation",
            name: "Comments Indentation",
            description: "Checks comment indentation",
            default_severity: Severity::Warning,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "new-line-at-end-of-file",
            name: "New Line At End Of File",
            description: "Ensures files end with a newline",
            default_severity: Severity::Error,
            can_fix: true,
            enabled_by_default: true,
            fix_order: Some(100),
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "braces",
            name: "Braces",
            description: "Checks brace formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "brackets",
            name: "Brackets",
            description: "Checks bracket formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "colons",
            name: "Colons",
            description: "Checks colon formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "commas",
            name: "Commas",
            description: "Checks comma formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "hyphens",
            name: "Hyphens",
            description: "Checks hyphen formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "quoted-strings",
            name: "Quoted Strings",
            description: "Checks quoted string formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: false,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "indentation",
            name: "Indentation",
            description: "Checks indentation",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "document-start",
            name: "Document Start",
            description: "Checks document start marker",
            default_severity: Severity::Warning,
            can_fix: true,
            enabled_by_default: true,
            fix_order: Some(1),
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "document-end",
            name: "Document End",
            description: "Checks document end marker",
            default_severity: Severity::Warning,
            can_fix: true,
            enabled_by_default: false,
            fix_order: Some(100),
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "empty-values",
            name: "Empty Values",
            description: "Checks for empty values",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: false,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "float-values",
            name: "Float Values",
            description: "Checks float value formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: false,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "octal-values",
            name: "Octal Values",
            description: "Checks octal value formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: false,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "key-duplicates",
            name: "Key Duplicates",
            description: "Checks for duplicate keys",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "key-ordering",
            name: "Key Ordering",
            description: "Checks key ordering",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: false,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "empty-lines",
            name: "Empty Lines",
            description: "Checks empty line formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "anchors",
            name: "Anchors",
            description: "Checks anchor formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });

        self.register_rule(RuleMetadata {
            id: "new-lines",
            name: "New Lines",
            description: "Checks new line formatting",
            default_severity: Severity::Error,
            can_fix: false,
            enabled_by_default: true,
            fix_order: None,
            dependencies: vec![],
        });
    }

    fn register_rule(&mut self, metadata: RuleMetadata) {
        self.metadata.insert(metadata.id.to_string(), metadata);
    }

    pub fn get_rule_metadata(&self, rule_id: &str) -> Option<&RuleMetadata> {
        self.metadata.get(rule_id)
    }

    pub fn get_rule_ids(&self) -> Vec<String> {
        self.metadata.keys().cloned().collect()
    }

    pub fn get_default_enabled_rules(&self) -> Vec<String> {
        self.metadata
            .iter()
            .filter(|(_, metadata)| metadata.enabled_by_default)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn get_fixable_rules(&self) -> Vec<String> {
        self.metadata
            .iter()
            .filter(|(_, metadata)| metadata.can_fix)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
