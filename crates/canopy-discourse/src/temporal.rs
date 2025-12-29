//! Temporal Reasoning with Allen's Interval Algebra
//!
//! Implements sophisticated temporal reasoning based on Allen (1983)
//! "Maintaining Knowledge about Temporal Intervals".
//!
//! This module extends the basic temporal relations in DRS with:
//! - Full interval algebra (13 relations with inverses)
//! - Tense-based temporal inference (Dowty 1986)
//! - Temporal constraint propagation
//! - Narrative sequence analysis

use crate::drs::TemporalRelationType;
use crate::referent::ReferentId;
use canopy_core::AspectualClass;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Allen's 13 interval relations (Allen 1983)
///
/// These relations are mutually exclusive and exhaustive for any two
/// temporal intervals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllenRelation {
    // === 7 Basic Relations ===
    /// e1 entirely precedes e2 with a gap: e1 |---| ... |---|  e2
    Before,

    /// e1 ends exactly when e2 starts: e1 |---|e2|---|
    Meets,

    /// e1 starts before e2, overlaps, ends during e2: e1 |--[--|--]  e2
    Overlaps,

    /// e1 and e2 start together, e1 ends first: [e1|---]-----|  e2
    Starts,

    /// e1 is completely contained in e2: e2 |---[e1]---|
    During,

    /// e1 starts after e2, both end together: e2 |-----[e1|---]
    Finishes,

    /// e1 and e2 are identical: e1 [========] e2
    Equals,

    // === 6 Inverse Relations ===
    /// e1 entirely follows e2 with a gap (inverse of Before)
    After,

    /// e2 ends exactly when e1 starts (inverse of Meets)
    MetBy,

    /// e1 starts during e2, ends after e2 (inverse of Overlaps)
    OverlappedBy,

    /// e2 starts with e1, e2 ends first (inverse of Starts)
    StartedBy,

    /// e1 completely contains e2 (inverse of During)
    Contains,

    /// e2 starts after e1, both end together (inverse of Finishes)
    FinishedBy,
}

impl AllenRelation {
    /// Get the inverse relation
    ///
    /// If R(e1, e2), then R.inverse()(e2, e1)
    #[must_use]
    pub fn inverse(self) -> Self {
        match self {
            Self::Before => Self::After,
            Self::Meets => Self::MetBy,
            Self::Overlaps => Self::OverlappedBy,
            Self::Starts => Self::StartedBy,
            Self::During => Self::Contains,
            Self::Finishes => Self::FinishedBy,
            Self::Equals => Self::Equals,
            // Inverses of inverses
            Self::After => Self::Before,
            Self::MetBy => Self::Meets,
            Self::OverlappedBy => Self::Overlaps,
            Self::StartedBy => Self::Starts,
            Self::Contains => Self::During,
            Self::FinishedBy => Self::Finishes,
        }
    }

    /// Check if this relation implies temporal precedence
    #[must_use]
    pub fn implies_before(self) -> bool {
        matches!(self, Self::Before | Self::Meets)
    }

    /// Check if this relation implies temporal overlap
    #[must_use]
    pub fn implies_overlap(self) -> bool {
        matches!(
            self,
            Self::Overlaps
                | Self::OverlappedBy
                | Self::Starts
                | Self::StartedBy
                | Self::During
                | Self::Contains
                | Self::Finishes
                | Self::FinishedBy
                | Self::Equals
        )
    }

    /// Convert to DRS temporal relation type (lossy - DRS has fewer types)
    #[must_use]
    pub fn to_drs_relation(self) -> TemporalRelationType {
        match self {
            Self::Before => TemporalRelationType::Before,
            Self::After => TemporalRelationType::After,
            Self::Meets => TemporalRelationType::Meets,
            Self::MetBy => TemporalRelationType::After, // Approximate
            Self::Overlaps | Self::OverlappedBy => TemporalRelationType::Overlaps,
            Self::Starts | Self::StartedBy => TemporalRelationType::Simultaneous, // Approximate
            Self::During => TemporalRelationType::During,
            Self::Contains => TemporalRelationType::Contains,
            Self::Finishes | Self::FinishedBy => TemporalRelationType::Simultaneous, // Approximate
            Self::Equals => TemporalRelationType::Simultaneous,
        }
    }
}

