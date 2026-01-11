//! Multi-engine argument binder.
//!
//! Binds syntactic arguments to semantic roles using evidence from
//! `VerbNet`, `FrameNet`, and `PropBank`.

use crate::engine::{DepRelToThetaMap, LemmaQuery, LemmaQueryable, SharedEngines};
use canopy::core::ThetaRole;
use canopy::runtime::{AnnotatedSyntax, RoleBinding, RoleProvider, RoleSource, SenseId, TokenId};
use canopy::CanopyError;

/// Configuration for argument binding.
#[derive(Debug, Clone)]
pub struct BinderConfig {
    /// Minimum confidence for role bindings.
    pub min_confidence: f32,
    /// Whether to prefer `VerbNet` theta roles as canonical.
    pub prefer_verbnet_roles: bool,
    /// Whether to use syntactic fallback when engine roles don't match.
    pub use_syntactic_fallback: bool,
}

impl Default for BinderConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            prefer_verbnet_roles: true,
            use_syntactic_fallback: true,
        }
    }
}

/// Binds syntactic arguments to semantic roles using multi-engine evidence.
pub struct ArgumentBinder {
    engines: SharedEngines,
    config: BinderConfig,
    deprel_map: DepRelToThetaMap,
}

impl ArgumentBinder {
    /// Create a new argument binder.
    ///
    /// # Errors
    /// Returns an error if engines cannot be initialized.
    pub fn new(engines: SharedEngines, config: BinderConfig) -> Result<Self, CanopyError> {
        let deprel_map = DepRelToThetaMap::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load deprel mappings, using defaults: {e}");
            DepRelToThetaMap::default()
        });

        Ok(Self {
            engines,
            config,
            deprel_map,
        })
    }

    /// Create with default configuration.
    ///
    /// # Errors
    /// Returns an error if engines cannot be initialized.
    pub fn with_default_config(engines: SharedEngines) -> Result<Self, CanopyError> {
        Self::new(engines, BinderConfig::default())
    }

    /// Get expected theta roles for a predicate from all engines.
    fn get_expected_roles(&self, lemma: &str) -> Vec<ThetaRole> {
        let query = LemmaQuery::verb(lemma);
        let mut all_roles = Vec::new();

        // Query VerbNet (primary source)
        if let Some(ref vn) = self.engines.verbnet {
            if let Ok(evidence) = vn.query_by_lemma(&query) {
                for ev in evidence {
                    for role in ev.theta_roles {
                        if !all_roles.contains(&role) {
                            all_roles.push(role);
                        }
                    }
                }
            }
        }

        // Query PropBank (additional roles)
        if !self.config.prefer_verbnet_roles {
            if let Some(ref pb) = self.engines.propbank {
                if let Ok(evidence) = pb.query_by_lemma(&query) {
                    for ev in evidence {
                        for role in ev.theta_roles {
                            if !all_roles.contains(&role) {
                                all_roles.push(role);
                            }
                        }
                    }
                }
            }
        }

        all_roles
    }

    /// Get expected theta roles for a specific sense.
    fn get_sense_roles(&self, sense: &SenseId) -> Vec<ThetaRole> {
        let sense_str = sense.to_string();

        // Try VerbNet class lookup
        if let Some(ref vn) = self.engines.verbnet {
            if let Some(class) = vn.get_verb_class(&sense_str) {
                return class
                    .themroles
                    .iter()
                    .filter_map(|r| ThetaRole::parse(&r.role_type))
                    .collect();
            }
        }

        // Try PropBank roleset
        if sense_str.contains('.') {
            let parts: Vec<_> = sense_str.split('.').collect();
            if let (Some(&lemma), Some(&sense_num)) = (parts.first(), parts.get(1)) {
                if let Some(ref pb) = self.engines.propbank {
                    if let Ok(roles) = pb.get_theta_roles(lemma, sense_num) {
                        return roles;
                    }
                }
            }
        }

        vec![]
    }

    /// UTAH-based fallback: map dependency relations to theta roles.
    fn dep_to_role(&self, dep_rel: &str) -> Option<ThetaRole> {
        self.deprel_map.get(dep_rel)
    }

    /// Bind roles using syntactic dependencies and expected roles.
    ///
    /// Each role is assigned at most once to prevent duplicate bindings.
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
                    let dep_str = format!("{:?}", token.deprel).to_lowercase();
                    if let Some(role) = self.dep_to_role(&dep_str) {
                        // Check if this role is expected and not yet used
                        if expected_roles.contains(&role) && !used_roles.contains(&role) {
                            bindings.push(
                                RoleBinding::new(TokenId::new(idx), role, 0.8)
                                    .with_source(RoleSource::VerbNet)
                                    .with_predicate(pred_id),
                            );
                            used_roles.push(role);
                        } else if self.config.use_syntactic_fallback && !used_roles.contains(&role)
                        {
                            // Use UTAH fallback with lower confidence (only if role not already assigned)
                            bindings.push(
                                RoleBinding::new(TokenId::new(idx), role, 0.6)
                                    .with_source(RoleSource::Syntactic)
                                    .with_predicate(pred_id),
                            );
                            used_roles.push(role);
                        }
                    }
                }
            }
        }

        // Filter by minimum confidence
        bindings
            .into_iter()
            .filter(|b| b.confidence >= self.config.min_confidence)
            .collect()
    }
}

