# Canopy Architecture - Complete Three-Layer Semantic Pipeline

## Overview

Canopy implements a **semantic-first linguistic analysis architecture** with clean layer separation. The system provides a complete three-layer pipeline: lexical semantics (Layer 1), event composition (Layer 2), and discourse representation (Layer 3).

### Core Architecture (Current: All Layers Complete)

```text
Text → Layer 1 → Layer 2 → Layer 3
       ↓          ↓          ↓
   [Lexical]  [Events]  [Discourse]
       ↓          ↓          ↓
  VerbNet    LittleV     DRT +
  FrameNet   Modality    Temporal +
  WordNet    Negation    Centering +
  PropBank   Presup      Coherence +
  Treebank   Plurality   Binding
```

**Key Innovation**: Three cleanly separated layers with rich semantic output at each stage.

## Current Status: M8 COMPLETE ✅

### Achievement Summary

- **Layer 1 Complete**: Lexical semantics with 5 parallel engines
- **Layer 2 Complete**: Neo-Davidsonian events with modality, presupposition, plurality
- **Layer 3 Complete**: DRT, temporal reasoning, centering, coherence, binding theory
- **Performance**: ~19ms per sentence end-to-end
- **Real Data**: 333 VerbNet classes, 117k+ WordNet synsets, 147k name-gender pairs

### Implementation Status

- ✅ **Layer 1 - Lexical Semantics**: VerbNet, FrameNet, WordNet, PropBank, Treebank
- ✅ **Layer 2 - Event Composition**: 7-stage pipeline with semantic enrichment
- ✅ **Layer 3 - Discourse**: DRT, temporal, centering, coherence, binding
- ✅ **Production Performance**: ~19ms/sentence with intelligent caching
- ✅ **Anti-Stub Architecture**: Real data loading only, no fake engines

## Architecture Components

### Crate Structure (11 crates)

```text
canopy/
├── crates/
│   ├── canopy-core/              # Fundamental types + unified CanopyError
│   ├── canopy-engine/            # Base engine traits & caching infrastructure
│   │
│   ├── canopy-semantic-engines/  # All semantic engines (consolidated)
│   │   ├── verbnet/              # VerbNet XML (333 classes)
│   │   ├── framenet/             # FrameNet XML (1200+ frames)
│   │   ├── wordnet/              # WordNet database (117k+ synsets)
│   │   ├── propbank/             # PropBank semantic roles
│   │   └── lexicon/              # Custom lexicon + 147k gender names
│   │
│   ├── canopy-tokenizer/         # LAYER 1: Lexical semantics
│   │   └── SemanticCoordinator   # Parallel engine orchestration
│   │
│   ├── canopy-treebank/          # UD treebank patterns + POS
│   │
│   ├── canopy-events/            # LAYER 2: Event composition
│   │   ├── composer.rs           # 7-stage EventComposer pipeline
│   │   ├── modality.rs           # Kratzerian modal resolution
│   │   ├── negation.rs           # Negation scope + neg-raising
│   │   ├── presupposition.rs     # VerbNet class-based detection
│   │   └── plurality.rs          # Semantic number + distributivity
│   │
│   ├── canopy-discourse/         # LAYER 3: Discourse semantics
│   │   ├── drs.rs                # Discourse Representation Structures
│   │   ├── coherence.rs          # Coherence relations + centering
│   │   └── context.rs            # Discourse context tracking
│   │
│   ├── canopy-pipeline/          # High-level orchestration
│   └── canopy-cli/               # Command-line interface
│
├── examples/
│   └── demo.rs                   # Main demo (cargo run --example demo)
│
├── data/                         # Linguistic resources
│   ├── verbnet/                  # VerbNet 3.4 XML files
│   ├── framenet/                 # FrameNet v15 frames + LUs
│   ├── wordnet/                  # WordNet 3.1 database
│   ├── propbank/                 # PropBank frames
│   ├── ud_english-ewt/           # UD English Web Treebank
│   └── canopy-lexicon/           # Gender-by-name dataset
│
└── docs/                         # Documentation
```

