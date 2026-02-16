# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-02-16

### Added

- **Language Server Protocol (LSP) Support**:
  - New `mkdlint-lsp` binary providing full LSP implementation
  - Real-time diagnostics as you type with 300ms debouncing
  - Quick-fix code actions for all auto-fixable rules
  - Configuration auto-discovery from workspace root
  - Thread-safe document management with DashMap
  - Support for VS Code, Neovim, and all LSP-compatible editors
  - Full document synchronization (didOpen, didChange, didSave, didClose)
  - Install with: `cargo install mkdlint --features lsp`

- **GitHub Action with SARIF Integration**:
  - Native GitHub Actions support at `.github/actions/mkdlint`
  - Pre-built binaries for 5 platforms (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64)
  - Automatic binary caching for 10-100x faster subsequent runs (1-2s vs 5-10s vs 60-90s cargo build)
  - SARIF output with automatic Code Scanning upload
  - Auto-fix support with commit integration capabilities
  - Cargo build fallback if binary download fails
  - 20 configuration inputs for complete control
  - 6 outputs for workflow integration
  - Comprehensive test workflow with 18 test jobs
  - Usage: `uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main`

- **Auto-fix support** for 7 additional rules (34 total fixable, 63% coverage):
  - MD014: Dollar signs before commands - removes `$` or `$` prefix from code blocks
  - MD020: No space inside closed ATX headings - inserts missing spaces after/before `#`
  - MD021: Multiple spaces inside closed ATX headings - removes extra spaces
  - MD028: Blank line inside blockquote - deletes blank lines within blockquotes
  - MD030: List marker spacing - adjusts spaces after list markers (already had fix_info, added "fixable" tag)
  - MD036: Emphasis as heading - converts emphasis/strong to `## heading`
  - MD042: Empty links - replaces empty URLs with `#link` placeholder (inline links only)

### Changed

- **Auto-fix coverage increased** from 27 to 34 rules (50% → 63%)
- **LSP dependencies** added under optional `lsp` feature flag
- **Documentation** updated with LSP setup, GitHub Action usage, and new auto-fix rules

### Technical Details

- New LSP modules: `backend`, `document`, `diagnostics`, `code_actions`, `utils`
- GitHub Action shell script with platform detection, version resolution, and SARIF parsing
- All 429 tests passing with zero clippy warnings
- 16 conventional commits for Phase 1-3 implementation

## [0.3.2] - 2026-02-16

### Fixed

- **CI/CD Pipeline**:
  - Fixed ARM64 binary stripping by using `aarch64-linux-gnu-strip` for cross-compiled binaries
  - Updated `cargo-deny` configuration to v2 format
  - Added MPL-2.0 and Unicode-3.0 to allowed licenses
  - Ignored unmaintained dependency advisories from transitive dependencies (bincode, yaml-rust via syntect)

## [0.3.1] - 2026-02-16

### Fixed

- **CI/CD Pipeline Improvements**:
  - Fixed ARM64 (aarch64) cross-compilation by configuring proper linker
  - Migrated from deprecated GitHub Actions to modern `softprops/action-gh-release@v2`
  - Fixed changelog extraction to use version-specific sections instead of Unreleased
  - Added `fail-fast: false` to allow partial releases when one platform fails
  - Added `permissions: contents: write` for GitHub release creation

## [0.3.0] - 2026-02-15

### Changed

- **Package renamed from `mdlint` to `mkdlint`** due to naming conflict with existing crate on crates.io
  - Repository moved to https://github.com/192d-Wing/mkdlint
  - Binary renamed from `mdlint` to `mkdlint`
  - All references updated in code, documentation, and CI/CD workflows

### Added

- **Auto-fix support** for 3 additional rules (27 total fixable):
  - MD001: Heading increment - adjusts heading levels automatically
  - MD041: First-line heading - inserts top-level heading at document start
  - MD047: Single trailing newline - adds missing newline at file end

## [0.2.0] - 2026-02-15

### Added

- **Enhanced CLI Features**:
  - `mkdlint init` subcommand to create configuration file templates (JSON/YAML/TOML)
  - `--stdin` flag for reading input from standard input (Unix pipeline support)
  - `--list-rules` flag to display all available rules with descriptions
  - `--enable/--disable` flags for per-invocation rule overrides
  - `--verbose` flag for detailed error statistics and summaries
  - `--quiet` flag for minimal output (only filenames with errors)
  - Optional FILES argument (not required with --stdin or --list-rules)
  - Better stdin/stdout integration for pipeline workflows

