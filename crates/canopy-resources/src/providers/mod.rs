//! Provider trait implementations for canopy kernel.
//!
//! This module implements the provider traits from `canopy::runtime` using
//! multi-engine evidence aggregation from `VerbNet`, `FrameNet`, and `PropBank`.
//!
//! # Architecture
//!
//! The kernel defines traits; this module provides implementations:
//!
//! ```text
//! canopy::runtime        canopy-resources::providers
//! ---------------        ---------------------------
//! SenseProvider    <---- PredicateDecomposer (multi-engine)
//! RoleProvider     <---- ArgumentBinder (multi-engine)
//! DiscourseCue     <---- LexiconDiscourseCueProvider
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use canopy_resources::providers::PredicateDecomposer;
//! use canopy::runtime::SenseProvider;
//!
//! let engines = SharedEngines::new()?;
//! let provider = PredicateDecomposer::with_default_config(engines)?;
//! let decompositions = provider.decompose_predicate(&syntax, pred_id)?;
//! ```

mod argument_binder;
mod discourse_cue;
mod predicate_decomposer;

pub use argument_binder::{ArgumentBinder, BinderConfig};
pub use discourse_cue::LexiconDiscourseCueProvider;
pub use predicate_decomposer::{DecomposerConfig, PredicateDecomposer};

// Combined provider that implements all traits
pub use combined::DefaultProvider;

mod combined {
    use super::{
        ArgumentBinder, BinderConfig, DecomposerConfig, LexiconDiscourseCueProvider,
        PredicateDecomposer,
    };
    use crate::engine::SharedEngines;
    use canopy::runtime::{
        AnnotatedSyntax, DiscourseCueProvider, DiscourseRelation, FrameId, PredicateDecomposition,
        RoleBinding, RoleProvider, SenseId, SenseInfo, SenseProvider, SyntaxProvider, TokenId,
    };
    use canopy::CanopyError;

    /// Default provider combining multi-engine providers.
    ///
    /// This provider implements all four provider traits required by the kernel:
    /// - `SyntaxProvider` - Stub (requires external parser)
    /// - `SenseProvider` - Uses multi-engine `PredicateDecomposer`
    /// - `RoleProvider` - Uses multi-engine `ArgumentBinder`
    /// - `DiscourseCueProvider` - Uses Lexicon for discourse connectives
    pub struct DefaultProvider {
        sense: PredicateDecomposer,
        role: ArgumentBinder,
        discourse: LexiconDiscourseCueProvider,
    }

    impl DefaultProvider {
        /// Create a new default provider with all engines loaded.
        ///
        /// # Errors
        /// Returns an error if any provider cannot be initialized.
        pub fn new() -> Result<Self, CanopyError> {
            let engines = SharedEngines::new()?;

            Ok(Self {
                sense: PredicateDecomposer::new(engines.clone(), DecomposerConfig::default())?,
                role: ArgumentBinder::new(engines, BinderConfig::default())?,
                discourse: LexiconDiscourseCueProvider::new()?,
            })
        }
    }

