//! Basic integration tests for working semantics components
//!
//! This test file focuses on the components that are currently working
//! and provides a foundation for testing the semantics library.

use canopy_core::{DepRel, MorphFeatures, UDVerbForm, UDVoice, UPos, Word};
use canopy_semantics::{AspectualClass, MovementDetector, MovementType, VoiceDetector, VoiceType};

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

/// Create test sentence: "John gives Mary a book"
fn create_active_sentence() -> Vec<Word> {
    vec![
        create_word(1, "John", "John", UPos::Propn, Some(2), DepRel::Nsubj),
        create_word(2, "gives", "give", UPos::Verb, Some(0), DepRel::Root),
        create_word(3, "Mary", "Mary", UPos::Propn, Some(2), DepRel::Iobj),
        create_word(4, "a", "a", UPos::Det, Some(5), DepRel::Det),
        create_word(5, "book", "book", UPos::Noun, Some(2), DepRel::Obj),
    ]
}

/// Create test sentence: "The ball was hit by John"
fn create_passive_sentence() -> Vec<Word> {
    let mut words = vec![
        create_word(1, "The", "the", UPos::Det, Some(2), DepRel::Det),
        create_word(2, "ball", "ball", UPos::Noun, Some(4), DepRel::NsubjPass),
        create_word(3, "was", "be", UPos::Aux, Some(4), DepRel::AuxPass),
        create_word(4, "hit", "hit", UPos::Verb, Some(0), DepRel::Root),
        create_word(5, "by", "by", UPos::Adp, Some(6), DepRel::Case),
        create_word(6, "John", "John", UPos::Propn, Some(4), DepRel::Obl),
    ];

    // Add passive voice morphology
    words[3].feats.voice = Some(UDVoice::Passive);
    words[4].feats.verbform = Some(UDVerbForm::Participle);

    words
}

/// Create test sentence: "What did John see?"
fn create_wh_question() -> Vec<Word> {
    vec![
        create_word(1, "What", "what", UPos::Pron, Some(4), DepRel::Obj),
        create_word(2, "did", "do", UPos::Aux, Some(4), DepRel::Aux),
        create_word(3, "John", "John", UPos::Propn, Some(4), DepRel::Nsubj),
        create_word(4, "see", "see", UPos::Verb, Some(0), DepRel::Root),
    ]
}

#[test]
fn test_voice_detection_active() {
    let detector = VoiceDetector::new();
    let words = create_active_sentence();

    let analysis = detector.detect_voice(&words, "give");

    // Should detect active voice for "John gives Mary a book"
    assert_eq!(analysis.voice_type, VoiceType::Active);
    assert!(analysis.confidence > 0.0);
    assert!(!analysis.has_passive_subject);
    assert!(!analysis.has_by_phrase);
}

#[test]
fn test_voice_detection_passive() {
    let detector = VoiceDetector::new();
    let words = create_passive_sentence();

    let analysis = detector.detect_voice(&words, "hit");

    // Should detect passive voice for "The ball was hit by John"
    assert_eq!(analysis.voice_type, VoiceType::Passive);
    assert!(analysis.confidence > 0.5);
    assert!(analysis.has_passive_subject);
    assert!(analysis.has_by_phrase);
}

#[test]
fn test_movement_detection_active() {
    let detector = MovementDetector::new();
    let words = create_active_sentence();

    let analysis = detector.detect_movement(&words);

    // Simple active sentence should have no movement
    assert!(analysis.movement_types.is_empty());
    assert_eq!(analysis.confidence, 0.0);
}

#[test]
fn test_movement_detection_passive() {
    let detector = MovementDetector::new();
    let words = create_passive_sentence();

    let analysis = detector.detect_movement(&words);

    // Passive sentence should show passive movement
    assert!(
        analysis
            .movement_types
            .contains(&MovementType::PassiveMovement)
    );
    assert!(analysis.signals.passive_voice);
    assert!(analysis.confidence > 0.0);
}

#[test]
fn test_movement_detection_wh() {
    let detector = MovementDetector::new();
    let words = create_wh_question();

    let analysis = detector.detect_movement(&words);

    // Wh-question should show wh-movement
    assert!(analysis.movement_types.contains(&MovementType::WhMovement));
    assert_eq!(analysis.signals.wh_word, Some("what".to_string()));
    assert!(analysis.signals.fronted_wh);
    assert!(analysis.confidence > 0.0);
}

#[test]
fn test_aspectual_classes_exist() {
    // Just verify that aspectual classes can be instantiated
    let state = AspectualClass::State;
    let activity = AspectualClass::Activity;
    let accomplishment = AspectualClass::Accomplishment;
    let achievement = AspectualClass::Achievement;

    // Test they're distinct
    assert_ne!(state, activity);
    assert_ne!(activity, accomplishment);
    assert_ne!(accomplishment, achievement);

    // Test debug formatting works
    assert!(!format!("{:?}", state).is_empty());
    assert!(!format!("{:?}", activity).is_empty());
    assert!(!format!("{:?}", accomplishment).is_empty());
    assert!(!format!("{:?}", achievement).is_empty());
}

