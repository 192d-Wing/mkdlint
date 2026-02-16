# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Documentation

- **Comprehensive IDE Integration Guide** 📝
  - Expanded USER_GUIDE.md IDE section from ~60 to ~550 lines
  - Copy-paste ready configurations for 6 major editors:
    - VS Code (tasks, auto-run, watch mode, keybindings)
    - Neovim/Vim (ALE, null-ls, autocmd, custom commands)
    - Emacs (Flycheck, auto-fix, compilation mode)
    - Zed (tasks configuration)
    - Sublime Text (build system)
    - Generic LSP (future-ready)
  - Each editor has 3-4 integration methods for different workflows
  - Extensive troubleshooting section with 8+ common issues
  - Recommended workflows for beginners, power users, and teams
  - **Completes v0.7.0 roadmap milestone** ✅

## [0.7.1] - 2026-02-16

### Added

- **Watch Mode** 👀
  - `mkdlint --watch` - Auto-lint files on changes with real-time feedback
  - Debounced file system notifications (300ms) to avoid excessive re-linting
  - Automatically filters for .md and .markdown files
  - Works with auto-fix: `mkdlint --watch --fix` for automatic fixing on save
  - `--watch-paths` flag to watch specific directories/files
  - Colorful output with status indicators (▸, ✓)
  - Uses `notify` v6.1 and `notify-debouncer-full` v0.3 crates
  - Perfect companion to the interactive wizard for smooth development workflow

### Changed

- Refactored main linting logic into `lint_files_once()` helper function
  - Shared between normal mode and watch mode
  - Cleaner code organization

## [0.7.0] - 2026-02-16

### Added

- **Interactive Configuration Wizard** 🧙
  - `mkdlint init --interactive` - Guided configuration setup with 9 comprehensive questions
  - Questions about:
    - Format preference (JSON/YAML/TOML)
    - Line length limits (with 0 to disable option)
    - Heading style preferences (ATX/Setext/Consistent)
    - List marker style (dash/asterisk/plus/consistent)
    - Emphasis style (asterisk/underscore/consistent)
    - Strong emphasis style (asterisk/underscore/consistent)
    - Inline HTML with multi-select element picker
    - Code block style (fenced/indented/consistent)
    - Common rules to disable (MD013, MD033, MD034, MD041)
  - Generates optimal config based on answers
  - Auto-adjusts output filename extension to match format
  - Uses `dialoguer` crate for interactive terminal prompts
  - Significantly improves first-time user experience

### Changed

- Refactored config generation to use `ConfigOptions` struct for cleaner code

### Technical

- Added `dialoguer` dependency (v0.11) for interactive prompts
- All 346 tests passing, zero clippy warnings

## [0.6.1] - 2026-02-16

### Added

- **Comprehensive Documentation** 📚
  - **User Guide**: Complete guide covering getting started, configuration, auto-fix, IDE integration, CI/CD, troubleshooting, and FAQ
  - **Auto-Fix Showcase**: Before/after examples in README showing 80% coverage
  - **Helpful Suggestions**: All 44 rules now include actionable suggestions
    - Improves user experience and discoverability
    - Provides clear guidance on how to fix issues
    - Displayed with 💡 icon in colored output

### Changed

- Updated README to reflect 80% auto-fix coverage (was showing 63%)
- Updated library version examples to 0.6
- Improved feature discovery and documentation

## [0.6.0] - 2026-02-16

### Added

- **Near 80% Auto-Fix Coverage!** 🎉
  - **MD060 Auto-Fix** - Remove dollar signs in code blocks:
    - Automatically removes `$` or `$ ` prefix from commands in fenced code blocks
    - Handles indented commands correctly
    - Useful for cleaning up copy-pasted terminal examples
  - **MD053 Auto-Fix** - Remove unused link definitions:
    - Automatically deletes unused link reference definitions
    - Keeps markdown files clean and maintainable
    - Respects `ignored_definitions` config (defaults to `["//"]`)
  - **MD055 Auto-Fix** - Fix table pipe style:
    - Automatically adds missing leading or trailing pipes to table rows
    - Normalizes tables to consistent pipe style (both ends or neither)
    - Handles indented tables correctly
  - **Auto-fix coverage: 43/54 rules (80%)** - up from 40/54 (74%)
  - Only 11 rules remaining without auto-fix (most are infeasible)

### Changed

- All three new fixable rules include helpful suggestions
- Improved table formatting consistency

## [0.5.3] - 2026-02-16

### Added

- **MD045 Auto-Fix** - Images without alt text:
  - Automatically adds placeholder "image" alt text to images missing alt text
  - Fixes accessibility issues with `![](image.png)` → `![image](image.png)`
  - Helpful suggestion to use descriptive alt text

