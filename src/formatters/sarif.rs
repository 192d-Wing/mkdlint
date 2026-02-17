//! SARIF v2.1.0 output formatter

use crate::types::{LintResults, Severity};

/// Convert a file path to a SARIF `artifactLocation.uri`.
///
/// Absolute paths become `file:///...` URIs; relative paths are kept as-is
/// (SARIF allows relative URIs resolved against `originalUriBaseIds`).
fn path_to_uri(path: &str) -> String {
    let p = std::path::Path::new(path);
    if p.is_absolute() {
        // Encode as file URI — percent-encode spaces and other special chars
        let encoded = path.replace(' ', "%20");
        format!("file://{encoded}")
    } else {
        path.to_string()
    }
}

/// Format lint results as SARIF v2.1.0 JSON
pub fn format_sarif(results: &LintResults) -> String {
    let mut sarif_results = Vec::new();
    // Ordered map: rule_id → (index, rule_json)
    let mut rule_map: std::collections::BTreeMap<String, (usize, serde_json::Value)> =
        std::collections::BTreeMap::new();

    let mut files: Vec<_> = results.results.keys().collect();
    files.sort();

    for file in &files {
        if let Some(errors) = results.results.get(*file) {
            let uri = path_to_uri(file);

            for error in errors {
                let rule_id = error.rule_names.first().copied().unwrap_or("unknown");

                // Register rule in the driver's rules array (deduped, ordered)
                let rule_index = if let Some((idx, _)) = rule_map.get(rule_id) {
                    *idx
                } else {
                    let idx = rule_map.len();
                    let rule_entry = serde_json::json!({
                        "id": rule_id,
                        "name": error.rule_names.get(1).unwrap_or(&error.rule_names[0]),
                        "shortDescription": {
                            "text": error.rule_description
                        },
                        "helpUri": error.rule_information.unwrap_or(""),
                        "properties": {
                            "tags": error.rule_names.iter().skip(1).collect::<Vec<_>>()
                        }
                    });
                    rule_map.insert(rule_id.to_string(), (idx, rule_entry));
                    idx
                };

                let level = match error.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                };

                let mut message_text = error.rule_description.to_string();
                if let Some(detail) = &error.error_detail {
                    message_text.push_str(&format!(" ({})", detail));
                }

                let mut region = serde_json::json!({
                    "startLine": error.line_number
                });
                if let Some((start, length)) = error.error_range {
                    region["startColumn"] = serde_json::json!(start);
                    region["endColumn"] = serde_json::json!(start + length);
                }

                let mut result = serde_json::json!({
                    "ruleId": rule_id,
                    "ruleIndex": rule_index,
                    "level": level,
                    "message": {
                        "text": message_text
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": uri,
                                "uriBaseId": "%SRCROOT%"
                            },
                            "region": region
                        }
                    }]
                });

                // Add fix suggestion if available
                if error.fix_info.is_some() {
                    let fix_description = error
                        .suggestion
                        .as_deref()
                        .unwrap_or("Apply automatic fix");
                    result["fixes"] = serde_json::json!([{
                        "description": {
                            "text": fix_description
                        }
                    }]);
                }

                // Add suggestion as a suppression hint if present (and no fix)
                if error.fix_info.is_none()
                    && let Some(suggestion) = &error.suggestion
                {
                    result["message"]["markdown"] = serde_json::json!(
                        format!("{message_text}\n\n> {suggestion}")
                    );
                }

                sarif_results.push(result);
            }
        }
    }

    let rules: Vec<_> = rule_map.into_values().map(|(_, v)| v).collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "mkdlint",
                    "version": crate::VERSION,
                    "informationUri": "https://github.com/192d-Wing/mkdlint",
                    "rules": rules
                }
            },
            "originalUriBaseIds": {
                "%SRCROOT%": {
                    "uri": "file:///"
                }
            },
            "results": sarif_results
        }]
    });

    serde_json::to_string_pretty(&sarif)
        .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize SARIF: {}\"}}", e))
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
                rule_names: &["MD001", "heading-increment"],
                rule_description: "Heading levels should increment by one",
                error_range: Some((1, 4)),
                severity: Severity::Error,
                ..Default::default()
            }],
        );

        let output = format_sarif(&results);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(parsed["runs"][0]["tool"]["driver"]["name"], "mkdlint");

        let result = &parsed["runs"][0]["results"][0];
        assert_eq!(result["ruleId"], "MD001");
        assert_eq!(result["ruleIndex"], 0);
        assert_eq!(result["level"], "error");
        assert_eq!(
            result["locations"][0]["physicalLocation"]["region"]["startLine"],
            3
        );
        assert_eq!(
            result["locations"][0]["physicalLocation"]["region"]["startColumn"],
            1
        );
        assert_eq!(
            result["locations"][0]["physicalLocation"]["region"]["endColumn"],
            5
        );
        // artifactLocation should have uriBaseId
        assert_eq!(
            result["locations"][0]["physicalLocation"]["artifactLocation"]["uriBaseId"],
            "%SRCROOT%"
        );

        let rules = &parsed["runs"][0]["tool"]["driver"]["rules"];
        assert_eq!(rules[0]["id"], "MD001");
        assert_eq!(rules[0]["name"], "heading-increment");
        // Rules should have properties.tags
        assert!(rules[0]["properties"]["tags"].is_array());
    }

    #[test]
    fn test_format_sarif_fixable_has_fixes_array() {
        use crate::types::FixInfo;
        let mut results = LintResults::new();
        results.add(
            "test.md".to_string(),
            vec![LintError {
                line_number: 1,
                rule_names: &["MD018", "no-missing-space-atx"],
                rule_description: "No space after hash on ATX heading",
                severity: Severity::Error,
                fix_info: Some(FixInfo {
                    line_number: Some(1),
                    edit_column: Some(2),
                    delete_count: None,
                    insert_text: Some(" ".to_string()),
                }),
                suggestion: Some("Add a space after the # symbol".to_string()),
                ..Default::default()
            }],
        );

        let output = format_sarif(&results);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let result = &parsed["runs"][0]["results"][0];
        // Fixable errors should have a fixes array
        assert!(result["fixes"].is_array());
        assert!(!result["fixes"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_format_sarif_absolute_path_uses_file_uri() {
        let mut results = LintResults::new();
        results.add(
            "/home/user/docs/readme.md".to_string(),
            vec![LintError {
                line_number: 1,
                rule_names: &["MD047"],
                rule_description: "Files should end with a single newline",
                severity: Severity::Error,
                ..Default::default()
            }],
        );

        let output = format_sarif(&results);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let uri = parsed["runs"][0]["results"][0]["locations"][0]["physicalLocation"]
            ["artifactLocation"]["uri"]
            .as_str()
            .unwrap();
        assert!(uri.starts_with("file://"), "absolute path should become file:// URI");
    }

    #[test]
    fn test_format_sarif_empty() {
        let results = LintResults::new();
        let output = format_sarif(&results);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["runs"][0]["results"].as_array().unwrap().len(), 0);
        // originalUriBaseIds should be present
        assert!(parsed["runs"][0]["originalUriBaseIds"].is_object());
    }
}
