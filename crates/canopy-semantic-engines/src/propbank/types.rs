//! Type definitions for PropBank semantic role labeling
//!
//! This module defines the core types used for PropBank predicate-argument structures,
//! including semantic roles, arguments, and analysis results.

use canopy_core::ThetaRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semantic roles in PropBank annotation scheme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SemanticRole {
    /// ARG0: Agent (prototypical agent of verb)
    Agent,
    /// ARG1: Patient/Theme (prototypical patient)
    Patient,
    /// ARG2: Indirect object, instrument, beneficiary, attribute
    IndirectObject,
    /// ARG3: Starting point, beneficiary, attribute
    StartingPoint,
    /// ARG4: Ending point
    EndingPoint,
    /// ARG5: Additional argument (rare)
    Additional,
    /// ARGM-*: Modifier roles
    Modifier(ArgumentModifier),
    /// Continuation argument (C-ARG*)
    Continuation(Box<SemanticRole>),
    /// Reference argument (R-ARG*)
    Reference(Box<SemanticRole>),
}

impl SemanticRole {
    /// Parse semantic role from PropBank annotation string
    pub fn from_propbank_label(label: &str) -> Self {
        match label {
            "ARG0" => Self::Agent,
            "ARG1" => Self::Patient,
            "ARG2" => Self::IndirectObject,
            "ARG3" => Self::StartingPoint,
            "ARG4" => Self::EndingPoint,
            "ARG5" => Self::Additional,
            label if label.starts_with("ARGM-") => {
                let modifier_type = &label[5..]; // Remove "ARGM-" prefix
                Self::Modifier(ArgumentModifier::from_propbank_label(modifier_type))
            }
            label if label.starts_with("C-ARG") => {
                let base_label = &label[2..]; // Remove "C-" prefix
                Self::Continuation(Box::new(Self::from_propbank_label(base_label)))
            }
            label if label.starts_with("R-ARG") => {
                let base_label = &label[2..]; // Remove "R-" prefix
                Self::Reference(Box::new(Self::from_propbank_label(base_label)))
            }
            _ => Self::Modifier(ArgumentModifier::Other(label.to_string())),
        }
    }

    /// Convert to PropBank annotation string
    pub fn to_propbank_label(&self) -> String {
        match self {
            Self::Agent => "ARG0".to_string(),
            Self::Patient => "ARG1".to_string(),
            Self::IndirectObject => "ARG2".to_string(),
            Self::StartingPoint => "ARG3".to_string(),
            Self::EndingPoint => "ARG4".to_string(),
            Self::Additional => "ARG5".to_string(),
            Self::Modifier(modifier) => format!("ARGM-{modifier}"),
            Self::Continuation(role) => format!("C-{}", role.to_propbank_label()),
            Self::Reference(role) => format!("R-{}", role.to_propbank_label()),
        }
    }

    /// Get canonical theta role mapping for compatibility with other engines
    pub fn to_theta_role(&self) -> Option<ThetaRole> {
        match self {
            Self::Agent => Some(ThetaRole::Agent),
            Self::Patient => Some(ThetaRole::Patient),
            Self::IndirectObject => Some(ThetaRole::Recipient),
            Self::StartingPoint => Some(ThetaRole::Source),
            Self::EndingPoint => Some(ThetaRole::Goal),
            Self::Modifier(ArgumentModifier::Location) => Some(ThetaRole::Location),
            Self::Modifier(ArgumentModifier::Time) => Some(ThetaRole::Temporal),
            Self::Modifier(ArgumentModifier::Manner) => Some(ThetaRole::Manner),
            Self::Modifier(ArgumentModifier::Cause) => Some(ThetaRole::Cause),
            _ => None,
        }
    }
}

