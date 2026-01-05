//! `SenseProvider` implementation using `VerbNet`.
//!
//! Maps `VerbNet` verb classes to `PredicateDecomposition` structures
//! for the kernel's event composition layer. Predicate mappings are
//! loaded from `data/mappings/predicate-to-littlev.toml`.

use crate::engine::PredicateToLittleVMap;
use crate::verbnet::{SemanticPredicate, VerbClass, VerbNetEngine};
use canopy::kernel::events::LittleVType;
use canopy::runtime::{
    AnnotatedSyntax, DecompositionSource, FrameId, PredicateDecomposition, SenseId, SenseInfo,
    SenseProvider, SenseSource, TokenId,
};
use canopy::{CanopyError, ThetaRole};
use std::sync::Arc;

/// `SenseProvider` implementation using `VerbNet` engine.
///
/// Decomposes predicates into `LittleV` structures based on `VerbNet`
/// semantic predicates and verb class information. Mappings are
/// loaded from external TOML configuration.
#[derive(Debug)]
pub struct VerbNetSenseProvider {
    engine: Arc<VerbNetEngine>,
    /// Predicate to `LittleV` mapping (loaded from TOML)
    predicate_map: PredicateToLittleVMap,
}

impl VerbNetSenseProvider {
    /// Create a new provider with `VerbNet` engine.
    ///
    /// # Errors
    /// Returns an error if `VerbNet` data cannot be loaded.
    pub fn new() -> Result<Self, CanopyError> {
        let engine = VerbNetEngine::new()
            .map_err(|e| CanopyError::data_load(format!("Failed to load VerbNet: {e}")))?;

        // Load predicate mappings from TOML file
        let predicate_map = PredicateToLittleVMap::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load predicate mappings, using defaults: {e}");
            PredicateToLittleVMap::default()
        });

        Ok(Self {
            engine: Arc::new(engine),
            predicate_map,
        })
    }

    /// Create from an existing engine (for sharing).
    #[must_use]
    pub fn with_engine(engine: Arc<VerbNetEngine>) -> Self {
        // Load predicate mappings from TOML file
        let predicate_map = PredicateToLittleVMap::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load predicate mappings, using defaults: {e}");
            PredicateToLittleVMap::default()
        });

        Self {
            engine,
            predicate_map,
        }
    }

    /// Map `VerbNet` semantic predicates to `LittleVType` using loaded mappings.
    fn predicate_to_little_v(&self, predicates: &[SemanticPredicate]) -> LittleVType {
        // Check each predicate against the loaded mapping
        for pred in predicates {
            let name = pred.value.to_lowercase();
            if self.predicate_map.contains(&name) {
                return self.predicate_map.get(&name);
            }
        }

        // Return default from mapping (typically LittleVType::Do)
        self.predicate_map.default_type()
    }

    /// Check if predicates indicate a causative structure (CAUSE(BECOME)).
    fn has_causative_structure(predicates: &[SemanticPredicate]) -> bool {
        let has_cause = predicates.iter().any(|p| p.value.to_lowercase() == "cause");
        let has_result = predicates.iter().any(|p| {
            matches!(
                p.value.to_lowercase().as_str(),
                "become" | "result" | "start"
            )
        });
        has_cause && has_result
    }

    /// Extract theta roles from `VerbNet` class.
    fn extract_theta_roles(class: &VerbClass) -> Vec<ThetaRole> {
        class
            .themroles
            .iter()
            .filter_map(|r| ThetaRole::parse(&r.role_type))
            .collect()
    }

    /// Decompose a single `VerbNet` class into `PredicateDecomposition`.
    fn decompose_class(&self, class: &VerbClass, confidence: f32) -> PredicateDecomposition {
        let predicates: Vec<_> = class
            .frames
            .iter()
            .flat_map(|f| &f.semantics)
            .cloned()
            .collect();

        let little_v = self.predicate_to_little_v(&predicates);
        let expected_roles = Self::extract_theta_roles(class);

        let mut decomp =
            PredicateDecomposition::new(SenseId::new(&class.id), little_v, expected_roles.clone())
                .with_confidence(confidence)
                .with_source(DecompositionSource::VerbNet);

        // Add sub-event for causatives (CAUSE contains BECOME)
        if Self::has_causative_structure(&predicates) && little_v == LittleVType::Cause {
            let sub_roles: Vec<_> = expected_roles
                .iter()
                .filter(|r| matches!(r, ThetaRole::Patient | ThetaRole::Theme))
                .copied()
                .collect();

            let sub_event = PredicateDecomposition::new(
                SenseId::new(format!("{}-become", class.id)),
                LittleVType::Become,
                sub_roles,
            )
            .with_confidence(confidence)
            .with_source(DecompositionSource::VerbNet);

            decomp = decomp.with_sub_event(sub_event);
        }

        decomp
    }
}

