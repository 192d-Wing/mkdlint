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

mkdlint can be integrated into your editor in multiple ways for real-time feedback.

### Option 1: CLI Integration (Simple & Universal)

The easiest way to integrate mkdlint is through your editor's task runner or build system.

**Pros:**
- Works in any editor
- No additional setup required
- Can use watch mode for auto-linting

**Cons:**
- No inline diagnostics
- Manual workflow

### Option 2: LSP Integration (Advanced)

For real-time inline diagnostics and code actions, use `mkdlint-lsp` (built with `--features lsp`).

**Note:** The LSP server is currently in development. For now, use CLI integration with watch mode.

---

### Visual Studio Code

#### Method 1: Tasks (Recommended)

Create `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "mkdlint",
      "type": "shell",
      "command": "mkdlint",
      "args": ["${file}"],
      "presentation": {
        "reveal": "silent",
        "panel": "shared"
      },
      "problemMatcher": {
        "owner": "mkdlint",
        "fileLocation": "absolute",
        "pattern": {
          "regexp": "^(.+):(\\d+):(\\d+)\\s+(error|warning)\\s+(.+)$",
          "file": 1,
          "line": 2,
          "column": 3,
          "severity": 4,
          "message": 5
        }
      }
    },
    {
      "label": "mkdlint --fix",
      "type": "shell",
      "command": "mkdlint",
      "args": ["--fix", "${file}"],
      "presentation": {
        "reveal": "always",
        "panel": "shared"
      }
    },
    {
      "label": "mkdlint --watch",
      "type": "shell",
      "command": "mkdlint",
      "args": ["--watch", "${workspaceFolder}"],
      "isBackground": true,
      "presentation": {
        "reveal": "always",
        "panel": "dedicated"
      }
    }
  ]
}
```

**Usage:**
1. Open Command Palette (`Cmd/Ctrl+Shift+P`)
2. Run `Tasks: Run Task` → Select `mkdlint`
3. Or bind to keyboard shortcut in `keybindings.json`:

```json
{
  "key": "cmd+shift+l",
  "command": "workbench.action.tasks.runTask",
  "args": "mkdlint"
}
```

#### Method 2: Auto-run on Save

Add to `.vscode/settings.json`:

```json
{
  "emeraldwalk.runonsave": {
    "commands": [
      {
        "match": "\\.md$",
        "cmd": "mkdlint --fix ${file}"
      }
    ]
  }
}
```

