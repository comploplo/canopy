#!/bin/bash
# Setup script for canopy.rs development environment

set -e

echo "🔧 Setting up canopy.rs development environment..."

# Setup pre-commit hooks
echo "🪝 Installing pre-commit hooks..."
if command -v pre-commit &> /dev/null; then
    pre-commit install
    echo "✅ Pre-commit hooks installed!"
else
    echo "❌ pre-commit not found. Please install with: brew install pre-commit"
    exit 1
fi

# Install additional development tools if not present
echo "🛠️ Checking development tools..."

tools_to_install=()

if ! command -v cargo-watch &> /dev/null; then
    tools_to_install+=("cargo-watch")
fi

if ! command -v cargo-tarpaulin &> /dev/null; then
    tools_to_install+=("cargo-tarpaulin")
fi

if ! command -v cargo-insta &> /dev/null; then
    tools_to_install+=("cargo-insta")
fi

if ! command -v just &> /dev/null; then
    echo "⚠️  'just' command runner not found. Please install via:"
    echo "   brew install just  (macOS)"
    echo "   or visit: https://github.com/casey/just"
fi

if [ ${#tools_to_install[@]} -gt 0 ]; then
    echo "📦 Installing additional tools: ${tools_to_install[*]}"
    cargo install "${tools_to_install[@]}"
fi

# Run initial quality checks
echo "🧪 Running initial quality checks..."
cargo fmt --check || (echo "⚠️  Code needs formatting. Run: just fmt" && exit 1)
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic || (echo "⚠️  Clippy issues found. Run: just lint" && exit 1)
cargo test --workspace --quiet || (echo "⚠️  Tests failing. Run: just test" && exit 1)

echo ""
echo "✅ Development environment setup complete!"
echo ""
echo "📋 Available commands:"
echo "   just --list     # Show all available commands"
echo "   just dev        # Start development mode"
echo "   just test       # Run tests"
echo "   just bench      # Run benchmarks"
echo "   just check-all  # Run all quality checks"
echo ""
echo "🚀 Ready for development!"
