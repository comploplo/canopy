# Repository Guidelines

## Project Structure & Module Organization

- crates: Rust workspace crates (e.g., `canopy-core`, `canopy-engine`, `canopy-tokenizer`). Source lives in `crates/<crate>/src`.
- src: Minimal top-level library glue for the workspace.
- tests: Integration tests (`tests/*.rs`).
- benches: Criterion benchmarks.
- docs, examples, scripts, data: Documentation, runnable examples, CI scripts, and linguistic resources.

## Build, Test, and Development Commands

- just setup: Install dev tools and build once.
- just build / just build-release: Debug or optimized build.
- just test / just test-verbose: Run workspace tests (uses `cargo nextest` or `cargo test`).
- just check / just lint / just fmt: Type-check, clippy lints (pedantic), and formatting.
- just coverage / just coverage-check: Generate and gate coverage (tarpaulin; threshold managed in `scripts/check-coverage.sh`).
- just bench: Run Criterion benchmarks; see `benches/`.
  (Without just: use the underlying `cargo …` commands in the recipes.)

## Coding Style & Naming Conventions

- Formatting: `rustfmt` (Rust 2024 edition). Run `just fmt` or `cargo fmt --all`.
- Linting: `clippy` with `-D warnings` and pedantic in local workflows (`just lint`). Avoid `unwrap()` in library code; prefer `Result` + `thiserror`.
- Naming: snake_case for functions/modules, CamelCase for types, SCREAMING_SNAKE_CASE for consts. Keep crate names kebab-case (`canopy-tokenizer`).
- Logging: Use `tracing` and `tracing-subscriber` with env filters.

## Testing Guidelines

- Locations: Unit tests inline with `#[cfg(test)]`; integration tests in `tests/`.
- Running: `just test` (fast via nextest). Verbose/debug: `just test-verbose`.
- Property tests: `just test-property` (proptest). Snapshots/golden: `just test-golden` and update via `just test-golden-update`.
- Coverage: Current gate 50% (rebuilding test suite), M7 target 70%. Enforced by `just coverage-check`.

## Commit & Pull Request Guidelines

- Commits: Follow Conventional Commits (e.g., `feat: …`, `fix: …`, `chore: …`, `checkpoint: …`). Keep changes scoped and descriptive.
- Before PR: Run `just check-all` (fmt, clippy, tests, audit, deny, coverage). Include performance notes if benchmarks change.
- PR Content: Clear description, linked issues, reproduction steps, and example commands. For CLI changes, add sample input/output; attach benchmarks if performance-sensitive.

## Security & Configuration

- Dependencies: `just audit` and `just deny` before merging.
- Pre-commit: Install and enable hooks (`pre-commit install`).
- Data: Do not commit large or proprietary corpora; prefer pointers in `data/`.
