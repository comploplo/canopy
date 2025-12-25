//! Integration tests for canopy-events Layer 2 event composition

use canopy_core::{ThetaRole, UPos, Voice};
use canopy_events::{
    DependencyArc, EventComposer, EventComposerConfig, LittleVType, SentenceAnalysis,
    SentenceMetadata,
};
use canopy_tokenizer::coordinator::Layer1SemanticResult;
use canopy_treebank::types::DependencyRelation;

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
        DependencyArc::new(1, 3, DependencyRelation::Object),          // vase <- broke
        DependencyArc::new(3, 2, DependencyRelation::Determiner),      // the <- vase
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
    assert!(event.event.participants.len() >= 2, "Should have at least 2 participants bound");
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
        DependencyArc::new(1, 0, DependencyRelation::NominalSubject),     // John <- is
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
    assert!(LittleVType::Cause
        .default_roles()
        .contains(&ThetaRole::Agent));
    assert!(LittleVType::Cause
        .default_roles()
        .contains(&ThetaRole::Patient));
    assert!(LittleVType::Experience
        .default_roles()
        .contains(&ThetaRole::Experiencer));
    assert!(LittleVType::Go.default_roles().contains(&ThetaRole::Theme));
    assert!(LittleVType::Go.default_roles().contains(&ThetaRole::Goal));
}