#[test]
fn test_voice_types_exist() {
    // Verify voice types work correctly
    let active = VoiceType::Active;
    let passive = VoiceType::Passive;
    let middle = VoiceType::Middle;
    let reflexive = VoiceType::Reflexive;

    // Test they're distinct
    assert_ne!(active, passive);
    assert_ne!(passive, middle);
    assert_ne!(middle, reflexive);

    // Test display formatting
    assert_eq!(format!("{}", active), "Active");
    assert_eq!(format!("{}", passive), "Passive");
    assert_eq!(format!("{}", middle), "Middle");
    assert_eq!(format!("{}", reflexive), "Reflexive");
}

#[test]
fn test_movement_types_exist() {
    // Verify movement types work correctly
    let passive_movement = MovementType::PassiveMovement;
    let wh_movement = MovementType::WhMovement;
    let relative_movement = MovementType::RelativeMovement;
    let topic_movement = MovementType::TopicMovement;

    // Test they're distinct
    assert_ne!(passive_movement, wh_movement);
    assert_ne!(wh_movement, relative_movement);
    assert_ne!(relative_movement, topic_movement);

    // Test display formatting
    assert_eq!(format!("{}", passive_movement), "Passive");
    assert_eq!(format!("{}", wh_movement), "Wh-Movement");
    assert_eq!(format!("{}", relative_movement), "Relative");
    assert_eq!(format!("{}", topic_movement), "Topic");
}

#[test]
fn test_performance_basic() {
    // Test that basic analysis is reasonably fast
    use std::time::Instant;

    let words = create_active_sentence();

    let start = Instant::now();

    // Run voice detection
    let voice_detector = VoiceDetector::new();
    let _voice_analysis = voice_detector.detect_voice(&words, "give");

    // Run movement detection
    let movement_detector = MovementDetector::new();
    let _movement_analysis = movement_detector.detect_movement(&words);

    let duration = start.elapsed();

    // Should be very fast for basic analysis
    println!("Basic analysis took: {:?}", duration);
    assert!(
        duration.as_millis() < 10,
        "Basic analysis took {}ms, should be <10ms",
        duration.as_millis()
    );
}

#[test]
fn test_complex_passive_with_wh() {
    // Test a sentence with both passive and wh-movement: "What was John given?"
    let mut words = vec![
        create_word(1, "What", "what", UPos::Pron, Some(4), DepRel::Obj),
        create_word(2, "was", "be", UPos::Aux, Some(4), DepRel::AuxPass),
        create_word(3, "John", "John", UPos::Propn, Some(4), DepRel::NsubjPass),
        create_word(4, "given", "give", UPos::Verb, Some(0), DepRel::Root),
    ];

    // Add passive morphology
    words[1].feats.voice = Some(UDVoice::Passive);
    words[3].feats.verbform = Some(UDVerbForm::Participle);

    let movement_detector = MovementDetector::new();
    let analysis = movement_detector.detect_movement(&words);

    // Should detect both types of movement
    assert!(analysis.movement_types.contains(&MovementType::WhMovement));
    assert!(
        analysis
            .movement_types
            .contains(&MovementType::PassiveMovement)
    );
    assert!(analysis.signals.passive_voice);
    assert_eq!(analysis.signals.wh_word, Some("what".to_string()));
    assert!(analysis.confidence > 0.0);
}

#[test]
fn test_reflexive_detection() {
    // Test reflexive voice detection: "John hurt himself"
    let words = vec![
        create_word(1, "John", "John", UPos::Propn, Some(2), DepRel::Nsubj),
        create_word(2, "hurt", "hurt", UPos::Verb, Some(0), DepRel::Root),
        create_word(3, "himself", "himself", UPos::Pron, Some(2), DepRel::Obj),
    ];

    let detector = VoiceDetector::new();
    let analysis = detector.detect_voice(&words, "hurt");

    // Should detect reflexive voice
    assert_eq!(analysis.voice_type, VoiceType::Reflexive);
    assert!(analysis.has_reflexive_marker);
}

#[test]
fn test_relative_clause_detection() {
    // Test relative clause: "The book that John read"
    let words = vec![
        create_word(1, "The", "the", UPos::Det, Some(2), DepRel::Det),
        create_word(2, "book", "book", UPos::Noun, Some(0), DepRel::Root),
        create_word(3, "that", "that", UPos::Pron, Some(5), DepRel::Obj),
        create_word(4, "John", "John", UPos::Propn, Some(5), DepRel::Nsubj),
        create_word(5, "read", "read", UPos::Verb, Some(2), DepRel::Acl),
    ];

    let detector = MovementDetector::new();
    let analysis = detector.detect_movement(&words);

    // Should detect relative movement
    assert!(
        analysis
            .movement_types
            .contains(&MovementType::RelativeMovement)
    );
    assert_eq!(analysis.signals.relative_pronoun, Some("that".to_string()));
}

