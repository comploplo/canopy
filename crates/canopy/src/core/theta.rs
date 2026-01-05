//! Thematic roles for argument structure.
//!
//! Based on `VerbNet` and `FrameNet` role inventories.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Thematic roles for semantic argument structure.
///
/// These roles capture the semantic relationship between a predicate
/// and its arguments, following linguistic theory (Dowty, Levin, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThetaRole {
    /// The initiator/doer of an action: "John broke the vase"
    Agent,
    /// The affected entity: "John broke the vase"
    Patient,
    /// The entity undergoing change or motion: "John gave Mary a book"
    Theme,
    /// The entity experiencing a mental state: "John fears spiders"
    Experiencer,
    /// The entity receiving something: "John gave Mary a book"
    Recipient,
    /// The entity benefiting: "John baked Mary a cake"
    Benefactive,
    /// The means used: "John opened the door with a key"
    Instrument,
    /// An accompanying entity: "John walked with Mary"
    Comitative,
    /// Where something is: "John put the book on the table"
    Location,
    /// Starting point: "John went from Boston"
    Source,
    /// Ending point: "John went to Boston"
    Goal,
    /// Path of motion: "John walked through the park"
    Direction,
    /// Temporal location: "John arrived at noon"
    Temporal,
    /// How often: "John visits weekly"
    Frequency,
    /// Quantity/extent: "John ran five miles"
    Measure,
    /// Reason/cause: "John left because of the noise"
    Cause,
    /// How something is done: "John spoke loudly"
    Manner,
    /// Controlled subject in raising/control: "John tried to leave"
    ControlledSubject,
    /// Trigger for experiencer: "Spiders frighten John"
    Stimulus,
}

impl ThetaRole {
    /// Returns all theta roles.
    #[must_use]
    pub const fn all() -> &'static [ThetaRole] {
        &[
            ThetaRole::Agent,
            ThetaRole::Patient,
            ThetaRole::Theme,
            ThetaRole::Experiencer,
            ThetaRole::Recipient,
            ThetaRole::Benefactive,
            ThetaRole::Instrument,
            ThetaRole::Comitative,
            ThetaRole::Location,
            ThetaRole::Source,
            ThetaRole::Goal,
            ThetaRole::Direction,
            ThetaRole::Temporal,
            ThetaRole::Frequency,
            ThetaRole::Measure,
            ThetaRole::Cause,
            ThetaRole::Manner,
            ThetaRole::ControlledSubject,
            ThetaRole::Stimulus,
        ]
    }

    /// Check if this is a core argument role (Agent, Patient, Theme, Experiencer, Recipient).
    #[must_use]
    pub const fn is_core_argument(&self) -> bool {
        matches!(
            self,
            ThetaRole::Agent
                | ThetaRole::Patient
                | ThetaRole::Theme
                | ThetaRole::Experiencer
                | ThetaRole::Recipient
        )
    }

    /// Parse from `VerbNet`/`FrameNet` role name.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "Agent" | "Actor" | "Actor1" | "Donor" | "Giver" | "Speaker" => Some(ThetaRole::Agent),
            "Patient" => Some(ThetaRole::Patient),
            "Theme" | "Topic" | "Content" | "Message" | "Goods" => Some(ThetaRole::Theme),
            "Experiencer" | "Cognizer" | "Perceiver" => Some(ThetaRole::Experiencer),
            "Recipient" | "Addressee" | "Audience" => Some(ThetaRole::Recipient),
            "Beneficiary" | "Benefactive" => Some(ThetaRole::Benefactive),
            "Instrument" | "Means" => Some(ThetaRole::Instrument),
            "Comitative" | "Co-Agent" => Some(ThetaRole::Comitative),
            "Location" | "Place" | "Ground" => Some(ThetaRole::Location),
            "Source" | "Origin" => Some(ThetaRole::Source),
            "Goal" | "Destination" | "Target" => Some(ThetaRole::Goal),
            "Direction" | "Path" | "Trajectory" => Some(ThetaRole::Direction),
            "Temporal" | "Time" | "Duration" => Some(ThetaRole::Temporal),
            "Frequency" => Some(ThetaRole::Frequency),
            "Measure" | "Extent" | "Value" | "Amount" => Some(ThetaRole::Measure),
            "Cause" | "Reason" | "Purpose" => Some(ThetaRole::Cause),
            "Manner" => Some(ThetaRole::Manner),
            "Pivot" | "Co-Theme" => Some(ThetaRole::ControlledSubject),
            "Stimulus" => Some(ThetaRole::Stimulus),
            _ => None,
        }
    }
}

