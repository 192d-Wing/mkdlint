//! `--generate-schema` handler — emit a JSON Schema for mkdlint config files

/// Generate a JSON Schema for the mkdlint configuration file.
///
/// The schema describes all top-level config keys (`default`, `extends`,
/// `preset`) as well as every rule ID as a known property with a description.
pub(crate) fn generate_config_schema() -> String {
    use mkdlint::rules::get_rules;

    let rules = get_rules();

    // Build per-rule property definitions
    let mut rule_props = serde_json::Map::new();
    for rule in rules.iter() {
        let id = rule.names()[0];
        let description = rule.description();
        let tags: Vec<&str> = rule.tags().to_vec();
        let is_fixable = tags.contains(&"fixable");

        // Each rule can be true/false, "warning"/"error", or an object with options
        let prop = serde_json::json!({
            "description": format!(
                "{description}{}",
                if is_fixable { " [auto-fixable]" } else { "" }
            ),
            "oneOf": [
                { "type": "boolean", "description": "Enable or disable the rule" },
                {
                    "type": "string",
                    "enum": ["error", "warning"],
                    "description": "Set severity level"
                },
                {
                    "type": "object",
                    "description": "Rule-specific options",
                    "additionalProperties": true
                }
            ]
        });
        rule_props.insert(id.to_string(), prop);
    }

    let mut properties = serde_json::Map::new();
    properties.insert(
        "default".to_string(),
        serde_json::json!({
            "description": "Default enabled/disabled state for all rules not explicitly configured",
            "type": "boolean"
        }),
    );
    properties.insert(
        "extends".to_string(),
        serde_json::json!({
            "description": "Path to another config file to extend",
            "type": "string"
        }),
    );
    properties.insert(
        "preset".to_string(),
        serde_json::json!({
            "description": "Named preset to apply (e.g. 'kramdown', 'github')",
            "type": "string",
            "enum": ["kramdown", "github"]
        }),
    );
    for (k, v) in rule_props {
        properties.insert(k, v);
    }

    let final_schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "mkdlint configuration",
        "description": "Configuration file for mkdlint (https://github.com/192d-Wing/mkdlint)",
        "type": "object",
        "properties": serde_json::Value::Object(properties),
        "additionalProperties": {
            "description": "Rule ID or alias (true/false/severity/options)",
            "oneOf": [
                { "type": "boolean" },
                { "type": "string", "enum": ["error", "warning"] },
                { "type": "object", "additionalProperties": true }
            ]
        }
    });

    serde_json::to_string_pretty(&final_schema)
        .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}
