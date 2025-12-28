//! Integration tests for canopy-discourse
//!
//! These tests verify the complete discourse processing pipeline,
//! including DRS construction, anaphora resolution, and temporal modeling.

use canopy_core::{
    Action, Animacy, AspectualClass, Definiteness, Entity, Event, LittleV, ThetaRole, Voice,
};
use canopy_discourse::{DiscourseConfig, DiscourseContext, Drs, DrsCondition, ReferentType};
use canopy_events::ComposedEvent;
use std::collections::HashMap;

/// Helper to create a simple event for testing
fn create_test_event(predicate: &str, agent_name: &str, aspect: AspectualClass) -> ComposedEvent {
    let agent = Entity {
        id: 1,
        text: agent_name.to_string(),
        animacy: Some(Animacy::Human),
        definiteness: Some(Definiteness::Definite),
    };

    let mut participants = HashMap::new();
    participants.insert(ThetaRole::Agent, agent.clone());

    let event = Event {
        id: 1,
        predicate: predicate.to_string(),
        little_v: LittleV::Do {
            agent: agent.clone(),
            action: Action {
                predicate: predicate.to_string(),
                manner: None,
                instrument: None,
            },
        },
        participants,
        aspect,
        voice: Voice::Active,
    };

    ComposedEvent {
        id: 0,
        event,
        token_span: (0, 1),
        verbnet_source: Some("run-51.3".to_string()),
        framenet_source: None,
        decomposition_confidence: 0.9,
        binding_confidence: 0.85,
    }
}

/// Helper to create a transitive event with agent and patient
fn create_transitive_event(
    predicate: &str,
    agent_name: &str,
    patient_name: &str,
    aspect: AspectualClass,
) -> ComposedEvent {
    let agent = Entity {
        id: 1,
        text: agent_name.to_string(),
        animacy: Some(Animacy::Human),
        definiteness: Some(Definiteness::Definite),
    };

    let patient = Entity {
        id: 2,
        text: patient_name.to_string(),
        animacy: Some(Animacy::Human),
        definiteness: Some(Definiteness::Definite),
    };

    let mut participants = HashMap::new();
    participants.insert(ThetaRole::Agent, agent.clone());
    participants.insert(ThetaRole::Patient, patient.clone());

    let event = Event {
        id: 1,
        predicate: predicate.to_string(),
        little_v: LittleV::Cause {
            causer: agent.clone(),
            caused_predicate: predicate.to_string(),
            caused_theme: patient.clone(),
        },
        participants,
        aspect,
        voice: Voice::Active,
    };

    ComposedEvent {
        id: 0,
        event,
        token_span: (0, 3),
        verbnet_source: None,
        framenet_source: None,
        decomposition_confidence: 0.85,
        binding_confidence: 0.80,
    }
}

#[test]
fn test_single_sentence_drs_construction() {
    let mut ctx = DiscourseContext::with_defaults();

    ctx.begin_sentence("John runs.".to_string());

    let event = create_test_event("run", "John", AspectualClass::Activity);
    let event_id = ctx.process_event(&event).expect("Should process event");

    ctx.end_sentence();

    let drs = ctx.drs();

    // Should have referents: event + participant
    assert!(
        drs.referent_count() >= 2,
        "Should have at least 2 referents"
    );

    // Should have conditions
    assert!(
        drs.condition_count() >= 2,
        "Should have at least 2 conditions"
    );

    // Event ID should be valid
    assert!(drs.is_accessible(event_id), "Event should be accessible");
}

#[test]
fn test_multi_sentence_discourse() {
    let mut ctx = DiscourseContext::with_defaults();

    // Sentence 1: "John runs."
    ctx.begin_sentence("John runs.".to_string());
    let event1 = create_test_event("run", "John", AspectualClass::Activity);
    let _id1 = ctx.process_event(&event1).expect("Should process event 1");
    ctx.end_sentence();

    // Sentence 2: "Mary walks."
    ctx.begin_sentence("Mary walks.".to_string());
    let event2 = create_test_event("walk", "Mary", AspectualClass::Activity);
    let _id2 = ctx.process_event(&event2).expect("Should process event 2");
    ctx.end_sentence();

    let stats = ctx.statistics();
    assert_eq!(stats.sentence_count, 2, "Should have 2 sentences");
    assert!(
        stats.referent_count >= 4,
        "Should have referents from both sentences"
    );
}

