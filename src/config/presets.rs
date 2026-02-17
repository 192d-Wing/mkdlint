//! Named rule presets for common Markdown dialects and use cases.

use crate::config::{Config, RuleConfig};
use std::collections::HashMap;

/// Resolve a named preset to a `Config` overlay.
///
/// Returns `None` if the preset name is unknown.
pub fn resolve_preset(name: &str) -> Option<Config> {
    match name {
        "kramdown" => Some(kramdown_preset()),
        _ => None,
    }
}

/// Returns a list of known preset names (for help text / `--list-rules`).
pub fn preset_names() -> &'static [&'static str] {
    &["kramdown"]
}

/// Kramdown preset — designed for RFC and technical documents using
/// the Kramdown Markdown dialect (<https://kramdown.gettalong.org/syntax.html>).
///
/// Disables rules that conflict with Kramdown-specific syntax and enables
/// the KMD extension rules that enforce Kramdown best practices.
fn kramdown_preset() -> Config {
    let mut rules: HashMap<String, RuleConfig> = HashMap::new();

    // ── Rules disabled because they conflict with Kramdown syntax ────────────

    // MD033 (no-inline-html): Kramdown IAL syntax `{: #id .class}` looks like
    // inline HTML to standard parsers; disable to avoid false positives.
    rules.insert("MD033".to_string(), RuleConfig::Enabled(false));

    // MD041 (first-line-heading): RFC documents commonly start with metadata
    // blocks (title, author, date) rather than a heading.
    rules.insert("MD041".to_string(), RuleConfig::Enabled(false));

    // ── Kramdown extension rules (KMD) ───────────────────────────────────────
    for name in &[
        "KMD001", "KMD002", "KMD003", "KMD004", "KMD005", "KMD006", "KMD007", "KMD008", "KMD009",
        "KMD010",
    ] {
        rules.insert(name.to_string(), RuleConfig::Enabled(true));
    }

    Config {
        default: None,
        extends: None,
        preset: None,
        rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_kramdown() {
        let config = resolve_preset("kramdown").unwrap();
        assert!(!config.is_rule_enabled("MD033"));
        assert!(!config.is_rule_enabled("MD041"));
        assert!(config.is_rule_enabled("KMD001"));
        assert!(config.is_rule_enabled("KMD006"));
        assert!(config.is_rule_enabled("KMD007"));
        assert!(config.is_rule_enabled("KMD010"));
    }

    #[test]
    fn test_resolve_unknown_preset() {
        assert!(resolve_preset("nonexistent").is_none());
    }

    #[test]
    fn test_preset_names() {
        assert!(preset_names().contains(&"kramdown"));
    }
}
