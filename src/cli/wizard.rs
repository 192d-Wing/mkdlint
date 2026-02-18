//! Interactive configuration wizard

/// Config options collected from wizard
pub(crate) struct ConfigOptions<'a> {
    pub(crate) line_length: usize,
    pub(crate) heading_style: &'a str,
    pub(crate) list_marker: &'a str,
    pub(crate) emphasis_style: &'a str,
    pub(crate) strong_style: &'a str,
    pub(crate) allow_html: bool,
    pub(crate) allowed_elements: Vec<&'a str>,
    pub(crate) code_style: &'a str,
    pub(crate) disabled_rules: Vec<&'a str>,
}

/// Generate configuration content based on wizard answers
pub(crate) fn generate_config(format: &str, options: &ConfigOptions) -> String {
    match format {
        "json" => generate_json_config(options),
        "yaml" => generate_yaml_config(options),
        "toml" => generate_toml_config(options),
        _ => String::new(),
    }
}

pub(crate) fn generate_json_config(options: &ConfigOptions) -> String {
    let schema_url =
        "https://raw.githubusercontent.com/192d-Wing/mkdlint/main/schema/mkdlint-schema.json";
    let mut config = format!("{{\n  \"$schema\": \"{schema_url}\",\n  \"default\": true");

    // Disabled rules
    for rule in &options.disabled_rules {
        config.push_str(&format!(",\n  \"{}\": false", rule));
    }

    // Line length
    if options.line_length > 0 {
        config.push_str(&format!(
            ",\n  \"MD013\": {{\n    \"line_length\": {},\n    \"code_blocks\": false,\n    \"tables\": false\n  }}",
            options.line_length
        ));
    }

    // Heading style (MD003)
    if options.heading_style != "consistent" {
        config.push_str(&format!(
            ",\n  \"MD003\": {{\n    \"style\": \"{}\"\n  }}",
            options.heading_style
        ));
    }

    // List marker style (MD004)
    if options.list_marker != "consistent" {
        config.push_str(&format!(
            ",\n  \"MD004\": {{\n    \"style\": \"{}\"\n  }}",
            options.list_marker
        ));
    }

    // Inline HTML (MD033)
    if options.allow_html && !options.allowed_elements.is_empty() {
        let elements_json = options
            .allowed_elements
            .iter()
            .map(|e| format!("\"{}\"", e))
            .collect::<Vec<_>>()
            .join(", ");
        config.push_str(&format!(
            ",\n  \"MD033\": {{\n    \"allowed_elements\": [{}]\n  }}",
            elements_json
        ));
    }

    // Code block style (MD046)
    if options.code_style != "consistent" {
        config.push_str(&format!(
            ",\n  \"MD046\": {{\n    \"style\": \"{}\"\n  }}",
            options.code_style
        ));
    }

    // Emphasis style (MD049)
    if options.emphasis_style != "consistent" {
        config.push_str(&format!(
            ",\n  \"MD049\": {{\n    \"style\": \"{}\"\n  }}",
            options.emphasis_style
        ));
    }

    // Strong emphasis style (MD050)
    if options.strong_style != "consistent" {
        config.push_str(&format!(
            ",\n  \"MD050\": {{\n    \"style\": \"{}\"\n  }}",
            options.strong_style
        ));
    }

    config.push_str("\n}\n");
    config
}

pub(crate) fn generate_yaml_config(options: &ConfigOptions) -> String {
    let mut config = String::from(
        "# mkdlint configuration\n# Schema: https://raw.githubusercontent.com/192d-Wing/mkdlint/main/schema/mkdlint-schema.json\ndefault: true\n",
    );

    // Disabled rules
    for rule in &options.disabled_rules {
        config.push_str(&format!("\n{}: false", rule));
    }

    // Line length
    if options.line_length > 0 {
        config.push_str(&format!(
            "\n\n# Line length\nMD013:\n  line_length: {}\n  code_blocks: false\n  tables: false",
            options.line_length
        ));
    }

    // Heading style (MD003)
    if options.heading_style != "consistent" {
        config.push_str(&format!(
            "\n\n# Heading style\nMD003:\n  style: {}",
            options.heading_style
        ));
    }

    // List marker style (MD004)
    if options.list_marker != "consistent" {
        config.push_str(&format!(
            "\n\n# List marker\nMD004:\n  style: {}",
            options.list_marker
        ));
    }

    // Inline HTML (MD033)
    if options.allow_html && !options.allowed_elements.is_empty() {
        config.push_str("\n\n# Inline HTML\nMD033:\n  allowed_elements:");
        for element in &options.allowed_elements {
            config.push_str(&format!("\n    - {}", element));
        }
    }

    // Code block style (MD046)
    if options.code_style != "consistent" {
        config.push_str(&format!(
            "\n\n# Code block style\nMD046:\n  style: {}",
            options.code_style
        ));
    }

    // Emphasis style (MD049)
    if options.emphasis_style != "consistent" {
        config.push_str(&format!(
            "\n\n# Emphasis style\nMD049:\n  style: {}",
            options.emphasis_style
        ));
    }

    // Strong emphasis style (MD050)
    if options.strong_style != "consistent" {
        config.push_str(&format!(
            "\n\n# Strong emphasis style\nMD050:\n  style: {}",
            options.strong_style
        ));
    }

    config.push('\n');
    config
}

