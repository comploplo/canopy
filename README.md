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
│  VerbNet:   give-13.1 (Agent, Theme, Recipient)                     │
│  FrameNet:  Giving [Donor→John, Theme→book, Recipient→Mary]         │
│  WordNet:   give.v.01 "transfer possession"                         │
│  PropBank:  give.01 (ARG0=giver, ARG1=thing, ARG2=entity)           │
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
- **High performance** — 50,000-2,000,000+ words/sec depending on engine
- **Pure Rust** — No runtime dependencies, memory-safe, concurrent

## Quick Start

```bash
# Build
cargo build --release

# Download linguistic data
./scripts/setup-data.sh

# Run demo
cargo run -p canopy-resources --example demo --release
```

### Data Setup

Canopy requires linguistic datasets (not included in repo):

| Dataset        | Location                    | Source                                                           |
| -------------- | --------------------------- | ---------------------------------------------------------------- |
| VerbNet 3.4    | `data/verbnet/`             | [GitHub](https://github.com/cu-clear/verbnet)                    |
| FrameNet 1.7   | `data/framenet/`            | [FrameNet](https://framenet.icsi.berkeley.edu/)                  |
| WordNet 3.1    | `data/wordnet/`             | [WordNet](https://wordnet.princeton.edu/)                        |
| PropBank       | `data/propbank/`            | [PropBank](https://propbank.github.io/)                          |
| UD English-EWT | `data/ud_english-ewt/`      | [UniversalDependencies](https://universaldependencies.org/)      |
| Gender names   | `data/name_gender_dataset/` | [UCI ML](https://archive.ics.uci.edu/dataset/591/gender+by+name) |

## Architecture

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
│                                    │                                        │
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

| Operation      | Time    |
| -------------- | ------- |
| Engine loading | ~300ms  |
| Binary cache   | ~50ms   |
| Memory usage   | \<100MB |

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
- ~2GB RAM for full semantic data loading
- Linguistic datasets (see Data Setup above)

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design details
- [Getting Started](docs/GETTING_STARTED.md) — Setup guide
- [Roadmap](docs/ROADMAP.md) — Development milestones

## License

MIT — see [LICENSE](LICENSE) for details.

______________________________________________________________________

**Status**: Production-ready • 3 crates • 80%+ test coverage • All 5 engines operational
