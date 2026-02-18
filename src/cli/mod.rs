//! CLI entry point — module declarations and the `run()` dispatcher

mod args;
mod files;
mod init;
mod lint;
mod rules;
mod schema;
mod watch;
mod wizard;

use args::{Args, Command, OutputFormat};
use clap::Parser;
use files::{expand_paths, filter_ignored};
use mkdlint::{LintOptions, apply_fixes, formatters, lint_sync};

/// Main CLI entry point — parse args and dispatch to the appropriate handler
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
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
        return init::init_config(&output, &format, interactive);
    }

    // Handle --generate-schema flag
    if args.generate_schema {
        print!("{}", schema::generate_config_schema());
        return Ok(());
    }

    // Handle --list-presets flag
    if args.list_presets {
        rules::list_presets();
        return Ok(());
    }

    // Handle --list-rules flag
    if args.list_rules {
        rules::list_rules(&args.preset);
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
        return watch::run_watch_mode(&args);
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
        let stdin_key = args
            .stdin_filename
            .clone()
            .unwrap_or_else(|| "-".to_string());
        strings.insert(stdin_key, content);
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
            let content = if file_path == "-" {
                options
                    .strings
                    .get("-")
                    .expect("stdin content must be present when reading from '-'")
                    .clone()
            } else {
                std::fs::read_to_string(file_path)?
            };

            let mut current = content.clone();

            // Multi-pass fix convergence for dry-run preview
            for _pass in 0..10 {
                // DEFAULT_FIX_PASSES = 10
                let pass_options = LintOptions {
                    files: vec![],
                    strings: [(file_path.clone(), current.clone())].into(),
                    config: options.config.clone(),
                    no_inline_config: options.no_inline_config,
                    front_matter: options.front_matter.clone(),
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
                    // Show errors from original lint
                    let original_errors = results.get(file_path).unwrap_or(&[]);
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
                options
                    .strings
                    .get("-")
                    .expect("stdin content must be present when reading from '-'")
                    .clone()
            } else {
                std::fs::read_to_string(file_path)?
            };

            // Multi-pass fix convergence: re-lint and re-fix until stable
            let mut current = content.clone();
            for _pass in 0..10 {
                // DEFAULT_FIX_PASSES = 10
                // Re-lint the current content
                let pass_options = LintOptions {
                    files: vec![],
                    strings: [(file_path.clone(), current.clone())].into(),
                    config: options.config.clone(),
                    no_inline_config: options.no_inline_config,
                    front_matter: options.front_matter.clone(),
                    ..Default::default()
                };

                let pass_results = lint_sync(&pass_options)?;
                let pass_errors = pass_results.get(file_path).unwrap_or(&[]);

                // Apply fixes
                let next = apply_fixes(&current, pass_errors);
                if next == current {
                    break; // Converged
                }
                current = next;
            }

            if current != content {
                if file_path == "-" {
                    // Output to stdout
                    print!("{}", current);
                } else {
                    std::fs::write(file_path, &current)?;
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
                        let stdin_key = args
                            .stdin_filename
                            .clone()
                            .unwrap_or_else(|| "-".to_string());
                        if let Some(content) = options.strings.get(&stdin_key) {
                            sources.insert(stdin_key, content.clone());
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
                OutputFormat::Github => formatters::format_github(&results),
            };
            println!("{}", output);
        }
        std::process::exit(1);
    }

    Ok(())
}
