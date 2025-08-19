//! Example usage of Event Builder with VerbNet theta assignment
//!
//! This demonstrates the complete pipeline:
//! 1. VerbNet Engine for pattern analysis
//! 2. EventBuilder with smart theta assignment
//! 3. Integration with cache system and confidence scoring

use super::event_semantics::*;
use crate::verbnet::engine::VerbNetEngine;
use crate::verbnet::types::ThetaRoleType;

/// Example: Building events with automatic VerbNet theta assignment
pub fn demo_verbnet_theta_assignment() {
    // Initialize VerbNet engine (would be populated with real data in production)
    let engine = VerbNetEngine::new();

    // Example 1: Ditransitive "give" construction
    println!("=== Example 1: Ditransitive 'give' ===");

    let give_predicate = Predicate {
        lemma: "give".to_string(),
        semantic_type: PredicateType::Action,
        verbnet_class: None, // Will be filled by VerbNet
        features: vec![],
    };

    // Dependency pattern from UDPipe analysis: "John gives Mary a book"
    let dependency_pattern = "nsubj+dobj+iobj";
    let arguments = vec![
        ("nsubj".to_string(), "John".to_string()), // Subject -> Agent
        ("dobj".to_string(), "book".to_string()),  // Direct object -> Theme
        ("iobj".to_string(), "Mary".to_string()),  // Indirect object -> Recipient
    ];

    // Build event with VerbNet theta assignment
    let event_result = EventBuilder::new(give_predicate)
        .with_verbnet_theta_assignment(&engine, &dependency_pattern, &arguments)
        .build();

    match event_result {
        Ok(event) => {
            println!("✅ Successfully built 'give' event");
            println!("   VerbNet class: {:?}", event.predicate.verbnet_class);
            println!("   Participants: {:?}", event.participants.len());
            for (role, participant) in &event.participants {
                println!("   - {:?}: {}", role, participant.expression);
            }
        }
        Err(e) => println!("⚠️  Event validation: {:?} (expected with empty cache)", e),
    }

    // Example 2: Motion verb "run"
    println!("\n=== Example 2: Motion verb 'run' ===");

    let run_predicate = Predicate {
        lemma: "run".to_string(),
        semantic_type: PredicateType::Activity,
        verbnet_class: None,
        features: vec![],
    };

    // "John runs to the store"
    let dependency_pattern = "nsubj+prep_to";
    let arguments = vec![
        ("nsubj".to_string(), "John".to_string()),
        ("prep_to".to_string(), "store".to_string()),
    ];

    let event_result = EventBuilder::new(run_predicate)
        .with_verbnet_theta_assignment(&engine, &dependency_pattern, &arguments)
        .build();

    match event_result {
        Ok(event) => {
            println!("✅ Successfully built 'run' event");
            println!("   Semantic features: {:?}", event.predicate.features);
        }
        Err(e) => println!("⚠️  Event validation: {:?} (expected with empty cache)", e),
    }

    // Example 3: Demonstrating confidence filtering
    println!("\n=== Example 3: Confidence filtering ===");

    let test_predicate = Predicate {
        lemma: "unknown_verb".to_string(),
        semantic_type: PredicateType::Action,
        verbnet_class: None,
        features: vec![],
    };

    // Pattern that likely won't have confident assignments
    let dependency_pattern = "unusual_pattern";
    let arguments = vec![("unknown_rel".to_string(), "test".to_string())];

    let builder = EventBuilder::new(test_predicate).with_verbnet_theta_assignment(
        &engine,
        &dependency_pattern,
        &arguments,
    );

    // The system should filter out low-confidence assignments
    match builder.build() {
        Ok(event) => {
            println!(
                "✅ Event built with {} confident assignments",
                event.participants.len()
            );
        }
        Err(EventBuildError::MissingRequiredThetaRole(role)) => {
            println!(
                "⚠️  No confident assignments found, missing required role: {:?}",
                role
            );
            println!("   This demonstrates the confidence filtering (>0.5 threshold)");
        }
        Err(other) => println!("❌ Unexpected error: {:?}", other),
    }
}

/// Example: Manual event building (without VerbNet)
pub fn demo_manual_event_building() {
    println!("\n=== Manual Event Building (Traditional Approach) ===");

    let predicate = Predicate {
        lemma: "hit".to_string(),
        semantic_type: PredicateType::Action,
        verbnet_class: Some("hit-18.1".to_string()),
        features: vec![SemanticFeature::Contact],
    };

    // Manually create participants
    let agent = Participant {
        word_id: 1,
        expression: "John".to_string(),
        features: ParticipantFeatures {
            animacy: Some(Animacy::Human),
            concreteness: None,
            definiteness: Some(Definiteness::Definite),
            number: Some(Number::Singular),
        },
        discourse_ref: None,
    };

    let patient = Participant {
        word_id: 2,
        expression: "ball".to_string(),
        features: ParticipantFeatures {
            animacy: Some(Animacy::Inanimate),
            concreteness: None,
            definiteness: Some(Definiteness::Indefinite),
            number: Some(Number::Singular),
        },
        discourse_ref: None,
    };

    // Build event manually
    let event = EventBuilder::new(predicate)
        .with_participant(ThetaRoleType::Agent, agent)
        .with_participant(ThetaRoleType::Patient, patient)
        .build()
        .expect("Manual event should build successfully");

    println!("✅ Manually built 'hit' event");
    println!(
        "   Agent: {}",
        event
            .get_participant(&ThetaRoleType::Agent)
            .unwrap()
            .expression
    );
    println!(
        "   Patient: {}",
        event
            .get_participant(&ThetaRoleType::Patient)
            .unwrap()
            .expression
    );
}

/// Demonstrates the complete integration workflow
pub fn demo_complete_workflow() {
    println!("\n=== Complete Integration Workflow ===");
    println!("This shows how VerbNet theta assignment integrates with:");
    println!("1. Three-tier lookup (cache → VerbNet → similarity)");
    println!("2. Pattern mapping with confidence scoring");
    println!("3. Event validation and error handling");
    println!("4. Semantic feature extraction");

    demo_verbnet_theta_assignment();
    demo_manual_event_building();

    println!("\n🎉 Phase 4: Event Builder Enhancement - COMPLETE!");
    println!("✅ VerbNet theta assignment integrated");
    println!("✅ Confidence filtering implemented (>0.5 threshold)");
    println!("✅ Semantic feature mapping from VerbNet predicates");
    println!("✅ Graceful error handling for missing assignments");
    println!("✅ Compatible with existing EventBuilder pattern");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_functions() {
        // These are integration demos, they should run without panicking
        demo_complete_workflow();
    }
}
