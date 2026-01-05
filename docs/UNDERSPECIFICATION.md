# Underspecification & Ambiguity Handling

Canopy uses packed semantic representations to handle ambiguity efficiently. Instead of enumerating all possible readings (exponential), it stores shared structure with choice points (linear).

## Overview

### The Ambiguity Problem

A sentence like "I saw the man with the telescope at the bank" has multiple ambiguities:

- **Lexical**: "bank" = financial institution OR river bank
- **Structural**: "with the telescope" attaches to "saw" OR "man"
- **Structural**: "at the bank" attaches to "saw" OR "man"

With 2 lexical + 2 structural + 2 structural choices = 8 readings. Add scope ambiguity and pronoun resolution, and readings multiply exponentially.

### Packed Representation

Canopy's solution: **O(n) storage for O(2^n) readings**.

```
Traditional: Store 8 complete readings (redundant structure)
Packed:      Store 1 shared structure + 3 binary choice points
```

## Core Types

### ChoicePoint

Represents a point of ambiguity with alternatives:

```rust
use canopy::{ChoiceId, ChoiceType, ChoicePoint};
use canopy::runtime::{TokenId, SenseId};

// Lexical ambiguity: "bank" has two senses
let bank_choice = ChoicePoint {
    id: ChoiceId::new(0),
    choice_type: ChoiceType::LexicalSense {
        token_id: TokenId::new(5),
        senses: vec![
            SenseId::new("bank.n.01"),  // financial
            SenseId::new("bank.n.02"),  // river
        ],
    },
    alternatives: vec![
        Alternative::new(0, 0.7, "financial institution"),
        Alternative::new(1, 0.3, "river bank"),
    ],
    default: Some(0),  // Most likely reading
};
```

### ChoiceType Variants

| Type           | Description            | Example                      |
| -------------- | ---------------------- | ---------------------------- |
| `LexicalSense` | Multiple word senses   | "bank" = financial/river     |
| `Attachment`   | PP/modifier attachment | "saw the man with telescope" |
| `Scope`        | Quantifier scope       | "every student read a book"  |
| `Reference`    | Pronoun antecedent     | "John told Bill he left"     |

### PackedSemantics

Container for shared structure plus choice points:

```rust
use canopy::{PackedSemantics, SharedStructure};

// Create packed representation
let shared = SharedStructure::new(/* ... */);
let mut packed = PackedSemantics::new(shared);

// Add choice points
packed.add_choice(bank_choice);
packed.add_choice(attachment_choice);

// Query ambiguity
println!("Reading count: {}", packed.reading_count());  // 4
println!("Is ambiguous: {}", packed.is_ambiguous());    // true
```

### Reading

A single interpretation selecting one alternative at each choice point:

```rust
use canopy::Reading;

// Iterate through all readings (lazy enumeration)
for reading in packed.readings() {
    println!("Reading {}: probability={:.3}",
        reading.id.0,
        reading.probability);

    // See which alternative was chosen at each point
    for (choice_id, alt_index) in &reading.choices {
        println!("  Choice {}: alternative {}", choice_id.0, alt_index);
    }
}

// Get best reading (highest probability)
if let Some(best) = packed.best_reading() {
    println!("Best reading: {:?}", best.id);
}
```

## Scope Underspecification (MRS-style)

For quantifier scope, Canopy uses Minimal Recursion Semantics (MRS) style underspecification.

### Elementary Predications

```rust
use canopy::kernel::underspec::scope::{ScopeUnderspec, HandleConstraintType};

let mut scope = ScopeUnderspec::new();

// Add elementary predications with handles
let h1 = scope.new_handle();  // "every student"
let h2 = scope.new_handle();  // "read"
let h3 = scope.new_handle();  // "a book"

scope.add_ep(h1, "every", vec![var_x]);
scope.add_ep(h2, "read", vec![var_e, var_x, var_y]);
scope.add_ep(h3, "a", vec![var_y]);

// Add scope constraints
scope.add_constraint(h1, HandleConstraintType::Qeq, h2);  // every outscopes read
```

### Scope Resolution

```rust
// Enumerate all valid scope orderings
let orderings = scope.enumerate_orderings();
for ordering in orderings {
    println!("Scope order: {:?}", ordering);
}
// Output:
// [h1, h3, h2]  -- every > a > read (∀x.∃y.read(x,y))
// [h3, h1, h2]  -- a > every > read (∃y.∀x.read(x,y))

// Get surface (default) scope
let default = scope.default_ordering();
```

## Disambiguation Strategies

Canopy provides 5 built-in disambiguators implementing the `Disambiguator` trait:

### MinSurprisalDisambiguator

Selects the reading with **lowest surprisal** (= highest probability). Based on surprisal theory (Hale 2001, Levy 2008).

```rust
use canopy::MinSurprisalDisambiguator;

let disambiguator = MinSurprisalDisambiguator;
let best = disambiguator.select_reading(&packed, &ctx);
```

**When to use**: Default choice when you have a language model providing probabilities.

### ConfidenceDisambiguator

Uses **provider confidence scores** from semantic engines (VerbNet, FrameNet, etc.).

