- **Current Status**: We have achieved 69.46% test coverage with comprehensive test suites. Coverage gate temporarily set to 69% to allow continued development while we work toward M3/M4 targets.

# CRITICAL COVERAGE REQUIREMENTS - DO NOT IGNORE

## Coverage Gates for Releases
- **CURRENT GATE**: 69% (baseline: 69.46% achieved)
- **M3 REQUIREMENT**: 80% minimum test coverage
- **M4 REQUIREMENT**: 90% minimum test coverage + clippy tech debt resolution

## Current Coverage Achievements
- **Server tests**: 18 comprehensive tests covering configuration, health, error handling, concurrency
- **VerbNet integration**: 99.7% success rate (332/333 XML files)
- **UDPipe tests**: All 7 tests passing with model loading and parsing
- **0% coverage files**: Added targeted tests for main.rs and lib.rs files

## M4 CLIPPY TECH DEBT REQUIREMENTS
**All clippy allows must be resolved for M4 release:**

### Current Tech Debt (to be fixed by M4):
- `#![allow(clippy::uninlined_format_args)]` - Convert to modern format syntax
- `#![allow(clippy::needless_borrow)]` - Remove unnecessary explicit borrows
- `#![allow(clippy::field_reassign_with_default)]` - Use struct initialization syntax
- `#![allow(clippy::collapsible_if)]` - Simplify nested conditionals
- `#![allow(clippy::useless_vec)]` - Use arrays where appropriate
- `#![allow(clippy::manual_clamp)]` - Use `.clamp()` method
- `#![allow(clippy::enum_variant_names)]` - Improve enum naming
- `#![allow(clippy::needless_range_loop)]` - Use iterators instead of indexing
- `#![allow(clippy::new_without_default)]` - Add Default implementations
- `#![allow(clippy::clone_on_copy)]` - Remove unnecessary clones

## IMPORTANT REMINDERS FOR CLAUDE
**NEVER LOWER COVERAGE GATES OR SKIP PRESUBMITS DURING RELEASES**

When working on releases:
1. ✅ Always run `scripts/check-coverage.sh` before release
2. ✅ Ensure all pre-commit hooks pass
3. ✅ Add tests to meet coverage requirements instead of lowering gates
4. ❌ NEVER modify coverage thresholds to make releases easier
5. ❌ NEVER skip or bypass presubmit checks
6. ❌ NEVER commit code that fails coverage requirements

The coverage gate exists for quality assurance. Lowering it defeats the purpose and reduces code quality. Always improve test coverage rather than lowering standards.
