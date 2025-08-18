# Coverage Improvement Plan - M3 Target: 90%

## Current Status (August 17, 2025)
- **Current Coverage**: 63.23% (975/1542 lines covered)
- **Target Coverage**: 90% for M3 presubmit hooks
- **Gap**: 26.77% (approximately 413 additional lines need coverage)

## Coverage Analysis by Crate

### 🔴 Low Coverage Crates (Priority for M3)

#### 1. canopy-cli (0% coverage)
- **Lines**: 0/2 covered
- **Priority**: LOW (small crate, non-critical)
- **Action**: Simple smoke tests

#### 2. canopy-lsp (43.3% coverage)
- **Lines**: 48/111 covered  
- **Priority**: HIGH (LSP integration critical)
- **Key uncovered areas**:
  - LSP backend (0/8 lines)
  - Main server entry (0/14 lines) 
  - VerbNet integration tests (0/30 lines)
  - Diagnostics (0/6 lines)

#### 3. canopy-semantics VerbNet feature extraction (15.7% coverage)
- **Lines**: 18/115 covered
- **Priority**: HIGH (core M3 functionality)
- **Key uncovered areas**:
  - Feature extraction algorithms
  - Selectional restriction processing
  - Confidence scoring
  - Error handling paths

### 🟡 Medium Coverage Crates (Needs improvement)

#### 4. canopy-parser evaluation (80.1% coverage)
- **Lines**: 201/251 covered
- **Priority**: MEDIUM
- **Key gaps**: CoNLL-U parsing edge cases, error handling

#### 5. canopy-semantics VerbNet parser (50.3% coverage)  
- **Lines**: 88/175 covered
- **Priority**: MEDIUM
- **Key gaps**: Complex XML parsing, error recovery

### ✅ Good Coverage Crates (Maintain)

#### 6. canopy-core (96% coverage)
- **Lines**: 93/100 covered
- **Status**: EXCELLENT ✅

#### 7. canopy-parser core (77.5% coverage)
- **Lines**: 465/600 covered  
- **Status**: GOOD - needs minor improvements

## M3 Coverage Improvement Strategy

### Phase 1: Quick Wins (Target: +15% coverage)

**Week 1-2: LSP Infrastructure Tests**
- Add LSP server startup/shutdown tests
- Test basic LSP message handling
- Cover diagnostics generation
- **Expected gain**: +8% coverage

**Week 2-3: CLI and Integration Tests**  
- Add CLI smoke tests
- Integration test coverage for main paths
- Error handling test paths
- **Expected gain**: +7% coverage

### Phase 2: Core Functionality (Target: +12% coverage)

**Week 3-4: VerbNet Feature Extraction**
- Test all feature extraction strategies
- Cover confidence scoring algorithms
- Test selectional restriction processing
- Edge case handling (malformed input, missing data)
- **Expected gain**: +8% coverage

**Week 4-5: Parser Edge Cases**
- CoNLL-U format edge cases
- UDPipe integration error paths
- Memory management error conditions
- **Expected gain**: +4% coverage

### Phase 3: Final Push to 90% (Target: +2% coverage)

**Week 5-6: Comprehensive Review**
- Property-based tests for uncovered branches
- Error injection testing
- Performance edge case testing
- **Expected gain**: +2% coverage

## Implementation Plan by Week

### Week 1: LSP Infrastructure (Current: 63% → Target: 68%)

```rust
// canopy-lsp/src/tests/integration_tests.rs
#[cfg(test)]
mod lsp_integration_tests {
    #[test]
    fn test_lsp_server_startup() { /* ... */ }
    
    #[test] 
    fn test_hover_requests() { /* ... */ }
    
    #[test]
    fn test_diagnostic_generation() { /* ... */ }
    
    #[test]
    fn test_server_shutdown() { /* ... */ }
}
```

### Week 2: CLI and Error Paths (Target: 68% → 75%)

```rust
// canopy-cli/src/tests.rs
#[test]
fn test_cli_help() { /* ... */ }

#[test] 
fn test_cli_file_processing() { /* ... */ }

// Add error path tests across all crates
#[test]
fn test_error_handling_paths() { /* ... */ }
```

### Week 3-4: VerbNet Feature Extraction (Target: 75% → 83%)

```rust
// canopy-semantics/src/verbnet/feature_extraction/tests.rs
#[cfg(test)]
mod comprehensive_feature_tests {
    #[test]
    fn test_all_extraction_strategies() { /* ... */ }
    
    #[test]
    fn test_confidence_scoring_edge_cases() { /* ... */ }
    
    #[test] 
    fn test_selectional_restriction_processing() { /* ... */ }
    
    #[test]
    fn test_malformed_input_handling() { /* ... */ }
}
```

