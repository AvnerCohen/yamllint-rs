#[macro_export]
macro_rules! create_rule {
    (
        $rule_name:ident,
        $config_type:ty,
        $rule_id:literal,
        $rule_display_name:literal,
        $rule_description:literal,
        $default_severity:expr,
        $can_fix:literal
    ) => {
        #[derive(Debug, Clone)]
        pub struct $rule_name {
            base: $crate::rules::base::BaseRule<$config_type>,
        }

        impl $rule_name {
            pub fn new() -> Self {
                Self {
                    base: $crate::rules::base::BaseRule::new(<$config_type>::default()),
                }
            }

            pub fn with_config(config: $config_type) -> Self {
                Self {
                    base: $crate::rules::base::BaseRule::new(config),
                }
            }

            pub fn config(&self) -> &$config_type {
                self.base.config()
            }

            pub fn set_config(&mut self, config: $config_type) {
                self.base.set_config(config);
            }

            pub fn get_severity(&self) -> $crate::Severity {
                self.base.get_severity(self.default_severity())
            }

            pub fn set_severity(&mut self, severity: $crate::Severity) {
                self.base.set_severity(severity);
            }

            pub fn has_severity_override(&self) -> bool {
                self.base.has_severity_override()
            }

            pub fn create_issue(
                &self,
                line: usize,
                column: usize,
                message: String,
            ) -> $crate::LintIssue {
                $crate::LintIssue {
                    line,
                    column,
                    message,
                    severity: self.get_severity(),
                }
            }
        }

        impl Default for $rule_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::rules::Rule for $rule_name {
            fn rule_id(&self) -> &'static str {
                $rule_id
            }

            fn rule_name(&self) -> &'static str {
                $rule_display_name
            }

            fn rule_description(&self) -> &'static str {
                $rule_description
            }

            fn default_severity(&self) -> $crate::Severity {
                $default_severity
            }

            fn get_severity(&self) -> $crate::Severity {
                self.base.get_severity(self.default_severity())
            }

            fn set_severity(&mut self, severity: $crate::Severity) {
                self.base.set_severity(severity);
            }

            fn has_severity_override(&self) -> bool {
                self.base.has_severity_override()
            }

            fn can_fix(&self) -> bool {
                $can_fix
            }

            fn check(&self, content: &str, file_path: &str) -> Vec<$crate::LintIssue> {
                self.check_impl(content, file_path)
            }

            fn check_with_analysis(
                &self,
                content: &str,
                _file_path: &str,
                _analysis: &$crate::analysis::ContentAnalysis,
            ) -> Vec<$crate::LintIssue> {
                self.check_impl(content, _file_path)
            }
        }
    };
}

#[macro_export]
macro_rules! create_regex_rule {
    (
        $rule_name:ident,
        $config_type:ty,
        $rule_id:literal,
        $rule_display_name:literal,
        $rule_description:literal,
        $default_severity:expr,
        $can_fix:literal
    ) => {
        #[derive(Debug, Clone)]
        pub struct $rule_name {
            base: BaseRuleWithRegex<$config_type>,
        }

        impl $rule_name {
            pub fn new() -> Self {
                Self {
                    base: BaseRuleWithRegex::new(<$config_type>::default()),
                }
            }

            pub fn with_config(config: $config_type) -> Self {
                Self {
                    base: BaseRuleWithRegex::new(config),
                }
            }

            pub fn config(&self) -> &$config_type {
                self.base.config()
            }

            pub fn set_config(&mut self, config: $config_type) {
                self.base.set_config(config);
            }

            pub fn get_or_compile_pattern(
                &mut self,
                pattern: &str,
            ) -> Result<&regex::Regex, regex::Error> {
                self.base.get_or_compile_pattern(pattern)
            }
        }

        impl Default for $rule_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::rules::Rule for $rule_name {
            fn rule_id(&self) -> &'static str {
                $rule_id
            }

            fn rule_name(&self) -> &'static str {
                $rule_display_name
            }

            fn rule_description(&self) -> &'static str {
                $rule_description
            }

            fn default_severity(&self) -> Severity {
                $default_severity
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
                $can_fix
            }
        }
    };
}

#[macro_export]
macro_rules! create_issue {
    ($line:expr, $column:expr, $message:expr, $severity:expr) => {
        LintIssue {
            line: $line,
            column: $column,
            message: $message,
            severity: $severity,
        }
    };
}

#[macro_export]
macro_rules! create_fix_result {
    ($content:expr, $changed:expr, $fixes_applied:expr) => {
        $crate::rules::FixResult {
            content: $content,
            changed: $changed,
            fixes_applied: $fixes_applied,
        }
    };
}
