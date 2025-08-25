# Construction Grammar Integration (M4 Priority)

## Overview

Construction Grammar (CxG) represents a fundamental shift from rule-based syntax to pattern-based form-meaning pairings. For canopy.rs M4, we integrate Construction Grammar to capture systematic alternations and idiomatic patterns that complement our VerbNet-based theta role system.

**Core Innovation**: Real-time construction pattern matching that operates in <100μs, enabling production-scale analysis of constructional meaning.

## Theoretical Foundation

### Construction Definition

A **construction** is a form-meaning pairing that exists at various levels of abstraction:

```rust
struct Construction {
    form: FormPattern,           // Syntactic/morphological pattern
    meaning: SemanticStructure,  // Semantic contribution
    constraints: Vec<Constraint>, // Usage restrictions
    frequency: f64,              // Corpus-derived frequency
    inheritance: Vec<ConstructionId>, // Inheritance hierarchy
}
```

### Construction Hierarchy

```text
Construction Inheritance Network:
┌─────────────────────────────────────────────┐
│ Ditransitive                                │
│ ├── Give-construction [NP V NP NP]          │
│ ├── Send-construction [NP V NP to NP]       │
│ └── Make-construction [NP V NP Adj]         │
│                                             │
│ Resultative                                 │
│ ├── Caused-motion [NP V NP PP]              │
│ ├── Adjectival-resultative [NP V NP Adj]    │
│ └── Way-construction [NP V Det way PP]      │
│                                             │
│ Argument Structure                          │
│ ├── Passive [NP be V-ed (by NP)]           │
│ ├── Middle [NP V-s easily/well]            │
│ └── Antipassive [NP V (at NP)]             │
└─────────────────────────────────────────────┘
```

## M4 Construction Inventory

### Priority Constructions (High Frequency, Clear Patterns)

#### 1. Ditransitive Construction

**Pattern**: `[NP₁ V NP₂ NP₃]`
**Meaning**: X causes Y to receive Z
**Example**: "John gave Mary the book"

```rust
struct DitransitiveConstruction {
    // Form constraints
    verb_class: Vec<VerbNetClass>,     // give-13.1, send-11.1
    recipient_constraints: AnimacyConstraint,
    theme_constraints: ConcretenesConstraint,

    // Semantic structure
    event_type: TransferEvent,
    participants: [Agent, Recipient, Theme],

    // Syntactic properties
    alternates_with: Vec<ConstructionId>, // Prepositional dative
    passivization: PassiveProperties,
}

impl Construction for DitransitiveConstruction {
    fn matches(&self, pattern: &SyntacticPattern) -> MatchResult {
        // Pattern: NP V NP NP
        if pattern.length() != 4 { return MatchResult::None; }

        let verb_match = self.verb_class.contains(&pattern.verb().verbnet_class);
        let recipient_animacy = pattern.object1().animacy.is_animate();
        let theme_concreteness = pattern.object2().concreteness.is_concrete();

        if verb_match && recipient_animacy && theme_concreteness {
            MatchResult::Strong(0.9)
        } else {
            MatchResult::Weak(0.3)
        }
    }
}
```

#### 2. Caused-Motion Construction

**Pattern**: `[NP₁ V NP₂ PP]`
**Meaning**: X causes Y to move along path Z
**Example**: "John pushed the cart into the garage"

```rust
struct CausedMotionConstruction {
    // Form pattern
    verb_constraint: MotionVerbClass,
    path_prepositions: Vec<Preposition>, // into, onto, through, etc.

    // Semantic contribution
    motion_component: MotionEvent,
    causation_component: CausativeEvent,

    // Coercion properties
    coerces_manner: bool,               // "John sneezed the napkin off the table"
    result_state: Option<LocationState>,
}
```

#### 3. Way-Construction

**Pattern**: `[NP V Det way PP]`
**Meaning**: X moves along a path by means of V-ing
**Example**: "John danced his way to the store"

```rust
struct WayConstruction {
    // Highly specific form
    determiner: DeterminerType,         // his, her, their, one's
    path_pp: PathPP,                   // to/into/through location

    // Semantic properties
    manner_of_motion: MannerComponent,  // From the verb
    path_creation: PathCreation,        // Metaphorical path

    // Usage constraints
    verb_restrictions: Vec<VerbConstraint>, // Usually manner verbs
    animacy_requirement: AnimacyConstraint, // Agent must be animate
}
```

#### 4. Resultative Construction

