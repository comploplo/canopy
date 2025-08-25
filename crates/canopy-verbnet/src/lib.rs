//! VerbNet integration for semantic analysis
//!
//! This crate provides VerbNet 3.4 XML parsing and verb class analysis capabilities
//! using the canopy-engine infrastructure. VerbNet is a broad-coverage verb lexicon
//! that maps verbs to syntactic and semantic frames.
//!
//! # Features
//!
//! - **Complete VerbNet 3.4 Support**: Parse all VerbNet XML files with full schema coverage
//! - **Engine Integration**: Uses canopy-engine traits for caching, statistics, and performance
//! - **Semantic Analysis**: Maps verbs to theta roles, frames, and semantic predicates
//! - **High Performance**: LRU caching and optimized data structures
//! - **Fuzzy Matching**: Basic lemmatization and partial verb matching
//!
//! # Example
//!
//! ```rust
//! use canopy_verbnet::{VerbNetEngine, DataLoader};
//!
//! let mut engine = VerbNetEngine::new();
//! engine.load_from_directory("data/verbnet/vn-gl").unwrap();
//!
//! let result = engine.analyze_verb("give").unwrap();
//! println!("VerbNet classes for 'give': {:?}", result.data.verb_classes);
//! ```

pub mod engine;
pub mod parser;
pub mod types;

// Re-export main types for convenience
pub use types::{
    Argument, Example, Frame, FrameDescription, LogicType, Member, SelectionalRestriction,
    SelectionalRestrictions, SemanticPredicate, SyntacticRestriction, SyntaxElement, SyntaxPattern,
    ThematicRole, ThetaRoleAssignment, VerbClass, VerbNetAnalysis, VerbNetConfig, VerbNetStats,
};

pub use engine::VerbNetEngine;
pub use parser::VerbClassParser;

// Re-export engine traits for convenience
pub use canopy_engine::{
    CachedEngine, DataLoader, EngineError, EngineResult, SemanticEngine, SemanticResult,
    StatisticsProvider,
};

/// VerbNet version information
pub const VERBNET_VERSION: &str = "3.4";

/// Default VerbNet data directory
pub const DEFAULT_DATA_DIR: &str = "data/verbnet/vn-gl";

/// Utility functions for VerbNet operations
pub mod utils {
    use crate::types::{ThematicRole, VerbClass};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::*;

    #[test]
    fn test_version_info() {
        assert_eq!(VERBNET_VERSION, "3.4");
        assert_eq!(DEFAULT_DATA_DIR, "data/verbnet/vn-gl");
    }

    #[test]
    fn test_parse_class_hierarchy() {
        let (base, major, minor) = parse_class_hierarchy("give-13.1").unwrap();
        assert_eq!(base, "give");
        assert_eq!(major, "13");
        assert_eq!(minor, "1");

        let (base, major, minor) = parse_class_hierarchy("run-51.3.2").unwrap();
        assert_eq!(base, "run");
        assert_eq!(major, "51");
        assert_eq!(minor, "3");
    }

    #[test]
    fn test_most_specific_class() {
        let classes = vec![
            "give-13".to_string(),
            "give-13.1".to_string(),
            "give-13.1.1".to_string(),
        ];
        assert_eq!(
            most_specific_class(&classes),
            Some("give-13.1.1".to_string())
        );
    }
}
