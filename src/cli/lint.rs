//! Core linting logic — lint files once (used by watch mode and normal mode)

use super::args::{Args, OutputFormat};
use super::files::{expand_paths, filter_ignored};
use mkdlint::{LintOptions, apply_fixes, formatters, lint_sync};

/// Lint files once (used by watch mode and normal mode)
pub(crate) fn lint_files_once(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
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

    // Pre-build workspace heading index once for convergence passes (fix/dry-run)
    let cached_headings = if files.len() > 1 && (args.fix || args.fix_dry_run) {
        let inputs: Vec<(String, String)> = files
            .iter()
            .filter_map(|f| std::fs::read_to_string(f).ok().map(|c| (f.clone(), c)))
            .collect();
        Some(mkdlint::build_workspace_headings(&inputs))
    } else {
        None
    };

    // Handle --fix-dry-run: show what would change without writing
    if args.fix_dry_run {
        let mut would_fix_count = 0;
        for file_path in &files {
            let content = std::fs::read_to_string(file_path)?;
            let mut current = content.clone();

            // Multi-pass fix convergence for dry-run preview
            for _pass in 0..10 {
                // DEFAULT_FIX_PASSES = 10
                let pass_options = LintOptions {
                    files: vec![],
                    strings: [(file_path.clone(), current.clone())].into(),
                    config: options.config.clone(),
                    no_inline_config: args.no_inline_config,
                    cached_workspace_headings: cached_headings.clone(),
                    ..Default::default()
                };

                let pass_results = lint_sync(&pass_options)?;
                let pass_errors = pass_results.get(file_path).unwrap_or(&[]);

                let next = apply_fixes(&current, pass_errors);
                if next == current {
                    break; // Converged
                }
                current = next;
            }

            if current != content {
                would_fix_count += 1;
                if !args.quiet {
                    println!("{} {}", "Would fix:".yellow().bold(), file_path);
                    // Re-lint final result to show what errors would be fixed
                    let original_errors = results.get(file_path).unwrap_or(&[]);

                    // Show errors that had fixes
                    for error in original_errors
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
            let content = std::fs::read_to_string(file_path)?;
            let mut current = content.clone();

            // Multi-pass fix convergence: re-lint and re-fix until stable
            for _pass in 0..10 {
                // DEFAULT_FIX_PASSES = 10
                let pass_options = LintOptions {
                    files: vec![],
                    strings: [(file_path.clone(), current.clone())].into(),
                    config: options.config.clone(),
                    no_inline_config: args.no_inline_config,
                    cached_workspace_headings: cached_headings.clone(),
                    ..Default::default()
                };

                let pass_results = lint_sync(&pass_options)?;
                let pass_errors = pass_results.get(file_path).unwrap_or(&[]);

                let next = apply_fixes(&current, pass_errors);
                if next == current {
                    break; // Converged
                }
                current = next;
            }

            if current != content {
                std::fs::write(file_path, &current)?;
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
                OutputFormat::Github => formatters::format_github(&results),
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
