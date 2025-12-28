# Movement Theory Implementation Strategy

## Executive Summary

canopy.rs implements movement theory sequentially through established theoretical frameworks, maintaining 25-80μs performance while adding sophisticated syntactic analysis. The hybrid architecture uses dependency parsing as foundation with selective phrase structure for complex cases.

**Strategy**: GB → A/A-bar → Minimalist → Multi-dominance progression, with each phase building incrementally on the previous.

## Current M3 Foundation

### Performance Achievement

- **Current baseline**: 7-76μs per sentence (production validated)
- **UDPipe 1.2**: 1.56ms latency, 641 sent/sec, 0% error rate
- **Massive headroom** for semantic analysis while maintaining sub-100μs target

### Event Structure Complete

- ✅ Neo-Davidsonian Event, Participant, Predicate types
- ✅ EventBuilder pattern for clean construction
- ✅ 19 theta role inventory from Python V1 system
- ✅ MovementChain representation integrated
- ✅ LittleV decomposition (Cause, Become, Do, Be, Go, Have)

## Sequential Theory Implementation

### Phase 1: Government & Binding (M3-M4)

#### Movement Detection Signals

```rust
// Detection without full representation
enum MovementSignal {
    PassiveVoice(Participant),        // "John was seen"
    WhConstruction(WhElement),         // "What did John see?"
    RaisingPattern(Subject),          // "John seems to leave"
    ToughConstruction(Object),        // "John is easy to please"
    ExistentialThere(Subject),        // "There are books"
    ExtraposedSubject(Subject),       // "It seems that John left"
}

struct GBMovementChain {
    antecedent: Participant,
    trace: TraceElement,
    governing_category: SyntacticDomain,
    case_assignment: CasePosition,
}
```

#### Implementation Priority

1. **Passive voice detection** (high frequency, clear morphological signals)
1. **Wh-movement** (clear syntactic patterns)
1. **Raising constructions** (matrix vs. embedded subject properties)
1. **Tough-constructions** (object-subject alternations)

### Phase 2: A/A-bar Movement (M4-M5)

#### Movement Classification

```rust
enum ChainType {
    AMovement,      // Passive, raising, unaccusatives
    ABarMovement,   // Wh-movement, topicalization
    HeadMovement,   // V-to-T (defer to M6+)
}

struct MovementChain {
    moved_element: Participant,
    chain_type: ChainType,
    positions: Vec<ChainPosition>,
    locality_constraints: Vec<LocalityViolation>,
}

// A-movement: NP positions (case-driven)
struct AMovementProperties {
    case_motivation: CaseFeature,      // EPP, case requirement
    theta_preservation: bool,          // Theta criterion satisfaction
    c_command_relations: Vec<CCommandRelation>,
}

// A-bar movement: operator positions (scope-driven)
struct ABarMovementProperties {
    operator_type: OperatorType,       // Wh, Topic, Focus
    scope_domain: ScopeDomain,
    reconstruction_sites: Vec<Position>,
}
```

#### Detection Heuristics

```rust
impl MovementDetector {
    fn classify_movement(&self, signal: MovementSignal) -> ChainType {
        match signal {
            MovementSignal::PassiveVoice(_) => ChainType::AMovement,
            MovementSignal::RaisingPattern(_) => ChainType::AMovement,
            MovementSignal::WhConstruction(_) => ChainType::ABarMovement,
            MovementSignal::ToughConstruction(_) => ChainType::ABarMovement,
            // Complex cases require more analysis
        }
    }
}
```

### Phase 3: Minimalist Movement (M5-M6)

#### Feature-Driven Movement

