//! MD018 - No space after hash on atx style heading
//!
//! This rule checks that ATX headings have a space after the hash

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD018;

impl Rule for MD018 {
    fn names(&self) -> &'static [&'static str] {
        &["MD018", "no-missing-space-atx"]
    }

    fn description(&self) -> &'static str {
        "No space after hash on atx style heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "atx", "spaces", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md018.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim_start();

            // Check for ATX heading without space
            if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                if hash_count > 0 && hash_count <= 6 {
                    let after_hash = &trimmed[hash_count..];
                    // Skip if nothing follows the hashes (empty heading),
                    // if content is only whitespace/newlines (avoids MD009 oscillation),
                    // or if a space/tab already exists.
                    if !after_hash.is_empty()
                        && !after_hash.trim().is_empty()
                        && !after_hash.starts_with(' ')
                        && !after_hash.starts_with('\t')
                    {
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: None,
                            error_context: Some(
                                trimmed
                                    .chars()
                                    .take(hash_count + 10.min(after_hash.chars().count()))
                                    .collect(),
                            ),
                            rule_information: self.information(),
                            error_range: Some((hash_count + 1, 1)),
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(hash_count + 1),
                                delete_count: None,
                                insert_text: Some(" ".to_string()),
                            }),
                            suggestion: Some(format!(
                                "Add a space after the # symbol: '{} {}'",
                                "#".repeat(hash_count),
                                after_hash.trim()
                            )),
                            severity: Severity::Error,
                        });
                    }
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

    #[test]
    fn test_md018_with_space() {
        let lines = vec!["# Heading\n", "## Heading 2\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD018;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md018_no_space() {
        let lines = vec!["#Heading\n", "##Heading 2\n"];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD018;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
        assert!(errors[0].fix_info.is_some());
    }
}
