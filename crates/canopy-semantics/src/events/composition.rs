//! Event composition following Pietroski (2005)
//!
//! This module implements compositional event semantics where complex events
//! are built from simpler components through conjunction, temporal sequencing,
//! and causal relations. Following insights from:
//! - Pietroski (2005): "Event composition and semantic types"
//! - Rothstein (2004): "Structuring Events"
//! - Landman (1992): "The progressive and the imperfective paradox"

use super::EventId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Complex event composed from multiple sub-events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeEvent {
    /// Unique identifier for the composite event
    pub id: EventId,

    /// The sub-events that compose this event
    pub sub_events: Vec<EventId>,

    /// How the sub-events are related
    pub composition_type: CompositionType,

    /// Temporal ordering constraints between sub-events
    pub temporal_relations: Vec<TemporalRelation>,

    /// Causal relations between sub-events
    pub causal_relations: Vec<CausalRelation>,
}

/// Types of event composition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositionType {
    /// Simultaneous events: "John ran and sang"
    Conjunction,

    /// Sequential events: "John ran then sang"
    Sequence,

    /// Alternative events: "John ran or sang"
    Disjunction,

    /// Causal composition: "John's pushing caused the door to open"
    Causation,

    /// Part-whole composition: "The concert included several pieces"
    PartWhole,

    /// Iterative composition: "John knocked repeatedly"
    Iteration,
}

/// Temporal relations between events (Allen 1983)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalRelation {
    /// First event in the relation
    pub event1: EventId,

    /// Second event in the relation
    pub event2: EventId,

    /// Type of temporal relation
    pub relation_type: TemporalRelationType,
}

/// Allen's interval relations adapted for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalRelationType {
    /// e1 before e2: |e1| < |e2|
    Before,

    /// e1 meets e2: |e1| meets |e2|
    Meets,

    /// e1 overlaps e2: |e1| overlaps |e2|
    Overlaps,

    /// e1 starts e2: |e1| starts |e2|
    Starts,

    /// e1 during e2: |e1| during |e2|
    During,

    /// e1 finishes e2: |e1| finishes |e2|
    Finishes,

    /// e1 equals e2: |e1| = |e2|
    Equals,

    /// e1 simultaneous with e2 (approximate equality)
    Simultaneous,
}

/// Causal relations between events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalRelation {
    /// Causing event
    pub cause: EventId,

    /// Caused event (effect)
    pub effect: EventId,

    /// Type of causal relation
    pub causation_type: CausationType,

    /// Confidence in the causal relation (0.0-1.0)
    pub confidence: f32,
}

/// Types of causation (following philosophical literature)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CausationType {
    /// Direct physical causation: "John pushed the door"
    Direct,

    /// Indirect causation: "John's arrival caused Mary to leave"
    Indirect,

    /// Enabling causation: "John's presence enabled the meeting"
    Enabling,

    /// Preventing causation: "John's intervention prevented the accident"
    Preventing,
}

/// Event composition operations
impl CompositeEvent {
    /// Create a new composite event
    pub fn new(id: EventId, composition_type: CompositionType) -> Self {
        Self {
            id,
            sub_events: Vec::new(),
            composition_type,
            temporal_relations: Vec::new(),
            causal_relations: Vec::new(),
        }
    }

    /// Add a sub-event to the composition
    pub fn add_sub_event(&mut self, event_id: EventId) {
        if !self.sub_events.contains(&event_id) {
            self.sub_events.push(event_id);
        }
    }

    /// Add a temporal relation between sub-events
    pub fn add_temporal_relation(&mut self, relation: TemporalRelation) {
        // Verify both events are in this composition
        if self.sub_events.contains(&relation.event1) && self.sub_events.contains(&relation.event2)
        {
            self.temporal_relations.push(relation);
        }
    }

    /// Add a causal relation between sub-events
    pub fn add_causal_relation(&mut self, relation: CausalRelation) {
        // Verify both events are in this composition
        if self.sub_events.contains(&relation.cause) && self.sub_events.contains(&relation.effect) {
            self.causal_relations.push(relation);
        }
    }

