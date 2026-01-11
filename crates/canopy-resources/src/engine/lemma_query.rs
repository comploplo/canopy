//! Lemma-based query interface for semantic engines.
//!
//! Provides a unified way to query engines using lemmatized forms
//! from the syntax provider, avoiding hacky string manipulation.

use canopy::core::{ThetaRole, UPos};
use canopy::kernel::events::LittleVType;
use canopy::runtime::TokenId;
use serde::{Deserialize, Serialize};

use super::error::EngineResult;

/// Standard input for lemma-based semantic queries.
///
/// Created from syntax provider output, containing the normalized
/// lemma form that should be used for all engine lookups.
#[derive(Debug, Clone, PartialEq)]
pub struct LemmaQuery {
    /// Lemma from syntax provider (already normalized).
    pub lemma: String,
    /// Part-of-speech tag from syntax.
    pub pos: UPos,
    /// Original surface form (for reference).
    pub form: Option<String>,
    /// Token ID if from an annotated syntax tree.
    pub token_id: Option<TokenId>,
}

impl LemmaQuery {
    /// Create a new lemma query.
    #[must_use]
    pub fn new(lemma: impl Into<String>, pos: UPos) -> Self {
        Self {
            lemma: lemma.into(),
            pos,
            form: None,
            token_id: None,
        }
    }

    /// Create a query for a verb.
    #[must_use]
    pub fn verb(lemma: impl Into<String>) -> Self {
        Self::new(lemma, UPos::Verb)
    }

    /// Create a query for a noun.
    #[must_use]
    pub fn noun(lemma: impl Into<String>) -> Self {
        Self::new(lemma, UPos::Noun)
    }

    /// Create a query for an adjective.
    #[must_use]
    pub fn adj(lemma: impl Into<String>) -> Self {
        Self::new(lemma, UPos::Adj)
    }

    /// Set the original surface form.
    #[must_use]
    pub fn with_form(mut self, form: impl Into<String>) -> Self {
        self.form = Some(form.into());
        self
    }

    /// Set the token ID.
    #[must_use]
    pub fn with_token_id(mut self, token_id: TokenId) -> Self {
        self.token_id = Some(token_id);
        self
    }

    /// Check if this is a verb query.
    #[must_use]
    pub fn is_verb(&self) -> bool {
        self.pos == UPos::Verb
    }

    /// Check if this is a noun query.
    #[must_use]
    pub fn is_noun(&self) -> bool {
        matches!(self.pos, UPos::Noun | UPos::Propn)
    }

    /// Check if this is an adjective query.
    #[must_use]
    pub fn is_adj(&self) -> bool {
        self.pos == UPos::Adj
    }
}

/// Identifies the semantic resource that provided evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceSource {
    /// `VerbNet` verb classes and thematic roles.
    VerbNet,
    /// `FrameNet` semantic frames and frame elements.
    FrameNet,
    /// `PropBank` predicate-argument structures.
    PropBank,
    /// `WordNet` synsets and semantic relations.
    WordNet,
    /// Lexicon closed-class word classification.
    Lexicon,
}

impl std::fmt::Display for ResourceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceSource::VerbNet => write!(f, "VerbNet"),
            ResourceSource::FrameNet => write!(f, "FrameNet"),
            ResourceSource::PropBank => write!(f, "PropBank"),
            ResourceSource::WordNet => write!(f, "WordNet"),
            ResourceSource::Lexicon => write!(f, "Lexicon"),
        }
    }
}

/// Semantic evidence from a resource query.
///
/// Represents a single piece of semantic information returned
/// by an engine, with calibrated confidence scores.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticEvidence {
    /// Which resource provided this evidence.
    pub source: ResourceSource,
    /// Identifier for this evidence (class ID, frame name, etc.).
    pub evidence_id: String,
    /// Confidence score after calibration (0.0-1.0).
    pub calibrated_confidence: f32,
    /// Theta roles associated with this evidence.
    pub theta_roles: Vec<ThetaRole>,
    /// `LittleV` type if applicable.
    pub little_v_type: Option<LittleVType>,
}

