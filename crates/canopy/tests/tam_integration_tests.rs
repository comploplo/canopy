//! TAM Integration Tests
//!
//! End-to-end tests verifying that TAM (Tense, Aspect, Modality) flows through
//! the pipeline from `ComposedEvent` → `DiscourseContext` → DRS → Reasoning.

use canopy::core::{AspectualClass, ThetaRole, Voice};
use canopy::kernel::discourse::{
    AspectualOperator, AspectualViewpoint, DiscourseConfig, DiscourseContext, DrsCondition,
    TemporalFrame,
};
use canopy::kernel::events::{ComposedEvent, ComposedEvents, LittleVType, Participant};
use canopy::kernel::logic::{ClosedWorldReasoner, Reasoner};
use canopy::runtime::TokenId;
use std::collections::HashMap;

/// Create a simple event with TAM features.
fn make_event_with_tam(
    predicate: &str,
    temporal_frame: Option<TemporalFrame>,
    aspectual_viewpoint: Option<AspectualViewpoint>,
) -> ComposedEvents {
    let mut participants = HashMap::new();
    participants.insert(ThetaRole::Agent, Participant::new(TokenId::new(0), "John"));

    let event = ComposedEvent {
        id: 0,
        predicate: predicate.to_string(),
        little_v_type: LittleVType::Go,
        participants,
        aspect: AspectualClass::Activity,
        voice: Voice::Active,
        token_span: (TokenId::new(0), TokenId::new(1)),
        source_sense: None,
        decomposition_confidence: 1.0,
        binding_confidence: 1.0,
        presuppositions: Vec::new(),
        polarity: true,
        temporal_frame,
        aspectual_viewpoint,
    };

    ComposedEvents {
        events: vec![event],
        unbound_participants: Vec::new(),
        confidence: 1.0,
        sources: Vec::new(),
    }
}

#[test]
fn test_tam_flows_through_pipeline_past_tense() {
    // Create event with past tense temporal frame
    let events = make_event_with_tam("run", Some(TemporalFrame::past()), None);

    // Process through discourse context
    let mut context = DiscourseContext::new(DiscourseConfig::default());
    context.begin_sentence();
    context.process_events(&events);
    context.end_sentence();

    // Verify DRS has TemporalFrameAssignment condition
    let has_temporal_frame = context
        .drs()
        .conditions
        .iter()
        .any(|c| matches!(c, DrsCondition::TemporalFrameAssignment { .. }));
    assert!(
        has_temporal_frame,
        "DRS should have TemporalFrameAssignment condition"
    );

    // Verify reasoner can evaluate the DRS
    let reasoner = ClosedWorldReasoner::new();
    let result = reasoner.check_consistent(context.drs());
    assert!(
        result.consistent,
        "Simple past tense event should be consistent"
    );
}

#[test]
fn test_tam_flows_through_pipeline_progressive() {
    // Create event with progressive aspect
    let events = make_event_with_tam(
        "run",
        Some(TemporalFrame::past_progressive()),
        Some(AspectualViewpoint::Progressive),
    );

    // Process through discourse context
    let mut context = DiscourseContext::new(DiscourseConfig::default());
    context.begin_sentence();
    context.process_events(&events);
    context.end_sentence();

    // Verify DRS has both TemporalFrameAssignment and AspectualOp conditions
    let has_temporal_frame = context
        .drs()
        .conditions
        .iter()
        .any(|c| matches!(c, DrsCondition::TemporalFrameAssignment { .. }));
    let has_aspectual_op = context.drs().conditions.iter().any(|c| {
        matches!(
            c,
            DrsCondition::AspectualOp {
                operator: AspectualOperator::Progressive,
                ..
            }
        )
    });

    assert!(
        has_temporal_frame,
        "DRS should have TemporalFrameAssignment"
    );
    assert!(
        has_aspectual_op,
        "DRS should have AspectualOp for Progressive"
    );

    // Verify reasoner can evaluate the DRS
    let reasoner = ClosedWorldReasoner::new();
    let result = reasoner.check_consistent(context.drs());
    assert!(result.consistent, "Progressive event should be consistent");
}

