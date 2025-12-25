> **ARCHIVED DESIGN DOCUMENT**
> This describes Layer 3 DRT design planned for M8+. Not yet implemented.
> For current implementation, see [ARCHITECTURE.md](../../ARCHITECTURE.md).

# Layer 3: DRT & Semantic Composition Design

This document outlines the design and API for Layer 3 of the Canopy semantic parser, which implements Discourse Representation Theory (DRT) with lambda calculus composition.

## Overview

Layer 3 transforms the event structures from Layer 2 into formal semantic representations using:

- **Discourse Representation Structures (DRS)**: Formal semantic representations
- **Lambda Calculus**: Type-driven compositional semantics
- **β-reduction**: Function application and simplification
- **Presupposition Handling**: Managing semantic assumptions

## Core Components

### 1. DRS (Discourse Representation Structure)

```rust
/// Core DRS representation following Kamp & Reyle (1993)
#[derive(Debug, Clone, PartialEq)]
pub struct DRS {
    /// Discourse referents (variables)
    pub referents: Vec<Referent>,

    /// Conditions over referents
    pub conditions: Vec<Condition>,

    /// Presuppositions
    pub presuppositions: Vec<DRS>,

    /// Accessibility relations
    pub accessibility: AccessibilityGraph,
}

/// Discourse referent
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Referent {
    pub id: ReferentId,
    pub sort: SemanticSort,
    pub constraints: Vec<Constraint>,
}

/// DRS conditions
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Atomic predication: P(x)
    Atomic(Predicate, Vec<Referent>),

    /// Negation: ¬φ
    Negation(Box<DRS>),

    /// Implication: φ → ψ
    Implication(Box<DRS>, Box<DRS>),

    /// Disjunction: φ ∨ ψ
    Disjunction(Box<DRS>, Box<DRS>),

    /// Equality: x = y
    Equality(Referent, Referent),

    /// Temporal relation: before(e1, e2)
    Temporal(TemporalRelation, EventVar, EventVar),
}
```

### 2. Lambda Calculus Terms

```rust
/// Lambda calculus terms for compositional semantics
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// Variable: x
    Variable(Variable),

    /// Lambda abstraction: λx.M
    Abstraction(Variable, Box<Term>),

    /// Function application: M N
    Application(Box<Term>, Box<Term>),

    /// Constant: john, love, etc.
    Constant(Constant),

    /// DRS embedding: |DRS|
    DRSEmbedding(DRS),
}

/// Type system for lambda terms
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Basic types
    Entity,           // e
    Truth,           // t
    Event,           // v
    Time,            // i

    /// Function types
    Function(Box<Type>, Box<Type>),  // α → β

    /// Complex types
    Proposition,     // <s,t>
    Predicate(usize), // <e^n,t>
}
```

### 3. Semantic Composition Engine

```rust
/// Main compositional semantics engine
pub struct SemanticComposer {
    /// Type inference engine
    type_checker: TypeChecker,

    /// β-reduction engine
    reducer: BetaReducer,

    /// DRS construction rules
    drs_rules: DRSConstructionRules,

    /// Lexical semantic entries
    lexicon: SemanticLexicon,
}

impl SemanticComposer {
    /// Convert Layer 2 events to DRS
    pub fn events_to_drs(&mut self, events: Vec<Event>) -> Result<DRS, CompositionError> {
        let mut drs = DRS::new();

        for event in events {
            let event_drs = self.event_to_drs(&event)?;
            drs = self.merge_drs(drs, event_drs)?;
        }

        // Apply presupposition projection
        self.project_presuppositions(&mut drs)?;

        Ok(drs)
    }

    /// Compose lambda terms with β-reduction
    pub fn compose_terms(&mut self, terms: Vec<Term>) -> Result<Term, CompositionError> {
        let mut result = terms.into_iter()
            .reduce(|acc, term| {
                Term::Application(Box::new(acc), Box::new(term))
            })
            .ok_or(CompositionError::EmptyTerms)?;

        // Apply β-reduction
        result = self.reducer.reduce(result)?;

        Ok(result)
    }
}
```

## API Design

### Layer 3 Main Interface

