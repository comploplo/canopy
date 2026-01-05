# Discourse Analysis

Canopy provides formal discourse semantics following linguistic theory. This document covers the discourse processing capabilities.

## Overview

The discourse module implements:

- **DRS Construction** — Discourse Representation Structures (Kamp 1981)
- **Coherence Relations** — SDRT-inspired relations between sentences
- **Discourse Moves** — Speech act classification
- **QUD Tracking** — Questions Under Discussion stack and tree
- **Presupposition Detection** — Trigger identification and accommodation
- **Anaphora Resolution** — Salience-based pronoun binding

## Basic Usage

```rust
use canopy::{DiscourseContext, DiscourseConfig};
use canopy_resources::CanopyPipeline;

let pipeline = CanopyPipeline::new()?;
let mut ctx = DiscourseContext::new(DiscourseConfig::default());

// Process sentences in sequence
for sentence in ["John entered the room.", "He saw Mary.", "She smiled."] {
    let analysis = pipeline.analyze(sentence)?;

    ctx.begin_sentence();
    ctx.process_events(&analysis.events);
    ctx.finalize_sentence();
}

// Access discourse structures
let drs = ctx.drs();
let coherence = ctx.coherence_graph();
let qud_report = ctx.qud_report();
```

## Coherence Relations

Coherence relations describe how sentences connect semantically. Canopy classifies relations following SDRT:

| Relation        | Description               | Example                              |
| --------------- | ------------------------- | ------------------------------------ |
| **Narration**   | Events in sequence        | "John left. Mary arrived."           |
| **Elaboration** | Second elaborates first   | "John left. He walked out the door." |
| **Explanation** | Second explains first     | "John left. He was angry."           |
| **Contrast**    | Opposing content          | "John left. Mary stayed."            |
| **Result**      | Second results from first | "John pushed Mary. She fell."        |
| **Background**  | Scene-setting             | "It was raining. John left."         |
| **TopicShift**  | New discourse topic       | "John left. The weather was nice."   |

### Accessing Coherence

```rust
// Get coherence classification for current sentence
let classification = ctx.classify_coherence(&events, &tokens);

println!("Relation: {:?}", classification.relation);
println!("Confidence: {:.2}", classification.confidence);

// Access full coherence graph
let graph = ctx.coherence_graph();
for edge in graph.edges() {
    println!("S{} -> S{}: {:?}",
        edge.from_sentence,
        edge.to_sentence,
        edge.classification.relation);
}
```

### Coherence Signals

The classifier uses multiple signals:

- **Lexical overlap** — Shared content words suggest Elaboration
- **Discourse cues** — "however" suggests Contrast, "because" suggests Explanation
- **Temporal markers** — "then", "next" suggest Narration
- **Referent sharing** — Pronouns referring back suggest continuity

## Discourse Moves

Discourse moves classify the communicative function of utterances:

| Move               | Description               | Example                   |
| ------------------ | ------------------------- | ------------------------- |
| **Assertion**      | States a proposition      | "The cat is on the mat."  |
| **Question**       | Requests information      | "Where is the cat?"       |
| **Correction**     | Contradicts prior content | "No, the cat is outside." |
| **Acknowledgment** | Confirms understanding    | "Yes, I see."             |
| **Elaboration**    | Adds detail to prior      | "It's a black cat."       |
| **Continuation**   | Continues same topic      | "And it's sleeping."      |

### Question Types

Questions are further classified:

```rust
pub enum QuestionType {
    YesNo,      // "Is the cat here?"
    WhSubject,  // "Who saw Mary?"
    WhObject,   // "What did John see?"
    WhLocation, // "Where is John?"
    WhTime,     // "When did John leave?"
    WhReason,   // "Why did John leave?"
    WhManner,   // "How did John leave?"
    Alternative,// "Did John leave or stay?"
}
```

### Accessing Moves

```rust
let move_classification = ctx.classify_move(&events, &tokens);

println!("Move: {:?}", move_classification.move_type);
if let Some(qt) = move_classification.question_type {
    println!("Question type: {:?}", qt);
}
```

## Questions Under Discussion (QUD)

QUD theory (Roberts 1996) models discourse as answering implicit or explicit questions. Canopy maintains both a stack and tree of active questions.

### QUD Stack

The stack tracks currently active questions:

```rust
let report = ctx.qud_report();

for entry in &report.entries {
    println!("Q{}: {} ({:?})",
        entry.issue.id,
        entry.issue.question,
        entry.issue.status);
}
```

### QUD Tree

The tree captures hierarchical question structure:

```rust
if let Some(tree) = &report.tree {
    for node in &tree.questions {
        let indent = "  ".repeat(node.depth);
        println!("{}Q{}: {}", indent, node.id, node.question);
    }
}
```

### QUD Origins

Questions can arise from:

