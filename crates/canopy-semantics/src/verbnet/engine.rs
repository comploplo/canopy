//! VerbNet lookup engine with efficient indexing
//!
//! This module provides fast lookups for VerbNet data using pre-built indices.

use crate::verbnet::types::*;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info}; // warn will be used for error handling in future

/// Errors that can occur when working with VerbNet
#[derive(Debug, Error)]
pub enum VerbNetError {
    #[error("Failed to load VerbNet data from {path}: {source}")]
    LoadError {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("VerbNet data directory not found: {path}")]
    DirectoryNotFound { path: String },

    #[error("No VerbNet classes found for verb: {verb}")]
    VerbNotFound { verb: String },

    #[error("Invalid VerbNet XML format: {reason}")]
    InvalidFormat { reason: String },
}

/// Fast lookup engine for VerbNet data
#[derive(Debug)]
pub struct VerbNetEngine {
    /// All verb classes indexed by ID
    classes: IndexMap<String, VerbClass>,

    /// Mapping from verb lemmas to class IDs
    verb_to_classes: HashMap<String, Vec<String>>,

    /// Index of predicates to classes that use them
    predicates_index: HashMap<PredicateType, Vec<String>>,

    /// Index of theta roles to classes that use them
    theta_roles_index: HashMap<ThetaRoleType, Vec<String>>,

    /// Index of selectional restrictions to classes
    restrictions_index: HashMap<SelectionalRestriction, Vec<String>>,
}

impl VerbNetEngine {
    /// Create a new empty VerbNet engine
    pub fn new() -> Self {
        Self {
            classes: IndexMap::new(),
            verb_to_classes: HashMap::new(),
            predicates_index: HashMap::new(),
            theta_roles_index: HashMap::new(),
            restrictions_index: HashMap::new(),
        }
    }

    /// Load VerbNet data from XML files in a directory
    pub fn load_from_directory<P: AsRef<Path>>(dir_path: P) -> Result<Self, VerbNetError> {
        let path = dir_path.as_ref();
        
        if !path.exists() || !path.is_dir() {
            return Err(VerbNetError::DirectoryNotFound {
                path: path.display().to_string(),
            });
        }

        info!("Loading VerbNet data from: {}", path.display());
        
        let mut engine = Self::new();
        
        // TODO: Implement actual XML parsing
        // For now, create some test data to establish the interface
        engine.add_test_data();
        
        info!("Loaded {} VerbNet classes", engine.classes.len());
        debug!("Verb coverage: {} verbs", engine.verb_to_classes.len());
        
        Ok(engine)
    }

