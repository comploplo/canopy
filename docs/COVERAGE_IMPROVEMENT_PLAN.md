# Coverage Improvement Plan - M3 Target: 80%

## Current Status (August 19, 2025)

- **Current Coverage**: 68.39% (1051/1536 lines covered)
- **Current Threshold**: 80% (enforced by presubmit hooks)
- **M3 Target Coverage**: 80% for presubmit hooks
- **M4 Target Coverage**: 90% for enhanced quality gates
- **Gap to M3**: 11.61% (approximately 179 additional lines need coverage)

## Progressive Coverage Strategy

We've made excellent progress from ~63% baseline to **68.39%** current state.
Now focusing on the final push to **80%** for M3 completion:

1. **Current level**: 68.39% achieved ✅
2. **M3 target**: 80% (11.61% remaining gap)
3. **M4 target**: 90% (future enhancement)

## Coverage Analysis by Priority

### 🔴 High Priority Areas (Critical for 80% target)

Based on current coverage gaps, these areas likely need the most attention:

#### 1. canopy-lsp (LSP Integration)

- **Priority**: CRITICAL (LSP functionality essential)
- **Likely gaps**:
  - LSP server startup/shutdown
  - Message handling and protocol compliance
  - Diagnostics generation and formatting
  - Error handling in LSP communication

#### 2. canopy-cli (Command Line Interface)

- **Priority**: HIGH (user-facing functionality)
- **Likely gaps**:
  - CLI argument parsing and validation
  - File processing workflows
  - Error reporting and help systems

#### 3. Error Handling Paths (All Crates)

- **Priority**: HIGH (robustness critical)
- **Likely gaps**:
  - Exception handling in parsing
  - Network/IO error conditions
  - Invalid input handling
  - Resource cleanup on failures

### 🟡 Medium Priority Areas (Good coverage, minor gaps)

#### 4. canopy-parser Edge Cases

- **Priority**: MEDIUM (core functionality mostly covered)
- **Likely gaps**:
  - Malformed input handling
  - Memory management edge cases
  - Complex parsing scenarios

#### 5. canopy-semantics Integration Points

- **Priority**: MEDIUM (core logic well-tested)
- **Likely gaps**:
  - VerbNet integration error paths
  - Feature extraction edge cases
  - Performance monitoring code

### ✅ Well-Covered Areas (Maintain quality)

#### 6. canopy-core (Excellent Coverage)

- **Status**: STRONG foundation ✅
- **Focus**: Maintain current high coverage

#### 7. canopy-semantics Core Logic

- **Status**: GOOD coverage (142 tests passing)
- **Focus**: Minor gap filling

## M3 Coverage Strategy (11.61% Gap Closure)

### Phase 1: LSP Infrastructure (Target: +6% coverage)

**Priority 1: LSP Core Functionality**

- Add comprehensive LSP server lifecycle tests
- Test message parsing and response generation
- Cover diagnostics and hover functionality
- **Expected gain**: +4-5% coverage

**Priority 2: LSP Error Handling**

- Test protocol violation handling
- Network disconnection scenarios
- Invalid request handling
- **Expected gain**: +1-2% coverage

### Phase 2: CLI and Integration (Target: +4% coverage)

**Priority 1: CLI Coverage**

- Command-line argument processing
- File input/output operations
- Help and error message generation
- **Expected gain**: +2-3% coverage

**Priority 2: Integration Test Paths**

- End-to-end workflow testing
- Cross-crate integration points
- Configuration and setup scenarios
- **Expected gain**: +1-2% coverage

### Phase 3: Error Paths and Edge Cases (Target: +2% coverage)

**Priority 1: Error Handling**

- Exception propagation testing
- Resource cleanup verification
- Graceful degradation scenarios
- **Expected gain**: +1-1.5% coverage

**Priority 2: Edge Case Coverage**

- Boundary condition testing
- Performance edge cases
- Memory constraint scenarios
- **Expected gain**: +0.5-1% coverage

## Implementation Plan

### Week 1: LSP Infrastructure

```rust
// canopy-lsp/src/tests/integration_tests.rs
#[cfg(test)]
mod lsp_server_tests {
    #[test]
    fn test_server_startup_shutdown() { /* ... */ }

    #[test]
    fn test_hover_request_handling() { /* ... */ }

    #[test]
    fn test_diagnostic_generation() { /* ... */ }

    #[test]
    fn test_protocol_error_handling() { /* ... */ }
}
```

### Week 2: CLI and Error Paths

```rust
// canopy-cli/src/tests.rs
#[cfg(test)]
mod cli_tests {
    #[test]
    fn test_argument_parsing() { /* ... */ }

    #[test]
    fn test_file_processing() { /* ... */ }

    #[test]
    fn test_error_reporting() { /* ... */ }
}

// Add error path tests across all crates
#[test]
fn test_parser_error_recovery() { /* ... */ }
```

