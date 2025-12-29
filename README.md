# Canopy

**Deep semantic analysis of natural language in Rust**

Canopy builds rich meaning representations from text by combining formal linguistics with high-performance computing. It integrates multiple linguistic resources—VerbNet, FrameNet, WordNet, PropBank—to produce event structures, discourse representations, and resolved anaphora.

## What Canopy Does

```
"John saw Mary. He waved to her."
         │
         ▼
┌─────────────────────────────────────────────────────────┐
│  Layer 1: Semantic Analysis                             │
│  • "saw" → VerbNet see-30.1, FrameNet Perception        │
│  • Theta roles: Agent(John), Theme(Mary)                │
│  • WordNet synsets, dependency structure                │
├─────────────────────────────────────────────────────────┤
│  Layer 2: Event Composition                             │
│  • ∃e[Experience(e) ∧ Experiencer(e,John) ∧             │
│       Stimulus(e,Mary) ∧ Past(e)]                       │
├─────────────────────────────────────────────────────────┤
│  Layer 3: Discourse & Anaphora                          │
│  • DRS: [x,y | John(x), Mary(y), saw(x,y), waved(x,y)]  │
│  • "He" → John (gender agreement, Condition B)          │
│  • "her" → Mary (cross-clause binding)                  │
└─────────────────────────────────────────────────────────┘
```

## Features

- **Three-layer semantic pipeline** — lexical analysis → event composition → discourse representation
- **Modern binding theory** — Reinhart & Reuland's reflexivity, Charnavel's logophoricity
- **Anaphora resolution** — pronoun binding with gender agreement (147k name dataset)
- **Neo-Davidsonian events** — LittleV primitives (Cause, Become, Do, Experience, ...)
- **Multi-engine coordination** — VerbNet, FrameNet, WordNet, PropBank, UD treebanks
- **Production performance** — ~19ms per sentence, intelligent caching
- **Pure Rust** — no runtime dependencies, memory-safe, concurrent

## Quick Start

```bash
git clone https://github.com/yourusername/canopy
cd canopy
cargo build --release

# Run the event composition demo
cargo run --release -p canopy-pipeline --example event_composition_demo
```

### Data Setup

Canopy requires linguistic datasets (not included in repo):

| Dataset        | Location               | Source                                                           |
| -------------- | ---------------------- | ---------------------------------------------------------------- |
| VerbNet        | `data/verbnet/`        | [GitHub](https://github.com/cu-clear/verbnet)                    |
| FrameNet       | `data/framenet/`       | [FrameNet](https://framenet.icsi.berkeley.edu/)                  |
| WordNet        | `data/wordnet/`        | [WordNet](https://wordnet.princeton.edu/)                        |
| UD English-EWT | `data/ud_english-ewt/` | [UniversalDependencies](https://universaldependencies.org/)      |
| Gender names   | `data/canopy-lexicon/` | [UCI ML](https://archive.ics.uci.edu/dataset/591/gender+by+name) |

## Architecture

```
                              Input Text
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  LAYER 1: Lexical Semantics                            canopy-tokenizer  │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                    SemanticCoordinator                             │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │  │
│  │  │ VerbNet  │ │ FrameNet │ │ WordNet  │ │ PropBank │ │ Treebank │  │  │
│  │  │ 333 verb │ │ 1,200+   │ │ 117,000+ │ │ semantic │ │ UD deps  │  │  │
│  │  │ classes  │ │ frames   │ │ synsets  │ │ roles    │ │ patterns │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  LAYER 2: Event Composition                              canopy-events   │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  EventComposer: VerbNet → Neo-Davidsonian event structures         │  │
│  │  • LittleV primitives: Cause, Become, Do, Experience, Go, ...      │  │
│  │  • Theta role binding: Agent, Patient, Theme, Experiencer, ...     │  │
│  │  • Voice/aspect detection from dependency patterns                 │  │
│  │                                                                    │  │
│  │  Output: ∃e[LittleV(e) ∧ Agent(e,x) ∧ Theme(e,y) ∧ Aspect(e)]      │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  LAYER 3: Discourse & Binding                          canopy-discourse  │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  Discourse Representation Theory (Kamp & Reyle)                    │  │
│  │  • DRS construction: referents, conditions, subordination          │  │
│  │  • Cross-sentence context tracking                                 │  │
│  │                                                                    │  │
│  │  Modern Binding Theory (Reuland 2011, Charnavel 2019)              │  │
│  │  • Condition B: reflexive predicates must be reflexive-marked      │  │
│  │  • Logophoric contexts: attitude holders, empathy loci             │  │
│  │  • Gender agreement via 147k name-gender dataset                   │  │
│  │                                                                    │  │
│  │  Output: DRS with resolved anaphora and temporal relations         │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate              | Description                                           |
| ------------------ | ----------------------------------------------------- |
| `canopy-pipeline`  | High-level analysis orchestration                     |
| `canopy-discourse` | **Layer 3**: DRT, anaphora resolution, binding theory |
| `canopy-events`    | **Layer 2**: Neo-Davidsonian event composition        |
| `canopy-tokenizer` | **Layer 1**: Semantic coordination, lemmatization     |
| `canopy-treebank`  | UD treebank parsing and pattern matching              |
| `canopy-verbnet`   | VerbNet verb class engine (333 classes)               |
| `canopy-framenet`  | FrameNet frame engine (1,200+ frames)                 |
| `canopy-wordnet`   | WordNet synset engine (117k+ synsets)                 |
| `canopy-propbank`  | PropBank semantic role engine                         |
| `canopy-lexicon`   | Custom lexicon support                                |
| `canopy-engine`    | Shared infrastructure (caching, XML)                  |
| `canopy-core`      | Core types (Word, Event, ThetaRole)                   |

## Performance

| Operation           | Time              |
| ------------------- | ----------------- |
| Engine loading      | ~900ms (one-time) |
| Layer 1 analysis    | 15-22ms/sentence  |
| Layer 2 composition | 78-148μs/sentence |
| Layer 3 discourse   | \<1ms/sentence    |

## Theoretical Foundations

Canopy implements insights from formal semantics and theoretical linguistics:

**Event Semantics**

- Neo-Davidsonian event structures (Parsons 1990)
- LittleV decomposition (Hale & Keyser, Ramchand 2008)
- VerbNet-to-primitive mapping

**Binding Theory**

- Reinhart & Reuland (1993) "Reflexivity" — predicates, not anaphors, are reflexive-marked
- Reuland (2011) *Anaphora and Language Design* — binding from agreement, not c-command
- Charnavel (2019) *Locality and Logophoricity* — exempt anaphors via perspective centers

**Discourse Semantics**

- Kamp & Reyle (1993) *From Discourse to Logic* — DRT foundations
- Temporal relations from aspectual class (Vendler, Dowty)

## Requirements

- Rust 1.75+
- ~4GB RAM for full semantic data loading
- Linguistic datasets (see Data Setup above)

## Documentation

- [Roadmap](docs/ROADMAP.md) — Development milestones and progress
- [Architecture](docs/ARCHITECTURE.md) — System design details
- [Contributing](CONTRIBUTING.md) — Development guidelines

## License

MIT — see [LICENSE](LICENSE) for details.

______________________________________________________________________

**Status**: Layer 3 (DRT & Binding) implemented • Layer 1-2-3 pipeline operational