/// Argument modifiers (ARGM-* roles)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ArgumentModifier {
    /// ARGM-LOC: Location
    Location,
    /// ARGM-TMP: Temporal
    Time,
    /// ARGM-MNR: Manner
    Manner,
    /// ARGM-CAU: Cause
    Cause,
    /// ARGM-PRP: Purpose
    Purpose,
    /// ARGM-DIR: Direction
    Direction,
    /// ARGM-EXT: Extent
    Extent,
    /// ARGM-REC: Reciprocal
    Reciprocal,
    /// ARGM-PRD: Predicate
    Predicate,
    /// ARGM-MOD: Modal
    Modal,
    /// ARGM-NEG: Negation
    Negation,
    /// ARGM-DIS: Discourse
    Discourse,
    /// ARGM-ADV: Adverbial
    Adverbial,
    /// ARGM-LVB: Light verb
    LightVerb,
    /// Other modifier type
    Other(String),
}

impl std::str::FromStr for ArgumentModifier {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "LOC" => Self::Location,
            "TMP" => Self::Time,
            "MNR" => Self::Manner,
            "CAU" => Self::Cause,
            "PRP" => Self::Purpose,
            "DIR" => Self::Direction,
            "EXT" => Self::Extent,
            "REC" => Self::Reciprocal,
            "PRD" => Self::Predicate,
            "MOD" => Self::Modal,
            "NEG" => Self::Negation,
            "DIS" => Self::Discourse,
            "ADV" => Self::Adverbial,
            "LVB" => Self::LightVerb,
            other => Self::Other(other.to_string()),
        })
    }
}

impl ArgumentModifier {
    /// Parse argument modifier from string (convenience method)
    pub fn from_propbank_label(s: &str) -> Self {
        s.parse().unwrap_or_else(|_| Self::Other(s.to_string()))
    }
}

impl std::fmt::Display for ArgumentModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Location => "LOC",
            Self::Time => "TMP",
            Self::Manner => "MNR",
            Self::Cause => "CAU",
            Self::Purpose => "PRP",
            Self::Direction => "DIR",
            Self::Extent => "EXT",
            Self::Reciprocal => "REC",
            Self::Predicate => "PRD",
            Self::Modal => "MOD",
            Self::Negation => "NEG",
            Self::Discourse => "DIS",
            Self::Adverbial => "ADV",
            Self::LightVerb => "LVB",
            Self::Other(s) => s,
        };
        write!(f, "{s}")
    }
}

/// A PropBank argument with its semantic role and position information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropBankArgument {
    /// Semantic role of this argument
    pub role: SemanticRole,
    /// Description or example of the argument
    pub description: String,
    /// Token indices where this argument appears (if available)
    pub token_span: Option<(usize, usize)>,
    /// Confidence score for this argument assignment
    pub confidence: f32,
}

impl PropBankArgument {
    /// Create a new PropBank argument
    pub fn new(role: SemanticRole, description: String, confidence: f32) -> Self {
        Self {
            role,
            description,
            token_span: None,
            confidence,
        }
    }

    /// Create argument with token span information
    pub fn with_span(
        role: SemanticRole,
        description: String,
        span: (usize, usize),
        confidence: f32,
    ) -> Self {
        Self {
            role,
            description,
            token_span: Some(span),
            confidence,
        }
    }

    /// Check if this is a core argument (ARG0-ARG5)
    pub fn is_core_argument(&self) -> bool {
        matches!(
            self.role,
            SemanticRole::Agent
                | SemanticRole::Patient
                | SemanticRole::IndirectObject
                | SemanticRole::StartingPoint
                | SemanticRole::EndingPoint
                | SemanticRole::Additional
        )
    }

    /// Check if this is a modifier argument (ARGM-*)
    pub fn is_modifier(&self) -> bool {
        matches!(self.role, SemanticRole::Modifier(_))
    }
}

