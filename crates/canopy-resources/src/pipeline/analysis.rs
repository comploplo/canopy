//! Analysis result types for the pipeline.

use canopy::kernel::discourse::Drs;
use canopy::kernel::events::ComposedEvents;
use canopy::runtime::{AnnotatedSyntax, PredicateDecomposition, RoleBinding};
use serde::{Deserialize, Serialize};

/// Result of semantic analysis for a single sentence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    /// The original sentence text.
    pub text: String,
    /// Annotated syntax (tokens with POS, dependencies).
    pub syntax: AnnotatedSyntax,
    /// Predicate decompositions (verb → `LittleV` structure).
    pub decompositions: Vec<PredicateDecomposition>,
    /// Role bindings (arguments → theta roles).
    pub role_bindings: Vec<RoleBinding>,
    /// Composed events for this sentence.
    pub events: Option<ComposedEvents>,
}

impl SemanticAnalysis {
    /// Create a new semantic analysis result.
    #[must_use]
    pub fn new(text: String, syntax: AnnotatedSyntax) -> Self {
        Self {
            text,
            syntax,
            decompositions: Vec::new(),
            role_bindings: Vec::new(),
            events: None,
        }
    }

    /// Add decompositions.
    #[must_use]
    pub fn with_decompositions(mut self, decompositions: Vec<PredicateDecomposition>) -> Self {
        self.decompositions = decompositions;
        self
    }

    /// Add role bindings.
    #[must_use]
    pub fn with_role_bindings(mut self, bindings: Vec<RoleBinding>) -> Self {
        self.role_bindings = bindings;
        self
    }

    /// Add composed events.
    #[must_use]
    pub fn with_events(mut self, events: ComposedEvents) -> Self {
        self.events = Some(events);
        self
    }

    /// Check if the sentence has any predicates.
    #[must_use]
    pub fn has_predicates(&self) -> bool {
        !self.decompositions.is_empty()
    }

    /// Get the number of events composed.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.as_ref().map_or(0, |e| e.events.len())
    }
}

/// Result of analyzing a multi-sentence document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAnalysis {
    /// Per-sentence analysis results.
    pub sentences: Vec<SemanticAnalysis>,
    /// Unified discourse representation (DRS).
    pub drs: Option<Drs>,
}

impl DocumentAnalysis {
    /// Create a new document analysis.
    #[must_use]
    pub fn new(sentences: Vec<SemanticAnalysis>) -> Self {
        Self {
            sentences,
            drs: None,
        }
    }

    /// Add discourse representation.
    #[must_use]
    pub fn with_drs(mut self, drs: Drs) -> Self {
        self.drs = Some(drs);
        self
    }

    /// Get total number of events across all sentences.
    pub fn total_events(&self) -> usize {
        self.sentences
            .iter()
            .map(SemanticAnalysis::event_count)
            .sum()
    }

    /// Get total number of sentences.
    #[must_use]
    pub fn sentence_count(&self) -> usize {
        self.sentences.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy::runtime::TokenId;
    use canopy::UPos;

    fn make_test_syntax() -> AnnotatedSyntax {
        use canopy::runtime::AnnotatedToken;
        use canopy::DepRel;

        let tokens = vec![AnnotatedToken::new(
            TokenId::new(0),
            "test".to_string(),
            "test".to_string(),
            UPos::Noun,
            DepRel::Root,
            (0, 4),
        )];
        AnnotatedSyntax::new("test".to_string(), tokens)
    }

    #[test]
    fn test_semantic_analysis_creation() {
        let syntax = make_test_syntax();
        let analysis = SemanticAnalysis::new("test".to_string(), syntax);

        assert_eq!(analysis.text, "test");
        assert!(!analysis.has_predicates());
        assert_eq!(analysis.event_count(), 0);
    }

    #[test]
    fn test_document_analysis() {
        let syntax = make_test_syntax();
        let sent1 = SemanticAnalysis::new("test".to_string(), syntax.clone());
        let sent2 = SemanticAnalysis::new("test2".to_string(), syntax);

        let doc = DocumentAnalysis::new(vec![sent1, sent2]);

        assert_eq!(doc.sentence_count(), 2);
        assert_eq!(doc.total_events(), 0);
    }
}
