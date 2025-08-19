# Information Structure Analysis (M4 Priority)

## Overview

Information Structure (IS) analyzes how speakers package information as given/new, topic/focus, and foreground/background. For canopy.rs M4, we implement Information Structure analysis to enhance discourse modeling and provide deeper semantic insights into how meaning is structured across sentences.

**Core Innovation**: Real-time topic/focus detection that operates in <20μs, enabling production-scale analysis of information packaging in discourse.

## Theoretical Foundation

### Information Structure Framework

Following Krifka (2008) and Lambrecht (1994), Information Structure operates on three main dimensions:

```rust
struct InformationStructure {
    topic_focus: TopicFocusStructure,
    given_new: GivenNewStructure,
    presupposition_assertion: PresuppositionStructure,
}

struct TopicFocusStructure {
    topic: Option<TopicExpression>,     // What the sentence is about
    focus: Option<FocusExpression>,     // New/contrasted information
    comment: CommentExpression,         // Predication about topic
}

struct GivenNewStructure {
    given_entities: Vec<Entity>,        // Discourse-old information
    new_entities: Vec<Entity>,          // Discourse-new information
    accessible_entities: Vec<Entity>,   // Inferrable information
}

enum TopicType {
    AboutnessTopic,      // Sentence is about this entity
    FrameSetting,        // Spatial/temporal frame
    Contrastive,         // Contrasted with alternative
}

enum FocusType {
    NewInformation,      // Answers "what happened?"
    Contrastive,         // Contrasts with alternative
    Corrective,          // Corrects previous assertion
    Completive,          // Completes partial information
}
```

## M4 Implementation Strategy

### Topic Detection

```rust
struct TopicDetector {
    definiteness_analyzer: DefinitenessAnalyzer,
    discourse_context: DiscourseContext,
    syntactic_analyzer: SyntacticPositionAnalyzer,
}

impl TopicDetector {
    fn detect_topic(&self, sentence: &EnhancedSentence) -> Option<TopicExpression> {
        let mut candidates = Vec::new();

        // Heuristic 1: Definite subjects (strong topic candidates)
        if let Some(subject) = sentence.subject() {
            if subject.definiteness == Definiteness::Definite {
                candidates.push(TopicCandidate {
                    entity: subject.entity.clone(),
                    evidence: vec![Evidence::DefiniteSubject],
                    confidence: 0.8,
                });
            }
        }

        // Heuristic 2: Discourse-old entities
        for entity in sentence.entities() {
            if self.discourse_context.is_discourse_old(entity) {
                candidates.push(TopicCandidate {
                    entity: entity.clone(),
                    evidence: vec![Evidence::DiscourseOld],
                    confidence: 0.6,
                });
            }
        }

        // Heuristic 3: Left-dislocated elements
        if let Some(left_dislocation) = sentence.left_dislocation() {
            candidates.push(TopicCandidate {
                entity: left_dislocation.entity.clone(),
                evidence: vec![Evidence::LeftDislocation],
                confidence: 0.9,
            });
        }

        // Select best candidate
        candidates.into_iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .map(|candidate| TopicExpression {
                entity: candidate.entity,
                topic_type: self.classify_topic_type(&candidate),
                confidence: candidate.confidence,
            })
    }

    fn classify_topic_type(&self, candidate: &TopicCandidate) -> TopicType {
        if candidate.evidence.contains(&Evidence::LeftDislocation) {
            TopicType::Contrastive
        } else if candidate.evidence.contains(&Evidence::TemporalFrame) {
            TopicType::FrameSetting
        } else {
            TopicType::AboutnessTopic
        }
    }
}
```

### Focus Detection