**Pattern**: `[NP V NP Adj/PP]`
**Meaning**: X causes Y to become Z by V-ing
**Example**: "John hammered the metal flat"

```rust
struct ResultativeConstruction {
    // Form variations
    adjective_resultative: AdjectivePattern,  // flat, clean, smooth
    pp_resultative: PPPattern,               // to pieces, into shape

    // Semantic structure
    causing_event: Event,
    result_state: ResultState,
    change_component: ChangeEvent,

    // Direct object constraint
    affected_argument: ThemeArgument,        // Must be affected by result
}
```

### Medium Priority (M4-M5)

#### 5. Conative Construction

**Pattern**: `[NP V at NP]`
**Meaning**: X attempts to V Y (but may not succeed)
**Example**: "John shot at the target" (vs "John shot the target")

#### 6. X's way Construction Variants

**Patterns**: Various extensions of way-construction
- "John worked his way up the ladder"
- "The idea forced its way into consciousness"

#### 7. Time-Away Construction

**Pattern**: `[NP V NP away]`
**Meaning**: X spends time V-ing
**Example**: "John danced the night away"

## Implementation Architecture

### Construction Detection Pipeline

```rust
struct ConstructionDetector {
    constructions: Vec<Box<dyn Construction>>,
    pattern_matcher: SyntacticPatternMatcher,
    semantic_constraints: ConstraintChecker,
    confidence_threshold: f64,
}

impl ConstructionDetector {
    fn detect(&self, sentence: &EnhancedSentence) -> Vec<ConstructionMatch> {
        let syntactic_pattern = self.pattern_matcher.extract(sentence);

        let mut matches = Vec::new();
        for construction in &self.constructions {
            if let MatchResult::Strong(confidence) = construction.matches(&syntactic_pattern) {
                if confidence > self.confidence_threshold {
                    matches.push(ConstructionMatch {
                        construction: construction.id(),
                        confidence,
                        semantic_contribution: construction.meaning(),
                        form_properties: construction.form_properties(),
                    });
                }
            }
        }

        // Resolve conflicts (multiple constructions matching same pattern)
        self.resolve_conflicts(matches)
    }
}
```

### Integration with Event Structure

```rust
struct ConstructionalEvent {
    base_event: Event,                    // From VerbNet analysis
    construction: Option<ConstructionMatch>, // Construction overlay
    meaning_composition: CompositionType,  // How meanings combine
}

impl ConstructionalEvent {
    fn compose_meaning(&self) -> SemanticRepresentation {
        match &self.construction {
            Some(construction_match) => {
                // Construction contributes additional semantic structure
                let base_meaning = self.base_event.semantic_representation();
                let construction_meaning = construction_match.meaning();

                // Compose via unification or overlay
                match self.meaning_composition {
                    CompositionType::Unification => {
                        base_meaning.unify_with(construction_meaning)
                    },
                    CompositionType::Overlay => {
                        construction_meaning.overlay_on(base_meaning)
                    },
                    CompositionType::Coercion => {
                        construction_meaning.coerce(base_meaning)
                    }
                }
            },
            None => self.base_event.semantic_representation()
        }
    }
}
```

### Performance Optimization

```rust
struct FastConstructionMatcher {
    // Pre-compiled pattern automata for O(n) matching
    pattern_automata: Vec<PatternAutomaton>,

    // Hash tables for quick verb class lookup
    verb_class_index: HashMap<VerbNetClass, Vec<ConstructionId>>,

    // Cached constraint evaluations
    constraint_cache: LRUCache<ConstraintKey, ConstraintResult>,
}

impl FastConstructionMatcher {
    fn match_constructions(&self, pattern: &SyntacticPattern) -> Vec<ConstructionMatch> {
        // O(n) pattern matching using pre-compiled automata
        let mut candidates = Vec::new();

        for automaton in &self.pattern_automata {
            if automaton.matches(pattern) {
                candidates.push(automaton.construction_id());
            }
        }

        // Filter by verb class constraints (O(1) hash lookup)
        let verb_class = pattern.main_verb().verbnet_class();
        if let Some(class_constructions) = self.verb_class_index.get(&verb_class) {
            candidates.retain(|id| class_constructions.contains(id));
        }

        // Evaluate semantic constraints with caching
        candidates.into_iter()
            .filter_map(|id| self.evaluate_constraints(id, pattern))
            .collect()
    }
}
```

## Semantic Composition Rules

