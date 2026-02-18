//! Command-line interface for mkdlint

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use mkdlint::{LintOptions, apply_fixes, formatters, lint_sync};

#[cfg(feature = "cli")]
#[derive(clap::ValueEnum, Clone, Debug, Default)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
}

#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[command(name = "mkdlint")]
#[command(about = "A linter for Markdown files", long_about = None)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// Files or directories to lint
    #[arg(global = true)]
    files: Vec<String>,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Output format
    #[arg(short = 'o', long, default_value = "text", global = true)]
    output_format: OutputFormat,

    /// Glob patterns for files to ignore (repeatable)
    #[arg(long, action = clap::ArgAction::Append, global = true)]
    ignore: Vec<String>,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Disable inline configuration comments
    #[arg(long, global = true)]
    no_inline_config: bool,

    /// Automatically fix violations where possible
    #[arg(short, long, global = true)]
    fix: bool,

    /// Show what --fix would change without writing any files
    #[arg(long, global = true)]
    fix_dry_run: bool,

    /// List all available rules
    #[arg(long, global = true)]
    list_rules: bool,

    /// List all available presets
    #[arg(long, global = true)]
    list_presets: bool,

    /// Read input from stdin (use '-' as filename)
    #[arg(long, global = true)]
    stdin: bool,

    /// Enable specific rules (can be repeated, e.g., --enable MD001 --enable MD003)
    #[arg(long, action = clap::ArgAction::Append, global = true)]
    enable: Vec<String>,

    /// Disable specific rules (can be repeated, e.g., --disable MD013 --disable MD033)
    #[arg(long, action = clap::ArgAction::Append, global = true)]
    disable: Vec<String>,

    /// Verbose output with detailed information
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Quiet mode - only show file names with errors
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Apply a named rule preset (e.g., "kramdown")
    #[arg(long, global = true)]
    preset: Option<String>,

    /// Watch mode - re-lint files on changes
    #[arg(short, long, global = true)]
    watch: bool,

    /// Watch specific paths (default: all input files/directories)
    #[arg(long, action = clap::ArgAction::Append, global = true)]
    watch_paths: Vec<String>,

    /// Print the JSON Schema for the configuration file to stdout
    #[arg(long, global = true)]
    generate_schema: bool,
}

#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
enum Command {
    /// Initialize a new configuration file
    Init {
        /// Output file path (default: .markdownlint.json)
        #[arg(long, default_value = ".markdownlint.json")]
        output: String,

        /// Output format (json, yaml, or toml)
        #[arg(long, default_value = "json")]
        format: String,

        /// Interactive mode with guided questions
        #[arg(long, short)]
        interactive: bool,
    },
}

/// Expand directories to .md/.markdown files recursively
#[cfg(feature = "cli")]
fn expand_paths(paths: &[String]) -> Vec<String> {
    use walkdir::WalkDir;

    let mut expanded = Vec::new();
    for path in paths {
        let p = std::path::Path::new(path);
        if p.is_dir() {
            for entry in WalkDir::new(p).into_iter().filter_map(|e| e.ok()) {
                let ep = entry.path();
                if ep.is_file()
                    && let Some(ext) = ep.extension().and_then(|e| e.to_str())
                    && (ext == "md" || ext == "markdown")
                {
                    expanded.push(ep.to_string_lossy().to_string());
                }
            }
        } else {
            expanded.push(path.clone());
        }
    }
    expanded.sort();
    expanded
}

/// Filter files by ignore glob patterns
#[cfg(feature = "cli")]
fn filter_ignored(
    files: Vec<String>,
    ignore_patterns: &[String],
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if ignore_patterns.is_empty() {
        return Ok(files);
    }

    use globset::{Glob, GlobSetBuilder};

    let mut builder = GlobSetBuilder::new();
    for pattern in ignore_patterns {
        builder.add(Glob::new(pattern)?);
    }
    let ignore_set = builder.build()?;

    Ok(files
        .into_iter()
        .filter(|f| !ignore_set.is_match(f))
        .collect())
}

/// Interactive configuration wizard
#[cfg(feature = "cli")]
fn init_config_interactive(
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

/// Config options collected from wizard
#[cfg(feature = "cli")]
struct ConfigOptions<'a> {
    line_length: usize,
    heading_style: &'a str,
    list_marker: &'a str,
    emphasis_style: &'a str,
    strong_style: &'a str,
    allow_html: bool,
    allowed_elements: Vec<&'a str>,
    code_style: &'a str,
    disabled_rules: Vec<&'a str>,
}

