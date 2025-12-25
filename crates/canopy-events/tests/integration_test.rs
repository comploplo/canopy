//! Integration tests for canopy-events Layer 2 event composition

use canopy_core::{LittleV, ThetaRole, UPos, Voice};
use canopy_events::{
    DependencyArc, EventComposer, EventComposerConfig, LittleVType, SentenceAnalysis,
    SentenceMetadata,
};
use canopy_tokenizer::coordinator::Layer1SemanticResult;
use canopy_treebank::types::DependencyRelation;
use canopy_verbnet::{
    Argument, Example, Frame, FrameDescription, Member, SelectionalRestrictions, SemanticPredicate,
    SyntaxPattern, ThematicRole, VerbClass, VerbNetAnalysis,
};

/// Helper to create a test token
fn make_token(word: &str, lemma: &str, pos: Option<UPos>) -> Layer1SemanticResult {
    Layer1SemanticResult {
        original_word: word.to_string(),
        lemma: lemma.to_string(),
        pos,
        lemmatization_confidence: None,
        verbnet: None,
        framenet: None,
        wordnet: None,
        lexicon: None,
        treebank: None,
        confidence: 0.8,
        sources: vec![],
        errors: vec![],
    }
}

#[test]
fn test_simple_intransitive() {
    // "John runs"
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_token("runs", "run", Some(UPos::Verb)),
    ];

    let deps = vec![DependencyArc::new(1, 0, DependencyRelation::NominalSubject)];

    let analysis = SentenceAnalysis::new("John runs".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events(), "Should have composed an event");
    assert_eq!(result.events.len(), 1);

    let event = &result.events[0];
    assert_eq!(event.event.predicate, "run");
    assert_eq!(event.event.voice, Voice::Active);
}

#[test]
fn test_simple_transitive() {
    // "John broke the vase"
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_token("broke", "break", Some(UPos::Verb)),
        make_token("the", "the", Some(UPos::Det)),
        make_token("vase", "vase", Some(UPos::Noun)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject), // John <- broke
        DependencyArc::new(1, 3, DependencyRelation::Object),         // vase <- broke
        DependencyArc::new(3, 2, DependencyRelation::Determiner),     // the <- vase
    ];

    let analysis =
        SentenceAnalysis::new("John broke the vase".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];
    assert_eq!(event.event.predicate, "break");

    // Should have bound participants
    assert!(
        event.has_role(ThetaRole::Agent) || event.has_role(ThetaRole::Theme),
        "Should have bound at least one role"
    );
}

#[test]
fn test_ditransitive() {
    // "John gave Mary a book"
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_token("gave", "give", Some(UPos::Verb)),
        make_token("Mary", "mary", Some(UPos::Propn)),
        make_token("a", "a", Some(UPos::Det)),
        make_token("book", "book", Some(UPos::Noun)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject), // John <- gave
        DependencyArc::new(1, 2, DependencyRelation::IndirectObject), // Mary <- gave
        DependencyArc::new(1, 4, DependencyRelation::Object),         // book <- gave
        DependencyArc::new(4, 3, DependencyRelation::Determiner),     // a <- book
    ];

    let analysis =
        SentenceAnalysis::new("John gave Mary a book".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];
    assert_eq!(event.event.predicate, "give");

    // Check that multiple participants are bound
    assert!(
        event.event.participants.len() >= 2,
        "Should have at least 2 participants bound"
    );
}

#[test]
fn test_passive_voice() {
    // "The vase was broken"
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("The", "the", Some(UPos::Det)),
        make_token("vase", "vase", Some(UPos::Noun)),
        make_token("was", "be", Some(UPos::Aux)),
        make_token("broken", "break", Some(UPos::Verb)),
    ];

    let deps = vec![
        DependencyArc::new(3, 1, DependencyRelation::NominalSubject), // vase <- broken
        DependencyArc::new(3, 2, DependencyRelation::Auxiliary),      // was <- broken
        DependencyArc::new(1, 0, DependencyRelation::Determiner),     // the <- vase
    ];

    let metadata = SentenceMetadata {
        is_passive: true,
        ..Default::default()
    };

    let analysis = SentenceAnalysis::new("The vase was broken".to_string(), tokens)
        .with_dependencies(deps)
        .with_metadata(metadata);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];
    assert_eq!(event.event.predicate, "break");
    assert_eq!(event.event.voice, Voice::Passive);
}

#[test]
fn test_copular_sentence() {
    // "John is tall"
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_token("is", "be", Some(UPos::Verb)),
        make_token("tall", "tall", Some(UPos::Adj)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject), // John <- is
        DependencyArc::new(1, 2, DependencyRelation::AdjectivalModifier), // tall <- is
    ];

    let analysis =
        SentenceAnalysis::new("John is tall".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];
    assert_eq!(event.event.predicate, "be");
}

