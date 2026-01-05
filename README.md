# Canopy

**Deep semantic analysis of natural language in Rust**

Canopy builds rich meaning representations from text by combining formal linguistics with high-performance computing. It integrates multiple linguistic resources—VerbNet, FrameNet, WordNet, PropBank—to produce event structures, theta roles, and semantic decompositions.

## What Canopy Does

```
Input: "John gave Mary a book"
                │
                ▼
┌─────────────────────────────────────────────────────────────────────┐
│  SEMANTIC ANALYSIS                                                  │
│                                                                     │
│  VerbNet:   give-13.1 → frames with Agent, Theme, Recipient roles   │
│  FrameNet:  Giving frame → frame elements (Donor, Theme, Recipient) │
│  WordNet:   give.v.01 → synset with hypernyms, definitions          │
│  PropBank:  give.01 → predicate-argument structure (ARG0, ARG1...)  │
│                                                                     │
│  Event:     ∃e[Cause(e) ∧ Agent(e,John) ∧ Theme(e,book)             │
│                        ∧ Recipient(e,Mary)]                         │
│                                                                     │
│  Theta Roles: John=Agent, book=Theme, Mary=Recipient                │
└─────────────────────────────────────────────────────────────────────┘
```

## Features

- **5 semantic engines** — VerbNet, FrameNet, WordNet, PropBank, Lexicon
- **Neo-Davidsonian events** — LittleV primitives (Cause, Become, Do, Experience, Go, Have)
- **Theta role binding** — Agent, Patient, Theme, Experiencer, Recipient, etc.
- **Surprisal-based disambiguation** — Probability-weighted reading selection using information theory
- **High performance** — 50,000-2,000,000+ words/sec depending on engine
- **Pure Rust** — No C/C++ dependencies; requires linguistic data files at runtime

## Quick Start

```bash
# Build
cargo build --release

# Download linguistic data (see Data Setup for licensing)
./scripts/setup-data.sh

# Run the demo
cargo run -p canopy-resources --example demo --release
```

### Data Setup

Canopy requires linguistic datasets (not included in repo). **Note**: Each dataset has its own license. VerbNet, WordNet, and UD treebanks are freely available. FrameNet requires registration. Check each source for terms before commercial use.