```rust
/// Layer 3 semantic analysis result
#[derive(Debug, Clone)]
pub struct Layer3Analysis {
    /// Final DRS representation
    pub drs: DRS,

    /// Lambda terms before reduction
    pub lambda_terms: Vec<Term>,

    /// Semantic composition tree
    pub composition_tree: CompositionTree,

    /// Type derivations
    pub type_derivations: Vec<TypeDerivation>,

    /// Processing metrics
    pub metrics: Layer3Metrics,
}

/// Main Layer 3 analyzer
pub struct Layer3Analyzer {
    composer: SemanticComposer,
    config: Layer3Config,
}

impl Layer3Analyzer {
    /// Analyze Layer 2 results to produce DRS
    pub fn analyze(&mut self, layer2_result: Layer2Analysis) -> Result<Layer3Analysis, Layer3Error> {
        let start = Instant::now();

        // 1. Convert events to DRS
        let drs = self.composer.events_to_drs(layer2_result.events)?;

        // 2. Build lambda terms
        let lambda_terms = self.build_lambda_terms(&layer2_result)?;

        // 3. Perform semantic composition
        let composition_tree = self.compose_semantics(&lambda_terms)?;

        // 4. Apply type checking
        let type_derivations = self.check_types(&lambda_terms)?;

        let metrics = Layer3Metrics {
            total_time_us: start.elapsed().as_micros() as u64,
            drs_construction_time_us: /* measured */,
            lambda_composition_time_us: /* measured */,
            type_checking_time_us: /* measured */,
            referents_created: drs.referents.len(),
            conditions_created: drs.conditions.len(),
        };

        Ok(Layer3Analysis {
            drs,
            lambda_terms,
            composition_tree,
            type_derivations,
            metrics,
        })
    }
}
```

### Configuration

```rust
/// Layer 3 configuration
#[derive(Debug, Clone)]
pub struct Layer3Config {
    /// Enable DRS construction
    pub enable_drs_construction: bool,

    /// Enable lambda calculus composition
    pub enable_lambda_composition: bool,

    /// Enable type checking
    pub enable_type_checking: bool,

    /// Enable presupposition projection
    pub enable_presupposition_projection: bool,

    /// Maximum β-reduction steps
    pub max_reduction_steps: usize,

    /// Performance mode
    pub performance_mode: PerformanceMode,
}
```

## Implementation Phases

### Phase 2: DRS Core Types

- [ ] Implement `DRS`, `Referent`, `Condition` types
- [ ] Add basic DRS construction from events
- [ ] Implement referent resolution
- [ ] Add presupposition handling

### Phase 3: Lambda Calculus

- [ ] Port lambda calculus from Python V1
- [ ] Implement β-reduction engine
- [ ] Add type inference system
- [ ] Create compositional semantic rules

### Phase 4: Integration & Optimization

- [ ] Integrate with FrameNet for semantic frame enrichment
- [ ] Optimize performance to maintain \<120μs total
- [ ] Add comprehensive error handling
- [ ] Create debugging and introspection tools

## Error Handling

```rust
/// Layer 3 specific errors
#[derive(Debug, thiserror::Error)]
pub enum Layer3Error {
    #[error("DRS construction failed: {context}")]
    DRSConstruction { context: String },

    #[error("Lambda composition failed: {context}")]
    LambdaComposition { context: String },

    #[error("Type checking failed: {context}")]
    TypeChecking { context: String },

    #[error("β-reduction exceeded maximum steps: {steps}")]
    ReductionLimit { steps: usize },

    #[error("Presupposition projection failed: {context}")]
    PresuppositionProjection { context: String },
}
```

## Performance Requirements

- **Total Layer 3 Processing**: \<50μs (of 120μs total budget)
- **DRS Construction**: \<20μs
- **Lambda Composition**: \<20μs
- **Type Checking**: \<10μs
- **Memory**: \<1MB additional allocation per analysis

## Integration Points

### With Layer 2

- Consumes `Layer2Analysis` containing events and theta assignments
- Maps VerbNet predicates to DRS conditions
- Preserves event structure in temporal DRS conditions

### With Canonical API

- Provides `Layer3Analysis` as part of `CanopyAnalysis`
- Exposes DRS through unified interface
- Supports serialization for LSP and PyO3 bindings

### With Future Layers

- DRS serves as input for discourse processing
- Lambda terms enable proof-theoretic reasoning
- Type system supports formal verification

## Research Integration

This implementation is based on:

- **Kamp & Reyle (1993)**: From Discourse to Logic - Core DRT
- **Blackburn & Bos (2005)**: Representation and Inference for Natural Language - Computational DRT
- **Heim & Kratzer (1998)**: Semantics in Generative Grammar - Lambda calculus composition
- **Carpenter (1997)**: Type-Logical Semantics - Type system design

## Testing Strategy

- **Unit Tests**: Each DRS component, lambda operations
- **Integration Tests**: Layer 2 → Layer 3 pipeline
- **Performance Tests**: Sub-50μs processing requirement
- **Correctness Tests**: DRS semantic equivalence, lambda normalization
- **Regression Tests**: Ensure compatibility with existing layers
