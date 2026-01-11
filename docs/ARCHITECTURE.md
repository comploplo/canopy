# Architecture

## Overview

Canopy is a semantic linguistic analysis library implementing a 4-layer pipeline from morphosyntax to logical reasoning. It uses a provider-based architecture separating the kernel (pure semantic operations) from heavy resources (VerbNet, FrameNet, WordNet, etc.).

## 4-Layer Semantic Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              INPUT TEXT                                      │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  LAYER 1: MORPHOSYNTAX                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Tokenizer   │→ │  POS Tagger  │→ │ Dep Parser   │→ │ TAM Analyzer │     │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                                              │
│  Output: AnnotatedSyntax (tokens, UPOS, DepRel, MorphFeatures)              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  LAYER 2: EVENT COMPOSITION                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │SenseProvider │→ │ RoleProvider │→ │EventComposer │→ │ TAM Builder  │     │
│  │(VerbNet etc) │  │(theta roles) │  │(little v)    │  │(tense/aspect)│     │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                                              │
│  Output: ComposedEvents (predicate, participants, temporal_frame, aspect)   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  LAYER 3: DISCOURSE                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │DiscourseCtx  │→ │   Anaphora   │→ │  Coherence   │→ │     QUD      │     │
│  │(DRS builder) │  │  Resolution  │  │  Relations   │  │   Stack      │     │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                                              │
│  Output: DRS (referents, conditions, TAM conditions, temporal relations)    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  LAYER 4: LOGIC & REASONING                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Compiler    │→ │  Temporal    │→ │    Modal     │→ │   Query      │     │
│  │(DRS→Facts)   │  │  Reasoner    │  │   Reasoner   │  │  Answering   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                                              │
│  Output: ConsistencyResult, EntailmentResult, QueryResult, Explanations     │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Crate Structure