| Dataset        | Location                    | Source                                                           |
| -------------- | --------------------------- | ---------------------------------------------------------------- |
| VerbNet 3.4    | `data/verbnet/`             | [GitHub](https://github.com/cu-clear/verbnet)                    |
| FrameNet 1.7   | `data/framenet/`            | [FrameNet](https://framenet.icsi.berkeley.edu/)                  |
| WordNet 3.1    | `data/wordnet/`             | [WordNet](https://wordnet.princeton.edu/)                        |
| PropBank       | `data/propbank/`            | [PropBank](https://propbank.github.io/)                          |
| UD English-EWT | `data/ud_english-ewt/`      | [UniversalDependencies](https://universaldependencies.org/)      |
| Gender names   | `data/name_gender_dataset/` | [UCI ML](https://archive.ics.uci.edu/dataset/591/gender+by+name) |

## Architecture

The core `canopy` crate defines abstract **provider traits** (`SenseProvider`, `RoleProvider`, etc.) with no knowledge of specific data formats. The `canopy-resources` crate implements these traits using VerbNet XML, FrameNet XML, WordNet database, etc. This dependency inversion means the core semantic types are independent of any particular linguistic resource format.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CANOPY ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         canopy-cli                                  │   │
│   │                   Command-line interface & demos                    │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        canopy-resources                             │   │
│   │                                                                     │   │
│   │   ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌──────────┐ ┌───────┐  │   │
│   │   │  VerbNet  │ │ FrameNet  │ │  WordNet  │ │ PropBank │ │Lexicon│  │   │
│   │   │           │ │           │ │           │ │          │ │       │  │   │
│   │   │ 333 verb  │ │  1,200+   │ │  117,000+ │ │ semantic │ │ word  │  │   │
│   │   │ classes   │ │  frames   │ │  synsets  │ │  roles   │ │ lists │  │   │
│   │   └───────────┘ └───────────┘ └───────────┘ └──────────┘ └───────┘  │   │
│   │                                                                     │   │
│   │   ┌─────────────────────────────────────────────────────────────┐   │   │
│   │   │                    Engine Infrastructure                    │   │   │
│   │   │  • Binary caching (50ms load vs 10-50s parse)               │   │   │
│   │   │  • O(1) indexed lookups                                     │   │   │
│   │   │  • LRU query caching                                        │   │   │
│   │   └─────────────────────────────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │ implements traits                      │
│                                    ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                            canopy                                   │   │
│   │                                                                     │   │
│   │   Core Types           Provider Traits         Event Semantics      │   │
│   │   • ThetaRole          • SenseProvider         • LittleVType        │   │
│   │   • DepRel             • RoleProvider          • PredicateDecomp    │   │
│   │   • UPos               • DiscourseCueProvider  • EventStructure     │   │
│   │   • CanopyError                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate              | Description                             | Lines   |
| ------------------ | --------------------------------------- | ------- |
| `canopy`           | Core types, traits, and event semantics | ~3,500  |
| `canopy-resources` | 5 semantic engines + infrastructure     | ~21,000 |
| `canopy-cli`       | Command-line interface and demos        | ~2,000  |

**Total: ~26,500 lines of Rust** (consolidated from 78,000+ lines / 11 crates)

## Performance

Benchmarked on Moby Dick (215,000 words, 3,716 sentences):

| Engine      | Throughput              | Notes                      |
| ----------- | ----------------------- | -------------------------- |
| VerbNet     | 51,000 words/sec        | 333 verb classes           |
| WordNet     | 77,000 words/sec        | 117,000+ synsets           |
| FrameNet    | 87,000 words/sec        | 1,200+ frames, 13,000+ LUs |
| PropBank    | 2.4M words/sec          | Indexed lemma lookup       |
| **Overall** | **84,000 analyses/sec** |                            |

| Operation      | Time/Size |
| -------------- | --------- |
| Engine loading | ~300ms    |
| Binary cache   | ~50ms     |
| Peak memory    | ~1.5-2GB  |

## Semantic Resources

### VerbNet

- 333 verb classes with syntactic frames
- Theta roles: Agent, Patient, Theme, Experiencer, etc.
- Maps to LittleV event primitives

### FrameNet

- 1,200+ semantic frames
- 13,000+ lexical units
- Frame elements and relations

### WordNet

- 117,000+ synsets
- Hypernym/hyponym hierarchies
- Definitions and examples

### PropBank

- Predicate-argument structures
- Semantic role labels (ARG0, ARG1, etc.)
- Sense disambiguation

### Lexicon

- Negation words and patterns (un-, dis-, in-)
- Discourse markers
- Stop words, quantifiers, modals

## Surprisal and Underspecification

Canopy implements information-theoretic disambiguation following academic psycholinguistics:

**Surprisal Theory** (Hale 2001, Levy 2008)

- Processing difficulty = -log₂ P(word | context)
- Incremental left-to-right processing with beam search
- Garden-path detection via surprisal spikes
- Entropy reduction tracking

**Underspecified Representations**

- Packed semantics share structure across readings (O(n) memory, not O(2^n))
- UDRT-style underspecified DRS with scope constraints (Reyle 1993)
- MRS-style handle-based scope underspecification (Copestake et al. 2005)
- Referential ambiguity with ranked candidates (Centering Theory, Grosz et al. 1995)

**Disambiguators**

- `MinSurprisalDisambiguator` — Select lowest surprisal (highest probability) reading
- `ConfidenceDisambiguator` — Use provider confidence scores (legacy)
- `HybridDisambiguator` — Weighted combination of surprisal + confidence
- `EntropyReductionDisambiguator` — Prefer readings that reduce uncertainty
- `InteractiveDisambiguator` — Return all readings for downstream selection

## Theoretical Foundations

**Event Semantics**

- Neo-Davidsonian event structures (Parsons 1990)
- LittleV decomposition (Hale & Keyser, Ramchand 2008)
- VerbNet-to-primitive mapping

**Thematic Roles**

- UTAH: Universal Theta Assignment Hypothesis (Baker 1988)
- Role hierarchy: Agent > Experiencer > Theme > Goal > Location

**Lexical Resources**

- VerbNet 3.4 (Kipper-Schuler 2005)
- FrameNet 1.7 (Fillmore & Baker 2010)
- WordNet 3.1 (Fellbaum 1998)
- PropBank (Palmer et al. 2005)

## Requirements

- Rust 1.75+
- ~2GB RAM for full semantic data loading (all engines loaded simultaneously)
- Linguistic datasets with their own licenses (see Data Setup)

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design details
- [Getting Started](docs/GETTING_STARTED.md) — Setup guide
- [Roadmap](docs/ROADMAP.md) — Development milestones

## License

MIT — see [LICENSE](LICENSE) for details.

______________________________________________________________________

**Status**: Research-grade • 3 crates • 80%+ test coverage • All 5 engines operational