- **MD025 Auto-Fix** - Multiple H1 headings:
  - Automatically converts additional H1s to H2s
  - Supports both ATX (`# Heading`) and Setext styles
  - Maintains document structure while fixing hierarchy issues
  - Suggestion to restructure document or convert to H2

- **Auto-fix coverage: 40/54 rules (74%)** - up from 38/54 (70%)

## [0.5.2] - 2026-02-16

### Changed

- **Auto-fix Coverage Boost** - Added "fixable" tag to 27 additional rules:
  - All rules that already had `fix_info` implementations now properly tagged as fixable
  - Includes: MD004, MD005, MD007, MD009, MD010, MD011, MD012, MD018, MD019, MD022, MD023, MD026, MD027, MD029, MD031, MD032, MD034, MD035, MD037, MD038, MD039, MD040, MD044, MD048, MD049, MD050, MD058
  - **Auto-fix coverage: 38/54 rules (70%)** - up from 35/54 (65%)
  - These rules already had working auto-fixes, just missing the documentation tag

### Note

This release makes existing auto-fixes discoverable by properly tagging them. No new fix implementations were added - these 27 rules have had working fixes all along.

## [0.5.1] - 2026-02-16

### Added

- **MD003 Auto-Fix** - Heading style conversion:
  - Automatic conversion between ATX (`# Heading`), ATX-closed (`# Heading #`), and Setext styles
  - Supports all style configurations: `atx`, `atx_closed`, `setext`, `setext_with_atx`, `setext_with_atx_closed`, `consistent`
  - Properly handles Setext underline deletion when converting to ATX/ATX-closed
  - Adds helpful suggestions for each heading style mismatch
  - Auto-fix coverage increased to 35/54 rules (65%)

### Changed

- MD003 now includes "fixable" tag
- Setext heading conversions generate helper fix for underline deletion

## [0.5.0] - 2026-02-16

### Added

- **Enhanced Error Messages with Actionable Suggestions**:
  - New `suggestion` field in `LintError` provides helpful hints for fixing issues
  - Suggestions displayed with 💡 icon in text formatter
  - "Fix available" indicator (🔧) for auto-fixable errors
  - Context-aware suggestions for common rules (MD018, MD041, MD042)
  - Improved user experience with clear, actionable error messages

### Changed

- **Text formatter improvements**:
  - Shows helpful suggestions inline with errors
  - Displays fix availability indicator
  - Better visual hierarchy with icons and colors
  - More informative error output

### Performance

- **Regex optimization analysis**:
  - Verified all 16 static regexes are properly cached with `once_cell::Lazy`
  - Config-dependent regexes (MD001, MD036) correctly avoid caching
  - No performance improvements needed - already optimal

## [0.4.2] - 2026-02-16

### Added

- **SHA256 Checksum Verification for GitHub Action**:
  - Automatic verification of downloaded binaries using SHA256 checksums
  - Release workflow generates `.sha256` files for all binary archives
  - Action script downloads and verifies checksums before extraction
  - Fail-safe: refuses to use binary if checksum doesn't match
  - Graceful degradation: warns but continues if checksum file unavailable
  - Uses platform-appropriate tools (`sha256sum` on Linux, `shasum` on macOS)

### Security

- **Binary Download Security**:
  - All pre-built binaries now verified with SHA256 checksums
  - Protects against tampered or corrupted downloads
  - Checksums generated during release build process
  - Published alongside each binary archive
  - Verified before extraction and usage

### Documentation

- Added comprehensive Security section to GitHub Action README
- Documented checksum verification process and security best practices
- Added test-checksum-verification job to test workflow

## [0.4.1] - 2026-02-16

### Added

- **LSP "Fix All" Command**:
  - New `mkdlint.fixAll` command to apply all auto-fixes in a document at once
  - Appears in code action menu with fix count (e.g., "Fix all mkdlint issues (5 fixes)")
  - Uses `CodeActionKind::SOURCE_FIX_ALL` for proper categorization
  - Leverages existing `apply_fixes()` function for consistency with CLI
  - All 34 auto-fixable rules supported

### Fixed

- Removed duplicate line in README CI/CD Integration section

### Documentation

- Added "Fix All" command to LSP feature list
- Added usage tips section: quick-fix vs fix all
- Documented `workspace/executeCommand` capability

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

[Unreleased]: https://github.com/192d-Wing/mkdlint/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/192d-Wing/mkdlint/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/192d-Wing/mkdlint/compare/v0.5.3...v0.6.0
[0.5.3]: https://github.com/192d-Wing/mkdlint/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/192d-Wing/mkdlint/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/192d-Wing/mkdlint/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/192d-Wing/mkdlint/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/192d-Wing/mkdlint/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/192d-Wing/mkdlint/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/192d-Wing/mkdlint/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/192d-Wing/mkdlint/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/192d-Wing/mkdlint/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/192d-Wing/mkdlint/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/192d-Wing/mkdlint/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/192d-Wing/mkdlint/releases/tag/v0.1.0