    /// Get all VerbNet classes for a given verb lemma
    pub fn get_verb_classes(&self, lemma: &str) -> Vec<&VerbClass> {
        self.verb_to_classes
            .get(lemma)
            .map(|class_ids| {
                class_ids
                    .iter()
                    .filter_map(|id| self.classes.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all possible theta roles for a verb
    pub fn get_theta_roles(&self, verb: &str) -> Vec<ThetaRole> {
        self.get_verb_classes(verb)
            .into_iter()
            .flat_map(|class| &class.theta_roles)
            .cloned()
            .collect()
    }

    /// Get selectional restrictions for a specific verb and theta role
    pub fn get_selectional_restrictions(
        &self,
        verb: &str,
        role: ThetaRoleType,
    ) -> Vec<SelectionalRestriction> {
        self.get_theta_roles(verb)
            .into_iter()
            .filter(|theta_role| theta_role.role_type == role)
            .flat_map(|theta_role| theta_role.selectional_restrictions)
            .collect()
    }

    /// Get syntactic frames for a verb
    pub fn get_syntactic_frames(&self, verb: &str) -> Vec<&SyntacticFrame> {
        self.get_verb_classes(verb)
            .into_iter()
            .flat_map(|class| &class.frames)
            .collect()
    }

    /// Get semantic predicates for a verb
    pub fn get_semantic_predicates(&self, verb: &str) -> Vec<SemanticPredicate> {
        self.get_syntactic_frames(verb)
            .into_iter()
            .flat_map(|frame| &frame.semantics)
            .cloned()
            .collect()
    }

    /// Infer aspectual class from VerbNet predicates
    pub fn infer_aspectual_class(&self, verb: &str) -> AspectualInfo {
        let predicates = self.get_semantic_predicates(verb);
        
        let has_motion = predicates
            .iter()
            .any(|p| matches!(p.predicate_type, PredicateType::Motion));
        
        let has_change = predicates
            .iter()
            .any(|p| matches!(p.predicate_type, PredicateType::Change | PredicateType::Transfer));
        
        let has_result = predicates
            .iter()
            .any(|p| matches!(p.predicate_type, PredicateType::Created | PredicateType::Destroyed | PredicateType::Transfer));
        
        let has_duration = predicates.len() > 1; // Simple heuristic
        
        AspectualInfo {
            durative: has_duration,
            dynamic: has_motion || has_change,
            telic: has_result,
            punctual: !has_duration,
        }
    }

    /// Check if a verb can take a specific theta role
    pub fn verb_allows_role(&self, verb: &str, role: ThetaRoleType) -> bool {
        self.get_theta_roles(verb)
            .iter()
            .any(|theta_role| theta_role.role_type == role)
    }

    /// Get all verbs that can take a specific theta role
    pub fn verbs_with_role(&self, role: ThetaRoleType) -> Vec<String> {
        self.theta_roles_index
            .get(&role)
            .map(|class_ids| {
                class_ids
                    .iter()
                    .filter_map(|id| self.classes.get(id))
                    .flat_map(|class| &class.members)
                    .map(|member| member.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get statistics about the loaded VerbNet data
    pub fn get_statistics(&self) -> VerbNetStats {
        VerbNetStats {
            total_classes: self.classes.len(),
            total_verbs: self.verb_to_classes.len(),
            total_roles: self.theta_roles_index.len(),
            total_predicates: self.predicates_index.len(),
            total_restrictions: self.restrictions_index.len(),
        }
    }

    /// Add a verb class and update all indices
    fn add_class(&mut self, class: VerbClass) {
        let class_id = class.id.clone();
        
        // Index verbs to classes
        for member in &class.members {
            self.verb_to_classes
                .entry(member.name.clone())
                .or_default()
                .push(class_id.clone());
        }
        
        // Index theta roles
        for role in &class.theta_roles {
            self.theta_roles_index
                .entry(role.role_type)
                .or_default()
                .push(class_id.clone());
                
            // Index selectional restrictions
            for restriction in &role.selectional_restrictions {
                self.restrictions_index
                    .entry(*restriction)
                    .or_default()
                    .push(class_id.clone());
            }
        }
        
        // Index semantic predicates
        for frame in &class.frames {
            for predicate in &frame.semantics {
                self.predicates_index
                    .entry(predicate.predicate_type.clone())
                    .or_default()
                    .push(class_id.clone());
            }
        }
        
        // Store the class
        self.classes.insert(class_id, class);
    }

    /// Add some test data for development
    pub fn add_test_data(&mut self) {
        // Example: "give" verb class
        let give_class = VerbClass {
            id: "give-13.1".to_string(),
            name: "Give verbs".to_string(),
            members: vec![
                VerbMember {
                    name: "give".to_string(),
                    wn_sense: Some("give%2:40:00".to_string()),
                    fn_mapping: Some("Giving".to_string()),
                    grouping: None,
                },
                VerbMember {
                    name: "hand".to_string(),
                    wn_sense: Some("hand%2:35:00".to_string()),
                    fn_mapping: Some("Giving".to_string()),
                    grouping: None,
                },
            ],
            theta_roles: vec![
                ThetaRole {
                    role_type: ThetaRoleType::Agent,
                    selectional_restrictions: vec![SelectionalRestriction::Animate],
                    syntax_restrictions: vec![],
                },
                ThetaRole {
                    role_type: ThetaRoleType::Theme,
                    selectional_restrictions: vec![SelectionalRestriction::Concrete],
                    syntax_restrictions: vec![],
                },
                ThetaRole {
                    role_type: ThetaRoleType::Recipient,
                    selectional_restrictions: vec![SelectionalRestriction::Animate],
                    syntax_restrictions: vec![],
                },
            ],
            frames: vec![SyntacticFrame {
                description: "NP V NP NP".to_string(),
                primary: "ditransitive".to_string(),
                secondary: None,
                example: "She gave him a book".to_string(),
                syntax: SyntaxPattern {
                    elements: vec![
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Agent),
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "V".to_string(),
                            theta_role: None,
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Recipient),
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Theme),
                            restrictions: vec![],
                        },
                    ],
                },
                semantics: vec![
                    SemanticPredicate {
                        predicate_type: PredicateType::Transfer,
                        event_time: EventTime::During,
                        arguments: vec!["Agent".to_string(), "Theme".to_string(), "Recipient".to_string()],
                        negated: false,
                    },
                    SemanticPredicate {
                        predicate_type: PredicateType::Cause,
                        event_time: EventTime::Start,
                        arguments: vec!["Agent".to_string()],
                        negated: false,
                    },
                ],
            }],
            subclasses: vec![],
        };

        self.add_class(give_class);
        
        info!("Added test VerbNet data for development");
    }
}

impl Default for VerbNetEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about loaded VerbNet data
#[derive(Debug, Clone)]
pub struct VerbNetStats {
    pub total_classes: usize,
    pub total_verbs: usize,
    pub total_roles: usize,
    pub total_predicates: usize,
    pub total_restrictions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = VerbNetEngine::new();
        assert_eq!(engine.classes.len(), 0);
        assert_eq!(engine.verb_to_classes.len(), 0);
    }

    #[test]
    fn test_test_data() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();
        
        let stats = engine.get_statistics();
        assert!(stats.total_classes > 0);
        assert!(stats.total_verbs > 0);
    }

    #[test]
    fn test_verb_lookup() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();
        
        let classes = engine.get_verb_classes("give");
        assert!(!classes.is_empty());
        assert_eq!(classes[0].id, "give-13.1");
    }

    #[test]
    fn test_theta_role_lookup() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();
        
        let roles = engine.get_theta_roles("give");
        assert!(!roles.is_empty());
        
        let has_agent = roles.iter().any(|r| r.role_type == ThetaRoleType::Agent);
        assert!(has_agent);
    }

    #[test]
    fn test_aspectual_inference() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();
        
        let aspectual = engine.infer_aspectual_class("give");
        // "give" should be telic (has endpoint) and dynamic
        assert!(aspectual.dynamic || aspectual.telic); // At least one should be true
    }

    #[test]
    fn test_verb_allows_role() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();
        
        assert!(engine.verb_allows_role("give", ThetaRoleType::Agent));
        assert!(engine.verb_allows_role("give", ThetaRoleType::Theme));
        assert!(engine.verb_allows_role("give", ThetaRoleType::Recipient));
        assert!(!engine.verb_allows_role("give", ThetaRoleType::Instrument));
    }
}