/// A PropBank predicate with its associated arguments
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropBankPredicate {
    /// The predicate lemma (e.g., "give", "take", "run")
    pub lemma: String,
    /// The sense number (e.g., "01", "02")
    pub sense: String,
    /// Full roleset identifier (e.g., "give.01")
    pub roleset: String,
    /// Arguments associated with this predicate
    pub arguments: Vec<PropBankArgument>,
    /// Predicate definition/description
    pub definition: String,
    /// Token position of the predicate (if available)
    pub predicate_span: Option<usize>,
}

impl PropBankPredicate {
    /// Create a new PropBank predicate
    pub fn new(lemma: String, sense: String, definition: String) -> Self {
        let roleset = format!("{lemma}.{sense}");
        Self {
            lemma,
            sense,
            roleset,
            arguments: Vec::new(),
            definition,
            predicate_span: None,
        }
    }

    /// Add an argument to this predicate
    pub fn add_argument(&mut self, argument: PropBankArgument) {
        self.arguments.push(argument);
    }

    /// Get arguments with a specific semantic role
    pub fn get_arguments_by_role(&self, role: &SemanticRole) -> Vec<&PropBankArgument> {
        self.arguments
            .iter()
            .filter(|arg| &arg.role == role)
            .collect()
    }

    /// Get all core arguments (ARG0-ARG5)
    pub fn get_core_arguments(&self) -> Vec<&PropBankArgument> {
        self.arguments
            .iter()
            .filter(|arg| arg.is_core_argument())
            .collect()
    }

    /// Get all modifier arguments (ARGM-*)
    pub fn get_modifiers(&self) -> Vec<&PropBankArgument> {
        self.arguments
            .iter()
            .filter(|arg| arg.is_modifier())
            .collect()
    }

    /// Check if predicate has a specific argument role
    pub fn has_role(&self, role: &SemanticRole) -> bool {
        self.arguments.iter().any(|arg| &arg.role == role)
    }
}

/// PropBank frameset containing multiple predicates/rolesets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropBankFrameset {
    /// Base predicate lemma
    pub lemma: String,
    /// All rolesets for this lemma
    pub rolesets: HashMap<String, PropBankPredicate>,
    /// Frameset-level notes or examples
    pub notes: String,
}

impl PropBankFrameset {
    /// Create a new PropBank frameset
    pub fn new(lemma: String, notes: String) -> Self {
        Self {
            lemma,
            rolesets: HashMap::new(),
            notes,
        }
    }

    /// Add a roleset to this frameset
    pub fn add_roleset(&mut self, predicate: PropBankPredicate) {
        self.rolesets.insert(predicate.sense.clone(), predicate);
    }

    /// Get a specific roleset by sense
    pub fn get_roleset(&self, sense: &str) -> Option<&PropBankPredicate> {
        self.rolesets.get(sense)
    }

    /// Get all rolesets for this frameset
    pub fn get_all_rolesets(&self) -> Vec<&PropBankPredicate> {
        self.rolesets.values().collect()
    }
}

/// PropBank analysis result for a word or predicate
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropBankAnalysis {
    /// Input word or predicate analyzed
    pub input: String,
    /// Matching PropBank predicate (if found)
    pub predicate: Option<PropBankPredicate>,
    /// Alternative rolesets that might match
    pub alternative_rolesets: Vec<PropBankPredicate>,
    /// Overall confidence score
    pub confidence: f32,
    /// Number of arguments found
    pub argument_count: usize,
    /// All theta roles represented
    pub theta_roles: Vec<ThetaRole>,
}

impl PropBankAnalysis {
    /// Create a new PropBank analysis
    pub fn new(input: String) -> Self {
        Self {
            input,
            predicate: None,
            alternative_rolesets: Vec::new(),
            confidence: 0.0,
            argument_count: 0,
            theta_roles: Vec::new(),
        }
    }

    /// Create analysis with a matching predicate
    pub fn with_predicate(input: String, predicate: PropBankPredicate, confidence: f32) -> Self {
        let argument_count = predicate.arguments.len();
        let theta_roles = predicate
            .arguments
            .iter()
            .filter_map(|arg| arg.role.to_theta_role())
            .collect();

        Self {
            input,
            predicate: Some(predicate),
            alternative_rolesets: Vec::new(),
            confidence,
            argument_count,
            theta_roles,
        }
    }