```rust
struct FocusDetector {
    stress_analyzer: StressAnalyzer,        // Prosodic focus (future)
    syntactic_analyzer: SyntacticAnalyzer, // Syntactic focus marking
    semantic_analyzer: SemanticAnalyzer,   // Semantic focus types
}

impl FocusDetector {
    fn detect_focus(&self, sentence: &EnhancedSentence) -> Option<FocusExpression> {
        let mut candidates = Vec::new();

        // Syntactic focus markers
        candidates.extend(self.detect_syntactic_focus(sentence));

        // Semantic focus (new information)
        candidates.extend(self.detect_semantic_focus(sentence));

        // Contrastive focus
        candidates.extend(self.detect_contrastive_focus(sentence));

        // Select highest confidence candidate
        candidates.into_iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .map(|candidate| candidate.into_focus_expression())
    }

    fn detect_syntactic_focus(&self, sentence: &EnhancedSentence) -> Vec<FocusCandidate> {
        let mut candidates = Vec::new();

        // Cleft constructions: "It was JOHN who left"
        if let Some(cleft) = sentence.cleft_construction() {
            candidates.push(FocusCandidate {
                constituent: cleft.focused_constituent.clone(),
                focus_type: FocusType::NewInformation,
                evidence: vec![Evidence::CleftConstruction],
                confidence: 0.95,
            });
        }

        // Wh-questions: Focus on the wh-word
        if sentence.is_wh_question() {
            if let Some(wh_word) = sentence.wh_word() {
                candidates.push(FocusCandidate {
                    constituent: wh_word.clone(),
                    focus_type: FocusType::NewInformation,
                    evidence: vec![Evidence::WhQuestion],
                    confidence: 0.9,
                });
            }
        }

        // Only-constructions: "Only JOHN left"
        if let Some(only_scope) = sentence.only_construction() {
            candidates.push(FocusCandidate {
                constituent: only_scope.focused_constituent.clone(),
                focus_type: FocusType::Contrastive,
                evidence: vec![Evidence::OnlyConstruction],
                confidence: 0.85,
            });
        }

        candidates
    }

    fn detect_semantic_focus(&self, sentence: &EnhancedSentence) -> Vec<FocusCandidate> {
        let mut candidates = Vec::new();

        // New discourse entities are likely focused
        for entity in sentence.entities() {
            if entity.discourse_status == DiscourseStatus::New {
                candidates.push(FocusCandidate {
                    constituent: Constituent::Entity(entity.clone()),
                    focus_type: FocusType::NewInformation,
                    evidence: vec![Evidence::NewEntity],
                    confidence: 0.6,
                });
            }
        }

        // Indefinite objects often carry focus
        if let Some(object) = sentence.direct_object() {
            if object.definiteness == Definiteness::Indefinite {
                candidates.push(FocusCandidate {
                    constituent: Constituent::Entity(object.entity.clone()),
                    focus_type: FocusType::NewInformation,
                    evidence: vec![Evidence::IndefiniteObject],
                    confidence: 0.5,
                });
            }
        }

        candidates
    }
}
```

### Given/New Analysis

```rust
struct GivenNewAnalyzer {
    discourse_context: DiscourseContext,
    accessibility_hierarchy: AccessibilityHierarchy,
}

impl GivenNewAnalyzer {
    fn analyze_given_new(&self, sentence: &EnhancedSentence) -> GivenNewStructure {
        let mut given = Vec::new();
        let mut new = Vec::new();
        let mut accessible = Vec::new();

        for entity in sentence.entities() {
            match self.classify_information_status(entity) {
                InformationStatus::Given => given.push(entity.clone()),
                InformationStatus::New => new.push(entity.clone()),
                InformationStatus::Accessible => accessible.push(entity.clone()),
            }
        }

        GivenNewStructure {
            given_entities: given,
            new_entities: new,
            accessible_entities: accessible,
        }
    }

    fn classify_information_status(&self, entity: &Entity) -> InformationStatus {
        // Check discourse context first
        if self.discourse_context.contains_entity(entity) {
            return InformationStatus::Given;
        }

        // Check accessibility via inference
        if self.accessibility_hierarchy.is_accessible(entity) {
            return InformationStatus::Accessible;
        }

        // Default to new
        InformationStatus::New
    }
}

enum InformationStatus {
    Given,       // Previously mentioned or activated
    New,         // First mention, not inferrable
    Accessible,  // Inferrable from context/knowledge
}
```

### Integration with Event Structure

