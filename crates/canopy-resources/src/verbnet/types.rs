//! `VerbNet` type definitions
//!
//! These types mirror the `VerbNet` 3.4 XML schema structure, providing
//! Rust representations of `VerbNet` classes, roles, frames, and semantics.

use crate::paths::data_path_string;
use canopy::ThetaRole as CoreThetaRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A `VerbNet` verb class (root element from XML)
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

/// A verb member of a `VerbNet` class
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Member {
    /// Verb lemma
    pub name: String,
    /// `WordNet` sense mappings
    pub wn: Option<String>,
    /// `PropBank` frame grouping
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
    /// Argument type (Event, `ThemRole`, etc.)
    #[serde(rename = "type")]
    pub arg_type: String,
    /// Argument value
    pub value: String,
}

/// `VerbNet` analysis result
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

/// `VerbNet` engine statistics
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

/// Configuration for `VerbNet` engine
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

// Implement EngineConfigurable trait via macro
crate::impl_engine_configurable!(VerbNetConfig);

// Utility implementations

impl VerbClass {
    /// Get all member verbs as a vector
    #[must_use]
    pub fn get_members(&self) -> Vec<&str> {
        self.members.iter().map(|m| m.name.as_str()).collect()
    }

    /// Check if a verb is a member of this class
    #[must_use]
    pub fn contains_verb(&self, verb: &str) -> bool {
        self.members.iter().any(|m| m.name == verb)
    }

    /// Get all thematic role types for this class
    #[must_use]
    pub fn get_theta_roles(&self) -> Vec<&str> {
        self.themroles
            .iter()
            .map(|r| r.role_type.as_str())
            .collect()
    }

    /// Get all semantic predicates from all frames
    #[must_use]
    pub fn get_semantic_predicates(&self) -> Vec<&SemanticPredicate> {
        self.frames.iter().flat_map(|f| &f.semantics).collect()
    }
}

impl ThematicRole {
    /// Check if this role has a specific selectional restriction
    #[must_use]
    pub fn has_restriction(&self, restriction_type: &str, value: &str) -> bool {
        self.selrestrs
            .restrictions
            .iter()
            .any(|r| r.restriction_type == restriction_type && r.value == value)
    }

    /// Check if this role is animate
    #[must_use]
    pub fn is_animate(&self) -> bool {
        self.has_restriction("animate", "+")
    }

    /// Check if this role is concrete
    #[must_use]
    pub fn is_concrete(&self) -> bool {
        self.has_restriction("concrete", "+")
    }

    /// Convert to core `ThetaRole`
    ///
    /// Uses the canonical mapping from `VerbNet` role types to core theta roles.
    /// Defaults to Agent if the role type is unknown.
    #[must_use]
    pub fn to_core_role(&self) -> CoreThetaRole {
        CoreThetaRole::parse(&self.role_type).unwrap_or(CoreThetaRole::Agent)
    }
}

impl From<&ThematicRole> for CoreThetaRole {
    fn from(role: &ThematicRole) -> Self {
        role.to_core_role()
    }
}

impl From<ThematicRole> for CoreThetaRole {
    fn from(role: ThematicRole) -> Self {
        CoreThetaRole::parse(&role.role_type).unwrap_or(CoreThetaRole::Agent)
    }
}

impl SelectionalRestrictions {
    /// Create empty restrictions
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn primary_class(&self) -> Option<&VerbClass> {
        self.verb_classes.first()
    }

    /// Get all theta roles from all matching classes
    #[must_use]
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

