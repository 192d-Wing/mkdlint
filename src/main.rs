//! Command-line interface for mdlint

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use mdlint::{apply_fixes, formatters, lint_sync, LintOptions};

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
#[command(name = "mdlint")]
#[command(about = "A linter for Markdown files", long_about = None)]
#[command(version)]
struct Args {
    /// Files to lint
    #[arg(required = true)]
    files: Vec<String>,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Output format
    #[arg(short = 'o', long, default_value = "text")]
    output_format: OutputFormat,

    /// Disable inline configuration comments
    #[arg(long)]
    no_inline_config: bool,

    /// Automatically fix violations where possible
    #[arg(short, long)]
    fix: bool,
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let options = LintOptions {
        files: args.files.clone(),
        config_file: args.config,
        no_inline_config: args.no_inline_config,
        ..Default::default()
    };

    let results = lint_sync(&options)?;

    if args.fix {
        let mut fixed_count = 0;
        for file_path in &args.files {
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
                println!("Fixed: {}", file_path);
            }
        }

        if fixed_count > 0 {
            println!("{} file(s) fixed.", fixed_count);
        } else {
            println!("No fixable issues found.");
        }
    } else if results.is_empty() {
        println!("No errors found!");
    } else {
        let output = match args.output_format {
            OutputFormat::Text => formatters::format_text(&results),
            OutputFormat::Json => formatters::format_json(&results),
            OutputFormat::Sarif => formatters::format_sarif(&results),
        };
        println!("{}", output);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature not enabled. Rebuild with --features cli");
    std::process::exit(1);
}
