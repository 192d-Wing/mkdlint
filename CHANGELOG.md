# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Auto-fix support** for 12 additional rules:
  - MD011: Reversed link syntax - automatically swaps text and URL
  - MD023: Indented headings - removes leading whitespace
  - MD026: Trailing punctuation in headings - removes punctuation
  - MD034: Bare URLs - wraps URLs in angle brackets
  - MD035: Horizontal rule style - converts to consistent style
  - MD037: Spaces inside emphasis markers - trims spaces
  - MD038: Spaces inside code spans - trims spaces
  - MD039: Spaces inside link text - trims spaces
  - MD044: Proper names capitalization - fixes per-occurrence
  - MD048: Code fence style - converts ``` to ~~~ or vice versa
  - MD049: Emphasis style consistency - converts to preferred style
  - MD050: Strong style consistency - converts to preferred style

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

- **MD035 and MD048 auto-fix**:
  - MD035 converts horizontal rules to consistent style (e.g., all to `---`)
  - MD048 converts code fence markers to consistent style (all to ``` or ~~~)

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

[Unreleased]: https://github.com/192d-Wing/mdlint/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/192d-Wing/mdlint/releases/tag/v0.1.0