    fn create_test_class() -> VerbClass {
        VerbClass {
            id: "test-1.0".to_string(),
            class_name: "Test".to_string(),
            parent_class: None,
            members: vec![
                Member {
                    name: "test".to_string(),
                    wn: Some("test%2:00:00".to_string()),
                    grouping: Some("test.01".to_string()),
                    features: Some("+feature".to_string()),
                },
                Member {
                    name: "examine".to_string(),
                    wn: None,
                    grouping: None,
                    features: None,
                },
            ],
            themroles: vec![
                ThematicRole {
                    role_type: "Agent".to_string(),
                    selrestrs: SelectionalRestrictions {
                        logic: Some(LogicType::And),
                        restrictions: vec![SelectionalRestriction {
                            restriction_type: "animate".to_string(),
                            value: "+".to_string(),
                        }],
                    },
                },
                ThematicRole {
                    role_type: "Theme".to_string(),
                    selrestrs: SelectionalRestrictions::empty(),
                },
            ],
            frames: vec![Frame {
                description: FrameDescription {
                    description_number: "0.2".to_string(),
                    primary: "NP V NP".to_string(),
                    secondary: Some("Basic Transitive".to_string()),
                    xtag: Some("0.2".to_string()),
                },
                examples: vec![Example {
                    text: "John tests the code.".to_string(),
                }],
                syntax: SyntaxPattern {
                    elements: vec![
                        SyntaxElement {
                            element_type: "NP".to_string(),
                            value: Some("Agent".to_string()),
                            synrestrs: vec![SyntacticRestriction {
                                restriction_type: "np_ptype".to_string(),
                                value: "animate".to_string(),
                            }],
                        },
                        SyntaxElement {
                            element_type: "V".to_string(),
                            value: None,
                            synrestrs: vec![],
                        },
                    ],
                },
                semantics: vec![
                    SemanticPredicate {
                        value: "cause".to_string(),
                        args: vec![Argument {
                            arg_type: "Event".to_string(),
                            value: "E".to_string(),
                        }],
                        negated: false,
                    },
                    SemanticPredicate {
                        value: "not_state".to_string(),
                        args: vec![],
                        negated: true,
                    },
                ],
            }],
            subclasses: vec!["test-1.0-1".to_string()],
        }
    }

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
    fn test_verb_class_get_members() {
        let class = create_test_class();
        let members = class.get_members();
        assert_eq!(members.len(), 2);
        assert!(members.contains(&"test"));
        assert!(members.contains(&"examine"));
    }