    impl std::fmt::Debug for DefaultProvider {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DefaultProvider")
                .field("sense", &"PredicateDecomposer")
                .field("role", &"ArgumentBinder")
                .field("discourse", &"LexiconDiscourseCueProvider")
                .finish()
        }
    }

    impl SyntaxProvider for DefaultProvider {
        fn parse(&self, _text: &str) -> Result<AnnotatedSyntax, CanopyError> {
            // SyntaxProvider requires an actual parser (UDPipe, spaCy, etc.)
            // DefaultProvider delegates parsing to external tools (orchestrator uses
            // pre-parsed syntax from the pipeline). Direct calls should fail explicitly.
            Err(CanopyError::config(
                "DefaultProvider does not include a parser. Use the orchestrator pipeline \
                 with pre-parsed syntax, or provide an AnnotatedSyntax directly.",
            ))
        }
    }

    impl SenseProvider for DefaultProvider {
        fn decompose_predicate(
            &self,
            syntax: &AnnotatedSyntax,
            pred_id: TokenId,
        ) -> Result<Vec<PredicateDecomposition>, CanopyError> {
            self.sense.decompose_predicate(syntax, pred_id)
        }

        fn frames_for_sense(&self, sense: &SenseId) -> Result<Vec<FrameId>, CanopyError> {
            self.sense.frames_for_sense(sense)
        }

        fn get_sense(&self, id: &SenseId) -> Result<Option<SenseInfo>, CanopyError> {
            self.sense.get_sense(id)
        }
    }

    impl RoleProvider for DefaultProvider {
        fn bind_roles(
            &self,
            syntax: &AnnotatedSyntax,
            pred_id: TokenId,
            sense: Option<&SenseId>,
        ) -> Result<Vec<RoleBinding>, CanopyError> {
            self.role.bind_roles(syntax, pred_id, sense)
        }
    }

    impl DiscourseCueProvider for DefaultProvider {
        fn is_discourse_connective(&self, syntax: &AnnotatedSyntax, token_id: TokenId) -> bool {
            self.discourse.is_discourse_connective(syntax, token_id)
        }

        fn discourse_relation(
            &self,
            syntax: &AnnotatedSyntax,
            token_id: TokenId,
        ) -> Option<DiscourseRelation> {
            self.discourse.discourse_relation(syntax, token_id)
        }
    }

    // Blanket implementation from trait bounds gives us CanopyProvider automatically

    #[cfg(test)]
    mod tests {
        use super::*;
        use canopy::runtime::{AnnotatedToken, CanopyProvider};
        use canopy::{DepRel, UPos};

        fn verbnet_available() -> bool {
            crate::paths::data_path("data/verbnet").exists()
        }

        #[test]
        fn test_default_provider_creation() {
            if !verbnet_available() {
                eprintln!("Skipping: VerbNet data not available");
                return;
            }

            let provider = DefaultProvider::new();
            assert!(provider.is_ok());
        }

        #[test]
        fn test_default_provider_is_canopy_provider() {
            // Compile-time check that DefaultProvider implements CanopyProvider
            fn takes_canopy_provider<P: CanopyProvider>(_p: &P) {}

            if !verbnet_available() {
                eprintln!("Skipping: VerbNet data not available");
                return;
            }

            let provider = DefaultProvider::new().unwrap();
            takes_canopy_provider(&provider);
        }

        #[test]
        fn test_syntax_provider_parse_returns_error() {
            if !verbnet_available() {
                eprintln!("Skipping: VerbNet data not available");
                return;
            }

            let provider = DefaultProvider::new().unwrap();
            let result = provider.parse("Hello world");

            // DefaultProvider does not include a parser - should return error
            assert!(
                result.is_err(),
                "DefaultProvider.parse() should return configuration error"
            );
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("does not include a parser"),
                "Error message should explain parser is not available"
            );
        }

        #[test]
        fn test_sense_provider_delegation() {
            if !verbnet_available() {
                eprintln!("Skipping: VerbNet data not available");
                return;
            }

            let provider = DefaultProvider::new().unwrap();

            // Test get_sense
            let sense = provider.get_sense(&SenseId::new("give-13.1"));
            assert!(sense.is_ok());

            // Test frames_for_sense
            let frames = provider.frames_for_sense(&SenseId::new("give-13.1"));
            assert!(frames.is_ok());
        }

        #[test]
        fn test_role_provider_delegation() {
            if !verbnet_available() {
                eprintln!("Skipping: VerbNet data not available");
                return;
            }

            let provider = DefaultProvider::new().unwrap();

            let syntax = AnnotatedSyntax::new(
                "John runs".to_string(),
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
                        "runs".to_string(),
                        "run".to_string(),
                        UPos::Verb,
                        DepRel::Root,
                        (5, 9),
                    ),
                ],
            );

            let bindings = provider.bind_roles(&syntax, TokenId::new(1), None);
            assert!(bindings.is_ok());
        }

        #[test]
        fn test_discourse_cue_provider_delegation() {
            if !verbnet_available() {
                eprintln!("Skipping: VerbNet data not available");
                return;
            }

            let provider = DefaultProvider::new().unwrap();

            let syntax = AnnotatedSyntax::new(
                "However".to_string(),
                vec![AnnotatedToken::new(
                    TokenId::new(0),
                    "However".to_string(),
                    "however".to_string(),
                    UPos::Adv,
                    DepRel::Advmod,
                    (0, 7),
                )],
            );

            assert!(provider.is_discourse_connective(&syntax, TokenId::new(0)));
            assert_eq!(
                provider.discourse_relation(&syntax, TokenId::new(0)),
                Some(DiscourseRelation::Contrast)
            );
        }

        #[test]
        fn test_decompose_predicate_delegation() {
            if !verbnet_available() {
                eprintln!("Skipping: VerbNet data not available");
                return;
            }

            let provider = DefaultProvider::new().unwrap();

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

            let decompositions = provider.decompose_predicate(&syntax, TokenId::new(0));
            assert!(decompositions.is_ok());
        }
    }
}