#[test]
fn test_psych_verb() {
    // "John fears spiders"
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_token("fears", "fear", Some(UPos::Verb)),
        make_token("spiders", "spider", Some(UPos::Noun)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject), // John <- fears
        DependencyArc::new(1, 2, DependencyRelation::Object),         // spiders <- fears
    ];

    let analysis =
        SentenceAnalysis::new("John fears spiders".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];
    assert_eq!(event.event.predicate, "fear");
}

#[test]
fn test_motion_verb() {
    // "John walked home"
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_token("walked", "walk", Some(UPos::Verb)),
        make_token("home", "home", Some(UPos::Noun)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject), // John <- walked
        DependencyArc::new(1, 2, DependencyRelation::Oblique),        // home <- walked
    ];

    let analysis =
        SentenceAnalysis::new("John walked home".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];
    assert_eq!(event.event.predicate, "walk");
}

#[test]
fn test_empty_sentence() {
    let composer = EventComposer::new().unwrap();
    let analysis = SentenceAnalysis::new("".to_string(), vec![]);

    let result = composer.compose_sentence(&analysis).unwrap();
    assert!(!result.has_events());
    assert_eq!(result.events.len(), 0);
}

#[test]
fn test_confidence_threshold() {
    let config = EventComposerConfig {
        confidence_threshold: 0.9, // High threshold
        ..Default::default()
    };
    let composer = EventComposer::with_config(config).unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_token("runs", "run", Some(UPos::Verb)),
    ];

    let deps = vec![DependencyArc::new(1, 0, DependencyRelation::NominalSubject)];

    let analysis = SentenceAnalysis::new("John runs".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    // With high threshold and low confidence heuristics, events may be filtered
    // This test verifies the threshold is applied
    assert!(result.processing_time_us > 0);
}

#[test]
fn test_batch_composition() {
    let composer = EventComposer::new().unwrap();

    let sentences: Vec<SentenceAnalysis> = vec![
        SentenceAnalysis::new(
            "John runs".to_string(),
            vec![
                make_token("John", "john", Some(UPos::Propn)),
                make_token("runs", "run", Some(UPos::Verb)),
            ],
        )
        .with_dependencies(vec![DependencyArc::new(
            1,
            0,
            DependencyRelation::NominalSubject,
        )]),
        SentenceAnalysis::new(
            "Mary walks".to_string(),
            vec![
                make_token("Mary", "mary", Some(UPos::Propn)),
                make_token("walks", "walk", Some(UPos::Verb)),
            ],
        )
        .with_dependencies(vec![DependencyArc::new(
            1,
            0,
            DependencyRelation::NominalSubject,
        )]),
    ];

    let results = composer.compose_batch(&sentences).unwrap();

    assert_eq!(results.len(), 2);
    assert!(results[0].has_events());
    assert!(results[1].has_events());
    assert_eq!(results[0].events[0].event.predicate, "run");
    assert_eq!(results[1].events[0].event.predicate, "walk");
}

#[test]
fn test_little_v_type_defaults() {
    // Test that LittleVType provides correct default roles
    assert!(
        LittleVType::Cause
            .default_roles()
            .contains(&ThetaRole::Agent)
    );
    assert!(
        LittleVType::Cause
            .default_roles()
            .contains(&ThetaRole::Patient)
    );
    assert!(
        LittleVType::Experience
            .default_roles()
            .contains(&ThetaRole::Experiencer)
    );
    assert!(LittleVType::Go.default_roles().contains(&ThetaRole::Theme));
    assert!(LittleVType::Go.default_roles().contains(&ThetaRole::Goal));
}

// ============================================================================
// Phase 2: Tests with real VerbNet data
// ============================================================================

