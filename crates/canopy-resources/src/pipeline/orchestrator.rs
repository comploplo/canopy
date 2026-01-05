//! Main pipeline orchestrator.
//!
//! `CanopyPipeline` coordinates all components for end-to-end analysis.

use super::analysis::{DocumentAnalysis, SemanticAnalysis, UnderspecifiedAnalysis};
use super::config::PipelineConfig;
use crate::engine::SharedEngines;
use crate::providers::{VerbNetRoleProvider, VerbNetSenseProvider};
use crate::syntax::{TreebankConfig, TreebankSyntaxProvider};
use crate::tokenizer::{SimpleTokenizer, Tokenizer};
use canopy::kernel::discourse::{DiscourseConfig, DiscourseContext, UnderspecDrs};
use canopy::kernel::events::{EventComposer, EventComposerConfig, SentenceAnalysis};
use canopy::kernel::underspec::{DisambiguationContext, Disambiguator};
use canopy::runtime::{
    AnnotatedSyntax, PredicateDecomposition, RoleBinding, RoleProvider, SenseProvider,
    SyntaxProvider, TokenId,
};
use canopy::CanopyError;
use std::collections::HashMap;

/// The main pipeline for semantic analysis.
///
/// Orchestrates the full flow from text to discourse representation.
pub struct CanopyPipeline {
    /// Tokenizer for sentence splitting.
    tokenizer: SimpleTokenizer,
    /// Syntax provider for parsing.
    syntax_provider: TreebankSyntaxProvider,
    /// Sense provider for predicate decomposition.
    sense_provider: VerbNetSenseProvider,
    /// Role provider for theta role assignment.
    role_provider: VerbNetRoleProvider,
    /// Event composer.
    event_composer: EventComposer,
    /// Pipeline configuration.
    config: PipelineConfig,
}

impl CanopyPipeline {
    /// Create a new pipeline with default configuration.
    ///
    /// # Errors
    /// Returns an error if pipeline components cannot be initialized.
    pub fn new() -> Result<Self, CanopyError> {
        Self::with_config(PipelineConfig::default())
    }

    /// Create a pipeline with custom configuration.
    ///
    /// # Errors
    /// Returns an error if pipeline components cannot be initialized.
    pub fn with_config(config: PipelineConfig) -> Result<Self, CanopyError> {
        // Create engines ONCE and share across all components
        let engines = SharedEngines::new()?;

        let tokenizer = SimpleTokenizer::from_ewt().unwrap_or_else(|_| SimpleTokenizer::new());

        // Use shared engines for syntax provider
        let syntax_provider =
            TreebankSyntaxProvider::with_shared_engines(TreebankConfig::default(), &engines)?;

        // Use shared VerbNet engine for sense and role providers
        let sense_provider = if let Some(ref verbnet) = engines.verbnet {
            VerbNetSenseProvider::with_engine(verbnet.clone())
        } else {
            VerbNetSenseProvider::new().map_err(|e| {
                CanopyError::data_load(format!("Failed to create sense provider: {e}"))
            })?
        };

        let role_provider = if let Some(ref verbnet) = engines.verbnet {
            VerbNetRoleProvider::with_engine(verbnet.clone())
        } else {
            VerbNetRoleProvider::new().map_err(|e| {
                CanopyError::data_load(format!("Failed to create role provider: {e}"))
            })?
        };

        let event_composer = EventComposer::new(EventComposerConfig::default());

        Ok(Self {
            tokenizer,
            syntax_provider,
            sense_provider,
            role_provider,
            event_composer,
            config,
        })
    }

    /// Analyze a single sentence.
    ///
    /// # Errors
    /// Returns an error if syntax parsing or semantic analysis fails.
    pub fn analyze(&self, text: &str) -> Result<SemanticAnalysis, CanopyError> {
        // 1. Parse syntax
        let syntax = self.syntax_provider.parse(text)?;

        // 2. Find predicates and decompose them
        let decompositions = self.decompose_predicates(&syntax)?;

        // 3. Bind theta roles
        let role_bindings = self.bind_roles(&syntax, &decompositions)?;

        // 4. Compose events (if we have decompositions)
        let events = if decompositions.is_empty() {
            None
        } else {
            let sentence_analysis = SentenceAnalysis::new(text.to_string(), syntax.clone());

            // Convert Vec to HashMap indexed by predicate token ID
            let decomp_map = Self::decomps_to_map(&syntax, &decompositions);
            let binding_map = Self::bindings_to_map(&syntax, &role_bindings);

            self.event_composer
                .compose(&sentence_analysis, &decomp_map, &binding_map)
                .ok()
        };

        let mut analysis = SemanticAnalysis::new(text.to_string(), syntax)
            .with_decompositions(decompositions)
            .with_role_bindings(role_bindings);

        if let Some(events) = events {
            analysis = analysis.with_events(events);
        }

        Ok(analysis)
    }

