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

## Query & Reasoning Output

```
Document: "John gave Mary a book. She read it carefully."

Query: Who gave Mary something?
Answer: John
  Proof: S1 give(e1) with Agent(e1,John), Recipient(e1,Mary), Theme(e1,book)

Query: What did Mary do?
Answer: read
  Proof: S2 read(e2) with Agent(e2,Mary), Theme(e2,book)
         Resolved: "She" → Mary, "it" → book

Consistency: ✓ No conflicts detected
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
| PropBank | Indexed rolesets | Predicate-argument structures     |
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

### Logical Reasoning

Executable logic layer for inference over DRS:

- **Query Answering** — Yes/no, wh-questions, existence checks with bindings
- **Entailment Checking** — Does the discourse entail a proposition?
- **Consistency Checking** — Detect polarity conflicts and temporal cycles
- **Explanations** — Sentence-level provenance for all answers
- **Reasoning Modes** — Open-world (Unknown) and closed-world (absence → false) for QA

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
│   │   │   Pattern Matcher   │  │       Engine Infrastructure        │   │   │
│   │   │  UD treebank-aware  │  │  Binary cache · O(1) lookup · LRU  │   │   │
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

| Operation               | Time   | Notes                                 |
| ----------------------- | ------ | ------------------------------------- |
| Pipeline initialization | ~750ms | Cache deserialization + engine wiring |
| First sentence          | ~225µs | Cold caches, full analysis path       |
| Subsequent sentences    | ~25µs  | Warm caches, cached patterns          |
| Complex event           | ~65µs  | With modifiers and decomposition      |
| 3-sentence document     | ~190µs | Full discourse analysis + DRS         |

*Times vary based on cache state. First analysis is slower due to pattern matching warmup.*

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

| Dataset        | Location                    | Source                                                             |
| -------------- | --------------------------- | ------------------------------------------------------------------ |
| VerbNet 3.4    | `data/verbnet/`             | [GitHub](https://github.com/cu-clear/verbnet)                      |
| FrameNet 1.7   | `data/framenet/`            | [FrameNet](https://framenet.icsi.berkeley.edu/) (registration)     |
| WordNet 3.1    | `data/wordnet/`             | [WordNet](https://wordnet.princeton.edu/)                          |
| PropBank       | `data/propbank/`            | [PropBank](https://propbank.github.io/)                            |
| UD English-EWT | `data/ud_english-ewt/`      | [UniversalDependencies](https://universaldependencies.org/)        |
| Gender names   | `data/name_gender_dataset/` | [UCI ML](https://archive.ics.uci.edu/dataset/591/gender+by+name) ¹ |
| SemLink 2      | `data/semlink/`             | [GitHub](https://github.com/cu-clear/semlink) (git submodule)      |

¹ *Gender names: weak heuristic for pronoun resolution only; can be disabled; not a model of identity; expect errors/bias.*

After cloning, initialize submodules:

```bash
git submodule update --init --recursive
```

## Theoretical Foundations

- **Events** — Neo-Davidsonian structures + LittleV decomposition (Parsons, Ramchand)
- **Roles** — UTAH theta assignment (Baker 1988)
- **Discourse** — DRT (Kamp), QUD (Roberts), Centering (Grosz et al.)
- **Information** — Surprisal theory (Hale, Levy)
- **Resources** — VerbNet, FrameNet, WordNet, PropBank

See [Formal Semantics](docs/FORMAL_SEMANTICS.md) for full theoretical background.

## Limitations

- **Not a parser** — Relies on UD parses or external syntax; does not parse raw text
- **Not a world model** — Symbolic reasoning only; no grounded perception or commonsense KB
- **Quantifiers improving** — Event/argument structure is strongest; generalized quantifiers in progress
- **No hallucinated inferences** — Discourse reasoning is symbolic; won't bridge beyond explicit content
- **Research-grade** — Not production-hardened; expect rough edges

## Why Rust?

- **Type system** — Rust's enums and traits model linguistic categories (theta roles, dependency relations, semantic frames) with compile-time guarantees that catch category errors early
- **Performance** — Zero-cost abstractions, deterministic execution, parallel pipeline stages without GC pauses

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