- **GitHub Actions CI/CD Pipeline**:
  - Multi-platform testing (Ubuntu, macOS, Windows) on every push and PR
  - Automated linting with rustfmt and clippy
  - Benchmark comparison for pull requests
  - Code coverage tracking with cargo-tarpaulin and Codecov integration
  - Security auditing with cargo-audit and cargo-deny (runs daily)
  - Automated binary releases for 5 platforms (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64)
  - Automatic crates.io publishing on version tags

- **Auto-fix support** for 14 additional rules:
  - MD011: Reversed link syntax - automatically swaps text and URL
  - MD023: Indented headings - removes leading whitespace
  - MD026: Trailing punctuation in headings - removes punctuation
  - MD034: Bare URLs - wraps URLs in angle brackets
  - MD035: Horizontal rule style - converts to consistent style
  - MD037: Spaces inside emphasis markers - trims spaces
  - MD038: Spaces inside code spans - trims spaces
  - MD039: Spaces inside link text - trims spaces
  - MD040: Fenced code language - adds configurable default language (default: "text")
  - MD044: Proper names capitalization - fixes per-occurrence
  - MD048: Code fence style - converts ``` to ~~~ or vice versa
  - MD049: Emphasis style consistency - converts to preferred style
  - MD050: Strong style consistency - converts to preferred style
  - MD058: Tables blank lines - inserts blank lines before/after tables

- **Rich error display** with source context:
  - Shows the source line containing the error
  - Displays colored caret underlines (^^^) pointing to the exact error location
  - Includes line number gutter for context
  - Respects `--no-color` flag for CI/automated environments

- **Enhanced text formatter** with `format_text_with_context()` function
  - Accepts source file content alongside lint results
  - Automatically displays source context when error_range is available

- **Comprehensive test suite**:
  - 20 new unit tests for MD022, MD024, MD025, MD046
  - 12 snapshot tests using `insta` crate for regression detection
  - 11 new E2E tests covering fixtures, output formats, and fix roundtrips
  - 6 markdown test fixtures for common error patterns
  - Total: 377 tests (309 unit + 19 E2E + 36 integration + 12 snapshot + 1 doc)

- **Performance benchmarks**:
  - Added config_load_json benchmark
  - All benchmarks run in CI for regression detection

### Changed

- **MD022 enhanced** to fix both before and after violations:
  - Previously only "before heading" was fixable
  - Now both "blank line before" and "blank line after" are auto-fixable

- **MD049 and MD050 completely rewritten**:
  - Now detect per-occurrence violations instead of aggregate counts
  - Provide accurate column ranges for each error
  - Include fix_info for automatic style conversion
  - Parse emphasis/strong markers at byte level for precision

- **MD044 improved** for per-occurrence detection:
  - Reports each incorrectly-cased occurrence separately
  - Shows actual vs expected casing in error_detail
  - Provides fix_info with exact column and replacement text

- **MD035, MD040, MD048, and MD058 auto-fix**:
  - MD035 converts horizontal rules to consistent style (e.g., all to `---`)
  - MD040 adds default language to fenced code blocks (configurable via `default_language`, defaults to "text")
  - MD048 converts code fence markers to consistent style (all to ``` or ~~~)
  - MD058 inserts blank lines before and after tables

### Performance

- **22% faster** on small files (lint_single_small: 130µs → 102µs)
- **19% faster** on multi-file workloads (lint_multi_20_files: 667µs → 540µs)
- **Optimizations**:
  - Pre-filter enabled rules before linting loop
  - Cache regex in MD007 using `once_cell::Lazy`
  - Parallel processing improvements

### Fixed

- MD049/MD050 false positives from simple character counting
- MD044 reporting entire line instead of specific occurrences

## [0.1.0] - 2024-01-XX

### Added

- Initial release with 54 markdown linting rules (MD001-MD060)
- Automatic fixing via `--fix` flag
- Multiple output formats: text, JSON, SARIF
- Configuration file support (JSON, YAML, TOML)
- Auto-discovery of configuration files
- Directory recursion with `--ignore` patterns
- Colored terminal output with `--no-color` flag
- Library and CLI interfaces
- Parallel file processing
- Inline configuration comments support

[Unreleased]: https://github.com/192d-Wing/mkdlint/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/192d-Wing/mkdlint/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/192d-Wing/mkdlint/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/192d-Wing/mkdlint/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/192d-Wing/mkdlint/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/192d-Wing/mkdlint/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/192d-Wing/mkdlint/releases/tag/v0.1.0