/// Grammatical tense for temporal reasoning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tense {
    /// Simple past: "walked"
    Past,
    /// Simple present: "walks"
    Present,
    /// Simple future: "will walk"
    Future,
    /// Past perfect: "had walked" - prior to past reference time
    PastPerfect,
    /// Present perfect: "has walked" - relevant to now
    PresentPerfect,
    /// Future perfect: "will have walked" - prior to future reference
    FuturePerfect,
    /// Progressive aspect: "is walking"
    Progressive,
    /// Past progressive: "was walking"
    PastProgressive,
}

impl Tense {
    /// Check if this tense indicates completed action before reference time
    #[must_use]
    pub fn is_anterior(self) -> bool {
        matches!(
            self,
            Self::PastPerfect | Self::PresentPerfect | Self::FuturePerfect
        )
    }

    /// Check if this tense indicates ongoing action
    #[must_use]
    pub fn is_progressive(self) -> bool {
        matches!(self, Self::Progressive | Self::PastProgressive)
    }
}

/// Temporal constraint between two events
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalConstraint {
    pub event1: ReferentId,
    pub event2: ReferentId,
    pub relation: AllenRelation,
    pub confidence: ConstraintConfidence,
}

/// Confidence level for temporal constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintConfidence {
    /// Derived from explicit temporal markers (before, after, when)
    Explicit,
    /// Inferred from tense/aspect interaction
    TenseInferred,
    /// Default narrative sequence assumption
    NarrativeDefault,
}

/// Temporal reasoning engine
///
/// Maintains a constraint network of temporal relations between events
/// and supports inference of new relations.
#[derive(Debug, Clone)]
pub struct TemporalReasoner {
    /// Stored constraints indexed by event pair
    constraints: HashMap<(ReferentId, ReferentId), (AllenRelation, ConstraintConfidence)>,

    /// Event tense information
    event_tenses: IndexMap<ReferentId, Tense>,

    /// Event aspectual classes
    event_aspects: IndexMap<ReferentId, AspectualClass>,
}

impl TemporalReasoner {
    /// Create a new temporal reasoner
    #[must_use]
    pub fn new() -> Self {
        Self {
            constraints: HashMap::new(),
            event_tenses: IndexMap::new(),
            event_aspects: IndexMap::new(),
        }
    }

    /// Register an event with its tense and aspect
    pub fn register_event(&mut self, event: ReferentId, tense: Tense, aspect: AspectualClass) {
        self.event_tenses.insert(event, tense);
        self.event_aspects.insert(event, aspect);
    }

    /// Add a temporal constraint between two events
    pub fn add_constraint(
        &mut self,
        event1: ReferentId,
        event2: ReferentId,
        relation: AllenRelation,
        confidence: ConstraintConfidence,
    ) {
        // Store both directions for efficient lookup
        self.constraints
            .insert((event1, event2), (relation, confidence));
        self.constraints
            .insert((event2, event1), (relation.inverse(), confidence));
    }

    /// Get the relation between two events, if known
    #[must_use]
    pub fn get_relation(&self, event1: ReferentId, event2: ReferentId) -> Option<AllenRelation> {
        self.constraints.get(&(event1, event2)).map(|(r, _)| *r)
    }

