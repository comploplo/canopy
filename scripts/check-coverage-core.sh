#!/bin/bash
# Focused coverage check for core packages only
# Uses cargo-llvm-cov for LLVM instrumentation-based coverage

set -e

COVERAGE_THRESHOLD=40  # Lower threshold for core packages only

echo "🔬 Running focused coverage analysis on core packages..."
echo "📊 Coverage threshold: ${COVERAGE_THRESHOLD}%"

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "❌ cargo-llvm-cov is not installed!"
    echo "   Install with: cargo install cargo-llvm-cov"
    echo "            or: brew install cargo-llvm-cov"
    exit 1
fi

# Run coverage on core packages
echo "🔍 Analyzing core packages: canopy-core, canopy-engine..."

if cargo llvm-cov \
    --package canopy-core \
    --package canopy-engine \
    --fail-under-lines "${COVERAGE_THRESHOLD}" 2>&1; then

    echo ""
    echo "✅ Core coverage check passed! (>= ${COVERAGE_THRESHOLD}%)"
    echo ""
    echo "📊 Core Coverage Summary:"
    echo "   Threshold: ${COVERAGE_THRESHOLD}%"
    echo "   Status: PASSED ✅"
    echo ""
    echo "Note: This is focused coverage on core packages only."
    echo "For full workspace coverage: ./scripts/check-coverage.sh"
    exit 0
else
    exit_code=$?
    echo ""
    echo "❌ Core coverage check failed!"
    echo "   Required: ${COVERAGE_THRESHOLD}%"
    exit $exit_code
fi