    /// Check if the composition is temporally consistent
    pub fn is_temporally_consistent(&self) -> bool {
        // Check for cycles in temporal ordering
        let mut graph = TemporalGraph::new();

        for relation in &self.temporal_relations {
            match relation.relation_type {
                TemporalRelationType::Before | TemporalRelationType::Meets => {
                    graph.add_edge(relation.event1, relation.event2);
                }
                TemporalRelationType::Equals | TemporalRelationType::Simultaneous => {
                    // Bidirectional for equality
                    graph.add_edge(relation.event1, relation.event2);
                    graph.add_edge(relation.event2, relation.event1);
                }
                _ => {
                    // For complex relations, add appropriate edges
                    // This is simplified - full implementation would be more complex
                    graph.add_edge(relation.event1, relation.event2);
                }
            }
        }

        !graph.has_cycle()
    }

    /// Get all events that must precede a given event
    pub fn get_predecessors(&self, event_id: EventId) -> HashSet<EventId> {
        let mut predecessors = HashSet::new();

        for relation in &self.temporal_relations {
            match relation.relation_type {
                TemporalRelationType::Before | TemporalRelationType::Meets => {
                    if relation.event2 == event_id {
                        predecessors.insert(relation.event1);
                    }
                }
                _ => {} // Handle other relations as needed
            }
        }

        predecessors
    }

    /// Get all events that must follow a given event
    pub fn get_successors(&self, event_id: EventId) -> HashSet<EventId> {
        let mut successors = HashSet::new();

        for relation in &self.temporal_relations {
            match relation.relation_type {
                TemporalRelationType::Before | TemporalRelationType::Meets => {
                    if relation.event1 == event_id {
                        successors.insert(relation.event2);
                    }
                }
                _ => {} // Handle other relations as needed
            }
        }

        successors
    }
}

/// Simple directed graph for temporal consistency checking
struct TemporalGraph {
    edges: Vec<(EventId, EventId)>,
}

impl TemporalGraph {
    fn new() -> Self {
        Self { edges: Vec::new() }
    }

    fn add_edge(&mut self, from: EventId, to: EventId) {
        self.edges.push((from, to));
    }

    fn has_cycle(&self) -> bool {
        // Simple cycle detection using DFS
        // In a real implementation, this would be more sophisticated
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();

        for &(start, _) in &self.edges {
            if !visited.contains(&start) && self.dfs_has_cycle(start, &mut visited, &mut in_stack) {
                return true;
            }
        }

        false
    }

    fn dfs_has_cycle(
        &self,
        node: EventId,
        visited: &mut HashSet<EventId>,
        in_stack: &mut HashSet<EventId>,
    ) -> bool {
        visited.insert(node);
        in_stack.insert(node);

        // Find all neighbors
        for &(from, to) in &self.edges {
            if from == node {
                if in_stack.contains(&to) {
                    return true; // Back edge found
                }
                if !visited.contains(&to) && self.dfs_has_cycle(to, visited, in_stack) {
                    return true;
                }
            }
        }

        in_stack.remove(&node);
        false
    }
}

/// Event composition functions
pub fn compose_conjunction(event1: EventId, event2: EventId, result_id: EventId) -> CompositeEvent {
    let mut composite = CompositeEvent::new(result_id, CompositionType::Conjunction);
    composite.add_sub_event(event1);
    composite.add_sub_event(event2);

    // Conjunctive events are typically simultaneous
    let temporal_relation = TemporalRelation {
        event1,
        event2,
        relation_type: TemporalRelationType::Simultaneous,
    };
    composite.add_temporal_relation(temporal_relation);

    composite
}

pub fn compose_sequence(event1: EventId, event2: EventId, result_id: EventId) -> CompositeEvent {
    let mut composite = CompositeEvent::new(result_id, CompositionType::Sequence);
    composite.add_sub_event(event1);
    composite.add_sub_event(event2);

    // Sequential events have before relation
    let temporal_relation = TemporalRelation {
        event1,
        event2,
        relation_type: TemporalRelationType::Before,
    };
    composite.add_temporal_relation(temporal_relation);

    composite
}

