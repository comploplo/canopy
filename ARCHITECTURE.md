# canopy.rs Architecture

## Overview

canopy.rs implements a **4-layer linguistic analysis architecture** designed for high performance and theoretical correctness. The system transforms raw text through increasingly sophisticated semantic representations, culminating in rich LSP responses.

### Core Transformation

```
Text → UDPipe → Events → DRT → LSP
```

This represents a fundamental shift from the Python V1 system:

- **V1**: `spaCy → JSON → Proto → LSP` (surface-level mapping)
- **V2**: `UDPipe → Events → DRT → LSP` (theory-driven representation)

## Design Principles

### 1. Type Safety First

Use Rust's type system to enforce linguistic constraints at compile time:

```rust
// Theta roles cannot be assigned incorrectly
struct Event {
    participants: HashMap<ThetaRole, Participant>,
    // Compile-time guarantee: only valid theta roles can be keys
}
```

### 2. Theory-Driven Design

Every architectural decision grounded in established linguistic theory:

- **Universal Dependencies** for syntactic representation
- **Neo-Davidsonian semantics** for event structures
- **Discourse Representation Theory** for compositional semantics
- **Optimality Theory** for constraint-based analysis (M6+)

### 3. Performance Through Design

Efficiency from architecture, not micro-optimization:

- Zero-copy parsing where possible
- Compile-time optimization through types
- Memory pools for frequent allocations
- Streaming analysis for large documents

### 4. Explicit Over Implicit

No hidden features or black-box processing:

- Every feature extraction step is traceable
- Multiple strategies can compete with confidence scoring
- Easy to debug and extend
- Clear data flow through layers

## 4-Layer Architecture

### Layer 1: Morphosyntactic Analysis

**Purpose**: Basic linguistic structure extraction

**Input**: Raw text
**Output**: Enhanced word structures with dependencies

**Core Types**:

```rust
struct Word {
    id: usize,
    text: String,
    lemma: String,
    upos: UPos,                    // Universal POS tags
    feats: MorphFeatures,          // From UDPipe
    head: Option<usize>,           // Dependency head
    deprel: DepRel,               // Dependency relation
}

struct EnhancedWord {
    base: Word,
    semantic: SemanticFeatures,    // Animacy, definiteness, etc.
    theta_potential: Vec<ThetaRole>,
    semantic_type: SemanticType,   // e, t, <e,t>, etc.
}
```

**Key Components**:

- **UDPipe Integration**: Lightweight, theory-aligned parsing
- **Feature Extraction**: Rule-based semantic feature detection
- **Morphological Analysis**: Rich feature representation

### Layer 2: Event Structure Analysis

**Purpose**: Neo-Davidsonian event representation with participants

**Input**: Enhanced words with dependencies
**Output**: Event structures with theta roles

**Core Types**:

```rust
struct Event {
    id: EventId,
    predicate: Predicate,
    participants: HashMap<ThetaRole, Participant>,
    modifiers: Vec<Modifier>,
    aspect: AspectualClass,
    little_v: Option<LittleV>,     // Event decomposition
    movement_chains: Vec<MovementChain>,
}

enum ThetaRole {
    Agent, Patient, Theme, Experiencer,
    Recipient, Goal, Source, Instrument,
    Benefactive, Location, Temporal,
    // ... (19 total from Python V1 system)
}

enum LittleV {
    Cause { causer: Participant, caused: Box<Event> },
    Become { theme: Participant, result: State },
    Do { agent: Participant, action: Action },
    Be { theme: Participant, state: State },
}
```

**Key Components**:

- **VerbNet Integration**: Port patterns from Python V1 system
- **Theta Role Assignment**: Confidence-scored role labeling
- **Event Decomposition**: Little v analysis for causatives
- **Movement Chains**: Basic syntactic movement representation

### Layer 3: Compositional Semantics

**Purpose**: DRT-based compositional semantic representation

**Input**: Event structures
**Output**: Discourse Representation Structures (DRS) and lambda terms

**Core Types**:

```rust
struct DRS {
    referents: HashSet<Referent>,
    conditions: Vec<DRSCondition>,
    presuppositions: Vec<DRSCondition>,
}

enum DRSCondition {
    Predication(Referent, String, Vec<Referent>),
    Equality(Referent, Referent),
    Negation(Box<DRS>),
    Implication(Box<DRS>, Box<DRS>),
    Quantification(Quantifier, Referent, Box<DRS>, Box<DRS>),
}

enum Term {
    Var(String, SemanticType),
    Const(String, SemanticType),
    Abs(String, SemanticType, Box<Term>),    // λ-abstraction
    App(Box<Term>, Box<Term>),               // Function application
    DRSEmbed(DRS),
}
```

**Key Components**:

