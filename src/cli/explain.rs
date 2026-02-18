//! `--explain <RULE>` handler — print per-rule documentation

use colored::Colorize;

/// Mapping of canonical rule ID (uppercase) to embedded doc content.
/// All docs are embedded at compile time via include_str!().
fn get_rule_doc(canonical: &str) -> Option<&'static str> {
    match canonical {
        "MD001" => Some(include_str!("../../docs/rules/md001.md")),
        "MD003" => Some(include_str!("../../docs/rules/md003.md")),
        "MD004" => Some(include_str!("../../docs/rules/md004.md")),
        "MD005" => Some(include_str!("../../docs/rules/md005.md")),
        "MD007" => Some(include_str!("../../docs/rules/md007.md")),
        "MD009" => Some(include_str!("../../docs/rules/md009.md")),
        "MD010" => Some(include_str!("../../docs/rules/md010.md")),
        "MD011" => Some(include_str!("../../docs/rules/md011.md")),
        "MD012" => Some(include_str!("../../docs/rules/md012.md")),
        "MD013" => Some(include_str!("../../docs/rules/md013.md")),
        "MD014" => Some(include_str!("../../docs/rules/md014.md")),
        "MD018" => Some(include_str!("../../docs/rules/md018.md")),
        "MD019" => Some(include_str!("../../docs/rules/md019.md")),
        "MD020" => Some(include_str!("../../docs/rules/md020.md")),
        "MD021" => Some(include_str!("../../docs/rules/md021.md")),
        "MD022" => Some(include_str!("../../docs/rules/md022.md")),
        "MD023" => Some(include_str!("../../docs/rules/md023.md")),
        "MD024" => Some(include_str!("../../docs/rules/md024.md")),
        "MD025" => Some(include_str!("../../docs/rules/md025.md")),
        "MD026" => Some(include_str!("../../docs/rules/md026.md")),
        "MD027" => Some(include_str!("../../docs/rules/md027.md")),
        "MD028" => Some(include_str!("../../docs/rules/md028.md")),
        "MD029" => Some(include_str!("../../docs/rules/md029.md")),
        "MD030" => Some(include_str!("../../docs/rules/md030.md")),
        "MD031" => Some(include_str!("../../docs/rules/md031.md")),
        "MD032" => Some(include_str!("../../docs/rules/md032.md")),
        "MD033" => Some(include_str!("../../docs/rules/md033.md")),
        "MD034" => Some(include_str!("../../docs/rules/md034.md")),
        "MD035" => Some(include_str!("../../docs/rules/md035.md")),
        "MD036" => Some(include_str!("../../docs/rules/md036.md")),
        "MD037" => Some(include_str!("../../docs/rules/md037.md")),
        "MD038" => Some(include_str!("../../docs/rules/md038.md")),
        "MD039" => Some(include_str!("../../docs/rules/md039.md")),
        "MD040" => Some(include_str!("../../docs/rules/md040.md")),
        "MD041" => Some(include_str!("../../docs/rules/md041.md")),
        "MD042" => Some(include_str!("../../docs/rules/md042.md")),
        "MD043" => Some(include_str!("../../docs/rules/md043.md")),
        "MD044" => Some(include_str!("../../docs/rules/md044.md")),
        "MD045" => Some(include_str!("../../docs/rules/md045.md")),
        "MD046" => Some(include_str!("../../docs/rules/md046.md")),
        "MD047" => Some(include_str!("../../docs/rules/md047.md")),
        "MD048" => Some(include_str!("../../docs/rules/md048.md")),
        "MD049" => Some(include_str!("../../docs/rules/md049.md")),
        "MD050" => Some(include_str!("../../docs/rules/md050.md")),
        "MD051" => Some(include_str!("../../docs/rules/md051.md")),
        "MD052" => Some(include_str!("../../docs/rules/md052.md")),
        "MD053" => Some(include_str!("../../docs/rules/md053.md")),
        "MD054" => Some(include_str!("../../docs/rules/md054.md")),
        "MD055" => Some(include_str!("../../docs/rules/md055.md")),
        "MD056" => Some(include_str!("../../docs/rules/md056.md")),
        "MD058" => Some(include_str!("../../docs/rules/md058.md")),
        "MD059" => Some(include_str!("../../docs/rules/md059.md")),
        "MD060" => Some(include_str!("../../docs/rules/md060.md")),
        "KMD001" => Some(include_str!("../../docs/rules/kmd001.md")),
        "KMD002" => Some(include_str!("../../docs/rules/kmd002.md")),
        "KMD003" => Some(include_str!("../../docs/rules/kmd003.md")),
        "KMD004" => Some(include_str!("../../docs/rules/kmd004.md")),
        "KMD005" => Some(include_str!("../../docs/rules/kmd005.md")),
        "KMD006" => Some(include_str!("../../docs/rules/kmd006.md")),
        "KMD007" => Some(include_str!("../../docs/rules/kmd007.md")),
        "KMD008" => Some(include_str!("../../docs/rules/kmd008.md")),
        "KMD009" => Some(include_str!("../../docs/rules/kmd009.md")),
        "KMD010" => Some(include_str!("../../docs/rules/kmd010.md")),
        "KMD011" => Some(include_str!("../../docs/rules/kmd011.md")),
        _ => None,
    }
}

