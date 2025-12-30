//! VerbNet integration for semantic analysis
//!
//! VerbNet 3.4 XML parsing and verb class analysis capabilities.

pub mod engine;
pub mod parser;
pub mod types;

// Re-export main types
pub use engine::VerbNetEngine;
pub use parser::VerbClassParser;
pub use types::{
    Argument, Example, Frame, FrameDescription, LogicType, Member, SelectionalRestriction,
    SelectionalRestrictions, SemanticPredicate, SyntacticRestriction, SyntaxElement, SyntaxPattern,
    ThematicRole, ThetaRoleAssignment, VerbClass, VerbNetAnalysis, VerbNetConfig, VerbNetStats,
};

/// VerbNet version information
pub const VERBNET_VERSION: &str = "3.4";

/// Default VerbNet data directory
pub const DEFAULT_DATA_DIR: &str = "data/verbnet/vn-gl";

/// Utility functions for VerbNet operations
pub mod utils {
    use super::types::{ThematicRole, VerbClass};

    /// Check if a verb class contains a specific thematic role
    pub fn class_has_role(verb_class: &VerbClass, role_type: &str) -> bool {
        verb_class
            .themroles
            .iter()
            .any(|r| r.role_type == role_type)
    }

    /// Get all verbs from a list of verb classes
    pub fn extract_all_verbs(classes: &[VerbClass]) -> Vec<String> {
        classes
            .iter()
            .flat_map(|c| &c.members)
            .map(|m| m.name.clone())
            .collect()
    }

    /// Check if a role has specific selectional restrictions
    pub fn role_matches_restrictions(role: &ThematicRole, restrictions: &[(&str, &str)]) -> bool {
        restrictions
            .iter()
            .all(|(restr_type, value)| role.has_restriction(restr_type, value))
    }

    /// Get the most specific (deepest) class ID from a list
    pub fn most_specific_class(class_ids: &[String]) -> Option<String> {
        class_ids
            .iter()
            .max_by_key(|id| id.matches('-').count())
            .cloned()
    }

    /// Parse class hierarchy from class ID (e.g., "give-13.1" -> ("give", "13", "1"))
    pub fn parse_class_hierarchy(class_id: &str) -> Option<(String, String, String)> {
        let parts: Vec<&str> = class_id.split('-').collect();
        if parts.len() >= 2 {
            let base_verb = parts[0].to_string();
            let number_parts: Vec<&str> = parts[1].split('.').collect();
            if number_parts.len() >= 2 {
                Some((
                    base_verb,
                    number_parts[0].to_string(),
                    number_parts[1].to_string(),
                ))
            } else {
                Some((base_verb, number_parts[0].to_string(), "0".to_string()))
            }
        } else {
            None
        }
    }
}