### Week 3: Integration and Polish

```rust
// Integration tests
#[test]
fn test_end_to_end_workflow() { /* ... */ }

// Property-based testing for edge cases
use proptest::prelude::*;
proptest! {
    #[test]
    fn test_parser_robustness(input in ".*") {
        // Ensure parser handles any input gracefully
    }
}
```

## Coverage Monitoring and Enforcement

### Current Setup ✅

- **Status**: ACTIVE AND ENFORCED
- Coverage check script: `scripts/check-coverage.sh`
- Current threshold: **80%** (M3 target)
- M4 future threshold: **90%**
- Integration: Pre-commit hooks + justfile commands

### Quality Gates

1. **M3 Milestone**: 80% minimum for presubmit ✅ ACTIVE
2. **M4 Milestone**: 90% minimum for presubmit (future)
3. **New Code**: High coverage requirement for new features
4. **Critical Paths**: Maximum coverage for error handling, security

## Tools and Infrastructure

### Current Setup ✅

- `cargo-tarpaulin` for coverage analysis
- HTML reports in `coverage/` directory
- Pre-commit hooks with threshold checking
- Justfile integration (`just coverage`, `just coverage-check`)

### Development Commands

```bash
# Generate coverage report
just coverage

# Check coverage threshold (used by presubmit)
just coverage-check

# Detailed HTML report
cargo tarpaulin --workspace --out Html --output-dir coverage

# Focus on specific crate
cargo tarpaulin --package canopy-lsp --out Html
```

## Risk Assessment and Mitigation

### Risk: Coverage vs. Quality Trade-off

- **Mitigation**: Focus on meaningful tests that verify actual behavior
- **Quality gates**: Tests must validate functionality, not just execute code

### Risk: Diminishing Returns

- **Current situation**: Last 11.61% will be harder to achieve
- **Mitigation**: Focus on high-value areas (LSP, CLI, error handling)
- **Pragmatic approach**: Prioritize critical functionality over edge cases

### Risk: Test Suite Performance

- **Current status**: 269 tests running efficiently
- **Monitor**: Keep test suite under 2 minutes total
- **Mitigation**: Parallel execution, optimized test data

## Success Metrics

### Quantitative Targets

- **M3 Target**: ≥80% line coverage ✅ (11.61% remaining)
- **M4 Target**: ≥90% line coverage (future)
- **Test Suite Speed**: <2 minutes total
- **Coverage Stability**: No decrease >2% between releases

### Qualitative Targets

- **Error Path Coverage**: All critical error conditions tested
- **LSP Coverage**: Full protocol compliance and error handling
- **CLI Coverage**: Complete user-facing functionality tested
- **Integration Coverage**: End-to-end workflows validated

## Current Progress Summary

### ✅ Achievements to Date

- **68.39% coverage achieved** (up from ~63% baseline)
- **269 tests passing** across all crates
- **Robust test infrastructure** with comprehensive coverage reporting
- **80% threshold enforced** by presubmit hooks
- **Clean codebase** with no compilation errors or critical warnings

### 🎯 Remaining Work for M3 (80% target)

- **11.61% coverage gap** to close
- **~179 additional lines** need test coverage
- **Primary focus areas**: LSP, CLI, error handling
- **Timeline**: Achievable with focused effort on high-impact areas

### 🚀 Future M4 Goals (90% target)

- **Additional 21.61% gap** after M3 completion
- **Enhanced quality gates** and more comprehensive testing
- **Advanced testing**: Property-based, mutation testing, behavioral coverage

## Command Reference

### Coverage Analysis

```bash
# Current coverage status
cargo tarpaulin --workspace --skip-clean

# Per-crate coverage breakdown
cargo tarpaulin --package canopy-lsp
cargo tarpaulin --package canopy-cli
cargo tarpaulin --package canopy-semantics
cargo tarpaulin --package canopy-parser
cargo tarpaulin --package canopy-core

# Generate HTML report
cargo tarpaulin --workspace --out Html --output-dir coverage
```

### Testing Commands

```bash
# Run all tests
cargo test --workspace

# Run specific test categories
cargo nextest run --workspace
cargo test --workspace --test "*integration*"
cargo test --workspace --test "*golden*"

# Run tests with coverage
cargo tarpaulin --workspace
```

---

## Summary

We have made excellent progress toward the M3 coverage target:

1. **✅ Strong Foundation**: 68.39% coverage with 269 passing tests
2. **🎯 Clear Path**: 11.61% gap to 80% M3 target is achievable
3. **📋 Focused Strategy**: LSP + CLI + error handling = high-impact areas
4. **🔧 Infrastructure Ready**: All coverage tooling and enforcement active
5. **📊 Quality Maintained**: Meaningful tests with robust verification

**Expected Outcome**: M3 completion with 80%+ coverage through focused testing
of LSP functionality, CLI interface, and error handling paths. This establishes
a strong foundation for M4's 90% target.