```
canopy/
├── crates/
│   ├── canopy/                     # KERNEL: Core types + semantic operations (~24k lines)
│   │   ├── src/
│   │   │   ├── core/               # Foundation types
│   │   │   │   ├── error.rs        # CanopyError
│   │   │   │   ├── event.rs        # AspectualClass, ModalForce, ModalFlavor
│   │   │   │   ├── syntax.rs       # DepRel, UPos, MorphFeatures
│   │   │   │   └── theta.rs        # ThetaRole (28 roles)
│   │   │   │
│   │   │   ├── kernel/
│   │   │   │   ├── discourse/      # Layer 3: DRS & discourse
│   │   │   │   │   ├── binding.rs      # Anaphora resolution
│   │   │   │   │   ├── coherence.rs    # Discourse relations (SDRT)
│   │   │   │   │   ├── context.rs      # DiscourseContext orchestrator
│   │   │   │   │   ├── drs.rs          # DRS data structures
│   │   │   │   │   ├── modal.rs        # WorldId, ModalFrame, Counterfactual
│   │   │   │   │   ├── moves.rs        # Discourse move classification
│   │   │   │   │   ├── presupposition.rs # Presupposition tracking
│   │   │   │   │   ├── qud.rs          # Question Under Discussion
│   │   │   │   │   ├── referent.rs     # Discourse referents
│   │   │   │   │   ├── relevance.rs    # QUD relevance scoring
│   │   │   │   │   ├── tam_builder.rs  # TAM → DRS conditions
│   │   │   │   │   ├── temporal.rs     # TemporalFrame, TimePoint, Aspect
│   │   │   │   │   └── validation.rs   # Real-time contradiction detection
│   │   │   │   │
│   │   │   │   ├── events/         # Layer 2: Event composition
│   │   │   │   │   ├── compose.rs      # EventComposer
│   │   │   │   │   ├── tam.rs          # Morphology → TAM features
│   │   │   │   │   └── types.rs        # ComposedEvent, LittleVType
│   │   │   │   │
│   │   │   │   ├── logic/          # Layer 4: Reasoning
│   │   │   │   │   ├── answer.rs       # QueryResult, AnswerBinding
│   │   │   │   │   ├── compiler.rs     # DRS → CompiledDrs (facts)
│   │   │   │   │   ├── modal_reasoner.rs   # Kripke semantics
│   │   │   │   │   ├── proof.rs        # Explanations with provenance
│   │   │   │   │   ├── query.rs        # Query types (yes/no, wh-)
│   │   │   │   │   ├── reasoner.rs     # Reasoner trait, Conflict types
│   │   │   │   │   ├── solver.rs       # ClosedWorldReasoner
│   │   │   │   │   └── temporal_reasoner.rs # Allen interval algebra
│   │   │   │   │
│   │   │   │   ├── incremental/    # Incremental processing
│   │   │   │   │   ├── beam.rs         # Beam search
│   │   │   │   │   ├── lm.rs           # Language model interface
│   │   │   │   │   ├── state.rs        # Incremental state
│   │   │   │   │   └── surprisal.rs    # Surprisal computation
│   │   │   │   │
│   │   │   │   ├── underspec/      # Underspecification
│   │   │   │   │   ├── disambiguation.rs # Reading selection
│   │   │   │   │   ├── scope.rs        # Scope underspecification
│   │   │   │   │   └── types.rs        # PackedSemantics, ChoicePoint
│   │   │   │   │
│   │   │   │   └── trace/          # Debugging traces & sense selection
│   │   │   │
│   │   │   └── runtime/            # Provider traits & IR
│   │   │       ├── ids.rs              # TokenId, SenseId, FrameId
│   │   │       ├── ir.rs               # AnnotatedSyntax, AnnotatedToken
│   │   │       └── providers.rs        # SenseProvider, RoleProvider traits
│   │   │
│   │   └── tests/
│   │       ├── golden_snapshots.rs     # Regression tests
│   │       └── tam_integration_tests.rs # TAM pipeline tests
│   │
│   ├── canopy-resources/           # RESOURCES: Engines + pipeline (~28k lines)
│   │   ├── src/
│   │   │   ├── engine/             # Shared infrastructure
│   │   │   │   ├── cacheable.rs        # LRU caching
│   │   │   │   └── shared.rs           # SharedEngines
│   │   │   ├── verbnet/            # VerbNet engine (333 classes)
│   │   │   ├── framenet/           # FrameNet engine (1200+ frames)
│   │   │   ├── wordnet/            # WordNet engine (117k synsets)
│   │   │   ├── propbank/           # PropBank engine
│   │   │   ├── lexicon/            # Closed-class words
│   │   │   ├── syntax/             # TreebankSyntaxProvider, MWE detection
│   │   │   ├── tokenizer/          # Tokenizers
│   │   │   ├── providers/          # DefaultProvider implementations
│   │   │   └── pipeline/           # CanopyPipeline orchestrator
│   │   │       ├── orchestrator.rs     # Main pipeline
│   │   │       ├── analysis.rs         # SemanticAnalysis, diagnostic types
│   │   │       ├── trace_builder.rs    # Sense selection tracing
│   │   │       └── tree.rs             # Pretty-printed semantic trees (ptree)
│   │   │
│   │   └── examples/
│   │       └── demo.rs             # Comprehensive feature demo
│   │
│   ├── canopy-lsp/                 # LSP: Language Server Protocol (~4k lines)
│   │   ├── src/
│   │   │   ├── handlers/
│   │   │   │   ├── diagnostics.rs      # Semantic diagnostics (10+ types)
│   │   │   │   ├── hover.rs            # Rich hover with DRS, bindings, traces
│   │   │   │   ├── code_actions.rs     # Quick-fixes for all diagnostics
│   │   │   │   ├── semantic_tokens.rs  # Semantic highlighting
│   │   │   │   ├── symbols.rs          # Document symbols
│   │   │   │   └── inlay_hints.rs      # Inline semantic hints
│   │   │   ├── backend.rs              # LSP backend implementation
│   │   │   └── state.rs                # Document state management
│   │
│   └── canopy-cli/                 # CLI + demos (~600 lines)
│
└── data/                           # Linguistic resources (gitignored)
    ├── verbnet/
    ├── framenet/
    ├── wordnet/
    ├── propbank/
    ├── lexicon/
    └── ud_english-ewt/
```

## Dependency Graph

```
         canopy-cli
              │
              ▼
      canopy-resources
              │
              ▼
           canopy
```