```rust
struct InformationStructureEnhancedEvent {
    base_event: Event,
    information_structure: InformationStructure,
    discourse_effects: Vec<DiscourseEffect>,
}

impl InformationStructureEnhancedEvent {
    fn analyze_information_packaging(&mut self) {
        // Integrate IS with theta roles
        self.mark_focal_participants();
        self.identify_topical_participants();
        self.track_discourse_effects();
    }

    fn mark_focal_participants(&mut self) {
        if let Some(focus) = &self.information_structure.topic_focus.focus {
            // Mark focused participants in event structure
            for (role, participant) in &mut self.base_event.participants {
                if focus.overlaps_with(participant) {
                    participant.information_status = InformationStatus::Focused;
                    participant.focus_type = Some(focus.focus_type.clone());
                }
            }
        }
    }

    fn identify_topical_participants(&mut self) {
        if let Some(topic) = &self.information_structure.topic_focus.topic {
            // Identify which theta role corresponds to topic
            for (role, participant) in &mut self.base_event.participants {
                if topic.corresponds_to(participant) {
                    participant.information_status = InformationStatus::Topical;
                    participant.topic_type = Some(topic.topic_type.clone());

                    // Topics often correspond to Agents in active voice
                    if *role == ThetaRole::Agent {
                        self.discourse_effects.push(DiscourseEffect::TopicalAgent);
                    }
                }
            }
        }
    }
}
```

## Discourse Context Management

### Topic Continuity Tracking

```rust
struct TopicContinuityTracker {
    current_topic: Option<Entity>,
    topic_stack: Vec<Entity>,           // Topic hierarchy
    topic_transitions: Vec<TopicTransition>,
}

impl TopicContinuityTracker {
    fn update_with_sentence(&mut self, sentence: &InformationStructureEnhancedEvent) {
        if let Some(new_topic) = &sentence.information_structure.topic_focus.topic {
            let transition = self.analyze_topic_transition(new_topic);

            match transition {
                TopicTransition::Continuation => {
                    // Same topic continues
                    self.mark_topic_continuation();
                },
                TopicTransition::Shift => {
                    // New topic introduced
                    self.push_new_topic(new_topic.entity.clone());
                },
                TopicTransition::Return => {
                    // Return to previous topic
                    self.return_to_topic(new_topic.entity.clone());
                },
            }

            self.topic_transitions.push(transition);
        }
    }

    fn analyze_topic_transition(&self, new_topic: &TopicExpression) -> TopicTransition {
        if let Some(current) = &self.current_topic {
            if current.corefers_with(&new_topic.entity) {
                TopicTransition::Continuation
            } else if self.topic_stack.iter().any(|t| t.corefers_with(&new_topic.entity)) {
                TopicTransition::Return
            } else {
                TopicTransition::Shift
            }
        } else {
            TopicTransition::Introduction
        }
    }
}

enum TopicTransition {
    Introduction,    // First topic in discourse
    Continuation,    // Same topic continues
    Shift,          // New topic introduced
    Return,         // Return to previous topic
}
```

### Accessibility Hierarchy

```rust
struct AccessibilityHierarchy {
    // Following Ariel (1990) accessibility marking hierarchy
    high_accessibility: Vec<Entity>,    // Pronouns, zero anaphora
    medium_accessibility: Vec<Entity>,  // Definite descriptions
    low_accessibility: Vec<Entity>,     // Indefinite descriptions
}

impl AccessibilityHierarchy {
    fn update_with_reference(&mut self, entity: &Entity, reference_form: ReferenceForm) {
        // Remove from all levels first
        self.remove_entity(entity);

        // Add to appropriate level based on reference form
        match reference_form {
            ReferenceForm::Pronoun | ReferenceForm::ZeroAnaphor => {
                self.high_accessibility.insert(0, entity.clone());
            },
            ReferenceForm::DefiniteDescription => {
                self.medium_accessibility.insert(0, entity.clone());
            },
            ReferenceForm::IndefiniteDescription => {
                self.low_accessibility.insert(0, entity.clone());
            },
        }

        // Maintain bounded size (last 10 entities per level)
        self.high_accessibility.truncate(10);
        self.medium_accessibility.truncate(10);
        self.low_accessibility.truncate(10);
    }

    fn predict_reference_form(&self, entity: &Entity) -> ReferenceForm {
        if self.high_accessibility.contains(entity) {
            ReferenceForm::Pronoun  // Highly accessible → use pronoun
        } else if self.medium_accessibility.contains(entity) {
            ReferenceForm::DefiniteDescription  // Medium accessibility
        } else {
            ReferenceForm::IndefiniteDescription  // Low/no accessibility
        }
    }
}

enum ReferenceForm {
    Pronoun,               // he, she, it
    ZeroAnaphor,          // Ø (empty subject in pro-drop languages)
    DefiniteDescription,   // the man, the book
    IndefiniteDescription, // a man, some book
}
```

