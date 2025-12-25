# Tech Debt Playbook: Retiring Stub Implementations

## Purpose

- Document every major stub or placeholder path that still props up the M5 codebase.
- Describe the engineering risk each stub adds (incorrect semantics, misleading benchmarks, brittle tests).
- Lay out pragmatic, sequenced work packages that move the workspace to production-ready layers without backsliding on coverage.

## Stub Landscape (Early 2025 Snapshot)

| Component                                                                          | Stub / Placeholder Symptom                                                                                                                       | Production Risk                                                                                                   | Exit Strategy Snapshot                                                                                                                                     |
| ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `canopy_core::layer1parser::Layer1ParserHandler`                                   | Tokenization + morphology are hand-rolled heuristics; no UDPipe/Treebank integration.                                                            | Wrong lemmas / POS propagate to every downstream layer; cache metrics meaningless.                                | Replace with `canopy-treebank` parser wrapper plus configurable UDPipe model loading via pipeline container.                                               |
| `canopy_core::layer1parser::SemanticAnalysisHandler`                               | Simply echoes tokens and sets ad-hoc animacy/definiteness flags.                                                                                 | Layer 2 gets no theta roles; confidence / cache stats useless.                                                    | Delegate to `canopy-tokenizer::SemanticCoordinator` (real engines + cache) and enforce data-availability gating.                                           |
| `canopy-tokenizer` Layer 1 (`coordinator.rs`, `lemmatizer.rs`, `test_fixtures.rs`) | Coordinator simplifies engine init, falls back to in-memory fixtures when data missing; lemmatizer is rule-based stub; treebank provider unused. | Impossible to validate "production metrics"; tests silently pass without resources.                               | Harden initialization (no silent degradation), wire shared lemma cache + treebank provider, retire fixture-based tests in favor of on-disk sample corpora. |
| `canopy-tokenizer::integration`                                                    | References `Layer2Analyzer` stub (no real compositional semantics).                                                                              | Event layer (M7 goals) never exercised; API misleads consumers.                                                   | Reintroduce real Layer 2 crate or new module, refactor integration to consume actual event builder, adjust DI to avoid cycles.                             |
| `canopy_pipeline`                                                                  | DI container builds against abstract traits but factory implementations missing; feature extraction disabled; cache warmups no-op.               | `PipelineBuilder` looks production-ready but nothing real is wired, leading to runtime panics or empty responses. | Implement concrete `MorphosyntacticParser` + `SemanticAnalyzer` adapters, re-enable feature stages, and enforce readiness checks before serving.           |
| `benches/`, coverage boost tests                                                   | Criterion benches use dummy functions; coverage suites assert on stub behavior.                                                                  | Metrics dashboards meaningless; encourages keeping stubs to satisfy coverage gates.                               | Replace with smoke benches that call real pipeline (on sampled corpora) and rewrite tests to assert substantive semantics.                                 |

## Track A – Restore Real Layer 1 Parsing

### Current State

- `Layer1ParserHandler::process_with_udpipe` uses whitespace splitting, heuristic POS, and hard-coded morphology.
- There is no path to load actual UDPipe models even though the roadmap promises it.

### Target State

- Parsing runs through a configurable backend (UDPipe 1.2/2.15, eventually other models) exposed by `canopy-pipeline`’s DI container.
- `canopy-treebank` supplies golden CoNLL-U samples for regression tests and lemma validation.

### Action Plan

1. **Create parser adapter** inside `canopy-pipeline::implementations` that wraps `canopy-treebank::ConlluParser` for offline tests and a UDPipe-backed parser for production.
1. **Inject via DI**: update `PipelineContainer::builder()` consumers so `Layer1ParserHandler` is constructed from the container instead of instantiating itself.
1. **Refactor handler**: move the heuristic logic into a fallback path and gate it behind `#[cfg(test)]`; primary implementation calls the adapter asynchronously, preserves sentence boundaries, and records real timings.
1. **Model lifecycle**: surface configuration in `docs/` for where to place UDPipe models (`data/ud_english-ewt`), add readiness + failure states to `ComponentHealth`.
1. **Testing**: add integration tests that feed a short CoNLL-U fixture through the pipeline and assert POS / lemma parity with the source treebank.

### Exit Criteria

- No direct `.split_whitespace()` / heuristic POS inside production handler.
- CI enforces presence of parser model for “full” test suite while allowing stub mode via feature flag for quick unit tests.

## Track B – Replace Semantic Layer Stub Logic

### Current State

- `SemanticAnalysisHandler` only tags animacy/definiteness using simple lookup tables.
- `SemanticCoordinator` allows data-less operation by silently ignoring engine load failures; tests rely on `test_fixtures` maps.

### Target State

- Layer 1 semantics always drive from on-disk VerbNet/FrameNet/WordNet resources (or fail fast with actionable errors).
- Coordinated analysis fills theta roles, frame elements, WordNet senses, and reports real cache metrics.

### Action Plan