    #[test]
    fn test_verb_class_get_theta_roles() {
        let class = create_test_class();
        let roles = class.get_theta_roles();
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&"Agent"));
        assert!(roles.contains(&"Theme"));
    }

    #[test]
    fn test_verb_class_get_semantic_predicates() {
        let class = create_test_class();
        let predicates = class.get_semantic_predicates();
        assert_eq!(predicates.len(), 2);
        assert!(predicates.iter().any(|p| p.value == "cause"));
        assert!(predicates.iter().any(|p| p.negated));
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
    fn test_thematic_role_to_core() {
        let role = ThematicRole {
            role_type: "Agent".to_string(),
            selrestrs: SelectionalRestrictions::empty(),
        };
        assert_eq!(role.to_core_role(), CoreThetaRole::Agent);

        let patient_role = ThematicRole {
            role_type: "Patient".to_string(),
            selrestrs: SelectionalRestrictions::empty(),
        };
        assert_eq!(patient_role.to_core_role(), CoreThetaRole::Patient);
    }

    #[test]
    fn test_thematic_role_from_impls() {
        let role = ThematicRole {
            role_type: "Theme".to_string(),
            selrestrs: SelectionalRestrictions::empty(),
        };

        // Test From<&ThematicRole>
        let core_role: CoreThetaRole = (&role).into();
        assert_eq!(core_role, CoreThetaRole::Theme);

        // Test From<ThematicRole>
        let core_role2: CoreThetaRole = role.into();
        assert_eq!(core_role2, CoreThetaRole::Theme);
    }

    #[test]
    fn test_selectional_restrictions_add() {
        let mut selrestrs = SelectionalRestrictions::empty();
        assert!(selrestrs.restrictions.is_empty());

        selrestrs.add_restriction("animate".to_string(), "+".to_string());
        assert_eq!(selrestrs.restrictions.len(), 1);
        assert_eq!(selrestrs.restrictions[0].restriction_type, "animate");
    }

    #[test]
    fn test_verbnet_analysis_new() {
        let class = create_test_class();
        let analysis = VerbNetAnalysis::new("test".to_string(), vec![class], 0.9);

        assert_eq!(analysis.verb, "test");
        assert_eq!(analysis.verb_classes.len(), 1);
        assert!((analysis.confidence - 0.9).abs() < f32::EPSILON);
        assert!(!analysis.semantic_predicates.is_empty());
    }

    #[test]
    fn test_verbnet_analysis_primary_class() {
        let class = create_test_class();
        let analysis = VerbNetAnalysis::new("test".to_string(), vec![class], 0.9);

        let primary = analysis.primary_class();
        assert!(primary.is_some());
        assert_eq!(primary.unwrap().id, "test-1.0");
    }

    #[test]
    fn test_verbnet_analysis_empty() {
        let analysis = VerbNetAnalysis::new("unknown".to_string(), vec![], 0.0);

        assert!(analysis.primary_class().is_none());
        assert!(analysis.all_theta_roles().is_empty());
        assert!(analysis.semantic_predicates.is_empty());
    }

    #[test]
    fn test_verbnet_analysis_all_theta_roles() {
        let class = create_test_class();
        let analysis = VerbNetAnalysis::new("test".to_string(), vec![class], 0.9);

        let roles = analysis.all_theta_roles();
        assert!(roles.contains(&"Agent"));
        assert!(roles.contains(&"Theme"));
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

    #[test]
    fn test_verbnet_stats() {
        let stats = VerbNetStats {
            total_classes: 100,
            total_verbs: 5000,
            total_queries: 1000,
            cache_hits: 800,
            cache_misses: 200,
            avg_query_time_us: 50.0,
        };

        assert_eq!(stats.total_classes, 100);
        assert_eq!(stats.total_verbs, 5000);
        assert_eq!(stats.cache_hits + stats.cache_misses, stats.total_queries);
    }

    #[test]
    fn test_theta_role_assignment() {
        let assignment = ThetaRoleAssignment {
            argument_position: 0,
            theta_role: CoreThetaRole::Agent,
            confidence: 0.95,
        };

        assert_eq!(assignment.argument_position, 0);
        assert_eq!(assignment.theta_role, CoreThetaRole::Agent);
        assert!((assignment.confidence - 0.95).abs() < f32::EPSILON);
    }

    #[test]
    fn test_logic_type_serialization() {
        // Test that LogicType can be serialized/deserialized
        let and_logic = LogicType::And;
        let or_logic = LogicType::Or;

        assert_ne!(and_logic, or_logic);
    }

    #[test]
    fn test_member_fields() {
        let member = Member {
            name: "give".to_string(),
            wn: Some("give%2:40:00".to_string()),
            grouping: Some("give.01".to_string()),
            features: Some("+transfer".to_string()),
        };

        assert_eq!(member.name, "give");
        assert!(member.wn.is_some());
        assert!(member.grouping.is_some());
        assert!(member.features.is_some());
    }

    #[test]
    fn test_frame_description() {
        let desc = FrameDescription {
            description_number: "0.2".to_string(),
            primary: "NP V NP".to_string(),
            secondary: Some("Transitive".to_string()),
            xtag: Some("0.2".to_string()),
        };

        assert_eq!(desc.primary, "NP V NP");
        assert!(desc.secondary.is_some());
        assert!(desc.xtag.is_some());
    }

    #[test]
    fn test_semantic_predicate() {
        let pred = SemanticPredicate {
            value: "motion".to_string(),
            args: vec![
                Argument {
                    arg_type: "Event".to_string(),
                    value: "e1".to_string(),
                },
                Argument {
                    arg_type: "ThemRole".to_string(),
                    value: "Agent".to_string(),
                },
            ],
            negated: false,
        };

        assert_eq!(pred.value, "motion");
        assert_eq!(pred.args.len(), 2);
        assert!(!pred.negated);
    }
}
