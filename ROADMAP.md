# mkdlint Roadmap

## Current State (v0.6.1)

### ✅ Completed Features

- **80% Auto-Fix Coverage** - 43 out of 54 rules automatically fixable
- **Comprehensive Documentation** - User guide, README, FAQ
- **Helpful Suggestions** - All 54 rules provide actionable guidance
- **Multiple Output Formats** - Text (colored), JSON, SARIF
- **Configuration System** - JSON, YAML, TOML with auto-discovery
- **CLI Tool** - Full-featured command-line interface
- **Library API** - Use as a Rust crate
- **Performance** - Parallel processing, optimized rules
- **GitHub Actions** - Native CI/CD integration

### 📊 Statistics

- **Rules**: 54 total (MD001-MD060, excluding 7 deprecated)
- **Auto-Fix**: 43 rules (80% coverage)
- **Tests**: 346 passing
- **Code Quality**: Zero clippy warnings

## Short-Term Roadmap (v0.7.x)

### v0.7.0 - Developer Experience
**Est. Time**: 2-3 weeks

#### Watch Mode
- `mkdlint --watch` for auto-lint on file changes
- Debounced file system notifications (300ms)
- Optional auto-fix: `mkdlint --watch --fix`
- Filter specific paths: `--watch-paths`

**Dependencies needed**:
- `notify = "6.0"` for file watching
- Debouncing logic with tokio

#### Interactive Configuration Wizard
- `mkdlint init --interactive` for guided setup
- Questions about:
  - Format preference (JSON/YAML/TOML)
  - Line length limits
  - Heading style preferences
  - Common rule customizations
- Generate optimal config based on answers

**Implementation**:
- Use `dialoguer` crate for CLI prompts
- Template-based config generation
- Validation and preview

#### IDE Integration Improvements
- **VS Code Extension** (separate repo)
  - Real-time linting with LSP
  - Quick-fix code actions
  - Configuration UI
- **Neovim Plugin** (separate repo)
  - Native LSP integration
  - Telescope integration
  - Status line integration

### v0.7.1 - Quality & Testing
**Est. Time**: 1-2 weeks

#### Fuzzing
- Add `cargo-fuzz` targets
- Fuzz `lint()` function
- Fuzz configuration parsing
- Fuzz `apply_fixes()` logic
- Run fuzzing for extended periods

#### Expanded Benchmarks
- Large file benchmarks (10MB+ markdown)
- Many errors scenarios (1000+ errors)
- Complex tables and nested lists
- Concurrent file processing
- Comparison with markdownlint-cli

#### End-to-End Tests
- Full workflow tests (lint → fix → verify)
- Configuration override precedence
- SARIF output validation
- Error recovery and edge cases
- Multi-file project tests

## Medium-Term Roadmap (v0.8.x-v0.9.x)

### v0.8.0 - Advanced Auto-Fixes
**Est. Time**: 3-4 weeks

Goal: Push auto-fix coverage to 85%+ (46/54 rules)

#### New Auto-Fixes
1. **MD046** - Code block style conversion
   - Convert between indented and fenced
   - Preserve language hints
   - Handle nested structures

2. **MD024** - Duplicate heading disambiguation
   - Append numbers: `## Title` → `## Title (2)`
   - Or convert to lower level
   - User-configurable strategy

3. **MD013** - Intelligent line wrapping (hard!)
   - Respect sentence boundaries
   - Preserve inline code and links
   - Configurable break points

### v0.9.0 - Custom Rules API
**Est. Time**: 4-6 weeks

#### Plugin System
```rust
pub trait CustomRule {
    fn names(&self) -> &[&'static str];
    fn lint(&self, params: &RuleParams) -> Vec<LintError>;
}
```

#### WASM Support
- Load custom rules from WebAssembly
- Sandbox execution for security
- Example custom rules

#### Rule Development Kit
- Template generator
- Testing utilities
- Documentation generator

## Long-Term Roadmap (v1.0+)

### v1.0.0 - Production Ready
**Est. Time**: 6-8 weeks

#### Stability Goals
- 100% test coverage for auto-fixes
- No panics in production use
- Comprehensive error handling
- Stable public API (semver guarantees)

#### Performance Goals
- Sub-second linting for 1000+ files
- Memory usage optimizations
- Incremental linting (only changed files)

#### Documentation Goals
- Video tutorials
- Interactive examples
- Architecture documentation
- Contributing guide improvements

### v1.1.0 - Enterprise Features

#### Advanced Configuration
- Configuration profiles (strict, relaxed, custom)
- Project-wide rule exemptions
- Per-file rule overrides
- Rule severity levels

#### Reporting
- HTML reports with visualizations
- Trend analysis over time
- Team-wide metrics
- Integration with quality dashboards

#### Integrations
- GitHub App for PR comments
- GitLab integration
- Bitbucket support
- Slack/Discord notifications

### v1.2.0 - Ecosystem Growth

#### Language Server Enhancements
- Workspace diagnostics
- Configuration from editor settings
- "Fix all" command
- Rule documentation on hover
- Inline config suggestions

#### Editor Extensions
- IntelliJ IDEA plugin
- Sublime Text package
- Atom package
- Zed integration

#### Build Tool Plugins
- Cargo plugin
- Make integration
- npm scripts helper
- Pre-commit framework

## Future Possibilities (v2.0+)

### AI-Powered Features
- Intelligent fix suggestions
- Context-aware error messages
- Auto-generate missing content
- Style consistency learning

### Multi-Format Support
- AsciiDoc linting
- reStructuredText support
- Org-mode files
- MDX (Markdown + JSX)

### Cloud Features
- Shared team configurations
- Remote rule updates
- Centralized reporting
- API for integrations

## Contributing

Want to help with any of these features?

1. Check [open issues](https://github.com/192d-Wing/mkdlint/issues)
2. Discuss in [discussions](https://github.com/192d-Wing/mkdlint/discussions)
3. Read [CONTRIBUTING.md](CONTRIBUTING.md)
4. Submit PRs!

## Release Schedule

- **Patch releases** (0.6.x): Bug fixes, documentation - as needed
- **Minor releases** (0.x.0): New features, improvements - monthly
- **Major releases** (x.0.0): Breaking changes, major milestones - quarterly

## Feedback

Your input shapes the roadmap! Please:
- 🐛 [Report bugs](https://github.com/192d-Wing/mkdlint/issues/new?template=bug_report.md)
- 💡 [Suggest features](https://github.com/192d-Wing/mkdlint/issues/new?template=feature_request.md)
- 💬 [Join discussions](https://github.com/192d-Wing/mkdlint/discussions)

---

Last updated: 2026-02-16
