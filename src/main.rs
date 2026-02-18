//! Command-line interface for mkdlint

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::run()
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("CLI feature not enabled. Rebuild with --features cli");
    std::process::exit(1);
}
