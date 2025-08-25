# canopy.rs Development Commands
# Install just: https://github.com/casey/just

# Default recipe lists all available commands
default:
    @just --list

# === Core Development ===

# Run all tests quickly with nextest
test:
    cargo nextest run --workspace

# Run tests with output for debugging
test-verbose:
    cargo test --workspace

# Run a specific test
test-one TEST:
    cargo test --workspace {{TEST}}

# Check all code compiles
check:
    cargo check --workspace --all-targets

# Build all targets
build:
    cargo build --workspace

# Build optimized release
build-release:
    cargo build --workspace --release

# === Code Quality ===

# Format all code
fmt:
    cargo fmt --all

# Check formatting without changing files
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints (pedantic level)
lint:
    cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic

# Fix auto-fixable clippy issues
lint-fix:
    cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged

# Run all quality checks including security
check-all: fmt-check lint test audit deny
    @echo "✅ All quality checks passed!"

# === Performance & Benchmarking ===

# Run benchmarks with HTML reports
bench:
    cargo bench --workspace

# Run benchmarks for a specific crate
bench-crate CRATE:
    cargo bench --package {{CRATE}}

# Generate flamegraph for performance analysis (requires cargo-flamegraph)
flamegraph:
    cargo flamegraph --bin canopy

# Profile memory usage (requires cargo-memprof)
memprof:
    cargo run --release --bin canopy

# === Testing & Coverage ===

# Run tests with coverage report (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --workspace --out Html --output-dir coverage

# Check coverage against threshold (for CI/presubmit)
coverage-check:
    scripts/check-coverage.sh

# Check coverage with extended timeout (for slow tests)
coverage-check-long:
    timeout 900 scripts/check-coverage.sh

# Run coverage without timeout (for troubleshooting)
coverage-raw:
    cargo tarpaulin --workspace --skip-clean

# Set coverage threshold for presubmit hooks
coverage-threshold THRESHOLD:
    sed -i.bak "s/COVERAGE_THRESHOLD=.*/COVERAGE_THRESHOLD={{THRESHOLD}}/" scripts/check-coverage.sh
    @echo "✅ Coverage threshold updated to {{THRESHOLD}}%"

# Progressive coverage improvement commands
coverage-current:
    @echo "📊 Current coverage status:"
    @cargo tarpaulin --workspace --skip-clean | grep "coverage"

coverage-increase-to THRESHOLD:
    @echo "🎯 Increasing coverage threshold to {{THRESHOLD}}%"
    @echo "   Current threshold: $(grep COVERAGE_THRESHOLD scripts/check-coverage.sh | cut -d'=' -f2)"
    sed -i.bak "s/COVERAGE_THRESHOLD=.*/COVERAGE_THRESHOLD={{THRESHOLD}}/" scripts/check-coverage.sh
    @echo "✅ New threshold set. Run 'just coverage-check' to validate."

# Run property-based tests with more iterations
test-property:
    cargo test --workspace -- --ignored proptest

# Run golden/snapshot tests
test-golden:
    cargo test --workspace golden

# Update golden test snapshots
test-golden-update:
    cargo test --workspace golden -- --accept

# === Development Workflow ===

# Watch for changes and run tests
watch:
    cargo watch -c -w src -x "test --workspace --quiet"

# Watch for changes and run checks
watch-check:
    cargo watch -c -w src -x "check --workspace"

# Development mode: watch + fast feedback
dev:
    cargo watch -c -w src -w tests -x "test --workspace --quiet" -x "clippy --workspace --quiet"

# === Dependencies ===

# Update all dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated --workspace

# Audit dependencies for security issues
audit:
    cargo audit

# Check dependencies for security, licenses, and policies
deny:
    cargo deny check

# === Documentation ===

# Build documentation
docs:
    cargo doc --workspace --no-deps

# Build and open documentation
docs-open:
    cargo doc --workspace --no-deps --open

# Check documentation links
docs-check:
    cargo doc --workspace --no-deps --document-private-items

# === Installation & Setup ===

# Install development tools
install-tools:
    cargo install cargo-watch cargo-tarpaulin cargo-outdated cargo-audit cargo-deny cargo-nextest cargo-flamegraph just

# Setup development environment
setup: install-tools
    @echo "🔧 Setting up canopy development environment..."
    cargo fetch
    cargo build --workspace
    @echo "✅ Development environment ready!"

# === Benchmarking Infrastructure ===

# Initialize benchmark baseline
bench-baseline:
    cargo bench --workspace -- --save-baseline main

# Compare against baseline
bench-compare:
    cargo bench --workspace -- --baseline main

# === Clean ===

# Clean build artifacts
clean:
    cargo clean

# Clean everything including dependencies
clean-all: clean
    rm -rf target/
    rm -rf coverage/

# === Release ===

# Check if ready for release
pre-release: check-all test coverage
    @echo "🚀 Ready for release!"

# Build release binaries
release: build-release
    @echo "📦 Release binaries built in target/release/"

# === Utility ===

# Show workspace information
info:
    @echo "📊 Workspace Information"
    @echo "======================="
    cargo tree --workspace --depth 1
    @echo ""
    @echo "📈 Line counts:"
    @find crates/ -name "*.rs" -exec wc -l {} + | tail -1

# Run a quick smoke test
smoke:
    @echo "🔍 Running smoke test..."
    cargo build --workspace --quiet
    cargo test --workspace --quiet --lib
    @echo "✅ Smoke test passed!"

# === Performance Monitoring ===

# Run performance regression check
perf-check:
    @echo "📊 Performance regression check..."
    cargo bench --workspace -- --baseline main || echo "⚠️  Performance regression detected!"

# Generate performance report
perf-report:
    @echo "📈 Generating performance report..."
    @echo "Benchmark results in target/criterion/"
    @ls -la target/criterion/ 2>/dev/null || echo "No benchmark results found. Run 'just bench' first."
