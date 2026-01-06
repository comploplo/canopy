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
│  VerbNet:   give-13.1 → Agent, Theme, Recipient roles               │
│  FrameNet:  Giving frame → Donor, Theme, Recipient elements         │
│  WordNet:   give.v.01 → synset with hypernyms, definitions          │
│  PropBank:  give.01 → ARG0, ARG1, ARG2 structure                    │
│                                                                     │
│  Event:     ∃e[Cause(e) ∧ Agent(e,John) ∧ Theme(e,book)             │
│                        ∧ Recipient(e,Mary)]                         │
│                                                                     │
│  Theta Roles: John=Agent, book=Theme, Mary=Recipient                │
└─────────────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Build
cargo build --release

# Download linguistic data (see Data Setup below)
./scripts/setup-data.sh

# Run the demo
cargo run --example demo --release
```

## Features

### Semantic Engines

Five integrated linguistic resources working together:

| Engine   | Coverage         | Purpose                           |
| -------- | ---------------- | --------------------------------- |
| VerbNet  | 333 verb classes | Syntactic frames and theta roles  |
| FrameNet | 1,200+ frames    | Semantic frame structures         |
| WordNet  | 117,000+ synsets | Lexical relations and definitions |
| PropBank | Full coverage    | Predicate-argument structures     |
| Lexicon  | Patterns + lists | Negation, discourse markers       |

### Event Semantics

Neo-Davidsonian event structures with LittleV decomposition:

- **LittleV Primitives**: Cause, Become, Do, Experience, Go, Have
- **Theta Roles**: Agent, Patient, Theme, Experiencer, Recipient, Goal, Location
- **Voice Detection**: Active, passive, and middle voice recognition
- **Event Composition**: Combines syntactic dependencies with semantic frames

### Discourse Processing

Formal discourse semantics following linguistic theory:

- **DRS Construction** — Discourse Representation Structures (Kamp 1981)
- **Anaphora Resolution** — Salience-based ranking with Binding Theory constraints
- **QUD Tracking** — Question Under Discussion stack and tree (Roberts 1996)
- **Coherence Relations** — SDRT-inspired classification (Narration, Elaboration, Contrast, Explanation)
- **Discourse Moves** — Speech act classification (Assertion, Question, Correction, Acknowledgment)
- **Presupposition Detection** — Trigger identification with accommodation tracking
- **Validation** — Entity state tracking and contradiction detection

### Underspecification

Efficient ambiguity handling without exponential blowup:

- **Packed Semantics** — Share structure across readings (O(n) not O(2^n))
- **Scope Underspecification** — UDRT and MRS-style representations
- **Referential Ambiguity** — Ranked pronoun-antecedent candidates
- **Multiple Disambiguators** — Surprisal, confidence, entropy-based selection

### Surprisal Processing

Information-theoretic analysis following psycholinguistics research:

- **Incremental Processing** — Left-to-right with beam search
- **Garden-Path Detection** — Via surprisal spikes (Hale 2001, Levy 2008)
- **Entropy Reduction** — Track uncertainty reduction across words
- **SurprisalModel Trait** — Pluggable probability models for P(word|context)

### UD Treebank Pattern Matching

VerbNet-aware dependency pattern matching using Universal Dependencies:

- **Semantic Signatures** — Hash-based lookup from lemma + VerbNet class
- **Adaptive Caching** — Core patterns + LRU cache for efficiency
- **UTAH Heuristics** — Dependency-to-role mapping fallback
- **Pattern Statistics** — Cache hit rates, coverage metrics

## Architecture

The `canopy` crate defines abstract provider traits with no knowledge of data formats. The `canopy-resources` crate implements these traits using VerbNet XML, FrameNet XML, WordNet database, etc. This dependency inversion keeps core semantic types independent of resource formats.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CANOPY ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         canopy-cli                                  │   │
│   │                   Command-line interface                            │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        canopy-resources                             │   │
│   │                                                                     │   │
│   │   ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌──────────┐ ┌───────┐  │   │
│   │   │  VerbNet  │ │ FrameNet  │ │  WordNet  │ │ PropBank │ │Lexicon│  │   │
│   │   │ 333 class │ │ 1200 frm  │ │ 117k syn  │ │   roles  │ │pattern│  │   │
│   │   └───────────┘ └───────────┘ └───────────┘ └──────────┘ └───────┘  │   │
│   │                                                                     │   │
│   │   ┌─────────────────────┐  ┌────────────────────────────────────┐   │   │
│   │   │   Pattern Matcher   │  │       Engine Infrastructure        │   │
│   │   │  UD treebank-aware  │  │  Binary cache · O(1) lookup · LRU  │   │
│   │   └─────────────────────┘  └────────────────────────────────────┘   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                    │ implements traits                      │
│                                    ▼                                        │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                            canopy                                   │   │
│   │                                                                     │   │
│   │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│   │   │  Discourse  │  │   Events    │  │ Incremental │  │ Underspec │  │   │
│   │   │ DRS · QUD   │  │  LittleV    │  │  Surprisal  │  │  Packed   │  │   │
│   │   │  Anaphora   │  │ Composition │  │ Beam search │  │ Semantics │  │   │
│   │   └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│   │                                                                     │   │
│   │   Core: ThetaRole · DepRel · UPos · CanopyError · Provider traits   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate              | Description                        | Purpose                  |
| ------------------ | ---------------------------------- | ------------------------ |
| `canopy`           | Core types, traits, kernel modules | Abstract semantic layer  |
| `canopy-resources` | 5 engines + pattern matcher        | Concrete implementations |
| `canopy-cli`       | Command-line interface             | User interaction         |

**~45,000 lines of Rust** across 3 crates

## Performance

Benchmarks on Apple Silicon (M-series), release mode:

### Sentence Analysis

| Operation               | Time   | Notes                            |
| ----------------------- | ------ | -------------------------------- |
| Pipeline initialization | ~750ms | Binary cache (vs 10-15s cold)    |
| Simple sentence         | ~225µs | "John runs quickly."             |
| Ditransitive            | ~25µs  | "Mary gave John a book."         |
| Complex event           | ~65µs  | With modifiers and decomposition |
| 3-sentence document     | ~190µs | Full discourse analysis + DRS    |

### Throughput (Moby Dick, 11,386 lines)

| Metric            | Value           |
| ----------------- | --------------- |
| Per-line analysis | ~440µs          |
| Throughput        | 2,280 lines/sec |
| Total time        | ~5 seconds      |

### Engine Performance

| Engine   | Throughput       | Notes                      |
| -------- | ---------------- | -------------------------- |
| VerbNet  | 51,000 words/sec | 333 verb classes           |
| WordNet  | 77,000 words/sec | 117,000+ synsets           |
| FrameNet | 87,000 words/sec | 1,200+ frames, 13,000+ LUs |
| PropBank | 2.4M words/sec   | Indexed lemma lookup       |

### Resources

| Resource     | Value   |
| ------------ | ------- |
| Binary cache | ~50ms   |
| Peak memory  | ~350 MB |

## Data Setup

Canopy requires linguistic datasets (not included). Each has its own license—check terms before commercial use.

| Dataset        | Location                    | Source                                                           |
| -------------- | --------------------------- | ---------------------------------------------------------------- |
| VerbNet 3.4    | `data/verbnet/`             | [GitHub](https://github.com/cu-clear/verbnet)                    |
| FrameNet 1.7   | `data/framenet/`            | [FrameNet](https://framenet.icsi.berkeley.edu/) (registration)   |
| WordNet 3.1    | `data/wordnet/`             | [WordNet](https://wordnet.princeton.edu/)                        |
| PropBank       | `data/propbank/`            | [PropBank](https://propbank.github.io/)                          |
| UD English-EWT | `data/ud_english-ewt/`      | [UniversalDependencies](https://universaldependencies.org/)      |
| Gender names   | `data/name_gender_dataset/` | [UCI ML](https://archive.ics.uci.edu/dataset/591/gender+by+name) |
| SemLink 2      | `data/semlink/`             | [GitHub](https://github.com/cu-clear/semlink) (git submodule)    |

After cloning, initialize submodules:

```bash
git submodule update --init --recursive
```

## Theoretical Foundations

### Event Semantics

- Neo-Davidsonian event structures (Parsons 1990)
- LittleV decomposition (Hale & Keyser, Ramchand 2008)
- VerbNet-to-primitive mapping

### Thematic Roles

- UTAH: Uniformity of Theta Assignment Hypothesis (Baker 1988)
- Role hierarchy: Agent > Experiencer > Theme > Goal > Location

### Discourse

- DRS: Discourse Representation Theory (Kamp 1981)
- QUD: Question Under Discussion (Roberts 1996)
- Centering Theory (Grosz, Joshi, Weinstein 1995)

### Information Theory

- Surprisal: S(word) = -log₂ P(word|context) (Hale 2001, Levy 2008)
- Entropy reduction for disambiguation

### Lexical Resources

- VerbNet 3.4 (Kipper-Schuler 2005)
- FrameNet 1.7 (Fillmore & Baker 2010)
- WordNet 3.1 (Fellbaum 1998)
- PropBank (Palmer et al. 2005)

## Requirements

- Rust 1.75+
- ~350 MB RAM (all engines loaded)
- Linguistic datasets (see Data Setup)

## Documentation

- [Getting Started](docs/GETTING_STARTED.md) — Setup guide
- [Architecture](docs/ARCHITECTURE.md) — System design
- [Discourse Analysis](docs/DISCOURSE.md) — Coherence, QUD, presuppositions
- [Underspecification](docs/UNDERSPECIFICATION.md) — Packed semantics, disambiguation
- [Surprisal Models](docs/SURPRISAL.md) — Custom language model integration
- [Formal Semantics](docs/FORMAL_SEMANTICS.md) — Theoretical foundations
- [Roadmap](docs/ROADMAP.md) — Development milestones

## License

MIT — see [LICENSE](LICENSE)

______________________________________________________________________

**Status**: Research-grade · 3 crates · 80%+ test coverage · 5 semantic engines · Full discourse analysis
