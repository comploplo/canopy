#!/bin/bash
# Coverage check script for canopy.rs presubmit hooks
# Uses cargo-llvm-cov for LLVM instrumentation-based coverage
#
# Install: cargo install cargo-llvm-cov
#      or: brew install cargo-llvm-cov

set -e

# Coverage threshold for release milestones
# Current gate: 50% (temporarily lowered while removing fake tests)
# REASON: Deleting tautological tests that always pass (assert!(true), is_ok() || is_err())
# GOAL: Rebuild with meaningful tests that verify real behavior
# M3 REQUIREMENT: 70% minimum with honest tests only
# M4 REQUIREMENT: 80% minimum + clippy tech debt resolution
COVERAGE_THRESHOLD=50

echo "🔬 Running coverage analysis with cargo-llvm-cov..."
echo "📊 Coverage threshold: ${COVERAGE_THRESHOLD}%"

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "❌ cargo-llvm-cov is not installed!"
    echo "   Install with: cargo install cargo-llvm-cov"
    echo "            or: brew install cargo-llvm-cov"
    exit 1
fi

# Run coverage with threshold check
# --fail-under-lines will exit non-zero if coverage is below threshold
echo "🔍 Analyzing test coverage..."

# Run llvm-cov with workspace coverage
# Uses LLVM instrumentation which is faster and more accurate than tarpaulin
if cargo llvm-cov \
    --workspace \
    --fail-under-lines "${COVERAGE_THRESHOLD}" \
    --ignore-filename-regex 'tests?\.rs$|/tests/|/benches/' 2>&1; then

    echo ""
    echo "✅ Coverage check passed! (>= ${COVERAGE_THRESHOLD}%)"
    echo ""
    echo "📊 Coverage Summary:"
    echo "   Threshold: ${COVERAGE_THRESHOLD}%"
    echo "   Status: PASSED ✅"
    echo ""
    echo "💡 For detailed HTML report: cargo llvm-cov --html --open"
    exit 0
else
    exit_code=$?
    echo ""
    echo "❌ Coverage check failed!"
    echo "   Required: ${COVERAGE_THRESHOLD}%"
    echo ""
    echo "💡 To improve coverage:"
    echo "   1. Run 'cargo llvm-cov --html --open' to see detailed report"
    echo "   2. Add tests for uncovered code paths"
    echo "   3. Focus on files with low coverage first"
    echo ""
    echo "⚠️  REMINDER: DO NOT lower the coverage threshold for releases!"
    echo "   The threshold must reach 70% for M3 and 80% for M4."
    echo "   Write more tests instead of lowering standards."
    exit $exit_code
fi