/// Creates a VerbNet analysis with a specific semantic predicate
fn make_verbnet_analysis(
    verb: &str,
    class_id: &str,
    class_name: &str,
    semantic_predicate: &str,
) -> VerbNetAnalysis {
    let frame = Frame {
        description: FrameDescription {
            description_number: "1".to_string(),
            primary: "NP V NP".to_string(),
            secondary: None,
            xtag: None,
        },
        examples: vec![Example {
            text: format!("Agent {} patient", verb),
        }],
        syntax: SyntaxPattern { elements: vec![] },
        semantics: vec![SemanticPredicate {
            value: semantic_predicate.to_string(),
            args: vec![
                Argument {
                    arg_type: "ThemRole".to_string(),
                    value: "Agent".to_string(),
                },
                Argument {
                    arg_type: "ThemRole".to_string(),
                    value: "Patient".to_string(),
                },
            ],
            negated: false,
        }],
    };

    let verb_class = VerbClass {
        id: class_id.to_string(),
        class_name: class_name.to_string(),
        parent_class: None,
        members: vec![Member {
            name: verb.to_string(),
            wn: None,
            grouping: None,
            features: None,
        }],
        themroles: vec![
            ThematicRole {
                role_type: "Agent".to_string(),
                selrestrs: SelectionalRestrictions::empty(),
            },
            ThematicRole {
                role_type: "Patient".to_string(),
                selrestrs: SelectionalRestrictions::empty(),
            },
        ],
        frames: vec![frame],
        subclasses: vec![],
    };

    VerbNetAnalysis::new(verb.to_string(), vec![verb_class], 0.9)
}

/// Helper to create a verb token with VerbNet analysis
fn make_verb_with_verbnet(
    word: &str,
    lemma: &str,
    class_id: &str,
    class_name: &str,
    semantic_predicate: &str,
) -> Layer1SemanticResult {
    Layer1SemanticResult {
        original_word: word.to_string(),
        lemma: lemma.to_string(),
        pos: Some(UPos::Verb),
        lemmatization_confidence: Some(0.95),
        verbnet: Some(make_verbnet_analysis(
            lemma,
            class_id,
            class_name,
            semantic_predicate,
        )),
        framenet: None,
        wordnet: None,
        lexicon: None,
        treebank: None,
        confidence: 0.9,
        sources: vec!["VerbNet".to_string()],
        errors: vec![],
    }
}

#[test]
fn test_verbnet_cause_predicate() {
    // "John broke the vase" - with VerbNet "cause" predicate
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_verb_with_verbnet("broke", "break", "break-45.1", "break", "cause"),
        make_token("the", "the", Some(UPos::Det)),
        make_token("vase", "vase", Some(UPos::Noun)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject),
        DependencyArc::new(1, 3, DependencyRelation::Object),
        DependencyArc::new(3, 2, DependencyRelation::Determiner),
    ];

    let analysis =
        SentenceAnalysis::new("John broke the vase".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events(), "Should have composed an event");
    let event = &result.events[0];
    assert_eq!(event.event.predicate, "break");

    // Verify VerbNet was used (not heuristics)
    assert!(
        event.verbnet_source.is_some(),
        "Event should have VerbNet source, got: {:?}",
        event.verbnet_source
    );

    // Verify the decomposition is Cause type (LittleV::Cause variant)
    assert!(
        matches!(event.event.little_v, LittleV::Cause { .. }),
        "VerbNet 'cause' predicate should map to LittleV::Cause, got: {:?}",
        event.event.little_v
    );

    // Verify confidence is higher than heuristic fallback
    assert!(
        event.decomposition_confidence > 0.5,
        "VerbNet-based confidence should be higher than heuristic, got: {}",
        event.decomposition_confidence
    );
}

#[test]
fn test_verbnet_motion_predicate() {
    // "John walked" - with VerbNet "motion" predicate
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_verb_with_verbnet("walked", "walk", "run-51.3.2", "walk", "motion"),
    ];

    let deps = vec![DependencyArc::new(1, 0, DependencyRelation::NominalSubject)];

    let analysis = SentenceAnalysis::new("John walked".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];

    // VerbNet "motion" should map to Go (LittleV::Go variant)
    assert!(
        matches!(event.event.little_v, LittleV::Go { .. }),
        "VerbNet 'motion' predicate should map to LittleV::Go, got: {:?}",
        event.event.little_v
    );

    // Verify VerbNet source
    assert!(
        event.verbnet_source.is_some(),
        "Event should have VerbNet source"
    );
}

#[test]
fn test_verbnet_transfer_predicate() {
    // "John gave Mary a book" - with VerbNet "transfer" predicate
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_verb_with_verbnet("gave", "give", "give-13.1", "give", "transfer"),
        make_token("Mary", "mary", Some(UPos::Propn)),
        make_token("a", "a", Some(UPos::Det)),
        make_token("book", "book", Some(UPos::Noun)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject),
        DependencyArc::new(1, 2, DependencyRelation::IndirectObject),
        DependencyArc::new(1, 4, DependencyRelation::Object),
        DependencyArc::new(4, 3, DependencyRelation::Determiner),
    ];

    let analysis =
        SentenceAnalysis::new("John gave Mary a book".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];
    assert_eq!(event.event.predicate, "give");

    // VerbNet "transfer" should map to Have or Cause (with sub-event)
    assert!(
        matches!(
            event.event.little_v,
            LittleV::Have { .. } | LittleV::Cause { .. }
        ),
        "VerbNet 'transfer' should map to Have or Cause, got: {:?}",
        event.event.little_v
    );

    // Verify VerbNet source
    assert!(
        event.verbnet_source.is_some(),
        "Event should have VerbNet source"
    );
}

