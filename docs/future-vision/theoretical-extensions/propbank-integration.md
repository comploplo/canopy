# PropBank Integration (M4 Priority)

## Overview

PropBank provides corpus-validated argument structure annotations that complement VerbNet's theoretical classifications. For canopy.rs M4, we integrate PropBank to enhance theta role confidence and provide empirical validation of our semantic analysis.

**Core Innovation**: Real-time PropBank frame matching that operates in <75μs, providing corpus-validated argument structure to enhance VerbNet-based theta role assignment.

## Theoretical Foundation

### PropBank Framework

PropBank annotates predicate-argument structure with numbered arguments:
- **Arg0**: Proto-agent (typically corresponds to Agent, Experiencer, Causer)
- **Arg1**: Proto-patient (typically corresponds to Patient, Theme)
- **Arg2**: Beneficiary, Instrument, Attribute, End state
- **Arg3**: Start point, Beneficiary, Attribute
- **Arg4**: End point
- **ArgM-X**: Adjunct arguments (LOC, TMP, MNR, CAU, etc.)

### VerbNet-PropBank Mapping

```rust
struct PropBankFrame {
    predicate: String,              // "give.01", "break.01"
    args: Vec<PropBankArgument>,    // Numbered arguments
    examples: Vec<AnnotatedExample>,
    frequency: f64,                 // Corpus frequency
    verbnet_mapping: Option<VerbNetClass>,
}

struct PropBankArgument {
    number: ArgumentNumber,         // Arg0, Arg1, Arg2, etc.
    description: String,            // "agent", "thing given", "recipient"
    selectional_restrictions: Vec<Constraint>,
    theta_role_mapping: Vec<ThetaRole>, // Multiple possible mappings
}

enum ArgumentNumber {
    Arg0, Arg1, Arg2, Arg3, Arg4, Arg5,
    ArgM(AdjunctType),              // LOC, TMP, MNR, CAU, etc.
}
```

## M4 Implementation Strategy

### PropBank Database Integration

```rust
struct PropBankDatabase {
    frames: HashMap<String, Vec<PropBankFrame>>, // Indexed by verb lemma
    verbnet_mappings: HashMap<VerbNetClass, Vec<PropBankFrame>>,
    frequency_index: BTreeMap<f64, Vec<PropBankFrame>>, // For high-frequency first
    argument_patterns: HashMap<ArgumentPattern, Vec<PropBankFrame>>,
}

impl PropBankDatabase {
    fn lookup_frames(&self, verb: &str) -> Vec<PropBankFrame> {
        // O(1) lookup by verb lemma
        self.frames.get(verb).cloned().unwrap_or_default()
    }

    fn find_best_frame(&self, verb: &str, args: &[SyntacticArgument]) -> Option<PropBankFrame> {
        let candidate_frames = self.lookup_frames(verb);

        // Score frames by argument structure match
        candidate_frames
            .into_iter()
            .map(|frame| (frame.score_match(args), frame))
            .max_by(|(score1, _), (score2, _)| score1.partial_cmp(score2).unwrap())
            .map(|(_, frame)| frame)
    }
}
```

### Enhanced Theta Role Assignment

