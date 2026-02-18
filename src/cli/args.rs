//! CLI argument definitions for mkdlint

use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub(crate) enum OutputFormat {
    #[default]
    Text,
    Json,
    Sarif,
    /// GitHub Actions workflow command annotations (::error file=...)
    Github,
}

#[derive(Parser, Debug)]
#[command(name = "mkdlint")]
#[command(about = "A linter for Markdown files", long_about = None)]
#[command(version)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,

    /// Files or directories to lint
    #[arg(global = true)]
    pub(crate) files: Vec<String>,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    pub(crate) config: Option<String>,

    /// Output format
    #[arg(short = 'o', long, default_value = "text", global = true)]
    pub(crate) output_format: OutputFormat,

    /// Glob patterns for files to ignore (repeatable)
    #[arg(long, action = clap::ArgAction::Append, global = true)]
    pub(crate) ignore: Vec<String>,

    /// Disable colored output
    #[arg(long, global = true)]
    pub(crate) no_color: bool,

    /// Disable inline configuration comments
    #[arg(long, global = true)]
    pub(crate) no_inline_config: bool,

    /// Automatically fix violations where possible
    #[arg(short, long, global = true)]
    pub(crate) fix: bool,

    /// Show what --fix would change without writing any files
    #[arg(long, global = true)]
    pub(crate) fix_dry_run: bool,

    /// List all available rules
    #[arg(long, global = true)]
    pub(crate) list_rules: bool,

    /// List all available presets
    #[arg(long, global = true)]
    pub(crate) list_presets: bool,

    /// Read input from stdin (use '-' as filename)
    #[arg(long, global = true)]
    pub(crate) stdin: bool,

    /// Enable specific rules (can be repeated, e.g., --enable MD001 --enable MD003)
    #[arg(long, action = clap::ArgAction::Append, global = true)]
    pub(crate) enable: Vec<String>,

    /// Disable specific rules (can be repeated, e.g., --disable MD013 --disable MD033)
    #[arg(long, action = clap::ArgAction::Append, global = true)]
    pub(crate) disable: Vec<String>,

    /// Verbose output with detailed information
    #[arg(short, long, global = true)]
    pub(crate) verbose: bool,

    /// Quiet mode - only show file names with errors
    #[arg(short, long, global = true)]
    pub(crate) quiet: bool,

    /// Apply a named rule preset (e.g., "kramdown")
    #[arg(long, global = true)]
    pub(crate) preset: Option<String>,

    /// Watch mode - re-lint files on changes
    #[arg(short, long, global = true)]
    pub(crate) watch: bool,

    /// Watch specific paths (default: all input files/directories)
    #[arg(long, action = clap::ArgAction::Append, global = true)]
    pub(crate) watch_paths: Vec<String>,

    /// Print the JSON Schema for the configuration file to stdout
    #[arg(long, global = true)]
    pub(crate) generate_schema: bool,

    /// Filename to use for stdin content in error output (requires --stdin)
    #[arg(long, global = true)]
    pub(crate) stdin_filename: Option<String>,
}

#[derive(Parser, Debug)]
pub(crate) enum Command {
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