- **Lambda Calculus**: Typed λ-terms with β-reduction (ported from Python V1)
- **DRT Construction**: Build DRS from event structures
- **Compositional Rules**: Function application, predicate modification
- **Type Inference**: Semantic type system (e, t, s, functions)

### Layer 4: Discourse & LSP Integration

**Purpose**: Document-level analysis and Language Server Protocol responses

**Input**: DRS and lambda terms
**Output**: LSP responses (hover, diagnostics, actions)

**Core Types**:

```rust
struct DiscourseContext {
    current_drs: DRS,
    referent_stack: Vec<Referent>,    // Accessibility hierarchy
    entity_map: HashMap<String, Referent>,
    focus: Option<Referent>,
}

struct SemanticAnalysis {
    words: Vec<EnhancedWord>,
    events: Vec<Event>,
    drs: DRS,
    lambda_term: Term,
    diagnostics: Vec<Diagnostic>,
}

enum DiagnosticKind {
    ThetaViolation(ThetaRole),
    BindingViolation(BindingPrinciple),
    ContradictionDetected(DRS, DRS),
    AspectMismatch(AspectualClass, AspectualClass),
    ScopeAmbiguity(Vec<ScopeReading>),
}
```

**Key Components**:

- **Discourse Context**: Cross-sentence entity tracking
- **LSP Server**: Tower-LSP async implementation
- **Rich Diagnostics**: Linguistic analysis beyond basic grammar
- **Intelligent Actions**: Theory-informed code actions

## Data Flow

### Sequential Processing Pipeline

```rust
// Layer 1: Morphosyntactic
let words = udpipe_parser.parse(text)?;
let enhanced_words = feature_extractor.extract(words)?;

// Layer 2: Event Structure
let events = event_analyzer.analyze(enhanced_words)?;
let theta_assigned = theta_assigner.assign_roles(events)?;

// Layer 3: Compositional Semantics
let drs = drt_composer.compose(theta_assigned)?;
let lambda_term = lambda_composer.build_term(drs)?;

// Layer 4: LSP Integration
let analysis = SemanticAnalysis::new(enhanced_words, events, drs, lambda_term);
let response = lsp_handler.handle_request(analysis)?;
```

### Error Propagation

```rust
type AnalysisResult<T> = Result<T, CanopyError>;

enum CanopyError {
    ParseError { context: String, source: ParseErrorKind },
    SemanticError { phase: String, details: String },
    LspError { request: String, cause: Box<dyn Error> },
}
```

## Key Design Patterns

### 1. Strategy Pattern (from Python V1)

Multiple implementations with shared interfaces:

```rust
trait FeatureExtractor {
    fn extract(&self, words: &[Word]) -> AnalysisResult<Vec<EnhancedWord>>;
}

struct RuleBasedExtractor { /* rules */ }
struct CorpusBasedExtractor { /* patterns */ }
struct NeuralExtractor { /* model */ }
```

### 2. Builder Pattern

Complex type construction:

```rust
impl EventBuilder {
    fn new(predicate: Predicate) -> Self;
    fn with_participant(mut self, role: ThetaRole, participant: Participant) -> Self;
    fn with_aspect(mut self, aspect: AspectualClass) -> Self;
    fn build(self) -> AnalysisResult<Event>;
}
```

### 3. Pipeline Pattern (enhanced from Python V1)

Sequential processing with typed interfaces:

```rust
trait AnalysisStage<Input, Output> {
    fn process(&self, input: Input) -> AnalysisResult<Output>;
}

struct AnalysisPipeline<S1, S2, S3, S4> {
    stage1: S1,  // Layer 1: UDPipe
    stage2: S2,  // Layer 2: Events
    stage3: S3,  // Layer 3: DRT
    stage4: S4,  // Layer 4: LSP
}
```

### 4. Type-State Pattern

Enforce correct usage at compile time:

```rust
struct EventAnalysis<State> {
    data: AnalysisData,
    _state: PhantomData<State>,
}

struct Parsed;
struct Analyzed;
struct Composed;

impl EventAnalysis<Parsed> {
    fn analyze(self) -> AnalysisResult<EventAnalysis<Analyzed>> { /* ... */ }
}

impl EventAnalysis<Analyzed> {
    fn compose(self) -> AnalysisResult<EventAnalysis<Composed>> { /* ... */ }
}
```

## Performance Architecture

### Memory Management

- **Arena Allocation**: For temporary linguistic structures
- **String Interning**: For repeated linguistic constants
- **Copy-on-Write**: For immutable sharing between layers
- **Memory Pools**: For frequent small allocations

### Caching Strategy

```rust
struct AnalysisCache {
    parsed_sentences: LruCache<String, Vec<Word>>,
    theta_assignments: LruCache<EventId, HashMap<ThetaRole, Participant>>,
    lambda_terms: LruCache<DRSId, Term>,
}
```

