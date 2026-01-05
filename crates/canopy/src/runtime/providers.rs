//! Provider traits for dependency injection.
//!
//! These traits define the boundary between the semantic kernel and external resources.
//! The kernel can only access `VerbNet`, `FrameNet`, `WordNet`, and parsing through these traits,
//! ensuring clean separation and testability.
//!
//! # Design Principles
//!
//! 1. **Fine-grained traits**: Each trait handles one concern, making mocking easy.
//! 2. **Sync over async**: The kernel is CPU-bound; async would add complexity without benefit.
//! 3. **Scores are optional but future-proof**: All lookups return confidence scores.
//! 4. **Send + Sync**: All providers must be thread-safe for parallel processing.
//!
//! # Example
//!
//! ```rust,ignore
//! use canopy::runtime::{SyntaxProvider, AnnotatedSyntax, CanopyError};
//!
//! struct MockParser;
//!
//! impl SyntaxProvider for MockParser {
//!     fn parse(&self, text: &str) -> Result<AnnotatedSyntax, CanopyError> {
//!         // Return mock parse for testing
//!         Ok(AnnotatedSyntax::new(text.to_string(), vec![]))
//!     }
//! }
//! ```

use super::ids::{FrameId, SenseId, TokenId};
use super::ir::AnnotatedSyntax;
use crate::core::{CanopyError, ThetaRole};
use crate::kernel::events::LittleVType;
use serde::{Deserialize, Serialize};

// =============================================================================
// Predicate Decomposition (returned by SenseProvider)
// =============================================================================

/// Structured return type for predicate decomposition.
///
/// The kernel receives pre-decomposed event structures from the `SenseProvider`,
/// keeping word-level knowledge (`VerbNet`, `FrameNet` mappings) in the resources layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredicateDecomposition {
    /// The sense identifier (`VerbNet` class, `FrameNet` frame, etc.)
    pub sense_id: SenseId,

    /// The primary `LittleV` type (Cause, Become, Do, etc.)
    pub little_v_type: LittleVType,

    /// Expected theta roles for this decomposition.
    pub expected_roles: Vec<ThetaRole>,

    /// Optional sub-event (e.g., Cause contains Become).
    pub sub_event: Option<Box<PredicateDecomposition>>,

    /// Confidence in this decomposition (0.0 to 1.0).
    pub confidence: f32,

    /// Source of this decomposition.
    pub source: DecompositionSource,

    /// The predicate token this decomposition applies to.
    /// Used to associate decompositions with their source predicate in multi-predicate sentences.
    pub token_id: Option<TokenId>,
}

impl PredicateDecomposition {
    /// Create a new predicate decomposition.
    #[must_use]
    pub fn new(
        sense_id: SenseId,
        little_v_type: LittleVType,
        expected_roles: Vec<ThetaRole>,
    ) -> Self {
        Self {
            sense_id,
            little_v_type,
            expected_roles,
            sub_event: None,
            confidence: 1.0,
            source: DecompositionSource::VerbNet,
            token_id: None,
        }
    }

    /// Set the predicate token ID this decomposition applies to.
    #[must_use]
    pub fn with_token_id(mut self, token_id: TokenId) -> Self {
        self.token_id = Some(token_id);
        self
    }

    /// Add a sub-event.
    #[must_use]
    pub fn with_sub_event(mut self, sub: PredicateDecomposition) -> Self {
        self.sub_event = Some(Box::new(sub));
        self
    }

    /// Set confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Set source.
    #[must_use]
    pub fn with_source(mut self, source: DecompositionSource) -> Self {
        self.source = source;
        self
    }
}

/// Source of a predicate decomposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DecompositionSource {
    /// Decomposed from `VerbNet` verb class.
    #[default]
    VerbNet,

    /// Decomposed from `FrameNet` frame.
    FrameNet,

    /// Decomposed from `PropBank` frameset.
    PropBank,

    /// Heuristic-based decomposition.
    Heuristic,

    /// Custom or hybrid decomposition.
    Custom,
}

// =============================================================================
// Core Provider Traits
// =============================================================================

/// Provides syntactic parsing: text -> annotated syntax tree.
///
/// Implementations may use:
/// - `UDPipe` (UD format)
/// - spaCy (via FFI)
/// - Custom rule-based parsers
/// - Mock parsers for testing
pub trait SyntaxProvider: Send + Sync {
    /// Parse text into annotated syntax.
    ///
    /// Returns an `AnnotatedSyntax` containing tokens with:
    /// - Lemmas
    /// - Universal POS tags
    /// - Morphological features
    /// - Dependency relations
    ///
    /// # Errors
    /// Returns an error if parsing fails.
    fn parse(&self, text: &str) -> Result<AnnotatedSyntax, CanopyError>;