1. **Tighten engine init**: update `SemanticCoordinator::new` to log and propagate a structured error when a required resource is missing unless `graceful_degradation` is explicitly requested.
1. **Wire handler**: have `SemanticAnalysisHandler::process` call into a shared `SemanticCoordinator` (owned by the container) rather than augmenting tokens inline. Store semantic output in `Word.misc` or a dedicated struct consumed by Layer 2.
1. **Cache discipline**: formalize cache sizing via `CoordinatorConfig` (bounded by `l1_cache_memory_mb`) and expose metrics to `ComponentHealth`.
1. **Clean fixtures**: shrink `test_fixtures` usage to unit tests behind `#[cfg(test)]`; generate deterministic sample outputs by loading a tiny subset of the real data (`data/verbnet/verbnet-test`, mini FrameNet bundle).
1. **Observability**: ensure `SemanticAnalysisHandler` increments real metrics (success rate, avg latency) derived from coordinator stats.

### Exit Criteria

- Handler returns enriched `Word`s with populated semantic metadata; heuristic animacy paths only live in dedicated fallback tests.
- Failing to load VerbNet/FrameNet during startup is a hard error in production builds.

## Track C – Reintroduce Real Layer 2 & Pipeline Integration

### Current State

- `canopy_tokenizer::integration` defines `Layer2Analyzer` / `Layer2Config` as implicit stubs (no actual crate), so `SemanticPipeline` never surfaces events.
- `canopy_pipeline` disables feature extraction and caches, while the DI factory lacks concrete implementations.

### Target State

- A concrete Layer 2 module assigns theta grids, builds events, and feeds them into `canopy-pipeline`.
- Pipeline stages (`Layer1Parsing`, `FeatureExtraction`, `Layer2Analysis`) execute real work, with caches + metrics turned on.

### Action Plan

1. **Define Layer 2 crate**: revive the old `canopy-semantics` functionality or create `canopy-layer2` implementing event builders, theta assignment (from VerbNet + dependency patterns), and aspect classification.
1. **Stabilize interfaces**: move shared types (`SemanticPredicate`, `Event`, etc.) into a neutral crate (`canopy-model`) to prevent circular dependencies.
1. **Replace stubs**: in `integration.rs`, import the real analyzer, delete the in-file stub struct, and adjust tests to validate actual event counts / roles.
1. **Pipeline wiring**: implement concrete `SemanticAnalyzer` trait adapter that wraps the Layer 2 analyzer; register it in `PipelineContainer::builder()` and re-enable `run_feature_extraction` stage using feature extractor plugins.
1. **Caching & batching**: finalize `check_cache` / `cache_result` by storing serialized `SemanticLayer1Output` (e.g., via `serde_json`) and validate TTL logic with integration tests.

### Exit Criteria

- `SemanticPipeline::analyze(..., true)` produces events with theta roles for canonical examples and integration tests assert on them.
- `canopy-pipeline` no longer uses `if false` guards; feature extraction executes or is feature-gated with clear documentation.

## Track D – Tests, Benchmarks, and Coverage

### Current State

- Coverage “boost” modules assert trivial truths just to raise percentages.
- Criterion benches benchmark dummy functions, hiding real regressions.
- Integration tests often `match` on `Err(_)` and treat failure as acceptable.

### Target State

- Tests verify real semantics, failing loudly when resources are missing.
- Benchmarks measure parser + semantic throughput on curated corpora.
- Coverage remains high through meaningful unit/integration tests.

### Action Plan

1. **Cull coverage shims**: remove modules like `quick_coverage_boost.rs` once substantive tests replace them. Ensure new tests keep coverage ≥ current gate.
1. **Golden datasets**: add tiny VerbNet/FrameNet/WordNet subsets under `data/test-fixtures/` to power deterministic tests without requiring the full corpora.
1. **Benchmark redesign**: update `benches/` to run `CanopyAnalyzer` on short passages (10–20 sentences) and record layer timings. Cache data downloads in CI using artifacts.
1. **Health assertions**: modify integration tests to assert on semantic content (number of predicates, frames, theta roles) and treat unexpected `Err` as failure except where explicitly feature-gated.
1. **CI profiles**: define two profiles—`quick` (no large data, stub-friendly) and `full` (requires corpora, runs nightly)—documented in `docs/CONTRIBUTING.md`.

### Exit Criteria

- No tests rely on `assert!(true)` when a stub fails.
- Benchmarks emit actionable metrics (latency, cache hit rate) derived from real pipeline executions.

## Sequencing & Milestones

1. **Milestone S1 (Pre-M7)** – Tracks A + B: land real parser + semantic handler; ensure Layer 1 parity with roadmap promises.
1. **Milestone S2 (M7)** – Track C core: ship event builder crate, rewire pipeline, update documentation.
1. **Milestone S3 (Continuous)** – Track D: replace coverage/bench placeholders as each track delivers real functionality.

Each milestone should conclude with:

- `just check-all` on the “full” profile.
- Regression benchmarks recorded in `docs/reference/performance.md`.
- Update to this playbook noting which stubs were retired.

## Immediate Next Steps

1. Draft parser adapter interface and spike integration with a single UDPipe model to validate DI plumbing.
1. Add failing tests that assert on theta role output for `"John gave Mary a book"`—they will guide Track B/C work.
1. Audit CI to ensure resource bundles (VerbNet/FrameNet) are available or explicitly skipped in quick profile.
1. Socialize this document with the team; assign owners + timelines for each track.

Once these foundations are in place, the workspace can legitimately claim “no stubs” for Layer 1 and have a clear runway for the Layer 2/3 roadmap.