```rust
struct MinimalistMovement {
    trigger_feature: Feature,         // What drives movement
    target_position: TargetHead,      // Where to move
    copy_chain: Vec<Copy>,            // Copy theory of movement
    feature_checking: FeatureMatrix,  // Which features get checked
}

// Feature-based triggers
enum TriggerFeature {
    EPP,                    // Extended Projection Principle
    WH,                     // Wh-feature
    FOCUS,                  // Focus feature
    TOPIC,                  // Topic feature
    NEG,                    // Negation
    Q,                      // Question feature
}

struct FeatureMatrix {
    interpretable: Vec<Feature>,      // Semantic features
    uninterpretable: Vec<Feature>,    // Formal features (deleted)
    checked: Vec<(Feature, Position)>, // Where checking occurred
}
```

#### Copy Theory Implementation

```rust
struct CopyChain {
    copies: Vec<CopyPosition>,
    pronunciation_site: usize,        // Which copy is pronounced
    interpretation_sites: Vec<usize>, // Which copies are interpreted
}

enum CopyPosition {
    Head(HeadPosition),
    Tail(TailPosition),
    Intermediate(IntermediatePosition),
}
```

### Phase 4: Multi-dominance (M6+)

#### Shared Structure Implementation

```rust
struct SharedStructure {
    shared_node: SyntacticNode,
    parent_positions: Vec<StructuralPosition>,
    sharing_mechanism: SharingType,
}

enum SharingType {
    Adjunction,             // Adjunct sharing
    Substitution,           // Argument sharing
    Sideward,              // Sideward movement
    Parallel,              // Parallel merge
}

struct MultiDominanceTree {
    nodes: Vec<SyntacticNode>,
    dominance_relations: Vec<DominanceRelation>,
    shared_positions: Vec<SharedPosition>,
}
```

## Integration with Event Structure

### Movement-Event Interface

```rust
struct MovementAwareEvent {
    base_event: Event,
    movement_chains: Vec<MovementChain>,
    surface_realization: SurfaceForm,
    underlying_structure: UnderlyingForm,
}

impl MovementAwareEvent {
    fn reconstruct_theta_assignment(&self) -> ThetaAssignment {
        // Use movement chains to determine underlying theta positions
        let mut assignment = ThetaAssignment::new();

        for chain in &self.movement_chains {
            let underlying_pos = chain.tail_position();
            let surface_pos = chain.head_position();

            assignment.map_surface_to_theta(surface_pos, underlying_pos);
        }

        assignment
    }
}
```

### VerbNet Integration with Movement

```rust
struct MovementAwareVerbNetAnalysis {
    standard_analysis: VerbNetAnalysis,
    movement_alternations: Vec<MovementAlternation>,
    voice_properties: VoiceProperties,
}

enum MovementAlternation {
    Passive(PassiveProperties),
    Dative(DativeShiftProperties),
    Locative(LocativeAlternationProperties),
    Causative(CausativeAlternationProperties),
}
```

## Performance-Preserving Implementation

### Lazy Evaluation Strategy

```rust
enum AnalysisDepth {
    Shallow(BasicParse),              // 25-80μs
    WithMovement(MovementAnalysis),   // 50-150μs
    FullStructure(ComplexAnalysis),   // 100-300μs
}

struct LazyMovementAnalyzer {
    complexity_threshold: f32,
    cache: MovementCache,
}

impl LazyMovementAnalyzer {
    fn analyze(&mut self, sentence: &Sentence) -> AnalysisDepth {
        let complexity = self.assess_complexity(sentence);

        if complexity < 0.1 {
            AnalysisDepth::Shallow(self.basic_parse(sentence))
        } else if complexity < 0.5 {
            AnalysisDepth::WithMovement(self.movement_analysis(sentence))
        } else {
            AnalysisDepth::FullStructure(self.full_analysis(sentence))
        }
    }
}
```

### Caching Strategy

```rust
struct MovementCache {
    chain_patterns: LRUCache<SentencePattern, MovementChain>,
    complexity_scores: LRUCache<SentenceHash, f32>,
    phrase_structures: LRUCache<DependencyPattern, XBarStructure>,
}
```

## Layer 1 → Layer 2 API

### Movement Signals in Layer 1

