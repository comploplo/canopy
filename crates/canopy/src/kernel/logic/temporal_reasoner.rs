//! Temporal reasoning using Allen interval algebra.
//!
//! Implements constraint propagation for temporal relations between events,
//! detecting inconsistencies and inferring implicit orderings.
//!
//! # Allen's Interval Algebra
//!
//! The 13 basic relations between intervals:
//! - before (<), after (>)
//! - meets (m), met-by (mi)
//! - overlaps (o), overlapped-by (oi)
//! - starts (s), started-by (si)
//! - during (d), contains (di)
//! - finishes (f), finished-by (fi)
//! - equals (=)

use crate::kernel::discourse::{ReferentId, TemporalRelationType};
use std::collections::HashMap;

/// Allen interval relation (extended for our needs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllenRelation {
    /// A is entirely before B
    Before,
    /// A is entirely after B
    After,
    /// A meets B (A ends exactly when B starts)
    Meets,
    /// A is met by B
    MetBy,
    /// A overlaps with B
    Overlaps,
    /// A is overlapped by B
    OverlappedBy,
    /// A starts B (same start, A shorter)
    Starts,
    /// A is started by B
    StartedBy,
    /// A is during B (A contained in B)
    During,
    /// A contains B
    Contains,
    /// A finishes B (same end, A shorter)
    Finishes,
    /// A is finished by B
    FinishedBy,
    /// A equals B
    Equals,
    /// Unknown relation
    Unknown,
}

impl AllenRelation {
    /// Get the inverse relation.
    #[must_use]
    pub fn inverse(&self) -> Self {
        match self {
            Self::Before => Self::After,
            Self::After => Self::Before,
            Self::Meets => Self::MetBy,
            Self::MetBy => Self::Meets,
            Self::Overlaps => Self::OverlappedBy,
            Self::OverlappedBy => Self::Overlaps,
            Self::Starts => Self::StartedBy,
            Self::StartedBy => Self::Starts,
            Self::During => Self::Contains,
            Self::Contains => Self::During,
            Self::Finishes => Self::FinishedBy,
            Self::FinishedBy => Self::Finishes,
            Self::Equals => Self::Equals,
            Self::Unknown => Self::Unknown,
        }
    }

    /// Convert from DRS temporal relation type.
    #[must_use]
    pub fn from_temporal_relation(rel: TemporalRelationType) -> Self {
        match rel {
            TemporalRelationType::Before => Self::Before,
            TemporalRelationType::After => Self::After,
            TemporalRelationType::During => Self::During,
            TemporalRelationType::Contains => Self::Contains,
            TemporalRelationType::Overlaps => Self::Overlaps,
            TemporalRelationType::Simultaneous => Self::Equals,
            TemporalRelationType::Meets => Self::Meets,
        }
    }

    /// Convert to DRS temporal relation type if possible.
    #[must_use]
    pub fn to_temporal_relation(&self) -> Option<TemporalRelationType> {
        match self {
            Self::Before => Some(TemporalRelationType::Before),
            Self::After => Some(TemporalRelationType::After),
            Self::During => Some(TemporalRelationType::During),
            Self::Contains => Some(TemporalRelationType::Contains),
            Self::Overlaps | Self::OverlappedBy => Some(TemporalRelationType::Overlaps),
            Self::Equals => Some(TemporalRelationType::Simultaneous),
            _ => None,
        }
    }
}

/// A temporal constraint between two events.
#[derive(Debug, Clone)]
pub struct TemporalConstraint {
    /// First event/interval.
    pub from: ReferentId,
    /// Second event/interval.
    pub to: ReferentId,
    /// The relation between them.
    pub relation: AllenRelation,
    /// Confidence in this constraint (0.0 - 1.0).
    pub confidence: f32,
    /// Source of this constraint (e.g., "coherence:narration", "explicit:before").
    pub source: String,
}

impl TemporalConstraint {
    /// Create a new constraint.
    #[must_use]
    pub fn new(
        from: ReferentId,
        to: ReferentId,
        relation: AllenRelation,
        source: impl Into<String>,
    ) -> Self {
        Self {
            from,
            to,
            relation,
            confidence: 1.0,
            source: source.into(),
        }
    }

    /// Create with explicit confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Result of temporal consistency checking.
#[derive(Debug, Clone)]
pub struct TemporalConsistencyResult {
    /// Whether the constraints are consistent.
    pub is_consistent: bool,
    /// Detected cycle if inconsistent.
    pub cycle: Option<Vec<ReferentId>>,
    /// Inferred constraints from propagation.
    pub inferred: Vec<TemporalConstraint>,
}

/// Temporal constraint reasoner using Allen algebra.
#[derive(Debug, Default)]
pub struct TemporalReasoner {
    /// All constraints, indexed by (from, to) pair.
    constraints: HashMap<(ReferentId, ReferentId), TemporalConstraint>,
    /// All event referents known.
    events: Vec<ReferentId>,
}

impl TemporalReasoner {
    /// Create a new temporal reasoner.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a temporal constraint.
    pub fn add_constraint(&mut self, constraint: TemporalConstraint) {
        // Track events
        if !self.events.contains(&constraint.from) {
            self.events.push(constraint.from);
        }
        if !self.events.contains(&constraint.to) {
            self.events.push(constraint.to);
        }

        // Add the constraint
        let key = (constraint.from, constraint.to);
        self.constraints.insert(key, constraint);
    }