#[test]
fn test_raising_detection() {
    // Test raising construction: "John seems to be happy"
    let mut words = vec![
        create_word(1, "John", "John", UPos::Propn, Some(2), DepRel::Nsubj),
        create_word(2, "seems", "seem", UPos::Verb, Some(0), DepRel::Root),
        create_word(3, "to", "to", UPos::Part, Some(5), DepRel::Mark),
        create_word(4, "be", "be", UPos::Aux, Some(5), DepRel::Cop),
        create_word(5, "happy", "happy", UPos::Adj, Some(2), DepRel::Xcomp),
    ];

    // Add infinitive feature
    words[4].feats.verbform = Some(UDVerbForm::Infinitive);

    let detector = MovementDetector::new();
    let analysis = detector.detect_movement(&words);

    // Should detect raising movement
    assert!(
        analysis
            .movement_types
            .contains(&MovementType::RaisingMovement)
    );
    assert!(analysis.signals.seems_construction);
    assert!(analysis.signals.infinitival_complement);
}

#[test]
fn test_semantic_types_integration() {
    // Test that the basic semantic type system is working
    use canopy_semantics::{Animacy, Definiteness, Number, PredicateType, SemanticFeature};

    // Test predicate types
    let action = PredicateType::Action;
    let state = PredicateType::State;
    let achievement = PredicateType::Achievement;

    assert_ne!(action, state);
    assert_ne!(state, achievement);

    // Test semantic features
    let motion = SemanticFeature::Motion;
    let transfer = SemanticFeature::Transfer;
    let contact = SemanticFeature::Contact;

    assert_ne!(motion, transfer);
    assert_ne!(transfer, contact);

    // Test animacy hierarchy
    let human = Animacy::Human;
    let animal = Animacy::Animal;
    let inanimate = Animacy::Inanimate;

    assert!(human > animal);
    assert!(animal > inanimate);

    // Test definiteness
    let definite = Definiteness::Definite;
    let indefinite = Definiteness::Indefinite;

    assert_ne!(definite, indefinite);

    // Test number
    let singular = Number::Singular;
    let plural = Number::Plural;

    assert_ne!(singular, plural);
}

#[test]
fn test_syntax_module_integration() {
    // Test that the syntax module properly exports everything
    use canopy_semantics::{
        MovementAnalysis, MovementDetector, MovementSignals, VoiceAnalysis, VoiceDetector,
        VoiceType,
    };

    // Should be able to create all the main types
    let voice_detector = VoiceDetector::new();
    let movement_detector = MovementDetector::new();

    assert!(voice_detector.detect_voice(&[], "test").confidence >= 0.0);
    assert!(movement_detector.detect_movement(&[]).confidence >= 0.0);

    // Test that the types have proper defaults
    let voice_analysis = VoiceAnalysis::default();
    let movement_analysis = MovementAnalysis::default();
    let movement_signals = MovementSignals::default();

    assert_eq!(voice_analysis.voice_type, VoiceType::Unknown);
    assert!(movement_analysis.movement_types.is_empty());
    assert!(!movement_signals.passive_voice);
}

/// Test the basic infrastructure is working
#[test]
fn test_canopy_semantics_module_structure() {
    // This test verifies that the main module structure is working
    // and that we can import the key types

    use canopy_semantics::{
        Animacy,
        AspectualClass,
        Definiteness,
        // Events module
        EventId,
        EventTime,

        MovementDetector,

        Number,
        PredicateType,
        // Features we can actually use
        SemanticFeature,
        // Syntax module
        VoiceDetector,
        VoiceType,
    };

    // Basic smoke test - can we create these types?
    let event_id = EventId(1);
    let aspectual_class = AspectualClass::Activity;
    let event_time = EventTime::Now;
    let predicate_type = PredicateType::Action;
    let semantic_feature = SemanticFeature::Motion;
    let animacy = Animacy::Human;
    let definiteness = Definiteness::Definite;
    let number = Number::Singular;
    let voice_type = VoiceType::Active;
    let movement_type = MovementType::None;

    // Test that debug formatting works
    assert!(!format!("{:?}", event_id).is_empty());
    assert!(!format!("{:?}", aspectual_class).is_empty());
    assert!(!format!("{:?}", event_time).is_empty());
    assert!(!format!("{:?}", predicate_type).is_empty());
    assert!(!format!("{:?}", semantic_feature).is_empty());
    assert!(!format!("{:?}", animacy).is_empty());
    assert!(!format!("{:?}", definiteness).is_empty());
    assert!(!format!("{:?}", number).is_empty());
    assert!(!format!("{:?}", voice_type).is_empty());
    assert!(!format!("{:?}", movement_type).is_empty());

    // Test we can create detectors
    let _voice_detector = VoiceDetector::new();
    let _movement_detector = MovementDetector::new();
}
