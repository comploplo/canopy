//! Tests for event structure module
//!
//! Comprehensive tests for Neo-Davidsonian event semantics, aspectual
//! classification, and event composition.

use super::*;
use crate::ThetaRoleType;
use canopy_core::{DepRel, MorphFeatures, UPos, Word};

fn create_test_word(id: usize, text: &str, lemma: &str, upos: UPos) -> Word {
    Word {
        id,
        text: text.to_string(),
        lemma: lemma.to_string(),
        upos,
        xpos: None,
        feats: MorphFeatures::default(),
        head: Some(0),
        deprel: DepRel::Root,
        deps: None,
        misc: None,
        start: 0,
        end: text.len(),
    }
}

#[test]
fn test_event_with_participants() {
    // Create a simple transitive event: "John hit Mary"
    let predicate = Predicate {
        lemma: "hit".to_string(),
        semantic_type: PredicateType::Action,
        verbnet_class: Some("hit-18.1".to_string()),
        features: vec![SemanticFeature::Contact],
    };

    let mut event = Event::new(EventId(1), predicate);

    // Add agent
    let john = create_test_word(1, "John", "John", UPos::Propn);
    let agent = Participant::from_word(&john);
    event.add_participant(ThetaRoleType::Agent, agent);

    // Add patient
    let mary = create_test_word(2, "Mary", "Mary", UPos::Propn);
    let patient = Participant::from_word(&mary);
    event.add_participant(ThetaRoleType::Patient, patient);

    // Verify event structure
    assert_eq!(event.participants.len(), 2);
    assert!(event.has_role(&ThetaRoleType::Agent));
    assert!(event.has_role(&ThetaRoleType::Patient));

    let agent_participant = event.get_participant(&ThetaRoleType::Agent).unwrap();
    assert_eq!(agent_participant.expression, "John");
    assert_eq!(agent_participant.features.animacy, Some(Animacy::Human));
}

#[test]
fn test_aspectual_classification_with_objects() {
    // Test accomplishment vs activity distinction
    assert_eq!(
        AspectualClass::classify_verb("build", true), // "build a house"
        AspectualClass::Accomplishment
    );
    assert_eq!(
        AspectualClass::classify_verb("build", false), // "build" (intransitive)
        AspectualClass::Activity
    );

    // Test consumption verbs
    assert_eq!(
        AspectualClass::classify_verb("eat", true), // "eat an apple"
        AspectualClass::Accomplishment
    );
    assert_eq!(
        AspectualClass::classify_verb("eat", false), // "eat" (general)
        AspectualClass::Activity
    );
}

#[test]
fn test_event_modification() {
    let predicate = Predicate {
        lemma: "run".to_string(),
        semantic_type: PredicateType::Activity,
        verbnet_class: Some("run-51.3.2".to_string()),
        features: vec![SemanticFeature::Motion],
    };

    let mut event = Event::new(EventId(1), predicate);

    // Add manner modifier: "run quickly"
    let manner_modifier = EventModifier {
        modifier_type: ModifierType::Manner,
        expression: "quickly".to_string(),
        word_id: 3,
    };
    event.add_modifier(manner_modifier);

    // Add locative modifier: "run in the park"
    let locative_modifier = EventModifier {
        modifier_type: ModifierType::Locative,
        expression: "in the park".to_string(),
        word_id: 4,
    };
    event.add_modifier(locative_modifier);

    assert_eq!(event.modifiers.len(), 2);
    assert_eq!(event.modifiers[0].modifier_type, ModifierType::Manner);
    assert_eq!(event.modifiers[1].modifier_type, ModifierType::Locative);
}

#[test]
fn test_causative_event_structure() {
    // Test causative decomposition: "John broke the vase"
    let predicate = Predicate {
        lemma: "break".to_string(),
        semantic_type: PredicateType::Causative,
        verbnet_class: Some("break-45.1".to_string()),
        features: vec![SemanticFeature::ChangeOfState],
    };

    let mut event = Event::new(EventId(1), predicate);

    // Add participants
    let john = create_test_word(1, "John", "John", UPos::Propn);
    let agent = Participant::from_word(&john);
    event.add_participant(ThetaRoleType::Agent, agent);

    let vase = create_test_word(2, "vase", "vase", UPos::Noun);
    let patient = Participant::from_word(&vase);
    event.add_participant(ThetaRoleType::Patient, patient.clone());

    // Add causative structure
    let caused_event = Event {
        id: EventId(2),
        predicate: Predicate {
            lemma: "become_broken".to_string(),
            semantic_type: PredicateType::Inchoative,
            verbnet_class: None,
            features: vec![SemanticFeature::ChangeOfState],
        },
        participants: [(ThetaRoleType::Theme, patient)].into_iter().collect(),
        modifiers: Vec::new(),
        aspect: AspectualClass::Achievement,
        structure: None,
        time: EventTime::Now,
        movement_chains: Vec::new(),
        little_v: None,
    };

    event.structure = Some(EventStructure::Causative {
        causer: event
            .get_participant(&ThetaRoleType::Agent)
            .unwrap()
            .clone(),
        caused_event: Box::new(caused_event),
    });

    // Verify causative structure
    if let Some(EventStructure::Causative {
        causer,
        caused_event,
    }) = &event.structure
    {
        assert_eq!(causer.expression, "John");
        assert_eq!(caused_event.predicate.lemma, "become_broken");
    } else {
        panic!("Expected causative structure");
    }
}

