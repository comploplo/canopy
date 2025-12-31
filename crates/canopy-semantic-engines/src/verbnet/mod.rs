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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbnet_version() {
        assert_eq!(VERBNET_VERSION, "3.4");
    }

    #[test]
    fn test_default_data_dir() {
        assert!(DEFAULT_DATA_DIR.contains("verbnet"));
    }

    #[test]
    fn test_parse_class_hierarchy_full() {
        let result = utils::parse_class_hierarchy("give-13.1");
        assert!(result.is_some());
        let (verb, major, minor) = result.unwrap();
        assert_eq!(verb, "give");
        assert_eq!(major, "13");
        assert_eq!(minor, "1");
    }

    #[test]
    fn test_parse_class_hierarchy_no_minor() {
        let result = utils::parse_class_hierarchy("run-51");
        assert!(result.is_some());
        let (verb, major, minor) = result.unwrap();
        assert_eq!(verb, "run");
        assert_eq!(major, "51");
        assert_eq!(minor, "0");
    }

    #[test]
    fn test_parse_class_hierarchy_complex() {
        let result = utils::parse_class_hierarchy("run-51.3.2");
        assert!(result.is_some());
        let (verb, major, minor) = result.unwrap();
        assert_eq!(verb, "run");
        assert_eq!(major, "51");
        assert_eq!(minor, "3"); // Takes first two parts only
    }

    #[test]
    fn test_parse_class_hierarchy_invalid() {
        assert!(utils::parse_class_hierarchy("justverb").is_none());
        assert!(utils::parse_class_hierarchy("").is_none());
    }

    #[test]
    fn test_most_specific_class() {
        let classes = vec![
            "give-13".to_string(),
            "give-13.1".to_string(),
            "give-13.1.1".to_string(),
        ];
        let result = utils::most_specific_class(&classes);
        assert_eq!(result, Some("give-13.1.1".to_string()));
    }

    #[test]
    fn test_most_specific_class_empty() {
        let classes: Vec<String> = vec![];
        assert!(utils::most_specific_class(&classes).is_none());
    }

    #[test]
    fn test_class_has_role() {
        let verb_class = VerbClass {
            id: "give-13.1".to_string(),
            class_name: "give".to_string(),
            parent_class: None,
            members: vec![],
            themroles: vec![
                ThematicRole {
                    role_type: "Agent".to_string(),
                    selrestrs: SelectionalRestrictions::empty(),
                },
                ThematicRole {
                    role_type: "Theme".to_string(),
                    selrestrs: SelectionalRestrictions::empty(),
                },
            ],
            frames: vec![],
            subclasses: vec![],
        };

        assert!(utils::class_has_role(&verb_class, "Agent"));
        assert!(utils::class_has_role(&verb_class, "Theme"));
        assert!(!utils::class_has_role(&verb_class, "Goal"));
    }

    #[test]
    fn test_extract_all_verbs() {
        let classes = vec![
            VerbClass {
                id: "give-13.1".to_string(),
                class_name: "give".to_string(),
                parent_class: None,
                members: vec![
                    Member {
                        name: "give".to_string(),
                        wn: Some("give%2:40:00".to_string()),
                        grouping: None,
                        features: None,
                    },
                    Member {
                        name: "donate".to_string(),
                        wn: None,
                        grouping: None,
                        features: None,
                    },
                ],
                themroles: vec![],
                frames: vec![],
                subclasses: vec![],
            },
            VerbClass {
                id: "run-51.3.2".to_string(),
                class_name: "run".to_string(),
                parent_class: None,
                members: vec![Member {
                    name: "run".to_string(),
                    wn: None,
                    grouping: None,
                    features: None,
                }],
                themroles: vec![],
                frames: vec![],
                subclasses: vec![],
            },
        ];

        let verbs = utils::extract_all_verbs(&classes);
        assert_eq!(verbs.len(), 3);
        assert!(verbs.contains(&"give".to_string()));
        assert!(verbs.contains(&"donate".to_string()));
        assert!(verbs.contains(&"run".to_string()));
    }

    #[test]
    fn test_extract_all_verbs_empty() {
        let classes: Vec<VerbClass> = vec![];
        let verbs = utils::extract_all_verbs(&classes);
        assert!(verbs.is_empty());
    }
}