    /// Parse multiple texts (batch optimization).
    ///
    /// Default implementation calls `parse` for each text.
    /// Implementations may override for better performance.
    ///
    /// # Errors
    /// Returns an error if any text fails to parse.
    fn parse_batch(&self, texts: &[&str]) -> Result<Vec<AnnotatedSyntax>, CanopyError> {
        texts.iter().map(|t| self.parse(t)).collect()
    }
}

/// Provides predicate decomposition: predicate -> `LittleV` structure.
///
/// This is the primary interface for the kernel to get event decompositions.
/// Implementations handle all word-level knowledge (`VerbNet`, `FrameNet` mappings).
pub trait SenseProvider: Send + Sync {
    /// Decompose a predicate into `LittleV` structure.
    ///
    /// This is the main method. Returns fully decomposed event structures
    /// including `LittleVType`, expected theta roles, and sub-events.
    ///
    /// # Arguments
    /// * `syntax` - The full annotated syntax (for context)
    /// * `pred_id` - The token ID of the predicate
    ///
    /// # Returns
    /// A list of decompositions, sorted by confidence descending.
    ///
    /// # Errors
    /// Returns an error if predicate decomposition fails.
    fn decompose_predicate(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
    ) -> Result<Vec<PredicateDecomposition>, CanopyError>;

    /// Get candidate senses for a predicate (legacy method).
    ///
    /// Prefer `decompose_predicate` for new code.
    ///
    /// # Errors
    /// Returns an error if sense extraction fails.
    fn predicate_senses(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
    ) -> Result<Vec<(SenseId, f32)>, CanopyError> {
        // Default: extract sense IDs from decompositions
        let decomps = self.decompose_predicate(syntax, pred_id)?;
        Ok(decomps
            .into_iter()
            .map(|d| (d.sense_id, d.confidence))
            .collect())
    }

    /// Get frames associated with a sense.
    ///
    /// # Errors
    /// Returns an error if frame lookup fails.
    fn frames_for_sense(&self, sense: &SenseId) -> Result<Vec<FrameId>, CanopyError>;

    /// Get sense by ID directly (for lookups without context).
    ///
    /// # Errors
    /// Returns an error if sense lookup fails.
    fn get_sense(&self, id: &SenseId) -> Result<Option<SenseInfo>, CanopyError>;
}

/// Information about a word sense.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenseInfo {
    /// The sense identifier.
    pub id: SenseId,

    /// Human-readable description.
    pub description: String,

    /// Source resource (`VerbNet`, `FrameNet`, `WordNet`).
    pub source: SenseSource,

    /// Associated theta roles for this sense.
    pub theta_roles: Vec<ThetaRole>,
}

/// Source of a word sense.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SenseSource {
    VerbNet,
    FrameNet,
    WordNet,
    PropBank,
    Custom,
}

/// Provides thematic role assignment: predicate + sense -> role bindings.
///
/// Maps syntactic arguments to semantic roles based on:
/// - `VerbNet` verb class patterns
/// - `FrameNet` frame element mappings
/// - Syntactic heuristics (subject -> Agent, etc.)
pub trait RoleProvider: Send + Sync {
    /// Bind thematic roles to syntactic arguments.
    ///
    /// # Arguments
    /// * `syntax` - The full annotated syntax
    /// * `pred_id` - The predicate token
    /// * `sense` - Optional sense to condition role assignment
    ///
    /// # Returns
    /// Role bindings linking tokens to thematic roles.
    ///
    /// # Errors
    /// Returns an error if role binding fails.
    fn bind_roles(
        &self,
        syntax: &AnnotatedSyntax,
        pred_id: TokenId,
        sense: Option<&SenseId>,
    ) -> Result<Vec<RoleBinding>, CanopyError>;
}

/// A binding between a token and a thematic role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    /// The token being assigned a role.
    pub token_id: TokenId,

    /// The assigned thematic role.
    pub role: ThetaRole,

    /// Confidence in this assignment (0.0 to 1.0).
    pub confidence: f32,

    /// Source of the role assignment.
    pub source: RoleSource,

    /// The predicate token this binding is associated with.
    /// Used to associate role bindings with their predicate in multi-predicate sentences.
    pub predicate_token_id: Option<TokenId>,
}

