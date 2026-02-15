//! MD026 - Trailing punctuation in heading

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD026;

impl Rule for MD026 {
    fn names(&self) -> &[&'static str] {
        &["MD026", "no-trailing-punctuation"]
    }

    fn description(&self) -> &'static str {
        "Trailing punctuation in heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md026.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();
        let punctuation = ".,;:!?";

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
                if hash_count > 0 && hash_count <= 6 {
                    let content = trimmed[hash_count..].trim();
                    // Remove trailing # for closed ATX
                    let content = content.trim_end_matches('#').trim_end();

                    if let Some(last_char) = content.chars().last() {
                        if punctuation.contains(last_char) {
                            errors.push(LintError {
                                line_number,
                                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                                rule_description: self.description().to_string(),
                                error_detail: Some(format!("Punctuation: '{}'", last_char)),
                                error_context: Some(content.to_string()),
                                rule_information: self.information().map(|s| s.to_string()),
                                error_range: None,
                                fix_info: None,
                                severity: Severity::Error,
                            });
                        }
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
    fn test_md026_no_punctuation() {
        let lines = vec!["# Heading\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD026;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md026_with_punctuation() {
        let lines = vec!["# Heading!\n".to_string(), "## Question?\n".to_string()];

        let params = RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines: &lines,
            front_matter_lines: &[],
            tokens: &[],
            config: &HashMap::new(),
        };

        let rule = MD026;
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }
}