pub(crate) fn generate_toml_config(options: &ConfigOptions) -> String {
    let mut config = String::from(
        "# mkdlint configuration\n# Schema: https://raw.githubusercontent.com/192d-Wing/mkdlint/main/schema/mkdlint-schema.json\ndefault = true\n",
    );

    // Disabled rules
    for rule in &options.disabled_rules {
        config.push_str(&format!("\n{} = false", rule));
    }

    // Line length
    if options.line_length > 0 {
        config.push_str(&format!(
            "\n\n# Line length\n[MD013]\nline_length = {}\ncode_blocks = false\ntables = false",
            options.line_length
        ));
    }

    // Heading style (MD003)
    if options.heading_style != "consistent" {
        config.push_str(&format!(
            "\n\n# Heading style\n[MD003]\nstyle = \"{}\"",
            options.heading_style
        ));
    }

    // List marker style (MD004)
    if options.list_marker != "consistent" {
        config.push_str(&format!(
            "\n\n# List marker\n[MD004]\nstyle = \"{}\"",
            options.list_marker
        ));
    }

    // Inline HTML (MD033)
    if options.allow_html && !options.allowed_elements.is_empty() {
        let elements_toml = options
            .allowed_elements
            .iter()
            .map(|e| format!("\"{}\"", e))
            .collect::<Vec<_>>()
            .join(", ");
        config.push_str(&format!(
            "\n\n# Inline HTML\n[MD033]\nallowed_elements = [{}]",
            elements_toml
        ));
    }

    // Code block style (MD046)
    if options.code_style != "consistent" {
        config.push_str(&format!(
            "\n\n# Code block style\n[MD046]\nstyle = \"{}\"",
            options.code_style
        ));
    }

    // Emphasis style (MD049)
    if options.emphasis_style != "consistent" {
        config.push_str(&format!(
            "\n\n# Emphasis style\n[MD049]\nstyle = \"{}\"",
            options.emphasis_style
        ));
    }

    // Strong emphasis style (MD050)
    if options.strong_style != "consistent" {
        config.push_str(&format!(
            "\n\n# Strong emphasis style\n[MD050]\nstyle = \"{}\"",
            options.strong_style
        ));
    }

    config.push('\n');
    config
}

