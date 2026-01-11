# Changelog

All notable changes to Canopy will be documented in this file.

> **Historical versions (M1-M6)**: See [archive/CHANGELOG_historical.md](archive/CHANGELOG_historical.md)

______________________________________________________________________

## [Unreleased]

### LSP Diagnostic Enhancements

Rich diagnostics surfacing kernel-derived semantic information:

#### New Diagnostic Types

| Code                 | Severity | Description                                           |
| -------------------- | -------- | ----------------------------------------------------- |
| `pronoun-ambiguous`  | Info     | Pronoun has multiple candidate antecedents            |
| `pronoun-unresolved` | Warning  | No accessible antecedent found                        |
| `binding-violation`  | Hint     | Binding theory constraint violation (Condition A/B/C) |
| `scope-ambiguous`    | Info     | Quantifier scope ambiguity detected                   |
| `conflict-detail`    | Warning  | Enhanced conflict info with type and conditions       |

#### Enhanced Hover Content

- **Pronoun binding**: Shows resolved antecedent, candidates with confidence, violations
- **Sense derivation**: Why this sense was selected, runner-up, selection reason
- **Logical form**: DRS conditions related to the predicate (for verbs)

#### Code Actions

Each diagnostic has corresponding quick-fix actions for exploration and disambiguation.

### Semantic Tree Visualization

Pretty-printed tree output using `ptree` for semantic analysis results:

```
Sentence: "John gave Mary a book."
├── Syntax
│   ├── John [Propn] ─ Nsubj
│   ├── gave [Verb] ─ Root
│   └── ...
├── Event 1: give (Cause)
│   ├── Aspect: Accomplishment
│   ├── Voice: Active
│   └── Participants
│       ├── Agent: "John" (95%)
│       └── Theme: "book" (92%)
└── Role Bindings
    └── ...
```

#### Tree Types

- `build_sentence_tree()` — Full analysis with syntax, events, decompositions
- `build_document_tree()` — Multi-sentence with coherence and discourse moves
- `build_dependency_tree()` — Hierarchical dependency parse
- `build_compact_event_tree()` — Minimal event view
- `format_drs_box()` — Classic box notation for DRS

### Sense Disambiguation Improvements

- **Lemma-based disambiguation**: Predicates disambiguated when lemma matches exactly one sense ID
- Example: "try" with senses ["try-61.1", "attempt-61.1"] → selects "try-61.1" as unambiguous

### MWE Integration & Improved Lemmatization

#### Multi-Word Expression Support

- **Phrasal verb detection**: `PhrasalVerbDetector` identifies verb-particle constructions (give up, look up)
- **MWE types**: Support for compound nouns, flat names, fixed expressions via `MweDetector`
- **Gerund classification**: `GerundClassifier` distinguishes nominal, verbal, and adjectival gerunds
- **AnnotatedSyntax extended**: New `phrasal_verbs` and `mwes` fields store detected MWEs
- **Predicate decomposition**: Decomposer now uses phrasal lemmas ("give_up") for VerbNet lookup

#### Treebank-Based Lemmatization

- **WordLemmaIndex**: New index extracts form→lemma mappings from UD treebank
- **Irregular verb support**: Correctly lemmatizes "gave"→"give", "went"→"go", etc.
- **ResourceBackedTagger**: Uses treebank lemmas first, suffix heuristics as fallback

#### Engine Improvements

- **LemmaQuery**: Unified query interface for semantic engines with calibrated confidence
- **ArgumentBinder**: Restructured role binding with proper confidence scoring
- **PredicateDecomposer**: Clean decomposition pipeline using phrasal lemmas

______________________________________________________________________

## Current Status

| Metric           | Value    |
| ---------------- | -------- |
| Test coverage    | 81%+     |
| Rust lines       | ~60,000  |
| Crates           | 4        |
| Semantic engines | 5        |
| Pipeline init    | ~750ms   |
| Per-sentence     | 30-200μs |

### Crates

| Crate              | Lines   | Description                                                        |
| ------------------ | ------- | ------------------------------------------------------------------ |
| `canopy`           | ~24,000 | Core kernel: events, discourse, logic, underspec                   |
| `canopy-resources` | ~28,000 | Engines (VerbNet, FrameNet, WordNet, PropBank, Lexicon) + pipeline |
| `canopy-lsp`       | ~4,000  | Language Server Protocol implementation                            |
| `canopy-cli`       | ~600    | Command-line interface                                             |

______________________________________________________________________

## Links

- [Architecture](ARCHITECTURE.md)
- [Roadmap](ROADMAP.md)
- [Getting Started](GETTING_STARTED.md)
