//! `--list-rules` and `--list-presets` handlers

/// List all available linting rules, optionally filtered/annotated by a preset
pub(crate) fn list_rules(preset: &Option<String>) {
    use colored::Colorize;
    use mkdlint::config::presets::resolve_preset;
    use mkdlint::rules::get_rules;

    // Resolve preset config to show which rules it enables/disables
    let preset_config = preset.as_deref().and_then(resolve_preset);

    if let Some(p) = preset {
        println!(
            "{}",
            format!("Available Linting Rules (preset: {p})")
                .bold()
                .underline()
        );
    } else {
        println!("{}", "Available Linting Rules".bold().underline());
    }
    println!();

    let rules = get_rules();
    let mut rules_info: Vec<_> = rules
        .iter()
        .map(|r| {
            let names = r.names();
            let description = r.description();
            let tags = r.tags();
            let fixable = if tags.contains(&"fixable") {
                "✓"
            } else {
                " "
            };
            let alias = if names.len() > 1 { names[1] } else { "" };
            let on_by_default = r.is_enabled_by_default();
            // Is this rule enabled under the given preset?
            let preset_state = preset_config.as_ref().map(|cfg| {
                if cfg.is_rule_enabled(names[0]) {
                    "enabled"
                } else {
                    "disabled"
                }
            });
            (
                names[0].to_string(),
                alias.to_string(),
                description.to_string(),
                fixable.to_string(),
                on_by_default,
                preset_state,
            )
        })
        .collect();

    // Sort by rule number (MD001, MD002, etc.)
    rules_info.sort_by(|(a, ..), (b, ..)| a.cmp(b));

    println!(
        "{:8} {:32} {:8} {}",
        "Rule".bold(),
        "Alias".bold(),
        "Fixable".bold(),
        "Description".bold()
    );
    println!("{}", "─".repeat(84));

    let mut last_prefix = "";
    for (rule_id, alias, description, fixable, on_by_default, preset_state) in &rules_info {
        // Print a blank separator line between MD and KMD groups
        let prefix = if rule_id.starts_with("KMD") {
            "KMD"
        } else {
            "MD"
        };
        if prefix != last_prefix && !last_prefix.is_empty() {
            println!();
        }
        last_prefix = prefix;

        let fixable_mark = if fixable == "✓" {
            fixable.green()
        } else {
            fixable.normal()
        };

        // Dim rules that are off by default (KMD rules without preset)
        let id_display = if !on_by_default && preset_state.is_none() {
            rule_id.truecolor(120, 120, 120)
        } else {
            rule_id.cyan()
        };

        // Preset annotation
        let preset_mark = match preset_state {
            Some("enabled") => " ●".green(),
            Some("disabled") => " ○".red(),
            _ => "".normal(),
        };

        print!(
            "{:8} {:32} {:^8} {}",
            id_display,
            alias.yellow(),
            fixable_mark,
            description
        );
        println!("{}", preset_mark);
    }

    println!();

    let total = rules.len();
    let fixable_count = rules_info
        .iter()
        .filter(|(_, _, _, f, ..)| f == "✓")
        .count();
    let off_by_default = rules_info.iter().filter(|(_, _, _, _, d, _)| !d).count();

    if let Some(p) = preset {
        let enabled_by_preset = rules_info
            .iter()
            .filter(|(_, _, _, _, _, ps)| ps.as_deref() == Some("enabled"))
            .count();
        println!("Total: {total} rules ({fixable_count} fixable, {off_by_default} off-by-default)");
        println!("Preset '{p}': {enabled_by_preset} rules enabled  ● = enabled  ○ = disabled");
    } else {
        println!("Total: {total} rules ({fixable_count} fixable, {off_by_default} off-by-default)");
        println!("Tip: use --preset <name> to see how a preset changes rule states");
    }
}

/// List all available named presets
pub(crate) fn list_presets() {
    use colored::Colorize;
    use mkdlint::config::presets::{preset_names, resolve_preset};
    use mkdlint::rules::get_rules;

    println!("{}", "Available Presets".bold().underline());
    println!();

    let all_rules = get_rules();
    for name in preset_names() {
        let config = match resolve_preset(name) {
            Some(c) => c,
            None => continue,
        };

        // Only show rules explicitly set in the preset's rule map
        let enabled: Vec<&str> = all_rules
            .iter()
            .filter(|r| {
                let id = r.names()[0];
                matches!(
                    config.get_rule_config(id),
                    Some(mkdlint::config::RuleConfig::Enabled(true))
                )
            })
            .map(|r| r.names()[0])
            .collect();

        let disabled: Vec<&str> = all_rules
            .iter()
            .filter(|r| {
                let id = r.names()[0];
                matches!(
                    config.get_rule_config(id),
                    Some(mkdlint::config::RuleConfig::Enabled(false))
                )
            })
            .map(|r| r.names()[0])
            .collect();

        let configured: Vec<&str> = all_rules
            .iter()
            .filter(|r| {
                let id = r.names()[0];
                matches!(
                    config.get_rule_config(id),
                    Some(mkdlint::config::RuleConfig::Options(_))
                )
            })
            .map(|r| r.names()[0])
            .collect();

        println!("  {}", name.cyan().bold());
        if !enabled.is_empty() {
            println!("    {} {}", "Enables: ".green(), enabled.join(", "));
        }
        if !disabled.is_empty() {
            println!("    {} {}", "Disables:".red(), disabled.join(", "));
        }
        if !configured.is_empty() {
            println!("    {} {}", "Options: ".yellow(), configured.join(", "));
        }
        println!();
    }

    println!("Use {} to apply a preset.", "--preset <name>".yellow());
    println!(
        "Use {} to see rule states with a preset applied.",
        "--list-rules --preset <name>".yellow()
    );
}