pub fn compose_causation(
    cause: EventId,
    effect: EventId,
    result_id: EventId,
    causation_type: CausationType,
) -> CompositeEvent {
    let mut composite = CompositeEvent::new(result_id, CompositionType::Causation);
    composite.add_sub_event(cause);
    composite.add_sub_event(effect);

    // Causal events have temporal ordering
    let temporal_relation = TemporalRelation {
        event1: cause,
        event2: effect,
        relation_type: TemporalRelationType::Before,
    };
    composite.add_temporal_relation(temporal_relation);

    // Add causal relation
    let causal_relation = CausalRelation {
        cause,
        effect,
        causation_type,
        confidence: 0.8, // Default confidence
    };
    composite.add_causal_relation(causal_relation);

    composite
}

/// Iterative event composition (for semelfactives and repeated actions)
pub fn compose_iteration(
    base_event: EventId,
    _count: Option<u32>,
    result_id: EventId,
) -> CompositeEvent {
    let mut composite = CompositeEvent::new(result_id, CompositionType::Iteration);
    composite.add_sub_event(base_event);

    // For iteration, we might add multiple instances of the base event
    // This is a simplified representation
    composite
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_event_creation() {
        let composite = CompositeEvent::new(EventId(1), CompositionType::Conjunction);
        assert_eq!(composite.id, EventId(1));
        assert_eq!(composite.composition_type, CompositionType::Conjunction);
        assert!(composite.sub_events.is_empty());
    }

    #[test]
    fn test_conjunction_composition() {
        let event1 = EventId(1);
        let event2 = EventId(2);
        let result = EventId(3);

        let composite = compose_conjunction(event1, event2, result);

        assert_eq!(composite.composition_type, CompositionType::Conjunction);
        assert!(composite.sub_events.contains(&event1));
        assert!(composite.sub_events.contains(&event2));
        assert_eq!(composite.temporal_relations.len(), 1);
        assert_eq!(
            composite.temporal_relations[0].relation_type,
            TemporalRelationType::Simultaneous
        );
    }

    #[test]
    fn test_sequence_composition() {
        let event1 = EventId(1);
        let event2 = EventId(2);
        let result = EventId(3);

        let composite = compose_sequence(event1, event2, result);

        assert_eq!(composite.composition_type, CompositionType::Sequence);
        assert_eq!(
            composite.temporal_relations[0].relation_type,
            TemporalRelationType::Before
        );
    }

    #[test]
    fn test_causation_composition() {
        let cause = EventId(1);
        let effect = EventId(2);
        let result = EventId(3);

        let composite = compose_causation(cause, effect, result, CausationType::Direct);

        assert_eq!(composite.composition_type, CompositionType::Causation);
        assert_eq!(composite.causal_relations.len(), 1);
        assert_eq!(
            composite.causal_relations[0].causation_type,
            CausationType::Direct
        );
    }

    #[test]
    fn test_temporal_consistency() {
        let mut composite = CompositeEvent::new(EventId(0), CompositionType::Sequence);
        composite.add_sub_event(EventId(1));
        composite.add_sub_event(EventId(2));
        composite.add_sub_event(EventId(3));

        // Add consistent temporal relations: 1 -> 2 -> 3
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
    }

    #[test]
    fn test_predecessors_and_successors() {
        let mut composite = CompositeEvent::new(EventId(0), CompositionType::Sequence);
        composite.add_sub_event(EventId(1));
        composite.add_sub_event(EventId(2));
        composite.add_sub_event(EventId(3));

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

        let predecessors = composite.get_predecessors(EventId(2));
        assert!(predecessors.contains(&EventId(1)));

        let successors = composite.get_successors(EventId(2));
        assert!(successors.contains(&EventId(3)));
    }
}