```rust
struct EnhancedThetaAssigner {
    verbnet: VerbNetDatabase,
    propbank: PropBankDatabase,
    confidence_calculator: ConfidenceCalculator,
}

impl EnhancedThetaAssigner {
    fn assign_roles(&self, event: &mut Event) -> ThetaAssignment {
        // Step 1: Get VerbNet analysis (existing M3 implementation)
        let verbnet_assignment = self.verbnet.assign_roles(&event.predicate, &event.participants);

        // Step 2: Get PropBank analysis
        let propbank_frame = self.propbank.find_best_frame(&event.predicate.lemma, &event.syntactic_args);

        // Step 3: Cross-validate and enhance
        match propbank_frame {
            Some(pb_frame) => {
                let enhanced = self.cross_validate(verbnet_assignment, pb_frame, event);
                enhanced
            },
            None => {
                // Fall back to VerbNet-only analysis
                verbnet_assignment.with_confidence(0.6) // Lower confidence without PropBank validation
            }
        }
    }

    fn cross_validate(&self,
        verbnet: ThetaAssignment,
        propbank: PropBankFrame,
        event: &Event
    ) -> ThetaAssignment {
        let mut enhanced = verbnet.clone();

        // Increase confidence when VerbNet and PropBank agree
        for (participant, vn_role) in &verbnet.assignments {
            if let Some(pb_arg) = propbank.find_mapping(participant) {
                if self.roles_compatible(vn_role, &pb_arg.theta_role_mapping) {
                    enhanced.increase_confidence(participant, 0.3); // Boost confidence
                } else {
                    // Conflict resolution: prefer higher-frequency source
                    if propbank.frequency > 0.1 { // High-frequency PropBank frame
                        enhanced.update_role(participant, pb_arg.best_theta_role());
                        enhanced.add_justification(participant, "PropBank corpus validation");
                    }
                }
            }
        }

        enhanced
    }
}
```

### Argument Structure Validation

```rust
struct ArgumentStructureValidator {
    propbank: PropBankDatabase,
    verbnet: VerbNetDatabase,
}

impl ArgumentStructureValidator {
    fn validate_event(&self, event: &Event) -> ValidationResult {
        let mut issues = Vec::new();

        // Check for missing required arguments
        if let Some(pb_frame) = self.propbank.lookup_best_frame(&event.predicate) {
            for required_arg in pb_frame.required_arguments() {
                if !event.has_argument_type(required_arg) {
                    issues.push(ValidationIssue::MissingRequiredArgument {
                        expected: required_arg,
                        frame: pb_frame.id.clone(),
                    });
                }
            }

            // Check for unexpected arguments
            for participant in &event.participants {
                if !pb_frame.allows_argument(participant) {
                    issues.push(ValidationIssue::UnexpectedArgument {
                        participant: participant.clone(),
                        frame: pb_frame.id.clone(),
                    });
                }
            }
        }

        ValidationResult { issues, confidence: self.calculate_confidence(&issues) }
    }
}
```

## Corpus Pattern Learning

### Statistical Enhancement

```rust
struct PropBankPatternLearner {
    corpus: PropBankCorpus,
    pattern_extractor: ArgumentPatternExtractor,
    frequency_analyzer: FrequencyAnalyzer,
}

impl PropBankPatternLearner {
    fn learn_argument_patterns(&self) -> Vec<ArgumentPattern> {
        // Extract frequent argument structure patterns
        let patterns = self.pattern_extractor.extract_patterns(&self.corpus);

        // Rank by frequency and reliability
        let ranked = self.frequency_analyzer.rank_patterns(patterns);

        // Filter for high-confidence patterns
        ranked.into_iter()
            .filter(|pattern| pattern.confidence > 0.8 && pattern.frequency > 0.01)
            .collect()
    }

    fn discover_new_roles(&self, verb_class: VerbNetClass) -> Vec<ThetaRole> {
        // Find PropBank arguments not covered by VerbNet
        let pb_frames = self.propbank.get_frames_for_class(verb_class);
        let vn_roles = self.verbnet.get_roles_for_class(verb_class);

        let mut new_roles = Vec::new();
        for frame in pb_frames {
            for arg in frame.arguments {
                if !vn_roles.iter().any(|role| self.roles_compatible(role, &arg.theta_role_mapping)) {
                    new_roles.extend(arg.theta_role_mapping);
                }
            }
        }

        new_roles.into_iter().unique().collect()
    }
}
```

### Frame Sense Disambiguation

