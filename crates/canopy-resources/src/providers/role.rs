//! `RoleProvider` implementation using `VerbNet`.
//!
//! Binds thematic roles to syntactic arguments based on `VerbNet`
//! verb class information and syntactic patterns. Dependency relation
//! mappings are loaded from `data/mappings/deprel-to-theta.toml`.

use crate::engine::DepRelToThetaMap;
use crate::verbnet::VerbNetEngine;
use canopy::runtime::{AnnotatedSyntax, RoleBinding, RoleProvider, RoleSource, SenseId, TokenId};
use canopy::{CanopyError, ThetaRole};
use std::sync::Arc;

/// `RoleProvider` implementation using `VerbNet` engine.
///
/// Binds thematic roles to syntactic arguments based on:
/// - `VerbNet` verb class theta role specifications
/// - Syntactic dependency relations (UTAH-based fallback, loaded from TOML)
#[derive(Debug)]
pub struct VerbNetRoleProvider {
    engine: Arc<VerbNetEngine>,
    /// Dependency relation to theta role mapping (loaded from TOML)
    deprel_map: DepRelToThetaMap,
}

impl VerbNetRoleProvider {
    /// Create a new provider with `VerbNet` engine.
    ///
    /// # Errors
    /// Returns an error if `VerbNet` data cannot be loaded.
    pub fn new() -> Result<Self, CanopyError> {
        let engine = VerbNetEngine::new()
            .map_err(|e| CanopyError::data_load(format!("Failed to load VerbNet: {e}")))?;

        // Load deprel mappings from TOML file
        let deprel_map = DepRelToThetaMap::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load deprel mappings, using defaults: {e}");
            DepRelToThetaMap::default()
        });

        Ok(Self {
            engine: Arc::new(engine),
            deprel_map,
        })
    }

    /// Create from an existing engine (for sharing).
    #[must_use]
    pub fn with_engine(engine: Arc<VerbNetEngine>) -> Self {
        // Load deprel mappings from TOML file
        let deprel_map = DepRelToThetaMap::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load deprel mappings, using defaults: {e}");
            DepRelToThetaMap::default()
        });

        Self { engine, deprel_map }
    }

    /// Get expected theta roles for a `VerbNet` class.
    fn get_class_roles(&self, class_id: &str) -> Vec<ThetaRole> {
        if let Some(class) = self.engine.get_verb_class(class_id) {
            class
                .themroles
                .iter()
                .filter_map(|r| ThetaRole::parse(&r.role_type))
                .collect()
        } else {
            vec![]
        }
    }

    /// UTAH-based fallback: map dependency relations to theta roles.
    ///
    /// Based on Baker's Uniformity of Theta Assignment Hypothesis.
    /// Mappings are loaded from `data/mappings/deprel-to-theta.toml`.
    fn dep_to_role(&self, dep_rel: &str) -> Option<ThetaRole> {
        self.deprel_map.get(dep_rel)
    }

    /// Bind roles using syntactic dependencies.
    fn bind_from_syntax(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
        expected_roles: &[ThetaRole],
    ) -> Vec<RoleBinding> {
        let mut bindings = Vec::new();
        let mut used_roles: Vec<ThetaRole> = Vec::new();

        // Find tokens that depend on the predicate
        for (idx, token) in syntax.tokens.iter().enumerate() {
            if let Some(head) = token.head {
                if head == pred_id {
                    // This token depends on the predicate
                    // Convert deprel to string for matching
                    let dep_str = format!("{:?}", token.deprel).to_lowercase();
                    if let Some(role) = self.dep_to_role(&dep_str) {
                        // Check if this role is expected and not yet used
                        if expected_roles.contains(&role) && !used_roles.contains(&role) {
                            bindings.push(
                                RoleBinding::new(TokenId::new(idx), role, 0.8)
                                    .with_source(RoleSource::VerbNet),
                            );
                            used_roles.push(role);
                        } else {
                            // Use UTAH fallback with lower confidence
                            bindings.push(
                                RoleBinding::new(TokenId::new(idx), role, 0.6)
                                    .with_source(RoleSource::Syntactic),
                            );
                        }
                    }
                }
            }
        }

        bindings
    }
}