- **Explicit questions** — Interrogative sentences
- **Implicit questions** — Raised by assertions (what happened next?)
- **Focus-induced** — Cleft constructions ("It was JOHN who left")

## Presupposition Detection

Presuppositions are implicit assumptions triggered by certain constructions.

### Presupposition Triggers

| Trigger             | Example                | Presupposition            |
| ------------------- | ---------------------- | ------------------------- |
| **Definite NP**     | "the king of France"   | There is a king of France |
| **Factive verb**    | "John knows Mary left" | Mary left                 |
| **Change of state** | "John stopped smoking" | John was smoking          |
| **Cleft**           | "It was John who left" | Someone left              |
| **Iterative**       | "John returned"        | John was here before      |

### Accessing Presuppositions

```rust
let presuppositions = ctx.detect_presuppositions(&events);

for p in &presuppositions {
    println!("Trigger: {:?}", p.trigger);
    println!("Content: {}", p.content);
    println!("Status: {:?}", p.status);
}
```

### Presupposition Status

```rust
pub enum PresuppositionStatus {
    Pending,      // Not yet resolved
    Accommodated, // Added to common ground
    Satisfied,    // Already in common ground
    Failed,       // Contradicts common ground
}
```

## Anaphora Resolution

Canopy resolves pronouns using salience-based ranking with Binding Theory constraints.

### Resolution Process

1. **Identify anaphors** — Pronouns, definite NPs
1. **Find candidates** — Previously mentioned referents
1. **Apply constraints** — Gender, number, syntactic position
1. **Rank by salience** — Recency, grammatical role, focus

### Accessing Bindings

```rust
let bindings = ctx.resolve_anaphora(&syntax);

for binding in &bindings {
    println!("{} -> {:?} (confidence: {:.2})",
        binding.anaphor,
        binding.antecedent,
        binding.confidence);
}
```

### Salience Factors

| Factor        | Weight | Description                          |
| ------------- | ------ | ------------------------------------ |
| Recency       | High   | More recent = more salient           |
| Subject       | High   | Subjects are prominent               |
| Topic         | High   | Discourse topic preferred            |
| Focus         | Medium | Focused elements preferred           |
| First mention | Low    | First in sentence slightly preferred |

## DRS Construction

Discourse Representation Structures capture semantic content:

```rust
let drs = ctx.drs();

// Referents introduced
println!("Referents: {:?}", drs.referents);

// Conditions (predicates)
for condition in &drs.conditions {
    println!("  {:?}", condition);
}
```

### DRS Conditions

```rust
pub enum DrsCondition {
    Predicate { name: String, args: Vec<ReferentId> },
    Equality { left: ReferentId, right: ReferentId },
    Negation { embedded: Box<Drs> },
    Implication { antecedent: Box<Drs>, consequent: Box<Drs> },
    Disjunction { left: Box<Drs>, right: Box<Drs> },
    // ... more conditions
}
```

## Configuration

```rust
pub struct DiscourseConfig {
    /// Salience decay between sentences (0.0-1.0)
    pub salience_decay: f32,

    /// Minimum confidence for pronoun resolution
    pub min_resolution_confidence: f32,

    /// Maximum QUD stack depth
    pub max_qud_depth: usize,

    /// Enable presupposition tracking
    pub track_presuppositions: bool,
}

impl Default for DiscourseConfig {
    fn default() -> Self {
        Self {
            salience_decay: 0.8,
            min_resolution_confidence: 0.3,
            max_qud_depth: 10,
            track_presuppositions: true,
        }
    }
}
```

## Surprisal Integration

Coherence classification can be refined using surprisal:

```rust
use canopy::{DiscourseContext, UniformSurprisalModel};

let lm = UniformSurprisalModel::default();
let ctx = DiscourseContext::new(DiscourseConfig::default())
    .with_surprisal_model(lm);

// Coherence classification now uses surprisal to adjust confidence
```

## Theoretical Background

### DRT (Kamp 1981)

Discourse Representation Theory models meaning construction across sentences, handling anaphora and quantifier scope.

### SDRT (Asher & Lascarides 2003)

Segmented DRT adds rhetorical relations between discourse segments.

### QUD (Roberts 1996)

Questions Under Discussion models discourse as organized around implicit questions that utterances address.

### Centering Theory (Grosz, Joshi, Weinstein 1995)

Models local coherence through tracking of salient entities (centers).

## See Also

- [ARCHITECTURE.md](ARCHITECTURE.md) — System design
- [GETTING_STARTED.md](GETTING_STARTED.md) — Quick start guide
- [UNDERSPECIFICATION.md](UNDERSPECIFICATION.md) — Referential ambiguity handling
- [SURPRISAL.md](SURPRISAL.md) — SurprisalModel for coherence adjustment
- [FORMAL_SEMANTICS.md](FORMAL_SEMANTICS.md) — DRT, SDRT, QUD theory