/// Print per-rule documentation to stdout with basic terminal highlighting.
pub(crate) fn explain_rule(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let rule = match mkdlint::rules::find_rule(name) {
        Some(r) => r,
        None => {
            eprintln!("{} unknown rule '{}'", "error:".red().bold(), name);
            suggest_similar_rules(name);
            std::process::exit(1);
        }
    };

    let canonical = rule.names()[0];

    match get_rule_doc(canonical) {
        Some(doc) => {
            for line in doc.lines() {
                if line.starts_with("# ") {
                    println!("{}", line.bold().cyan());
                } else if line.starts_with("## ") {
                    println!("{}", line.bold().yellow());
                } else if line.starts_with("### ") {
                    println!("{}", line.bold());
                } else if line.starts_with("```") {
                    println!("{}", line.dimmed());
                } else {
                    println!("{}", line);
                }
            }
            Ok(())
        }
        None => {
            eprintln!(
                "{} documentation not found for rule '{}'",
                "error:".red().bold(),
                canonical
            );
            std::process::exit(1);
        }
    }
}

/// Suggest rules with similar names on lookup failure.
fn suggest_similar_rules(name: &str) {
    let name_upper = name.to_uppercase();

    let mut suggestions: Vec<(&str, &str)> = Vec::new();
    for rule in mkdlint::rules::get_rules().iter() {
        let names = rule.names();
        for n in names {
            if n.to_uppercase().contains(&name_upper) || name_upper.contains(&n.to_uppercase()) {
                suggestions.push((names[0], names.get(1).copied().unwrap_or("")));
                break;
            }
        }
    }

    if !suggestions.is_empty() {
        eprintln!("\nDid you mean one of these?");
        for (id, alias) in suggestions.iter().take(5) {
            if alias.is_empty() {
                eprintln!("  {}", id);
            } else {
                eprintln!("  {} ({})", id, alias);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_rules_have_docs() {
        for rule in mkdlint::rules::get_rules().iter() {
            let canonical = rule.names()[0];
            assert!(
                get_rule_doc(canonical).is_some(),
                "Missing documentation for rule {}",
                canonical
            );
        }
    }

    #[test]
    fn test_doc_content_not_empty() {
        for rule in mkdlint::rules::get_rules().iter() {
            let canonical = rule.names()[0];
            let doc = get_rule_doc(canonical).unwrap();
            assert!(
                !doc.is_empty(),
                "Empty documentation for rule {}",
                canonical
            );
            assert!(
                doc.contains(&format!("# {}", canonical)),
                "Documentation for {} should contain the rule name in the title",
                canonical
            );
        }
    }

    #[test]
    fn test_alias_lookup_resolves_to_doc() {
        // "heading-increment" is an alias for MD001
        let rule = mkdlint::rules::find_rule("heading-increment").unwrap();
        assert_eq!(rule.names()[0], "MD001");
        assert!(get_rule_doc("MD001").is_some());
    }

    #[test]
    fn test_unknown_rule_returns_none() {
        assert!(get_rule_doc("NONEXISTENT").is_none());
    }
}