    /// Infer temporal relation from tense/aspect interaction
    ///
    /// Based on Dowty (1986) "The Effects of Aspectual Class on Temporal Structure"
    #[must_use]
    pub fn infer_from_tense_aspect(
        e1_aspect: AspectualClass,
        e1_tense: Tense,
        e2_aspect: AspectualClass,
        e2_tense: Tense,
    ) -> AllenRelation {
        // Past perfect indicates prior completion
        if e1_tense.is_anterior() && !e2_tense.is_anterior() {
            return AllenRelation::Before;
        }
        if e2_tense.is_anterior() && !e1_tense.is_anterior() {
            return AllenRelation::After;
        }

        // States provide background (overlap with other events)
        if e1_aspect == AspectualClass::State && e2_aspect != AspectualClass::State {
            return AllenRelation::Contains; // State contains the event
        }
        if e2_aspect == AspectualClass::State && e1_aspect != AspectualClass::State {
            return AllenRelation::During; // Event during the state
        }

        // Progressive indicates ongoing (overlaps with punctual events)
        if e1_tense.is_progressive() && !e2_tense.is_progressive() {
            if e2_aspect == AspectualClass::Achievement {
                return AllenRelation::Contains;
            }
            return AllenRelation::Overlaps;
        }

        // Achievements are punctual - they "meet" in sequence
        if e1_aspect == AspectualClass::Achievement && e2_aspect == AspectualClass::Achievement {
            return AllenRelation::Meets;
        }

        // Activities and Accomplishments typically sequence
        match (e1_aspect, e2_aspect) {
            (AspectualClass::Activity, AspectualClass::Activity)
            | (AspectualClass::Accomplishment, AspectualClass::Accomplishment) => {
                AllenRelation::Before
            }
            (AspectualClass::Activity, AspectualClass::Accomplishment) => AllenRelation::Before,
            (AspectualClass::Accomplishment, AspectualClass::Activity) => AllenRelation::Before,
            _ => AllenRelation::Before, // Default narrative progression
        }
    }

    /// Infer temporal ordering from narrative tense sequence
    ///
    /// Handles sequences like:
    /// - "John arrived. Mary had left." → left BEFORE arrived
    /// - "John was sleeping. The phone rang." → sleeping CONTAINS rang
    pub fn infer_from_narrative(
        &mut self,
        events: &[(ReferentId, Tense, AspectualClass)],
    ) -> Vec<TemporalConstraint> {
        let mut constraints = Vec::new();

        for window in events.windows(2) {
            let (e1, tense1, aspect1) = window[0];
            let (e2, tense2, aspect2) = window[1];

            let relation = Self::infer_from_tense_aspect(aspect1, tense1, aspect2, tense2);

            // Determine confidence based on tense explicitness
            let confidence = if tense1.is_anterior() || tense2.is_anterior() {
                ConstraintConfidence::TenseInferred
            } else {
                ConstraintConfidence::NarrativeDefault
            };

            self.add_constraint(e1, e2, relation, confidence);

            constraints.push(TemporalConstraint {
                event1: e1,
                event2: e2,
                relation,
                confidence,
            });
        }

        constraints
    }

