//! MD026 - Trailing punctuation in heading

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD026;

impl Rule for MD026 {
    fn names(&self) -> &[&'static str] {
        &["MD026", "no-trailing-punctuation"]
    }

    fn description(&self) -> &'static str {
        "Trailing punctuation in heading"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "fixable"]
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

                    if let Some(last_char) = content.chars().last()
                        && punctuation.contains(last_char)
                    {
                        // Compute 1-based column of the punctuation char in the original line
                        let leading_ws = line.len() - line.trim_start().len();
                        // content is a sub-slice of trimmed; find its end position
                        // relative to trimmed start
                        let trimmed_start_in_line = leading_ws;
                        let content_offset_in_trimmed =
                            content.as_ptr() as usize - trimmed.as_ptr() as usize;
                        let punc_byte_offset = content.len() - last_char.len_utf8();
                        let punc_col_0based =
                            trimmed_start_in_line + content_offset_in_trimmed + punc_byte_offset;

                        errors.push(LintError {
                            line_number,
                            rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                            rule_description: self.description().to_string(),
                            error_detail: Some(format!("Punctuation: '{}'", last_char)),
                            error_context: Some(content.to_string()),
                            rule_information: self.information().map(|s| s.to_string()),
                            error_range: None,
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(punc_col_0based + 1), // 1-based
                                delete_count: Some(last_char.len_utf8() as i32),
                                insert_text: None,
                            }),
                            suggestion: Some(
                                "Remove trailing punctuation from heading".to_string(),
                            ),
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

    #[test]
    fn test_md026_fix_info_exclamation() {
        let lines = vec!["# Heading!\n".to_string()];

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
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        // "# Heading!" -> '!' is at column 10 (1-based)
        assert_eq!(fix.edit_column, Some(10));
        assert_eq!(fix.delete_count, Some(1));
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md026_fix_info_question() {
        let lines = vec!["## Question?\n".to_string()];

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
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        // "## Question?" -> '?' is at column 12 (1-based)
        assert_eq!(fix.edit_column, Some(12));
        assert_eq!(fix.delete_count, Some(1));
        assert_eq!(fix.insert_text, None);
    }

    #[test]
    fn test_md026_fix_info_closed_atx() {
        let lines = vec!["# Heading! ##\n".to_string()];

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
        assert_eq!(errors.len(), 1);

        let fix = errors[0]
            .fix_info
            .as_ref()
            .expect("fix_info should be present");
        assert_eq!(fix.line_number, None);
        // "# Heading! ##" -> content after stripping trailing '##' and space is "Heading!"
        // '!' is at column 10 (1-based) in the original line
        assert_eq!(fix.edit_column, Some(10));
        assert_eq!(fix.delete_count, Some(1));
        assert_eq!(fix.insert_text, None);
    }
}