/// Interactive configuration wizard
pub(crate) fn init_config_interactive(
    output_path: &str,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::Colorize;
    use dialoguer::{Confirm, Input, MultiSelect, Select};

    println!("{}", "mkdlint Configuration Wizard".cyan().bold());
    println!();
    println!("This wizard will help you create a custom configuration file.");
    println!();

    // Question 1: Format preference (already set via --format, but allow override)
    let formats = vec!["JSON", "YAML", "TOML"];
    let default_format_idx = match format {
        "yaml" | "yml" => 1,
        "toml" => 2,
        _ => 0,
    };
    let format_idx = Select::new()
        .with_prompt("What format would you like for your config file?")
        .items(&formats)
        .default(default_format_idx)
        .interact()?;
    let selected_format = match format_idx {
        1 => "yaml",
        2 => "toml",
        _ => "json",
    };

    // Question 2: Line length
    let line_length: usize = Input::new()
        .with_prompt("Maximum line length (0 to disable)")
        .default(120)
        .interact()?;

    // Question 3: Heading style
    let heading_styles = vec![
        "ATX (# Heading)",
        "Setext (Underlined)",
        "Consistent (auto-detect)",
    ];
    let heading_style_idx = Select::new()
        .with_prompt("Preferred heading style?")
        .items(&heading_styles)
        .default(0)
        .interact()?;
    let heading_style = match heading_style_idx {
        1 => "setext",
        2 => "consistent",
        _ => "atx",
    };

    // Question 4: List marker style
    let list_markers = vec![
        "Dash (-)",
        "Asterisk (*)",
        "Plus (+)",
        "Consistent (auto-detect)",
    ];
    let list_marker_idx = Select::new()
        .with_prompt("Preferred unordered list marker?")
        .items(&list_markers)
        .default(0)
        .interact()?;
    let list_marker = match list_marker_idx {
        1 => "asterisk",
        2 => "plus",
        3 => "consistent",
        _ => "dash",
    };

    // Question 5: Emphasis style
    let emphasis_styles = vec!["Asterisk (*text*)", "Underscore (_text_)", "Consistent"];
    let emphasis_style_idx = Select::new()
        .with_prompt("Preferred emphasis style?")
        .items(&emphasis_styles)
        .default(0)
        .interact()?;
    let emphasis_style = match emphasis_style_idx {
        1 => "underscore",
        2 => "consistent",
        _ => "asterisk",
    };

    // Question 6: Strong emphasis style
    let strong_styles = vec!["Asterisk (**text**)", "Underscore (__text__)", "Consistent"];
    let strong_style_idx = Select::new()
        .with_prompt("Preferred strong emphasis style?")
        .items(&strong_styles)
        .default(0)
        .interact()?;
    let strong_style = match strong_style_idx {
        1 => "underscore",
        2 => "consistent",
        _ => "asterisk",
    };

    // Question 7: Inline HTML
    let allow_html = Confirm::new()
        .with_prompt("Allow inline HTML in markdown?")
        .default(false)
        .interact()?;

    let allowed_elements = if allow_html {
        let elements = vec!["br", "img", "details", "summary", "div", "span"];
        let selected = MultiSelect::new()
            .with_prompt(
                "Which HTML elements should be allowed? (use Space to select, Enter to confirm)",
            )
            .items(&elements)
            .defaults(&[true, true, true, true, false, false])
            .interact()?;

        selected
            .iter()
            .map(|&idx| elements[idx])
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    // Question 8: Code block style
    let code_styles = vec!["Fenced (```)", "Indented (4 spaces)", "Consistent"];
    let code_style_idx = Select::new()
        .with_prompt("Preferred code block style?")
        .items(&code_styles)
        .default(0)
        .interact()?;
    let code_style = match code_style_idx {
        1 => "indented",
        2 => "consistent",
        _ => "fenced",
    };

    // Question 9: Rules to disable
    let common_rules = vec![
        "MD013 (Line length)",
        "MD033 (Inline HTML)",
        "MD034 (Bare URLs)",
        "MD041 (First line H1)",
    ];
    let disabled_rules_selection = MultiSelect::new()
        .with_prompt("Which rules would you like to disable? (optional)")
        .items(&common_rules)
        .interact()?;

    let disabled_rules: Vec<&str> = disabled_rules_selection
        .iter()
        .map(|&idx| match idx {
            0 => "MD013",
            1 => "MD033",
            2 => "MD034",
            3 => "MD041",
            _ => "",
        })
        .filter(|s| !s.is_empty())
        .collect();

    println!();
    println!("{}", "Generating configuration...".green());

    // Build config options
    let options = ConfigOptions {
        line_length,
        heading_style,
        list_marker,
        emphasis_style,
        strong_style,
        allow_html,
        allowed_elements,
        code_style,
        disabled_rules,
    };

    // Generate configuration based on answers
    let content = generate_config(selected_format, &options);

    // Update output path extension if format changed
    let output_path = if selected_format != format {
        match selected_format {
            "yaml" => output_path
                .replace(".json", ".yaml")
                .replace(".toml", ".yaml"),
            "toml" => output_path
                .replace(".json", ".toml")
                .replace(".yaml", ".toml"),
            _ => output_path
                .replace(".yaml", ".json")
                .replace(".toml", ".json"),
        }
    } else {
        output_path.to_string()
    };

    // Write to file
    std::fs::write(&output_path, content)?;

    println!();
    println!(
        "{} Created configuration file: {}",
        "✓".green().bold(),
        output_path.cyan()
    );
    println!();
    println!("Next steps:");
    println!(
        "  1. Review and edit {} to fine-tune rules",
        output_path.cyan()
    );
    println!(
        "  2. Run: {} {} {}",
        "mkdlint".cyan(),
        "--config".yellow(),
        output_path.yellow()
    );
    println!(
        "  3. Auto-fix issues: {} {} {} {}",
        "mkdlint".cyan(),
        "--fix".yellow(),
        "--config".yellow(),
        output_path.yellow()
    );

    Ok(())
}