**Key property**: The `canopy` kernel has NO dependencies on `canopy-resources`. This enables testing the kernel in isolation.

## TAM (Tense, Aspect, Modality) Data Flow

```
Layer 2: EventComposer
    │
    ├── temporal_frame: Some(TemporalFrame::past())
    ├── aspectual_viewpoint: Some(AspectualViewpoint::Progressive)
    └── predicate, participants, ...
            │
            ▼
Layer 3: DiscourseContext.process_single_event()
    │
    ├── Creates event referent
    ├── Adds predicate to DRS
    ├── Adds theta roles
    ├── ┌─────────────────────────────────────────┐
    │   │ TamBuilder.build_tam_conditions()       │
    │   │   → DrsCondition::TemporalFrameAssignment│
    │   │   → DrsCondition::AspectualOp           │
    │   └─────────────────────────────────────────┘
    └── Handles polarity
            │
            ▼
        DRS (with TAM conditions)
            │
            ▼
Layer 4: ClosedWorldReasoner.check_consistent()
    │
    ├── ┌─────────────────────────────────────────┐
    │   │ validate_temporal_consistency()         │
    │   │   → TemporalReasoner (Allen algebra)    │
    │   │   → Detects temporal cycles             │
    │   │   → Infers transitive constraints       │
    │   └─────────────────────────────────────────┘
    ├── ┌─────────────────────────────────────────┐
    │   │ validate_modal_consistency()            │
    │   │   → ModalReasoner (Kripke semantics)    │
    │   │   → Evaluates necessity/possibility     │
    │   │   → Checks accessible worlds            │
    │   └─────────────────────────────────────────┘
    └── find_polarity_conflicts()
            │
            ▼
    ConsistencyResult { conflicts, explanation }
```

## Key Components

### Temporal Reasoning (Allen Interval Algebra)

13 basic relations between intervals:

- `Before (<)`, `After (>)`
- `Meets (m)`, `Met-by (mi)`
- `Overlaps (o)`, `Overlapped-by (oi)`
- `Starts (s)`, `Started-by (si)`
- `During (d)`, `Contains (di)`
- `Finishes (f)`, `Finished-by (fi)`
- `Equals (=)`

```rust
let mut reasoner = TemporalReasoner::new();
reasoner.add_constraint(TemporalConstraint::new(e1, e2, AllenRelation::Before, "narration"));
reasoner.add_constraint(TemporalConstraint::new(e2, e3, AllenRelation::Before, "narration"));
let result = reasoner.check_consistency();
// Infers: e1 Before e3 (transitive closure)
```

### Modal Reasoning (Kripke Semantics)

```rust
let mut reasoner = ModalReasoner::new();
let w1 = reasoner.create_world();
reasoner.make_accessible(WorldId::ACTUAL, w1, AccessibilityType::Epistemic);
reasoner.get_world_mut(&w1).unwrap().add_fact("raining");

// "It might be raining" (epistemic possibility)
let eval = reasoner.evaluate_modal_fact(
    ModalForce::Possibility,
    ModalFlavor::Epistemic,
    "raining",
);
assert!(eval.holds);
```

### DRS Conditions

```rust
pub enum DrsCondition {
    // Basic conditions
    Predicate { name, referent },
    Relation { name, arg1, arg2 },
    ThetaRole { event_id, role, filler },

    // Temporal conditions
    TemporalRelation { relation, event1, event2 },
    TemporalFrameAssignment { event, frame },
    AspectualOp { operator, event, scope },
    TemporalAnchor { event, anchor_type, reference },

    // Modal conditions
    ModalOp { force, flavor, scope, world_var },
    Accessible { from_world, to_world, relation },
    Counterfactual { antecedent, consequent, modal_force, closest_worlds },

    // Logical operators
    Negation(Box<Drs>),
    Disjunction(Box<Drs>, Box<Drs>),
    Implication { antecedent, consequent },
}
```

## Core Types

### ThetaRole (28 semantic roles)

```rust
pub enum ThetaRole {
    Agent,       // Initiator: "John broke the vase"
    Patient,     // Affected: "John broke the vase"
    Theme,       // Moved/transferred: "John gave Mary a book"
    Experiencer, // Mental state: "John fears spiders"
    Recipient,   // Receiving: "John gave Mary a book"
    Goal,        // Endpoint: "John went to the store"
    Source,      // Origin: "John came from Paris"
    Location,    // Place: "John lives in Paris"
    Instrument,  // Tool: "John cut with a knife"
    // ... 19 more roles
}
```

