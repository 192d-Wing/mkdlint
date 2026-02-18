//! Configuration parsing and management

pub mod presets;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::types::Result;

/// Configuration for markdownlint
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Default setting for all rules (true, false, or "warning")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,

    /// Path to config file to extend
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Named preset to apply (e.g., "kramdown")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,

    /// Rule-specific configuration
    #[serde(flatten)]
    pub rules: HashMap<String, RuleConfig>,
}

/// Configuration for an individual rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
    /// Simple boolean (enabled/disabled)
    Enabled(bool),

    /// String severity ("error" or "warning")
    Severity(String),

    /// Detailed configuration with options
    Options(HashMap<String, serde_json::Value>),
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a JSON file
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from a YAML file
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_yaml_ng::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from a TOML file
    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from a file (auto-detect format)
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let ext = path.extension().and_then(|e| e.to_str());

        match ext {
            Some("json") => Self::from_json_file(path),
            Some("yaml") | Some("yml") => Self::from_yaml_file(path),
            Some("toml") => Self::from_toml_file(path),
            _ => {
                // Try JSON first, then YAML, then TOML
                Self::from_json_file(path)
                    .or_else(|_| Self::from_yaml_file(path))
                    .or_else(|_| Self::from_toml_file(path))
            }
        }
    }

    /// Config file names to search for during auto-discovery
    const DISCOVERY_NAMES: [&'static str; 5] = [
        ".markdownlint.json",
        ".markdownlint.yaml",
        ".markdownlint.yml",
        ".markdownlint.toml",
        ".markdownlintrc",
    ];

    /// Walk up from `start_dir` looking for a config file
    pub fn discover(start_dir: impl AsRef<Path>) -> Option<Self> {
        let mut dir = start_dir.as_ref().to_path_buf();
        loop {
            for name in &Self::DISCOVERY_NAMES {
                let candidate = dir.join(name);
                if candidate.is_file()
                    && let Ok(config) = Self::from_file(&candidate)
                {
                    return Some(config);
                }
            }
            if !dir.pop() {
                break;
            }
        }
        None
    }

    /// Apply the named preset (if any) as a base, then re-apply explicit rules on top.
    ///
    /// Preset rules are overridden by any explicit rule config in `self`.
    pub fn apply_preset(&mut self) {
        if let Some(ref name) = self.preset.clone()
            && let Some(mut base) = presets::resolve_preset(name)
        {
            // Explicit rules in `self` override preset rules
            base.merge(self.clone());
            *self = base;
            // Preserve the preset name so callers can inspect it
            self.preset = Some(name.clone());
        }
    }

    /// Resolve the `extends` chain: load the parent config and merge self on top.
    /// Also applies any named preset after the chain is resolved.
    pub fn resolve_extends(&self) -> Result<Self> {
        if let Some(ref extends_path) = self.extends {
            let parent = Config::from_file(extends_path)?;
            let mut resolved = parent.resolve_extends()?;
            resolved.merge(self.clone());
            resolved.extends = None;
            resolved.apply_preset();
            Ok(resolved)
        } else {
            let mut resolved = self.clone();
            resolved.apply_preset();
            Ok(resolved)
        }
    }

    /// Merge another configuration into this one
    pub fn merge(&mut self, other: Config) {
        if other.default.is_some() {
            self.default = other.default;
        }
        self.rules.extend(other.rules);
    }

    /// Get effective configuration for a rule
    pub fn get_rule_config(&self, rule_name: &str) -> Option<&RuleConfig> {
        self.rules.get(rule_name)
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        match self.get_rule_config(rule_name) {
            Some(RuleConfig::Enabled(enabled)) => *enabled,
            Some(RuleConfig::Severity(_)) => true,
            Some(RuleConfig::Options(opts)) => opts
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            None => self.default.unwrap_or(true),
        }
    }

    /// Get the configured severity for a rule, if set.
    ///
    /// Returns None if no explicit severity is configured (rule uses its default).
    /// Supports both `"MD001": "warning"` and `"MD001": {"severity": "warning"}` formats.
    pub fn get_rule_severity(&self, rule_name: &str) -> Option<crate::types::Severity> {
        match self.get_rule_config(rule_name) {
            Some(RuleConfig::Severity(s)) => match s.to_lowercase().as_str() {
                "warning" | "warn" => Some(crate::types::Severity::Warning),
                "error" => Some(crate::types::Severity::Error),
                _ => None,
            },
            Some(RuleConfig::Options(opts)) => opts
                .get("severity")
                .and_then(|v| v.as_str())
                .and_then(|s| match s.to_lowercase().as_str() {
                    "warning" | "warn" => Some(crate::types::Severity::Warning),
                    "error" => Some(crate::types::Severity::Error),
                    _ => None,
                }),
            _ => None,
        }
    }
}