    /// Add constraint from DRS temporal relation.
    pub fn add_from_drs_relation(
        &mut self,
        from: ReferentId,
        to: ReferentId,
        relation: TemporalRelationType,
        source: impl Into<String>,
    ) {
        let allen = AllenRelation::from_temporal_relation(relation);
        self.add_constraint(TemporalConstraint::new(from, to, allen, source));
    }

    /// Get the constraint between two events.
    #[must_use]
    pub fn get_constraint(&self, from: ReferentId, to: ReferentId) -> Option<&TemporalConstraint> {
        self.constraints.get(&(from, to))
    }

    /// Check consistency and propagate constraints.
    ///
    /// Uses path consistency (Allen's composition) to detect cycles
    /// and infer new constraints.
    #[must_use]
    pub fn check_consistency(&self) -> TemporalConsistencyResult {
        // Build adjacency representation for cycle detection
        let mut graph: HashMap<ReferentId, Vec<(ReferentId, AllenRelation)>> = HashMap::new();

        for constraint in self.constraints.values() {
            graph
                .entry(constraint.from)
                .or_default()
                .push((constraint.to, constraint.relation));
        }

        // DFS for cycle detection (simplified - focuses on Before/After chains)
        let mut visited = HashMap::new();
        let mut in_stack = HashMap::new();
        let mut cycle_path = Vec::new();

        for event in &self.events {
            if !visited.contains_key(event) {
                if let Some(cycle) = Self::dfs_cycle_check(
                    *event,
                    &graph,
                    &mut visited,
                    &mut in_stack,
                    &mut cycle_path,
                ) {
                    return TemporalConsistencyResult {
                        is_consistent: false,
                        cycle: Some(cycle),
                        inferred: vec![],
                    };
                }
            }
        }

        // Run constraint propagation to infer new constraints
        let inferred = self.propagate_constraints();

        TemporalConsistencyResult {
            is_consistent: true,
            cycle: None,
            inferred,
        }
    }

    /// DFS for cycle detection in Before/After relations.
    fn dfs_cycle_check(
        node: ReferentId,
        graph: &HashMap<ReferentId, Vec<(ReferentId, AllenRelation)>>,
        visited: &mut HashMap<ReferentId, bool>,
        in_stack: &mut HashMap<ReferentId, bool>,
        path: &mut Vec<ReferentId>,
    ) -> Option<Vec<ReferentId>> {
        visited.insert(node, true);
        in_stack.insert(node, true);
        path.push(node);

        if let Some(neighbors) = graph.get(&node) {
            for (neighbor, relation) in neighbors {
                // Only follow Before relations for cycle detection
                // (Before cycles are impossible in a consistent ordering)
                if *relation == AllenRelation::Before {
                    if !visited.contains_key(neighbor) {
                        if let Some(cycle) =
                            Self::dfs_cycle_check(*neighbor, graph, visited, in_stack, path)
                        {
                            return Some(cycle);
                        }
                    } else if in_stack.get(neighbor).copied().unwrap_or(false) {
                        // Found a cycle
                        let cycle_start = path.iter().position(|x| x == neighbor).unwrap_or(0);
                        return Some(path[cycle_start..].to_vec());
                    }
                }
            }
        }

        in_stack.insert(node, false);
        path.pop();
        None
    }

    /// Propagate constraints to infer new orderings.
    ///
    /// Uses Allen's composition table for transitive inference.
    fn propagate_constraints(&self) -> Vec<TemporalConstraint> {
        let mut inferred = Vec::new();

        // Simple transitive closure for Before relations
        // If A before B and B before C, then A before C
        for c1 in self.constraints.values() {
            if c1.relation == AllenRelation::Before {
                for c2 in self.constraints.values() {
                    if c2.relation == AllenRelation::Before && c1.to == c2.from {
                        // Check if we already have this constraint
                        if !self.constraints.contains_key(&(c1.from, c2.to)) {
                            let new_constraint = TemporalConstraint::new(
                                c1.from,
                                c2.to,
                                AllenRelation::Before,
                                format!("inferred:transitive({},{})", c1.source, c2.source),
                            )
                            .with_confidence(c1.confidence * c2.confidence * 0.9);

                            inferred.push(new_constraint);
                        }
                    }
                }
            }
        }

        // During is also transitive
        for c1 in self.constraints.values() {
            if c1.relation == AllenRelation::During {
                for c2 in self.constraints.values() {
                    if c2.relation == AllenRelation::During
                        && c1.to == c2.from
                        && !self.constraints.contains_key(&(c1.from, c2.to))
                    {
                        let new_constraint = TemporalConstraint::new(
                            c1.from,
                            c2.to,
                            AllenRelation::During,
                            "inferred:during-transitive".to_string(),
                        )
                        .with_confidence(c1.confidence * c2.confidence * 0.9);

                        inferred.push(new_constraint);
                    }
                }
            }
        }

        inferred
    }