### LittleVType (Event decomposition)

```rust
pub enum LittleVType {
    Cause,      // Causative: "break", "kill" → [CAUSE [BECOME [STATE]]]
    Become,     // Change: "open", "melt" → [BECOME [STATE]]
    Be,         // State: "know", "love" → [BE [STATE]]
    Do,         // Activity: "run", "swim" → [DO [ACT]]
    Experience, // Psych: "fear", "admire"
    Go,         // Motion: "go", "run"
    Have,       // Possession: "have", "own"
    Say,        // Communication: "say", "tell"
    Exist,      // Existence: "exist", "be"
}
```

### TemporalFrame (Reichenbachian tense)

```rust
pub struct TemporalFrame {
    pub speech_time: TimePoint,      // S: utterance time
    pub reference_time: TimePoint,   // R: perspective time
    pub event_time: TimeInterval,    // E: event duration
}

// E < R < S: Past perfect ("had left")
// E,R < S:   Simple past ("left")
// S,R,E:     Simple present ("leaves")
// S < R,E:   Simple future ("will leave")
```

## Provider Traits

```rust
// Predicate decomposition (VerbNet, FrameNet)
pub trait SenseProvider: Send + Sync {
    fn decompose_predicate(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
    ) -> Result<Vec<PredicateDecomposition>, CanopyError>;
}

// Thematic role binding
pub trait RoleProvider: Send + Sync {
    fn bind_roles(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
        sense: Option<&SenseId>,
    ) -> Result<Vec<RoleBinding>, CanopyError>;
}

// Syntax parsing
pub trait SyntaxProvider: Send + Sync {
    fn parse(&self, text: &str) -> Result<AnnotatedSyntax, CanopyError>;
}
```

## Usage Examples

### Full Pipeline

```rust
use canopy_resources::CanopyPipeline;

let pipeline = CanopyPipeline::new()?;
let analysis = pipeline.analyze("John gave Mary a book.")?;

println!("Events: {}", analysis.events.len());
println!("Consistent: {}", analysis.consistency.consistent);
```

### Discourse Context with TAM

```rust
use canopy::kernel::discourse::{DiscourseContext, DiscourseConfig};
use canopy::kernel::events::{ComposedEvent, TemporalFrame, AspectualViewpoint};

let mut ctx = DiscourseContext::new(DiscourseConfig::default());

// Process event with TAM features
let event = ComposedEvent {
    predicate: "run".to_string(),
    temporal_frame: Some(TemporalFrame::past_progressive()),
    aspectual_viewpoint: Some(AspectualViewpoint::Progressive),
    // ...
};

ctx.begin_sentence();
ctx.process_events(&events);
ctx.end_sentence();

// DRS now contains TemporalFrameAssignment and AspectualOp conditions
```

### Consistency Checking

```rust
use canopy::kernel::logic::{ClosedWorldReasoner, Reasoner};

let reasoner = ClosedWorldReasoner::new();
let result = reasoner.check_consistent(&drs);

if !result.consistent {
    for conflict in &result.conflicts {
        println!("Conflict: {:?} - {}", conflict.conflict_type, conflict.description);
    }
}
```

### Query Answering

```rust
use canopy::kernel::logic::{Query, Proposition};

let query = Query::yes_no(Proposition::predicate("leave", "John", ThetaRole::Agent));
let result = reasoner.answer(&drs, &query);

match result.answer {
    Answer::Yes => println!("John left."),
    Answer::No => println!("John didn't leave."),
    Answer::Unknown => println!("Cannot determine."),
}
```

## Performance

| Operation         | Time                |
| ----------------- | ------------------- |
| Pipeline init     | ~730ms (one-time)   |
| Sentence analysis | 30-200μs            |
| Engine loading    | ~300ms (from cache) |
| Memory            | ~350 MB             |

## Test Coverage

- **Threshold**: 80%
- **Current**: ~81%
- Integration tests cover TAM flow through all 4 layers
