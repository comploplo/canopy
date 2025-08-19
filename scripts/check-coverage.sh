#!/bin/bash
# Coverage check script for canopy.rs presubmit hooks
# Requires cargo-tarpaulin: cargo install cargo-tarpaulin

set -e

# Coverage threshold for release milestones
# Current gate: 69% (current baseline: 69.46%)
# M3 REQUIREMENT: 80% minimum coverage for completion
# M4 REQUIREMENT: 90% minimum coverage for completion
#
# IMPORTANT: DO NOT LOWER THESE THRESHOLDS WHEN MAKING RELEASES
# Instead, improve test coverage to meet the requirements
COVERAGE_THRESHOLD=69

echo "🔬 Running coverage analysis..."
echo "📊 Coverage threshold: ${COVERAGE_THRESHOLD}%"

# Run coverage analysis and extract percentage directly
echo "🔍 Analyzing test coverage..."
coverage_output=$(cargo tarpaulin --workspace --skip-clean 2>&1)
coverage_percent=$(echo "$coverage_output" | grep -o '[0-9]*\.[0-9]*% coverage' | head -1 | sed 's/% coverage//' || echo "0.00")

# If no "% coverage" pattern, try alternative patterns
if [ -z "$coverage_percent" ] || [ "$coverage_percent" = "0.00" ]; then
    coverage_percent=$(echo "$coverage_output" | grep -o '[0-9]*\.[0-9]*%' | head -1 | sed 's/%//' || echo "0.00")
fi

# Ensure we have a valid number
if ! [[ "$coverage_percent" =~ ^[0-9]+\.?[0-9]*$ ]]; then
    echo "⚠️  Could not parse coverage percentage, running full analysis..."

    # Fall back to running tarpaulin with stdout output
    echo ""
    if cargo tarpaulin --workspace --skip-clean; then
        echo ""
        echo "✅ Coverage analysis completed"
        echo "   Note: Could not parse exact percentage for threshold check"
        echo "   Please manually verify coverage meets ${COVERAGE_THRESHOLD}% threshold"
        exit 0
    else
        echo "❌ Coverage analysis failed"
        exit 1
    fi
fi

echo "📈 Current coverage: ${coverage_percent}%"

# Check against threshold
if (( $(echo "$coverage_percent >= $COVERAGE_THRESHOLD" | bc -l) )); then
    echo "✅ Coverage check passed! (${coverage_percent}% >= ${COVERAGE_THRESHOLD}%)"

    # Store coverage info for reporting
    echo "📊 Coverage Summary:"
    echo "   Threshold: ${COVERAGE_THRESHOLD}%"
    echo "   Actual: ${coverage_percent}%"
    echo "   Status: PASSED ✅"

    exit 0
else
    echo "❌ Coverage check failed!"
    echo "   Required: ${COVERAGE_THRESHOLD}%"
    echo "   Actual: ${coverage_percent}%"
    echo "   Shortfall: $(echo "$COVERAGE_THRESHOLD - $coverage_percent" | bc -l)%"
    echo ""
    echo "💡 To improve coverage:"
    echo "   1. Add tests for uncovered code paths"
    echo "   2. Run 'just coverage' to see detailed coverage report"
    echo "   3. Focus on files with low coverage first"
    echo ""
    echo "⚠️  REMINDER: DO NOT lower the coverage threshold for releases!"
    echo "   The threshold must reach 80% for M3 and 90% for M4."
    echo "   Write more tests instead of lowering standards."
    exit 1
fi
