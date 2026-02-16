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
    /// Files or directories to lint
    #[arg(required = true)]
    files: Vec<String>,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Output format
    #[arg(short = 'o', long, default_value = "text")]
    output_format: OutputFormat,

    /// Glob patterns for files to ignore (repeatable)
    #[arg(long, action = clap::ArgAction::Append)]
    ignore: Vec<String>,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Disable inline configuration comments
    #[arg(long)]
    no_inline_config: bool,

    /// Automatically fix violations where possible
    #[arg(short, long)]
    fix: bool,
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
                if ep.is_file() {
                    if let Some(ext) = ep.extension().and_then(|e| e.to_str()) {
                        if ext == "md" || ext == "markdown" {
                            expanded.push(ep.to_string_lossy().to_string());
                        }
                    }
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
fn filter_ignored(files: Vec<String>, ignore_patterns: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if ignore_patterns.is_empty() {
        return Ok(files);
    }

    use globset::{Glob, GlobSetBuilder};

    let mut builder = GlobSetBuilder::new();
    for pattern in ignore_patterns {
        builder.add(Glob::new(pattern)?);
    }
    let ignore_set = builder.build()?;

    Ok(files.into_iter().filter(|f| !ignore_set.is_match(f)).collect())
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.no_color {
        colored::control::set_override(false);
    }

    // Expand directories and filter ignored files
    let files = expand_paths(&args.files);
    let files = filter_ignored(files, &args.ignore)?;

    if files.is_empty() {
        println!("No files to lint.");
        return Ok(());
    }

    let options = LintOptions {
        files: files.clone(),
        config_file: args.config,
        no_inline_config: args.no_inline_config,
        ..Default::default()
    };

    let results = lint_sync(&options)?;

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
