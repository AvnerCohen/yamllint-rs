use crate::LintIssue;

pub mod base;
pub mod factory;
pub mod macros;
pub mod registry;

#[derive(Debug, Clone)]
pub struct FixResult {
    pub content: String,
    pub changed: bool,
    pub fixes_applied: usize,
}

pub trait Rule: Send + Sync {
    fn rule_id(&self) -> &'static str;
    fn rule_name(&self) -> &'static str;
    fn rule_description(&self) -> &'static str;
    fn default_severity(&self) -> crate::Severity;
    fn get_severity(&self) -> crate::Severity;
    fn set_severity(&mut self, severity: crate::Severity);
    fn has_severity_override(&self) -> bool;

    fn check(&self, content: &str, file_path: &str) -> Vec<LintIssue>;

    fn check_with_analysis(
        &self,
        content: &str,
        file_path: &str,
        _analysis: &crate::analysis::ContentAnalysis,
    ) -> Vec<LintIssue> {
        self.check(content, file_path)
    }

    fn is_enabled_by_default(&self) -> bool {
        true
    }

    fn can_fix(&self) -> bool {
        false
    }

    fn fix(&self, content: &str, _file_path: &str) -> FixResult {
        FixResult {
            content: content.to_string(),
            changed: false,
            fixes_applied: 0,
        }
    }
}

pub mod anchors;
pub mod braces;
pub mod brackets;
pub mod colons;
pub mod commas;
pub mod comments;
pub mod comments_indentation;
pub mod document_end;
pub mod document_start;
pub mod empty_lines;
pub mod empty_values;
pub mod float_values;
pub mod hyphens;
pub mod indentation;
pub mod key_duplicates;
pub mod key_ordering;
pub mod line_length;
pub mod new_line_at_end_of_file;
pub mod new_lines;
pub mod octal_values;
pub mod quoted_strings;
pub mod trailing_spaces;
pub mod truthy;

pub use anchors::AnchorsRule;
pub use braces::BracesRule;
pub use brackets::BracketsRule;
pub use colons::ColonsRule;
pub use commas::CommasRule;
pub use comments::CommentsRule;
pub use comments_indentation::CommentsIndentationRule;
pub use document_end::DocumentEndRule;
pub use document_start::DocumentStartRule;
pub use empty_lines::EmptyLinesRule;
pub use empty_values::EmptyValuesRule;
pub use float_values::FloatValuesRule;
pub use hyphens::HyphensRule;
pub use indentation::IndentationRule;
pub use key_duplicates::KeyDuplicatesRule;
pub use key_ordering::KeyOrderingRule;
pub use line_length::LineLengthRule;
pub use new_line_at_end_of_file::NewLineAtEndOfFileRule;
pub use new_lines::NewLinesRule;
pub use octal_values::OctalValuesRule;
pub use quoted_strings::QuotedStringsRule;
pub use trailing_spaces::TrailingSpacesRule;
pub use truthy::TruthyRule;