/// Configuration parser trait for custom formats
pub trait ConfigParser {
    /// Parse configuration from a string
    fn parse(&self, content: &str) -> Result<Config>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config::new();
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_json_parsing() {
        let json = r#"{"default": true, "MD001": false}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.default, Some(true));
        assert!(!config.is_rule_enabled("MD001"));
    }

    #[test]
    fn test_discover_json() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join(".markdownlint.json");
        std::fs::write(&config_path, r#"{"default": false}"#).unwrap();

        let config = Config::discover(dir.path()).unwrap();
        assert_eq!(config.default, Some(false));
    }

    #[test]
    fn test_discover_walks_up() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub").join("deep");
        std::fs::create_dir_all(&sub).unwrap();
        let config_path = dir.path().join(".markdownlint.json");
        std::fs::write(&config_path, r#"{"MD001": false}"#).unwrap();

        let config = Config::discover(&sub).unwrap();
        assert!(!config.is_rule_enabled("MD001"));
    }

    #[test]
    fn test_discover_none_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Config::discover(dir.path()).is_none());
    }

    #[test]
    fn test_discover_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join(".markdownlint.yaml");
        std::fs::write(&config_path, "default: false\n").unwrap();

        let config = Config::discover(dir.path()).unwrap();
        assert_eq!(config.default, Some(false));
    }

    #[test]
    fn test_resolve_extends() {
        let dir = tempfile::tempdir().unwrap();
        let base_path = dir.path().join("base.json");
        std::fs::write(&base_path, r#"{"default": true, "MD001": false}"#).unwrap();

        let child_json = format!(
            r#"{{"extends": "{}", "MD013": false}}"#,
            base_path.to_str().unwrap().replace('\\', "\\\\")
        );
        let child: Config = serde_json::from_str(&child_json).unwrap();
        let resolved = child.resolve_extends().unwrap();

        assert_eq!(resolved.default, Some(true));
        assert!(!resolved.is_rule_enabled("MD001"));
        assert!(!resolved.is_rule_enabled("MD013"));
        assert!(resolved.extends.is_none());
    }

    #[test]
    fn test_resolve_extends_no_extends() {
        let config = Config::new();
        let resolved = config.resolve_extends().unwrap();
        assert!(resolved.extends.is_none());
    }

    #[test]
    fn test_get_rule_severity_warning() {
        let json = r#"{"MD001": "warning"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.get_rule_severity("MD001"),
            Some(crate::types::Severity::Warning)
        );
        assert_eq!(config.get_rule_severity("MD002"), None);
    }

    #[test]
    fn test_get_rule_severity_error_string() {
        let json = r#"{"MD001": "error"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.get_rule_severity("MD001"),
            Some(crate::types::Severity::Error)
        );
    }

    #[test]
    fn test_get_rule_severity_in_options() {
        let json = r#"{"MD013": {"severity": "warning", "line_length": 100}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.get_rule_severity("MD013"),
            Some(crate::types::Severity::Warning)
        );
    }

    #[test]
    fn test_get_rule_severity_warn_alias() {
        let json = r#"{"MD001": "warn"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.get_rule_severity("MD001"),
            Some(crate::types::Severity::Warning)
        );
    }
}
