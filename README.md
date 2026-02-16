# mdlint

[![CI](https://github.com/192d-Wing/mdlint/workflows/CI/badge.svg)](https://github.com/192d-Wing/mdlint/actions/workflows/ci.yml)
[![Security Audit](https://github.com/192d-Wing/mdlint/workflows/Security%20Audit/badge.svg)](https://github.com/192d-Wing/mdlint/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/192d-Wing/mdlint/branch/main/graph/badge.svg)](https://codecov.io/gh/192d-Wing/mdlint)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A fast Markdown linter written in Rust, inspired by [markdownlint](https://github.com/DavidAnson/markdownlint).

## Features

- **54 lint rules** (MD001-MD060) enforcing Markdown best practices
- **Automatic fixing** for 24+ rules with `--fix` flag
- **Rich error display** with source context and colored underlines pointing to errors
- **Multiple output formats** — text (default), JSON, or SARIF
- **Configuration** via JSON, YAML, or TOML files with auto-discovery
- **High performance** — parallel file processing with optimized rules
- **Library + CLI** — use as a Rust crate or standalone command-line tool

## Installation

### From source

```sh
cargo install --path .
```

### As a library dependency

```toml
[dependencies]
mdlint = { version = "0.1", default-features = false }
```

## CLI Usage

### Basic Commands

```sh
# Lint files
mdlint README.md docs/*.md

# Lint with auto-fix
mdlint --fix README.md

# Lint a directory recursively
mdlint docs/

# Lint from stdin
cat README.md | mdlint --stdin

# List all available rules with descriptions
mdlint --list-rules
```

### Configuration Management

```sh
# Initialize a new config file with defaults
mdlint init

# Initialize with custom path and format
mdlint init --output .mdlint.yaml --format yaml

# Use a specific config file
mdlint --config .markdownlint.json README.md

# Enable/disable specific rules on the fly
mdlint --enable MD001 --disable MD013 README.md

# Combine multiple rule overrides
mdlint --config base.json --enable MD001 --disable MD033 docs/
```

### Output Control

```sh
# Output in JSON format (machine-readable)
mdlint --output-format json README.md

# Output in SARIF format (for CI/CD integration)
mdlint --output-format sarif README.md

# Quiet mode - only show filenames with errors
mdlint --quiet docs/

# Verbose mode - show detailed error statistics
mdlint --verbose docs/

# Disable colored output (for CI environments)
mdlint --no-color README.md
```

### Advanced Usage

```sh
# Ignore specific files/patterns
mdlint --ignore "**/node_modules/**" --ignore "**/.git/**" .

# Combine multiple options
mdlint --config .mdlint.json \
       --ignore "build/**" \
       --ignore "dist/**" \
       --fix \
       --verbose \
       .

# Fix with specific rules disabled
mdlint --fix --disable MD013 --disable MD033 docs/

# Stdin with fix output to stdout
cat README.md | mdlint --stdin --fix > README_fixed.md
```

### Example Output

mdlint provides rich error display with source context:

```text
README.md: 42: MD009/no-trailing-spaces Trailing spaces [Expected: 0; Actual: 3]
  42 |
  42 |  This line has trailing spaces
     |                               ^^^

README.md: 58: MD034/no-bare-urls Bare URL used
  58 |
  58 |  Visit https://example.com for more info.
     |        ^^^^^^^^^^^^^^^^^^^
```

### Commands

| Command | Description |
|---------|-------------|
| `mdlint [FILES...]` | Lint markdown files (default command) |
| `mdlint init` | Create a new configuration file with defaults |

### Options

| Flag | Description |
|------|-------------|
| `-f`, `--fix` | Automatically fix violations where possible |
| `-c`, `--config <PATH>` | Path to configuration file (.json, .yaml, or .toml) |
| `-o`, `--output-format <FORMAT>` | Output format: `text` (default), `json`, or `sarif` |
| `--ignore <PATTERN>` | Glob pattern to ignore (can be repeated) |
| `--stdin` | Read input from stdin instead of files |
| `--list-rules` | List all available linting rules with descriptions |
| `--enable <RULE>` | Enable specific rule (can be repeated) |
| `--disable <RULE>` | Disable specific rule (can be repeated) |
| `-v`, `--verbose` | Show detailed output with error statistics |
| `-q`, `--quiet` | Quiet mode - only show filenames with errors |
| `--no-color` | Disable colored output |
| `--no-inline-config` | Disable inline configuration comments |

## Library Usage

```rust
use mdlint::{lint_sync, apply_fixes, LintOptions};

let options = LintOptions {
    files: vec!["README.md".to_string()],
    ..Default::default()
};

let results = lint_sync(&options).unwrap();
for (file, errors) in results.iter() {
    for error in errors {
        println!("{}: {}", file, error);
    }
}
```

### Auto-fixing

```rust
use mdlint::{lint_sync, apply_fixes, LintOptions};
use std::collections::HashMap;

let content = "# Title\n\nSome text   \n";
let mut strings = HashMap::new();
strings.insert("test.md".to_string(), content.to_string());

let options = LintOptions { strings, ..Default::default() };
let results = lint_sync(&options).unwrap();

if let Some(errors) = results.get("test.md") {
    let fixed = apply_fixes(content, errors);
    println!("{}", fixed); // trailing whitespace removed
}
```

## Configuration

Create a `.markdownlint.json` (or `.yaml` / `.toml`) file:

```json
{
  "default": true,
  "MD013": { "line_length": 120 },
  "MD033": false
}
```

Rules can be enabled/disabled by name (`"MD013"`) or alias (`"line-length"`). Pass a boolean to enable/disable, or an object to configure options.

## Rules

| Rule | Alias | Description | Fixable |
|------|-------|-------------|---------|
| MD001 | heading-increment | Heading levels should increment by one | |
| MD003 | heading-style | Heading style | |
| MD004 | ul-style | Unordered list style | Yes |
| MD005 | list-indent | Inconsistent indentation for list items | Yes |
| MD007 | ul-indent | Unordered list indentation | Yes |
| MD009 | no-trailing-spaces | Trailing spaces | Yes |
| MD010 | no-hard-tabs | Hard tabs | Yes |
| MD011 | no-reversed-links | Reversed link syntax | **Yes** |
| MD012 | no-multiple-blanks | Multiple consecutive blank lines | Yes |
| MD013 | line-length | Line length | |
| MD014 | commands-show-output | Dollar signs used before commands | |
| MD018 | no-missing-space-atx | No space after hash on atx heading | Yes |
| MD019 | no-multiple-space-atx | Multiple spaces after hash on atx heading | Yes |
| MD022 | blanks-around-headings | Headings should be surrounded by blank lines | Yes |
| MD023 | heading-start-left | Headings must start at the beginning of the line | **Yes** |
| MD024 | no-duplicate-heading | No duplicate heading content | |
| MD025 | single-title | Single title / single h1 | |
| MD026 | no-trailing-punctuation | Trailing punctuation in heading | **Yes** |
| MD027 | no-multiple-space-blockquote | Multiple spaces after blockquote symbol | Yes |
| MD028 | no-blanks-blockquote | Blank line inside blockquote | |
| MD029 | ol-prefix | Ordered list item prefix | Yes |
| MD030 | list-marker-space | Spaces after list markers | |
| MD031 | blanks-around-fences | Fenced code blocks should be surrounded by blank lines | Yes |
| MD032 | blanks-around-lists | Lists should be surrounded by blank lines | Yes |
| MD033 | no-inline-html | Inline HTML | |
| MD034 | no-bare-urls | Bare URL used | **Yes** |
| MD035 | hr-style | Horizontal rule style | **Yes** |
| MD036 | no-emphasis-as-heading | Emphasis used instead of a heading | |
| MD037 | no-space-in-emphasis | Spaces inside emphasis markers | **Yes** |
| MD038 | no-space-in-code | Spaces inside code span elements | **Yes** |
| MD039 | no-space-in-links | Spaces inside link text | **Yes** |
| MD040 | fenced-code-language | Fenced code blocks should have a language specified | **Yes** |
| MD041 | first-line-heading | First line in a file should be a top-level heading | |
| MD042 | no-empty-links | No empty links | |
| MD044 | proper-names | Proper names should have correct capitalization | **Yes** |
| MD045 | no-alt-text | Images should have alternate text | |
| MD046 | code-block-style | Code block style | |
| MD047 | single-trailing-newline | Files should end with a single trailing newline | Yes |
| MD048 | code-fence-style | Code fence style | **Yes** |
| MD049 | emphasis-style | Emphasis style | **Yes** |
| MD050 | strong-style | Strong style | **Yes** |
| MD051 | link-fragments | Link fragments should be valid | |
| MD052 | reference-links-images | Reference links and images should use a label that is defined | |
| MD053 | link-image-reference-definitions | Link and image reference definitions should be needed | |
| MD054 | link-image-style | Link and image style | |
| MD058 | blanks-around-tables | Tables should be surrounded by blank lines | **Yes** |
| MD059 | emphasis-marker-style-math | Emphasis marker style in math | |
| MD060 | dollar-in-code-fence | Dollar signs in fenced code blocks | |

**Bold** entries indicate newly added auto-fix support.

## License

Apache-2.0
