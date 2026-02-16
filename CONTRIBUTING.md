# Contributing to mkdlint

Thank you for your interest in contributing to mkdlint! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Rust 1.93 or later
- Git

### Setting Up Your Development Environment

1. Fork the repository on GitHub
2. Clone your fork:

   ```bash
   git clone https://github.com/YOUR_USERNAME/mkdlint.git
   cd mkdlint
   ```

3. Add the upstream repository:

   ```bash
   git remote add upstream https://github.com/192d-Wing/mkdlint.git
   ```

4. Install dependencies and run tests:

   ```bash
   cargo test
   ```

## Development Workflow

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run snapshot tests
cargo test --test snapshot_tests

# Run E2E tests
cargo test --test e2e_tests

# Run benchmarks
cargo bench
```

### Code Formatting and Linting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets --all-features

# Fix clippy warnings automatically
cargo clippy --fix
```

### Adding a New Rule

1. Create a new file in `src/rules/` (e.g., `md999.rs`)
2. Implement the `Rule` trait
3. Add tests in the same file under `#[cfg(test)]`
4. Register the rule in `src/rules/mod.rs`
5. Add documentation to the rule's doc comment
6. Update `README.md` rules table
7. Add entry to `CHANGELOG.md`

### Adding Auto-Fix Support

1. Add `FixInfo` to error creation in your rule
2. Provide:
   - `line_number`: Line to modify
   - `edit_column`: Column to start edit (1-based)
   - `delete_count`: Characters to delete (or None)
   - `insert_text`: Text to insert
3. Add tests for fix_info
4. Mark rule as fixable in README

### Running Your Changes Locally

```bash
# Build and run
cargo run -- path/to/test.md

# Run with auto-fix
cargo run -- --fix path/to/test.md

# Run with specific config
cargo run -- --config .markdownlint.json path/to/test.md
```

## Pull Request Process

1. **Create a branch** for your changes:

   ```bash
   git checkout -b feature/my-new-feature
   ```

2. **Make your changes** following the code style and conventions

3. **Add tests** for your changes

4. **Update documentation** (README, CHANGELOG, doc comments)

5. **Run tests and linting**:

   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

6. **Commit your changes** with clear, descriptive messages:

   ```bash
   git commit -m "feat: add support for XYZ"
   ```

   Follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `test:` for tests
   - `refactor:` for refactoring
   - `perf:` for performance improvements

7. **Push to your fork**:

   ```bash
   git push origin feature/my-new-feature
   ```

8. **Create a Pull Request** on GitHub

### Pull Request Requirements

- All tests must pass
- Code must be formatted with `cargo fmt`
- No clippy warnings
- Documentation updated if needed
- CHANGELOG.md updated in the `[Unreleased]` section

## Code Style Guidelines

- Use `cargo fmt` for formatting (enforced by CI)
- Follow Rust naming conventions
- Write doc comments for public APIs
- Keep functions focused and reasonably sized
- Prefer explicit over implicit
- Add tests for all new functionality

## Testing Guidelines

- **Unit tests**: Test individual functions and rule logic
- **Snapshot tests**: Use `insta` for regression testing
- **E2E tests**: Test CLI behavior end-to-end
- **Benchmarks**: Add benchmarks for performance-critical code

### Writing Good Tests

```rust
#[test]
fn test_rule_name_specific_case() {
    // Arrange: Set up test data
    let lines = vec!["test\n".to_string()];
    
    // Act: Run the code
    let errors = rule.lint(&params);
    
    // Assert: Check results
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].line_number, 1);
}
```

## Performance Considerations

- Profile before optimizing
- Use byte-level operations for text parsing when appropriate
- Avoid unnecessary allocations
- Consider caching for expensive operations
- Run benchmarks to measure impact

## Documentation

- All public APIs must have doc comments
- Use examples in doc comments when helpful
- Keep README.md up to date
- Update CHANGELOG.md for all user-facing changes

## Getting Help

- Open an issue for questions
- Check existing issues and PRs
- Read the codebase for examples

## License

By contributing to mkdlint, you agree that your contributions will be licensed under the Apache-2.0 license.