impl SemanticEvidence {
    /// Create new semantic evidence.
    #[must_use]
    pub fn new(source: ResourceSource, evidence_id: impl Into<String>) -> Self {
        Self {
            source,
            evidence_id: evidence_id.into(),
            calibrated_confidence: 0.5,
            theta_roles: Vec::new(),
            little_v_type: None,
        }
    }

    /// Set the calibrated confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.calibrated_confidence = confidence;
        self
    }

    /// Set theta roles.
    #[must_use]
    pub fn with_roles(mut self, roles: Vec<ThetaRole>) -> Self {
        self.theta_roles = roles;
        self
    }

    /// Set the `LittleV` type.
    #[must_use]
    pub fn with_little_v(mut self, little_v: LittleVType) -> Self {
        self.little_v_type = Some(little_v);
        self
    }

    /// Check if this is high-confidence evidence.
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        self.calibrated_confidence >= 0.7
    }
}

/// Trait for engines that can be queried by lemma.
///
/// Implementing this trait allows an engine to participate in
/// the multi-engine semantic analysis pipeline.
pub trait LemmaQueryable: Send + Sync {
    /// Query the engine with a lemma.
    ///
    /// Returns semantic evidence for the given lemma, or an empty
    /// vector if no information is available.
    ///
    /// # Errors
    /// Returns an error if the engine fails to process the query.
    fn query_by_lemma(&self, query: &LemmaQuery) -> EngineResult<Vec<SemanticEvidence>>;

    /// Get the resource source for this engine.
    fn resource_source(&self) -> ResourceSource;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lemma_query_creation() {
        let q = LemmaQuery::verb("run");
        assert_eq!(q.lemma, "run");
        assert_eq!(q.pos, UPos::Verb);
        assert!(q.is_verb());
        assert!(!q.is_noun());
    }

    #[test]
    fn test_lemma_query_with_form() {
        let q = LemmaQuery::verb("run").with_form("running");
        assert_eq!(q.form, Some("running".to_string()));
    }

    #[test]
    fn test_lemma_query_noun() {
        let q = LemmaQuery::noun("cat");
        assert!(q.is_noun());
        assert!(!q.is_verb());
    }

    #[test]
    fn test_lemma_query_adj() {
        let q = LemmaQuery::adj("happy");
        assert!(q.is_adj());
    }

    #[test]
    fn test_resource_source_display() {
        assert_eq!(ResourceSource::VerbNet.to_string(), "VerbNet");
        assert_eq!(ResourceSource::FrameNet.to_string(), "FrameNet");
        assert_eq!(ResourceSource::PropBank.to_string(), "PropBank");
        assert_eq!(ResourceSource::WordNet.to_string(), "WordNet");
        assert_eq!(ResourceSource::Lexicon.to_string(), "Lexicon");
    }

    #[test]
    fn test_semantic_evidence_creation() {
        let ev = SemanticEvidence::new(ResourceSource::VerbNet, "run-51.3.2");
        assert_eq!(ev.source, ResourceSource::VerbNet);
        assert_eq!(ev.evidence_id, "run-51.3.2");
        assert!((ev.calibrated_confidence - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_semantic_evidence_builder() {
        let ev = SemanticEvidence::new(ResourceSource::VerbNet, "give-13.1")
            .with_confidence(0.9)
            .with_roles(vec![
                ThetaRole::Agent,
                ThetaRole::Theme,
                ThetaRole::Recipient,
            ])
            .with_little_v(LittleVType::Cause);

        assert!((ev.calibrated_confidence - 0.9).abs() < f32::EPSILON);
        assert_eq!(ev.theta_roles.len(), 3);
        assert_eq!(ev.little_v_type, Some(LittleVType::Cause));
        assert!(ev.is_high_confidence());
    }

    #[test]
    fn test_semantic_evidence_low_confidence() {
        let ev = SemanticEvidence::new(ResourceSource::WordNet, "bank.n.01").with_confidence(0.4);
        assert!(!ev.is_high_confidence());
    }
}