#[test]
fn test_tam_flows_through_pipeline_perfect() {
    // Create event with perfect aspect
    let events = make_event_with_tam(
        "run",
        Some(TemporalFrame::past_perfect()),
        Some(AspectualViewpoint::Perfect),
    );

    // Process through discourse context
    let mut context = DiscourseContext::new(DiscourseConfig::default());
    context.begin_sentence();
    context.process_events(&events);
    context.end_sentence();

    // Verify DRS has AspectualOp for Perfect
    let has_aspectual_op = context.drs().conditions.iter().any(|c| {
        matches!(
            c,
            DrsCondition::AspectualOp {
                operator: AspectualOperator::Perfect,
                ..
            }
        )
    });

    assert!(has_aspectual_op, "DRS should have AspectualOp for Perfect");

    // Verify consistency
    let reasoner = ClosedWorldReasoner::new();
    let result = reasoner.check_consistent(context.drs());
    assert!(result.consistent, "Perfect event should be consistent");
}

#[test]
fn test_tam_with_temporal_cycle_detection() {
    use canopy::kernel::discourse::TemporalRelationType;

    // Create context with events that form a temporal cycle
    let mut context = DiscourseContext::new(DiscourseConfig::default());

    // Manually add temporal relations that form a cycle
    // This tests that the TemporalReasoner integration works
    context.begin_sentence();
    let e1 = context.introduce_event("event1");
    let e2 = context.introduce_event("event2");
    let e3 = context.introduce_event("event3");
    context.end_sentence();

    // Add cyclic temporal relations: e1 < e2 < e3 < e1
    context
        .drs_mut()
        .add_condition(DrsCondition::TemporalRelation {
            relation: TemporalRelationType::Before,
            event1: e1,
            event2: e2,
        });
    context
        .drs_mut()
        .add_condition(DrsCondition::TemporalRelation {
            relation: TemporalRelationType::Before,
            event1: e2,
            event2: e3,
        });
    context
        .drs_mut()
        .add_condition(DrsCondition::TemporalRelation {
            relation: TemporalRelationType::Before,
            event1: e3,
            event2: e1,
        });

    // The reasoner should detect this cycle
    let reasoner = ClosedWorldReasoner::new();
    let result = reasoner.check_consistent(context.drs());
    assert!(
        !result.consistent,
        "Temporal cycle should be detected as inconsistent"
    );
    assert!(
        !result.conflicts.is_empty(),
        "Should have conflict for temporal cycle"
    );
}

#[test]
fn test_multiple_events_with_tam() {
    // Create multiple events with different TAM configurations
    let mut context = DiscourseContext::new(DiscourseConfig::default());

    // First event: past perfective
    let events1 = make_event_with_tam(
        "arrive",
        Some(TemporalFrame::past()),
        Some(AspectualViewpoint::Perfective),
    );
    context.begin_sentence();
    context.process_events(&events1);
    context.end_sentence();

    // Second event: past progressive
    let events2 = make_event_with_tam(
        "run",
        Some(TemporalFrame::past_progressive()),
        Some(AspectualViewpoint::Progressive),
    );
    context.begin_sentence();
    context.process_events(&events2);
    context.end_sentence();

    // Count TAM conditions
    let temporal_frame_count = context
        .drs()
        .conditions
        .iter()
        .filter(|c| matches!(c, DrsCondition::TemporalFrameAssignment { .. }))
        .count();
    let aspectual_op_count = context
        .drs()
        .conditions
        .iter()
        .filter(|c| matches!(c, DrsCondition::AspectualOp { .. }))
        .count();

    assert_eq!(
        temporal_frame_count, 2,
        "Should have 2 TemporalFrameAssignments"
    );
    // Only progressive creates AspectualOp (perfective doesn't)
    assert_eq!(
        aspectual_op_count, 1,
        "Should have 1 AspectualOp (for progressive only)"
    );

    // Verify consistency
    let reasoner = ClosedWorldReasoner::new();
    let result = reasoner.check_consistent(context.drs());
    assert!(
        result.consistent,
        "Multiple events with TAM should be consistent"
    );
}

#[test]
fn test_tam_no_conditions_when_none() {
    // Create event without TAM features
    let events = make_event_with_tam("run", None, None);

    let mut context = DiscourseContext::new(DiscourseConfig::default());
    context.begin_sentence();
    context.process_events(&events);
    context.end_sentence();

    // Should NOT have TAM conditions
    let has_temporal_frame = context
        .drs()
        .conditions
        .iter()
        .any(|c| matches!(c, DrsCondition::TemporalFrameAssignment { .. }));
    let has_aspectual_op = context
        .drs()
        .conditions
        .iter()
        .any(|c| matches!(c, DrsCondition::AspectualOp { .. }));

    assert!(
        !has_temporal_frame,
        "Should NOT have TemporalFrameAssignment when None"
    );
    assert!(!has_aspectual_op, "Should NOT have AspectualOp when None");
}
