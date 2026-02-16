//! MD051 - Link fragments should be valid

use crate::types::{LintError, ParserType, Rule, RuleParams, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static FRAGMENT_LINK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([^\]]*)\]\(#([^)]+)\)").unwrap());

pub struct MD051;

/// Convert heading text to a GitHub-style anchor ID
fn heading_to_id(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut id = String::new();
    for ch in lower.chars() {
        if ch.is_alphanumeric() {
            id.push(ch);
        } else if ch == ' ' || ch == '-' {
            id.push('-');
        }
        // Skip other characters (punctuation, etc.)
    }
    // Collapse multiple hyphens
    while id.contains("--") {
        id = id.replace("--", "-");
    }
    // Trim leading/trailing hyphens
    id.trim_matches('-').to_string()
}

/// Extract heading text from a markdown heading line
fn extract_heading_text(line: &str) -> Option<String> {
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
    if text.is_empty() {
        return None;
    }
    Some(text)
}

/// Collect all heading IDs with duplicate handling
fn collect_heading_ids(lines: &[String]) -> Vec<String> {
    let mut ids = Vec::new();
    let mut id_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut in_code_block = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if let Some(text) = extract_heading_text(trimmed) {
            let base_id = heading_to_id(&text);
            let count = id_counts.entry(base_id.clone()).or_insert(0);
            let final_id = if *count == 0 {
                base_id.clone()
            } else {
                format!("{}-{}", base_id, count)
            };
            *count += 1;
            ids.push(final_id);
        }
    }

    ids
}

impl Rule for MD051 {
    fn names(&self) -> &[&'static str] {
        &["MD051", "link-fragments"]
    }

    fn description(&self) -> &'static str {
        "Link fragments should be valid"
    }

    fn tags(&self) -> &[&'static str] {
        &["links"]
    }

    fn parser_type(&self) -> ParserType {
        ParserType::None
    }

    fn information(&self) -> Option<&'static str> {
        Some("https://github.com/DavidAnson/markdownlint/blob/main/doc/md051.md")
    }

    fn lint(&self, params: &RuleParams) -> Vec<LintError> {
        let mut errors = Vec::new();

        // Collect all valid heading IDs
        let heading_ids = collect_heading_ids(params.lines);

        // Find all fragment links and check them
        let mut in_code_block = false;
        for (idx, line) in params.lines.iter().enumerate() {
            let line_number = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block {
                continue;
            }

            for cap in FRAGMENT_LINK_RE.captures_iter(line) {
                let fragment = &cap[2];
                if !heading_ids.contains(&fragment.to_string()) {
                    errors.push(LintError {
                        line_number,
                        rule_names: self.names().iter().map(|s| s.to_string()).collect(),
                        rule_description: self.description().to_string(),
                        error_detail: Some(format!(
                            "No matching heading for fragment: #{}",
                            fragment
                        )),
                        error_context: Some(cap[0].to_string()),
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
    fn test_heading_to_id() {
        assert_eq!(heading_to_id("Hello World"), "hello-world");
        assert_eq!(heading_to_id("Getting Started"), "getting-started");
        assert_eq!(heading_to_id("What's New?"), "whats-new");
        assert_eq!(heading_to_id("API Reference"), "api-reference");
        assert_eq!(heading_to_id("v2.0 Release"), "v20-release");
    }

    #[test]
    fn test_collect_heading_ids_duplicates() {
        let lines = vec![
            "# Title\n".to_string(),
            "## Section\n".to_string(),
            "## Section\n".to_string(),
            "## Section\n".to_string(),
        ];
        let ids = collect_heading_ids(&lines);
        assert_eq!(ids, vec!["title", "section", "section-1", "section-2"]);
    }

    #[test]
    fn test_md051_valid_fragment() {
        let rule = MD051;
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "## Getting Started\n".to_string(),
            "\n".to_string(),
            "See [title](#title) and [start](#getting-started).\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md051_invalid_fragment() {
        let rule = MD051;
        let lines = vec![
            "# Title\n".to_string(),
            "\n".to_string(),
            "See [missing](#nonexistent).\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .error_detail
                .as_ref()
                .unwrap()
                .contains("nonexistent")
        );
    }

    #[test]
    fn test_md051_duplicate_heading_ids() {
        let rule = MD051;
        let lines = vec![
            "# Title\n".to_string(),
            "## Section\n".to_string(),
            "## Section\n".to_string(),
            "\n".to_string(),
            "See [first](#section) and [second](#section-1).\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md051_fragment_in_code_block_ignored() {
        let rule = MD051;
        let lines = vec![
            "# Title\n".to_string(),
            "```\n".to_string(),
            "[link](#nonexistent)\n".to_string(),
            "```\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_md051_multiple_fragments_one_line() {
        let rule = MD051;
        let lines = vec![
            "# Title\n".to_string(),
            "## About\n".to_string(),
            "\n".to_string(),
            "See [a](#title) and [b](#missing).\n".to_string(),
        ];
        let config = HashMap::new();
        let params = make_params(&lines, &config);
        let errors = rule.lint(&params);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_extract_heading_text() {
        assert_eq!(extract_heading_text("# Title"), Some("Title".to_string()));
        assert_eq!(
            extract_heading_text("## Sub Title"),
            Some("Sub Title".to_string())
        );
        assert_eq!(extract_heading_text("Not heading"), None);
        assert_eq!(extract_heading_text("#"), None); // Empty heading
    }
}
