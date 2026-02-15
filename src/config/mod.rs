//! Configuration parsing and management

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
            Some(RuleConfig::Options(opts)) => {
                opts.get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true)
            }
            None => self.default.unwrap_or(true),
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
}
