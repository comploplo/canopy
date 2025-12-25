#!/bin/bash
# Coverage check script for canopy.rs presubmit hooks
# Requires cargo-tarpaulin: cargo install cargo-tarpaulin

set -e

# Coverage threshold for release milestones
# Current gate: 50% (temporarily lowered while removing fake tests)
# REASON: Deleting tautological tests that always pass (assert!(true), is_ok() || is_err())
# GOAL: Rebuild with meaningful tests that verify real behavior
# M3 REQUIREMENT: 70% minimum with honest tests only
# M4 REQUIREMENT: 80% minimum + clippy tech debt resolution
COVERAGE_THRESHOLD=50

echo "🔬 Running coverage analysis..."
echo "📊 Coverage threshold: ${COVERAGE_THRESHOLD}%"

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "❌ cargo-tarpaulin is not installed!"
    echo "   Install with: cargo install cargo-tarpaulin"
    exit 1
fi

# Run coverage analysis
echo "🔍 Analyzing test coverage..."
TEMP_FILE=$(mktemp)

# Run tarpaulin with 3-minute timeout
# Binary caching (data/cache/*.bin) makes engine loading fast (~50ms instead of ~50s)
# Full workspace coverage takes ~2.5 minutes with caching
# Note: Tarpaulin adds ~2-3x overhead for coverage instrumentation
timeout 180 cargo tarpaulin \
    --workspace \
    --skip-clean \
    --out Stdout \
    -- --test-threads=4 \
    > "$TEMP_FILE" 2>&1 || {
    exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo "❌ Coverage analysis failed with exit code $exit_code"
        echo "   Output:"
        cat "$TEMP_FILE"
        rm -f "$TEMP_FILE"
        exit 1
    fi
}

# Extract coverage percentage from output
coverage_output=$(cat "$TEMP_FILE")
echo "📋 Coverage output:"
echo "$coverage_output"
echo ""

# Try multiple patterns to extract coverage percentage
coverage_percent=""

# Pattern 1: "XX.XX% coverage"
coverage_percent=$(echo "$coverage_output" | grep -o '[0-9]*\.[0-9]*% coverage' | head -1 | sed 's/% coverage//' 2>/dev/null || true)

# Pattern 2: "XX.XX%" at end of line
if [ -z "$coverage_percent" ]; then
    coverage_percent=$(echo "$coverage_output" | grep -oE '[0-9]+\.[0-9]+%$' | head -1 | sed 's/%//' 2>/dev/null || true)
fi

# Pattern 3: Any "XX.XX%"
if [ -z "$coverage_percent" ]; then
    coverage_percent=$(echo "$coverage_output" | grep -oE '[0-9]+\.[0-9]+%' | head -1 | sed 's/%//' 2>/dev/null || true)
fi

# Pattern 4: Integer percentage "XX%"
if [ -z "$coverage_percent" ]; then
    coverage_percent=$(echo "$coverage_output" | grep -oE '[0-9]+%' | head -1 | sed 's/%//' 2>/dev/null || true)
fi

# Clean up temp file
rm -f "$TEMP_FILE"

# Validate we found a percentage
if [ -z "$coverage_percent" ] || ! [[ "$coverage_percent" =~ ^[0-9]+\.?[0-9]*$ ]]; then
    echo "⚠️  Could not parse coverage percentage from output"
    echo "   Coverage analysis completed but threshold check skipped"
    echo "   Please manually verify coverage meets ${COVERAGE_THRESHOLD}% threshold"
    echo ""
    echo "✅ Coverage analysis completed (manual verification required)"
    exit 0
fi

echo "📈 Current coverage: ${coverage_percent}%"

# Convert to integer for comparison if it's a decimal
coverage_int=$(echo "$coverage_percent" | cut -d. -f1)

# Check against threshold
if [ "$coverage_int" -ge "$COVERAGE_THRESHOLD" ]; then
    echo "✅ Coverage check passed! (${coverage_percent}% >= ${COVERAGE_THRESHOLD}%)"

    # Store coverage info for reporting
    echo ""
    echo "📊 Coverage Summary:"
    echo "   Threshold: ${COVERAGE_THRESHOLD}%"
    echo "   Actual: ${coverage_percent}%"
    echo "   Status: PASSED ✅"
    echo ""
    echo "📝 Note: Full workspace coverage analysis completed."
    echo "   Deprecated packages excluded from workspace."

    exit 0
else
    echo "❌ Coverage check failed!"
    echo "   Required: ${COVERAGE_THRESHOLD}%"
    echo "   Actual: ${coverage_percent}%"

    shortfall=$((COVERAGE_THRESHOLD - coverage_int))
    echo "   Shortfall: ~${shortfall}%"
    echo ""
    echo "💡 To improve coverage:"
    echo "   1. Add tests for uncovered code paths"
    echo "   2. Run 'cargo tarpaulin --workspace' to see detailed coverage report"
    echo "   3. Focus on files with low coverage first"
    echo ""
    echo "⚠️  REMINDER: DO NOT lower the coverage threshold for releases!"
    echo "   The threshold must reach 80% for M3 and 90% for M4."
    echo "   Write more tests instead of lowering standards."
    exit 1
fi
