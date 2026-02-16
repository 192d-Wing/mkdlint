//! MD043 - Required heading structure

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};

pub struct MD043;

/// Extract heading level and text from a markdown heading line
fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return None;
    }
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    if level > 6 {
        return None;
    }
    let text = trimmed[level..]
        .trim()
        .trim_end_matches('#')
        .trim()
        .to_string();
    Some((level, text))
}

/// Check if an actual heading matches an expected pattern
fn heading_matches(actual_level: usize, actual_text: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();

    // "#+" matches any heading at any level
    if pattern == "#+" {
        return true;
    }

    if let Some((expected_level, expected_text)) = parse_heading(pattern) {
        if actual_level != expected_level {
            return false;
        }
        // "*" wildcard matches any text at this level
        if expected_text == "*" {
            return true;
        }
        // Exact text match
        actual_text == expected_text
    } else {
        false
    }
}

impl Rule for MD043 {
    fn names(&self) -> &[&'static str] {
        &["MD043", "required-headings", "required-headers"]
    }

    fn description(&self) -> &'static str {
        "Required heading structure"
    }

    fn tags(&self) -> &[&'static str] {
        &["headings", "headers"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md043.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        // Get required headings from config
        let required = match params.config.get("headings") {
            Some(val) => match val.as_array() {
                Some(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>(),
                None => return vec![],
            },
            None => return vec![], // opt-in rule
        };

        if required.is_empty() {
            return vec![];
        }

        let mut errors = Vec::new();

        // Collect actual headings from lines
        let mut actual_headings: Vec<(usize, usize, String)> = Vec::new(); // (line_number, level, text)
        let mut in_code_block = false;
        for (idx, line) in params.lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }
            if let Some((level, text)) = parse_heading(trimmed) {
                actual_headings.push((idx + 1, level, text));
            }
        }

        // Compare expected vs actual
        let mut actual_idx = 0;
        for expected in &required {
            if actual_idx >= actual_headings.len() {
                // Missing expected heading
                let last_line = params.lines.len();
                errors.push(LintError {
                    line_number: last_line,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: Some(format!("Expected: {}", expected)),
                    error_context: None,
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: None,
                    fix_info: None,
                    suggestion: Some("Follow the required heading structure".to_string()),
                    severity: Severity::Error,
                });
                continue;
            }

            let (line_num, level, ref text) = actual_headings[actual_idx];
            if !heading_matches(level, text, expected) {
                errors.push(LintError {
                    line_number: line_num,
                    rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                    rule_description: self.description().to_string(),
                    error_detail: Some(format!(
                        "Expected: {}; Actual: {} {}",
                        expected,
                        "#".repeat(level),
                        text
                    )),
                    error_context: None,
                    rule_information: self.information().map(|s| s.to_string()),
                    error_range: None,
                    fix_info: None,
                    suggestion: Some("Follow the required heading structure".to_string()),
                    severity: Severity::Error,
                });
            }
            actual_idx += 1;
        }

        // Report extra headings beyond what's expected
        while actual_idx < actual_headings.len() {
            let (line_num, level, ref text) = actual_headings[actual_idx];
            errors.push(LintError {
                line_number: line_num,
                rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                rule_description: self.description().to_string(),
                error_detail: Some(format!("Extra heading: {} {}", "#".repeat(level), text)),
                error_context: None,
                rule_information: self.information().map(|s| s.to_string()),
                error_range: None,
                fix_info: None,
                suggestion: Some("Follow the required heading structure".to_string()),
                severity: Severity::Error,
            });
            actual_idx += 1;
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
    ) -> crate::types::RuleParams<'a> {
        crate::types::RuleParams {
            name: "test.md",
            version: "0.1.0",
            lines,
            front_matter_lines: &[],
            tokens: &[],
            config,
        }
    }

    #[test]
    fn test_md043_no_config() {
        let rule = MD043;
        let lines = vec!["# Title\n".to_string()];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        assert_eq!(rule.lint(&params).len(), 0);
    }

    #[test]
    fn test_md043_matching_structure() {
        let rule = MD043;
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "## Section\n".to_string(),
        ];
        let mut config = HashMap::new();
        config.insert(
            "headings".to_string(),
            serde_json::json!(["# Title", "## Section"]),
        );
        let params = make_params(&lines, &config);
        assert_eq!(rule.lint(&params).len(), 0);
    }

    #[test]
    fn test_md043_missing_heading() {
        let rule = MD043;
        let lines = vec!["# Title\n".to_string()];
        let mut config = HashMap::new();
        config.insert(
            "headings".to_string(),
            serde_json::json!(["# Title", "## Section"]),
        );
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("Expected")
        );
    }

    #[test]
    fn test_md043_extra_heading() {
        let rule = MD043;
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "## Section\n".to_string(),
            "\n".to_string(),
            "## Extra\n".to_string(),
        ];
        let mut config = HashMap::new();
        config.insert("headings".to_string(), serde_json::json!(["# Title"]));
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 2); // Two extra headings
    }

    #[test]
    fn test_md043_wildcard() {
        let rule = MD043;
        let lines = vec![
            "# Any Title\n".to_string(),
            "\n".to_string(),
            "## Anything\n".to_string(),
        ];
        let mut config = HashMap::new();
        config.insert("headings".to_string(), serde_json::json!(["# *", "## *"]));
        let params = make_params(&lines, &config);
        assert_eq!(rule.lint(&params).len(), 0);
    }

    #[test]
    fn test_md043_any_heading_pattern() {
        let rule = MD043;
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "### Deep heading\n".to_string(),
        ];
        let mut config = HashMap::new();
        config.insert("headings".to_string(), serde_json::json!(["#+", "#+"]));
        let params = make_params(&lines, &config);
        assert_eq!(rule.lint(&params).len(), 0);
    }

    #[test]
    fn test_md043_wrong_level() {
        let rule = MD043;
        let lines = vec!["## Not Title\n".to_string()];
        let mut config = HashMap::new();
        config.insert("headings".to_string(), serde_json::json!(["# Title"]));
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_parse_heading() {
        assert_eq!(parse_heading("# Title"), Some((1, "Title".to_string())));
        assert_eq!(parse_heading("## Sub"), Some((2, "Sub".to_string())));
        assert_eq!(parse_heading("###### Deep"), Some((6, "Deep".to_string())));
        assert_eq!(parse_heading("Not a heading"), None);
    }
}
