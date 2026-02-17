# mkdlint Language Server Protocol (LSP)

The mkdlint LSP server provides real-time linting and code actions in your favorite editor.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Editor Setup](#editor-setup)
  - [VS Code](#vs-code)
  - [Neovim](#neovim)
  - [Emacs](#emacs)
  - [Helix](#helix)
  - [Zed](#zed)
  - [Sublime Text](#sublime-text)
- [Architecture](#architecture)
- [Capabilities](#capabilities)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [Performance](#performance)

## Features

✨ **Real-time Diagnostics**
- Lint as you type with 300ms debouncing
- Instant feedback on save (bypasses debounce)
- UTF-8 aware range calculation

🔧 **Code Actions (Quick Fixes)**
- Individual fixes for each fixable error
- "Fix All" command to apply all fixes at once
- 48/53 rules support auto-fix (90.6% coverage)

📂 **Workspace Aware**
- Automatic config discovery (`.markdownlint.json`, `.yaml`, `.yml`)
- Walks up directory tree to workspace root
- Config caching for performance
- Multi-workspace support

⚡ **Performance**
- Debounced edits prevent excessive re-linting
- In-memory document cache
- Parallel file processing (via mkdlint core)

## Installation

### From Pre-built Binary

Download the latest binary from [GitHub Releases](https://github.com/192d-Wing/mkdlint/releases):

```bash
# Linux x86_64
curl -LO https://github.com/192d-Wing/mkdlint/releases/latest/download/mkdlint-linux-x86_64.tar.gz
tar -xzf mkdlint-linux-x86_64.tar.gz
sudo mv mkdlint mkdlint-lsp /usr/local/bin/

# macOS (Apple Silicon)
curl -LO https://github.com/192d-Wing/mkdlint/releases/latest/download/mkdlint-macos-aarch64.tar.gz
tar -xzf mkdlint-macos-aarch64.tar.gz
sudo mv mkdlint mkdlint-lsp /usr/local/bin/

# Verify
mkdlint-lsp --version
```

### From Source

Build with the `lsp` feature enabled:

```bash
cargo install mkdlint --features lsp

# Binary will be at ~/.cargo/bin/mkdlint-lsp
which mkdlint-lsp
```

### From Repository

```bash
git clone https://github.com/192d-Wing/mkdlint.git
cd mkdlint
cargo build --release --features lsp

# Binary at target/release/mkdlint-lsp
```

## Editor Setup

### VS Code

#### Option 1: Using a Generic LSP Client

Install the [vscode-languageclient](https://marketplace.visualstudio.com/items?itemName=vscode.languageclient) extension, then create a custom extension:

**`.vscode/extensions/mkdlint-lsp/package.json`**:
```json
{
  "name": "mkdlint-lsp",
  "version": "1.0.0",
  "engines": {
    "vscode": "^1.75.0"
  },
  "activationEvents": [
    "onLanguage:markdown"
  ],
  "main": "./out/extension.js",
  "contributes": {
    "configuration": {
      "type": "object",
      "title": "mkdlint",
      "properties": {
        "mkdlint.enable": {
          "type": "boolean",
          "default": true,
          "description": "Enable mkdlint LSP"
        },
        "mkdlint.trace.server": {
          "type": "string",
          "enum": ["off", "messages", "verbose"],
          "default": "off",
          "description": "Trace LSP communication"
        }
      }
    }
  }
}
```

**`.vscode/extensions/mkdlint-lsp/src/extension.ts`**:
```typescript
import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const serverOptions: ServerOptions = {
    command: 'mkdlint-lsp',
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'markdown' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/.markdownlint{.json,.yaml,.yml,rc}')
    }
  };

  client = new LanguageClient(
    'mkdlint',
    'mkdlint Language Server',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
```

Compile and reload VS Code to activate.

#### Option 2: Settings-based (Simpler)

If you have a generic LSP extension, add to `.vscode/settings.json`:

```json
{
  "markdown.validate.enabled": false,
  "lsp.servers": {
    "mkdlint": {
      "command": "mkdlint-lsp",
      "filetypes": ["markdown"],
      "rootPatterns": [".markdownlint.json", ".git"]
    }
  }
}
```

### Neovim

#### Using nvim-lspconfig

Add to your Neovim config (`~/.config/nvim/init.lua` or `~/.config/nvim/lua/lsp.lua`):

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define mkdlint LSP config
if not configs.mkdlint then
  configs.mkdlint = {
    default_config = {
      cmd = { 'mkdlint-lsp' },
      filetypes = { 'markdown' },
      root_dir = lspconfig.util.root_pattern(
        '.markdownlint.json',
        '.markdownlint.yaml',
        '.markdownlint.yml',
        '.git'
      ),
      settings = {},
    },
  }
end

-- Setup
lspconfig.mkdlint.setup({
  on_attach = function(client, bufnr)
    -- Enable completion
    vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

    -- Keybindings
    local opts = { noremap = true, silent = true, buffer = bufnr }
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', '<leader>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<leader>f', function()
      vim.lsp.buf.format({ async = true })
    end, opts)

    -- Auto-fix on save
    vim.api.nvim_create_autocmd("BufWritePre", {
      buffer = bufnr,
      callback = function()
        -- Request code actions and apply "Fix All"
        vim.lsp.buf.code_action({
          context = { only = { 'source.fixAll' } },
          apply = true,
        })
      end,
    })
  end,
})
```

#### Minimal Config

```lua
require('lspconfig').mkdlint.setup({})
```

### Emacs

#### Using lsp-mode

Add to your Emacs config (`~/.emacs.d/init.el` or `~/.emacs`):

```elisp
(use-package lsp-mode
  :hook ((markdown-mode . lsp))
  :commands lsp
  :config
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection "mkdlint-lsp")
    :major-modes '(markdown-mode)
    :server-id 'mkdlint
    :priority 1)))

;; Optional: Enable which-key for LSP bindings
(use-package lsp-ui
  :commands lsp-ui-mode
  :config
  (setq lsp-ui-doc-enable t
        lsp-ui-doc-position 'at-point
        lsp-ui-sideline-enable t))
```

#### Auto-fix on save

```elisp
(add-hook 'markdown-mode-hook
  (lambda ()
    (add-hook 'before-save-hook #'lsp-format-buffer nil t)))
```

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "markdown"
language-servers = ["mkdlint-lsp"]
auto-format = true

[language-server.mkdlint-lsp]
command = "mkdlint-lsp"
```

### Zed

Add to `~/.config/zed/settings.json`:

```json
{
  "lsp": {
    "mkdlint": {
      "binary": {
        "path": "/usr/local/bin/mkdlint-lsp"
      },
      "settings": {}
    }
  },
  "languages": {
    "Markdown": {
      "language_servers": ["mkdlint"]
    }
  }
}
```

### Sublime Text

#### Using LSP Package

1. Install [LSP](https://packagecontrol.io/packages/LSP) package
2. Add to `Preferences > Package Settings > LSP > Settings`:

```json
{
  "clients": {
    "mkdlint": {
      "enabled": true,
      "command": ["mkdlint-lsp"],
      "selector": "text.html.markdown",
      "settings": {}
    }
  }
}
```

## Architecture

### Components

```
mkdlint-lsp (binary)
  ↓
MkdlintLanguageServer (backend)
  ├─ DocumentManager (in-memory cache)
  ├─ ConfigManager (config discovery & caching)
  ├─ Debouncer (300ms delay)
  └─ Client (LSP communication)
      ├─ Diagnostics (LintError → LSP Diagnostic)
      ├─ Code Actions (FixInfo → LSP TextEdit)
      └─ Utils (position/range helpers)
```

### Lifecycle

1. **Initialize**: Client sends workspace roots, server stores them
2. **Open Document**: Server caches content, lints immediately
3. **Change Document**: Debounced lint after 300ms
4. **Save Document**: Immediate lint (bypasses debounce)
5. **Close Document**: Remove from cache, clear diagnostics
6. **Shutdown**: Clean up resources

### Config Discovery

For each file:
1. Start at file's directory
2. Look for `.markdownlint.json`, `.yaml`, `.yml`, `.markdownlintrc`
3. Walk up to workspace root
4. Cache result by directory (includes negative results)
5. Apply config to lint options

## Capabilities

The mkdlint LSP server advertises these capabilities:

- **Text Document Sync**: Full document sync
- **Code Action Provider**: Provides quick-fix actions
- **Execute Command Provider**: `mkdlint.fixAll` command
- **Hover Provider**: Rule documentation on hover
- **Document Symbol Provider**: Heading outline for breadcrumbs and navigation

### Supported Methods

| Method | Description |
|--------|-------------|
| `initialize` | Initialize with workspace roots |
| `initialized` | Confirm initialization |
| `shutdown` | Clean shutdown |
| `textDocument/didOpen` | Document opened, lint immediately |
| `textDocument/didChange` | Document changed, debounced lint |
| `textDocument/didSave` | Document saved, immediate lint |
| `textDocument/didClose` | Document closed, clear diagnostics |
| `textDocument/codeAction` | Provide quick-fix actions |
| `textDocument/hover` | Show rule documentation and error details |
| `textDocument/documentSymbol` | Show headings as outline symbols |
| `workspace/executeCommand` | Execute commands (e.g., Fix All) |
| `workspace/didChangeWatchedFiles` | Reload config on file change |

### Planned Features

- [x] `textDocument/hover` - Show rule documentation
- [ ] `textDocument/formatting` - Format entire document
- [x] `textDocument/documentSymbol` - Show headings as symbols
- [x] `workspace/didChangeWatchedFiles` - Reload config on change
- [ ] `workspace/configuration` - Client-provided settings

## Configuration

### Config File Discovery

The LSP server automatically discovers config files in this order:

1. `.markdownlint.json`
2. `.markdownlint.jsonc` (JSON with comments)
3. `.markdownlint.yaml`
4. `.markdownlint.yml`
5. `.markdownlintrc`

Walks up from the file's directory to the workspace root.

### Example Config

**`.markdownlint.json`**:
```json
{
  "default": true,
  "MD013": { "line_length": 120 },
  "MD033": false,
  "MD041": false
}
```

See [Configuration Guide](USER_GUIDE.md#configuration) for full details.

## Troubleshooting

### LSP Server Not Starting

**Check binary exists:**
```bash
which mkdlint-lsp
mkdlint-lsp --version
```

**Check editor LSP logs:**
- **VS Code**: Output → mkdlint Language Server
- **Neovim**: `:LspLog`
- **Emacs**: `*lsp-log*` buffer

**Enable verbose logging:**
```bash
RUST_LOG=debug mkdlint-lsp
```

### No Diagnostics Appearing

1. **File must be saved**: Some editors require save to trigger LSP
2. **Check file extension**: Must be `.md` or `.markdown`
3. **Check for errors in config**: Invalid `.markdownlint.json` will fail silently
4. **Verify workspace root**: LSP needs a workspace root to discover config

### Code Actions Not Working

1. **Only fixable rules show actions**: Check if rule supports auto-fix
2. **Cursor must be on error line**: Position cursor on diagnostic
3. **Try "Fix All" command**: Should always be available if any fixes exist

### Performance Issues

**Increase debounce delay** (future feature):
Currently hardcoded to 300ms, will be configurable.

**Disable expensive rules**:
```json
{
  "MD013": false  // Line length checking can be slow on huge files
}
```

**Large files**:
Files > 10,000 lines may be slow. Consider splitting or excluding from linting.

### Config Not Found

**Check workspace roots:**
```
# In editor LSP logs, look for:
mkdlint LSP initialized with N workspace root(s)
```

If 0 roots, config discovery won't work properly.

**Verify config file name and location:**
```bash
# Must be in or above the file's directory
ls -la .markdownlint.json
```

## Performance

### Benchmarks

| Operation | Time | Notes |
|-----------|------|-------|
| Open document | ~10-50ms | Includes initial lint |
| Change (debounced) | ~5-20ms | After 300ms delay |
| Save | ~5-20ms | Immediate, no debounce |
| Code action request | ~1-5ms | Cache lookup |
| Config discovery | ~1ms | Cached after first lookup |

Times depend on file size and number of errors.

### Optimization Tips

1. **Let debouncing work**: Don't save after every keystroke
2. **Use config to disable unwanted rules**: Faster linting
3. **Cache hits are fast**: Config caching is very efficient
4. **Workspace roots matter**: Proper roots enable config caching

### Memory Usage

- **Per document**: ~5-10 KB (content + cached errors)
- **Config cache**: ~1-2 KB per directory
- **Total overhead**: < 1 MB for typical projects

### Scaling

Tested with:
- ✅ 100+ markdown files in workspace
- ✅ Files up to 10,000 lines
- ✅ Multiple concurrent editors

## Contributing

Want to improve the LSP server? See:
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Development guidelines
- [I-PLAN-LSP.md](../I-PLAN-LSP.md) - Implementation plan
- [GitHub Issues](https://github.com/192d-Wing/mkdlint/issues) - Current work

## License

Apache-2.0 - See [LICENSE](../LICENSE) for details.