impl fmt::Display for ThetaRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_roles() {
        assert_eq!(ThetaRole::all().len(), 19);
    }

    #[test]
    fn test_core_arguments() {
        assert!(ThetaRole::Agent.is_core_argument());
        assert!(ThetaRole::Patient.is_core_argument());
        assert!(ThetaRole::Theme.is_core_argument());
        assert!(ThetaRole::Experiencer.is_core_argument());
        assert!(ThetaRole::Recipient.is_core_argument());
        assert!(!ThetaRole::Instrument.is_core_argument());
        assert!(!ThetaRole::Location.is_core_argument());
        assert!(!ThetaRole::Benefactive.is_core_argument());
        assert!(!ThetaRole::Comitative.is_core_argument());
        assert!(!ThetaRole::Source.is_core_argument());
        assert!(!ThetaRole::Goal.is_core_argument());
        assert!(!ThetaRole::Direction.is_core_argument());
        assert!(!ThetaRole::Temporal.is_core_argument());
        assert!(!ThetaRole::Frequency.is_core_argument());
        assert!(!ThetaRole::Measure.is_core_argument());
        assert!(!ThetaRole::Cause.is_core_argument());
        assert!(!ThetaRole::Manner.is_core_argument());
        assert!(!ThetaRole::ControlledSubject.is_core_argument());
        assert!(!ThetaRole::Stimulus.is_core_argument());
    }

    #[test]
    fn test_from_str_agent_variants() {
        assert_eq!(ThetaRole::parse("Agent"), Some(ThetaRole::Agent));
        assert_eq!(ThetaRole::parse("Actor"), Some(ThetaRole::Agent));
        assert_eq!(ThetaRole::parse("Actor1"), Some(ThetaRole::Agent));
        assert_eq!(ThetaRole::parse("Donor"), Some(ThetaRole::Agent));
        assert_eq!(ThetaRole::parse("Giver"), Some(ThetaRole::Agent));
        assert_eq!(ThetaRole::parse("Speaker"), Some(ThetaRole::Agent));
    }

    #[test]
    fn test_from_str_patient() {
        assert_eq!(ThetaRole::parse("Patient"), Some(ThetaRole::Patient));
    }

    #[test]
    fn test_from_str_theme_variants() {
        assert_eq!(ThetaRole::parse("Theme"), Some(ThetaRole::Theme));
        assert_eq!(ThetaRole::parse("Topic"), Some(ThetaRole::Theme));
        assert_eq!(ThetaRole::parse("Content"), Some(ThetaRole::Theme));
        assert_eq!(ThetaRole::parse("Message"), Some(ThetaRole::Theme));
        assert_eq!(ThetaRole::parse("Goods"), Some(ThetaRole::Theme));
    }

    #[test]
    fn test_from_str_experiencer_variants() {
        assert_eq!(
            ThetaRole::parse("Experiencer"),
            Some(ThetaRole::Experiencer)
        );
        assert_eq!(ThetaRole::parse("Cognizer"), Some(ThetaRole::Experiencer));
        assert_eq!(ThetaRole::parse("Perceiver"), Some(ThetaRole::Experiencer));
    }

    #[test]
    fn test_from_str_recipient_variants() {
        assert_eq!(ThetaRole::parse("Recipient"), Some(ThetaRole::Recipient));
        assert_eq!(ThetaRole::parse("Addressee"), Some(ThetaRole::Recipient));
        assert_eq!(ThetaRole::parse("Audience"), Some(ThetaRole::Recipient));
    }

    #[test]
    fn test_from_str_benefactive_variants() {
        assert_eq!(
            ThetaRole::parse("Beneficiary"),
            Some(ThetaRole::Benefactive)
        );
        assert_eq!(
            ThetaRole::parse("Benefactive"),
            Some(ThetaRole::Benefactive)
        );
    }

    #[test]
    fn test_from_str_instrument_variants() {
        assert_eq!(ThetaRole::parse("Instrument"), Some(ThetaRole::Instrument));
        assert_eq!(ThetaRole::parse("Means"), Some(ThetaRole::Instrument));
    }

    #[test]
    fn test_from_str_comitative_variants() {
        assert_eq!(ThetaRole::parse("Comitative"), Some(ThetaRole::Comitative));
        assert_eq!(ThetaRole::parse("Co-Agent"), Some(ThetaRole::Comitative));
    }

    #[test]
    fn test_from_str_location_variants() {
        assert_eq!(ThetaRole::parse("Location"), Some(ThetaRole::Location));
        assert_eq!(ThetaRole::parse("Place"), Some(ThetaRole::Location));
        assert_eq!(ThetaRole::parse("Ground"), Some(ThetaRole::Location));
    }

    #[test]
    fn test_from_str_source_variants() {
        assert_eq!(ThetaRole::parse("Source"), Some(ThetaRole::Source));
        assert_eq!(ThetaRole::parse("Origin"), Some(ThetaRole::Source));
    }

    #[test]
    fn test_from_str_goal_variants() {
        assert_eq!(ThetaRole::parse("Goal"), Some(ThetaRole::Goal));
        assert_eq!(ThetaRole::parse("Destination"), Some(ThetaRole::Goal));
        assert_eq!(ThetaRole::parse("Target"), Some(ThetaRole::Goal));
    }

    #[test]
    fn test_from_str_direction_variants() {
        assert_eq!(ThetaRole::parse("Direction"), Some(ThetaRole::Direction));
        assert_eq!(ThetaRole::parse("Path"), Some(ThetaRole::Direction));
        assert_eq!(ThetaRole::parse("Trajectory"), Some(ThetaRole::Direction));
    }

    #[test]
    fn test_from_str_temporal_variants() {
        assert_eq!(ThetaRole::parse("Temporal"), Some(ThetaRole::Temporal));
        assert_eq!(ThetaRole::parse("Time"), Some(ThetaRole::Temporal));
        assert_eq!(ThetaRole::parse("Duration"), Some(ThetaRole::Temporal));
    }

    #[test]
    fn test_from_str_frequency() {
        assert_eq!(ThetaRole::parse("Frequency"), Some(ThetaRole::Frequency));
    }

    #[test]
    fn test_from_str_measure_variants() {
        assert_eq!(ThetaRole::parse("Measure"), Some(ThetaRole::Measure));
        assert_eq!(ThetaRole::parse("Extent"), Some(ThetaRole::Measure));
        assert_eq!(ThetaRole::parse("Value"), Some(ThetaRole::Measure));
        assert_eq!(ThetaRole::parse("Amount"), Some(ThetaRole::Measure));
    }

    #[test]
    fn test_from_str_cause_variants() {
        assert_eq!(ThetaRole::parse("Cause"), Some(ThetaRole::Cause));
        assert_eq!(ThetaRole::parse("Reason"), Some(ThetaRole::Cause));
        assert_eq!(ThetaRole::parse("Purpose"), Some(ThetaRole::Cause));
    }

    #[test]
    fn test_from_str_manner() {
        assert_eq!(ThetaRole::parse("Manner"), Some(ThetaRole::Manner));
    }

    #[test]
    fn test_from_str_controlled_subject_variants() {
        assert_eq!(
            ThetaRole::parse("Pivot"),
            Some(ThetaRole::ControlledSubject)
        );
        assert_eq!(
            ThetaRole::parse("Co-Theme"),
            Some(ThetaRole::ControlledSubject)
        );
    }

    #[test]
    fn test_from_str_stimulus() {
        assert_eq!(ThetaRole::parse("Stimulus"), Some(ThetaRole::Stimulus));
    }

    #[test]
    fn test_from_str_unknown() {
        assert_eq!(ThetaRole::parse("Unknown"), None);
        assert_eq!(ThetaRole::parse(""), None);
        assert_eq!(ThetaRole::parse("NotARole"), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ThetaRole::Agent), "Agent");
        assert_eq!(format!("{}", ThetaRole::Patient), "Patient");
        assert_eq!(format!("{}", ThetaRole::Theme), "Theme");
        assert_eq!(format!("{}", ThetaRole::Experiencer), "Experiencer");
        assert_eq!(format!("{}", ThetaRole::Recipient), "Recipient");
        assert_eq!(format!("{}", ThetaRole::Benefactive), "Benefactive");
        assert_eq!(format!("{}", ThetaRole::Instrument), "Instrument");
        assert_eq!(format!("{}", ThetaRole::Comitative), "Comitative");
        assert_eq!(format!("{}", ThetaRole::Location), "Location");
        assert_eq!(format!("{}", ThetaRole::Source), "Source");
        assert_eq!(format!("{}", ThetaRole::Goal), "Goal");
        assert_eq!(format!("{}", ThetaRole::Direction), "Direction");
        assert_eq!(format!("{}", ThetaRole::Temporal), "Temporal");
        assert_eq!(format!("{}", ThetaRole::Frequency), "Frequency");
        assert_eq!(format!("{}", ThetaRole::Measure), "Measure");
        assert_eq!(format!("{}", ThetaRole::Cause), "Cause");
        assert_eq!(format!("{}", ThetaRole::Manner), "Manner");
        assert_eq!(
            format!("{}", ThetaRole::ControlledSubject),
            "ControlledSubject"
        );
        assert_eq!(format!("{}", ThetaRole::Stimulus), "Stimulus");
    }
}