impl RoleBinding {
    /// Create a new role binding.
    #[must_use]
    pub fn new(token_id: TokenId, role: ThetaRole, confidence: f32) -> Self {
        Self {
            token_id,
            role,
            confidence,
            source: RoleSource::Syntactic,
            predicate_token_id: None,
        }
    }

    /// Create with a specific source.
    #[must_use]
    pub fn with_source(mut self, source: RoleSource) -> Self {
        self.source = source;
        self
    }

    /// Set the predicate token this binding is associated with.
    #[must_use]
    pub fn with_predicate(mut self, predicate_token_id: TokenId) -> Self {
        self.predicate_token_id = Some(predicate_token_id);
        self
    }
}

/// Source of a role assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoleSource {
    /// Assigned based on syntactic position (subject -> Agent).
    Syntactic,

    /// Assigned from `VerbNet` verb class.
    VerbNet,

    /// Assigned from `FrameNet` frame elements.
    FrameNet,

    /// Assigned from `PropBank` framesets.
    PropBank,

    /// Custom or hybrid assignment.
    Custom,
}

/// Identifies discourse connectives and cues.
///
/// Used for discourse-level processing (QUD, discourse moves).
pub trait DiscourseCueProvider: Send + Sync {
    /// Check if a token is a discourse connective.
    ///
    /// Examples: "however", "therefore", "because", "although"
    fn is_discourse_connective(&self, syntax: &AnnotatedSyntax, token_id: TokenId) -> bool;

    /// Get the discourse relation signaled by a connective.
    ///
    /// Returns None if the token is not a connective.
    fn discourse_relation(
        &self,
        syntax: &AnnotatedSyntax,
        token_id: TokenId,
    ) -> Option<DiscourseRelation>;
}

/// Types of discourse relations (simplified PDTB-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscourseRelation {
    /// Cause: "because", "since", "as a result"
    Cause,

    /// Contrast: "but", "however", "although"
    Contrast,

    /// Elaboration: "specifically", "in particular"
    Elaboration,

    /// Temporal: "then", "after", "before"
    Temporal,

    /// Condition: "if", "unless", "provided that"
    Condition,

    /// Concession: "although", "even though"
    Concession,

    /// Addition: "and", "also", "moreover"
    Addition,
}

// =============================================================================
// Supertrait for Full Provider
// =============================================================================

/// Combined provider trait for production use.
///
/// Implementations that provide all capabilities can implement this
/// supertrait for ergonomic use in the kernel.
pub trait CanopyProvider:
    SyntaxProvider + SenseProvider + RoleProvider + DiscourseCueProvider
{
}

// Blanket implementation: any type implementing all traits gets CanopyProvider
impl<T> CanopyProvider for T where
    T: SyntaxProvider + SenseProvider + RoleProvider + DiscourseCueProvider
{
}

// =============================================================================
// Default/Stub Implementations for Testing
// =============================================================================

/// A stub provider that returns empty results.
///
/// Useful for testing the kernel in isolation.
/// **WARNING**: This is for testing only - not for production use.
#[derive(Debug, Clone, Default)]
pub struct StubProvider;

impl SyntaxProvider for StubProvider {
    fn parse(&self, text: &str) -> Result<AnnotatedSyntax, CanopyError> {
        // Return empty syntax for testing
        Ok(AnnotatedSyntax::new(text.to_string(), vec![]))
    }
}

impl SenseProvider for StubProvider {
    fn decompose_predicate(
        &self,
        _syntax: &AnnotatedSyntax,
        _pred_id: TokenId,
    ) -> Result<Vec<PredicateDecomposition>, CanopyError> {
        Ok(vec![])
    }

    fn frames_for_sense(&self, _sense: &SenseId) -> Result<Vec<FrameId>, CanopyError> {
        Ok(vec![])
    }

    fn get_sense(&self, _id: &SenseId) -> Result<Option<SenseInfo>, CanopyError> {
        Ok(None)
    }
}

impl RoleProvider for StubProvider {
    fn bind_roles(
        &self,
        _syntax: &AnnotatedSyntax,
        _pred_id: TokenId,
        _sense: Option<&SenseId>,
    ) -> Result<Vec<RoleBinding>, CanopyError> {
        Ok(vec![])
    }
}

