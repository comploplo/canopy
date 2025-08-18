# Development Commands and Conventions

## Python V1 System Commands (Reference)
Located at `/Users/gabe/projects/canopy/spacy-lsp/`

### Core Development
- `make test` - Run full test suite with coverage (ALWAYS use this, not direct pytest)
- `make test-fast` - Fast tests without formatting (use TEST=path filter)
- `make test-one TEST=tests/path/file.py::Class::method` - Run single test
- `make setup` - Set up local development environment with .venv
- `make format` - Auto-format code and fix basic issues
- `make lint` - Comprehensive linting (20 non-critical issues remain for M12)
- `make check` - Quick code quality check with error count
- `make typecheck` - Type checking (using ruff for now)

### Quality Gates
- `make presubmit` - Run pre-submit quality gates (M11+)
- `make presubmit-fix` - Run pre-submit gates with auto-fix
- Coverage requirement: 85% (temporarily lowered from 90% for M12)

### Release Management  
- `make release-patch/minor/major` - Semantic version bumping
- `make release-status` - Show current version and commits since last tag

### Performance Analysis
- `make perf-view` - View performance metrics summary
- `make perf-detailed` - View detailed performance analysis

### Editor Integration
- `make demo` - Launch Neovim demo with enhanced LSP features
- `make nvim-smoke` - Headless Neovim LSP smoke test

## Rust V2 System Commands (To Be Implemented)
Will be located at `/Users/gabe/projects/canopy/` (root level)

### Standard Rust Commands
- `cargo new canopy --lib` - Create new library project
- `cargo test` - Run test suite
- `cargo bench` - Run benchmarks
- `cargo build --release` - Release build
- `cargo clippy` - Linting
- `cargo fmt` - Formatting

### Dependencies to Add
```bash
cargo add tower-lsp tokio serde
cargo add ufal-udpipe --features bindings  # UDPipe parser
```

### Project Setup
```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install UDPipe
pip install ufal.udpipe

# Create project (in /Users/gabe/projects/canopy/)
cargo new canopy --lib
```

## macOS System Commands
- `ls` - List files
- `find` - Search files/directories  
- `grep` - Pattern matching (prefer `rg` ripgrep when available)
- `git` - Version control
- `brew install` - Package management

## Code Style Conventions
- **Python V1**: ruff formatter, line length 80, Google-style checks
- **Rust V2**: rustfmt, clippy for linting, standard Rust conventions
- **Documentation**: All public APIs documented
- **Type Safety**: Type hints throughout (Python), compile-time types (Rust)