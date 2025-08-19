//! Integration tests for canopy-semantics
//!
//! These tests verify that the core semantic components work together to create
//! Neo-Davidsonian event structures with Layer 2 semantic analysis.

use canopy_core::{DepRel, MorphFeatures, UPos, Word};
use canopy_semantics::{
    Event, EventId, Layer2Analyzer, MovementDetector, Participant, PredicateType, ThetaRoleType,
    VoiceDetector, VoiceType,
};

/// Helper function to create test words with proper structure
fn create_word(
    id: usize,
    text: &str,
    lemma: &str,
    upos: UPos,
    head: Option<usize>,
    deprel: DepRel,
) -> Word {
    Word {
        id,
        text: text.to_string(),
        lemma: lemma.to_string(),
        upos,
        xpos: None,
        feats: MorphFeatures::default(),
        head,
        deprel,
        deps: None,
        misc: None,
        start: 0,
        end: text.len(),
    }
}

#[test]
fn test_layer2_integration() {
    let mut analyzer = Layer2Analyzer::new();

    // Create test sentence: "John runs quickly"
    let words = vec![
        create_word(1, "John", "John", UPos::Propn, Some(2), DepRel::Nsubj),
        create_word(2, "runs", "run", UPos::Verb, Some(0), DepRel::Root),
        create_word(3, "quickly", "quickly", UPos::Adv, Some(2), DepRel::Advmod),
    ];

    let result = analyzer.analyze(words);
    assert!(result.is_ok(), "Layer 2 analysis should succeed");

    let analysis = result.unwrap();
    assert_eq!(analysis.words.len(), 3, "Should process all words");
    assert_eq!(analysis.events.len(), 1, "Should create one event for verb");

    let event = &analysis.events[0];
    assert_eq!(event.predicate.lemma, "run", "Event should be for 'run'");
    assert_eq!(
        event.predicate.semantic_type,
        PredicateType::Action,
        "Should classify as action"
    );
}

#[test]
fn test_voice_detection_integration() {
    let detector = VoiceDetector::new();

    // Test active voice
    let active_word = create_word(1, "runs", "run", UPos::Verb, Some(0), DepRel::Root);
    let words = vec![active_word];
    let voice_analysis = detector.detect_voice(&words, "run");

    assert_eq!(
        voice_analysis.voice_type,
        VoiceType::Active,
        "Should detect active voice"
    );
}

#[test]
fn test_movement_detection_integration() {
    let detector = MovementDetector::new();

    // Test simple sentence without movement
    let words = vec![
        create_word(1, "John", "John", UPos::Propn, Some(2), DepRel::Nsubj),
        create_word(2, "walks", "walk", UPos::Verb, Some(0), DepRel::Root),
    ];

    let movement_analysis = detector.detect_movement(&words);

    // Simple sentence should have no movement
    assert!(
        movement_analysis.movement_types.is_empty(),
        "Simple sentence should have no movement"
    );
}

#[test]
fn test_event_creation_integration() {
    // Test direct event creation
    let predicate = canopy_semantics::events::event_semantics::Predicate {
        lemma: "love".to_string(),
        semantic_type: PredicateType::State,
        verbnet_class: Some("love-31.2".to_string()),
        features: vec![],
    };

    let mut event = Event::new(EventId(1), predicate);

    // Add participants
    let agent = Participant {
        word_id: 1,
        expression: "John".to_string(),
        features: Default::default(),
        discourse_ref: None,
    };
    event.add_participant(ThetaRoleType::Experiencer, agent);

    let theme = Participant {
        word_id: 2,
        expression: "Mary".to_string(),
        features: Default::default(),
        discourse_ref: None,
    };
    event.add_participant(ThetaRoleType::Theme, theme);

    // Verify event structure
    assert_eq!(event.participants.len(), 2, "Should have two participants");
    assert!(
        event.has_role(&ThetaRoleType::Experiencer),
        "Should have experiencer"
    );
    assert!(event.has_role(&ThetaRoleType::Theme), "Should have theme");

    let experiencer = event.get_participant(&ThetaRoleType::Experiencer).unwrap();
    assert_eq!(experiencer.expression, "John", "Experiencer should be John");
}