    /// Analyze a multi-sentence document.
    ///
    /// # Errors
    /// Returns an error if any sentence analysis fails.
    pub fn analyze_document(&self, text: &str) -> Result<DocumentAnalysis, CanopyError> {
        // Split into sentences
        let sentences = self.tokenizer.split_sentences(text);

        let max_sents = self.config.max_sentences.unwrap_or(usize::MAX);
        let sentences = sentences.into_iter().take(max_sents);

        // Analyze each sentence
        let mut sentence_results = Vec::new();
        for sentence in sentences {
            let result = self.analyze(&sentence)?;
            sentence_results.push(result);
        }

        // Build discourse representation if enabled
        let drs = if self.config.enable_discourse {
            let mut ctx = DiscourseContext::new(DiscourseConfig::default());

            for result in &sentence_results {
                if let Some(events) = &result.events {
                    ctx.process_events(events);
                }
            }

            Some(ctx.drs().clone())
        } else {
            None
        };

        let mut doc = DocumentAnalysis::new(sentence_results);
        if let Some(drs) = drs {
            doc = doc.with_drs(drs);
        }

        Ok(doc)
    }

    /// Analyze a sentence preserving all readings (underspecified).
    ///
    /// Unlike `analyze()`, this method does not select a single reading,
    /// but instead preserves all possible interpretations.
    ///
    /// # Errors
    /// Returns an error if syntax parsing fails.
    pub fn analyze_underspecified(
        &self,
        text: &str,
    ) -> Result<UnderspecifiedAnalysis, CanopyError> {
        // 1. Parse syntax
        let syntax = self.syntax_provider.parse(text)?;

        // 2. Find predicates and decompose them (keeping all senses)
        let decompositions = self.decompose_predicates(&syntax)?;

        // 3. Bind theta roles
        let role_bindings = self.bind_roles(&syntax, &decompositions)?;

        // 4. Compose packed events (preserving all readings)
        let packed_events = if decompositions.is_empty() {
            None
        } else {
            let sentence_analysis = SentenceAnalysis::new(text.to_string(), syntax.clone());
            let decomp_map = Self::decomps_to_map(&syntax, &decompositions);
            let binding_map = Self::bindings_to_map(&syntax, &role_bindings);

            self.event_composer
                .compose_packed(&sentence_analysis, &decomp_map, &binding_map)
                .ok()
        };

        // 5. Build underspecified DRS
        let underspec_drs = packed_events.as_ref().map(|packed| {
            UnderspecDrs::from_packed(
                &packed.to_underspec(),
                canopy::kernel::discourse::Drs::default(),
            )
        });

        let mut analysis = UnderspecifiedAnalysis::new(text.to_string(), syntax);

        if let Some(packed) = packed_events {
            analysis = analysis.with_packed_events(packed);
        }

        if let Some(drs) = underspec_drs {
            analysis = analysis.with_underspec_drs(drs);
        }

        Ok(analysis)
    }

    /// Analyze a sentence with explicit disambiguation.
    ///
    /// First performs underspecified analysis, then applies the disambiguator
    /// to select a single reading.
    ///
    /// # Errors
    /// Returns an error if syntax parsing fails.
    pub fn analyze_with_disambiguator(
        &self,
        text: &str,
        disambiguator: &dyn Disambiguator,
    ) -> Result<SemanticAnalysis, CanopyError> {
        let underspec = self.analyze_underspecified(text)?;

        // If we have packed events, use disambiguator to select best reading
        if let Some(ref packed) = underspec.packed_events {
            let packed_semantics = packed.to_underspec();
            let ctx = DisambiguationContext::minimal();

            if let Some(reading) = disambiguator.select_reading(&packed_semantics, &ctx) {
                // Convert reading to composed events
                if let Some(composed) = packed.reading_to_composed(reading.id) {
                    let mut analysis = SemanticAnalysis::new(text.to_string(), underspec.syntax);
                    analysis = analysis.with_events(composed);
                    return Ok(analysis);
                }
            }
        }

        // Fall back to the default resolved analysis
        Ok(underspec.to_resolved())
    }