#[test]
fn test_event_composition_conjunction() {
    // Test "John ran and sang"
    let event1 = EventId(1); // ran
    let event2 = EventId(2); // sang
    let result = EventId(3);

    let composite = compose_conjunction(event1, event2, result);

    assert_eq!(composite.composition_type, CompositionType::Conjunction);
    assert_eq!(composite.sub_events.len(), 2);
    assert!(composite.sub_events.contains(&event1));
    assert!(composite.sub_events.contains(&event2));

    // Should have simultaneity relation
    assert_eq!(composite.temporal_relations.len(), 1);
    assert_eq!(
        composite.temporal_relations[0].relation_type,
        TemporalRelationType::Simultaneous
    );
}

#[test]
fn test_event_composition_causation() {
    // Test "John's pushing caused the door to open"
    let cause = EventId(1); // pushing
    let effect = EventId(2); // door opening
    let result = EventId(3);

    let composite = compose_causation(cause, effect, result, CausationType::Direct);

    assert_eq!(composite.composition_type, CompositionType::Causation);
    assert_eq!(composite.causal_relations.len(), 1);

    let causal_rel = &composite.causal_relations[0];
    assert_eq!(causal_rel.cause, cause);
    assert_eq!(causal_rel.effect, effect);
    assert_eq!(causal_rel.causation_type, CausationType::Direct);
}

#[test]
fn test_temporal_consistency_checking() {
    let mut composite = CompositeEvent::new(EventId(0), CompositionType::Sequence);
    composite.add_sub_event(EventId(1));
    composite.add_sub_event(EventId(2));
    composite.add_sub_event(EventId(3));

    // Add consistent chain: 1 -> 2 -> 3
    composite.add_temporal_relation(TemporalRelation {
        event1: EventId(1),
        event2: EventId(2),
        relation_type: TemporalRelationType::Before,
    });
    composite.add_temporal_relation(TemporalRelation {
        event1: EventId(2),
        event2: EventId(3),
        relation_type: TemporalRelationType::Before,
    });

    assert!(composite.is_temporally_consistent());

    // Add inconsistent relation that creates cycle: 3 -> 1
    composite.add_temporal_relation(TemporalRelation {
        event1: EventId(3),
        event2: EventId(1),
        relation_type: TemporalRelationType::Before,
    });

    assert!(!composite.is_temporally_consistent());
}

#[test]
fn test_progressive_aspect_compatibility() {
    // States should be incompatible with progressive
    assert_eq!(
        AspectualClass::State.progressive_compatibility(),
        ProgressiveCompatibility::Incompatible
    );

    // Activities should be compatible
    assert_eq!(
        AspectualClass::Activity.progressive_compatibility(),
        ProgressiveCompatibility::Compatible
    );

    // Achievements should be coercible
    assert_eq!(
        AspectualClass::Achievement.progressive_compatibility(),
        ProgressiveCompatibility::Coercible
    );
}

#[test]
fn test_temporal_modifier_constraints() {
    // States: compatible with "for", not "in"
    assert!(
        AspectualClass::State.temporal_modifier_compatibility(TemporalModifierType::ForAdverbial)
    );
    assert!(
        !AspectualClass::State.temporal_modifier_compatibility(TemporalModifierType::InAdverbial)
    );

    // Activities: compatible with "for", not "in"
    assert!(
        AspectualClass::Activity
            .temporal_modifier_compatibility(TemporalModifierType::ForAdverbial)
    );
    assert!(
        !AspectualClass::Activity
            .temporal_modifier_compatibility(TemporalModifierType::InAdverbial)
    );

    // Accomplishments: compatible with both
    assert!(
        AspectualClass::Accomplishment
            .temporal_modifier_compatibility(TemporalModifierType::ForAdverbial)
    );
    assert!(
        AspectualClass::Accomplishment
            .temporal_modifier_compatibility(TemporalModifierType::InAdverbial)
    );

    // Achievements: not compatible with "for", compatible with "at"
    assert!(
        !AspectualClass::Achievement
            .temporal_modifier_compatibility(TemporalModifierType::ForAdverbial)
    );
    assert!(
        AspectualClass::Achievement
            .temporal_modifier_compatibility(TemporalModifierType::AtAdverbial)
    );
}

#[test]
fn test_participant_feature_extraction() {
    // Test animacy extraction from pronouns
    let he_word = create_test_word(1, "he", "he", UPos::Pron);
    let participant = Participant::from_word(&he_word);
    assert_eq!(participant.features.animacy, Some(Animacy::Human));

    let it_word = create_test_word(2, "it", "it", UPos::Pron);
    let participant = Participant::from_word(&it_word);
    assert_eq!(participant.features.animacy, Some(Animacy::Inanimate));

    // Test with proper nouns
    let john_word = create_test_word(3, "John", "John", UPos::Propn);
    let participant = Participant::from_word(&john_word);
    assert_eq!(participant.features.animacy, Some(Animacy::Human));
}

#[test]
fn test_aspectual_feature_composition() {
    let state_features = AspectualClass::State.features();
    let activity_features = AspectualClass::Activity.features();

    let composed = state_features.compose(&activity_features);

    // Should inherit dynamic from activity
    assert!(composed.dynamic);
    // Should inherit durative from both
    assert!(composed.durative);
    // Should not be telic (neither component is)
    assert!(!composed.telic);
}

#[test]
fn test_event_display() {
    let predicate = Predicate {
        lemma: "run".to_string(),
        semantic_type: PredicateType::Activity,
        verbnet_class: None,
        features: vec![],
    };

    let mut event = Event::new(EventId(1), predicate);

    let john = create_test_word(1, "John", "John", UPos::Propn);
    let agent = Participant::from_word(&john);
    event.add_participant(ThetaRoleType::Agent, agent);

    let display_str = format!("{}", event);
    assert!(display_str.contains("run"));
    assert!(display_str.contains("e1"));
    assert!(display_str.contains("Agent: John"));
}