    /// Get all constraints.
    pub fn all_constraints(&self) -> impl Iterator<Item = &TemporalConstraint> {
        self.constraints.values()
    }

    /// Get count of events tracked.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Clear all constraints.
    pub fn clear(&mut self) {
        self.constraints.clear();
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allen_relation_inverse() {
        assert_eq!(AllenRelation::Before.inverse(), AllenRelation::After);
        assert_eq!(AllenRelation::After.inverse(), AllenRelation::Before);
        assert_eq!(AllenRelation::During.inverse(), AllenRelation::Contains);
        assert_eq!(AllenRelation::Equals.inverse(), AllenRelation::Equals);
    }

    #[test]
    fn test_allen_from_temporal_relation() {
        assert_eq!(
            AllenRelation::from_temporal_relation(TemporalRelationType::Before),
            AllenRelation::Before
        );
        assert_eq!(
            AllenRelation::from_temporal_relation(TemporalRelationType::Simultaneous),
            AllenRelation::Equals
        );
    }

    #[test]
    fn test_consistent_constraints() {
        let mut reasoner = TemporalReasoner::new();

        let e1 = ReferentId::new(0);
        let e2 = ReferentId::new(1);
        let e3 = ReferentId::new(2);

        // e1 before e2 before e3
        reasoner.add_constraint(TemporalConstraint::new(
            e1,
            e2,
            AllenRelation::Before,
            "test",
        ));
        reasoner.add_constraint(TemporalConstraint::new(
            e2,
            e3,
            AllenRelation::Before,
            "test",
        ));

        let result = reasoner.check_consistency();

        assert!(result.is_consistent);
        assert!(result.cycle.is_none());
        // Should infer e1 before e3
        assert!(!result.inferred.is_empty());
    }

    #[test]
    fn test_inconsistent_cycle() {
        let mut reasoner = TemporalReasoner::new();

        let e1 = ReferentId::new(0);
        let e2 = ReferentId::new(1);
        let e3 = ReferentId::new(2);

        // Create a cycle: e1 < e2 < e3 < e1
        reasoner.add_constraint(TemporalConstraint::new(
            e1,
            e2,
            AllenRelation::Before,
            "test",
        ));
        reasoner.add_constraint(TemporalConstraint::new(
            e2,
            e3,
            AllenRelation::Before,
            "test",
        ));
        reasoner.add_constraint(TemporalConstraint::new(
            e3,
            e1,
            AllenRelation::Before,
            "test",
        ));

        let result = reasoner.check_consistency();

        assert!(!result.is_consistent);
        assert!(result.cycle.is_some());
    }

    #[test]
    fn test_from_drs_relation() {
        let mut reasoner = TemporalReasoner::new();

        let e1 = ReferentId::new(0);
        let e2 = ReferentId::new(1);

        reasoner.add_from_drs_relation(e1, e2, TemporalRelationType::Before, "coherence:narration");

        let constraint = reasoner.get_constraint(e1, e2).unwrap();
        assert_eq!(constraint.relation, AllenRelation::Before);
    }

    #[test]
    fn test_transitive_inference() {
        let mut reasoner = TemporalReasoner::new();

        let e1 = ReferentId::new(0);
        let e2 = ReferentId::new(1);
        let e3 = ReferentId::new(2);

        reasoner.add_constraint(TemporalConstraint::new(
            e1,
            e2,
            AllenRelation::Before,
            "explicit",
        ));
        reasoner.add_constraint(TemporalConstraint::new(
            e2,
            e3,
            AllenRelation::Before,
            "explicit",
        ));

        let result = reasoner.check_consistency();

        // Should infer e1 before e3
        assert_eq!(result.inferred.len(), 1);
        assert_eq!(result.inferred[0].relation, AllenRelation::Before);
    }

    #[test]
    fn test_empty_reasoner() {
        let reasoner = TemporalReasoner::new();
        let result = reasoner.check_consistency();

        assert!(result.is_consistent);
        assert!(result.cycle.is_none());
        assert!(result.inferred.is_empty());
    }

    #[test]
    fn test_single_constraint() {
        let mut reasoner = TemporalReasoner::new();

        let e1 = ReferentId::new(0);
        let e2 = ReferentId::new(1);

        reasoner.add_constraint(TemporalConstraint::new(
            e1,
            e2,
            AllenRelation::Before,
            "test",
        ));

        assert_eq!(reasoner.event_count(), 2);

        let result = reasoner.check_consistency();
        assert!(result.is_consistent);
    }
}