```rust
struct FrameSenseDisambiguator {
    sense_classifier: SenseClassifier,
    context_analyzer: ContextAnalyzer,
}

impl FrameSenseDisambiguator {
    fn disambiguate_frame(&self, verb: &str, context: &SentenceContext) -> PropBankFrame {
        let candidate_frames = self.propbank.lookup_frames(verb);

        if candidate_frames.len() == 1 {
            return candidate_frames[0].clone();
        }

        // Multi-frame disambiguation
        let context_features = self.context_analyzer.extract_features(context);

        candidate_frames
            .into_iter()
            .map(|frame| {
                let score = self.sense_classifier.score_frame(&frame, &context_features);
                (score, frame)
            })
            .max_by(|(score1, _), (score2, _)| score1.partial_cmp(score2).unwrap())
            .map(|(_, frame)| frame)
            .unwrap_or_else(|| candidate_frames[0].clone()) // Fallback to first frame
    }
}

struct SenseClassifier {
    // Simple heuristic-based classifier for M4
    // Can be enhanced with ML in later milestones
}

impl SenseClassifier {
    fn score_frame(&self, frame: &PropBankFrame, context: &ContextFeatures) -> f64 {
        let mut score = 0.0;

        // Argument structure match
        score += self.score_argument_match(&frame.args, &context.syntactic_args) * 0.4;

        // Selectional restrictions
        score += self.score_selectional_fit(&frame.args, &context.participants) * 0.3;

        // Corpus frequency
        score += frame.frequency.ln() * 0.2; // Log frequency to avoid overwhelming

        // Semantic field compatibility
        score += self.score_semantic_field(&frame.predicate, &context.semantic_field) * 0.1;

        score
    }
}
```

## Performance Optimization

### Frame Lookup Optimization

```rust
struct OptimizedPropBankLookup {
    // Pre-computed indices for O(1) lookup
    lemma_index: HashMap<String, Vec<FrameId>>,
    pattern_index: HashMap<ArgumentPattern, Vec<FrameId>>,
    frequency_sorted: Vec<FrameId>, // Most frequent first

    // LRU cache for repeated lookups
    frame_cache: LRUCache<(String, ArgumentPattern), PropBankFrame>,

    // Pre-loaded frames in memory
    frame_store: Vec<PropBankFrame>,
}

impl OptimizedPropBankLookup {
    fn fast_lookup(&mut self, verb: &str, args: &ArgumentPattern) -> Option<PropBankFrame> {
        // Check cache first (O(1))
        let cache_key = (verb.to_string(), args.clone());
        if let Some(cached) = self.frame_cache.get(&cache_key) {
            return Some(cached.clone());
        }

        // Fast index lookup (O(1))
        let candidate_ids = self.lemma_index.get(verb)?;

        // Score only promising candidates
        let best_frame = candidate_ids
            .iter()
            .take(5) // Limit to top 5 most frequent frames
            .filter_map(|id| self.frame_store.get(*id as usize))
            .map(|frame| (frame.score_pattern(args), frame))
            .max_by(|(score1, _), (score2, _)| score1.partial_cmp(score2).unwrap())
            .map(|(_, frame)| frame.clone());

        // Cache result
        if let Some(ref frame) = best_frame {
            self.frame_cache.put(cache_key, frame.clone());
        }

        best_frame
    }
}
```

### Integration Performance Target

```rust
// M4 Performance Budget for PropBank Integration
struct PropBankPerformanceProfile {
    frame_lookup: Duration,        // Target: <30μs
    argument_validation: Duration, // Target: <20μs
    cross_validation: Duration,    // Target: <25μs
    total_addition: Duration,      // Target: <75μs (fits in M4 budget)
}

// Performance monitoring
impl PropBankIntegration {
    fn analyze_with_monitoring(&self, event: &mut Event) -> (ThetaAssignment, PerformanceMetrics) {
        let start = Instant::now();

        let lookup_start = Instant::now();
        let frame = self.database.fast_lookup(&event.predicate.lemma, &event.argument_pattern());
        let lookup_time = lookup_start.elapsed();

        let validation_start = Instant::now();
        let validation = self.validator.validate_event(event);
        let validation_time = validation_start.elapsed();

        let assignment_start = Instant::now();
        let assignment = self.assigner.assign_with_propbank(event, frame, validation);
        let assignment_time = assignment_start.elapsed();

        let total_time = start.elapsed();

        let metrics = PerformanceMetrics {
            lookup_time,
            validation_time,
            assignment_time,
            total_time,
        };

        (assignment, metrics)
    }
}
```