/// Generate configuration content based on wizard answers
#[cfg(feature = "cli")]
fn generate_config(format: &str, options: &ConfigOptions) -> String {
    match format {
        "json" => generate_json_config(options),
        "yaml" => generate_yaml_config(options),
        "toml" => generate_toml_config(options),
        _ => String::new(),
    }
}

#[cfg(feature = "cli")]
fn generate_json_config(options: &ConfigOptions) -> String {
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

#[cfg(feature = "cli")]
fn generate_yaml_config(options: &ConfigOptions) -> String {
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

#[cfg(feature = "cli")]
fn generate_toml_config(options: &ConfigOptions) -> String {
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

/// Initialize a new configuration file
#[cfg(feature = "cli")]
fn init_config(
    output_path: &str,
    format: &str,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::Colorize;
    use std::path::Path;

    // Check if file already exists
    if Path::new(output_path).exists() {
        eprintln!(
            "{} Configuration file '{}' already exists.",
            "Error:".red().bold(),
            output_path
        );
        eprintln!("Remove it first or choose a different output path with --output");
        std::process::exit(1);
    }

    // Interactive mode: ask questions and generate customized config
    if interactive {
        return init_config_interactive(output_path, format);
    }

    // Create a useful default configuration with examples
    let content = match format {
        "json" => r#"{
  "default": true,
  "MD013": {
    "line_length": 120,
    "code_blocks": false,
    "tables": false
  },
  "MD033": {
    "allowed_elements": ["br", "img", "details", "summary"]
  },
  "MD040": {
    "default_language": "text"
  }
}"#
        .to_string(),
        "yaml" | "yml" => r#"# Markdownlint configuration
default: true

# Line length
MD013:
  line_length: 120
  code_blocks: false
  tables: false

# Inline HTML
MD033:
  allowed_elements:
    - br
    - img
    - details
    - summary

# Fenced code language
MD040:
  default_language: text
"#
        .to_string(),
        "toml" => r#"# Markdownlint configuration
default = true

[MD013]
line_length = 120
code_blocks = false
tables = false

[MD033]
allowed_elements = ["br", "img", "details", "summary"]

[MD040]
default_language = "text"
"#
        .to_string(),
        _ => {
            eprintln!(
                "{} Unsupported format '{}'. Use json, yaml, or toml.",
                "Error:".red().bold(),
                format
            );
            std::process::exit(1);
        }
    };

    // Write to file
    std::fs::write(output_path, content)?;

    println!(
        "{} Created configuration file: {}",
        "✓".green().bold(),
        output_path.cyan()
    );
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to customize rules", output_path.cyan());
    println!(
        "  2. Run: {} {} {}",
        "mkdlint".cyan(),
        "--config".yellow(),
        output_path.cyan()
    );

    Ok(())
}

/// List all available linting rules, optionally filtered/annotated by a preset
#[cfg(feature = "cli")]
fn list_rules(preset: &Option<String>) {
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
#[cfg(feature = "cli")]
fn list_presets() {
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

/// Generate a JSON Schema for the mkdlint configuration file.
///
/// The schema describes all top-level config keys (`default`, `extends`,
/// `preset`) as well as every rule ID as a known property with a description.
#[cfg(feature = "cli")]
fn generate_config_schema() -> String {
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

/// Run watch mode with file change detection
#[cfg(feature = "cli")]
fn run_watch_mode(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    use colored::Colorize;
    use notify_debouncer_full::{new_debouncer, notify::*};
    use std::path::Path;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    println!("{}", "Starting watch mode...".cyan().bold());
    println!();

    // Determine paths to watch
    let watch_paths = if args.watch_paths.is_empty() {
        &args.files
    } else {
        &args.watch_paths
    };

    // Initial lint
    println!("{} Initial lint...", "▸".cyan());
    if let Err(e) = lint_files_once(args) {
        eprintln!("{} {}", "Error:".red().bold(), e);
    }
    println!();

    // Set up file watcher with debouncing (300ms)
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(300), None, tx)?;

    // Watch all specified paths
    for path in watch_paths {
        let path_obj = Path::new(path);
        if path_obj.exists() {
            debouncer
                .watcher()
                .watch(path_obj, RecursiveMode::Recursive)?;
            println!("{} Watching: {}", "✓".green(), path.cyan());
        } else {
            eprintln!(
                "{} Path does not exist: {}",
                "Warning:".yellow().bold(),
                path
            );
        }
    }

    println!();
    println!("{} Press {} to exit", "▸".cyan(), "Ctrl+C".yellow().bold());
    println!();

    // Main watch loop
    loop {
        match rx.recv() {
            Ok(result) => match result {
                Ok(events) => {
                    // Filter for markdown file changes
                    let has_markdown_changes = events.iter().any(|event| {
                        event.paths.iter().any(|path| {
                            path.extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext == "md" || ext == "markdown")
                                .unwrap_or(false)
                        })
                    });

                    if has_markdown_changes {
                        println!("{} File changed, re-linting...", "▸".cyan());
                        if let Err(e) = lint_files_once(args) {
                            eprintln!("{} {}", "Error:".red().bold(), e);
                        }
                        println!();
                    }
                }
                Err(errors) => {
                    for error in errors {
                        eprintln!("{} Watch error: {:?}", "Error:".red().bold(), error);
                    }
                }
            },
            Err(e) => {
                eprintln!("{} Channel error: {}", "Error:".red().bold(), e);
                break;
            }
        }
    }

    Ok(())
}

/// Lint files once (used by watch mode and normal mode)
#[cfg(feature = "cli")]
fn lint_files_once(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    use colored::Colorize;

    // Expand directories and filter ignored files
    let files = expand_paths(&args.files);
    let files = filter_ignored(files, &args.ignore)?;

    if files.is_empty() {
        if !args.quiet {
            println!("No files to lint.");
        }
        return Ok(());
    }

    // Build configuration
    let mut config = if let Some(ref config_path) = args.config {
        mkdlint::Config::from_file(config_path)?
    } else {
        mkdlint::Config::default()
    };

    // Apply --enable and --disable flags
    use mkdlint::RuleConfig;
    for rule in &args.enable {
        config
            .rules
            .insert(rule.to_uppercase(), RuleConfig::Enabled(true));
    }
    for rule in &args.disable {
        config
            .rules
            .insert(rule.to_uppercase(), RuleConfig::Enabled(false));
    }

    // Apply --preset flag
    if let Some(ref preset_name) = args.preset {
        config.preset = Some(preset_name.clone());
    }
    config.apply_preset();

    let options = LintOptions {
        files: files.clone(),
        strings: std::collections::HashMap::new(),
        config: Some(config),
        no_inline_config: args.no_inline_config,
        ..Default::default()
    };

    let results = lint_sync(&options)?;

    // Handle --fix-dry-run: show what would change without writing
    if args.fix_dry_run {
        let mut would_fix_count = 0;
        for file_path in &files {
            let errors = match results.get(file_path) {
                Some(errors) if !errors.is_empty() => errors,
                _ => continue,
            };

            let has_fixes = errors.iter().any(|e| e.fix_info.is_some());
            if !has_fixes {
                continue;
            }

            let content = std::fs::read_to_string(file_path)?;
            let fixed = apply_fixes(&content, errors);
            if fixed != content {
                would_fix_count += 1;
                if !args.quiet {
                    println!("{} {}", "Would fix:".yellow().bold(), file_path);
                    // Show per-error breakdown (skip fix-only helpers)
                    for error in errors
                        .iter()
                        .filter(|e| e.fix_info.is_some() && !e.fix_only)
                    {
                        let rule = error.rule_names.first().copied().unwrap_or("?");
                        println!(
                            "  line {}: {} {}",
                            error.line_number,
                            rule.yellow(),
                            error.rule_description
                        );
                    }
                }
            }
        }

        if !args.quiet {
            if would_fix_count > 0 {
                println!(
                    "\n{} {} file(s) would be fixed (run with {} to apply).",
                    "»".yellow().bold(),
                    would_fix_count.to_string().yellow(),
                    "--fix".bold()
                );
            } else {
                println!("{}", "No fixable issues found.".dimmed());
            }
        }
        // Exit 1 if there are fixable issues (useful for CI), 0 if clean
        if would_fix_count > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }

    // Handle auto-fix
    if args.fix {
        let mut fixed_count = 0;
        for file_path in &files {
            let errors = match results.get(file_path) {
                Some(errors) if !errors.is_empty() => errors,
                _ => continue,
            };

            let has_fixes = errors.iter().any(|e| e.fix_info.is_some());
            if !has_fixes {
                continue;
            }

            let content = std::fs::read_to_string(file_path)?;
            let fixed = apply_fixes(&content, errors);
            if fixed != content {
                std::fs::write(file_path, &fixed)?;
                fixed_count += 1;
                if args.verbose || !args.quiet {
                    println!("{} {}", "Fixed:".green().bold(), file_path);
                }
            }
        }

        if !args.quiet {
            if fixed_count > 0 {
                println!(
                    "{} {} file(s) fixed.",
                    "✓".green().bold(),
                    fixed_count.to_string().green()
                );
            } else {
                println!("{}", "No fixable issues found.".dimmed());
            }
        }
    } else if results.is_empty() {
        if !args.quiet {
            println!("{} No errors found!", "✓".green().bold());
        }
    } else {
        // Display errors
        if args.quiet {
            for (file, errors) in &results.results {
                if !errors.is_empty() {
                    println!("{}", file);
                }
            }
        } else {
            let output = match args.output_format {
                OutputFormat::Text => {
                    let mut sources = std::collections::HashMap::new();
                    for file in &files {
                        if let Ok(content) = std::fs::read_to_string(file) {
                            sources.insert(file.clone(), content);
                        }
                    }
                    formatters::format_text_with_context(&results, &sources)
                }
                OutputFormat::Json => formatters::format_json(&results),
                OutputFormat::Sarif => formatters::format_sarif(&results),
            };
            print!("{}", output);
        }

        // In watch mode, don't return error - just continue watching
        if args.watch {
            return Ok(());
        }
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.no_color {
        colored::control::set_override(false);
    }

    // Handle init subcommand
    if let Some(Command::Init {
        output,
        format,
        interactive,
    }) = args.command
    {
        return init_config(&output, &format, interactive);
    }

    // Handle --generate-schema flag
    if args.generate_schema {
        print!("{}", generate_config_schema());
        return Ok(());
    }

    // Handle --list-presets flag
    if args.list_presets {
        list_presets();
        return Ok(());
    }

    // Handle --list-rules flag
    if args.list_rules {
        list_rules(&args.preset);
        return Ok(());
    }

    // Validate files are provided
    if args.files.is_empty() && !args.stdin {
        eprintln!("error: FILES argument required (or use --stdin)");
        std::process::exit(1);
    }

    // Watch mode requires files, not stdin
    if args.watch && args.stdin {
        eprintln!("error: --watch cannot be used with --stdin");
        std::process::exit(1);
    }

    // If watch mode, delegate to watch function
    if args.watch {
        return run_watch_mode(&args);
    }

    // Handle stdin input
    let (files, stdin_content) = if args.stdin {
        (
            vec!["-".to_string()],
            Some(std::io::read_to_string(std::io::stdin())?),
        )
    } else {
        // Expand directories and filter ignored files
        let files = expand_paths(&args.files);
        let files = filter_ignored(files, &args.ignore)?;

        if files.is_empty() {
            if !args.quiet {
                println!("No files to lint.");
            }
            return Ok(());
        }
        (files, None)
    };

    // Build configuration with enable/disable rules
    let mut config = if let Some(ref config_path) = args.config {
        mkdlint::Config::from_file(config_path)?
    } else {
        mkdlint::Config::default()
    };

    // Apply --enable and --disable flags
    use mkdlint::RuleConfig;
    for rule in &args.enable {
        config
            .rules
            .insert(rule.to_uppercase(), RuleConfig::Enabled(true));
    }
    for rule in &args.disable {
        config
            .rules
            .insert(rule.to_uppercase(), RuleConfig::Enabled(false));
    }

    // Apply --preset flag (overrides config-file preset if both are set)
    if let Some(ref preset_name) = args.preset {
        config.preset = Some(preset_name.clone());
    }
    // apply_preset is called inside resolve_extends() via load_config(),
    // but since we bypass load_config here, call it explicitly.
    config.apply_preset();

    let mut strings = std::collections::HashMap::new();
    if let Some(content) = stdin_content {
        strings.insert("-".to_string(), content);
    }

    let options = LintOptions {
        files: if args.stdin { vec![] } else { files.clone() },
        strings,
        config: Some(config),
        no_inline_config: args.no_inline_config,
        ..Default::default()
    };

    let results = lint_sync(&options)?;

    // Handle --fix-dry-run: show what would change without writing
    if args.fix_dry_run {
        use colored::Colorize;
        let mut would_fix_count = 0;
        let file_list: Vec<String> = if args.stdin {
            vec!["-".to_string()]
        } else {
            files.clone()
        };
        for file_path in &file_list {
            let errors = match results.get(file_path) {
                Some(errors) if !errors.is_empty() => errors,
                _ => continue,
            };
            let has_fixes = errors.iter().any(|e| e.fix_info.is_some());
            if !has_fixes {
                continue;
            }
            let content = if file_path == "-" {
                options.strings.get("-").unwrap().clone()
            } else {
                std::fs::read_to_string(file_path)?
            };
            let fixed = apply_fixes(&content, errors);
            if fixed != content {
                would_fix_count += 1;
                if !args.quiet {
                    println!("{} {}", "Would fix:".yellow().bold(), file_path);
                    for error in errors
                        .iter()
                        .filter(|e| e.fix_info.is_some() && !e.fix_only)
                    {
                        let rule = error.rule_names.first().copied().unwrap_or("?");
                        println!(
                            "  line {}: {} {}",
                            error.line_number,
                            rule.yellow(),
                            error.rule_description
                        );
                    }
                }
            }
        }
        if !args.quiet {
            if would_fix_count > 0 {
                println!(
                    "\n{} {} file(s) would be fixed (run with {} to apply).",
                    "»".yellow().bold(),
                    would_fix_count.to_string().yellow(),
                    "--fix".bold()
                );
            } else {
                println!("{}", "No fixable issues found.".dimmed());
            }
        }
        if would_fix_count > 0 {
            std::process::exit(1);
        }
        return Ok(());
    }

    if args.fix {
        let mut fixed_count = 0;
        let file_list = if args.stdin {
            vec!["-".to_string()]
        } else {
            files.clone()
        };

        for file_path in &file_list {
            let errors = match results.get(file_path) {
                Some(errors) if !errors.is_empty() => errors,
                _ => continue,
            };

            let has_fixes = errors.iter().any(|e| e.fix_info.is_some());
            if !has_fixes {
                continue;
            }

            let content = if file_path == "-" {
                options.strings.get("-").unwrap().clone()
            } else {
                std::fs::read_to_string(file_path)?
            };

            let fixed = apply_fixes(&content, errors);
            if fixed != content {
                if file_path == "-" {
                    // Output to stdout
                    print!("{}", fixed);
                } else {
                    std::fs::write(file_path, &fixed)?;
                    fixed_count += 1;
                    if args.verbose || !args.quiet {
                        println!("Fixed: {}", file_path);
                    }
                }
            }
        }

        if !args.quiet && !args.stdin {
            if fixed_count > 0 {
                println!("{} file(s) fixed.", fixed_count);
            } else {
                println!("No fixable issues found.");
            }
        }
    } else if results.is_empty() {
        if !args.quiet {
            println!("No errors found!");
        }
    } else {
        // Handle different output modes
        if args.quiet {
            // Quiet mode: just list files with errors
            for (file, errors) in &results.results {
                if !errors.is_empty() {
                    println!("{}", file);
                }
            }
        } else {
            let output = match args.output_format {
                OutputFormat::Text => {
                    // Read source files for context display
                    let mut sources = std::collections::HashMap::new();
                    if args.stdin {
                        if let Some(content) = options.strings.get("-") {
                            sources.insert("-".to_string(), content.clone());
                        }
                    } else {
                        for file_path in &files {
                            if let Ok(content) = std::fs::read_to_string(file_path) {
                                sources.insert(file_path.clone(), content);
                            }
                        }
                    }

                    let formatted = formatters::format_text_with_context(&results, &sources);

                    // Add summary if verbose
                    if args.verbose {
                        let total_errors: usize = results.results.values().map(|e| e.len()).sum();
                        let total_files = results.results.len();
                        format!(
                            "{}\n\nSummary: {} error(s) in {} file(s)",
                            formatted, total_errors, total_files
                        )
                    } else {
                        formatted
                    }
                }
                OutputFormat::Json => formatters::format_json(&results),
                OutputFormat::Sarif => formatters::format_sarif(&results),
            };
            println!("{}", output);
        }
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature not enabled. Rebuild with --features cli");
    std::process::exit(1);
}