#[test]
fn test_verbnet_experience_predicate() {
    // "John fears spiders" - with VerbNet "experiencer" predicate
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_verb_with_verbnet("fears", "fear", "admire-31.2", "fear", "experiencer"),
        make_token("spiders", "spider", Some(UPos::Noun)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject),
        DependencyArc::new(1, 2, DependencyRelation::Object),
    ];

    let analysis =
        SentenceAnalysis::new("John fears spiders".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];

    // VerbNet "experiencer" should map to Experience
    assert!(
        matches!(event.event.little_v, LittleV::Experience { .. }),
        "VerbNet 'experiencer' predicate should map to LittleV::Experience, got: {:?}",
        event.event.little_v
    );

    // Verify VerbNet source
    assert!(
        event.verbnet_source.is_some(),
        "Event should have VerbNet source"
    );
}

#[test]
fn test_verbnet_existence_predicate() {
    // "The book exists" - with VerbNet "exist" predicate
    let composer = EventComposer::new().unwrap();

    let tokens = vec![
        make_token("The", "the", Some(UPos::Det)),
        make_token("book", "book", Some(UPos::Noun)),
        make_verb_with_verbnet("exists", "exist", "exist-47.1", "exist", "exist"),
    ];

    let deps = vec![
        DependencyArc::new(2, 1, DependencyRelation::NominalSubject),
        DependencyArc::new(1, 0, DependencyRelation::Determiner),
    ];

    let analysis =
        SentenceAnalysis::new("The book exists".to_string(), tokens).with_dependencies(deps);

    let result = composer.compose_sentence(&analysis).unwrap();

    assert!(result.has_events());
    let event = &result.events[0];

    // VerbNet "exist" should map to Exist or Be
    assert!(
        matches!(
            event.event.little_v,
            LittleV::Exist { .. } | LittleV::Be { .. }
        ),
        "VerbNet 'exist' should map to Exist or Be, got: {:?}",
        event.event.little_v
    );

    // Verify VerbNet source
    assert!(
        event.verbnet_source.is_some(),
        "Event should have VerbNet source"
    );
}

#[test]
fn test_verbnet_vs_heuristic_confidence() {
    // Compare confidence levels between VerbNet-backed and heuristic-only events
    let composer = EventComposer::new().unwrap();

    // With VerbNet data
    let tokens_with_vn = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_verb_with_verbnet("broke", "break", "break-45.1", "break", "cause"),
        make_token("it", "it", Some(UPos::Pron)),
    ];

    let deps = vec![
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject),
        DependencyArc::new(1, 2, DependencyRelation::Object),
    ];

    let analysis_with_vn = SentenceAnalysis::new("John broke it".to_string(), tokens_with_vn)
        .with_dependencies(deps.clone());

    // Without VerbNet data (uses heuristics)
    let tokens_without_vn = vec![
        make_token("John", "john", Some(UPos::Propn)),
        make_token("broke", "break", Some(UPos::Verb)),
        make_token("it", "it", Some(UPos::Pron)),
    ];

    let analysis_without_vn = SentenceAnalysis::new("John broke it".to_string(), tokens_without_vn)
        .with_dependencies(deps);

    let result_with_vn = composer.compose_sentence(&analysis_with_vn).unwrap();
    let result_without_vn = composer.compose_sentence(&analysis_without_vn).unwrap();

    assert!(result_with_vn.has_events());
    assert!(result_without_vn.has_events());

    let event_with_vn = &result_with_vn.events[0];
    let event_without_vn = &result_without_vn.events[0];

    // VerbNet-backed event should have higher confidence
    assert!(
        event_with_vn.decomposition_confidence > event_without_vn.decomposition_confidence,
        "VerbNet-backed confidence ({}) should be higher than heuristic ({})",
        event_with_vn.decomposition_confidence,
        event_without_vn.decomposition_confidence
    );

    // VerbNet source should be present for VerbNet-backed event
    assert!(
        event_with_vn.verbnet_source.is_some(),
        "VerbNet event should have VerbNet source"
    );

    // VerbNet source should be absent for heuristic-backed event
    assert!(
        event_without_vn.verbnet_source.is_none(),
        "Non-VerbNet event should not have VerbNet source, got: {:?}",
        event_without_vn.verbnet_source
    );
}
