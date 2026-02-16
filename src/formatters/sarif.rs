//! SARIF v2.1.0 output formatter

use crate::types::{LintResults, Severity};

/// Format lint results as SARIF v2.1.0 JSON
pub fn format_sarif(results: &LintResults) -> String {
    let mut sarif_results = Vec::new();
    let mut rule_set = std::collections::BTreeMap::new();

    let mut files: Vec<_> = results.results.keys().collect();
    files.sort();

    for file in &files {
        if let Some(errors) = results.results.get(*file) {
            for error in errors {
                let rule_id = error.rule_names.first().map(|s| s.as_str()).unwrap_or("unknown");

                // Track unique rules for the tool driver
                rule_set.entry(rule_id.to_string()).or_insert_with(|| {
                    serde_json::json!({
                        "id": rule_id,
                        "name": error.rule_names.get(1).unwrap_or(&error.rule_names[0]),
                        "shortDescription": {
                            "text": error.rule_description
                        },
                        "helpUri": error.rule_information.as_deref().unwrap_or("")
                    })
                });

                let level = match error.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                };

                let mut message_text = error.rule_description.clone();
                if let Some(detail) = &error.error_detail {
                    message_text.push_str(&format!(" [{}]", detail));
                }

                let mut region = serde_json::json!({
                    "startLine": error.line_number
                });
                if let Some((start, length)) = error.error_range {
                    region["startColumn"] = serde_json::json!(start);
                    region["endColumn"] = serde_json::json!(start + length);
                }

                sarif_results.push(serde_json::json!({
                    "ruleId": rule_id,
                    "level": level,
                    "message": {
                        "text": message_text
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": file
                            },
                            "region": region
                        }
                    }]
                }));
            }
        }
    }

    let rules: Vec<_> = rule_set.into_values().collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "mdlint",
                    "version": crate::VERSION,
                    "informationUri": "https://github.com/1456055067/mdlint",
                    "rules": rules
                }
            },
            "results": sarif_results
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_else(|e| {
        format!("{{\"error\": \"Failed to serialize SARIF: {}\"}}", e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LintError, LintResults};

    #[test]
    fn test_format_sarif_structure() {
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![LintError {
                line_number: 3,
                rule_names: vec!["MD001".to_string(), "heading-increment".to_string()],
                rule_description: "Heading levels should increment by one".to_string(),
                error_range: Some((1, 4)),
                severity: Severity::Error,
                ..Default::default()
            }],
        );

        let output = format_sarif(&results);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(parsed["runs"][0]["tool"]["driver"]["name"], "mdlint");

        let result = &parsed["runs"][0]["results"][0];
        assert_eq!(result["ruleId"], "MD001");
        assert_eq!(result["level"], "error");
        assert_eq!(result["locations"][0]["physicalLocation"]["region"]["startLine"], 3);
        assert_eq!(result["locations"][0]["physicalLocation"]["region"]["startColumn"], 1);
        assert_eq!(result["locations"][0]["physicalLocation"]["region"]["endColumn"], 5);

        let rules = &parsed["runs"][0]["tool"]["driver"]["rules"];
        assert_eq!(rules[0]["id"], "MD001");
        assert_eq!(rules[0]["name"], "heading-increment");
    }

    #[test]
    fn test_format_sarif_empty() {
        let results = LintResults::new();
        let output = format_sarif(&results);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["runs"][0]["results"].as_array().unwrap().len(), 0);
    }
}
