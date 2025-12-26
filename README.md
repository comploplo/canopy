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
Input Sentence
     │
     ▼
┌─────────────────────────────────────────────────────────┐
│  Layer 1: Semantic Analysis                             │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │ VerbNet │ │FrameNet │ │ WordNet │ │ Lexicon │       │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘       │
│       └──────────┬┴──────────┬┴───────────┘            │
│                  ▼                                      │
│           Unified Semantic Roles                        │
└─────────────────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│  Layer 2: Event Composition                             │
│  Neo-Davidsonian events, theta binding, voice analysis  │
└─────────────────────────────────────────────────────────┘
                   │
                   ▼
            Event Structure
```

## Crates

| Crate | Description |
|-------|-------------|
| `canopy-pipeline` | High-level analysis pipeline |
| `canopy-events` | Neo-Davidsonian event composition |
| `canopy-tokenizer` | Tokenization, lemmatization, coordination |
| `canopy-treebank` | UD treebank parsing and pattern matching |
| `canopy-verbnet` | VerbNet verb class engine |
| `canopy-framenet` | FrameNet frame engine |
| `canopy-wordnet` | WordNet synset engine |
| `canopy-propbank` | PropBank semantic role engine |
| `canopy-lexicon` | Custom lexicon support |
| `canopy-engine` | Shared engine infrastructure |
| `canopy-core` | Core types and utilities |
| `canopy-cli` | Command-line interface |

## Examples

```bash
# Full pipeline with event composition
cargo run --release -p canopy-pipeline --example event_composition_demo

# Layer 1 semantic analysis
cargo run --release -p canopy-tokenizer --example semantic_layer1_demo

# Treebank pattern visualization
cargo run --release -p canopy-treebank --example tree_visualization_demo

# PropBank analysis
cargo run --release -p canopy-propbank --example propbank_demo
```

## Performance

| Operation | Time |
|-----------|------|
| Engine loading | ~900ms (one-time) |
| Layer 1 analysis | 15-22ms per sentence |
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

---

**Status**: M7 Complete — Layer 2 Event Composition
**Next**: M8 Discourse Representation Theory (DRT)
