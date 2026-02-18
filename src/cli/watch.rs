//! `--watch` mode — re-lint files on filesystem changes

use super::args::Args;
use super::lint::lint_files_once;

/// Run watch mode with file change detection
pub(crate) fn run_watch_mode(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
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