impl SenseProvider for VerbNetSenseProvider {
    fn decompose_predicate(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
    ) -> Result<Vec<PredicateDecomposition>, CanopyError> {
        // Get the predicate lemma from syntax
        let Some(token) = syntax.tokens.get(pred_id.index()) else {
            return Ok(vec![]);
        };

        let lemma = &token.lemma;

        // Look up in VerbNet
        let result = self
            .engine
            .analyze_verb(lemma)
            .map_err(|e| CanopyError::analysis(lemma, format!("VerbNet analysis failed: {e}")))?;

        if result.data.verb_classes.is_empty() {
            return Ok(vec![]);
        }

        // Convert each matching class to a decomposition
        let decompositions: Vec<_> = result
            .data
            .verb_classes
            .iter()
            .map(|class| self.decompose_class(class, result.confidence))
            .collect();

        Ok(decompositions)
    }

    fn frames_for_sense(&self, sense: &SenseId) -> Result<Vec<FrameId>, CanopyError> {
        // Get the VerbNet class
        let class_id = sense.as_str();
        let Some(class) = self.engine.get_verb_class(class_id) else {
            return Ok(vec![]);
        };

        // Extract frame IDs from the class
        let frames: Vec<_> = class
            .frames
            .iter()
            .enumerate()
            .map(|(i, _)| FrameId::new(format!("{class_id}-frame{i}")))
            .collect();

        Ok(frames)
    }

