# Repository Guidelines

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

## Build & Test

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

## Coding Style

- **Format**: `rustfmt` (Rust 2024 edition)
- **Lint**: Clippy pedantic, zero warnings
- **Naming**: snake_case functions, CamelCase types, kebab-case crates
- **Errors**: Use `Result` + `thiserror`, avoid `unwrap()` in library code
- **Logging**: Use `tracing`

## Testing

- Unit tests inline with `#[cfg(test)]`
- Integration tests in `tests/`
- Coverage gate: 80%

## Commits

Follow Conventional Commits:

- `feat:` new feature
- `fix:` bug fix
- `refactor:` code restructure
- `docs:` documentation
- `test:` tests
- `chore:` maintenance

## Before PR

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
./scripts/check-coverage.sh
```