```rust
use canopy::ConfidenceDisambiguator;

let disambiguator = ConfidenceDisambiguator;
let best = disambiguator.select_reading(&packed, &ctx);
```

**When to use**: When you trust the semantic engine's internal confidence scores.

### EntropyReductionDisambiguator

Prefers readings that **maximize uncertainty reduction**. Based on entropy reduction hypothesis (Roark et al. 2009).

```rust
use canopy::kernel::underspec::EntropyReductionDisambiguator;

let disambiguator = EntropyReductionDisambiguator;
let best = disambiguator.select_reading(&packed, &ctx);
```

**When to use**: When processing incrementally and want readings that resolve the most ambiguity.

### HybridDisambiguator

**Weighted combination** of surprisal and confidence:

```rust
use canopy::kernel::underspec::HybridDisambiguator;

// Default: 70% surprisal, 30% confidence
let disambiguator = HybridDisambiguator::default();

// Custom weights
let custom = HybridDisambiguator::new(0.5, 0.5);  // Equal weight
```

**When to use**: When you want to balance language model predictions with semantic engine confidence.

### InteractiveDisambiguator

Returns **all readings** without selecting—useful for presenting choices to users.

```rust
use canopy::kernel::underspec::InteractiveDisambiguator;

let disambiguator = InteractiveDisambiguator;
let ranked = disambiguator.rank_readings(&packed, &ctx);

// Present all options to user
for (reading, score) in ranked {
    println!("Option: {:?} (score: {:.2})", reading.id, score);
}
```

**When to use**: Interactive systems where humans make final disambiguation decisions.

## Pipeline Integration

### Preserve All Readings

```rust
use canopy_resources::CanopyPipeline;

let pipeline = CanopyPipeline::new()?;

// Get underspecified analysis (preserves ambiguity)
let underspec = pipeline.analyze_underspecified("I saw the man with the telescope")?;

// Access packed events
if let Some(ref packed) = underspec.packed_events {
    println!("Choice points: {}", packed.sense_choices.len());
}

// Get ambiguity breakdown
let summary = underspec.ambiguity;
println!("Lexical: {}, Structural: {}, Scope: {}, Referential: {}",
    summary.lexical, summary.structural, summary.scope, summary.referential);
```

### Apply Disambiguation Strategy

```rust
use canopy::{MinSurprisalDisambiguator, Disambiguator};

let pipeline = CanopyPipeline::new()?;
let disambiguator = MinSurprisalDisambiguator;

// Analyze with specific strategy
let analysis = pipeline.analyze_with_disambiguator(
    "Every student read a book",
    &disambiguator,
)?;

// Result is fully disambiguated
println!("Selected reading applied");
```

### Compare Strategies

```rust
use canopy::{MinSurprisalDisambiguator, ConfidenceDisambiguator, Disambiguator};

let sentence = "John told Bill he was tired";
let underspec = pipeline.analyze_underspecified(sentence)?;

let strategies: Vec<(&str, Box<dyn Disambiguator>)> = vec![
    ("Surprisal", Box::new(MinSurprisalDisambiguator)),
    ("Confidence", Box::new(ConfidenceDisambiguator)),
];

for (name, strat) in &strategies {
    if let Some(reading) = strat.select_reading(&packed, &ctx) {
        println!("{}: selected reading {:?}", name, reading.id);
    }
}
```

## Referential Ambiguity

Pronoun resolution integrates with underspecification:

```rust
use canopy::kernel::discourse::{UnderspecBinding, AnaphorType, ReferentId};

// Create binding with multiple candidates
let binding = UnderspecBinding::new(
    vec![
        (ReferentId::new(1), 0.8),  // "John" - 80% confidence
        (ReferentId::new(2), 0.6),  // "Bill" - 60% confidence
    ],
    AnaphorType::Personal,
    false,  // not reflexive
);

// Add to packed semantics
let choice_id = packed.add_referential_ambiguity(
    ReferentId::new(3),  // "he"
    &binding,
);

// Now packed.reading_count() includes pronoun alternatives
```

## AmbiguitySummary

Get a breakdown of ambiguity types:

```rust
let summary = packed.ambiguity_summary();

println!("Ambiguity breakdown:");
println!("  Lexical choices:    {}", summary.lexical);
println!("  Structural choices: {}", summary.structural);
println!("  Scope choices:      {}", summary.scope);
println!("  Referential choices:{}", summary.referential);
println!("  Total readings:     {}", summary.total_readings);
```

## Memory Efficiency

The packed representation provides significant memory savings:

| Ambiguity         | Explicit Readings  | Packed Storage   |
| ----------------- | ------------------ | ---------------- |
| 5 binary choices  | 32 readings        | 5 choice points  |
| 10 binary choices | 1,024 readings     | 10 choice points |
| 20 binary choices | 1,048,576 readings | 20 choice points |

Readings are enumerated lazily—only materialized when iterated.

## See Also

- [SURPRISAL.md](SURPRISAL.md) — SurprisalModel integration for disambiguation
- [DISCOURSE.md](DISCOURSE.md) — Anaphora resolution and referential ambiguity
- [FORMAL_SEMANTICS.md](FORMAL_SEMANTICS.md) — Theoretical foundations (MRS, UDRT)