```rust
struct Layer1Output {
    words: Vec<EnhancedWord>,
    sentence_features: SentenceFeatures,
    parse_metadata: ParseMetadata,

    // Movement signals for Layer 2
    movement_signals: Vec<MovementSignal>,
}

struct EnhancedWord {
    // UDPipe outputs
    udpipe_analysis: UDPipeWord,

    // VerbNet enhancements
    theta_potential: Vec<ThetaRole>,
    verbnet_class: Option<VerbNetClass>,
    selectional_restrictions: Vec<Constraint>,

    // Movement potential (new in M3)
    movement_signals: Vec<MovementSignal>,
    chain_potential: Option<ChainRole>,
}

enum ChainRole {
    Head,           // Landing site
    Tail,           // Base position
    Intermediate,   // Intermediate position
    Adjunct,        // Adjunct position
}
```

### Layer 2 Event Construction

```rust
struct Layer2Output {
    events: Vec<Event>,
    discourse_entities: Vec<Entity>,
    semantic_relations: Vec<Relation>,
    movement_chains: Vec<MovementChain>,  // Basic chains
    complexity_score: f32,
}

impl Layer2Processor {
    fn process(&self, layer1: Layer1Output) -> Layer2Output {
        // Detect complexity early
        let complexity = self.assess_movement_complexity(&layer1.movement_signals);

        let chains = if complexity > self.threshold {
            self.build_movement_chains(&layer1.movement_signals)
        } else {
            Vec::new()  // Skip expensive analysis
        };

        Layer2Output {
            events: self.build_events(&layer1, &chains),
            movement_chains: chains,
            complexity_score: complexity,
            // ...
        }
    }
}
```

## Implementation Milestones

### M3 Current (VerbNet Integration)

🎯 **IN PROGRESS**: VerbNet theta role assignment

- Event structure foundation complete
- VerbNet integration framework ready
- UDPipe feature extraction providing clean input
- 7-76μs performance baseline with massive headroom

### M4 Extensions (Multi-Resource Integration)

1. **Construction Grammar**: Ditransitive, resultative patterns
1. **PropBank Integration**: Corpus-validated argument structure
1. **Information Structure**: Topic/focus articulation
1. **A/A-bar Movement**: Classification and chain building

### M5 Advanced Features (GPU Acceleration)

1. **Feature-driven Movement**: Minimalist triggers
1. **Copy Theory**: Implementation with proper interpretation
1. **GPU Database**: RAPIDS → Custom kernels for massive speedup
1. **Modal Logic**: Possible worlds semantics

### M6+ Research Applications (Neurosymbolic AI)

1. **Multi-dominance**: Shared structure implementation
1. **Linguistic Tokenization**: Replace BPE/WordPiece with semantic tokens
1. **Transformer Enhancement**: Structured attention mechanisms
1. **Training Integration**: Real-time linguistic analysis during ML training

## Success Criteria

### M3 Targets (Current)

- [🎯] Implement VerbNet theta role assignment
- [🎯] Achieve 80% test coverage milestone
- [✅] Maintain \<100μs average processing time
- [✅] Event structure foundation complete

### M4 Targets

- [ ] Distinguish A vs A-bar movement correctly
- [ ] Handle wh-questions and topicalization
- [ ] Implement locality constraint checking
- [ ] \<120μs for movement-complex sentences

### M5+ Targets

- [ ] Feature-driven movement analysis
- [ ] Copy theory with proper interpretation
- [ ] Handle complex syntactic phenomena
- [ ] Research-grade movement analysis

## Conclusion

The sequential implementation strategy allows canopy.rs to gradually build sophisticated movement analysis while preserving its extraordinary performance. By using dependency parsing as foundation and adding phrase structure only when needed, the system maintains sub-100μs performance for most sentences while enabling deep syntactic analysis for complex cases.

With the M3 foundation complete (7-76μs baseline, event structures, unified features), canopy.rs is perfectly positioned to implement movement theory incrementally while maintaining its performance advantage. This approach positions canopy.rs as the first system to combine linguistic sophistication with practical performance, enabling real-world applications of formal syntactic theory.
