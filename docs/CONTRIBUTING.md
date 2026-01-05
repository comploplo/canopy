# Contributing

## Quick Start

```bash
# Build
cargo build --release

# Test
cargo test --workspace

# Coverage (80% gate)
./scripts/check-coverage.sh

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

## Code Standards

- **Format**: `rustfmt` (Rust 2024 edition)
- **Lint**: Clippy pedantic, zero warnings
- **Naming**: snake_case functions, CamelCase types
- **Errors**: Use `Result` + `thiserror`, avoid `unwrap()` in library code
- **Logging**: Use `tracing`

## Testing

- Unit tests inline with `#[cfg(test)]`
- Integration tests in `tests/`
- Coverage gate: 80%

## Quality Gates

All must pass before PR:

- ✅ `cargo test --workspace` - all tests pass
- ✅ `cargo clippy --workspace -- -D warnings` - zero warnings
- ✅ `cargo fmt --all --check` - properly formatted
- ✅ `./scripts/check-coverage.sh` - ≥80% coverage

## Commits

Follow Conventional Commits:

```
feat(syntax): add improved POS tagging
fix(verbnet): resolve class lookup issue
refactor(pipeline): consolidate engine sharing
docs: update architecture documentation
test: add integration tests for discourse
chore: update dependencies
```

## Project Structure

```
canopy/
├── crates/
│   ├── canopy/           # Core types, kernel (events, discourse)
│   ├── canopy-resources/ # 5 semantic engines + pipeline
│   └── canopy-cli/       # CLI + demos
├── data/                 # Linguistic resources (gitignored)
├── docs/                 # Documentation
├── scripts/              # Development tools
└── tests/                # Integration tests
```

## PR Requirements

1. Clear description of changes
1. Tests for new functionality
1. All quality gates passing
1. Documentation updated if needed

## Getting Help

- **Documentation**: [ARCHITECTURE.md](ARCHITECTURE.md), [GETTING_STARTED.md](GETTING_STARTED.md)
- **Demo**: `cargo run -p canopy-resources --example demo --release`
- **API docs**: `cargo doc --open`