## Performance Optimization

### Lightweight IS Analysis

```rust
struct FastInformationStructureAnalyzer {
    // Cached patterns for common IS configurations
    pattern_cache: LRUCache<SyntacticPattern, InformationStructure>,

    // Pre-computed topic/focus indicators
    topic_indicators: HashSet<LinguisticCue>,
    focus_indicators: HashSet<LinguisticCue>,

    // Bounded discourse context (performance vs. accuracy trade-off)
    context_window: BoundedDeque<Entity>,  // Last N entities only
}

impl FastInformationStructureAnalyzer {
    fn quick_analyze(&mut self, sentence: &EnhancedSentence) -> InformationStructure {
        let pattern = sentence.syntactic_pattern();

        // Check cache first
        if let Some(cached) = self.pattern_cache.get(&pattern) {
            return self.adapt_cached_analysis(cached, sentence);
        }

        // Fast heuristic-based analysis
        let topic_focus = self.quick_topic_focus_analysis(sentence);
        let given_new = self.quick_given_new_analysis(sentence);

        let result = InformationStructure {
            topic_focus,
            given_new,
            presupposition_assertion: PresuppositionStructure::default(), // Simplified for M4
        };

        // Cache for future use
        self.pattern_cache.put(pattern, result.clone());

        result
    }

    fn quick_topic_focus_analysis(&self, sentence: &EnhancedSentence) -> TopicFocusStructure {
        // Simplified heuristics for speed
        let topic = if sentence.has_definite_subject() {
            Some(TopicExpression {
                entity: sentence.subject().unwrap().entity.clone(),
                topic_type: TopicType::AboutnessTopic,
                confidence: 0.7,
            })
        } else {
            None
        };

        let focus = if sentence.has_indefinite_object() {
            Some(FocusExpression {
                constituent: Constituent::Entity(sentence.direct_object().unwrap().entity.clone()),
                focus_type: FocusType::NewInformation,
                confidence: 0.6,
            })
        } else {
            None
        };

        TopicFocusStructure {
            topic,
            focus,
            comment: CommentExpression::default(),
        }
    }
}
```

### Integration Performance

```rust
// M4 Performance Budget for Information Structure
struct ISPerformanceProfile {
    topic_detection: Duration,    // Target: <8μs
    focus_detection: Duration,    // Target: <7μs
    given_new_analysis: Duration, // Target: <5μs
    total_addition: Duration,     // Target: <20μs (fits in M4 budget)
}

impl InformationStructureAnalyzer {
    fn analyze_with_monitoring(&self, sentence: &EnhancedSentence) -> (InformationStructure, PerformanceMetrics) {
        let start = Instant::now();

        let topic_start = Instant::now();
        let topic = self.topic_detector.detect_topic(sentence);
        let topic_time = topic_start.elapsed();

        let focus_start = Instant::now();
        let focus = self.focus_detector.detect_focus(sentence);
        let focus_time = focus_start.elapsed();

        let given_new_start = Instant::now();
        let given_new = self.given_new_analyzer.analyze_given_new(sentence);
        let given_new_time = given_new_start.elapsed();

        let total_time = start.elapsed();

        let structure = InformationStructure {
            topic_focus: TopicFocusStructure {
                topic,
                focus,
                comment: CommentExpression::default(),
            },
            given_new,
            presupposition_assertion: PresuppositionStructure::default(),
        };

        let metrics = PerformanceMetrics {
            topic_detection: topic_time,
            focus_detection: focus_time,
            given_new_analysis: given_new_time,
            total_time,
        };

        (structure, metrics)
    }
}
```