impl DiscourseCueProvider for StubProvider {
    fn is_discourse_connective(&self, _syntax: &AnnotatedSyntax, _token_id: TokenId) -> bool {
        false
    }

    fn discourse_relation(
        &self,
        _syntax: &AnnotatedSyntax,
        _token_id: TokenId,
    ) -> Option<DiscourseRelation> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_provider_implements_all_traits() {
        let provider = StubProvider;

        // Test SyntaxProvider
        let syntax = provider.parse("test").unwrap();
        assert_eq!(syntax.text, "test");
        assert!(syntax.tokens.is_empty());

        // Test SenseProvider
        let decomps = provider
            .decompose_predicate(&syntax, TokenId::new(0))
            .unwrap();
        assert!(decomps.is_empty());

        let frames = provider.frames_for_sense(&SenseId::new("test")).unwrap();
        assert!(frames.is_empty());

        // Test RoleProvider
        let roles = provider.bind_roles(&syntax, TokenId::new(0), None).unwrap();
        assert!(roles.is_empty());

        // Test DiscourseCueProvider
        assert!(!provider.is_discourse_connective(&syntax, TokenId::new(0)));
        assert!(provider
            .discourse_relation(&syntax, TokenId::new(0))
            .is_none());
    }

    #[test]
    fn test_stub_provider_is_canopy_provider() {
        // Compile-time check that StubProvider implements CanopyProvider
        fn takes_canopy_provider<P: CanopyProvider>(_p: &P) {}
        let provider = StubProvider;
        takes_canopy_provider(&provider);
    }

    #[test]
    fn test_role_binding() {
        let binding = RoleBinding::new(TokenId::new(0), ThetaRole::Agent, 0.95)
            .with_source(RoleSource::VerbNet);

        assert_eq!(binding.token_id.index(), 0);
        assert_eq!(binding.role, ThetaRole::Agent);
        assert!((binding.confidence - 0.95).abs() < f32::EPSILON);
        assert_eq!(binding.source, RoleSource::VerbNet);
    }

    #[test]
    fn test_sense_info() {
        let info = SenseInfo {
            id: SenseId::new("give-13.1"),
            description: "Transfer of possession".to_string(),
            source: SenseSource::VerbNet,
            theta_roles: vec![ThetaRole::Agent, ThetaRole::Theme, ThetaRole::Recipient],
        };

        assert_eq!(info.id.as_str(), "give-13.1");
        assert_eq!(info.source, SenseSource::VerbNet);
        assert_eq!(info.theta_roles.len(), 3);
    }

    #[test]
    fn test_predicate_decomposition() {
        let decomp = PredicateDecomposition::new(
            SenseId::new("break-45.1"),
            LittleVType::Cause,
            vec![ThetaRole::Agent, ThetaRole::Patient],
        )
        .with_confidence(0.9)
        .with_source(DecompositionSource::VerbNet)
        .with_sub_event(PredicateDecomposition::new(
            SenseId::new("break-45.1-become"),
            LittleVType::Become,
            vec![ThetaRole::Patient],
        ));

        assert_eq!(decomp.sense_id.as_str(), "break-45.1");
        assert_eq!(decomp.little_v_type, LittleVType::Cause);
        assert_eq!(decomp.expected_roles.len(), 2);
        assert!((decomp.confidence - 0.9).abs() < f32::EPSILON);
        assert!(decomp.sub_event.is_some());

        let sub = decomp.sub_event.as_ref().unwrap();
        assert_eq!(sub.little_v_type, LittleVType::Become);
    }

    #[test]
    fn test_discourse_relation_variants() {
        // Ensure all variants are distinct
        let relations = [
            DiscourseRelation::Cause,
            DiscourseRelation::Contrast,
            DiscourseRelation::Elaboration,
            DiscourseRelation::Temporal,
            DiscourseRelation::Condition,
            DiscourseRelation::Concession,
            DiscourseRelation::Addition,
        ];

        for (i, r1) in relations.iter().enumerate() {
            for (j, r2) in relations.iter().enumerate() {
                if i == j {
                    assert_eq!(r1, r2);
                } else {
                    assert_ne!(r1, r2);
                }
            }
        }
    }

    #[test]
    fn test_parse_batch_default() {
        let provider = StubProvider;
        let texts = ["hello", "world"];
        let results = provider.parse_batch(&texts).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].text, "hello");
        assert_eq!(results[1].text, "world");
    }
}
