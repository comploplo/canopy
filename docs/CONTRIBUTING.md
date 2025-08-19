# Contributing to canopy.rs

## Philosophy

canopy.rs follows **infrastructure-first development** with rigorous performance
monitoring and quality gates. We prioritize:

1. **Performance-First Mindset**: Establish baselines before building features
2. **Type Safety**: Use Rust's type system to enforce linguistic constraints
3. **Theory-Driven Design**: Every architectural decision grounded in linguistic
   theory
4. **Comprehensive Testing**: Property-based tests, golden tests, benchmarks
5. **Developer Experience**: Fast feedback loops and excellent tooling

## Development Workflow

### Quick Start

```bash
# Setup development environment
just setup

# Start development mode (watch + fast feedback)
just dev

# Run full quality checks
just check-all
```

### Core Commands

```bash
just test         # Run all tests
just bench        # Run benchmarks with regression detection
just lint         # Run clippy lints (pedantic level)
just fmt          # Format all code
just smoke        # Quick verification everything works
```

## Code Standards

### Rust-Specific Standards

- **Formatting**: Use `rustfmt` defaults (run `just fmt`)
- **Linting**: Pass `clippy::pedantic` level (run `just lint`)
- **Documentation**: `///` doc comments for all public items
- **Error Handling**: Use `Result<T, E>` pattern, avoid panics in library code
- **Performance**: Zero-copy where possible, avoid unnecessary allocations

### Naming Conventions (Standard Rust)

- **Types**: `PascalCase` (Word, ThetaRole, SemanticType)
- **Functions**: `snake_case` (parse_sentence, extract_features, beta_reduce)
- **Constants**: `SCREAMING_SNAKE_CASE` (ROLE_SPECS, DEFAULT_TIMEOUT)
- **Variables**: `snake_case`
- **Files**: `snake_case.rs`
- **Modules**: lowercase

### Architecture Patterns

- **Enum-Heavy Design**: Use enums for type safety (ThetaRole, PartOfSpeech,
  SemanticType)
- **Trait-Based Interfaces**: Define traits for swappable components
- **Builder Patterns**: For complex type construction
- **Strategy Pattern**: Multiple implementations with shared interfaces
- **Pipeline Pattern**: Sequential processing through analysis stages

### Linguistic Standards (Ported from Python V1)

- **Theory-Driven**: Every feature grounded in established linguistic theory
- **Explicit Over Implicit**: No hidden features or black-box processing
- **Compositionality**: Each layer's output is next layer's well-typed input
- **Word-Centric Design**: Core data structures built around Word concept

## Testing Requirements

### Test Types Required

1. **Unit Tests**: Component-level with high coverage
2. **Property Tests**: Linguistic invariants with `proptest`
3. **Golden Tests**: Deterministic output validation with `insta`
4. **Benchmarks**: Performance regression detection with `criterion`
5. **Integration Tests**: End-to-end LSP scenarios

### Quality Gates (ALL MUST PASS)

- ✅ **Tests**: 100% pass rate (`just test`)
- ✅ **Lints**: Zero clippy warnings (`just lint`)
- ✅ **Format**: Code properly formatted (`just fmt-check`)
- ✅ **Performance**: <5% regression (`just perf-check`)
- 🎯 **Coverage**: >90% target (M2+)

### Golden Test Philosophy

From Python V1 system: Use deterministic output validation for:

- Parsing results (sentence → word structures)
- Semantic analysis (theta role assignments)
- Lambda calculus operations (term construction, β-reduction)
- LSP responses (hover content, diagnostics)

## Performance Standards

### Benchmarking Requirements

- **Every major component** must have benchmarks
- **Baseline establishment** before feature development
- **Regression detection** in CI (5% threshold)
- **Memory profiling** for complex operations

### Performance Targets (from ROADMAP.md)

- Parse Latency: <10ms (vs 100ms Python)
- LSP Response: <50ms (vs 200ms Python)
- Throughput: >100 sent/sec (vs 10 Python)
- Memory/Sentence: <25KB (vs 250KB Python)

### Performance Workflow

```bash
# Establish baseline
just bench-baseline

# During development
just bench-compare    # Compare against baseline
just flamegraph      # Profile performance hotspots
just memprof         # Check memory usage
```

## Code Review Standards

### Before Submitting PR

1. **Run quality checks**: `just check-all`
2. **Update benchmarks**: `just bench` if performance-critical changes
3. **Update documentation**: README, architecture docs if needed
4. **Add tests**: Unit, property, golden tests as appropriate
5. **Check coverage**: Maintain high coverage standards

### PR Requirements

