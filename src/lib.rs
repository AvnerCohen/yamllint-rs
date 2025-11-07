use anyhow::Result;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub mod analysis;
pub mod config;
pub mod directives;
pub mod formatter;
pub mod rule_pool;
pub mod rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Standard,
    Colored,
}

#[derive(Debug, Clone)]
pub struct ProcessingOptions {
    pub recursive: bool,
    pub verbose: bool,
    pub output_format: OutputFormat,
    pub show_progress: bool,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            recursive: false,
            verbose: false,
            output_format: OutputFormat::Colored,
            show_progress: true,
        }
    }
}

pub fn detect_output_format(format_str: &str) -> OutputFormat {
    match format_str {
        "standard" => OutputFormat::Standard,
        "colored" => OutputFormat::Colored,
        "auto" | _ => {
            if std::env::var("NO_COLOR").is_ok() {
                return OutputFormat::Standard;
            }

            if !atty::is(atty::Stream::Stdout) {
                return OutputFormat::Standard;
            }

            OutputFormat::Colored
        }
    }
}

pub struct FileProcessor {
    options: ProcessingOptions,
    rules: Arc<Vec<Box<dyn rules::Rule>>>,
    fix_mode: bool,
    config: Option<Arc<config::Config>>,
    formatter: Box<dyn formatter::Formatter>,
}