impl RoleProvider for VerbNetRoleProvider {
    fn bind_roles(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
        sense: Option<&SenseId>,
    ) -> Result<Vec<RoleBinding>, CanopyError> {
        // Get expected roles from VerbNet if sense is provided
        let expected_roles = if let Some(sense_id) = sense {
            self.get_class_roles(sense_id.as_str())
        } else {
            // Try to look up the verb to get expected roles
            if let Some(token) = syntax.tokens.get(pred_id.index()) {
                let lemma = &token.lemma;
                if let Ok(result) = self.engine.analyze_verb(lemma) {
                    if let Some(class) = result.data.verb_classes.first() {
                        class
                            .themroles
                            .iter()
                            .filter_map(|r| ThetaRole::parse(&r.role_type))
                            .collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        };

        // Bind roles from syntax
        let bindings = self.bind_from_syntax(syntax, pred_id, &expected_roles);

        Ok(bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy::runtime::AnnotatedToken;

    fn verbnet_available() -> bool {
        crate::paths::data_path("data/verbnet").exists()
    }

    #[test]
    fn test_provider_creation() {
        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetRoleProvider::new();
        assert!(provider.is_ok());
    }

    #[test]
    fn test_dep_to_role() {
        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetRoleProvider::new().unwrap();

        assert_eq!(provider.dep_to_role("nsubj"), Some(ThetaRole::Agent));
        assert_eq!(provider.dep_to_role("obj"), Some(ThetaRole::Patient));
        assert_eq!(provider.dep_to_role("iobj"), Some(ThetaRole::Recipient));
        assert_eq!(provider.dep_to_role("nsubj:pass"), Some(ThetaRole::Patient));
        assert_eq!(provider.dep_to_role("unknown"), None);
    }

    #[test]
    fn test_bind_roles_basic() {
        use canopy::{DepRel, UPos};

        if !verbnet_available() {
            eprintln!("Skipping: VerbNet data not available");
            return;
        }

        let provider = VerbNetRoleProvider::new().unwrap();

        // Create simple syntax: "John gave Mary a book"
        // John(0) gave(1) Mary(2) a(3) book(4)
        let syntax = AnnotatedSyntax::new(
            "John gave Mary a book".to_string(),
            vec![
                AnnotatedToken::new(
                    TokenId::new(0),
                    "John".to_string(),
                    "john".to_string(),
                    UPos::Propn,
                    DepRel::Nsubj,
                    (0, 4),
                )
                .with_head(TokenId::new(1)),
                AnnotatedToken::new(
                    TokenId::new(1),
                    "gave".to_string(),
                    "give".to_string(),
                    UPos::Verb,
                    DepRel::Root,
                    (5, 9),
                ),
                AnnotatedToken::new(
                    TokenId::new(2),
                    "Mary".to_string(),
                    "mary".to_string(),
                    UPos::Propn,
                    DepRel::Iobj,
                    (10, 14),
                )
                .with_head(TokenId::new(1)),
                AnnotatedToken::new(
                    TokenId::new(3),
                    "a".to_string(),
                    "a".to_string(),
                    UPos::Det,
                    DepRel::Det,
                    (15, 16),
                )
                .with_head(TokenId::new(4)),
                AnnotatedToken::new(
                    TokenId::new(4),
                    "book".to_string(),
                    "book".to_string(),
                    UPos::Noun,
                    DepRel::Obj,
                    (17, 21),
                )
                .with_head(TokenId::new(1)),
            ],
        );

        let bindings = provider.bind_roles(&syntax, TokenId::new(1), None).unwrap();

        // Should have bindings for John (Agent), Mary (Recipient), book (Patient)
        assert!(!bindings.is_empty());

        // Check that we have an Agent binding
        let has_agent = bindings.iter().any(|b| b.role == ThetaRole::Agent);
        assert!(has_agent, "Should have Agent binding for subject");
    }
}