    /// Add an alternative roleset
    pub fn add_alternative(&mut self, predicate: PropBankPredicate) {
        self.alternative_rolesets.push(predicate);
    }

    /// Check if analysis found any matching predicates
    pub fn has_match(&self) -> bool {
        self.predicate.is_some() || !self.alternative_rolesets.is_empty()
    }

    /// Get all predicates (primary + alternatives)
    pub fn all_predicates(&self) -> Vec<&PropBankPredicate> {
        let mut predicates = Vec::new();
        if let Some(ref pred) = self.predicate {
            predicates.push(pred);
        }
        predicates.extend(self.alternative_rolesets.iter());
        predicates
    }

    /// Get the most likely predicate (highest confidence)
    pub fn best_predicate(&self) -> Option<&PropBankPredicate> {
        self.predicate.as_ref()
    }

    /// Update confidence based on multiple matches
    pub fn calculate_confidence(&mut self) {
        if let Some(ref pred) = self.predicate {
            // Base confidence on number of core arguments
            let core_args = pred.get_core_arguments().len() as f32;
            let modifier_args = pred.get_modifiers().len() as f32;

            // More core arguments = higher confidence
            let base_confidence = (core_args * 0.3 + modifier_args * 0.1).min(1.0);

            // Boost confidence if we have alternatives (indicates rich coverage)
            let alternative_boost = if !self.alternative_rolesets.is_empty() {
                0.1
            } else {
                0.0
            };

            self.confidence = (base_confidence + alternative_boost).min(0.95);
        } else if !self.alternative_rolesets.is_empty() {
            // Only alternatives, lower confidence
            self.confidence = 0.4;
        } else {
            self.confidence = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === SemanticRole Tests ===

    #[test]
    fn test_semantic_role_from_propbank_label() {
        assert_eq!(
            SemanticRole::from_propbank_label("ARG0"),
            SemanticRole::Agent
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARG1"),
            SemanticRole::Patient
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARG2"),
            SemanticRole::IndirectObject
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARG3"),
            SemanticRole::StartingPoint
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARG4"),
            SemanticRole::EndingPoint
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARG5"),
            SemanticRole::Additional
        );
    }

    #[test]
    fn test_semantic_role_from_propbank_label_modifiers() {
        assert_eq!(
            SemanticRole::from_propbank_label("ARGM-LOC"),
            SemanticRole::Modifier(ArgumentModifier::Location)
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARGM-TMP"),
            SemanticRole::Modifier(ArgumentModifier::Time)
        );
        assert_eq!(
            SemanticRole::from_propbank_label("ARGM-NEG"),
            SemanticRole::Modifier(ArgumentModifier::Negation)
        );
    }

    #[test]
    fn test_semantic_role_from_propbank_label_continuation() {
        let role = SemanticRole::from_propbank_label("C-ARG0");
        assert!(matches!(role, SemanticRole::Continuation(_)));
        if let SemanticRole::Continuation(inner) = role {
            assert_eq!(*inner, SemanticRole::Agent);
        }
    }

    #[test]
    fn test_semantic_role_from_propbank_label_reference() {
        let role = SemanticRole::from_propbank_label("R-ARG1");
        assert!(matches!(role, SemanticRole::Reference(_)));
        if let SemanticRole::Reference(inner) = role {
            assert_eq!(*inner, SemanticRole::Patient);
        }
    }

    #[test]
    fn test_semantic_role_to_propbank_label() {
        assert_eq!(SemanticRole::Agent.to_propbank_label(), "ARG0");
        assert_eq!(SemanticRole::Patient.to_propbank_label(), "ARG1");
        assert_eq!(SemanticRole::IndirectObject.to_propbank_label(), "ARG2");
        assert_eq!(SemanticRole::StartingPoint.to_propbank_label(), "ARG3");
        assert_eq!(SemanticRole::EndingPoint.to_propbank_label(), "ARG4");
        assert_eq!(SemanticRole::Additional.to_propbank_label(), "ARG5");
    }

    #[test]
    fn test_semantic_role_to_propbank_label_modifiers() {
        assert_eq!(
            SemanticRole::Modifier(ArgumentModifier::Location).to_propbank_label(),
            "ARGM-LOC"
        );
        assert_eq!(
            SemanticRole::Modifier(ArgumentModifier::Time).to_propbank_label(),
            "ARGM-TMP"
        );
    }

    #[test]
    fn test_semantic_role_to_propbank_label_continuation() {
        let role = SemanticRole::Continuation(Box::new(SemanticRole::Agent));
        assert_eq!(role.to_propbank_label(), "C-ARG0");
    }

    #[test]
    fn test_semantic_role_to_theta_role() {
        assert_eq!(SemanticRole::Agent.to_theta_role(), Some(ThetaRole::Agent));
        assert_eq!(
            SemanticRole::Patient.to_theta_role(),
            Some(ThetaRole::Patient)
        );
        assert_eq!(
            SemanticRole::IndirectObject.to_theta_role(),
            Some(ThetaRole::Recipient)
        );
        assert_eq!(
            SemanticRole::StartingPoint.to_theta_role(),
            Some(ThetaRole::Source)
        );
        assert_eq!(
            SemanticRole::EndingPoint.to_theta_role(),
            Some(ThetaRole::Goal)
        );

        // Modifier to theta role
        assert_eq!(
            SemanticRole::Modifier(ArgumentModifier::Location).to_theta_role(),
            Some(ThetaRole::Location)
        );
        assert_eq!(
            SemanticRole::Modifier(ArgumentModifier::Time).to_theta_role(),
            Some(ThetaRole::Temporal)
        );
        assert_eq!(
            SemanticRole::Modifier(ArgumentModifier::Manner).to_theta_role(),
            Some(ThetaRole::Manner)
        );
        assert_eq!(
            SemanticRole::Modifier(ArgumentModifier::Cause).to_theta_role(),
            Some(ThetaRole::Cause)
        );

        // Some modifiers have no theta role mapping
        assert_eq!(
            SemanticRole::Modifier(ArgumentModifier::Negation).to_theta_role(),
            None
        );
    }

    // === ArgumentModifier Tests ===

    #[test]
    fn test_argument_modifier_from_propbank_label() {
        assert_eq!(
            ArgumentModifier::from_propbank_label("LOC"),
            ArgumentModifier::Location
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("TMP"),
            ArgumentModifier::Time
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("MNR"),
            ArgumentModifier::Manner
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("CAU"),
            ArgumentModifier::Cause
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("PRP"),
            ArgumentModifier::Purpose
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("DIR"),
            ArgumentModifier::Direction
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("EXT"),
            ArgumentModifier::Extent
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("REC"),
            ArgumentModifier::Reciprocal
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("PRD"),
            ArgumentModifier::Predicate
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("MOD"),
            ArgumentModifier::Modal
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("NEG"),
            ArgumentModifier::Negation
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("DIS"),
            ArgumentModifier::Discourse
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("ADV"),
            ArgumentModifier::Adverbial
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("LVB"),
            ArgumentModifier::LightVerb
        );
        assert_eq!(
            ArgumentModifier::from_propbank_label("UNKNOWN"),
            ArgumentModifier::Other("UNKNOWN".to_string())
        );
    }

    #[test]
    fn test_argument_modifier_display() {
        assert_eq!(format!("{}", ArgumentModifier::Location), "LOC");
        assert_eq!(format!("{}", ArgumentModifier::Time), "TMP");
        assert_eq!(format!("{}", ArgumentModifier::Manner), "MNR");
        assert_eq!(format!("{}", ArgumentModifier::Cause), "CAU");
        assert_eq!(format!("{}", ArgumentModifier::Negation), "NEG");
        assert_eq!(
            format!("{}", ArgumentModifier::Other("CUSTOM".to_string())),
            "CUSTOM"
        );
    }

    #[test]
    fn test_argument_modifier_from_str() {
        let loc: ArgumentModifier = "LOC".parse().unwrap();
        assert_eq!(loc, ArgumentModifier::Location);

        let tmp: ArgumentModifier = "TMP".parse().unwrap();
        assert_eq!(tmp, ArgumentModifier::Time);

        let other: ArgumentModifier = "XYZ".parse().unwrap();
        assert_eq!(other, ArgumentModifier::Other("XYZ".to_string()));
    }

    // === PropBankArgument Tests ===

    #[test]
    fn test_propbank_argument_new() {
        let arg = PropBankArgument::new(SemanticRole::Agent, "the giver".to_string(), 0.9);
        assert_eq!(arg.role, SemanticRole::Agent);
        assert_eq!(arg.description, "the giver");
        assert!(arg.token_span.is_none());
        assert_eq!(arg.confidence, 0.9);
    }

    #[test]
    fn test_propbank_argument_with_span() {
        let arg = PropBankArgument::with_span(
            SemanticRole::Patient,
            "the gift".to_string(),
            (2, 4),
            0.85,
        );
        assert_eq!(arg.role, SemanticRole::Patient);
        assert_eq!(arg.description, "the gift");
        assert_eq!(arg.token_span, Some((2, 4)));
        assert_eq!(arg.confidence, 0.85);
    }

    #[test]
    fn test_propbank_argument_is_core_argument() {
        let agent = PropBankArgument::new(SemanticRole::Agent, "test".to_string(), 1.0);
        assert!(agent.is_core_argument());

        let patient = PropBankArgument::new(SemanticRole::Patient, "test".to_string(), 1.0);
        assert!(patient.is_core_argument());

        let modifier = PropBankArgument::new(
            SemanticRole::Modifier(ArgumentModifier::Location),
            "test".to_string(),
            1.0,
        );
        assert!(!modifier.is_core_argument());
    }

    #[test]
    fn test_propbank_argument_is_modifier() {
        let modifier = PropBankArgument::new(
            SemanticRole::Modifier(ArgumentModifier::Time),
            "yesterday".to_string(),
            0.9,
        );
        assert!(modifier.is_modifier());
        assert!(!modifier.is_core_argument());

        let agent = PropBankArgument::new(SemanticRole::Agent, "John".to_string(), 1.0);
        assert!(!agent.is_modifier());
    }

    // === PropBankPredicate Tests ===

    #[test]
    fn test_propbank_predicate_new() {
        let pred = PropBankPredicate::new(
            "give".to_string(),
            "01".to_string(),
            "transfer possession".to_string(),
        );
        assert_eq!(pred.lemma, "give");
        assert_eq!(pred.sense, "01");
        assert_eq!(pred.roleset, "give.01");
        assert_eq!(pred.definition, "transfer possession");
        assert!(pred.arguments.is_empty());
        assert!(pred.predicate_span.is_none());
    }

    #[test]
    fn test_propbank_predicate_add_argument() {
        let mut pred =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Agent,
            "giver".to_string(),
            1.0,
        ));
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Patient,
            "gift".to_string(),
            1.0,
        ));

        assert_eq!(pred.arguments.len(), 2);
    }

    #[test]
    fn test_propbank_predicate_get_arguments_by_role() {
        let mut pred =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Agent,
            "giver".to_string(),
            1.0,
        ));
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Patient,
            "gift".to_string(),
            1.0,
        ));

        let agents = pred.get_arguments_by_role(&SemanticRole::Agent);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].description, "giver");

        let themes = pred.get_arguments_by_role(&SemanticRole::Patient);
        assert_eq!(themes.len(), 1);

        let goals = pred.get_arguments_by_role(&SemanticRole::EndingPoint);
        assert!(goals.is_empty());
    }

    #[test]
    fn test_propbank_predicate_get_core_arguments() {
        let mut pred =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Agent,
            "giver".to_string(),
            1.0,
        ));
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Modifier(ArgumentModifier::Time),
            "yesterday".to_string(),
            0.9,
        ));

        let core = pred.get_core_arguments();
        assert_eq!(core.len(), 1);
        assert_eq!(core[0].description, "giver");
    }

    #[test]
    fn test_propbank_predicate_get_modifiers() {
        let mut pred =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Agent,
            "giver".to_string(),
            1.0,
        ));
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Modifier(ArgumentModifier::Time),
            "yesterday".to_string(),
            0.9,
        ));
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Modifier(ArgumentModifier::Location),
            "at home".to_string(),
            0.8,
        ));

        let modifiers = pred.get_modifiers();
        assert_eq!(modifiers.len(), 2);
    }

    #[test]
    fn test_propbank_predicate_has_role() {
        let mut pred =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Agent,
            "giver".to_string(),
            1.0,
        ));

        assert!(pred.has_role(&SemanticRole::Agent));
        assert!(!pred.has_role(&SemanticRole::Patient));
    }

    // === PropBankFrameset Tests ===

    #[test]
    fn test_propbank_frameset_new() {
        let frameset =
            PropBankFrameset::new("give".to_string(), "transfer possession verb".to_string());
        assert_eq!(frameset.lemma, "give");
        assert_eq!(frameset.notes, "transfer possession verb");
        assert!(frameset.rolesets.is_empty());
    }

    #[test]
    fn test_propbank_frameset_add_and_get_roleset() {
        let mut frameset = PropBankFrameset::new("give".to_string(), String::new());

        let pred1 =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        let pred2 =
            PropBankPredicate::new("give".to_string(), "02".to_string(), "yield".to_string());

        frameset.add_roleset(pred1);
        frameset.add_roleset(pred2);

        assert_eq!(frameset.rolesets.len(), 2);

        let rs01 = frameset.get_roleset("01");
        assert!(rs01.is_some());
        assert_eq!(rs01.unwrap().definition, "transfer");

        let rs02 = frameset.get_roleset("02");
        assert!(rs02.is_some());
        assert_eq!(rs02.unwrap().definition, "yield");

        assert!(frameset.get_roleset("99").is_none());
    }

    #[test]
    fn test_propbank_frameset_get_all_rolesets() {
        let mut frameset = PropBankFrameset::new("give".to_string(), String::new());
        frameset.add_roleset(PropBankPredicate::new(
            "give".to_string(),
            "01".to_string(),
            "def1".to_string(),
        ));
        frameset.add_roleset(PropBankPredicate::new(
            "give".to_string(),
            "02".to_string(),
            "def2".to_string(),
        ));

        let all = frameset.get_all_rolesets();
        assert_eq!(all.len(), 2);
    }

    // === PropBankAnalysis Tests ===

    #[test]
    fn test_propbank_analysis_new() {
        let analysis = PropBankAnalysis::new("give".to_string());
        assert_eq!(analysis.input, "give");
        assert!(analysis.predicate.is_none());
        assert!(analysis.alternative_rolesets.is_empty());
        assert_eq!(analysis.confidence, 0.0);
        assert_eq!(analysis.argument_count, 0);
        assert!(analysis.theta_roles.is_empty());
    }

    #[test]
    fn test_propbank_analysis_with_predicate() {
        let mut pred =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Agent,
            "giver".to_string(),
            1.0,
        ));
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Patient,
            "gift".to_string(),
            1.0,
        ));

        let analysis = PropBankAnalysis::with_predicate("give".to_string(), pred, 0.9);
        assert!(analysis.predicate.is_some());
        assert_eq!(analysis.confidence, 0.9);
        assert_eq!(analysis.argument_count, 2);
        assert_eq!(analysis.theta_roles.len(), 2);
        assert!(analysis.theta_roles.contains(&ThetaRole::Agent));
        assert!(analysis.theta_roles.contains(&ThetaRole::Patient));
    }

    #[test]
    fn test_propbank_analysis_add_alternative() {
        let mut analysis = PropBankAnalysis::new("give".to_string());
        let alt = PropBankPredicate::new("give".to_string(), "02".to_string(), "yield".to_string());
        analysis.add_alternative(alt);

        assert_eq!(analysis.alternative_rolesets.len(), 1);
    }

    #[test]
    fn test_propbank_analysis_has_match() {
        let mut analysis = PropBankAnalysis::new("give".to_string());
        assert!(!analysis.has_match());

        analysis.add_alternative(PropBankPredicate::new(
            "give".to_string(),
            "01".to_string(),
            "test".to_string(),
        ));
        assert!(analysis.has_match());
    }

    #[test]
    fn test_propbank_analysis_all_predicates() {
        let mut analysis = PropBankAnalysis::new("give".to_string());
        let primary =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "primary".to_string());
        let alt = PropBankPredicate::new("give".to_string(), "02".to_string(), "alt".to_string());

        analysis.predicate = Some(primary);
        analysis.add_alternative(alt);

        let all = analysis.all_predicates();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_propbank_analysis_best_predicate() {
        let analysis = PropBankAnalysis::new("give".to_string());
        assert!(analysis.best_predicate().is_none());

        let pred = PropBankPredicate::new("give".to_string(), "01".to_string(), "test".to_string());
        let analysis = PropBankAnalysis::with_predicate("give".to_string(), pred, 0.9);
        assert!(analysis.best_predicate().is_some());
    }

    #[test]
    fn test_propbank_analysis_calculate_confidence() {
        // Empty analysis
        let mut empty = PropBankAnalysis::new("test".to_string());
        empty.calculate_confidence();
        assert_eq!(empty.confidence, 0.0);

        // Analysis with only alternatives
        let mut with_alt = PropBankAnalysis::new("give".to_string());
        with_alt.add_alternative(PropBankPredicate::new(
            "give".to_string(),
            "01".to_string(),
            "test".to_string(),
        ));
        with_alt.calculate_confidence();
        assert_eq!(with_alt.confidence, 0.4);

        // Analysis with predicate and arguments
        let mut pred =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Agent,
            "giver".to_string(),
            1.0,
        ));
        pred.add_argument(PropBankArgument::new(
            SemanticRole::Patient,
            "gift".to_string(),
            1.0,
        ));
        let mut analysis = PropBankAnalysis::with_predicate("give".to_string(), pred, 0.5);
        analysis.calculate_confidence();
        // 2 core args * 0.3 = 0.6
        assert!(analysis.confidence > 0.5);
    }

    // === Serialization Tests ===

    #[test]
    fn test_semantic_role_serialization() {
        let role = SemanticRole::Agent;
        let json = serde_json::to_string(&role).unwrap();
        let deserialized: SemanticRole = serde_json::from_str(&json).unwrap();
        assert_eq!(role, deserialized);
    }

    #[test]
    fn test_argument_modifier_serialization() {
        let modifier = ArgumentModifier::Location;
        let json = serde_json::to_string(&modifier).unwrap();
        let deserialized: ArgumentModifier = serde_json::from_str(&json).unwrap();
        assert_eq!(modifier, deserialized);
    }

    #[test]
    fn test_propbank_predicate_serialization() {
        let pred =
            PropBankPredicate::new("give".to_string(), "01".to_string(), "transfer".to_string());
        let json = serde_json::to_string(&pred).unwrap();
        let deserialized: PropBankPredicate = serde_json::from_str(&json).unwrap();
        assert_eq!(pred.lemma, deserialized.lemma);
        assert_eq!(pred.sense, deserialized.sense);
    }
}
