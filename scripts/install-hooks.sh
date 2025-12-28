#!/bin/bash
# Git hooks installer for canopy.rs
# Sets up pre-commit hooks with coverage and performance checks

set -e

echo "🔧 Installing Git hooks for canopy.rs"
echo "====================================="

# Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo "⚠️  pre-commit not found. Installing..."

    # Try different installation methods
    if command -v brew &> /dev/null; then
        echo "📦 Installing pre-commit via Homebrew..."
        brew install pre-commit
    elif command -v pip &> /dev/null; then
        echo "📦 Installing pre-commit via pip..."
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        echo "📦 Installing pre-commit via pip3..."
        pip3 install pre-commit
    else
        echo "❌ Could not install pre-commit automatically"
        echo "   Please install pre-commit manually:"
        echo "   - macOS: brew install pre-commit"
        echo "   - Python: pip install pre-commit"
        echo "   - Other: https://pre-commit.com/#installation"
        exit 1
    fi
fi

# Install pre-commit hooks
echo "🔗 Installing pre-commit hooks..."
if pre-commit install; then
    echo "✅ Pre-commit hooks installed successfully"
else
    echo "❌ Failed to install pre-commit hooks"
    exit 1
fi

# Check if cargo-llvm-cov is installed (required for coverage)
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "📊 Installing cargo-llvm-cov for coverage analysis..."
    if cargo install cargo-llvm-cov && rustup component add llvm-tools-preview; then
        echo "✅ cargo-llvm-cov installed"
    else
        echo "⚠️  Failed to install cargo-llvm-cov"
        echo "   Coverage checks may not work properly"
    fi
fi

# Check if cargo-nextest is installed (faster test runner)
if ! command -v cargo-nextest &> /dev/null; then
    echo "🧪 Installing cargo-nextest for faster testing..."
    if cargo install cargo-nextest; then
        echo "✅ cargo-nextest installed"
    else
        echo "⚠️  Failed to install cargo-nextest"
        echo "   Will fall back to standard cargo test"
    fi
fi

# Check if cargo-deny is installed (security + license + dependency policy)
if ! command -v cargo-deny &> /dev/null; then
    echo "🚫 Installing cargo-deny for dependency policy..."
    if cargo install cargo-deny; then
        echo "✅ cargo-deny installed"
    else
        echo "⚠️  Failed to install cargo-deny"
        echo "   Dependency policy checks may not work properly"
    fi
fi

# Check if bc is available (needed for coverage calculations)
if ! command -v bc &> /dev/null; then
    echo "🧮 bc (calculator) not found"
    if command -v brew &> /dev/null; then
        echo "📦 Installing bc via Homebrew..."
        brew install bc
    else
        echo "⚠️  Please install bc for coverage percentage calculations"
        echo "   - macOS: brew install bc"
        echo "   - Ubuntu: apt-get install bc"
        echo "   - CentOS: yum install bc"
    fi
fi

# Establish performance baseline
echo "📊 Establishing performance baseline..."
if ./scripts/performance-monitor.sh baseline; then
    echo "✅ Performance baseline established"
else
    echo "⚠️  Could not establish performance baseline"
    echo "   Performance regression checks may not work properly"
fi

# Test the hook installation
echo "🧪 Testing pre-commit hook installation..."
if pre-commit run --all-files >/dev/null 2>&1; then
    echo "✅ Pre-commit hooks are working correctly"
else
    echo "⚠️  Pre-commit hooks test failed"
    echo "   Some hooks may not be configured correctly"
    echo "   Run 'pre-commit run --all-files' to see specific issues"
fi

echo ""
echo "🎉 Git hooks installation complete!"
echo ""
echo "📋 Installed hooks:"
echo "   ✅ Code formatting (cargo fmt)"
echo "   ✅ Linting (cargo clippy)"
echo "   ✅ Tests (cargo nextest)"
echo "   ✅ Security + license audit (cargo deny)"
echo "   ✅ Coverage check (cargo-llvm-cov, 50% gate)"
echo "   ✅ Performance regression check"
echo "   ✅ File hygiene (trailing whitespace, merge conflicts)"
echo "   ✅ Markdown formatting (mdformat)"
echo ""
echo "💡 Usage:"
echo "   • Hooks run automatically on 'git commit'"
echo "   • Manual run: 'pre-commit run --all-files'"
echo "   • Performance monitoring: './scripts/performance-monitor.sh [check|trends|reset]'"
echo "   • Coverage analysis: './scripts/check-coverage.sh'"
echo ""
echo "🎯 Quality gates enforce:"
echo "   • No code style violations"
echo "   • No clippy warnings"
echo "   • All tests pass"
echo "   • Coverage ≥69% (baseline), ≥80% (M3), ≥90% (M4)"
echo "   • No performance regressions (<50μs latency, >95% accuracy)"
echo "   • No security vulnerabilities"
echo ""
echo "⚠️  IMPORTANT: DO NOT bypass these checks for releases!"
echo "    They exist to maintain our exceptional quality standards."
