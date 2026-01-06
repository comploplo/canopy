//! TAM-to-DRS builder: converts TAM features into DRS conditions.
//!
//! Takes temporal frames and aspectual viewpoints from composed events
//! and generates appropriate DRS conditions for temporal/aspectual semantics.

use super::drs::{Drs, DrsCondition};
use super::referent::ReferentId;
use super::temporal::{
    AspectualOperator, AspectualViewpoint, TemporalAnchorType, TemporalFrame, TimePoint,
};

/// Builder for TAM-related DRS conditions.
#[derive(Debug, Default)]
pub struct TamBuilder {
    /// Counter for generating underspecified time points.
    time_counter: u32,
}

impl TamBuilder {
    /// Create a new TAM builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build DRS conditions for an event's temporal frame.
    ///
    /// Creates a `TemporalFrameAssignment` condition that records
    /// the Reichenbachian temporal structure (S, R, E) for the event.
    #[must_use]
    pub fn build_temporal_frame_condition(
        &self,
        event_ref: ReferentId,
        frame: &TemporalFrame,
    ) -> DrsCondition {
        DrsCondition::TemporalFrameAssignment {
            event: event_ref,
            frame: frame.clone(),
        }
    }

    /// Build DRS conditions for aspectual viewpoint.
    ///
    /// Converts the aspectual viewpoint into an `AspectualOp` condition
    /// that wraps the event in the appropriate aspectual operator.
    #[must_use]
    pub fn build_aspectual_condition(
        &self,
        event_ref: ReferentId,
        viewpoint: AspectualViewpoint,
        event_drs: Drs,
    ) -> Option<DrsCondition> {
        let operator = match viewpoint {
            AspectualViewpoint::Progressive => AspectualOperator::Progressive,
            AspectualViewpoint::Perfect => AspectualOperator::Perfect,
            AspectualViewpoint::Habitual => AspectualOperator::Habitual,
            // Perfective and Imperfective don't create explicit operators
            AspectualViewpoint::Perfective | AspectualViewpoint::Imperfective => return None,
            // Prospective maps to inchoative
            AspectualViewpoint::Prospective => AspectualOperator::Inchoative,
        };

        Some(DrsCondition::AspectualOp {
            operator,
            event: event_ref,
            scope: Box::new(event_drs),
        })
    }

    /// Build a temporal anchor condition.
    ///
    /// Anchors an event to a specific time point (speech time, another event, etc.).
    #[must_use]
    pub fn build_temporal_anchor(
        &self,
        event_ref: ReferentId,
        anchor_type: TemporalAnchorType,
        reference: TimePoint,
    ) -> DrsCondition {
        DrsCondition::TemporalAnchor {
            event: event_ref,
            anchor_type,
            reference,
        }
    }

    /// Generate a fresh underspecified time point.
    pub fn fresh_time_point(&mut self) -> TimePoint {
        let id = self.time_counter;
        self.time_counter += 1;
        TimePoint::Underspecified(id)
    }

    /// Build all TAM conditions for an event.
    ///
    /// Returns a vector of conditions representing the full TAM structure.
    #[must_use]
    pub fn build_tam_conditions(
        &self,
        event_ref: ReferentId,
        temporal_frame: Option<&TemporalFrame>,
        aspectual_viewpoint: Option<AspectualViewpoint>,
        event_drs: Option<Drs>,
    ) -> Vec<DrsCondition> {
        let mut conditions = Vec::new();

        // Add temporal frame if present
        if let Some(frame) = temporal_frame {
            conditions.push(self.build_temporal_frame_condition(event_ref, frame));
        }

        // Add aspectual operator if applicable
        if let (Some(viewpoint), Some(drs)) = (aspectual_viewpoint, event_drs) {
            if let Some(aspectual_cond) = self.build_aspectual_condition(event_ref, viewpoint, drs)
            {
                conditions.push(aspectual_cond);
            }
        }

        conditions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::discourse::temporal::TemporalFrame;

    #[test]
    fn test_build_temporal_frame_condition() {
        let builder = TamBuilder::new();
        let event_ref = ReferentId::new(0);
        let frame = TemporalFrame::past();

        let cond = builder.build_temporal_frame_condition(event_ref, &frame);

        match cond {
            DrsCondition::TemporalFrameAssignment { event, frame: f } => {
                assert_eq!(event, event_ref);
                assert!(f.is_simple_past());
            }
            _ => panic!("Expected TemporalFrameAssignment"),
        }
    }

    #[test]
    fn test_build_progressive_aspectual_condition() {
        let builder = TamBuilder::new();
        let event_ref = ReferentId::new(0);
        let drs = Drs::default();

        let cond =
            builder.build_aspectual_condition(event_ref, AspectualViewpoint::Progressive, drs);

        assert!(cond.is_some());
        match cond.unwrap() {
            DrsCondition::AspectualOp {
                operator, event, ..
            } => {
                assert_eq!(operator, AspectualOperator::Progressive);
                assert_eq!(event, event_ref);
            }
            _ => panic!("Expected AspectualOp"),
        }
    }

    #[test]
    fn test_perfective_no_operator() {
        let builder = TamBuilder::new();
        let event_ref = ReferentId::new(0);
        let drs = Drs::default();

        // Perfective should not create an explicit operator
        let cond =
            builder.build_aspectual_condition(event_ref, AspectualViewpoint::Perfective, drs);
        assert!(cond.is_none());
    }

    #[test]
    fn test_fresh_time_point() {
        let mut builder = TamBuilder::new();

        let t1 = builder.fresh_time_point();
        let t2 = builder.fresh_time_point();

        match (t1, t2) {
            (TimePoint::Underspecified(id1), TimePoint::Underspecified(id2)) => {
                assert_ne!(id1, id2);
                assert_eq!(id1 + 1, id2);
            }
            _ => panic!("Expected Underspecified time points"),
        }
    }

    #[test]
    fn test_build_tam_conditions() {
        let builder = TamBuilder::new();
        let event_ref = ReferentId::new(0);
        let frame = TemporalFrame::past_progressive();
        let drs = Drs::default();

        let conditions = builder.build_tam_conditions(
            event_ref,
            Some(&frame),
            Some(AspectualViewpoint::Progressive),
            Some(drs),
        );

        // Should have both temporal frame and aspectual operator
        assert_eq!(conditions.len(), 2);
    }

    #[test]
    fn test_build_temporal_anchor() {
        let builder = TamBuilder::new();
        let event_ref = ReferentId::new(0);
        let reference = TimePoint::Now;

        let cond = builder.build_temporal_anchor(event_ref, TemporalAnchorType::Before, reference);

        match cond {
            DrsCondition::TemporalAnchor {
                event, anchor_type, ..
            } => {
                assert_eq!(event, event_ref);
                assert_eq!(anchor_type, TemporalAnchorType::Before);
            }
            _ => panic!("Expected TemporalAnchor"),
        }
    }
}