### Streaming Analysis

For large documents:

```rust
struct StreamingAnalyzer {
    buffer: VecDeque<Sentence>,
    window_size: usize,
    discourse_context: DiscourseContext,
}

impl StreamingAnalyzer {
    fn process_sentence(&mut self, sentence: Sentence) -> Vec<AnalysisResult<SemanticAnalysis>>;
}
```

## Module Organization

### Crate Structure

```
canopy/
├── canopy-core/           # Core linguistic types and utilities
│   ├── types.rs          # Word, Sentence, Document
│   ├── enums.rs          # ThetaRole, PartOfSpeech, etc.
│   ├── errors.rs         # Error types and handling
│   └── utils.rs          # Common utilities
│
├── canopy-parser/         # Layer 1: UDPipe integration
│   ├── udpipe.rs         # UDPipe bindings
│   ├── features.rs       # Semantic feature extraction
│   └── morphology.rs     # Morphological analysis
│
├── canopy-semantics/      # Layers 2-3: Events and DRT
│   ├── events/           # Layer 2: Event structures
│   │   ├── mod.rs
│   │   ├── theta.rs      # Theta role assignment
│   │   ├── verbnet.rs    # VerbNet integration
│   │   └── movement.rs   # Movement chains
│   ├── drt/              # Layer 3: Compositional semantics
│   │   ├── mod.rs
│   │   ├── lambda.rs     # Lambda calculus (ported from Python)
│   │   ├── composition.rs # Semantic composition
│   │   └── scope.rs      # Quantifier scope
│   └── lib.rs
│
├── canopy-lsp/            # Layer 4: LSP integration
│   ├── server.rs         # LSP server implementation
│   ├── handlers/         # Request handlers
│   │   ├── hover.rs
│   │   ├── diagnostics.rs
│   │   └── actions.rs
│   └── responses.rs      # Response formatting
│
└── canopy-cli/            # Command-line interface
    └── main.rs
```

## Testing Architecture

### Test Categories

1. **Unit Tests**: Component-level testing
2. **Property Tests**: Linguistic invariant testing with `proptest`
3. **Golden Tests**: Deterministic output validation with `insta`
4. **Integration Tests**: End-to-end pipeline testing
5. **Benchmark Tests**: Performance regression detection

### Property Testing Examples

```rust
proptest! {
    #[test]
    fn parsing_preserves_word_count(text in "\\w+( \\w+){0,20}") {
        let words = parse_sentence(&text)?;
        let word_count = text.split_whitespace().count();
        prop_assert_eq!(words.len(), word_count);
    }

    #[test]
    fn theta_assignment_is_complete(event in any::<Event>()) {
        let assigned = assign_theta_roles(&event)?;
        // Every participant should have a theta role
        prop_assert!(assigned.participants.iter().all(|(role, _)| role.is_valid()));
    }
}
```

### Golden Test Strategy

Capture outputs from each layer for regression testing:

```rust
#[test]
fn test_full_pipeline_golden() {
    let input = "John gives Mary a book.";
    let analysis = analyze_sentence(input).unwrap();
    insta::assert_debug_snapshot!(analysis);
}
```

## Extension Points

### Plugin Architecture (Future)

```rust
trait LanguageExtension {
    fn language_code(&self) -> &str;
    fn custom_features(&self) -> Vec<Box<dyn FeatureExtractor>>;
    fn custom_rules(&self) -> Vec<Box<dyn SemanticRule>>;
}
```

### Theory Testing Framework (M6+)

```rust
trait LinguisticTheory {
    fn name(&self) -> &str;
    fn predict(&self, input: &EnhancedWord) -> Prediction;
    fn evaluate(&self, gold: &Annotation) -> Score;
}
```

## Migration from Python V1

### Preserved Concepts

- **19 Theta Roles**: Exact same inventory as Python system
- **VerbNet Integration**: Port existing patterns and strategies
- **Lambda Calculus**: Core β-reduction and type inference algorithms
- **Test Cases**: Golden tests from Python system for cross-validation

### Enhanced Concepts

- **Type Safety**: Compile-time guarantees for linguistic constraints
- **Performance**: 10x improvement through zero-copy and better algorithms
- **Theory Integration**: Deeper linguistic theory implementation
- **Compositionality**: Proper semantic composition throughout

### Discontinued Concepts

- **Protobuf Serialization**: Pure Rust types are better than protobuf overhead
- **JSON Intermediate**: Direct type-to-type transformations
- **spaCy Dependency**: UDPipe is more transparent and theory-aligned

---

This architecture provides the foundation for high-performance, theoretically-grounded linguistic analysis while maintaining the proven concepts from the Python V1 system.
