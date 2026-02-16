# mkdlint

[![docs.rs](https://img.shields.io/docsrs/mkdlint?style=for-the-badge&logo=rust)](https://docs.rs/ntp-usg/latest/mkdlint)
[![Crates.io](https://img.shields.io/crates/v/mkdlint.svg?style=for-the-badge&logo=rust)](https://crates.io/crates/mkdlint)
![Crates.io Total Downloads](https://img.shields.io/crates/d/mkdlint?style=for-the-badge&logo=rust)
[![GitHub License](https://img.shields.io/github/license/192d-Wing/mkdlint?style=for-the-badge)](https://github.com/192d-Wing/mkdlint/blob/main/LICENSE)
[![Security Audit](https://img.shields.io/github/actions/workflow/status/192d-Wing/mkdlint/security.yml?branch=main&style=for-the-badge&logo=github&label=Security%20Audit)](https://github.com/192d-Wing/mkdlint/actions/workflows/security.yml)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/192d-Wing/mkdlint/ci.yml?branch=main&style=for-the-badge&logo=github)](https://github.com/192d-Wing/mkdlint/actions/workflows/ci.yml)
[![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/192d-Wing/mkdlint?style=for-the-badge&logo=github)](https://github.com/192d-Wing/mkdlint/issues)
[![GitHub Issues or Pull Requests](https://img.shields.io/github/issues-pr/192d-Wing/mkdlint?style=for-the-badge&logo=github)](https://github.com/192d-Wing/mkdlint/pulls)
[![Codecov](https://img.shields.io/codecov/c/github/192d-Wing/mkdlint?style=for-the-badge&logo=codecov)](https://codecov.io/github/192d-Wing/mkdlint)

A fast Markdown linter written in Rust, inspired by [markdownlint](https://github.com/DavidAnson/markdownlint).

## Features

- **54 lint rules** (MD001-MD060) enforcing Markdown best practices
- **🎉 Automatic fixing** for **43 rules (80% coverage)** with `--fix` flag
- **💡 Helpful suggestions** for all rules with actionable guidance
- **Language Server Protocol (LSP)** — real-time linting in VS Code, Neovim, and other editors
- **GitHub Action** — native integration with SARIF Code Scanning
- **Rich error display** with source context and colored underlines pointing to errors
- **Multiple output formats** — text (default), JSON, or SARIF
- **Configuration** via JSON, YAML, or TOML files with auto-discovery
- **High performance** — parallel file processing with optimized rules
- **Library + CLI** — use as a Rust crate or standalone command-line tool

## Auto-Fix Showcase

mkdlint can automatically fix **43 out of 54 rules (80%)**! Here are some examples:

### Before Auto-Fix

```markdown
#Missing space after hash

![](image.png)

# Title
# Another Title

>Blockquote with  trailing spaces
>
>And blank lines inside

[link](http://example.com)
http://example.com

$ npm install
$ echo "commands with dollar signs"
```

### After `mkdlint --fix`

```markdown
# Missing space after hash

![image](image.png)

# Title

## Another Title

> Blockquote with trailing spaces
> And blank lines inside

[link](http://example.com)
<http://example.com>

npm install
echo "commands with dollar signs"
```

### What Gets Fixed Automatically

- **Headings**: spacing, levels, ATX style consistency
- **Links & Images**: alt text, bare URLs, unused references
- **Lists**: indentation, marker consistency, spacing
- **Code**: fence styles, dollar sign prefixes, language tags
- **Whitespace**: trailing spaces, blank lines, tabs
- **Tables**: pipe consistency, surrounding blank lines
- And much more!

## Installation

### CLI Tool

```sh
# From crates.io
cargo install mkdlint

# From source
cargo install --path .
```

### Language Server (LSP)

```sh
# Install with LSP feature
cargo install mkdlint --features lsp

# The binary will be available as: mkdlint-lsp
```

### GitHub Action

Add to your workflow:

```yaml
- uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
  with:
    files: '.'
```

See [GitHub Action documentation](.github/actions/mkdlint/README.md) for full details.

### As a library dependency

```toml
[dependencies]
mkdlint = "0.6"

# With async support
mkdlint = { version = "0.6", features = ["async"] }

# With LSP support
mkdlint = { version = "0.6", features = ["lsp"] }
```

## CLI Usage

### Basic Commands

```sh
# Lint files
mkdlint README.md docs/*.md

# Lint with auto-fix
mkdlint --fix README.md

# Lint a directory recursively
mkdlint docs/

# Lint from stdin
cat README.md | mkdlint --stdin

# List all available rules with descriptions
mkdlint --list-rules
```

### Configuration Management

```sh
# Initialize a new config file with defaults
mkdlint init

# Initialize with custom path and format
mkdlint init --output .mkdlint.yaml --format yaml

# Use a specific config file
mkdlint --config .markdownlint.json README.md

# Enable/disable specific rules on the fly
mkdlint --enable MD001 --disable MD013 README.md

# Combine multiple rule overrides
mkdlint --config base.json --enable MD001 --disable MD033 docs/
```

### Output Control

```sh
# Output in JSON format (machine-readable)
mkdlint --output-format json README.md

# Output in SARIF format (for CI/CD integration)
mkdlint --output-format sarif README.md

# Quiet mode - only show filenames with errors
mkdlint --quiet docs/

# Verbose mode - show detailed error statistics
mkdlint --verbose docs/

# Disable colored output (for CI environments)
mkdlint --no-color README.md
```

### Advanced Usage

```sh
# Ignore specific files/patterns
mkdlint --ignore "**/node_modules/**" --ignore "**/.git/**" .

# Combine multiple options
mkdlint --config .mkdlint.json \
       --ignore "build/**" \
       --ignore "dist/**" \
       --fix \
       --verbose \
       .

# Fix with specific rules disabled
mkdlint --fix --disable MD013 --disable MD033 docs/

# Stdin with fix output to stdout
cat README.md | mkdlint --stdin --fix > README_fixed.md
```

### Example Output

mkdlint provides rich error display with source context:

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
| `mkdlint [FILES...]` | Lint markdown files (default command) |
| `mkdlint init` | Create a new configuration file with defaults |

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

## Language Server Protocol (LSP)

mkdlint includes a full-featured Language Server for real-time linting in your editor.

### VS Code Setup

1. Install the LSP binary:

   ```sh
   cargo install mkdlint --features lsp
   ```

2. Configure VS Code (`.vscode/settings.json`):

   ```json
   {
     "mkdlint.server.path": "/path/to/mkdlint-lsp",
     "mkdlint.server.enable": true
   }
   ```

3. Features:

   - Real-time diagnostics as you type
   - Quick-fix code actions (auto-fix on Ctrl+.)
   - **"Fix All" command** - Fix all issues in a document at once
   - Respects your `.markdownlint.json` config
   - Debounced linting (300ms) for performance

### Neovim Setup

```lua
require('lspconfig').mkdlint.setup{
  cmd = { '/path/to/mkdlint-lsp' },
  filetypes = { 'markdown' },
  root_dir = require('lspconfig.util').root_pattern('.markdownlint.json', '.git'),
}
```

### Other Editors

Any editor with LSP support can use `mkdlint-lsp`. The server uses stdio for communication and supports:

- `textDocument/didOpen`, `didChange`, `didSave`, `didClose`
- `textDocument/codeAction` (for individual auto-fixes)
- `workspace/executeCommand` (for "Fix All" command)
- Full document synchronization

**Usage Tips:**

- Use quick-fix (Ctrl+.) to fix individual issues
- Use "Fix All" command to apply all fixes at once
- All 34 auto-fixable rules are supported

## CI/CD Integration

Use mkdlint in your CI/CD with native GitHub Code Scanning integration:

```yaml
name: Lint Markdown
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write

    steps:
      - uses: actions/checkout@v4

      - uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
        with:
          files: '.'
          fix: true  # Optional: auto-fix issues
```

**Features:**

- Pre-built binaries for Linux, macOS, Windows (x86_64, aarch64)
- Automatic binary caching (10-100x faster subsequent runs)
- SARIF output with automatic Code Scanning upload
- Auto-fix support with commit integration
- Cargo fallback if binary download fails

See [full documentation](.github/actions/mkdlint/README.md) for all options.

## Library Usage

```rust
use mkdlint::{lint_sync, apply_fixes, LintOptions};

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
use mkdlint::{lint_sync, apply_fixes, LintOptions};
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
| MD001 | heading-increment | Heading levels should increment by one | ***Yes*** |
| MD003 | heading-style | Heading style | |
| MD004 | ul-style | Unordered list style | Yes |
| MD005 | list-indent | Inconsistent indentation for list items | Yes |
| MD007 | ul-indent | Unordered list indentation | Yes |
| MD009 | no-trailing-spaces | Trailing spaces | Yes |
| MD010 | no-hard-tabs | Hard tabs | Yes |
| MD011 | no-reversed-links | Reversed link syntax | **Yes** |
| MD012 | no-multiple-blanks | Multiple consecutive blank lines | Yes |
| MD013 | line-length | Line length | |
| MD014 | commands-show-output | Dollar signs used before commands | ***Yes*** |
| MD018 | no-missing-space-atx | No space after hash on atx heading | Yes |
| MD019 | no-multiple-space-atx | Multiple spaces after hash on atx heading | Yes |
| MD020 | no-missing-space-closed-atx | No space inside hashes on closed atx heading | ***Yes*** |
| MD021 | no-multiple-space-closed-atx | Multiple spaces inside hashes on closed atx heading | ***Yes*** |
| MD022 | blanks-around-headings | Headings should be surrounded by blank lines | Yes |
| MD023 | heading-start-left | Headings must start at the beginning of the line | **Yes** |
| MD024 | no-duplicate-heading | No duplicate heading content | |
| MD025 | single-title | Single title / single h1 | |
| MD026 | no-trailing-punctuation | Trailing punctuation in heading | **Yes** |
| MD027 | no-multiple-space-blockquote | Multiple spaces after blockquote symbol | Yes |
| MD028 | no-blanks-blockquote | Blank line inside blockquote | ***Yes*** |
| MD029 | ol-prefix | Ordered list item prefix | Yes |
| MD030 | list-marker-space | Spaces after list markers | ***Yes*** |
| MD031 | blanks-around-fences | Fenced code blocks should be surrounded by blank lines | Yes |
| MD032 | blanks-around-lists | Lists should be surrounded by blank lines | Yes |
| MD033 | no-inline-html | Inline HTML | |
| MD034 | no-bare-urls | Bare URL used | **Yes** |
| MD035 | hr-style | Horizontal rule style | **Yes** |
| MD036 | no-emphasis-as-heading | Emphasis used instead of a heading | ***Yes*** |
| MD037 | no-space-in-emphasis | Spaces inside emphasis markers | **Yes** |
| MD038 | no-space-in-code | Spaces inside code span elements | **Yes** |
| MD039 | no-space-in-links | Spaces inside link text | **Yes** |
| MD040 | fenced-code-language | Fenced code blocks should have a language specified | **Yes** |
| MD041 | first-line-heading | First line in a file should be a top-level heading | ***Yes*** |
| MD042 | no-empty-links | No empty links | ***Yes*** |
| MD044 | proper-names | Proper names should have correct capitalization | **Yes** |
| MD045 | no-alt-text | Images should have alternate text | |
| MD046 | code-block-style | Code block style | |
| MD047 | single-trailing-newline | Files should end with a single trailing newline | ***Yes*** |
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

**Bold** entries indicate auto-fix added in v0.2.0. ***Bold+Italic*** entries are new in v0.3.0 and v0.4.0.

## License

Apache-2.0