    /// Decompose predicates in the syntax.
    fn decompose_predicates(
        &self,
        syntax: &AnnotatedSyntax,
    ) -> Result<Vec<PredicateDecomposition>, CanopyError> {
        let mut all_decomps = Vec::new();

        // Find all predicate tokens (verbs)
        for pred in syntax.predicates() {
            let decomps = self.sense_provider.decompose_predicate(syntax, pred.id)?;

            // Filter by confidence threshold
            for decomp in decomps {
                if decomp.confidence >= self.config.decomposition_confidence_threshold {
                    all_decomps.push(decomp);
                }
            }
        }

        Ok(all_decomps)
    }

    /// Bind theta roles to arguments.
    fn bind_roles(
        &self,
        syntax: &AnnotatedSyntax,
        decompositions: &[PredicateDecomposition],
    ) -> Result<Vec<RoleBinding>, CanopyError> {
        let mut all_bindings = Vec::new();

        // For each decomposition, bind roles using the sense
        for decomp in decompositions {
            // Find the predicate token
            if let Some(pred) = syntax.predicates().next() {
                let bindings =
                    self.role_provider
                        .bind_roles(syntax, pred.id, Some(&decomp.sense_id))?;

                for binding in bindings {
                    if binding.confidence >= self.config.role_binding_confidence_threshold {
                        all_bindings.push(binding);
                    }
                }
            }
        }

        Ok(all_bindings)
    }

    /// Convert decompositions Vec to `HashMap` by predicate token ID.
    fn decomps_to_map(
        syntax: &AnnotatedSyntax,
        decompositions: &[PredicateDecomposition],
    ) -> HashMap<TokenId, Vec<PredicateDecomposition>> {
        let mut map = HashMap::new();

        // Find predicates and associate decompositions with them
        for pred in syntax.predicates() {
            let pred_decomps: Vec<_> = decompositions.to_vec();
            if !pred_decomps.is_empty() {
                map.insert(pred.id, pred_decomps);
            }
        }

        map
    }

    /// Convert role bindings Vec to `HashMap` by predicate token ID.
    fn bindings_to_map(
        syntax: &AnnotatedSyntax,
        bindings: &[RoleBinding],
    ) -> HashMap<TokenId, Vec<RoleBinding>> {
        let mut map = HashMap::new();

        // Associate all bindings with the first predicate for now
        // A more sophisticated implementation would track which predicate each binding belongs to
        if let Some(pred) = syntax.predicates().next() {
            let pred_bindings: Vec<_> = bindings.to_vec();
            if !pred_bindings.is_empty() {
                map.insert(pred.id, pred_bindings);
            }
        }

        map
    }
}

impl std::fmt::Debug for CanopyPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanopyPipeline")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_available() -> bool {
        crate::paths::data_path("data/verbnet").exists()
            && crate::paths::data_path("data/lexicon").exists()
    }

    #[test]
    fn test_pipeline_creation() {
        if !data_available() {
            eprintln!("Skipping: Required data not available");
            return;
        }

        let pipeline = CanopyPipeline::new();
        assert!(
            pipeline.is_ok(),
            "Failed to create pipeline: {:?}",
            pipeline.err()
        );
    }

    #[test]
    fn test_analyze_simple_sentence() {
        if !data_available() {
            eprintln!("Skipping: Required data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let result = pipeline.analyze("John runs.");

        assert!(result.is_ok(), "Failed to analyze: {:?}", result.err());
        let analysis = result.unwrap();
        assert_eq!(analysis.text, "John runs.");
        assert!(!analysis.syntax.tokens.is_empty());
    }

    #[test]
    fn test_analyze_document() {
        if !data_available() {
            eprintln!("Skipping: Required data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let result = pipeline.analyze_document("John runs. Mary walks.");

        assert!(
            result.is_ok(),
            "Failed to analyze document: {:?}",
            result.err()
        );
        let doc = result.unwrap();
        assert_eq!(doc.sentence_count(), 2);
    }

    #[test]
    fn test_pipeline_with_minimal_config() {
        if !data_available() {
            eprintln!("Skipping: Required data not available");
            return;
        }

        let config = PipelineConfig::minimal();
        let pipeline = CanopyPipeline::with_config(config).unwrap();
        let result = pipeline.analyze("Test sentence.");

        assert!(result.is_ok());
    }
}