## Layer 1: Lexical Semantics (canopy-tokenizer)

```rust
use canopy_pipeline::create_l1_analyzer_with_treebank;

// Create analyzer (loads all engines)
let l1 = create_l1_analyzer_with_treebank()?;

// Analyze a sentence
let result = l1.analyze_sentence("The captain saw the whale.")?;

// Access token data
for token in &result.tokens {
    println!("{}/{:?}", token.original_word, token.pos);
    if let Some(vn) = &token.verbnet {
        println!("  VerbNet: {:?}", vn.verb_classes);
    }
}

// Token structure
pub struct Layer1SemanticResult {
    original_word: String,
    lemma: String,
    pos: Option<UPos>,
    verbnet: Option<VerbNetAnalysis>,   // 333 verb classes
    framenet: Option<FrameNetAnalysis>, // 1200+ frames
    wordnet: Option<WordNetAnalysis>,   // 117k+ synsets
    propbank: Option<PropBankAnalysis>, // semantic roles
    treebank: Option<TreebankAnalysis>, // UD dependencies
    confidence: f32,
}
```

**Engines**: VerbNet, FrameNet, WordNet, PropBank, Treebank (parallel execution)

## Layer 2: Event Composition (canopy-events)

```rust
use canopy_events::EventComposer;

// Create composer
let l2 = EventComposer::new()?;

// Compose events from Layer 1 analysis
let events = l2.compose_sentence(&sentence_analysis)?;

for event in &events.events {
    println!("{} : {}", event.event.little_v, event.event.predicate);
    for (role, entity) in &event.event.participants {
        println!("  {} = \"{}\"", role, entity.text);
    }
}

// Event structure
pub struct ComposedEvent {
    event: Event,                           // Core event structure
    presuppositions: Vec<Presupposition>,   // Triggered presuppositions
    polarity: bool,                         // Affirmative or negated
}

pub struct Event {
    little_v: LittleV,                      // Cause, Become, Do, Experience, Go, ...
    participants: HashMap<ThetaRole, Entity>,
    modality: Option<EventModality>,        // Force + 5 flavors
    aspect: AspectualClass,
}
```

### Layer 2 Features

**LittleV Decomposition**: VerbNet predicates → primitives

- `cause` → Cause, `motion` → Go, `transfer` → Cause(Have), `state` → Be, `experience` → Experience

**Kratzerian Modality**:

- Force: Necessity (must, have to) vs Possibility (can, may, might)
- Flavors: Epistemic, Deontic, Circumstantial, Bouletic, Teleological

**Presupposition Detection** (VerbNet class-based, no hardcoded word lists):

- Factive: admire-31.2, marvel-31.3, discover-84, comprehend-87.2
- Aspectual: stop-55.4, continue-55.3, begin-55.1

**Negation Scope**:

- Standard negation → polarity: false
- Neg-raising (want-32.1, conjecture-29.5) → negation raised to complement

**Plurality Inference**:

- Semantic number: Singular, Plural, Mass
- Distributivity: Collective ("boys gathered") vs Distributive ("boys each ran")

## Layer 3: Discourse Semantics (canopy-discourse)

```rust
use canopy_pipeline::DiscourseProcessor;

// Create discourse processor
let mut l3 = DiscourseProcessor::new();

// Process sentences to build DRS
for (sentence, events) in sentences {
    l3.process_sentence(&sentence, &events)?;
}

// Access the Discourse Representation Structure
let drs = l3.drs();
println!("Entities: {:?}", drs.universe.iter()
    .filter(|(_, r)| !r.is_event)
    .map(|(id, r)| (id, &r.name))
    .collect::<Vec<_>>());
```

### Layer 3 Features

**Discourse Representation Theory** (Kamp & Reyle 1993):