    /// Check if the temporal constraint network is consistent
    ///
    /// Uses a simplified consistency check - full Allen algebra
    /// constraint propagation is O(n³) and may be added later.
    #[must_use]
    pub fn is_consistent(&self) -> bool {
        // Check for direct contradictions: R(a,b) and R'(a,b) where R ≠ R'
        // (Our data structure prevents this by design)

        // Check transitivity for Before chains
        for (&(e1, e2), &(r1, _)) in &self.constraints {
            if r1 == AllenRelation::Before {
                // If e1 Before e2 and e2 Before e3, then e1 Before e3
                for (&(e2_check, e3), &(r2, _)) in &self.constraints {
                    if e2_check == e2 && r2 == AllenRelation::Before {
                        // Check if e1-e3 constraint exists and is compatible
                        if let Some(&(r13, _)) = self.constraints.get(&(e1, e3)) {
                            if r13 != AllenRelation::Before {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }

    /// Get all events that must occur before a given event
    #[must_use]
    pub fn events_before(&self, event: ReferentId) -> Vec<ReferentId> {
        self.constraints
            .iter()
            .filter_map(|(&(e1, e2), &(rel, _))| {
                if e2 == event && rel.implies_before() {
                    Some(e1)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all events that overlap with a given event
    #[must_use]
    pub fn overlapping_events(&self, event: ReferentId) -> Vec<ReferentId> {
        self.constraints
            .iter()
            .filter_map(|(&(e1, e2), &(rel, _))| {
                if e2 == event && rel.implies_overlap() {
                    Some(e1)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the number of constraints
    #[must_use]
    pub fn constraint_count(&self) -> usize {
        // Divide by 2 because we store both directions
        self.constraints.len() / 2
    }

    /// Clear all constraints
    pub fn clear(&mut self) {
        self.constraints.clear();
        self.event_tenses.clear();
        self.event_aspects.clear();
    }
}

impl Default for TemporalReasoner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allen_relation_inverse() {
        assert_eq!(AllenRelation::Before.inverse(), AllenRelation::After);
        assert_eq!(AllenRelation::Meets.inverse(), AllenRelation::MetBy);
        assert_eq!(
            AllenRelation::Overlaps.inverse(),
            AllenRelation::OverlappedBy
        );
        assert_eq!(AllenRelation::Equals.inverse(), AllenRelation::Equals);
    }

    #[test]
    fn test_past_perfect_ordering() {
        // "John arrived. Mary had left." → left BEFORE arrived
        let relation = TemporalReasoner::infer_from_tense_aspect(
            AspectualClass::Achievement,
            Tense::PastPerfect, // "had left"
            AspectualClass::Achievement,
            Tense::Past, // "arrived"
        );
        assert_eq!(relation, AllenRelation::Before);
    }

    #[test]
    fn test_state_as_background() {
        // "John was sleeping. Mary entered." → sleeping CONTAINS entered
        let relation = TemporalReasoner::infer_from_tense_aspect(
            AspectualClass::State,
            Tense::PastProgressive, // "was sleeping"
            AspectualClass::Achievement,
            Tense::Past, // "entered"
        );
        assert_eq!(relation, AllenRelation::Contains);
    }

    #[test]
    fn test_achievement_sequence() {
        // "John arrived. Mary left." → arrived MEETS left
        let relation = TemporalReasoner::infer_from_tense_aspect(
            AspectualClass::Achievement,
            Tense::Past,
            AspectualClass::Achievement,
            Tense::Past,
        );
        assert_eq!(relation, AllenRelation::Meets);
    }

    #[test]
    fn test_narrative_inference() {
        let mut reasoner = TemporalReasoner::new();

        let e1 = ReferentId(1);
        let e2 = ReferentId(2);
        let e3 = ReferentId(3);

        let events = vec![
            (e1, Tense::Past, AspectualClass::Achievement), // arrived
            (e2, Tense::PastPerfect, AspectualClass::Achievement), // had left
            (e3, Tense::Past, AspectualClass::Activity),    // walked
        ];

        let constraints = reasoner.infer_from_narrative(&events);
        assert_eq!(constraints.len(), 2);

        // e2 (had left) should be before e1 (arrived) due to past perfect
        // But note: our inference is sequential, so e1→e2 is inferred first
        // The narrative order gives us e1 before e2 in the text, but
        // past perfect indicates e2 happened before e1 in reality
    }

    #[test]
    fn test_consistency_check() {
        let mut reasoner = TemporalReasoner::new();

        let e1 = ReferentId(1);
        let e2 = ReferentId(2);
        let e3 = ReferentId(3);

        // e1 before e2, e2 before e3 → should be consistent
        reasoner.add_constraint(
            e1,
            e2,
            AllenRelation::Before,
            ConstraintConfidence::Explicit,
        );
        reasoner.add_constraint(
            e2,
            e3,
            AllenRelation::Before,
            ConstraintConfidence::Explicit,
        );

        assert!(reasoner.is_consistent());
    }

    #[test]
    fn test_events_before() {
        let mut reasoner = TemporalReasoner::new();

        let e1 = ReferentId(1);
        let e2 = ReferentId(2);
        let e3 = ReferentId(3);

        reasoner.add_constraint(
            e1,
            e3,
            AllenRelation::Before,
            ConstraintConfidence::Explicit,
        );
        reasoner.add_constraint(e2, e3, AllenRelation::Meets, ConstraintConfidence::Explicit);

        let before_e3 = reasoner.events_before(e3);
        assert_eq!(before_e3.len(), 2);
        assert!(before_e3.contains(&e1));
        assert!(before_e3.contains(&e2));
    }

    #[test]
    fn test_drs_conversion() {
        assert_eq!(
            AllenRelation::Before.to_drs_relation(),
            TemporalRelationType::Before
        );
        assert_eq!(
            AllenRelation::Meets.to_drs_relation(),
            TemporalRelationType::Meets
        );
        assert_eq!(
            AllenRelation::Contains.to_drs_relation(),
            TemporalRelationType::Contains
        );
    }
}
