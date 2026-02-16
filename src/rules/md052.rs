//! MD052 - Reference links and images should use a label that is defined

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// Regex for reference link definitions: `[label]: url`
static DEF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\[([^\]]+)\]:\s+").unwrap());

/// Regex for full reference links: `[text][label]`
static FULL_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]*)\]\[([^\]]+)\]").unwrap());

/// Regex for collapsed reference links: `[label][]`
static COLLAPSED_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\[\]").unwrap());

pub struct MD052;

/// Check if a line is a code fence opener/closer (``` or ~~~)
fn is_code_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

impl Rule for MD052 {
    fn names(&self) -> &[&'static str] {
        &["MD052", "reference-links-images"]
    }

    fn description(&self) -> &'static str {
        "Reference links and images should use a label that is defined"
    }

    fn tags(&self) -> &[&'static str] {
        &["links", "images"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md052.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let mut defined_labels: HashSet<String> = HashSet::new();

        // Pass 1: Collect all reference definitions (skipping code blocks)
        let mut in_code_block = false;
        for line in params.lines.iter() {
            if is_code_fence(line) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            if let Some(caps) = DEF_RE.captures(line) {
                let label = caps[1].to_lowercase();
                defined_labels.insert(label);
            }
        }

        // Pass 2: Find all reference usages and check if they are defined
        in_code_block = false;
        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;

            if is_code_fence(line) {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            // Check full reference links: [text][label]
            for caps in FULL_REF_RE.captures_iter(line) {
                let label = caps[2].to_lowercase();
                if !defined_labels.contains(&label) {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!(
                            "Reference label \"{}\" is not defined",
                            &caps[2]
                        )),
                        error_context: Some(caps[0].to_string()),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: None,
                        severity: Severity::Error,
                    });
                }
            }

            // Check collapsed reference links: [label][]
            for caps in COLLAPSED_REF_RE.captures_iter(line) {
                let label = caps[1].to_lowercase();
                if !defined_labels.contains(&label) {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!(
                            "Reference label \"{}\" is not defined",
                            &caps[1]
                        )),
                        error_context: Some(caps[0].to_string()),
                        rule_information: self.information().map(|s| s.to_string()),
                        error_range: None,
                        fix_info: None,
                        severity: Severity::Error,
                    });
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_params<'a>(
        lines: &'a [String],
        config: &'a HashMap<String, serde_json::Value>,
    ) -> RuleParams<'a> {
        RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens: &[],
            config,
        }
    }

    #[test]
    fn test_md052_valid_references() {
        let lines: Vec<String> = vec![
            "This has a [link][foo] reference.\n".to_string(),
            "\n".to_string(),
            "[foo]: https://example.com\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md052_undefined_reference() {
        let lines: Vec<String> = vec![
            "This has a [link][bar] reference.\n".to_string(),
            "\n".to_string(),
            "[foo]: https://example.com\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line_number, 1);
    }

    #[test]
    fn test_md052_case_insensitive() {
        let lines: Vec<String> = vec![
            "This has a [link][foo] reference.\n".to_string(),
            "\n".to_string(),
            "[Foo]: https://example.com\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);

        let rule = MD052;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }
}