## LSP Integration

### Enhanced Diagnostics

```rust
enum PropBankDiagnostic {
    // Frame validation issues
    FrameMismatch {
        expected_frame: PropBankFrame,
        actual_arguments: Vec<SyntacticArgument>,
        confidence: f64,
    },

    // Missing required arguments
    MissingRequiredArgument {
        frame: PropBankFrame,
        missing_arg: ArgumentNumber,
        suggestion: Option<String>,
    },

    // VerbNet-PropBank conflicts
    CrossResourceConflict {
        verbnet_role: ThetaRole,
        propbank_arg: PropBankArgument,
        resolution: ConflictResolution,
    },

    // Low confidence warnings
    LowConfidenceAssignment {
        participant: Participant,
        confidence: f64,
        alternative_frames: Vec<PropBankFrame>,
    },
}
```

### Code Actions

```rust
enum PropBankCodeAction {
    // Suggest argument structure fixes
    FixArgumentStructure {
        frame: PropBankFrame,
        corrections: Vec<ArgumentCorrection>,
    },

    // Resolve frame ambiguity
    SelectFrame {
        alternatives: Vec<PropBankFrame>,
        recommendation: PropBankFrame,
        rationale: String,
    },

    // Add missing arguments
    AddMissingArgument {
        argument: ArgumentNumber,
        suggested_text: String,
        insertion_point: TextRange,
    },
}
```

## Testing Strategy

### Corpus Validation

```rust
struct PropBankTestSuite {
    // Gold standard annotations
    gold_annotations: HashMap<SentenceId, PropBankAnnotation>,

    // Cross-validation with VerbNet
    verbnet_alignment_tests: Vec<AlignmentTest>,

    // Performance benchmarks
    performance_tests: Vec<PerformanceTest>,

    // Frame disambiguation tests
    disambiguation_tests: Vec<DisambiguationTest>,
}

impl PropBankTestSuite {
    fn test_frame_assignment(&self) -> TestResult {
        let mut correct = 0;
        let mut total = 0;

        for (sentence_id, gold_annotation) in &self.gold_annotations {
            let sentence = self.get_sentence(sentence_id);
            let predicted = self.propbank.analyze_sentence(sentence);

            if self.annotations_match(&predicted, gold_annotation) {
                correct += 1;
            }
            total += 1;
        }

        TestResult {
            accuracy: correct as f64 / total as f64,
            precision: self.calculate_precision(),
            recall: self.calculate_recall(),
            f1: self.calculate_f1(),
        }
    }
}
```

## Future Extensions

### Machine Learning Enhancement

```rust
struct MLPropBankEnhancer {
    frame_classifier: FrameClassifier,
    argument_detector: ArgumentDetector,
    confidence_estimator: ConfidenceEstimator,
}

// Can be added in M5+ for improved disambiguation
impl MLPropBankEnhancer {
    fn enhance_frame_selection(&self, candidates: Vec<PropBankFrame>, context: &Context) -> PropBankFrame {
        let features = self.extract_features(context);
        let scores = self.frame_classifier.predict(&features, &candidates);

        candidates
            .into_iter()
            .zip(scores)
            .max_by(|(_, score1), (_, score2)| score1.partial_cmp(score2).unwrap())
            .map(|(frame, _)| frame)
            .unwrap()
    }
}
```

## Conclusion

PropBank integration in M4 provides corpus-validated argument structure that significantly enhances the confidence and accuracy of our theta role assignment. By combining VerbNet's theoretical framework with PropBank's empirical annotations, we create a robust semantic analysis system that leverages both formal linguistic theory and real-world usage patterns.

The integration operates within our <75μs performance budget while providing:
- **Cross-validation** between theoretical and corpus-based approaches
- **Enhanced confidence** through multiple evidence sources
- **Empirical grounding** for semantic role assignment
- **Frame disambiguation** for polysemous verbs
- **Corpus pattern discovery** for extending theoretical coverage

This positions canopy.rs as the first system to combine real-time theoretical analysis with corpus validation at production scale.
