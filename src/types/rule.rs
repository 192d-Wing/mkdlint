//! Rule trait and related types

use crate::parser::Token;
use crate::types::LintError;
use std::collections::HashMap;

#[cfg(feature = "async")]
use std::future::Future;
#[cfg(feature = "async")]
use std::pin::Pin;

/// Parser type required by a rule
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserType {
    /// Micromark parser (token-based)
    Micromark,
    /// No parser needed (text analysis only)
    None,
}

/// Parameters passed to a rule's lint function
pub struct RuleParams<'a> {
    /// Name or identifier for the content being linted
    pub name: &'a str,

    /// Library version
    pub version: &'a str,

    /// Lines of the markdown content (including line endings)
    pub lines: &'a [String],

    /// Front matter lines (if present)
    pub front_matter_lines: &'a [String],

    /// Parsed tokens from the markdown content
    pub tokens: &'a [Token],

    /// Rule-specific configuration
    pub config: &'a HashMap<String, serde_json::Value>,
}

/// Callback type for reporting errors
pub type OnErrorFn<'a> = &'a mut dyn FnMut(LintError);

/// Trait that all rules must implement
pub trait Rule: Send + Sync {
    /// Get the rule names (first is primary, rest are aliases)
    ///
    /// Example: `["MD001", "heading-increment"]`
    fn names(&self) -> &[&'static str];

    /// Get the rule description
    ///
    /// Example: "Heading levels should only increment by one level at a time"
    fn description(&self) -> &'static str;

    /// Get the rule tags (categories)
    ///
    /// Example: `["headings"]`
    fn tags(&self) -> &[&'static str];

    /// Get the parser type required by this rule
    fn parser_type(&self) -> ParserType {
        ParserType::Micromark
    }

    /// Get the URL with more information about this rule
    fn information(&self) -> Option<&'static str> {
        None
    }

    /// Whether this rule is asynchronous
    fn is_async(&self) -> bool {
        false
    }

    /// Lint the markdown content (synchronous)
    fn lint(&self, params: &RuleParams) -> Vec<LintError>;

    /// Lint the markdown content (asynchronous)
    #[cfg(feature = "async")]
    fn lint_async<'a>(
        &'a self,
        params: &'a RuleParams<'a>,
    ) -> Pin<Box<dyn Future<Output = Vec<LintError>> + Send + 'a>> {
        Box::pin(async move { self.lint(params) })
    }
}

/// Type-erased rule reference
pub type BoxedRule = Box<dyn Rule>;

/// Helper trait for creating rule registries
pub trait RuleRegistry {
    /// Get all rules in the registry
    fn rules(&self) -> &[BoxedRule];

    /// Find a rule by name or alias
    fn find_rule(&self, name: &str) -> Option<&dyn Rule> {
        let name_upper = name.to_uppercase();
        self.rules().iter().find_map(|rule| {
            if rule
                .names()
                .iter()
                .any(|n| n.to_uppercase() == name_upper)
            {
                Some(&**rule)
            } else {
                None
            }
        })
    }

    /// Find rules by tag
    fn find_rules_by_tag(&self, tag: &str) -> Vec<&dyn Rule> {
        let tag_upper = tag.to_uppercase();
        self.rules()
            .iter()
            .filter(|rule| {
                rule.tags()
                    .iter()
                    .any(|t| t.to_uppercase() == tag_upper)
            })
            .map(|r| &**r)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRule;

    impl Rule for TestRule {
        fn names(&self) -> &[&'static str] {
            &["TEST001", "test-rule"]
        }

        fn description(&self) -> &'static str {
            "Test rule"
        }

        fn tags(&self) -> &[&'static str] {
            &["test"]
        }

        fn lint(&self, _params: &RuleParams) -> Vec<LintError> {
            vec![]
        }
    }

    #[test]
    fn test_rule_names() {
        let rule = TestRule;
        assert_eq!(rule.names(), &["TEST001", "test-rule"]);
        assert_eq!(rule.description(), "Test rule");
        assert_eq!(rule.tags(), &["test"]);
    }
}
