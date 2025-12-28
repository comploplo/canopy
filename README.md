# Canopy

**Semantic linguistic analysis in Rust**

Canopy is a high-performance library for deep semantic analysis of text. It combines multiple linguistic resources (VerbNet, FrameNet, WordNet, PropBank) to produce rich semantic representations including event structures, theta roles, and frame semantics.

> **Note**: Canopy currently analyzes pre-parsed sentences from Universal Dependencies treebanks. Arbitrary text parsing is planned for future releases.

## Features

- **Multi-engine semantic analysis** — VerbNet verb classes, FrameNet frames, WordNet synsets, and custom lexicon support
- **Event composition** — Neo-Davidsonian event structures with theta role assignment
- **Treebank integration** — Pattern matching against UD English-EWT corpus
- **Production performance** — ~19ms per sentence end-to-end, with intelligent caching
- **Pure Rust** — No external runtime dependencies, memory-safe, concurrent

## Quick Start

```bash
git clone https://github.com/yourusername/canopy
cd canopy
cargo build --release

# Run the event composition demo
cargo run --release -p canopy-pipeline --example event_composition_demo
```

## Architecture

```
                              Input Text
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                        canopy-pipeline                                   │
│                     (orchestration layer)                                │
└──────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  LAYER 1: Semantic Analysis                             canopy-tokenizer │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                    SemanticCoordinator                             │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │  │
│  │  │ VerbNet  │ │ FrameNet │ │ WordNet  │ │ PropBank │ │ Treebank │  │  │
│  │  │ 333 verb │ │ 1,200+   │ │ 117,000+ │ │ semantic │ │ UD deps  │  │  │
│  │  │ classes  │ │ frames   │ │ synsets  │ │ roles    │ │ patterns │  │  │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │  │
│  │       │            │            │            │            │        │  │
│  │       └────────────┴─────┬──────┴────────────┴────────────┘        │  │
│  │                          ▼                                         │  │
│  │              Layer1SemanticResult                                  │  │
│  │    (lemma, verb classes, frames, synsets, theta roles, deps)       │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  LAYER 2: Event Composition                                canopy-events │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                       EventComposer                                │  │
│  │                                                                    │  │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │  │
│  │  │ Event Decomposer│  │ Participant     │  │ Voice & Aspect  │     │  │
│  │  │ VerbNet → LittleV  │ Binder          │  │ Detection       │     │  │
│  │  │ primitives      │  │ theta roles     │  │ active/passive  │     │  │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │  │
│  │           └───────────────┬────┴───────────────────┘               │  │
│  │                           ▼                                        │  │
│  │              Neo-Davidsonian Event Structure                       │  │
│  │   ∃e[LittleV(e) ∧ Agent(e,x) ∧ Theme(e,y) ∧ Voice(e,active)]       │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
                         SentenceAnalysis
              (events, participants, temporal structure)
```

**Crate Dependencies:**

```
canopy-pipeline ─┬─► canopy-events ─┬─► canopy-tokenizer ─┬─► canopy-verbnet
                 │                  │                     ├─► canopy-framenet
                 │                  │                     ├─► canopy-wordnet
                 │                  │                     ├─► canopy-propbank
                 │                  │                     └─► canopy-lexicon
                 │                  │
                 │                  └─► canopy-treebank
                 │
                 └─► canopy-core ◄── (shared types: Word, ThetaRole, Event, etc.)
                          ▲
                          │
                     canopy-engine (shared infrastructure: caching, XML parsing)
```

## Crates

| Crate              | Description                               |
| ------------------ | ----------------------------------------- |
| `canopy-pipeline`  | High-level analysis pipeline              |
| `canopy-events`    | Neo-Davidsonian event composition         |
| `canopy-tokenizer` | Tokenization, lemmatization, coordination |
| `canopy-treebank`  | UD treebank parsing and pattern matching  |
| `canopy-verbnet`   | VerbNet verb class engine                 |
| `canopy-framenet`  | FrameNet frame engine                     |
| `canopy-wordnet`   | WordNet synset engine                     |
| `canopy-propbank`  | PropBank semantic role engine             |
| `canopy-lexicon`   | Custom lexicon support                    |
| `canopy-engine`    | Shared engine infrastructure              |
| `canopy-core`      | Core types and utilities                  |
| `canopy-cli`       | Command-line interface                    |

## Example

```bash
# Full semantic analysis pipeline with event composition
cargo run --release -p canopy-pipeline --example event_composition_demo
```

The demo showcases:

- **Layer 1**: VerbNet, FrameNet, WordNet, Treebank, Lemmatization
- **Layer 2**: Neo-Davidsonian events, LittleV primitives, theta roles, voice detection
- **Performance**: Engine loading, per-sentence timing, cache statistics

## Performance

| Operation           | Time                  |
| ------------------- | --------------------- |
| Engine loading      | ~900ms (one-time)     |
| Layer 1 analysis    | 15-22ms per sentence  |
| Layer 2 composition | 78-148μs per sentence |

Cache hit rates improve with lemmatization normalization.

## Requirements

- Rust 1.75+
- ~4GB RAM for full semantic data loading
- Linguistic data files (VerbNet XML, FrameNet XML, WordNet database)

## Documentation

- [Roadmap](docs/ROADMAP.md) — Development milestones and progress
- [Architecture](docs/ARCHITECTURE.md) — System design and data flow
- [Contributing](docs/CONTRIBUTING.md) — Development guidelines
- [Performance](docs/reference/performance.md) — Benchmarks and optimization

## License

MIT — see [LICENSE](LICENSE) for details.

______________________________________________________________________

**Status**: M7 Complete — Layer 2 Event Composition
**Next**: M8 Discourse Representation Theory (DRT)
