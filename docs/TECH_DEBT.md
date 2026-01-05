# Tech Debt

## Current Status

Most tech debt has been retired. The system is production-ready.

## Resolved

| Item                 | Resolution                              |
| -------------------- | --------------------------------------- |
| Stub implementations | Eliminated - all engines load real data |
| 11-crate sprawl      | Consolidated to 3 crates                |
| Coverage gaps        | 81%+ coverage achieved                  |
| Tautological tests   | Removed                                 |
| Dead code            | HeuristicParser and duplicates deleted  |
| Duplicated utilities | Extracted to shared modules             |

## Remaining (Low Priority)

| Item                 | Priority | Notes                                 |
| -------------------- | -------- | ------------------------------------- |
| UDPipe integration   | Low      | CoNLL-U input acceptable for research |
| Criterion benchmarks | Low      | Demo provides metrics                 |
| Parallel processing  | Low      | Sequential performance acceptable     |

## Architecture Improvements (January 2026)

Recent cleanup:

- Deleted 438-line `HeuristicParser` (unused dead code)
- Created `shared.rs` for deduplicated utilities
- Added `SharedEngines` for efficient engine sharing
- Pipeline now creates engines once, shares across components

## Quality Gates

- **Coverage**: 80% gate enforced
- **No stubs**: Engines fail fast if data unavailable
- **No word lists**: All vocabulary from data files