### Construction-Verb Interaction

```rust
enum CompositionType {
    // Construction inherits verb semantics
    Inheritance,    // "John gave Mary a book" - construction + verb align

    // Construction overrides verb semantics
    Coercion,      // "John sneezed the napkin off" - motion from construction

    // Construction adds to verb semantics
    Overlay,       // "John danced his way home" - manner + motion

    // Construction unifies with verb semantics
    Unification,   // "John broke the vase to pieces" - result specification
}
```

### Coercion Examples

```rust
impl ConstructionCoercion {
    fn coerce_manner_to_motion(&self, manner_verb: Verb) -> MotionEvent {
        // "John danced to the store" - dance (manner) + to-PP (path) = motion
        MotionEvent {
            manner: manner_verb.manner_component(),
            path: self.extract_path_from_pp(),
            agent: manner_verb.agent(),
        }
    }

    fn coerce_contact_to_motion(&self, contact_verb: Verb) -> CausedMotionEvent {
        // "John pushed the cart into the garage" - push + into-PP = caused motion
        CausedMotionEvent {
            causing_event: contact_verb.base_event(),
            motion_result: self.extract_motion_from_pp(),
            agent: contact_verb.agent(),
            theme: contact_verb.patient(),
        }
    }
}
```

## Corpus-Based Construction Learning

### Pattern Extraction

```rust
struct ConstructionLearner {
    corpus: AnnotatedCorpus,
    pattern_extractor: PatternExtractor,
    frequency_counter: FrequencyCounter,
    statistical_analyzer: StatisticalAnalyzer,
}

impl ConstructionLearner {
    fn learn_constructions(&self) -> Vec<LearnedConstruction> {
        // Extract frequent syntactic patterns
        let patterns = self.pattern_extractor.extract_patterns(&self.corpus);

        // Count pattern frequencies
        let frequencies = self.frequency_counter.count(&patterns);

        // Identify statistically significant patterns
        let significant = self.statistical_analyzer.find_significant(&frequencies);

        // Convert to construction objects
        significant.into_iter()
            .map(|pattern| self.pattern_to_construction(pattern))
            .collect()
    }

    fn pattern_to_construction(&self, pattern: FrequentPattern) -> LearnedConstruction {
        LearnedConstruction {
            form: pattern.syntactic_form,
            meaning: self.infer_meaning(&pattern),
            frequency: pattern.frequency,
            examples: pattern.example_sentences,
            constraints: self.infer_constraints(&pattern),
        }
    }
}
```

### Integration with VerbNet

```rust
struct VerbNetConstructionBridge {
    verbnet: VerbNetDatabase,
    constructions: ConstructionDatabase,
    mapping_rules: Vec<MappingRule>,
}

impl VerbNetConstructionBridge {
    fn map_verbnet_to_constructions(&self, verb_class: VerbNetClass) -> Vec<ConstructionId> {
        // Map VerbNet alternations to constructions
        match verb_class {
            VerbNetClass::Give13_1 => vec![
                ConstructionId::Ditransitive,
                ConstructionId::PrepositionalDative,
            ],
            VerbNetClass::Put9_1 => vec![
                ConstructionId::CausedMotion,
                ConstructionId::LocationalInversion,
            ],
            VerbNetClass::Break45_1 => vec![
                ConstructionId::Resultative,
                ConstructionId::AdjectivalResultative,
            ],
            _ => Vec::new()
        }
    }

    fn resolve_construction_conflicts(&self,
        verbnet_analysis: VerbNetAnalysis,
        construction_matches: Vec<ConstructionMatch>
    ) -> ResolvedAnalysis {
        // Prefer constructions that align with VerbNet alternations
        let compatible_constructions: Vec<_> = construction_matches
            .into_iter()
            .filter(|c_match| {
                let verb_constructions = self.map_verbnet_to_constructions(verbnet_analysis.class);
                verb_constructions.contains(&c_match.construction_id)
            })
            .collect();

        ResolvedAnalysis {
            verbnet: verbnet_analysis,
            constructions: compatible_constructions,
            confidence: self.calculate_confidence(&compatible_constructions),
        }
    }
}
```

## Performance Targets

### M4 Performance Goals

| Component | Target Time | Strategy |
|-----------|-------------|----------|
| **Pattern Matching** | <50μs | Pre-compiled automata |
| **Constraint Evaluation** | <30μs | Cached constraint checking |
| **Semantic Composition** | <20μs | Pre-computed composition rules |
| **Total Addition** | **<100μs** | **Fits within M4 budget** |

