//! MD044 - Proper names should have the correct capitalization

use crate::types::{FixInfo, LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD044;

impl Rule for MD044 {
    fn names(&self) -> &'static [&'static str] {
        &["MD044", "proper-names"]
    }

    fn description(&self) -> &'static str {
        "Proper names should have the correct capitalization"
    }

    fn tags(&self) -> &[&'static str] {
        &["spelling", "fixable"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md044.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Read names from config or use defaults
        let names: Vec<String> = params
            .config
            .get("names")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| {
                vec![
                    "JavaScript".to_string(),
                    "TypeScript".to_string(),
                    "GitHub".to_string(),
                    "Node.js".to_string(),
                ]
            });

        let check_code_blocks = params
            .config
            .get("code_blocks")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Build lookup pairs: (lowercase, correct)
        let proper_names: Vec<(String, String)> = names
            .iter()
            .map(|name| (name.to_lowercase(), name.clone()))
            .collect();

        let mut in_code_block = false;

        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            // Track code blocks
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }

            // Skip code block content unless configured to check
            if in_code_block && !check_code_blocks {
                continue;
            }

            let lower_line = line.to_lowercase();

            for (incorrect, correct) in &proper_names {
                // Iterate over all occurrences of the lowercase name in the line
                let mut search_start = 0;
                while let Some(pos) = lower_line[search_start..].find(incorrect.as_str()) {
                    let absolute_pos = search_start + pos;
                    let end_pos = absolute_pos + correct.len();

                    // Check if this particular occurrence is already correctly cased
                    if end_pos <= line.len() && &line[absolute_pos..end_pos] != correct.as_str() {
                        let actual = &line[absolute_pos..end_pos];
                        errors.push(LintError {
                            line_number,
                            rule_names: self.names(),
                            rule_description: self.description(),
                            error_detail: Some(format!(
                                "Expected: {}; Actual: {}",
                                correct, actual
                            )),
                            error_context: None,
                            rule_information: self.information(),
                            error_range: Some((absolute_pos + 1, correct.len())),
                            fix_info: Some(FixInfo {
                                line_number: None,
                                edit_column: Some(absolute_pos + 1), // 1-based
                                delete_count: Some(correct.len() as i32),
                                insert_text: Some(correct.clone()),
                            }),
                            suggestion: Some(
                                "Use proper capitalization for this proper noun".to_string(),
                            ),
                            severity: Severity::Error,
                            fix_only: false,
                        });
                    }

                    search_start = absolute_pos + incorrect.len();
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
    fn test_md044_default_names() {
        let rule = MD044;
        let lines = vec!["I love javascript and github.\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_md044_correct_names() {
        let rule = MD044;
        let lines = vec!["I love JavaScript and GitHub.\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md044_config_names() {
        let rule = MD044;
        let lines = vec!["Use rust for everything.\n"];
        let mut config = HashMap::new();
        config.insert("names".to_string(), serde_json::json!(["Rust"]));
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md044_code_block_excluded() {
        let rule = MD044;
        let lines = vec!["```\n", "javascript code\n", "```\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0); // code blocks excluded by default
    }

    #[test]
    fn test_md044_code_block_included() {
        let rule = MD044;
        let lines = vec!["```\n", "javascript code\n", "```\n"];
        let mut config = HashMap::new();
        config.insert("code_blocks".to_string(), serde_json::json!(true));
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1); // code blocks checked when configured
    }

    #[test]
    fn test_md044_fix_info_single_occurrence() {
        let rule = MD044;
        let lines = vec!["I love javascript.\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("should have fix_info");
        assert_eq!(fix.line_number, None);
        // "I love javascript" -> "javascript" starts at index 7, 1-based = 8
        assert_eq!(fix.edit_column, Some(8));
        assert_eq!(fix.delete_count, Some(10)); // "JavaScript".len() == 10
        assert_eq!(fix.insert_text, Some("JavaScript".to_string()));
    }

    #[test]
    fn test_md044_fix_info_multiple_occurrences() {
        let rule = MD044;
        let lines = vec!["javascript and javascript are great.\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2);

        let fix0 = errors[0].fix_info.as_ref().expect("first fix_info");
        assert_eq!(fix0.edit_column, Some(1)); // starts at position 0, 1-based = 1
        assert_eq!(fix0.delete_count, Some(10));
        assert_eq!(fix0.insert_text, Some("JavaScript".to_string()));

        let fix1 = errors[1].fix_info.as_ref().expect("second fix_info");
        assert_eq!(fix1.edit_column, Some(16)); // "javascript and " = 15 chars, 1-based = 16
        assert_eq!(fix1.delete_count, Some(10));
        assert_eq!(fix1.insert_text, Some("JavaScript".to_string()));
    }

    #[test]
    fn test_md044_fix_info_mixed_correct_and_incorrect() {
        let rule = MD044;
        let lines = vec!["JavaScript and javascript here.\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        // Only the second occurrence is wrong
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("should have fix_info");
        // "JavaScript and javascript" -> second "javascript" starts at index 15, 1-based = 16
        assert_eq!(fix.edit_column, Some(16));
        assert_eq!(fix.delete_count, Some(10));
        assert_eq!(fix.insert_text, Some("JavaScript".to_string()));
    }

    #[test]
    fn test_md044_fix_info_custom_name() {
        let rule = MD044;
        let lines = vec!["Use rust for everything.\n"];
        let mut config = HashMap::new();
        config.insert("names".to_string(), serde_json::json!(["Rust"]));
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);

        let fix = errors[0].fix_info.as_ref().expect("should have fix_info");
        // "Use rust" -> "rust" at index 4, 1-based = 5
        assert_eq!(fix.edit_column, Some(5));
        assert_eq!(fix.delete_count, Some(4)); // "Rust".len() == 4
        assert_eq!(fix.insert_text, Some("Rust".to_string()));
    }

    #[test]
    fn test_md044_fix_info_error_detail_shows_actual() {
        let rule = MD044;
        let lines = vec!["I use Github daily.\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].error_detail.as_deref(),
            Some("Expected: GitHub; Actual: Github")
        );
    }

    #[test]
    fn test_md044_multibyte_utf8_no_panic() {
        let rule = MD044;
        // Multi-byte chars before a proper name match
        let lines = vec!["Sch\u{00f6}ne javascript Welt.\n"];
        let config = HashMap::new();
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        // Should find the "javascript" match without panicking
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_md044_multibyte_in_proper_name_config() {
        let rule = MD044;
        let lines = vec!["Use caf\u{00e9} brew.\n"];
        let mut config = HashMap::new();
        config.insert("names".to_string(), serde_json::json!(["Caf\u{00e9}"]));
        let params = crate::types::RuleParams::test(&lines, &config);
        let errors = rule.lint(&params);
        // "caf\u{e9}" should be detected as wrong case for "Caf\u{e9}"
        assert_eq!(errors.len(), 1);
    }
}
