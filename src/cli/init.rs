//! `mkdlint init` subcommand — initialize a new configuration file

use super::wizard::init_config_interactive;

/// Initialize a new configuration file
pub(crate) fn init_config(
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