- **Clear description** of changes and motivation
- **Performance impact** analysis for significant changes
- **Breaking changes** clearly documented
- **Test coverage** for new functionality
- **Benchmark results** if performance-related

### Review Criteria

- **Correctness**: Code works as intended
- **Performance**: Meets performance standards
- **Maintainability**: Clear, well-documented code
- **Testing**: Comprehensive test coverage
- **Architecture**: Follows established patterns

## Error Handling Standards

### Error Types

Define custom error enums with context:

```rust
#[derive(Error, Debug)]
pub enum CanopyError {
    #[error("parsing failed: {context}")]
    ParseError { context: String },
    #[error("semantic analysis failed: {0}")]
    SemanticError(String),
    #[error("LSP protocol error: {0}")]
    LspError(String),
}
```

### Error Handling Principles

- **Fail fast**: Return errors immediately, don't continue with invalid state
- **Rich context**: Provide detailed error information for debugging
- **Recovery**: Allow graceful degradation where possible
- **No panics**: Library code should never panic on user input

## Documentation Standards

### Required Documentation

- **Public APIs**: All public functions, types, and modules
- **Architecture**: High-level design decisions
- **Examples**: Usage examples for complex APIs
- **Performance**: Benchmark results and analysis

### Documentation Style

````rust
/// Extracts semantic features from a parsed sentence.
///
/// This function implements rule-based feature extraction following
/// the strategy pattern. Features include animacy, definiteness, and
/// quantifier detection.
///
/// # Arguments
/// * `sentence` - A parsed sentence with dependency information
/// * `strategy` - The feature extraction strategy to use
///
/// # Returns
/// A vector of enhanced words with semantic features attached.
///
/// # Performance
/// Typical extraction takes ~1ms per sentence of 20 words.
///
/// # Examples
/// ```rust
/// let features = extract_semantic_features(&sentence, &strategy)?;
/// assert!(features.iter().any(|w| w.animacy.is_some()));
/// ```
pub fn extract_semantic_features(
    sentence: &Sentence,
    strategy: &dyn FeatureStrategy,
) -> Result<Vec<EnhancedWord>, CanopyError> {
    // Implementation...
}
````

## Linguistic Theory Standards

### Theory Implementation Requirements

- **Cite sources**: Reference linguistic papers for theoretical decisions
- **Explicit assumptions**: Document theoretical assumptions clearly
- **Testable predictions**: Theory should make testable predictions
- **Cross-validation**: Compare with established implementations

### Key Theoretical Frameworks

- **Universal Dependencies**: For syntactic representation
- **Neo-Davidsonian Semantics**: For event structures
- **Discourse Representation Theory**: For discourse semantics
- **Optimality Theory**: For constraint-based analysis (M6+)

## Development Tools

### Required Tools

```bash
# Install development tools
cargo install just cargo-watch cargo-tarpaulin cargo-flamegraph
```

### Recommended IDE Setup

- **VS Code** with rust-analyzer extension
- **Vim/Neovim** with rust-analyzer LSP
- **IntelliJ** with Rust plugin

### Pre-commit Hooks

Will be configured to run:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test --quiet`

## Communication Standards

### Issue Reporting

- **Clear description** of problem or feature request
- **Reproduction steps** for bugs
- **Performance impact** if relevant
- **Theoretical motivation** for linguistic features

### Commit Messages

Follow conventional commits:

```
feat(parser): add UDPipe integration with error recovery
perf(semantics): optimize theta role assignment by 15%
fix(lsp): resolve hover timeout on large documents
docs(architecture): update layer design documentation
test(golden): add lambda calculus β-reduction test cases
```

## Getting Help

### Resources

- **Documentation**: [ARCHITECTURE.md](ARCHITECTURE.md),
  [ROADMAP.md](ROADMAP.md)
- **Code Examples**: See `examples/` directory (M2+)
- **Benchmarks**: Check `target/criterion/` for performance analysis

### Questions

- **Development**: Focus on development workflow and tooling
- **Performance**: Benchmarking, optimization, profiling
- **Linguistics**: Theoretical questions about implementation
- **Architecture**: Design decisions and patterns

## Release Process

### Version Bumping

- **Patch**: Bug fixes, minor improvements
- **Minor**: New features, significant improvements
- **Major**: Breaking changes, architectural changes

### Quality Requirements

All releases must pass:

- 100% test success rate
- Zero clippy warnings
- Performance benchmarks within 5% of baseline
- Documentation builds without errors

---

**Remember**: Infrastructure-first development means we build the tools and
processes that enable rapid, high-quality feature development. Quality gates are
not obstacles—they're the foundation that enables confident iteration.
