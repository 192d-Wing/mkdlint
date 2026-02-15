#!/bin/bash
# Build and test script for markdownlint Rust rewrite

set -e  # Exit on error

echo "=========================================="
echo "Markdownlint Rust Build & Test"
echo "=========================================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo ""
    echo "Install Rust with:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  source \$HOME/.cargo/env"
    echo ""
    exit 1
fi

echo "✅ Rust version: $(rustc --version)"
echo ""

# Build the project
echo "🔨 Building project..."
cargo build

echo ""
echo "✅ Build successful!"
echo ""

# Run tests
echo "🧪 Running tests..."
cargo test

echo ""
echo "✅ All tests passed!"
echo ""

# Run on test file
echo "🔍 Testing on test_sample.md..."
echo ""
cargo run -- test_sample.md || true

echo ""
echo "=========================================="
echo "Build Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  - Implement more rules (currently 5/54)"
echo "  - Compare output with Node.js version"
echo "  - Add integration tests"
echo ""
