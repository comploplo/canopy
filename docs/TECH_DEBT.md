# Tech Debt Playbook

## Purpose

Document remaining technical debt and track progress on retiring stub implementations.

## Current Status (Post-M7, December 2025)

**M7 Complete** - Layer 2 Event Composition is fully working. The `canopy-events` crate provides real Neo-Davidsonian event structures with theta role assignment.

## Component Status

| Component                               | Status          | Notes                                                                        |
| --------------------------------------- | --------------- | ---------------------------------------------------------------------------- |
| `canopy-events` (Layer 2)               | ✅ **COMPLETE** | Real event composition with LittleV primitives, theta roles, voice detection |
| `canopy-tokenizer::SemanticCoordinator` | ✅ **COMPLETE** | Real VerbNet/FrameNet/WordNet/PropBank engines, 91.7% cache hit rate         |
| `canopy-treebank`                       | ✅ **COMPLETE** | UD pattern matching, dependency parsing from CoNLL-U                         |
| `canopy-pipeline`                       | ✅ **COMPLETE** | Full L1→L2 pipeline, ~19ms/sentence end-to-end                               |
| `Layer1ParserHandler`                   | ⚠️ **DEFERRED** | Expects pre-parsed UD dependencies (no UDPipe integration)                   |
| `benches/`                              | ⚠️ **PARTIAL**  | Demo provides real metrics; formal benchmarks need update                    |

______________________________________________________________________

## Track A – Real Layer 1 Parsing

### Status: ⚠️ DEFERRED

The system currently expects pre-parsed Universal Dependencies input (CoNLL-U format). UDPipe integration is deferred as the current approach is acceptable for research use cases.

**Current Workaround:**

- Use `canopy-treebank::ConlluParser` for CoNLL-U input
- Real treebank data (UD English-EWT) available for testing
- Works well for linguistic research workflows

**Future Work (Low Priority):**

- Add UDPipe model loading for arbitrary text input
- Create parser adapter in `canopy-pipeline::implementations`

______________________________________________________________________

## Track B – Semantic Layer

### Status: ✅ COMPLETE (M6)

All semantic engines now load real data and fail fast if unavailable:

- **VerbNet**: 333 XML files, 99.7% success rate
- **FrameNet**: 1,200+ frames with lexical units
- **WordNet**: 117,000+ synsets
- **PropBank**: Semantic role labeling
- **Cache**: 91.7% hit rate with lemmatization

**Achievements:**

- `SemanticCoordinator` enforces real data loading
- Engines return `Result<Self>` - no silent degradation
- Production metrics are meaningful
- Test fixtures retired in favor of real data subsets

______________________________________________________________________

## Track C – Layer 2 & Pipeline Integration

### Status: ✅ COMPLETE (M7)

The `canopy-events` crate provides full Layer 2 event composition:

- **EventComposer**: Neo-Davidsonian event structures
- **EventDecomposer**: VerbNet predicates → LittleV primitives
- **ParticipantBinder**: Dependency relations → theta roles
- **Voice Detection**: Active/passive from dependency patterns

**Performance Achieved:**

- Layer 2 composition: 78-148μs per sentence
- Full L1→L2 pipeline: ~19ms per sentence
- 100 sentences processed in 62ms total

**Integration:**

- `canopy-pipeline` provides `create_l1_analyzer_with_treebank()`
- `event_composition_demo.rs` demonstrates full pipeline
- All integration tests pass

______________________________________________________________________

## Track D – Tests, Benchmarks, and Coverage

### Status: ⚠️ IN PROGRESS

**Completed:**

- Removed tautological tests (`assert!(true)`, `is_ok() || is_err()`)
- Deleted placeholder test files with no real assertions
- Coverage at 67% (above 50% gate)

**Remaining:**

- Raise coverage to 70% target
- Update Criterion benchmarks to use real pipeline
- Define CI profiles (`quick` vs `full`)

______________________________________________________________________

## Remaining Tech Debt

| Item                 | Priority | Notes                                   |
| -------------------- | -------- | --------------------------------------- |
| UDPipe Integration   | Low      | Pre-parsed deps acceptable for research |
| Parallel Processing  | Low      | Sequential performance acceptable       |
| Criterion Benchmarks | Medium   | Demo provides metrics for now           |
| Coverage to 70%      | Medium   | Currently at 67%                        |

______________________________________________________________________

## Next Milestone: M8 (Layer 3: DRT & Discourse)

With Tracks B and C complete, the next major work is:

1. Create `canopy-discourse` crate
1. Implement Discourse Representation Structures (DRS)
1. Add reference resolution across sentences
1. Add context tracking and anaphora resolution

See [ROADMAP.md](ROADMAP.md) for M8 details.