impl FileProcessor {
    fn should_run_rule_for_file(
        rule_id: &str,
        file_path: &str,
        config: &Option<Arc<config::Config>>,
    ) -> bool {
        if let Some(config) = config {
            if let Some(rule_config) = config.get_rule_config(rule_id) {
                if let Some(ignore_val) = rule_config.other.get("ignore") {
                    if let Some(ignore_str) = ignore_val.as_str() {
                        let patterns: Vec<&str> = ignore_str
                            .lines()
                            .map(|line| line.trim())
                            .filter(|line| !line.is_empty())
                            .collect();

                        for pattern in patterns {
                            if file_path.contains(pattern) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }

    pub fn new(options: ProcessingOptions) -> Self {
        let formatter = formatter::create_formatter(options.output_format);
        Self {
            options,
            rules: Arc::new(Vec::new()),
            fix_mode: false,
            config: None,
            formatter,
        }
    }

    pub fn with_default_rules(options: ProcessingOptions) -> Self {
        let factory = rules::factory::RuleFactory::new();
        let config = config::Config::default();
        let enabled_rules = config.get_enabled_rules();
        let mut rules = factory.create_rules_by_ids_with_config(&enabled_rules, &config);
        let config_arc = Arc::new(config);

        for rule in &mut rules {
            let severity = config_arc.get_rule_severity(rule.rule_id());
            rule.set_severity(severity);
        }

        let formatter = formatter::create_formatter(options.output_format);
        Self {
            options,
            rules: Arc::new(rules),
            fix_mode: false,
            config: Some(config_arc),
            formatter,
        }
    }

    pub fn with_fix_mode(options: ProcessingOptions) -> Self {
        let mut processor = Self::with_default_rules(options);
        processor.fix_mode = true;
        processor
    }

    pub fn with_config(options: ProcessingOptions, config: config::Config) -> Self {
        let factory = rules::factory::RuleFactory::new();
        let enabled_rules = config.get_enabled_rules();

        let config_arc = Arc::new(config);
        let mut rules = factory.create_rules_by_ids_with_config(&enabled_rules, &config_arc);

        for rule in &mut rules {
            let severity = config_arc.get_rule_severity(rule.rule_id());
            rule.set_severity(severity);
        }

        let formatter = formatter::create_formatter(options.output_format);
        Self {
            options,
            rules: Arc::new(rules),
            fix_mode: false,
            config: Some(config_arc),
            formatter,
        }
    }

    pub fn with_config_and_fix_mode(options: ProcessingOptions, config: config::Config) -> Self {
        let mut processor = Self::with_config(options, config);
        processor.fix_mode = true;
        processor
    }

    pub fn add_rule(&mut self, rule: Box<dyn rules::Rule>) {
        Arc::get_mut(&mut self.rules)
            .expect("Cannot add rule when rules are shared")
            .push(rule);
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<LintResult> {
        let path = file_path.as_ref();

        if let Some(config) = &self.config {
            let cwd = std::env::current_dir().ok();
            let config_dir = cwd.as_deref();
            if config.is_file_ignored(path, config_dir) {
                return Ok(LintResult {
                    file: self.get_relative_path(path),
                    issues: vec![],
                });
            }
        }

        let relative_path = self.get_relative_path(path);

        if self.options.verbose {
            println!("Processing file: {}", relative_path);
        }

        let content = std::fs::read_to_string(path)?;

        if self.fix_mode {
            self.process_file_with_fixes(path, &content, &relative_path)
        } else {
            self.process_file_check_only(&content, &relative_path)
        }
    }

    fn check_file_content(
        rules: &[Box<dyn rules::Rule>],
        content: &str,
        relative_path: &str,
        config: &Option<Arc<config::Config>>,
    ) -> LintResult {
        let all_rule_ids: std::collections::HashSet<String> =
            rules.iter().map(|r| r.rule_id().to_string()).collect();
        let mut directive_state = directives::DirectiveState::new(all_rule_ids);
        directive_state.parse_from_content(content);

        let analysis = analysis::ContentAnalysis::analyze(content);

        let estimated_issues = rules.len() * 3;
        let mut all_issues = Vec::with_capacity(estimated_issues);
        for rule in rules {
            let rule_id = rule.rule_id();
            if !Self::should_run_rule_for_file(rule_id, relative_path, config) {
                continue;
            }
            let issues = rule.check_with_analysis(content, relative_path, &analysis);
            for issue in issues {
                all_issues.push((issue, rule_id.to_string()));
            }
        }

        let filtered_issues = directive_state.filter_issues(all_issues);
        let mut sorted_issues = filtered_issues;
        sorted_issues.sort_by(|a, b| a.0.line.cmp(&b.0.line).then(a.0.column.cmp(&b.0.column)));

        LintResult {
            file: relative_path.to_string(),
            issues: sorted_issues,
        }
    }

    fn process_file_check_only(&self, content: &str, relative_path: &str) -> Result<LintResult> {
        let result =
            Self::check_file_content(self.rules.as_slice(), content, relative_path, &self.config);

        if result.issues.is_empty() {
            if self.options.verbose {
                println!("✓ No issues found in {}", result.file);
            }
        } else {
            println!("{}", self.formatter.format_filename(&result.file));

            let mut output = String::with_capacity(result.issues.len() * 120);

            for (issue, rule_name) in &result.issues {
                let formatted = self.formatter.format_issue(issue, rule_name);
                output.push_str(&formatted);
            }

            print!("{}", output);
        }

        Ok(result)
    }

    fn apply_fixes_and_check(
        rules: &[Box<dyn rules::Rule>],
        content: &str,
        relative_path: &str,
        config: &Option<Arc<config::Config>>,
    ) -> (String, usize, usize, Vec<(LintIssue, String)>) {
        let registry = rules::registry::RuleRegistry::new();
        let mut fixed_content = String::with_capacity(content.len());
        fixed_content.push_str(content);
        let mut total_fixes = 0;
        let mut fixable_issues = 0;

        let mut fixable_rules: Vec<(usize, usize)> = rules
            .iter()
            .enumerate()
            .filter_map(|(idx, rule)| {
                let rule_id = rule.rule_id();
                if !Self::should_run_rule_for_file(rule_id, relative_path, config) {
                    return None;
                }
                if !rule.can_fix() {
                    return None;
                }
                let metadata = registry.get_rule_metadata(rule_id)?;
                let order = metadata.fix_order.unwrap_or(999);
                Some((idx, order))
            })
            .collect();

        fixable_rules.sort_by_key(|(_, order)| *order);

        for (idx, _) in fixable_rules {
            let rule = &rules[idx];
            let fix_result = rule.fix(&fixed_content, relative_path);
            if fix_result.changed || fix_result.fixes_applied > 0 {
                fixed_content = fix_result.content;
                total_fixes += fix_result.fixes_applied;
                fixable_issues += fix_result.fixes_applied;
            }
        }

        let analysis = analysis::ContentAnalysis::analyze(&fixed_content);
        let estimated_issues = rules.len() * 3;
        let mut all_issues = Vec::with_capacity(estimated_issues);
        for rule in rules {
            let rule_id = rule.rule_id();
            if !Self::should_run_rule_for_file(rule_id, relative_path, config) {
                continue;
            }
            let issues = rule.check_with_analysis(&fixed_content, relative_path, &analysis);
            for issue in issues {
                all_issues.push((issue, rule_id.to_string()));
            }
        }

        all_issues.sort_by(|a, b| a.0.line.cmp(&b.0.line).then(a.0.column.cmp(&b.0.column)));

        (fixed_content, total_fixes, fixable_issues, all_issues)
    }

    fn process_file_with_fixes<P: AsRef<Path>>(
        &self,
        path: P,
        content: &str,
        relative_path: &str,
    ) -> Result<LintResult> {
        let (fixed_content, total_fixes, fixable_issues, all_issues) = Self::apply_fixes_and_check(
            self.rules.as_slice(),
            content,
            relative_path,
            &self.config,
        );

        let _non_fixable_issues = all_issues.len();

        if fixed_content != content {
            std::fs::write(path, &fixed_content)?;
            if total_fixes > 0 {
                println!(
                    "Fixed {} issues in {} ({} fixable, {} remaining)",
                    total_fixes, relative_path, fixable_issues, _non_fixable_issues
                );
            }
        } else if _non_fixable_issues > 0 {
            println!(
                "Found {} non-fixable issues in {}:",
                _non_fixable_issues, relative_path
            );
            for (issue, _rule_name) in &all_issues {
                println!(
                    "  {}:{}: {}: {}",
                    issue.line,
                    issue.column,
                    format!("{:?}", issue.severity).to_lowercase(),
                    issue.message
                );
            }
        } else {
            if self.options.verbose {
                println!("✓ No issues found in {}", relative_path);
            }
        }

        Ok(LintResult {
            file: relative_path.to_string(),
            issues: all_issues,
        })
    }

    pub fn process_directory<P: AsRef<Path>>(&self, dir_path: P) -> Result<usize> {
        let path = dir_path.as_ref();

        if !path.is_dir() {
            return Err(anyhow::anyhow!(
                "Path is not a directory: {}",
                path.display()
            ));
        }

        if self.options.verbose {
            println!("Processing directory: {}", path.display());
        }

        let mut yaml_files = Vec::with_capacity(100);

        let walker = WalkBuilder::new(path).follow_links(false).build();

        for result in walker {
            let entry = result?;
            let file_path = entry.path();
            if file_path.is_file() && self.is_yaml_file(file_path) {
                if let Some(config) = &self.config {
                    let config_dir = Some(path);
                    if config.is_file_ignored(file_path, config_dir) {
                        continue;
                    }
                }
                yaml_files.push(file_path.to_path_buf());
            }
        }

        if yaml_files.is_empty() {
            if self.options.verbose {
                println!("No YAML files found in directory");
            }
            return Ok(0);
        }

        if self.options.verbose {
            println!(
                "Found {} YAML files, processing in parallel...",
                yaml_files.len()
            );
        }

        let options = self.options.clone();
        let fix_mode = self.fix_mode;
        let shared_rules = self.rules.clone();

        let results = if options.show_progress {
            let total = yaml_files.len();
            let counter = Arc::new(AtomicUsize::new(0));
            Self::process_files_list(
                &yaml_files,
                shared_rules,
                &options,
                fix_mode,
                &self.config,
                Some(counter),
                Some(total),
            )?
        } else {
            Self::process_files_list(
                &yaml_files,
                shared_rules,
                &options,
                fix_mode,
                &self.config,
                None,
                None,
            )?
        };

        let formatter = formatter::create_formatter(options.output_format);
        let mut stdout = std::io::stdout().lock();
        let mut total_issues = 0;
        for result in &results {
            if !result.issues.is_empty() {
                total_issues += result.issues.len();
                writeln!(stdout, "{}", formatter.format_filename(&result.file))?;

                let mut output = String::with_capacity(result.issues.len() * 120);

                for (issue, rule_name) in &result.issues {
                    let formatted = formatter.format_issue(issue, rule_name);
                    output.push_str(&formatted);
                }

                write!(stdout, "{}", output)?;
            }
        }

        if self.options.verbose {
            writeln!(stdout, "Successfully processed {} files", results.len())?;
        }

        if self.options.verbose {
            writeln!(stdout, "Completed processing {} files", yaml_files.len())?;
        }

        Ok(total_issues)
    }

    fn is_yaml_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_string_lossy().to_lowercase().as_str(),
                "yaml" | "yml"
            )
        } else {
            false
        }
    }

    fn get_relative_path(&self, path: &Path) -> String {
        Self::get_relative_path_static(path)
    }

    fn get_relative_path_static(path: &Path) -> String {
        if let Ok(cwd) = std::env::current_dir() {
            if let Ok(relative) = path.strip_prefix(&cwd) {
                return relative.to_string_lossy().to_string();
            }
        }
        path.to_string_lossy().to_string()
    }

    fn process_files_list(
        files: &[PathBuf],
        rules: Arc<Vec<Box<dyn rules::Rule>>>,
        options: &ProcessingOptions,
        fix_mode: bool,
        config: &Option<Arc<config::Config>>,
        counter: Option<Arc<AtomicUsize>>,
        total: Option<usize>,
    ) -> Result<Vec<LintResult>> {
        if files.len() > 3 {
            files
                .par_iter()
                .map(|file| {
                    Self::process_single_file(
                        rules.clone(),
                        file,
                        options,
                        fix_mode,
                        config,
                        counter.as_ref().map(Arc::clone),
                        total,
                    )
                })
                .collect()
        } else {
            files
                .iter()
                .map(|file| {
                    Self::process_single_file(
                        rules.clone(),
                        file,
                        options,
                        fix_mode,
                        config,
                        counter.as_ref().map(Arc::clone),
                        total,
                    )
                })
                .collect()
        }
    }

    fn process_single_file(
        rules: Arc<Vec<Box<dyn rules::Rule>>>,
        file_path: &Path,
        options: &ProcessingOptions,
        fix_mode: bool,
        config: &Option<Arc<config::Config>>,
        counter: Option<Arc<AtomicUsize>>,
        total: Option<usize>,
    ) -> Result<LintResult> {
        let relative_path = Self::get_relative_path_static(file_path);

        if options.verbose {
            eprintln!("Processing file: {}", relative_path);
        }

        let content = std::fs::read_to_string(file_path)?;

        let result = if fix_mode {
            Self::process_file_with_fixes_static(
                &rules,
                file_path,
                &content,
                &relative_path,
                config,
            )
        } else {
            Self::process_file_check_only_static(&rules, &content, &relative_path, config)
        }?;

        if let (Some(counter), Some(total)) = (counter, total) {
            let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
            if count % 1000 == 0 || count == total {
                let percent = (count * 100) / total;
                eprintln!(
                    "[Progress] Processed {}/{} files ({}%)",
                    count, total, percent
                );
            }
        }

        Ok(result)
    }

    fn process_file_check_only_static(
        rules: &[Box<dyn rules::Rule>],
        content: &str,
        relative_path: &str,
        config: &Option<Arc<config::Config>>,
    ) -> Result<LintResult> {
        let result = Self::check_file_content(rules, content, relative_path, config);
        Ok(result)
    }

    fn process_file_with_fixes_static(
        rules: &[Box<dyn rules::Rule>],
        path: &Path,
        content: &str,
        relative_path: &str,
        config: &Option<Arc<config::Config>>,
    ) -> Result<LintResult> {
        let (fixed_content, total_fixes, fixable_issues, all_issues) =
            Self::apply_fixes_and_check(rules, content, relative_path, config);

        let _non_fixable_issues = all_issues.len();

        if total_fixes > 0 {
            std::fs::write(path, &fixed_content)?;
            println!(
                "Fixed {} issues in {} ({} fixable, {} remaining)",
                total_fixes, relative_path, fixable_issues, _non_fixable_issues
            );
        } else if !all_issues.is_empty() {
            println!(
                "Found {} non-fixable issues in {}:",
                _non_fixable_issues, relative_path
            );
            for (issue, rule_name) in &all_issues {
                let level = match issue.severity {
                    crate::Severity::Error => "error",
                    crate::Severity::Warning => "warning",
                    crate::Severity::Info => "info",
                };
                println!(
                    "  {}:{}:{}: {} {} ({})",
                    relative_path, issue.line, issue.column, level, issue.message, rule_name
                );
            }
        }

        Ok(LintResult {
            file: relative_path.to_string(),
            issues: all_issues,
        })
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<config::Config> {
    let content = std::fs::read_to_string(path)?;

    match parse_original_yamllint_format(&content) {
        Ok(original_config) => return Ok(original_config),
        Err(e) => {
            if !e.to_string().contains("Not original yamllint format") {
                return Err(e);
            }
        }
    }

    let config: config::Config = serde_yaml::from_str(&content)?;
    Ok(config)
}

fn yaml_value_to_json(yaml_val: &serde_yaml::Value) -> serde_json::Value {
    match yaml_val {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                serde_json::Value::Number(u.into())
            } else if let Some(i) = n.as_i64() {
                serde_json::Value::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(yaml_value_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => serde_json::Value::Object(
            map.iter()
                .filter_map(|(k, v)| {
                    k.as_str()
                        .map(|key| (key.to_string(), yaml_value_to_json(v)))
                })
                .collect(),
        ),
        serde_yaml::Value::Tagged(_) => serde_json::Value::Null,
    }
}

fn parse_original_yamllint_format(content: &str) -> Result<config::Config> {
    use serde_yaml::Value;

    let yaml_value: Value = serde_yaml::from_str(content)?;

    let has_extends = yaml_value.get("extends").is_some();
    let has_rules_simple_format = yaml_value
        .get("rules")
        .and_then(|r| r.as_mapping())
        .map(|rules_map| {
            rules_map
                .values()
                .any(|v| v.is_string() || (v.is_mapping() && v.get("level").is_some()))
        })
        .unwrap_or(false);

    if has_extends {
        return convert_original_yamllint_config(yaml_value);
    }

    if has_rules_simple_format {
        if let Some(rules) = yaml_value.get("rules") {
            if let Some(rules_map) = rules.as_mapping() {
                let has_simple_values = rules_map
                    .values()
                    .any(|v| v.is_string() || (v.is_mapping() && v.get("level").is_some()));

                if has_simple_values {
                    return convert_original_yamllint_config(yaml_value);
                }
            }
        }
    }

    Err(anyhow::anyhow!("Not original yamllint format"))
}

fn convert_original_yamllint_config(yaml_value: serde_yaml::Value) -> Result<config::Config> {
    let mut config = config::Config::new();

    if let Some(ignore_val) = yaml_value.get("ignore") {
        if let Some(ignore_str) = ignore_val.as_str() {
            config.ignore = Some(ignore_str.to_string());
        } else if let Some(ignore_seq) = ignore_val.as_sequence() {
            let patterns: Vec<String> = ignore_seq
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            config.ignore = Some(patterns.join("\n"));
        }
    }

    if let Some(ignore_from_file_val) = yaml_value.get("ignore-from-file") {
        if let Some(ignore_file_str) = ignore_from_file_val.as_str() {
            config.ignore_from_file = Some(ignore_file_str.to_string());
        } else if let Some(ignore_file_seq) = ignore_from_file_val.as_sequence() {
            if let Some(first_file) = ignore_file_seq.first().and_then(|v| v.as_str()) {
                config.ignore_from_file = Some(first_file.to_string());
            }
        }
    }

    if let Some(rules) = yaml_value.get("rules").and_then(|r| r.as_mapping()) {
        for (rule_name, rule_config) in rules {
            let rule_name = rule_name.as_str().unwrap_or("");

            if let Some(rule_str) = rule_config.as_str() {
                match rule_str {
                    "disable" => {
                        config.set_rule_enabled(rule_name, false);
                    }
                    "enable" => {
                        config.set_rule_enabled(rule_name, true);
                    }
                    _ => {
                        config.set_rule_enabled(rule_name, true);
                    }
                }
            } else if let Some(rule_map) = rule_config.as_mapping() {
                let mut enabled = None;
                let mut severity = None;
                let mut settings: Option<serde_json::Value> = None;

                if let Some(enable_val) = rule_map.get("enable") {
                    enabled = enable_val.as_bool();
                }
                if let Some(disable_val) = rule_map.get("disable") {
                    if let Some(disable_bool) = disable_val.as_bool() {
                        enabled = Some(!disable_bool);
                    }
                }

                if let Some(level_val) = rule_map.get("level") {
                    if let Some(level_str) = level_val.as_str() {
                        match level_str {
                            "error" => severity = Some(crate::Severity::Error),
                            "warning" => severity = Some(crate::Severity::Warning),
                            "info" => severity = Some(crate::Severity::Info),
                            "disable" => enabled = Some(false),
                            _ => {}
                        }
                    }
                }

                match rule_name {
                    "line-length" => {
                        let mut max_length = 80;
                        let mut allow_non_breakable_words = true;

                        if let Some(max_val) = rule_map.get("max").and_then(|v| v.as_u64()) {
                            max_length = max_val as usize;
                        }
                        if let Some(allow_val) = rule_map.get("allow-non-breakable-words") {
                            if let Some(allow_bool) = allow_val.as_bool() {
                                allow_non_breakable_words = allow_bool;
                            }
                        }

                        let mut allow_non_breakable_inline_mappings = false;
                        if let Some(allow_val) = rule_map.get("allow-non-breakable-inline-mappings")
                        {
                            if let Some(allow_bool) = allow_val.as_bool() {
                                allow_non_breakable_inline_mappings = allow_bool;
                            }
                        }

                        let rule_settings = serde_json::to_value(config::LineLengthConfig {
                            max_length,
                            allow_non_breakable_words,
                            allow_non_breakable_inline_mappings,
                        })
                        .unwrap();
                        settings = Some(rule_settings);
                    }
                    "document-start" => {
                        if let Some(present_val) = rule_map.get("present") {
                            if let Some(present_bool) = present_val.as_bool() {
                                let rule_settings =
                                    serde_json::to_value(config::DocumentStartConfig {
                                        present: Some(present_bool),
                                    })
                                    .unwrap();
                                settings = Some(rule_settings);
                            }
                        }
                    }
                    "indentation" => {
                        let mut spaces = Some(2);
                        let mut indent_sequences = Some(true);
                        let check_multi_line_strings = Some(false);
                        let mut ignore = None;

                        if let Some(spaces_val) = rule_map.get("spaces").and_then(|v| v.as_u64()) {
                            spaces = Some(spaces_val as usize);
                        }
                        if let Some(indent_val) = rule_map.get("indent-sequences") {
                            if let Some(indent_bool) = indent_val.as_bool() {
                                indent_sequences = Some(indent_bool);
                            } else {
                                enabled = Some(false);
                            }
                        }

                        if let Some(ignore_val) = rule_map.get("ignore") {
                            if let Some(s) = ignore_val.as_str() {
                                ignore = Some(s.to_string());
                            } else {
                                ignore = serde_yaml::to_string(ignore_val)
                                    .ok()
                                    .map(|s| s.trim_matches('"').to_string());
                            }
                        }
                        let rule_settings = serde_json::to_value(config::IndentationConfig {
                            spaces,
                            indent_sequences,
                            check_multi_line_strings,
                            ignore,
                        })
                        .unwrap();
                        settings = Some(rule_settings);
                    }
                    "comments" => {
                        if let Some(min_spaces_val) = rule_map
                            .get("min-spaces-from-content")
                            .and_then(|v| v.as_u64())
                        {
                            let rule_settings = serde_json::to_value(config::CommentsConfig {
                                min_spaces_from_content: Some(min_spaces_val as usize),
                            })
                            .unwrap();
                            settings = Some(rule_settings);
                        }
                    }
                    "truthy" => {
                        let mut allowed_values = vec!["false".to_string(), "true".to_string()];
                        if let Some(allowed_vals) =
                            rule_map.get("allowed-values").and_then(|v| v.as_sequence())
                        {
                            allowed_values = allowed_vals
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();
                        }
                        let rule_settings =
                            serde_json::to_value(config::TruthyConfig { allowed_values }).unwrap();
                        settings = Some(rule_settings);
                    }
                    "empty-lines" => {
                        let mut max = None;
                        let mut max_start = None;
                        let mut max_end = None;

                        if let Some(max_val) = rule_map.get("max").and_then(|v| v.as_u64()) {
                            max = Some(max_val as usize);
                        }
                        if let Some(start_val) = rule_map.get("max-start").and_then(|v| v.as_u64())
                        {
                            max_start = Some(start_val as usize);
                        }
                        if let Some(end_val) = rule_map.get("max-end").and_then(|v| v.as_u64()) {
                            max_end = Some(end_val as usize);
                        }

                        let rule_settings = serde_json::to_value(config::EmptyLinesConfig {
                            max,
                            max_start,
                            max_end,
                        })
                        .unwrap();
                        settings = Some(rule_settings);
                    }
                    "trailing-spaces" => {
                        let allow = rule_map
                            .get("allow")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let rule_settings =
                            serde_json::to_value(config::TrailingSpacesConfig { allow }).unwrap();
                        settings = Some(rule_settings);
                    }
                    "document-end" => {
                        if let Some(present_val) = rule_map.get("present") {
                            if let Some(present_bool) = present_val.as_bool() {
                                let rule_settings =
                                    serde_json::to_value(config::DocumentEndConfig {
                                        present: Some(present_bool),
                                    })
                                    .unwrap();
                                settings = Some(rule_settings);
                            }
                        }
                    }
                    "key-ordering" => {
                        if let Some(order_vals) =
                            rule_map.get("order").and_then(|v| v.as_sequence())
                        {
                            let order: Vec<String> = order_vals
                                .iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect();
                            let rule_settings = serde_json::to_value(config::KeyOrderingConfig {
                                order: Some(order),
                            })
                            .unwrap();
                            settings = Some(rule_settings);
                        }
                    }
                    "anchors" => {
                        if let Some(max_len_val) =
                            rule_map.get("max-length").and_then(|v| v.as_u64())
                        {
                            let rule_settings = serde_json::to_value(config::AnchorsConfig {
                                max_length: Some(max_len_val as usize),
                            })
                            .unwrap();
                            settings = Some(rule_settings);
                        }
                    }
                    "new-lines" => {
                        if let Some(type_val) = rule_map.get("type").and_then(|v| v.as_str()) {
                            let type_str = type_val.to_string();
                            let rule_settings = serde_json::to_value(config::NewLinesConfig {
                                type_: Some(type_str),
                            })
                            .unwrap();
                            settings = Some(rule_settings);
                        }
                    }
                    _ => {}
                }

                let existing = config.rules.get(rule_name).cloned();
                let final_enabled = if let Some(ref existing_config) = existing {
                    enabled.or(existing_config.enabled)
                } else {
                    enabled
                };

                let final_severity =
                    severity.or_else(|| existing.as_ref().and_then(|c| c.severity));
                let final_settings = settings.or_else(|| existing.clone().and_then(|c| c.settings));

                let mut final_other = existing.map(|c| c.other).unwrap_or_default();

                for (key, value) in rule_map {
                    if let Some(key_str) = key.as_str() {
                        let json_val = yaml_value_to_json(value);
                        final_other.insert(key_str.to_string(), json_val);
                    }
                }

                config.rules.insert(
                    rule_name.to_string(),
                    config::RuleConfig {
                        enabled: final_enabled,
                        severity: final_severity,
                        settings: final_settings,
                        other: final_other,
                    },
                );
            }
        }
    }

    Ok(config)
}

pub fn discover_config_file() -> Option<PathBuf> {
    discover_config_file_from_dir(std::env::current_dir().ok()?)
}

pub fn discover_config_file_from_dir(start_dir: PathBuf) -> Option<PathBuf> {
    let mut dir = start_dir.as_path();
    loop {
        let config_path = dir.join(".yamllint");
        if config_path.exists() {
            return Some(config_path);
        }

        if let Some(parent) = dir.parent() {
            dir = parent;
        } else {
            break;
        }
    }

    None
}

#[derive(Debug, Clone)]
pub struct LintResult {
    pub file: String,
    pub issues: Vec<(LintIssue, String)>,
}

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            _ => Err(anyhow::anyhow!("Invalid severity: {}", s)),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Severity::Error => "error".to_string(),
            Severity::Warning => "warning".to_string(),
            Severity::Info => "info".to_string(),
        }
    }
}

pub fn lint_yaml<P: AsRef<Path>>(file_path: P) -> Result<LintResult> {
    let path = file_path.as_ref();
    let _content = std::fs::read_to_string(path)?;

    let result = LintResult {
        file: path.to_string_lossy().to_string(),
        issues: vec![],
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_lint_valid_yaml() {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(file, "key: value").expect("Failed to write to temp file");
        writeln!(file, "nested:").expect("Failed to write to temp file");
        writeln!(file, "  subkey: subvalue").expect("Failed to write to temp file");

        let result = lint_yaml(file.path()).expect("Failed to lint YAML");
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_default_config() {
        let config = config::Config::default();
        assert!(config.rules.contains_key("line-length"));
        assert!(config.rules.contains_key("indentation"));
    }
}