    fn get_sense(&self, id: &SenseId) -> Result<Option<SenseInfo>, CanopyError> {
        let class_id = id.as_str();
        let Some(class) = self.engine.get_verb_class(class_id) else {
            return Ok(None);
        };

        let theta_roles = Self::extract_theta_roles(class);

        Ok(Some(SenseInfo {
            id: id.clone(),
            description: class.class_name.clone(),
            source: SenseSource::VerbNet,
            theta_roles,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verbnet_available() -> bool {
        crate::paths::data_path("data/verbnet").exists()
    }

    #[test]
    fn test_provider_creation() {
        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetSenseProvider::new();
        assert!(provider.is_ok());
    }

    #[test]
    fn test_predicate_to_little_v() {
        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetSenseProvider::new().unwrap();

        // Test cause
        let cause_pred = vec![SemanticPredicate {
            value: "cause".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(
            provider.predicate_to_little_v(&cause_pred),
            LittleVType::Cause
        );

        // Test motion
        let motion_pred = vec![SemanticPredicate {
            value: "motion".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(
            provider.predicate_to_little_v(&motion_pred),
            LittleVType::Go
        );

        // Test default
        let unknown_pred = vec![SemanticPredicate {
            value: "unknown".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(
            provider.predicate_to_little_v(&unknown_pred),
            LittleVType::Do
        );
    }

    #[test]
    fn test_get_sense() {
        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetSenseProvider::new().unwrap();

        // Test a known VerbNet class
        let sense = provider.get_sense(&SenseId::new("give-13.1"));
        assert!(sense.is_ok());
        // May or may not exist depending on loaded data
    }

    #[test]
    fn test_predicate_to_little_v_all_types() {
        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetSenseProvider::new().unwrap();

        // Test become
        let become_pred = vec![SemanticPredicate {
            value: "become".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(
            provider.predicate_to_little_v(&become_pred),
            LittleVType::Become
        );

        // Test state (Be)
        let state_pred = vec![SemanticPredicate {
            value: "state".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(provider.predicate_to_little_v(&state_pred), LittleVType::Be);

        // Test experience
        let exp_pred = vec![SemanticPredicate {
            value: "emotional_state".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(
            provider.predicate_to_little_v(&exp_pred),
            LittleVType::Experience
        );

        // Test say
        let say_pred = vec![SemanticPredicate {
            value: "transfer_info".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(provider.predicate_to_little_v(&say_pred), LittleVType::Say);

        // Test have
        let have_pred = vec![SemanticPredicate {
            value: "has_possession".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(
            provider.predicate_to_little_v(&have_pred),
            LittleVType::Have
        );

        // Test exist
        let exist_pred = vec![SemanticPredicate {
            value: "exist".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(
            provider.predicate_to_little_v(&exist_pred),
            LittleVType::Exist
        );

        // Test path (Go)
        let path_pred = vec![SemanticPredicate {
            value: "path".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(provider.predicate_to_little_v(&path_pred), LittleVType::Go);

        // Test perceive (Experience)
        let perceive_pred = vec![SemanticPredicate {
            value: "perceive".to_string(),
            args: vec![],
            negated: false,
        }];
        assert_eq!(
            provider.predicate_to_little_v(&perceive_pred),
            LittleVType::Experience
        );
    }

    #[test]
    fn test_has_causative_structure() {
        // Causative: cause + become
        let causative = vec![
            SemanticPredicate {
                value: "cause".to_string(),
                args: vec![],
                negated: false,
            },
            SemanticPredicate {
                value: "become".to_string(),
                args: vec![],
                negated: false,
            },
        ];
        assert!(VerbNetSenseProvider::has_causative_structure(&causative));

        // Causative: cause + result
        let cause_result = vec![
            SemanticPredicate {
                value: "cause".to_string(),
                args: vec![],
                negated: false,
            },
            SemanticPredicate {
                value: "result".to_string(),
                args: vec![],
                negated: false,
            },
        ];
        assert!(VerbNetSenseProvider::has_causative_structure(&cause_result));

        // Not causative: just cause
        let just_cause = vec![SemanticPredicate {
            value: "cause".to_string(),
            args: vec![],
            negated: false,
        }];
        assert!(!VerbNetSenseProvider::has_causative_structure(&just_cause));

        // Not causative: just become
        let just_become = vec![SemanticPredicate {
            value: "become".to_string(),
            args: vec![],
            negated: false,
        }];
        assert!(!VerbNetSenseProvider::has_causative_structure(&just_become));
    }

    #[test]
    fn test_decompose_predicate_empty_tokens() {
        use canopy::runtime::AnnotatedSyntax;

        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetSenseProvider::new().unwrap();

        // Empty syntax
        let syntax = AnnotatedSyntax::new(String::new(), vec![]);
        let result = provider.decompose_predicate(&syntax, TokenId::new(0));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_decompose_predicate_with_verb() {
        use canopy::runtime::{AnnotatedSyntax, AnnotatedToken};
        use canopy::{DepRel, UPos};

        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetSenseProvider::new().unwrap();

        let syntax = AnnotatedSyntax::new(
            "give".to_string(),
            vec![AnnotatedToken::new(
                TokenId::new(0),
                "give".to_string(),
                "give".to_string(),
                UPos::Verb,
                DepRel::Root,
                (0, 4),
            )],
        );

        let result = provider.decompose_predicate(&syntax, TokenId::new(0));
        assert!(result.is_ok());
        // "give" should have VerbNet entries
    }

    #[test]
    fn test_frames_for_sense_nonexistent() {
        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetSenseProvider::new().unwrap();

        let frames = provider.frames_for_sense(&SenseId::new("nonexistent-999.999"));
        assert!(frames.is_ok());
        assert!(frames.unwrap().is_empty());
    }

    #[test]
    fn test_get_sense_nonexistent() {
        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetSenseProvider::new().unwrap();

        let sense = provider.get_sense(&SenseId::new("nonexistent-999.999"));
        assert!(sense.is_ok());
        assert!(sense.unwrap().is_none());
    }

    #[test]
    fn test_with_engine() {
        use std::sync::Arc;

        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let engine = VerbNetEngine::new().unwrap();
        let provider = VerbNetSenseProvider::with_engine(Arc::new(engine));

        // Should work with shared engine
        let sense = provider.get_sense(&SenseId::new("give-13.1"));
        assert!(sense.is_ok());
    }
}