### Week 5-6: Parser and Final Push (Target: 83% → 90%+)

```rust
// Property-based testing
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parser_invariants(input in ".*") {
        // Ensure parser never crashes
    }
    
    #[test]
    fn test_feature_extraction_consistency(
        word in any::<String>(),
        pos in any::<UPos>()
    ) {
        // Ensure feature extraction is deterministic
    }
}
```

## Coverage Monitoring and Enforcement

### Presubmit Hook Configuration ✅
- **Status**: COMPLETED
- Coverage check script: `scripts/check-coverage.sh`
- Threshold: 90% for M3
- Integration: Pre-commit hooks + justfile commands

### Continuous Integration
- Coverage reporting on every PR
- Trend tracking (coverage should not decrease)
- Automatic coverage reports in HTML format

### Coverage Quality Gates
1. **M3 Milestone**: 90% minimum for presubmit
2. **New Code**: 95% coverage requirement for new features
3. **Critical Paths**: 100% coverage for error handling, security

## Tools and Infrastructure

### Current Setup ✅
- `cargo-tarpaulin` for coverage analysis
- HTML reports in `coverage/` directory
- Pre-commit hooks with threshold checking
- Justfile integration (`just coverage`, `just coverage-check`)

### Additional Tools for M3
```bash
# Advanced coverage analysis
just coverage-detailed     # Detailed per-function analysis
just coverage-diff         # Coverage comparison between branches
just coverage-critical     # Focus on critical path coverage
```

## Risk Assessment and Mitigation

### Risk: Coverage vs. Quality Trade-off
- **Mitigation**: Focus on meaningful tests, not just line coverage
- **Quality gates**: All tests must actually test behavior, not just execute code

### Risk: Performance Impact of Extensive Testing
- **Mitigation**: Parallel test execution, optimized test data
- **Monitor**: Test suite should complete in <2 minutes

### Risk: False Sense of Security
- **Mitigation**: Combine coverage with:
  - Property-based testing
  - Mutation testing (future)
  - Integration testing
  - Manual testing of critical paths

## Success Metrics

### Quantitative Targets
- **M3 Launch**: ≥90% line coverage
- **Test Suite Speed**: <2 minutes total
- **Coverage Stability**: No decrease >2% between releases

### Qualitative Targets  
- **Error Path Coverage**: All error conditions tested
- **Integration Coverage**: Full LSP protocol compliance tested
- **Performance Coverage**: All performance-critical paths tested

## Implementation Timeline

```
Week 1 (Aug 18-24): LSP Infrastructure Tests        → 68% coverage
Week 2 (Aug 25-31): CLI & Integration Tests         → 75% coverage  
Week 3 (Sep 1-7):   VerbNet Feature Extraction     → 83% coverage
Week 4 (Sep 8-14):  Parser Edge Cases               → 87% coverage
Week 5 (Sep 15-21): Property-based & Final Push    → 90%+ coverage
Week 6 (Sep 22-28): M3 Launch Readiness Review     → M3 READY ✅
```

## Command Reference

### Development Commands
```bash
# Generate coverage report
just coverage

# Check coverage threshold (used by presubmit)
just coverage-check

# Update coverage threshold
just coverage-threshold 90

# Run specific test categories
cargo test --workspace --test "*integration*"
cargo test --workspace --test "*golden*"
```

### Coverage Analysis
```bash
# Detailed HTML report
cargo tarpaulin --workspace --out Html --output-dir coverage

# Focus on specific crate
cargo tarpaulin --package canopy-lsp --out Html

# Line-by-line analysis
cargo tarpaulin --workspace --out Lcov --output-dir coverage
```

## Long-term Sustainability

### M4+ Enhancements
- **Mutation Testing**: Verify test quality, not just coverage
- **Behavioral Coverage**: Track semantic behavior coverage
- **Cross-platform Testing**: Coverage on multiple platforms
- **Benchmark Coverage**: Performance regression testing

### Documentation Coverage
- **API Documentation**: 100% public API documented
- **Example Coverage**: All major features have examples
- **Integration Guides**: Complete setup and usage guides

---

## Summary

This plan provides a systematic approach to achieving 90% coverage for M3:

1. **✅ Infrastructure Complete**: Presubmit hooks, threshold checking, reporting
2. **🎯 Clear Targets**: Weekly milestones with specific coverage goals
3. **📋 Prioritized Work**: Focus on high-impact, low-effort improvements first
4. **🔬 Quality Focus**: Meaningful tests that actually verify behavior
5. **📊 Continuous Monitoring**: Automated coverage tracking and reporting

**Expected Outcome**: M3 launch with 90%+ coverage, robust test suite, and automated quality gates that maintain coverage standards going forward.