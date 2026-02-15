//! Lint results types

use crate::types::LintError;
use std::collections::HashMap;
use std::fmt;

/// Results from linting operations
#[derive(Debug, Clone, Default)]
pub struct LintResults {
    /// Map of file/string name to lint errors
    pub results: HashMap<String, Vec<LintError>>,
}

impl LintResults {
    /// Create a new empty LintResults
    pub fn new() -> Self {
        Self::default()
    }

    /// Add results for a file or string
    pub fn add(&mut self, name: String, errors: Vec<LintError>) {
        self.results.insert(name, errors);
    }

    /// Get errors for a specific file or string
    pub fn get(&self, name: &str) -> Option<&[LintError]> {
        self.results.get(name).map(|v| v.as_slice())
    }

    /// Get total number of errors across all files
    pub fn error_count(&self) -> usize {
        self.results
            .values()
            .map(|errors| errors.iter().filter(|e| e.severity == crate::types::Severity::Error).count())
            .sum()
    }

    /// Get total number of warnings across all files
    pub fn warning_count(&self) -> usize {
        self.results
            .values()
            .map(|errors| errors.iter().filter(|e| e.severity == crate::types::Severity::Warning).count())
            .sum()
    }

    /// Check if there are any errors (not warnings)
    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    /// Check if results are empty (no errors or warnings)
    pub fn is_empty(&self) -> bool {
        self.results.values().all(|v| v.is_empty())
    }

    /// Get all file/string names with errors
    pub fn files_with_errors(&self) -> Vec<&str> {
        self.results
            .iter()
            .filter(|(_, errors)| !errors.is_empty())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Format results as a string (similar to toString in JS version)
    pub fn to_string_with_alias(&self, use_alias: bool) -> String {
        let mut output = Vec::new();
        let mut files: Vec<_> = self.results.keys().collect();
        files.sort();

        for file in files {
            if let Some(errors) = self.results.get(file) {
                for error in errors {
                    let rule_moniker = if use_alias && error.rule_names.len() > 1 {
                        error.rule_names[1].clone()
                    } else {
                        error.rule_names.join("/")
                    };

                    let mut line = format!(
                        "{}: {}: {} {}",
                        file, error.line_number, rule_moniker, error.rule_description
                    );

                    if let Some(detail) = &error.error_detail {
                        line.push_str(&format!(" [{}]", detail));
                    }

                    if let Some(context) = &error.error_context {
                        line.push_str(&format!(" [Context: \"{}\"]", context));
                    }

                    output.push(line);
                }
            }
        }

        output.join("\n")
    }
}

impl fmt::Display for LintResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_alias(false))
    }
}

impl IntoIterator for LintResults {
    type Item = (String, Vec<LintError>);
    type IntoIter = std::collections::hash_map::IntoIter<String, Vec<LintError>>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.into_iter()
    }
}

impl<'a> IntoIterator for &'a LintResults {
    type Item = (&'a String, &'a Vec<LintError>);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Vec<LintError>>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Severity;

    #[test]
    fn test_lint_results() {
        let mut results = LintResults::new();

        results.add(
            "file1.md".to_string(),
            vec![LintError {
                line_number: 1,
                rule_names: vec!["MD001".to_string()],
                rule_description: "Test error".to_string(),
                severity: Severity::Error,
                ..Default::default()
            }],
        );

        results.add(
            "file2.md".to_string(),
            vec![LintError {
                line_number: 5,
                rule_names: vec!["MD003".to_string()],
                rule_description: "Test warning".to_string(),
                severity: Severity::Warning,
                ..Default::default()
            }],
        );

        assert_eq!(results.error_count(), 1);
        assert_eq!(results.warning_count(), 1);
        assert!(results.has_errors());
        assert!(!results.is_empty());
        assert_eq!(results.files_with_errors().len(), 2);
    }
}