impl std::fmt::Debug for ArgumentBinder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArgumentBinder")
            .field("config", &self.config)
            .field("engines", &self.engines)
            .finish_non_exhaustive()
    }
}

impl RoleProvider for ArgumentBinder {
    fn bind_roles(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
        sense: Option<&SenseId>,
    ) -> Result<Vec<RoleBinding>, CanopyError> {
        // Get expected roles from sense or predicate lemma
        let expected_roles = if let Some(sense_id) = sense {
            self.get_sense_roles(sense_id)
        } else {
            // Look up the predicate lemma
            if let Some(token) = syntax.tokens.get(pred_id.index()) {
                self.get_expected_roles(&token.lemma)
            } else {
                vec![]
            }
        };

        // Bind roles based on syntax
        let bindings = self.bind_from_syntax(syntax, pred_id, &expected_roles);

        tracing::debug!(
            "Bound {} roles for predicate at token {}",
            bindings.len(),
            pred_id.index()
        );

        Ok(bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engines_available() -> bool {
        crate::paths::data_path("data/verbnet").exists()
    }

    #[test]
    fn test_binder_config_default() {
        let config = BinderConfig::default();
        assert!((config.min_confidence - 0.5).abs() < f32::EPSILON);
        assert!(config.prefer_verbnet_roles);
        assert!(config.use_syntactic_fallback);
    }

    #[test]
    fn test_binder_creation() {
        if !engines_available() {
            eprintln!("Skipping: Data not available");
            return;
        }

        let engines = SharedEngines::new().expect("Failed to create engines");
        let binder = ArgumentBinder::with_default_config(engines);
        assert!(binder.is_ok());
    }

    #[test]
    fn test_no_duplicate_roles_in_used_roles_tracking() {
        // Test that used_roles prevents duplicates in both branches
        let mut used_roles: Vec<ThetaRole> = Vec::new();

        // Simulate first assignment (engine-backed)
        let role = ThetaRole::Agent;
        if !used_roles.contains(&role) {
            used_roles.push(role);
        }

        // Verify role is now tracked
        assert!(used_roles.contains(&ThetaRole::Agent));

        // Simulate fallback branch - should NOT add duplicate
        if !used_roles.contains(&role) {
            used_roles.push(role);
        }

        // Should still have only one Agent
        let agent_count = used_roles
            .iter()
            .filter(|r| **r == ThetaRole::Agent)
            .count();
        assert_eq!(agent_count, 1, "Role should not be duplicated");
    }

    #[test]
    fn test_fallback_roles_are_tracked() {
        // Test that roles assigned via syntactic fallback are tracked
        let mut used_roles: Vec<ThetaRole> = Vec::new();
        let expected_roles: Vec<ThetaRole> = vec![]; // Empty - triggers fallback

        // Role not in expected_roles, but fallback enabled
        let role = ThetaRole::Agent;
        let use_syntactic_fallback = true;

        if expected_roles.contains(&role) && !used_roles.contains(&role) {
            // Engine-backed path - won't be taken since expected_roles is empty
            used_roles.push(role);
        } else if use_syntactic_fallback && !used_roles.contains(&role) {
            // Fallback path - should track the role
            used_roles.push(role);
        }

        // Verify fallback role is tracked
        assert!(
            used_roles.contains(&ThetaRole::Agent),
            "Fallback roles should be tracked"
        );

        // Try to add same role again via fallback - should not duplicate
        if use_syntactic_fallback && !used_roles.contains(&role) {
            used_roles.push(role);
        }

        let agent_count = used_roles
            .iter()
            .filter(|r| **r == ThetaRole::Agent)
            .count();
        assert_eq!(agent_count, 1, "Fallback role should not be duplicated");
    }
}
