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
