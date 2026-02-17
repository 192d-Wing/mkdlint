# mkdlint GitHub Action

Fast Markdown linting with auto-fix and SARIF Code Scanning support.

## Features

- ⚡ **Fast binary caching** - 10-100x faster than building from source
- 🔧 **Auto-fix** - Automatically fix 48/53 rules (90.6% coverage)
- 📊 **SARIF Support** - Native GitHub Code Scanning integration
- 📋 **Job Summary** - Rich markdown summary with stats and top violated rules
- 🔄 **Incremental Linting** - Only lint changed files in pull requests
- ⏱️ **Performance Metrics** - Timing and file counts in outputs
- 🎯 **Zero config** - Works out of the box
- 📝 **Rich output** - Text, JSON, or SARIF formats

## Quick Start

```yaml
- uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
  with:
    files: '.'
```

That's it! This will:

1. Lint all Markdown files in your repository
2. Upload results to GitHub Code Scanning
3. Write a rich job summary with stats
4. Fail the workflow if errors are found

## Usage Examples

### Basic Linting

```yaml
name: Lint Markdown
on: [push, pull_request]

jobs:
  markdown:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
        with:
          files: '.'
```

### Incremental Linting (PRs Only)

```yaml
name: Lint Changed Markdown
on: pull_request

jobs:
  markdown:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
        with:
          changed-only: true
```

### Auto-Fix and Commit

```yaml
name: Auto-Fix Markdown
on: [push]

jobs:
  fix:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
        with:
          files: '.'
          fix: true
          fail-on-error: false

      - uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: 'docs: auto-fix markdown issues'
```

### Custom Configuration

```yaml
- uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
  with:
    files: 'docs/ README.md'
    config: '.markdownlint.json'
    ignore: '**/node_modules/**,vendor/**'
    disable: 'MD013,MD033'
```

### Use Outputs

```yaml
- id: lint
  uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
  with:
    fail-on-error: false

- name: Report
  run: |
    echo "Errors: ${{ steps.lint.outputs.error-count }}"
    echo "Warnings: ${{ steps.lint.outputs.warning-count }}"
    echo "Duration: ${{ steps.lint.outputs.duration-ms }}ms"
```

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `files` | Files or directories to lint | `.` |
| `version` | mkdlint version (`latest` or specific like `0.9.1`) | `latest` |
| `use-binary` | Use pre-built binary (much faster) | `true` |
| `config` | Path to configuration file | `` |
| `output-format` | Output format: `text`, `json`, or `sarif` | `sarif` |
| `sarif-file` | Path to SARIF output file | `mkdlint.sarif` |
| `fix` | Auto-fix violations | `false` |
| `ignore` | Comma-separated glob patterns to ignore | `` |
| `enable` | Comma-separated rules to enable | `` |
| `disable` | Comma-separated rules to disable | `` |
| `no-color` | Disable colored output | `false` |
| `verbose` | Verbose output | `false` |
| `quiet` | Quiet mode | `false` |
| `fail-on-error` | Fail if errors found | `true` |
| `upload-sarif` | Upload to Code Scanning | `true` |
| `working-directory` | Working directory | `.` |
| `changed-only` | Only lint changed `.md` files in PRs | `false` |
| `job-summary` | Write rich summary to job summary | `true` |

## Outputs

| Output | Description |
|--------|-------------|
| `exit-code` | Exit code from mkdlint |
| `error-count` | Number of errors found |
| `warning-count` | Number of warnings found |
| `file-count` | Number of files with issues |
| `duration-ms` | Linting duration in milliseconds |
| `sarif-file` | Path to SARIF file |
| `binary-path` | Path to mkdlint binary |
| `cache-hit` | Whether binary was cached |

## Job Summary

When `job-summary: true` (default), the action writes a rich markdown summary to the GitHub Actions job summary page, including:

- Error and warning counts
- Number of files with issues
- Linting duration
- Top 5 most violated rules (when using SARIF output)

## Incremental Linting

When `changed-only: true`, the action detects changed `.md` and `.markdown` files in pull requests using `git diff` against the base branch. This is useful for large repositories where linting all files is slow.

Requirements:

- Only works on `pull_request` events (falls back to normal linting on `push`)
- Requires `fetch-depth: 0` in the checkout step for accurate diff detection

## Performance

**Binary caching makes this action 10-100x faster than alternatives:**

- With cache hit: ~1-2 seconds
- Without cache (first run): ~5-10 seconds
- Building from source: ~60-90 seconds

Cache is keyed by OS, architecture, and version, so it persists across runs.

## Permissions

Minimal permissions required:

```yaml
permissions:
  contents: read           # To checkout code
  security-events: write   # To upload SARIF (if using Code Scanning)
```

## Troubleshooting

### SARIF upload fails

Ensure you have the correct permissions:

```yaml
permissions:
  security-events: write
```

### Binary download fails

The action automatically falls back to building from source. To force source build:

```yaml
- uses: 192d-Wing/mkdlint/.github/actions/mkdlint@main
  with:
    use-binary: false
```

### Errors in node_modules

Use the `ignore` input:

```yaml
with:
  ignore: '**/node_modules/**,**/vendor/**'
```

## Comparison with Alternatives

| Feature | mkdlint | markdownlint-cli | markdownlint-cli2 |
|---------|---------|------------------|-------------------|
| Speed | ⚡ Rust (parallel) | Node.js | Node.js |
| Auto-fix | 90.6% (48/53) | Limited | Limited |
| SARIF | ✅ Native | ❌ | ✅ Via plugin |
| Binary caching | ✅ Yes | ❌ | ❌ |
| Job summary | ✅ Yes | ❌ | ❌ |
| Incremental lint | ✅ Yes | ❌ | ❌ |
| Watch mode | ✅ Yes | ❌ | ❌ |
| Config wizard | ✅ Yes | ❌ | ❌ |

## License

Apache-2.0

## Links

- [mkdlint Repository](https://github.com/192d-Wing/mkdlint)
- [Documentation](https://github.com/192d-Wing/mkdlint/blob/main/docs/USER_GUIDE.md)
- [Rules Reference](https://github.com/DavidAnson/markdownlint/tree/main/doc)
- [Report Issues](https://github.com/192d-Wing/mkdlint/issues)
