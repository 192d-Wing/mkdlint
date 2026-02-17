//! Built-in and custom rules

use crate::types::{BoxedRule, Rule};
use once_cell::sync::Lazy;

// ALL 54 RULES IMPLEMENTED!
// Kramdown extension rules (KMD)
mod kmd001;
mod kmd002;
mod kmd003;
mod kmd004;
mod kmd005;
mod kmd006;
mod kmd007;
mod kmd008;
mod kmd009;
mod kmd010;
mod kmd011;

mod md001;
mod md003;
mod md004;
mod md005;
mod md007;
mod md009;
mod md010;
mod md011;
mod md012;
mod md013;
mod md014;
mod md018;
mod md019;
mod md020;
mod md021;
mod md022;
mod md023;
mod md024;
mod md025;
mod md026;
mod md027;
mod md028;
mod md029;
mod md030;
mod md031;
mod md032;
mod md033;
mod md034;
mod md035;
mod md036;
mod md037;
mod md038;
mod md039;
mod md040;
mod md041;
mod md042;
mod md043;
mod md044;
mod md045;
mod md046;
mod md047;
mod md048;
mod md049;
mod md050;
mod md051;
mod md052;
mod md053;
mod md054;
mod md055;
mod md056;
mod md058;
mod md059;
mod md060;

/// Global rule registry - standard + Kramdown extension rules
pub static RULES: Lazy<Vec<BoxedRule>> = Lazy::new(|| {
    vec![
        // Kramdown extension rules (disabled by default; enabled by kramdown preset)
        Box::new(kmd001::KMD001),
        Box::new(kmd002::KMD002),
        Box::new(kmd003::KMD003),
        Box::new(kmd004::KMD004),
        Box::new(kmd005::KMD005),
        Box::new(kmd006::KMD006),
        Box::new(kmd007::KMD007),
        Box::new(kmd008::KMD008),
        Box::new(kmd009::KMD009),
        Box::new(kmd010::KMD010),
        Box::new(kmd011::KMD011),
        // Standard markdownlint rules
        Box::new(md001::MD001),
        Box::new(md003::MD003),
        Box::new(md004::MD004),
        Box::new(md005::MD005),
        Box::new(md007::MD007),
        Box::new(md009::MD009),
        Box::new(md010::MD010),
        Box::new(md011::MD011),
        Box::new(md012::MD012),
        Box::new(md013::MD013),
        Box::new(md014::MD014),
        Box::new(md018::MD018),
        Box::new(md019::MD019),
        Box::new(md020::MD020),
        Box::new(md021::MD021),
        Box::new(md022::MD022),
        Box::new(md023::MD023),
        Box::new(md024::MD024),
        Box::new(md025::MD025),
        Box::new(md026::MD026),
        Box::new(md027::MD027),
        Box::new(md028::MD028),
        Box::new(md029::MD029),
        Box::new(md030::MD030),
        Box::new(md031::MD031),
        Box::new(md032::MD032),
        Box::new(md033::MD033),
        Box::new(md034::MD034),
        Box::new(md035::MD035),
        Box::new(md036::MD036),
        Box::new(md037::MD037),
        Box::new(md038::MD038),
        Box::new(md039::MD039),
        Box::new(md040::MD040),
        Box::new(md041::MD041),
        Box::new(md042::MD042),
        Box::new(md043::MD043),
        Box::new(md044::MD044),
        Box::new(md045::MD045),
        Box::new(md046::MD046),
        Box::new(md047::MD047),
        Box::new(md048::MD048),
        Box::new(md049::MD049),
        Box::new(md050::MD050),
        Box::new(md051::MD051),
        Box::new(md052::MD052),
        Box::new(md053::MD053),
        Box::new(md054::MD054),
        Box::new(md055::MD055),
        Box::new(md056::MD056),
        Box::new(md058::MD058),
        Box::new(md059::MD059),
        Box::new(md060::MD060),
    ]
});

/// Get all built-in rules
pub fn get_rules() -> &'static [BoxedRule] {
    &RULES
}

/// Find a rule by name
pub fn find_rule(name: &str) -> Option<&'static dyn Rule> {
    let name_upper = name.to_uppercase();
    RULES.iter().find_map(|rule| {
        if rule.names().iter().any(|n| n.to_uppercase() == name_upper) {
            Some(&**rule)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_counts() {
        let rules = get_rules();
        // 53 standard rules (MD001-MD060 minus 7 deprecated: MD002, MD006, MD008, MD015, MD016, MD017, MD057)
        // + 11 Kramdown extension rules (KMD001-KMD011)
        assert_eq!(
            rules.len(),
            64,
            "Should have 53 standard + 11 KMD extension rules"
        );
    }

    #[test]
    fn test_find_rule_by_id() {
        assert!(find_rule("MD001").is_some());
        assert!(find_rule("MD007").is_some());
        assert!(find_rule("MD009").is_some());
        assert!(find_rule("MD030").is_some());
        assert!(find_rule("MD047").is_some());
        assert!(find_rule("KMD001").is_some());
        assert!(find_rule("KMD006").is_some());
    }

    #[test]
    fn test_find_rule_by_alias() {
        assert!(find_rule("ul-indent").is_some());
        assert!(find_rule("no-trailing-spaces").is_some());
        assert!(find_rule("no-hard-tabs").is_some());
        assert!(find_rule("list-marker-space").is_some());
    }
}