Requires: [Run on Save extension](https://marketplace.visualstudio.com/items?itemName=emeraldwalk.RunOnSave)

#### Method 3: Watch Mode in Terminal

1. Open integrated terminal
2. Run: `mkdlint --watch --fix`
3. Split editor and terminal side-by-side
4. Edit markdown files → see live results

---

### Neovim / Vim

#### Method 1: ALE (Asynchronous Lint Engine)

Add to your `init.vim` or `init.lua`:

**Vim:**
```vim
" Add mkdlint as a linter
let g:ale_linters = {
\   'markdown': ['mkdlint'],
\}

" Enable auto-fix on save
let g:ale_fixers = {
\   'markdown': ['mkdlint'],
\}
let g:ale_fix_on_save = 1

" Configure mkdlint executable
let g:ale_markdown_mkdlint_executable = 'mkdlint'
let g:ale_markdown_mkdlint_options = ''
```

**Lua (Neovim):**
```lua
vim.g.ale_linters = {
  markdown = {'mkdlint'}
}
vim.g.ale_fixers = {
  markdown = {'mkdlint'}
}
vim.g.ale_fix_on_save = 1
```

#### Method 2: null-ls (Neovim only)

```lua
local null_ls = require("null-ls")

null_ls.setup({
  sources = {
    null_ls.builtins.diagnostics.mkdlint,
    null_ls.builtins.formatting.mkdlint,
  },
})

-- Auto-format on save
vim.api.nvim_create_autocmd("BufWritePre", {
  pattern = "*.md",
  callback = function()
    vim.lsp.buf.format()
  end,
})
```

#### Method 3: Simple Autocmd

Add to your config:

```vim
" Auto-fix on save
autocmd BufWritePost *.md silent !mkdlint --fix %
```

Or in Lua:

```lua
vim.api.nvim_create_autocmd("BufWritePost", {
  pattern = "*.md",
  command = "silent !mkdlint --fix %"
})
```

#### Method 4: Custom Command

```vim
" Add :Mkdlint command
command! Mkdlint :!mkdlint --fix %

" Keybinding
nnoremap <leader>ml :Mkdlint<CR>
```

**Recommended Workflow:**
- Use `:!mkdlint --watch --fix` in a tmux/screen split
- Or run in `:terminal` split with `:split | terminal mkdlint --watch --fix`

---

### Emacs

#### Method 1: Flycheck

Add to your `init.el`:

```elisp
(require 'flycheck)

;; Define mkdlint checker
(flycheck-define-checker markdown-mkdlint
  "A Markdown syntax checker using mkdlint."
  :command ("mkdlint" source)
  :error-patterns
  ((error line-start
          (file-name) ":" line ":" column
          " error " (message)
          line-end))
  :modes (markdown-mode gfm-mode))

;; Add to markdown checkers
(add-to-list 'flycheck-checkers 'markdown-mkdlint)

;; Enable flycheck in markdown
(add-hook 'markdown-mode-hook 'flycheck-mode)
```

#### Method 2: Auto-fix on Save

```elisp
(defun mkdlint-fix-buffer ()
  "Run mkdlint --fix on current buffer."
  (interactive)
  (when (eq major-mode 'markdown-mode)
    (shell-command-to-string
     (format "mkdlint --fix %s" (buffer-file-name)))
    (revert-buffer :ignore-auto :noconfirm)))

;; Keybinding
(define-key markdown-mode-map (kbd "C-c C-f") 'mkdlint-fix-buffer)

;; Auto-fix on save
(add-hook 'markdown-mode-hook
          (lambda ()
            (add-hook 'after-save-hook 'mkdlint-fix-buffer nil t)))
```

#### Method 3: Compilation Mode

```elisp
(defun mkdlint-current-file ()
  "Run mkdlint on current file."
  (interactive)
  (compile (concat "mkdlint " (buffer-file-name))))

(define-key markdown-mode-map (kbd "C-c C-l") 'mkdlint-current-file)
```

---

### Zed

Create `.zed/tasks.json`:

```json
[
  {
    "label": "Lint Markdown",
    "command": "mkdlint",
    "args": ["${ZED_FILE}"]
  },
  {
    "label": "Fix Markdown",
    "command": "mkdlint",
    "args": ["--fix", "${ZED_FILE}"]
  },
  {
    "label": "Watch Markdown",
    "command": "mkdlint",
    "args": ["--watch", "${ZED_WORKTREE_ROOT}"]
  }
]
```

**Usage:**
- `Cmd/Ctrl+Shift+P` → Tasks → Select task

---

### Sublime Text

Create `mkdlint.sublime-build`:

```json
{
  "shell_cmd": "mkdlint \"$file\"",
  "file_regex": "^(.+):(\\d+):(\\d+)\\s+(error|warning)\\s+(.+)$",
  "selector": "text.html.markdown",
  "variants": [
    {
      "name": "Fix",
      "shell_cmd": "mkdlint --fix \"$file\""
    },
    {
      "name": "Watch",
      "shell_cmd": "mkdlint --watch \"$file_path\""
    }
  ]
}
```

**Usage:**
- `Cmd/Ctrl+B` to run
- `Cmd/Ctrl+Shift+B` to select variant

---

### Generic LSP Setup (Future)

Once `mkdlint-lsp` is stable, any LSP-compatible editor can use it:

**Installation:**
```bash
cargo install mkdlint --features lsp
```

**LSP Configuration:**
```json
{
  "command": "mkdlint-lsp",
  "filetypes": ["markdown"],
  "rootPatterns": [".markdownlint.json", ".markdownlint.yaml", ".git"]
}
```

**Editors with LSP support:**
- VS Code (via extension)
- Neovim (nvim-lspconfig)
- Emacs (lsp-mode, eglot)
- Vim (vim-lsp, coc.nvim)
- Sublime Text (LSP package)
- Zed (built-in LSP)
- Helix (built-in LSP)

---

### Recommended Workflows

**For beginners:**
1. Use watch mode in a split terminal: `mkdlint --watch --fix`
2. Simple, visual, works everywhere

**For power users:**
1. Set up editor task/command for manual linting
2. Configure auto-fix on save
3. Use watch mode for real-time feedback during heavy editing

**For teams:**
1. Commit `.markdownlint.json` config
2. Add pre-commit hook
3. Configure CI/CD (see next section)
4. Share editor configs in `.vscode/` or `.zed/`

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

### IDE Integration Issues

#### mkdlint not found in PATH

**Symptoms:**
- Editor shows "command not found: mkdlint"
- Tasks fail to run

**Solution:**
```bash
# Verify mkdlint is installed
which mkdlint

# If not found, install it
cargo install mkdlint

# Add cargo bin to PATH if needed
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc  # or ~/.bashrc
source ~/.zshrc
```

#### Watch mode doesn't detect changes

**Symptoms:**
- `mkdlint --watch` running but not re-linting on file changes

**Solution:**
1. Check file extension (must be .md or .markdown)
2. Verify watched path exists: `mkdlint --watch /path/to/docs`
3. Check file system notifications work:
   ```bash
   # Test with a simple change
   echo "# Test" >> test.md
   ```
4. On macOS, ensure you have required permissions for file watching

#### Auto-fix not working in editor

**Symptoms:**
- Errors are shown but not fixed automatically

**Solution:**
1. Verify `--fix` flag is in the command:
   ```bash
   mkdlint --fix file.md  # Good
   mkdlint file.md        # Won't fix
   ```
2. Check that the rule is fixable (look for 🔧 icon in output)
3. For Vim/Neovim: Ensure buffer is reloaded after fix:
   ```vim
   :e!  " Reload buffer
   ```

#### VS Code task not showing errors

**Symptoms:**
- Task runs but doesn't populate Problems panel

**Solution:**
Ensure your problem matcher regex is correct:
```json
{
  "problemMatcher": {
    "owner": "mkdlint",
    "fileLocation": "absolute",
    "pattern": {
      "regexp": "^(.+):(\\d+):(\\d+)\\s+(error|warning)\\s+(.+)$",
      "file": 1,
      "line": 2,
      "column": 3,
      "severity": 4,
      "message": 5
    }
  }
}
```

#### Neovim/Vim: Auto-fix breaks undo history

**Symptoms:**
- After auto-fix on save, can't undo previous changes

**Solution:**
Use a plugin that preserves undo history:
```lua
-- For Neovim with null-ls
local null_ls = require("null-ls")
null_ls.setup({
  sources = {
    null_ls.builtins.formatting.mkdlint.with({
      extra_args = { "--fix" }
    }),
  },
  -- This preserves undo history
  on_attach = function(client, bufnr)
    vim.api.nvim_create_autocmd("BufWritePre", {
      buffer = bufnr,
      callback = function()
        vim.lsp.buf.format({ bufnr = bufnr })
      end,
    })
  end,
})
```

#### Emacs: Buffer not reloading after fix

**Symptoms:**
- File is fixed on disk but not updated in Emacs

**Solution:**
Add `revert-buffer` to your fix function:
```elisp
(defun mkdlint-fix-buffer ()
  "Run mkdlint --fix on current buffer."
  (interactive)
  (when (eq major-mode 'markdown-mode)
    (shell-command-to-string
     (format "mkdlint --fix %s" (buffer-file-name)))
    (revert-buffer :ignore-auto :noconfirm)))  ;; This line is important
```

#### Performance: Editor sluggish with watch mode

**Symptoms:**
- Editor lags when watch mode is running

**Solution:**
1. Use ignore patterns to exclude large directories:
   ```bash
   mkdlint --watch --ignore "**/node_modules/**" --ignore "vendor/**" .
   ```
2. Watch specific directories only:
   ```bash
   mkdlint --watch --watch-paths docs/ --watch-paths README.md
   ```
3. Increase debounce time (future feature)

#### LSP: "mkdlint-lsp not found"

**Note:** LSP support is currently in development.

**Current Status:**
- CLI integration is fully supported and recommended
- LSP server is planned for future releases
- Use watch mode with terminal splits for now

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