## LSP Integration

### Information Structure Diagnostics

```rust
enum InformationStructureDiagnostic {
    // Topic-focus misalignment
    TopicFocusConflict {
        topic: TopicExpression,
        focus: FocusExpression,
        explanation: String,
    },

    // Discourse discontinuity
    AbruptTopicShift {
        previous_topic: Entity,
        new_topic: Entity,
        suggestion: String,
    },

    // Accessibility violations
    AccessibilityViolation {
        entity: Entity,
        reference_form: ReferenceForm,
        expected_form: ReferenceForm,
    },

    // Information packaging issues
    InformationPackagingIssue {
        issue_type: PackagingIssueType,
        problematic_constituent: Constituent,
        suggestion: String,
    },
}

enum PackagingIssueType {
    GivenInformationFocused,     // Old information presented as new
    NewInformationTopicalized,   // New information used as topic
    OverlyHeavyTopic,           // Complex topic expression
    AmbiguousReferenceForm,     // Unclear reference choice
}
```

### Code Actions

```rust
enum InformationStructureCodeAction {
    // Improve topic continuity
    ImproveContinuity {
        insertion_point: TextRange,
        transition_phrase: String,
    },

    // Fix reference form
    CorrectReferenceForm {
        range: TextRange,
        current_form: String,
        suggested_form: String,
        rationale: String,
    },

    // Restructure for better information flow
    RestructureInformation {
        sentence_range: TextRange,
        restructured_text: String,
        improvement_description: String,
    },
}
```

## Testing Strategy

### Information Structure Test Suite

```rust
struct InformationStructureTestSuite {
    // Topic detection tests
    topic_tests: Vec<TopicTest>,

    // Focus detection tests
    focus_tests: Vec<FocusTest>,

    // Discourse continuity tests
    continuity_tests: Vec<ContinuityTest>,

    // Cross-linguistic tests (for future)
    cross_linguistic_tests: Vec<CrossLinguisticTest>,
}

struct TopicTest {
    sentence: String,
    expected_topic: Option<TopicExpression>,
    context: DiscourseContext,
}

struct FocusTest {
    sentence: String,
    expected_focus: Option<FocusExpression>,
    focus_type: FocusType,
}

impl InformationStructureTestSuite {
    fn test_topic_detection(&self) -> TestResult {
        let mut correct = 0;
        let mut total = 0;

        for test in &self.topic_tests {
            let sentence = self.parse_sentence(&test.sentence, &test.context);
            let predicted_topic = self.analyzer.detect_topic(&sentence);

            if self.topics_match(&predicted_topic, &test.expected_topic) {
                correct += 1;
            }
            total += 1;
        }

        TestResult {
            accuracy: correct as f64 / total as f64,
            component: "topic_detection".to_string(),
        }
    }
}
```

## Future Extensions

### Prosodic Integration

```rust
// Future enhancement for M5+
struct ProsodicInformationStructure {
    stress_patterns: Vec<StressPattern>,
    intonational_phrases: Vec<IntonationalPhrase>,
    pitch_accents: Vec<PitchAccent>,
}

// Integration with speech processing
impl ProsodicIntegration {
    fn enhance_focus_detection(&self, sentence: &EnhancedSentence, prosody: &ProsodicAnalysis) -> FocusExpression {
        // Use pitch accent placement to identify focus
        // Can significantly improve focus detection accuracy
        todo!("Implement in M5+ with speech integration")
    }
}
```

## Conclusion

Information Structure analysis in M4 provides essential discourse-level insights that complement our sentence-level semantic analysis. By tracking topic/focus structure, given/new information flow, and discourse accessibility, we enable more sophisticated text understanding and generation.

The lightweight implementation operates within our <20μs performance budget while providing:
- **Topic continuity tracking** across discourse
- **Focus detection** for information highlighting
- **Given/new analysis** for information status
- **Accessibility hierarchy** for reference form prediction
- **Discourse effects** integration with event structure

This positions canopy.rs as the first system to combine real-time Information Structure analysis with formal semantic analysis, enabling applications in discourse generation, text coherence analysis, and cross-linguistic information packaging studies.