### Optimization Techniques

1. **Pre-compiled Patterns**: Convert construction patterns to finite automata
2. **Constraint Caching**: Memoize expensive constraint evaluations
3. **Lazy Evaluation**: Only compute constructions when needed
4. **Parallel Matching**: Evaluate multiple constructions concurrently

```rust
// Parallel construction matching
impl ParallelConstructionDetector {
    fn detect_parallel(&self, sentence: &EnhancedSentence) -> Vec<ConstructionMatch> {
        self.constructions
            .par_iter()
            .filter_map(|construction| {
                construction.try_match(sentence)
            })
            .collect()
    }
}
```

## Integration with LSP

### Construction-Aware Diagnostics

```rust
enum ConstructionDiagnostic {
    // Argument structure mismatches
    ArgumentStructureMismatch {
        construction: ConstructionId,
        expected_args: Vec<ThematicRole>,
        actual_args: Vec<ThematicRole>,
    },

    // Construction-specific constraints violated
    ConstraintViolation {
        construction: ConstructionId,
        constraint: ConstructionConstraint,
        violation_description: String,
    },

    // Potential construction coercion
    CoercionOpportunity {
        base_meaning: SemanticRepresentation,
        construction_meaning: SemanticRepresentation,
        coercion_type: CoercionType,
    },
}
```

### Code Actions

```rust
enum ConstructionCodeAction {
    // Suggest alternative constructions
    SuggestAlternative {
        current: ConstructionId,
        alternative: ConstructionId,
        explanation: String,
    },

    // Fix argument structure
    FixArgumentStructure {
        construction: ConstructionId,
        corrections: Vec<ArgumentCorrection>,
    },

    // Resolve construction ambiguity
    ResolveAmbiguity {
        ambiguous_constructions: Vec<ConstructionMatch>,
        preferred: ConstructionId,
        rationale: String,
    },
}
```

## Testing Strategy

### Construction Test Suite

```rust
struct ConstructionTestSuite {
    // Positive examples (should match)
    positive_examples: HashMap<ConstructionId, Vec<TestSentence>>,

    // Negative examples (should not match)
    negative_examples: HashMap<ConstructionId, Vec<TestSentence>>,

    // Ambiguous cases (multiple valid constructions)
    ambiguous_cases: Vec<AmbiguousTestCase>,

    // Performance benchmarks
    performance_tests: Vec<PerformanceTest>,
}

impl ConstructionTestSuite {
    fn test_construction(&self, id: ConstructionId) -> TestResult {
        let construction = self.get_construction(id);

        // Test positive examples
        let positive_results = self.positive_examples[&id]
            .iter()
            .map(|sentence| construction.matches(sentence))
            .collect::<Vec<_>>();

        // Test negative examples
        let negative_results = self.negative_examples[&id]
            .iter()
            .map(|sentence| construction.matches(sentence))
            .collect::<Vec<_>>();

        TestResult {
            precision: self.calculate_precision(&positive_results, &negative_results),
            recall: self.calculate_recall(&positive_results),
            f1: self.calculate_f1(&positive_results, &negative_results),
            performance: self.measure_performance(id),
        }
    }
}
```

## Future Extensions (M5+)

### Cross-Linguistic Constructions

```rust
struct CrossLinguisticConstruction {
    english_pattern: ConstructionPattern,
    cross_linguistic_variants: HashMap<Language, ConstructionPattern>,
    semantic_universal: UniversalSemantics,
    typological_parameters: Vec<TypologicalParameter>,
}
```

### Machine Learning Enhancement

```rust
struct MLConstructionLearner {
    neural_pattern_recognizer: NeuralPatternRecognizer,
    construction_embeddings: ConstructionEmbeddings,
    transfer_learning: CrossLinguisticTransfer,
}
```

## Conclusion

Construction Grammar integration in M4 provides canopy.rs with systematic handling of form-meaning pairings that complement VerbNet's verb-centered analysis. By implementing high-frequency constructions with optimized pattern matching (<100μs addition), we enable real-time analysis of constructional meaning while maintaining our performance advantage.

The construction system bridges the gap between lexical semantics (VerbNet) and syntactic productivity, providing a complete framework for analyzing how speakers create novel meanings through established form-meaning patterns. This positions canopy.rs as the first system to combine constructional analysis with production-scale performance.