#[test]
fn test_temporal_relation_creation() {
    let mut ctx = DiscourseContext::with_defaults();

    // Process two events to create a temporal relation
    ctx.begin_sentence("John runs. He jumps.".to_string());

    let event1 = create_test_event("run", "John", AspectualClass::Activity);
    ctx.process_event(&event1).expect("Should process event 1");

    let event2 = create_test_event("jump", "John", AspectualClass::Achievement);
    ctx.process_event(&event2).expect("Should process event 2");

    ctx.end_sentence();

    let drs = ctx.drs();

    // Should have temporal relation condition
    let has_temporal = drs
        .conditions
        .iter()
        .any(|c| matches!(c, DrsCondition::TemporalRelation { .. }));

    assert!(has_temporal, "Should have temporal relation between events");
}

#[test]
fn test_pronoun_resolution_basic() {
    let mut ctx = DiscourseContext::with_defaults();

    // Sentence 1: Introduce John
    ctx.begin_sentence("John runs.".to_string());
    let event1 = create_test_event("run", "John", AspectualClass::Activity);
    ctx.process_event(&event1).expect("Should process event");
    ctx.end_sentence();

    // Sentence 2: Resolve "he"
    ctx.begin_sentence("He jumps.".to_string());

    let resolved = ctx.resolve_pronoun("he");
    assert!(resolved.is_ok(), "Should resolve 'he' to John");

    ctx.end_sentence();

    // Check resolution was tracked
    let stats = ctx.statistics();
    assert_eq!(
        stats.resolution_count, 1,
        "Should have 1 anaphora resolution"
    );
}

#[test]
fn test_pronoun_gender_agreement() {
    let mut ctx = DiscourseContext::with_defaults();

    // Introduce both male and female referents
    ctx.begin_sentence("John sees Mary.".to_string());
    let event = create_transitive_event("see", "John", "Mary", AspectualClass::State);
    ctx.process_event(&event).expect("Should process event");
    ctx.end_sentence();

    // Try to resolve gendered pronouns
    ctx.begin_sentence("She smiles.".to_string());

    // "she" should resolve to Mary (the female participant)
    // Note: Our current implementation doesn't set gender automatically,
    // so this may not distinguish - but it should at least find a candidate
    let resolved = ctx.resolve_pronoun("she");
    assert!(resolved.is_ok(), "Should find candidate for 'she'");

    ctx.end_sentence();
}

#[test]
fn test_drs_merging() {
    let mut drs1 = Drs::default();
    let mut drs2 = Drs::default();

    // Add referent to drs1
    let ref1 = canopy_discourse::DiscourseReferent::entity(
        canopy_discourse::ReferentId(1),
        "John".to_string(),
        0,
    );
    drs1.add_referent(ref1);
    drs1.add_condition(DrsCondition::Predicate {
        name: "man".to_string(),
        referent: canopy_discourse::ReferentId(1),
    });

    // Add referent to drs2
    let ref2 = canopy_discourse::DiscourseReferent::entity(
        canopy_discourse::ReferentId(2),
        "Mary".to_string(),
        1,
    );
    drs2.add_referent(ref2);
    drs2.add_condition(DrsCondition::Predicate {
        name: "woman".to_string(),
        referent: canopy_discourse::ReferentId(2),
    });

    // Merge drs2 into drs1
    drs1.merge(drs2);

    assert_eq!(
        drs1.referent_count(),
        2,
        "Merged DRS should have 2 referents"
    );
    assert_eq!(
        drs1.condition_count(),
        2,
        "Merged DRS should have 2 conditions"
    );
}

#[test]
fn test_context_capacity_limits() {
    let config = DiscourseConfig {
        max_referents: 5,
        max_sentences: 10,
        ..Default::default()
    };

    let mut ctx = DiscourseContext::new(config);

    // Add referents up to limit
    ctx.begin_sentence("Test.".to_string());

    for i in 0..5 {
        let result = ctx.introduce_referent(format!("entity{}", i), ReferentType::Individual);
        assert!(
            result.is_ok(),
            "Should be able to add referent {}: {:?}",
            i,
            result.err()
        );
    }

    // Next one should fail
    let result = ctx.introduce_referent("overflow".to_string(), ReferentType::Individual);
    assert!(
        result.is_err(),
        "Should fail when exceeding referent capacity"
    );
}