- Universe: discourse referents (entities + events)
- Conditions: predicates and relations over referents
- Subordination: embedded DRSs for modals, conditionals

**Temporal Reasoning** (Allen 1983):

- 13 interval relations: Before, Meets, Overlaps, Starts, During, Finishes, Equals + inverses
- Inference from tense/aspect (Dowty 1986): Past perfect → Before, State + Achievement → Overlaps

**Centering Theory** (Grosz, Joshi & Weinstein 1995):

- Forward-looking centers (Cf): ranked by salience
- Backward-looking center (Cb): current topic
- Transitions: Continue, Retain, SmoothShift, RoughShift

**Coherence Relations** (Hobbs 1979, Asher & Lascarides 2003):

- Causal: Result, Explanation
- Temporal: Narration, Background
- Similarity: Parallel, Contrast
- Elaboration: Detail, Exemplification

**Binding Theory** (Reuland 2011, Charnavel 2019):

- Condition B: reflexive predicates must be reflexive-marked
- Logophoric contexts: attitude holders, empathy loci
- Gender agreement: 147k name-gender dataset

## Performance

| Operation           | Time               |
| ------------------- | ------------------ |
| Engine loading      | ~900ms (one-time)  |
| Layer 1 analysis    | 15-22ms/sentence   |
| Layer 2 composition | 78-148μs/sentence  |
| Layer 3 discourse   | \<1ms/sentence     |
| **End-to-end**      | **~19ms/sentence** |

**Optimization Strategies**:

- Parallel engine execution (Layer 1)
- L1/L2 cache with memory budgets
- Batch processing for throughput
- Real data only (no stubs)

## Design Principles

### 1. Semantic-First Approach

- **Theory-Grounded**: Based on established formal semantics (Kratzer, Kamp, Reuland, etc.)
- **Real Linguistic Resources**: Actual VerbNet/FrameNet/WordNet data
- **No Stubs**: All engines load and process real data

### 2. Clean Layer Separation

- **Layer 1**: Raw lexical data from engines (word-level)
- **Layer 2**: Compositional event structures (sentence-level)
- **Layer 3**: Discourse representation (multi-sentence)

### 3. Performance Through Design

- **Parallel Execution**: Concurrent engine queries in Layer 1
- **Smart Caching**: L1/L2 cache with memory budgets
- **Batch Optimization**: Group processing for throughput
- **Fail-Fast**: Errors propagate cleanly, no silent degradation

### 4. Extensible Architecture

- **Plugin Engines**: Easy to add new semantic resources
- **Uniform Interface**: Consistent API across all engines
- **Configuration-Driven**: Runtime engine selection

## Quality Assurance

### Testing

- **Unit Tests**: Each module tested independently
- **Integration Tests**: Full L1→L2→L3 pipeline validation
- **Performance Tests**: Latency and throughput benchmarks
- **Coverage**: ~67% with 50% gate

### Performance Validation

| Metric              | Target        | Achieved      |
| ------------------- | ------------- | ------------- |
| Latency per word    | \<100μs       | 66-85μs       |
| Throughput          | >1000 words/s | 2000+ words/s |
| Memory (cache)      | \<5MB         | \<1MB         |
| End-to-end sentence | \<50ms        | ~19ms         |

## Future Directions

### Near-term (M9-M10)

- Comprehensive documentation and tutorials
- Research platform for linguistic theory testing
- Corpus analysis tools

### Long-term

- Multi-language support (Universal Dependencies)
- Neural model integration (hybrid symbolic-neural)
- Real-time collaborative editing
- Publication-ready evaluation framework

## Conclusion

Canopy provides a complete three-layer semantic analysis pipeline:

1. **Layer 1** delivers rich lexical semantics from 5 parallel engines
1. **Layer 2** composes Neo-Davidsonian events with modality, presupposition, and plurality
1. **Layer 3** builds discourse representations with temporal, centering, and coherence analysis

All layers are production-ready with ~19ms end-to-end latency and real linguistic data.
