# Task Completion Checklist

## When Working on Python V1 System
Located at `/Users/gabe/projects/canopy/spacy-lsp/`

### Before Making Changes
1. **Read existing implementation** - Use symbolic tools to understand current code
2. **Check test coverage** - Identify areas that need testing
3. **Run baseline tests** - `make test` to ensure starting from clean state

### During Development
1. **Follow Word-centric architecture** - Work with Word objects, not JSON dicts
2. **Maintain type safety** - Use enums and type hints
3. **Write tests first** - Add test cases before implementation
4. **Check performance** - Monitor for regressions with pytest-benchmark

### Before Submitting Changes
1. **Run formatting** - `make format`
2. **Fix linting issues** - `make lint` (20 issues are known and deferred to M12)
3. **Run full test suite** - `make test` (673/673 tests must pass)
4. **Check coverage** - Must maintain 85% minimum (targeting 90% restoration)
5. **Run pre-submit gates** - `make presubmit`
6. **Performance check** - `make perf-view` to check for regressions

### Quality Gates (Enforced by M11+ Infrastructure)
- ✅ 100% test pass rate (673/673 tests)
- ✅ 85% code coverage minimum (with plan to restore 90%)
- ✅ Performance regression detection (50% degradation threshold)
- ⚠️ 20 linting issues deferred to M12 (non-blocking)

## When Working on Rust V2 System
Located at `/Users/gabe/projects/canopy/` (root level)

### Before Making Changes
1. **Study V1 reference implementation** - Understand Python system architecture
2. **Review linguistic theory** - DRT, Optimality Theory, movement chains
3. **Set up Rust environment** - Install Rust, UDPipe, create cargo project

### During Development
1. **Layer-by-layer implementation** - Follow 4-layer architecture (morphosyntax → events → DRT → LSP)
2. **Type-driven design** - Use Rust's type system for linguistic constraints
3. **Port V1 patterns** - Theta roles, VerbNet integration, corpus patterns
4. **Benchmark against V1** - Target 10x performance improvement

### Before Submitting Changes
1. **Run Rust toolchain** - `cargo fmt`, `cargo clippy`, `cargo test`
2. **Performance benchmarks** - `cargo bench`, compare against Python baseline
3. **Integration tests** - Ensure LSP protocol compliance
4. **Cross-validation** - Compare outputs with Python V1 system

### Success Metrics for V2
- Sub-50ms LSP response times (vs 200ms Python)
- 10x throughput improvement (500 sentences/sec vs 50)
- 95%+ theta role accuracy on VerbNet test suite
- Feature parity with Python V1 LSP capabilities
- Theory-testing framework operational

## File Deletion Policy (Important)
**PRAGMA**: Before deleting any file, always read its contents first to verify what would be lost. Check for:
- Undocumented implementations or logic
- Performance insights or research findings  
- Integration patterns or architectural decisions
- Test cases or validation approaches
- Preserve valuable content in appropriate documentation before deletion

## Release Process (V1 System)
1. **Complete milestone requirements** - All deliverables implemented
2. **Pass all quality gates** - Tests, coverage, performance, linting
3. **Run pre-submit validation** - `make presubmit`
4. **Create release** - `make release-patch/minor/major`
5. **Update documentation** - README.md, ROADMAP.md, SYSTEM_WALKTHROUGH.md