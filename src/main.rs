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

    /// List all available rules
    #[arg(long, global = true)]
    list_rules: bool,

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

/// Initialize a new configuration file
#[cfg(feature = "cli")]
fn init_config(output_path: &str, format: &str) -> Result<(), Box<dyn std::error::Error>> {
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

/// List all available linting rules
#[cfg(feature = "cli")]
fn list_rules() {
    use colored::Colorize;
    use mkdlint::rules::get_rules;

    println!("{}", "Available Linting Rules".bold().underline());
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
            (
                names[0].to_string(),
                alias.to_string(),
                description.to_string(),
                fixable.to_string(),
            )
        })
        .collect();

    // Sort by rule number (MD001, MD002, etc.)
    rules_info.sort_by(|(a, _, _, _), (b, _, _, _)| a.cmp(b));

    println!(
        "{:8} {:30} {:8} {}",
        "Rule".bold(),
        "Alias".bold(),
        "Fixable".bold(),
        "Description".bold()
    );
    println!("{}", "─".repeat(80));

    for (rule_id, alias, description, fixable) in &rules_info {
        let fixable_mark = if fixable == "✓" {
            fixable.green()
        } else {
            fixable.normal()
        };
        println!(
            "{:8} {:30} {:^8} {}",
            rule_id.cyan(),
            alias.yellow(),
            fixable_mark,
            description
        );
    }

    println!();
    println!(
        "Total: {} rules ({} fixable)",
        rules.len(),
        rules_info.iter().filter(|(_, _, _, f)| f == "✓").count()
    );
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.no_color {
        colored::control::set_override(false);
    }

    // Handle init subcommand
    if let Some(Command::Init { output, format }) = args.command {
        return init_config(&output, &format);
    }

    // Handle --list-rules flag
    if args.list_rules {
        list_rules();
        return Ok(());
    }

    // Validate files are provided
    if args.files.is_empty() && !args.stdin {
        eprintln!("error: FILES argument required (or use --stdin)");
        std::process::exit(1);
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
