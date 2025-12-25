//! VerbNet type definitions
//!
//! These types mirror the VerbNet 3.4 XML schema structure, providing
//! Rust representations of VerbNet classes, roles, frames, and semantics.

use canopy_core::paths::data_path_string;
use canopy_core::ThetaRole as CoreThetaRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A VerbNet verb class (root element from XML)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerbClass {
    /// Class identifier (e.g., "give-13.1")
    pub id: String,
    /// Human-readable class name
    pub class_name: String,
    /// Parent class ID for inheritance
    pub parent_class: Option<String>,
    /// List of verb members in this class
    pub members: Vec<Member>,
    /// Thematic roles for this class
    pub themroles: Vec<ThematicRole>,
    /// Syntactic and semantic frames
    pub frames: Vec<Frame>,
    /// Subclass IDs
    pub subclasses: Vec<String>,
}

/// A verb member of a VerbNet class
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Member {
    /// Verb lemma
    pub name: String,
    /// WordNet sense mappings
    pub wn: Option<String>,
    /// PropBank frame grouping
    pub grouping: Option<String>,
    /// Additional features
    pub features: Option<String>,
}

/// Thematic role definition with selectional restrictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThematicRole {
    /// Role type (Agent, Patient, Theme, etc.)
    pub role_type: String,
    /// Selectional restrictions on this role
    pub selrestrs: SelectionalRestrictions,
}

/// Selectional restrictions on thematic roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectionalRestrictions {
    /// Logic operator for combining restrictions
    pub logic: Option<LogicType>,
    /// Individual restrictions
    pub restrictions: Vec<SelectionalRestriction>,
}

/// Logic type for combining selectional restrictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogicType {
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
}

/// Individual selectional restriction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectionalRestriction {
    /// Restriction type (animate, concrete, etc.)
    #[serde(rename = "type")]
    pub restriction_type: String,
    /// Value (+ or -)
    #[serde(rename = "Value")]
    pub value: String,
}

/// Syntactic and semantic frame
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Frame {
    /// Frame description
    pub description: FrameDescription,
    /// Example sentences
    pub examples: Vec<Example>,
    /// Syntactic pattern
    pub syntax: SyntaxPattern,
    /// Semantic predicates
    pub semantics: Vec<SemanticPredicate>,
}

/// Frame description with numbering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameDescription {
    /// Description number
    #[serde(rename = "descriptionNumber")]
    pub description_number: String,
    /// Primary description
    pub primary: String,
    /// Secondary description
    pub secondary: Option<String>,
    /// XTAG reference
    pub xtag: Option<String>,
}

/// Example sentence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Example {
    /// Example text
    pub text: String,
}

/// Syntactic pattern for a frame
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyntaxPattern {
    /// Syntax elements (NP, V, PP, etc.)
    pub elements: Vec<SyntaxElement>,
}

/// Individual syntax element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyntaxElement {
    /// Element type (NP, V, PREP, etc.)
    pub element_type: String,
    /// Value (for specific elements like prepositions)
    pub value: Option<String>,
    /// Syntactic restrictions
    pub synrestrs: Vec<SyntacticRestriction>,
}

/// Syntactic restriction on syntax elements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyntacticRestriction {
    /// Restriction type
    #[serde(rename = "type")]
    pub restriction_type: String,
    /// Restriction value
    pub value: String,
}

/// Semantic predicate in a frame
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticPredicate {
    /// Predicate name
    pub value: String,
    /// Predicate arguments
    pub args: Vec<Argument>,
    /// Whether the predicate is negated
    #[serde(default)]
    pub negated: bool,
}

/// Argument in a semantic predicate
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Argument {
    /// Argument type (Event, ThemRole, etc.)
    #[serde(rename = "type")]
    pub arg_type: String,
    /// Argument value
    pub value: String,
}

/// VerbNet analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbNetAnalysis {
    /// Analyzed verb
    pub verb: String,
    /// Matching verb classes
    pub verb_classes: Vec<VerbClass>,
    /// Theta role assignments
    pub theta_role_assignments: Vec<ThetaRoleAssignment>,
    /// Semantic predicates
    pub semantic_predicates: Vec<SemanticPredicate>,
    /// Confidence score
    pub confidence: f32,
}

/// Theta role assignment for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThetaRoleAssignment {
    /// Argument position in sentence
    pub argument_position: usize,
    /// Assigned theta role
    pub theta_role: CoreThetaRole,
    /// Assignment confidence
    pub confidence: f32,
}

/// VerbNet engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbNetStats {
    /// Total number of classes loaded
    pub total_classes: usize,
    /// Total number of verbs
    pub total_verbs: usize,
    /// Total queries processed
    pub total_queries: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Average query time in microseconds
    pub avg_query_time_us: f64,
}

