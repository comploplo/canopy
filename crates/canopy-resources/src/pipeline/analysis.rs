//! Analysis result types for the pipeline.

use canopy::kernel::discourse::{
    CoherenceClassification, Drs, MoveClassification, RelevanceReport, TrackedPresupposition,
    UnderspecDrs, ValidationReport,
};
use canopy::kernel::events::{ComposedEvents, PackedEvents};
use canopy::kernel::underspec::AmbiguitySummary;
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
    /// Optional discourse relevance report.
    pub relevance: Option<RelevanceReport>,
    /// Validation reports for events within this sentence.
    pub validations: Vec<ValidationReport>,
    /// Discourse move classification (assertion, question, etc.).
    pub discourse_move: Option<MoveClassification>,
    /// Coherence relation to the previous sentence.
    pub coherence: Option<CoherenceClassification>,
    /// Presuppositions triggered by this sentence.
    pub presuppositions: Vec<TrackedPresupposition>,
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
            relevance: None,
            validations: Vec::new(),
            discourse_move: None,
            coherence: None,
            presuppositions: Vec::new(),
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

    /// Add relevance report.
    #[must_use]
    pub fn with_relevance(mut self, relevance: RelevanceReport) -> Self {
        self.relevance = Some(relevance);
        self
    }

    /// Add validation reports.
    #[must_use]
    pub fn with_validations(mut self, validations: Vec<ValidationReport>) -> Self {
        self.validations = validations;
        self
    }

    /// Add discourse move classification.
    #[must_use]
    pub fn with_discourse_move(mut self, move_class: MoveClassification) -> Self {
        self.discourse_move = Some(move_class);
        self
    }

    /// Add coherence classification.
    #[must_use]
    pub fn with_coherence(mut self, coherence: CoherenceClassification) -> Self {
        self.coherence = Some(coherence);
        self
    }

    /// Add presuppositions.
    #[must_use]
    pub fn with_presuppositions(mut self, presuppositions: Vec<TrackedPresupposition>) -> Self {
        self.presuppositions = presuppositions;
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

/// Result of underspecified semantic analysis.
///
/// Preserves all readings rather than committing to a single interpretation.
/// Useful for:
/// - Interactive disambiguation (present options to users)
/// - Downstream processing that can handle ambiguity
/// - Research on ambiguity resolution
#[derive(Debug, Clone)]
pub struct UnderspecifiedAnalysis {
    /// The original sentence text.
    pub text: String,
    /// Annotated syntax (tokens with POS, dependencies).
    pub syntax: AnnotatedSyntax,
    /// Packed events preserving all sense readings.
    pub packed_events: Option<PackedEvents>,
    /// Underspecified DRS with scope ambiguity.
    pub underspec_drs: Option<UnderspecDrs>,
    /// Summary of ambiguity types present.
    pub ambiguity: AmbiguitySummary,
}

impl UnderspecifiedAnalysis {
    /// Create a new underspecified analysis.
    #[must_use]
    pub fn new(text: String, syntax: AnnotatedSyntax) -> Self {
        Self {
            text,
            syntax,
            packed_events: None,
            underspec_drs: None,
            ambiguity: AmbiguitySummary::default(),
        }
    }

    /// Add packed events.
    #[must_use]
    pub fn with_packed_events(mut self, events: PackedEvents) -> Self {
        // Update ambiguity summary from packed events
        self.ambiguity = events.ambiguity_summary();
        self.packed_events = Some(events);
        self
    }

    /// Add underspecified DRS.
    #[must_use]
    pub fn with_underspec_drs(mut self, drs: UnderspecDrs) -> Self {
        self.underspec_drs = Some(drs);
        self
    }

    /// Check if the analysis is ambiguous.
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        self.ambiguity.is_ambiguous()
    }

    /// Get the total number of readings.
    #[must_use]
    pub fn reading_count(&self) -> usize {
        self.ambiguity.total_readings.max(1)
    }

    /// Convert to a resolved `SemanticAnalysis` by selecting the best reading.
    ///
    /// Uses the default reading from packed events.
    #[must_use]
    pub fn to_resolved(&self) -> SemanticAnalysis {
        let mut analysis = SemanticAnalysis::new(self.text.clone(), self.syntax.clone());

        if let Some(ref packed) = self.packed_events {
            let composed = packed.best_reading();
            if !composed.events.is_empty() {
                analysis = analysis.with_events(composed);
            }
        }

        analysis
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
