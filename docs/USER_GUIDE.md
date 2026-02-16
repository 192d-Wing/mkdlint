# mkdlint User Guide

Complete guide to using mkdlint for linting and auto-fixing Markdown files.

## Table of Contents

- [Getting Started](#getting-started)
- [Configuration](#configuration)
- [Auto-Fix Guide](#auto-fix-guide)
- [IDE Integration](#ide-integration)
- [CI/CD Integration](#ci-cd-integration)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)

## Getting Started

### Installation

```bash
# Install CLI tool
cargo install mkdlint

# Install with LSP support
cargo install mkdlint --features lsp
```

### Basic Usage

```bash
# Lint a single file
mkdlint README.md

# Lint multiple files
mkdlint README.md CONTRIBUTING.md

# Lint a directory recursively
mkdlint docs/

# Lint with glob patterns
mkdlint "**/*.md"

# Auto-fix issues
mkdlint --fix README.md
```

### Understanding Output

mkdlint provides colored, detailed output for each error:

```
README.md:5:1 error MD001/heading-increment Heading levels should only increment by one level at a time [Expected: h2; Actual: h3]
  💡 Suggestion: Heading levels should increment by one level at a time
  🔧 Fix available - use --fix to apply automatically
```

- **Line:Column** - Exact location of the issue
- **Rule Code** - MD001 (numeric) and heading-increment (descriptive name)
- **Description** - What the issue is
- **Detail** - Additional context in brackets
- **💡 Suggestion** - How to fix it manually
- **🔧 Fix indicator** - Tells you `--fix` can auto-fix this

## Configuration

### Configuration Files

mkdlint automatically discovers configuration files:

- `.markdownlint.json` (JSON)
- `.markdownlint.yaml` or `.markdownlint.yml` (YAML)
- `.markdownlint.toml` (TOML)

Files are searched from the current directory up to the root.

### Creating a Config File

```bash
# Interactive wizard
mkdlint init

# Create with specific format
mkdlint init --format json
mkdlint init --format yaml
mkdlint init --format toml
```

### Configuration Examples

#### JSON Format

```json
{
  "default": true,
  "MD013": false,
  "MD033": {
    "allowed_elements": ["br", "img"]
  },
  "line-length": {
    "line_length": 120,
    "code_blocks": false
  }
}
```

#### YAML Format

```yaml
default: true
MD013: false
MD033:
  allowed_elements: ["br", "img"]
line-length:
  line_length: 120
  code_blocks: false
```

#### TOML Format

```toml
default = true
MD013 = false

[MD033]
allowed_elements = ["br", "img"]

[line-length]
line_length = 120
code_blocks = false
```

### Common Configuration Options

#### Disable Specific Rules

```json
{
  "default": true,
  "MD013": false,
  "MD033": false
}
```

#### Configure Rule Behavior

```json
{
  "MD003": {
    "style": "atx"
  },
  "MD004": {
    "style": "dash"
  },
  "MD007": {
    "indent": 4
  },
  "MD013": {
    "line_length": 120,
    "code_blocks": false
  }
}
```

#### Extends Feature

```json
{
  "extends": "../.markdownlint.json",
  "MD013": {
    "line_length": 100
  }
}
```

### Command-Line Overrides

```bash
# Disable specific rules
mkdlint --disable MD013 MD033 file.md

# Enable specific rules only
mkdlint --enable MD001 MD002 file.md

# Ignore patterns
mkdlint --ignore "**/node_modules/**" --ignore "vendor/**" .
```

## Auto-Fix Guide

### What Can Be Fixed

mkdlint can automatically fix **43 out of 54 rules (80%)**:

**Headings (11 rules)**:
- MD001: Heading level increments
- MD003: Heading style consistency
- MD018: No space after hash
- MD019: Multiple spaces after hash
- MD020: Spaces in closed ATX headings
- MD021: Multiple spaces in closed ATX
- MD022: Blank lines around headings
- MD023: Indentation of headings
- MD025: Multiple H1 headings
- MD026: Trailing punctuation
- MD041: First line heading

**Lists (8 rules)**:
- MD004: List marker style
- MD005: List indentation
- MD007: Nested list indentation
- MD029: List numbering
- MD030: List marker spacing
- MD031: Blank lines around fenced code
- MD032: Blank lines around lists
- MD058: Tables surrounded by blank lines

**Links & Images (6 rules)**:
- MD011: Reversed link syntax
- MD034: Bare URLs
- MD039: Spaces inside link text
- MD042: Empty links
- MD045: Images without alt text
- MD053: Unused link definitions

**Code Blocks (7 rules)**:
- MD014: Dollar signs in commands
- MD031: Blank lines around code blocks
- MD040: Fenced code language
- MD046: Code block style
- MD047: Single trailing newline
- MD048: Code fence style
- MD060: Dollar signs in fenced blocks

**Whitespace (6 rules)**:
- MD009: Trailing spaces
- MD010: Hard tabs
- MD012: Multiple blank lines
- MD027: Multiple spaces after blockquote
- MD028: Blank lines in blockquote
- MD047: File end newline

**Emphasis & Inline (5 rules)**:
- MD035: Horizontal rule style
- MD037: Spaces inside emphasis
- MD038: Spaces inside code spans
- MD049: Emphasis style
- MD050: Strong emphasis style

**Tables (2 rules)**:
- MD055: Table pipe style
- MD058: Blank lines around tables

**Other (1 rule)**:
- MD044: Proper names capitalization

### Using Auto-Fix

```bash
# Fix a single file
mkdlint --fix README.md

# Fix multiple files
mkdlint --fix docs/**/*.md

# Fix recursively
mkdlint --fix .

# Preview changes without applying
mkdlint README.md  # See what would be fixed

# Fix specific rules only
mkdlint --fix --enable MD001 MD003 README.md
```

### Safe Auto-Fixing

mkdlint's auto-fix is safe:
- ✅ Only fixes issues it's confident about
- ✅ Preserves file formatting and structure
- ✅ Doesn't modify code blocks or HTML
- ✅ Works on UTF-8 encoded files
- ⚠️ Always commit before running `--fix` on large changes

### What Cannot Be Fixed

11 rules cannot be auto-fixed because they require human judgment:

- **MD013**: Line length (requires context-aware wrapping)
- **MD024**: Duplicate headings (which to rename?)
- **MD033**: Inline HTML (may be intentional)
- **MD043**: Required heading structure (document-specific)
- **MD046**: Code block style (complex conversion)
- **MD051**: Link fragment validation (requires checking targets)
- **MD052**: Reference link validation (external validation)
- **MD054**: Link/image style (multiple valid options)
- **MD056**: Table column count (structural issue)
- **MD059**: Emphasis vs math syntax (context-dependent)

## IDE Integration

### Visual Studio Code

1. Install the mkdlint LSP extension (coming soon) or configure manually:

```json
// settings.json
{
  "mkdlint.enable": true,
  "mkdlint.config": "${workspaceFolder}/.markdownlint.json",
  "mkdlint.lsp.path": "/path/to/mkdlint-lsp"
}
```

2. Set up auto-fix on save:

```json
{
  "editor.codeActionsOnSave": {
    "source.fixAll.mkdlint": true
  }
}
```

### Neovim

Using `nvim-lspconfig`:

```lua
require('lspconfig').mkdlint.setup{
  cmd = { "mkdlint-lsp" },
  filetypes = { "markdown" },
  root_dir = require('lspconfig').util.root_pattern(
    ".markdownlint.json",
    ".markdownlint.yaml",
    ".git"
  ),
  settings = {
    mkdlint = {
      config = ".markdownlint.json"
    }
  }
}
```

### Emacs

Using `lsp-mode`:

```elisp
(use-package lsp-mode
  :hook (markdown-mode . lsp)
  :config
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection '("mkdlint-lsp"))
    :activation-fn (lsp-activate-on "markdown")
    :server-id 'mkdlint)))
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Lint Markdown

on: [push, pull_request]

jobs:
  markdown-lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
        with:
          files: '.'
          fix: false
          fail-on-error: true
```

### GitLab CI

```yaml
markdown-lint:
  image: rust:latest
  script:
    - cargo install mkdlint
    - mkdlint --output-format json . > mkdlint-report.json
  artifacts:
    reports:
      codequality: mkdlint-report.json
```

### Pre-commit Hook

`.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: mkdlint
        name: mkdlint
        entry: mkdlint
        language: system
        files: \.md$
```

### Make Target

```makefile
.PHONY: lint-md
lint-md:
\t@mkdlint docs/ README.md

.PHONY: fix-md
fix-md:
\t@mkdlint --fix docs/ README.md
```

## Troubleshooting

### Common Issues

#### "No configuration file found"

**Solution**: Create a config file or use command-line flags:

```bash
mkdlint init
# or
mkdlint --disable MD013 file.md
```

#### "Too many errors"

**Solution**: Start by fixing auto-fixable issues:

```bash
mkdlint --fix .
```

Then disable rules you don't need:

```json
{
  "MD013": false,
  "MD033": false
}
```

#### "Rule not working as expected"

**Solution**: Check the rule configuration:

```bash
# List all rules with descriptions
mkdlint --list-rules

# Test with only specific rules
mkdlint --enable MD001 file.md
```

#### "Performance issues with large repos"

**Solution**: Use ignore patterns:

```bash
mkdlint --ignore "**/node_modules/**" --ignore "vendor/**" .
```

Or add to config:

```json
{
  "ignores": ["**/node_modules/**", "vendor/**"]
}
```

### Debug Mode

```bash
# Enable verbose output
mkdlint --verbose file.md

# See what files are being processed
mkdlint --verbose .
```

## FAQ

### Q: How do I disable a rule for just one line?

A: Use inline comments:

```markdown
<!-- markdownlint-disable-next-line MD013 -->
This is a very long line that would normally trigger MD013 but won't because of the comment above.

<!-- markdownlint-disable MD033 -->
<div>HTML is allowed here</div>
<!-- markdownlint-enable MD033 -->
```

### Q: Can I use mkdlint with pre-commit?

A: Yes! See the [Pre-commit Hook](#pre-commit-hook) section.

### Q: Does mkdlint work with GitHub Flavored Markdown?

A: Yes! mkdlint uses the CommonMark spec which is compatible with GFM. It includes rules for tables and other GFM extensions.

### Q: How do I fix only certain types of errors?

A: Use the `--enable` flag:

```bash
mkdlint --fix --enable MD001 MD003 MD018 file.md
```

### Q: Can I create custom rules?

A: Not yet, but it's on the roadmap! Custom rule API is planned for v0.7.0.

### Q: Why is auto-fix changing my code blocks?

A: Make sure your code blocks are properly fenced with \`\`\`. Indented code blocks (4 spaces) can sometimes be confused with regular indented text.

### Q: How do I ignore specific files?

A: Use the `--ignore` flag or add to config:

```bash
mkdlint --ignore "CHANGELOG.md" --ignore "**/vendor/**" .
```

Or in your config file:

```json
{
  "ignores": ["CHANGELOG.md", "**/vendor/**"]
}
```

### Q: Is mkdlint faster than markdownlint?

A: Yes! mkdlint is written in Rust and uses parallel processing, making it significantly faster on large repositories.

### Q: Can I use mkdlint as a library?

A: Yes! Add it to your `Cargo.toml`:

```toml
[dependencies]
mkdlint = "0.6"
```

See the [API documentation](https://docs.rs/mkdlint) for details.

## Need More Help?

- 📖 [Full documentation](https://github.com/192d-Wing/mkdlint)
- 🐛 [Report issues](https://github.com/192d-Wing/mkdlint/issues)
- 💬 [Discussions](https://github.com/192d-Wing/mkdlint/discussions)
- 📧 Contact: [maintainers](https://github.com/192d-Wing/mkdlint/blob/main/MAINTAINERS.md)
