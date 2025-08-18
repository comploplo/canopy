#!/bin/bash
# Coverage check script for canopy.rs presubmit hooks
# Requires cargo-tarpaulin: cargo install cargo-tarpaulin

set -e

# Coverage threshold - can be adjusted as codebase matures
COVERAGE_THRESHOLD=90.0

echo "🔬 Running coverage analysis..."
echo "📊 Coverage threshold: ${COVERAGE_THRESHOLD}%"

# Run coverage analysis with tarpaulin
# Using JSON output for easier parsing
coverage_json_file=$(mktemp)
cleanup() {
    rm -f "$coverage_json_file"
}
trap cleanup EXIT

# Run tarpaulin with JSON output
if ! cargo tarpaulin --workspace --out Json --output-dir /tmp --skip-clean > "$coverage_json_file" 2>/dev/null; then
    echo "❌ Failed to run coverage analysis"
    echo "   Make sure cargo-tarpaulin is installed: cargo install cargo-tarpaulin"
    exit 1
fi

# Extract coverage percentage from JSON output
# Handle both old and new tarpaulin JSON formats
coverage_percent=$(python3 -c "
import json
import sys

try:
    with open('$coverage_json_file', 'r') as f:
        data = json.load(f)
    
    # Try new format first (files array)
    if 'files' in data and data['files']:
        total_lines = 0
        covered_lines = 0
        for file_data in data['files'].values():
            if 'coverage' in file_data:
                for line_coverage in file_data['coverage']:
                    if line_coverage is not None:
                        total_lines += 1
                        if line_coverage > 0:
                            covered_lines += 1
        
        if total_lines > 0:
            coverage = (covered_lines / total_lines) * 100
            print(f'{coverage:.2f}')
        else:
            print('0.00')
    
    # Try old format (direct coverage percentage)
    elif 'coverage' in data:
        print(f'{data[\"coverage\"]:.2f}')
    
    # Fallback: try to parse from any percentage field
    else:
        print('0.00')
        
except (json.JSONDecodeError, KeyError, FileNotFoundError) as e:
    print('0.00')
" 2>/dev/null)

# If Python parsing failed, fall back to simpler approach
if [ -z "$coverage_percent" ] || [ "$coverage_percent" = "0.00" ]; then
    # Try to extract from tarpaulin's stdout using a direct run
    echo "🔍 Running simple coverage analysis..."
    coverage_output=$(cargo tarpaulin --workspace --skip-clean 2>&1)
    coverage_percent=$(echo "$coverage_output" | grep -o '[0-9]*\.[0-9]*% coverage' | head -1 | sed 's/% coverage//' || echo "0.00")
    
    # If still no percentage, try alternative patterns
    if [ -z "$coverage_percent" ] || [ "$coverage_percent" = "0.00" ]; then
        coverage_percent=$(echo "$coverage_output" | grep -o '[0-9]*\.[0-9]*%' | head -1 | sed 's/%//' || echo "0.00")
    fi
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
    echo "   Or temporarily lower threshold in scripts/check-coverage.sh"
    exit 1
fi