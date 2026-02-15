//! Command-line interface for mdlint

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
use mdlint::{lint_sync, LintOptions};

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

    /// Disable inline configuration comments
    #[arg(long)]
    no_inline_config: bool,
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let options = LintOptions {
        files: args.files,
        config_file: args.config,
        no_inline_config: args.no_inline_config,
        ..Default::default()
    };

    let results = lint_sync(&options)?;

    if results.is_empty() {
        println!("No errors found!");
    } else {
        println!("{}", results);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature not enabled. Rebuild with --features cli");
    std::process::exit(1);
}