/// Configuration for VerbNet engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbNetConfig {
    /// Data directory path
    pub data_path: String,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache capacity
    pub cache_capacity: usize,
    /// Confidence threshold for results
    pub confidence_threshold: f32,
    /// Additional settings
    pub settings: HashMap<String, String>,
}

impl Default for VerbNetConfig {
    fn default() -> Self {
        Self {
            data_path: data_path_string("data/verbnet/vn-gl"),
            enable_cache: true,
            cache_capacity: 10000,
            confidence_threshold: 0.5,
            settings: HashMap::new(),
        }
    }
}

// Utility implementations

impl VerbClass {
    /// Get all member verbs as a vector
    pub fn get_members(&self) -> Vec<&str> {
        self.members.iter().map(|m| m.name.as_str()).collect()
    }

    /// Check if a verb is a member of this class
    pub fn contains_verb(&self, verb: &str) -> bool {
        self.members.iter().any(|m| m.name == verb)
    }

    /// Get all thematic role types for this class
    pub fn get_theta_roles(&self) -> Vec<&str> {
        self.themroles
            .iter()
            .map(|r| r.role_type.as_str())
            .collect()
    }

    /// Get all semantic predicates from all frames
    pub fn get_semantic_predicates(&self) -> Vec<&SemanticPredicate> {
        self.frames.iter().flat_map(|f| &f.semantics).collect()
    }
}

impl ThematicRole {
    /// Check if this role has a specific selectional restriction
    pub fn has_restriction(&self, restriction_type: &str, value: &str) -> bool {
        self.selrestrs
            .restrictions
            .iter()
            .any(|r| r.restriction_type == restriction_type && r.value == value)
    }

    /// Check if this role is animate
    pub fn is_animate(&self) -> bool {
        self.has_restriction("animate", "+")
    }

    /// Check if this role is concrete
    pub fn is_concrete(&self) -> bool {
        self.has_restriction("concrete", "+")
    }
}

impl SelectionalRestrictions {
    /// Create empty restrictions
    pub fn empty() -> Self {
        Self {
            logic: None,
            restrictions: Vec::new(),
        }
    }

    /// Add a restriction
    pub fn add_restriction(&mut self, restriction_type: String, value: String) {
        self.restrictions.push(SelectionalRestriction {
            restriction_type,
            value,
        });
    }
}

impl VerbNetAnalysis {
    /// Create a new analysis result
    pub fn new(verb: String, verb_classes: Vec<VerbClass>, confidence: f32) -> Self {
        let theta_role_assignments = Vec::new(); // Will be populated by engine
        let semantic_predicates = verb_classes
            .iter()
            .flat_map(|c| &c.frames)
            .flat_map(|f| &f.semantics)
            .cloned()
            .collect();

        Self {
            verb,
            verb_classes,
            theta_role_assignments,
            semantic_predicates,
            confidence,
        }
    }

    /// Get the primary (most likely) verb class
    pub fn primary_class(&self) -> Option<&VerbClass> {
        self.verb_classes.first()
    }

    /// Get all theta roles from all matching classes
    pub fn all_theta_roles(&self) -> Vec<&str> {
        self.verb_classes
            .iter()
            .flat_map(|c| c.get_theta_roles())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verb_class_creation() {
        let class = VerbClass {
            id: "test-1.0".to_string(),
            class_name: "Test".to_string(),
            parent_class: None,
            members: vec![Member {
                name: "test".to_string(),
                wn: None,
                grouping: None,
                features: None,
            }],
            themroles: vec![],
            frames: vec![],
            subclasses: vec![],
        };

        assert_eq!(class.id, "test-1.0");
        assert!(class.contains_verb("test"));
        assert!(!class.contains_verb("other"));
    }

    #[test]
    fn test_thematic_role_restrictions() {
        let role = ThematicRole {
            role_type: "Agent".to_string(),
            selrestrs: SelectionalRestrictions {
                logic: Some(LogicType::Or),
                restrictions: vec![
                    SelectionalRestriction {
                        restriction_type: "animate".to_string(),
                        value: "+".to_string(),
                    },
                    SelectionalRestriction {
                        restriction_type: "concrete".to_string(),
                        value: "-".to_string(),
                    },
                ],
            },
        };

        assert!(role.is_animate());
        assert!(!role.is_concrete());
        assert!(role.has_restriction("animate", "+"));
    }

    #[test]
    fn test_verbnet_config_default() {
        let config = VerbNetConfig::default();
        // Path is resolved to workspace-relative, so just check it contains expected suffix
        assert!(
            config.data_path.ends_with("data/verbnet/vn-gl")
                || config.data_path.contains("verbnet/vn-gl"),
            "Expected path to contain verbnet/vn-gl, got: {}",
            config.data_path
        );
        assert!(config.enable_cache);
        assert_eq!(config.cache_capacity, 10000);
    }
}
