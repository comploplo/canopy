# Canopy

**Deep semantic analysis of natural language in Rust**

Canopy builds rich meaning representations from text by combining formal linguistics with high-performance computing. It integrates multiple linguistic resources—VerbNet, FrameNet, WordNet, PropBank—to produce event structures, discourse representations, and resolved anaphora.

## What Canopy Does

```
"John must have seen Mary. He waved to her."
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: Lexical Semantics                    canopy-tokenizer│
│  • "seen" → VerbNet see-30.1, FrameNet Perception           │
│  • "must" → Modal auxiliary (epistemic necessity)           │
│  • WordNet synsets, UD dependency structure, PropBank       │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Event Composition                     canopy-events│
│  • ∃e[Experience(e) ∧ Experiencer(e,John) ∧ Stimulus(e,Mary)]│
│  • Modality: force=Necessity, flavor=Epistemic              │
│  • Polarity: affirmative, Presuppositions: []               │
│  • Plurality: John=Singular, Mary=Singular                  │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Discourse & Binding                canopy-discourse│
│  • DRS: [x,y,e1,e2 | John(x), Mary(y), see(e1), wave(e2)]   │
│  • Temporal: e1 Before e2 (Allen's interval algebra)        │
│  • "He" → John (Binding Theory, gender agreement)           │
│  • "her" → Mary (cross-clause, Centering Theory)            │
│  • Coherence: Narration relation between sentences          │
└─────────────────────────────────────────────────────────────┘
```

## Features

### Semantic Pipeline

- **Three-layer architecture** — lexical → event composition → discourse
- **Neo-Davidsonian events** — LittleV primitives (Cause, Become, Do, Experience, Go, Have, ...)
- **Multi-engine coordination** — VerbNet, FrameNet, WordNet, PropBank, UD treebanks

### Event Semantics (Layer 2)

- **Kratzerian modality** — modal force (necessity/possibility) + 5 flavors (epistemic, deontic, circumstantial, bouletic, teleological)
- **Presupposition detection** — factive, aspectual, definite triggers via VerbNet class patterns
- **Negation scope** — neg-raising detection for believe/want/think class verbs
- **Plurality inference** — semantic number + distributivity (collective vs distributive)

### Discourse Semantics (Layer 3)

- **DRT** — Discourse Representation Theory (Kamp & Reyle)
- **Temporal reasoning** — Allen's 13 interval relations with aspectual inference
- **Centering Theory** — topic continuity tracking (Grosz, Joshi & Weinstein)
- **Coherence relations** — narration, result, contrast, elaboration (Hobbs, Asher & Lascarides)
- **Modern binding theory** — Reinhart & Reuland's reflexivity, Charnavel's logophoricity
- **Anaphora resolution** — pronoun binding with gender agreement (147k name dataset)

### Performance & Quality

- **Production performance** — ~19ms per sentence end-to-end
- **Real linguistic data** — no stubs, 333 VerbNet classes, 117k+ WordNet synsets
- **Pure Rust** — no runtime dependencies, memory-safe, concurrent

## Quick Start

```bash
# Build
cargo build --release

# Download linguistic data
./scripts/setup-data.sh

# Run the demo (analyzes 100 sentences from Moby Dick)
cargo run --release --example demo
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
│  │  EventComposer Pipeline (7 stages):                                │  │
│  │  1. Predicate identification (verbs from L1)                       │  │
│  │  2. LittleV decomposition: Cause, Become, Do, Experience, Go, ...  │  │
│  │  3. Theta role binding: Agent, Patient, Theme, Experiencer, ...    │  │
│  │  4. Modality resolution: force (necessity/possibility) + flavor    │  │
│  │  5. Negation scope: polarity + neg-raising                         │  │
│  │  6. Presupposition detection: factive, aspectual, definite         │  │
│  │  7. Plurality inference: semantic number + distributivity          │  │
│  │                                                                    │  │
│  │  Output: ComposedEvent with modality, presuppositions, polarity    │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌──────────────────────────────────────────────────────────────────────────┐
│  LAYER 3: Discourse & Binding                          canopy-discourse  │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  Discourse Representation Theory (Kamp & Reyle 1993)               │  │
│  │  • DRS: universe of referents + conditions                         │  │
│  │  • Cross-sentence context and subordination                        │  │
│  │                                                                    │  │
│  │  Temporal Reasoning (Allen 1983, Dowty 1986)                       │  │
│  │  • 13 Allen interval relations (before, meets, overlaps, ...)      │  │
│  │  • Aspectual class inference from tense/aspect                     │  │
│  │                                                                    │  │
│  │  Centering Theory (Grosz, Joshi & Weinstein 1995)                  │  │
│  │  • Forward/backward-looking centers for topic tracking             │  │
│  │  • Transition types: Continue, Retain, Shift                       │  │
│  │                                                                    │  │
│  │  Coherence Relations (Hobbs 1979, Asher & Lascarides 2003)         │  │
│  │  • Result, Narration, Background, Contrast, Elaboration            │  │
│  │                                                                    │  │
│  │  Binding Theory (Reuland 2011, Charnavel 2019)                     │  │
│  │  • Condition B, logophoric contexts, gender agreement (147k names) │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate                     | Description                                                                   |
| ------------------------- | ----------------------------------------------------------------------------- |
| `canopy-pipeline`         | High-level analysis orchestration                                             |
| `canopy-discourse`        | **Layer 3**: DRT, anaphora resolution, binding theory                         |
| `canopy-events`           | **Layer 2**: Neo-Davidsonian event composition                                |
| `canopy-tokenizer`        | **Layer 1**: Semantic coordination, lemmatization                             |
| `canopy-treebank`         | UD treebank parsing and pattern matching                                      |
| `canopy-semantic-engines` | Consolidated semantic engines (VerbNet, FrameNet, WordNet, PropBank, Lexicon) |
| `canopy-engine`           | Shared infrastructure (caching, traits, errors)                               |
| `canopy-core`             | Core types (Word, Event, ThetaRole, CanopyError)                              |

## Performance

| Operation      | Time              |
| -------------- | ----------------- |
| Engine loading | ~900ms (one-time) |
| Full analysis  | ~19ms/sentence    |
| Cache hit rate | ~60%+ (improves)  |
| Memory usage   | \<100MB typical   |

## Theoretical Foundations

Canopy implements insights from formal semantics and theoretical linguistics:

**Event Semantics**

- Neo-Davidsonian event structures (Parsons 1990)
- LittleV decomposition (Hale & Keyser, Ramchand 2008)
- VerbNet-to-primitive mapping

**Modality & Presupposition**

- Kratzerian modal semantics — force/flavor distinction (Kratzer 1981, 1991)
- Presupposition triggers — factive, aspectual, definite (Beaver & Geurts 2014)
- Neg-raising predicates — want/believe/think class verbs

**Binding Theory**

- Reinhart & Reuland (1993) "Reflexivity" — predicates, not anaphors, are reflexive-marked
- Reuland (2011) *Anaphora and Language Design* — binding from agreement, not c-command
- Charnavel (2019) *Locality and Logophoricity* — exempt anaphors via perspective centers

**Discourse Semantics**

- Kamp & Reyle (1993) *From Discourse to Logic* — DRT foundations
- Allen (1983) — temporal interval algebra
- Grosz, Joshi & Weinstein (1995) — Centering Theory
- Hobbs (1979), Asher & Lascarides (2003) — coherence relations

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

**Status**: All three layers complete • M7 semantic enrichment (modality, presupposition, plurality) • M8 discourse features (DRT, temporal, centering, coherence)
