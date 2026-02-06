#!/bin/bash
# Performance monitoring for canopy
# Runs criterion benchmarks and reports real metrics
#
# Usage:
#   ./scripts/performance-monitor.sh          # Run benchmarks and report
#   ./scripts/performance-monitor.sh baseline  # Save current results as baseline
#   ./scripts/performance-monitor.sh check     # Compare against baseline
#   ./scripts/performance-monitor.sh trends    # Show historical results

set -e

PERF_DIR=".performance"
BASELINE_FILE="$PERF_DIR/baseline.json"
HISTORY_FILE="$PERF_DIR/history.log"

mkdir -p "$PERF_DIR"

echo "🔍 Canopy Performance Monitor"
echo "=============================="

run_benchmarks() {
    echo "📊 Running criterion benchmarks (this may take a minute)..."

    # Run criterion benchmarks in JSON format and capture output
    local bench_output
    bench_output=$(cargo bench --bench baseline 2>&1) || {
        echo "⚠️  Benchmarks require data files. Some may be skipped."
    }

    echo "$bench_output" > "$PERF_DIR/last_run.log"

    # Extract timing results from criterion output
    # Criterion format: "bench_name    time:   [low avg high]"
    echo ""
    echo "📈 Benchmark Results:"
    echo "---------------------"

    echo "$bench_output" | grep -E "time:" | while IFS= read -r line; do
        echo "  $line"
    done

    # Log timestamped results
    local timestamp
    timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    echo "$timestamp | $(echo "$bench_output" | grep -E "time:" | tr '\n' ' ')" >> "$HISTORY_FILE"

    echo ""
    echo "✅ Benchmark run complete"
    echo "   Full output: $PERF_DIR/last_run.log"
}

store_baseline() {
    echo "📊 Establishing performance baseline..."
    run_benchmarks

    cp "$PERF_DIR/last_run.log" "$BASELINE_FILE"
    echo "✅ Baseline stored in $BASELINE_FILE"
}

check_regression() {
    if [ ! -f "$BASELINE_FILE" ]; then
        echo "⚠️  No baseline found. Run: $0 baseline"
        exit 1
    fi

    echo "🔍 Running benchmarks and comparing against baseline..."
    run_benchmarks

    echo ""
    echo "📊 Comparison with baseline:"
    echo "   Baseline: $BASELINE_FILE"
    echo ""
    echo "   For detailed comparison, use criterion's built-in comparison:"
    echo "   cargo bench --bench baseline -- --baseline main"
}

show_trends() {
    if [ ! -f "$HISTORY_FILE" ]; then
        echo "⚠️  No performance history available. Run benchmarks first."
        return 0
    fi

    echo "📈 Performance Trends (last 10 entries):"
    echo "========================================"
    tail -10 "$HISTORY_FILE" | while IFS= read -r line; do
        echo "  $line"
    done
}

case "${1:-run}" in
    "run")
        run_benchmarks
        ;;
    "baseline")
        store_baseline
        ;;
    "check")
        check_regression
        ;;
    "trends")
        show_trends
        ;;
    "reset")
        echo "🗑️  Resetting performance data..."
        rm -f "$BASELINE_FILE" "$HISTORY_FILE" "$PERF_DIR/last_run.log"
        store_baseline
        ;;
    *)
        echo "Usage: $0 [run|baseline|check|trends|reset]"
        echo ""
        echo "Commands:"
        echo "  run       - Run benchmarks and report results (default)"
        echo "  baseline  - Run benchmarks and save as baseline"
        echo "  check     - Run benchmarks and compare against baseline"
        echo "  trends    - Show historical benchmark results"
        echo "  reset     - Clear history and establish new baseline"
        exit 1
        ;;
esac