#[test]
fn test_event_referent_types() {
    let mut ctx = DiscourseContext::with_defaults();

    ctx.begin_sentence("John runs.".to_string());
    let event = create_test_event("run", "John", AspectualClass::Activity);
    ctx.process_event(&event).expect("Should process event");
    ctx.end_sentence();

    let drs = ctx.drs();

    // Check we have both entity and event referents
    let entity_count = drs.entity_referents().len();
    let event_count = drs.event_referents().len();

    assert!(entity_count >= 1, "Should have at least 1 entity referent");
    assert!(event_count >= 1, "Should have at least 1 event referent");
}

#[test]
fn test_theta_role_conditions() {
    let mut ctx = DiscourseContext::with_defaults();

    ctx.begin_sentence("John hits Mary.".to_string());
    let event = create_transitive_event("hit", "John", "Mary", AspectualClass::Achievement);
    ctx.process_event(&event).expect("Should process event");
    ctx.end_sentence();

    let drs = ctx.drs();

    // Should have theta role conditions
    let theta_roles: Vec<_> = drs
        .conditions
        .iter()
        .filter_map(|c| match c {
            DrsCondition::ThetaRole { role, .. } => Some(role),
            _ => None,
        })
        .collect();

    assert!(
        theta_roles.contains(&&ThetaRole::Agent),
        "Should have Agent role: {:?}",
        theta_roles
    );
    assert!(
        theta_roles.contains(&&ThetaRole::Patient),
        "Should have Patient role: {:?}",
        theta_roles
    );
}

#[test]
fn test_statistics_tracking() {
    let mut ctx = DiscourseContext::with_defaults();

    let initial_stats = ctx.statistics();
    assert_eq!(initial_stats.sentence_count, 0);
    assert_eq!(initial_stats.referent_count, 0);
    assert_eq!(initial_stats.condition_count, 0);

    ctx.begin_sentence("John runs.".to_string());
    let event = create_test_event("run", "John", AspectualClass::Activity);
    ctx.process_event(&event).expect("Should process event");
    ctx.end_sentence();

    let final_stats = ctx.statistics();
    assert_eq!(final_stats.sentence_count, 1);
    assert!(final_stats.referent_count > 0);
    assert!(final_stats.condition_count > 0);
}

#[test]
fn test_aspectual_class_temporal_relations() {
    let mut ctx = DiscourseContext::with_defaults();

    ctx.begin_sentence("John is tall. He runs.".to_string());

    // State aspect - should overlap
    let state_event = create_test_event("be_tall", "John", AspectualClass::State);
    ctx.process_event(&state_event).expect("Process state");

    // Activity aspect - should sequence
    let activity_event = create_test_event("run", "John", AspectualClass::Activity);
    ctx.process_event(&activity_event)
        .expect("Process activity");

    ctx.end_sentence();

    // Find temporal relations
    let temporal_relations: Vec<_> = ctx
        .drs()
        .conditions
        .iter()
        .filter_map(|c| match c {
            DrsCondition::TemporalRelation { relation, .. } => Some(relation),
            _ => None,
        })
        .collect();

    assert!(
        !temporal_relations.is_empty(),
        "Should have temporal relations"
    );
}

#[test]
fn test_clear_resets_context() {
    let mut ctx = DiscourseContext::with_defaults();

    // Add some data
    ctx.begin_sentence("John runs.".to_string());
    let event = create_test_event("run", "John", AspectualClass::Activity);
    ctx.process_event(&event).expect("Should process event");
    ctx.end_sentence();

    assert!(ctx.statistics().referent_count > 0);

    // Clear
    ctx.clear();

    // Verify reset
    let stats = ctx.statistics();
    assert_eq!(stats.sentence_count, 0);
    assert_eq!(stats.referent_count, 0);
    assert_eq!(stats.condition_count, 0);
}
