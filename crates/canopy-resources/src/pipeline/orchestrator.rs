//! Main pipeline orchestrator.
//!
//! `CanopyPipeline` coordinates all components for end-to-end analysis.

use super::analysis::{DocumentAnalysis, SemanticAnalysis, UnderspecifiedAnalysis};
use super::config::PipelineConfig;
use super::trace_builder::TraceBuilder;
use crate::engine::SharedEngines;
use crate::providers::{ArgumentBinder, BinderConfig, DecomposerConfig, PredicateDecomposer};
use crate::syntax::{TreebankConfig, TreebankSyntaxProvider};
use crate::tokenizer::{SimpleTokenizer, Tokenizer};
use canopy::kernel::discourse::{DiscourseConfig, DiscourseContext, UnderspecDrs};
use canopy::kernel::events::{EventComposer, EventComposerConfig, SentenceAnalysis};
use canopy::kernel::trace::DerivationTrace;
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
    sense_provider: PredicateDecomposer,
    /// Role provider for theta role assignment.
    role_provider: ArgumentBinder,
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

        // Use multi-engine PredicateDecomposer for sense decomposition
        let sense_provider = PredicateDecomposer::new(engines.clone(), DecomposerConfig::default())
            .map_err(|e| CanopyError::data_load(format!("Failed to create sense provider: {e}")))?;

        // Use multi-engine ArgumentBinder for role binding
        let role_provider = ArgumentBinder::new(engines, BinderConfig::default())
            .map_err(|e| CanopyError::data_load(format!("Failed to create role provider: {e}")))?;

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

    /// Get a reference to the syntax provider.
    ///
    /// This allows access to pattern matching functionality:
    /// - `pattern_stats()` - Get cache hit statistics
    /// - `get_pattern()` - Get pattern for a semantic signature
    /// - `get_patterns_for_syntax()` - Get all patterns for verbs in syntax
    #[must_use]
    pub fn syntax_provider(&self) -> &TreebankSyntaxProvider {
        &self.syntax_provider
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

            for result in &mut sentence_results {
                let events_ref = result.events.as_ref();
                ctx.prepare_sentence(&result.syntax, events_ref);
                let sentence_index = ctx.current_sentence();

                // Store the discourse move classification
                if let Some(move_class) = ctx.last_move().cloned() {
                    result.discourse_move = Some(move_class);
                }

                if let Some(events) = events_ref {
                    ctx.process_events(events);
                    if let Some(report) = ctx.relevance_history().last().cloned() {
                        result.relevance = Some(report);
                    }
                    let validations: Vec<_> = ctx
                        .validation_history()
                        .iter()
                        .filter(|v| v.sentence_index == sentence_index)
                        .cloned()
                        .collect();
                    if !validations.is_empty() {
                        result.validations = validations;
                    }
                }

                // Classify coherence relation to previous sentence
                if let Some(coherence) = ctx.classify_coherence(&result.syntax, events_ref) {
                    result.coherence = Some(coherence);
                }

                // Store presuppositions detected in this sentence
                let presuppositions: Vec<_> =
                    ctx.current_presuppositions().into_iter().cloned().collect();
                if !presuppositions.is_empty() {
                    result.presuppositions = presuppositions;
                }

                // Finalize sentence state for next iteration
                ctx.finalize_sentence(&result.syntax, events_ref);
                ctx.end_sentence();
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

    /// Analyze a sentence with derivation trace.
    ///
    /// Returns both the semantic analysis and a trace explaining
    /// the reasoning behind sense selection and event composition.
    ///
    /// # Errors
    /// Returns an error if syntax parsing or semantic analysis fails.
    pub fn analyze_with_trace(
        &self,
        text: &str,
    ) -> Result<(SemanticAnalysis, DerivationTrace), CanopyError> {
        let mut trace_builder = TraceBuilder::new(text);

        // 1. Parse syntax
        let syntax = self.syntax_provider.parse(text)?;
        trace_builder.record_syntax(&syntax);

        // 2. Find predicates and get ALL decompositions (before filtering)
        let all_decomps = self.decompose_predicates_for_trace(&syntax)?;

        // Record sense selections for each predicate
        for pred in syntax.predicates() {
            if let Some(decomps) = all_decomps.get(&pred.id) {
                trace_builder.record_sense_selection(&pred.lemma, pred.id.0, decomps);
            }
        }

        // 3. Filter to get final decompositions
        let decompositions: Vec<_> = all_decomps
            .values()
            .flat_map(|v| v.iter())
            .filter(|d| d.confidence >= self.config.decomposition_confidence_threshold)
            .cloned()
            .collect();

        // 4. Bind theta roles
        let role_bindings = self.bind_roles(&syntax, &decompositions)?;

        // 5. Compose events
        let events = if decompositions.is_empty() {
            None
        } else {
            let sentence_analysis = SentenceAnalysis::new(text.to_string(), syntax.clone());
            let decomp_map = Self::decomps_to_map(&syntax, &decompositions);
            let binding_map = Self::bindings_to_map(&syntax, &role_bindings);

            let result = self
                .event_composer
                .compose(&sentence_analysis, &decomp_map, &binding_map)
                .ok();

            if let Some(ref events) = result {
                trace_builder.record_event_composition(events);
            }

            result
        };

        let mut analysis = SemanticAnalysis::new(text.to_string(), syntax)
            .with_decompositions(decompositions)
            .with_role_bindings(role_bindings);

        if let Some(events) = events {
            analysis = analysis.with_events(events);
        }

        let trace = trace_builder.build();
        Ok((analysis, trace))
    }

    /// Analyze a multi-sentence document with derivation trace.
    ///
    /// # Errors
    /// Returns an error if any sentence analysis fails.
    pub fn analyze_document_with_trace(
        &self,
        text: &str,
    ) -> Result<(DocumentAnalysis, DerivationTrace), CanopyError> {
        let mut trace_builder = TraceBuilder::new(text);

        // Split into sentences
        let sentences = self.tokenizer.split_sentences(text);
        let max_sents = self.config.max_sentences.unwrap_or(usize::MAX);
        let sentences: Vec<_> = sentences.into_iter().take(max_sents).collect();

        // Analyze each sentence (collect syntax for trace)
        let mut sentence_results = Vec::new();
        let mut all_syntax = Vec::new();

        for sentence in &sentences {
            let syntax = self.syntax_provider.parse(sentence)?;
            all_syntax.push(syntax.clone());

            // Get decompositions for trace
            let all_decomps = self.decompose_predicates_for_trace(&syntax)?;
            for pred in syntax.predicates() {
                if let Some(decomps) = all_decomps.get(&pred.id) {
                    trace_builder.record_sense_selection(&pred.lemma, pred.id.0, decomps);
                }
            }

            let result = self.analyze(sentence)?;
            if let Some(ref events) = result.events {
                trace_builder.record_event_composition(events);
            }
            sentence_results.push(result);
        }

        // Record combined syntax summary from all sentences
        trace_builder.record_combined_syntax(&all_syntax);

        // Build discourse representation if enabled
        let drs = if self.config.enable_discourse {
            let mut ctx = DiscourseContext::new(DiscourseConfig::default());

            for result in &mut sentence_results {
                let events_ref = result.events.as_ref();
                ctx.prepare_sentence(&result.syntax, events_ref);
                let sentence_index = ctx.current_sentence();

                // Store the discourse move classification
                if let Some(move_class) = ctx.last_move().cloned() {
                    result.discourse_move = Some(move_class);
                }

                if let Some(events) = events_ref {
                    ctx.process_events(events);
                    if let Some(report) = ctx.relevance_history().last().cloned() {
                        result.relevance = Some(report);
                    }
                    let validations: Vec<_> = ctx
                        .validation_history()
                        .iter()
                        .filter(|v| v.sentence_index == sentence_index)
                        .cloned()
                        .collect();
                    if !validations.is_empty() {
                        result.validations = validations;
                    }
                }

                // Classify coherence relation to previous sentence
                if let Some(coherence) = ctx.classify_coherence(&result.syntax, events_ref) {
                    result.coherence = Some(coherence);
                }

                // Store presuppositions detected in this sentence
                let presuppositions: Vec<_> =
                    ctx.current_presuppositions().into_iter().cloned().collect();
                if !presuppositions.is_empty() {
                    result.presuppositions = presuppositions;
                }

                // Finalize sentence state for next iteration
                ctx.finalize_sentence(&result.syntax, events_ref);
                ctx.end_sentence();
            }

            let drs = ctx.drs().clone();
            let qud_report = ctx.qud_report();
            trace_builder.record_discourse(
                &drs,
                qud_report,
                ctx.relevance_history(),
                ctx.validation_history(),
            );
            Some(drs)
        } else {
            None
        };

        let mut doc = DocumentAnalysis::new(sentence_results);
        if let Some(drs) = drs {
            doc = doc.with_drs(drs);
        }

        let trace = trace_builder.build();
        Ok((doc, trace))
    }

    /// Decompose predicates without filtering (for trace recording).
    fn decompose_predicates_for_trace(
        &self,
        syntax: &AnnotatedSyntax,
    ) -> Result<HashMap<TokenId, Vec<PredicateDecomposition>>, CanopyError> {
        let mut all_decomps = HashMap::new();

        for pred in syntax.predicates() {
            let decomps = self.sense_provider.decompose_predicate(syntax, pred.id)?;
            if !decomps.is_empty() {
                // Tag each decomposition with its source predicate
                let tagged: Vec<_> = decomps
                    .into_iter()
                    .map(|d| d.with_token_id(pred.id))
                    .collect();
                all_decomps.insert(pred.id, tagged);
            }
        }

        Ok(all_decomps)
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
            let filtered: Vec<_> = decomps
                .into_iter()
                .filter(|d| d.confidence >= self.config.decomposition_confidence_threshold)
                .collect();

            // Apply per-predicate limit
            let limited: Vec<_> = match self.config.max_decompositions_per_predicate {
                Some(max) => filtered.into_iter().take(max).collect(),
                None => filtered,
            };

            // Tag with source predicate and add to results
            for decomp in limited {
                all_decomps.push(decomp.with_token_id(pred.id));
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
            // Decompositions should always have token_id set by decompose_predicates()
            let Some(pred_id) = decomp.token_id else {
                continue;
            };

            let bindings =
                self.role_provider
                    .bind_roles(syntax, pred_id, Some(&decomp.sense_id))?;

            for binding in bindings {
                if binding.confidence >= self.config.role_binding_confidence_threshold {
                    // Tag binding with its source predicate
                    all_bindings.push(binding.with_predicate(pred_id));
                }
            }
        }

        Ok(all_bindings)
    }

    /// Convert decompositions Vec to `HashMap` by predicate token ID.
    fn decomps_to_map(
        _syntax: &AnnotatedSyntax,
        decompositions: &[PredicateDecomposition],
    ) -> HashMap<TokenId, Vec<PredicateDecomposition>> {
        let mut map: HashMap<TokenId, Vec<PredicateDecomposition>> = HashMap::new();

        // Group decompositions by their source predicate token
        for decomp in decompositions {
            if let Some(token_id) = decomp.token_id {
                map.entry(token_id).or_default().push(decomp.clone());
            }
        }

        map
    }

    /// Convert role bindings Vec to `HashMap` by predicate token ID.
    fn bindings_to_map(
        _syntax: &AnnotatedSyntax,
        bindings: &[RoleBinding],
    ) -> HashMap<TokenId, Vec<RoleBinding>> {
        let mut map: HashMap<TokenId, Vec<RoleBinding>> = HashMap::new();

        // Group bindings by their source predicate token
        for binding in bindings {
            if let Some(pred_id) = binding.predicate_token_id {
                map.entry(pred_id).or_default().push(binding.clone());
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

    #[test]
    fn test_multi_predicate_decomposition_separation() {
        if !data_available() {
            eprintln!("Skipping: Required data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        // "gave" and "ran" are separate predicates
        let result = pipeline.analyze("John gave Mary a book and ran home.");

        assert!(result.is_ok(), "Failed to analyze: {:?}", result.err());
        let analysis = result.unwrap();

        // Verify decompositions have token_id set
        for decomp in &analysis.decompositions {
            assert!(
                decomp.token_id.is_some(),
                "Decomposition for {:?} should have token_id set",
                decomp.sense_id
            );
        }

        // Verify role bindings have predicate_token_id set
        for binding in &analysis.role_bindings {
            assert!(
                binding.predicate_token_id.is_some(),
                "Binding for {:?} should have predicate_token_id set",
                binding.role
            );
        }

        // If we have multiple decompositions, verify they point to different predicates
        if analysis.decompositions.len() >= 2 {
            let token_ids: std::collections::HashSet<_> = analysis
                .decompositions
                .iter()
                .filter_map(|d| d.token_id)
                .collect();
            assert!(
                token_ids.len() >= 2 || analysis.decompositions.len() == token_ids.len(),
                "Multiple decompositions should be associated with distinct predicates when present"
            );
        }
    }

    #[test]
    fn test_decomps_to_map_groups_by_token_id() {
        use canopy::{LittleVType, ThetaRole};

        if !data_available() {
            eprintln!("Skipping: Required data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let syntax = pipeline.syntax_provider.parse("John runs.").unwrap();

        // Create mock decompositions with different token_ids
        let token1 = TokenId(1);
        let token2 = TokenId(2);

        let decomp1 =
            PredicateDecomposition::new("run-1".into(), LittleVType::Do, vec![ThetaRole::Agent])
                .with_token_id(token1);

        let decomp2 =
            PredicateDecomposition::new("run-2".into(), LittleVType::Go, vec![ThetaRole::Theme])
                .with_token_id(token2);

        let decompositions = vec![decomp1, decomp2];
        let map = CanopyPipeline::decomps_to_map(&syntax, &decompositions);

        // Verify each token_id maps to its own decomposition
        assert!(map.contains_key(&token1));
        assert!(map.contains_key(&token2));
        assert_eq!(map.get(&token1).unwrap().len(), 1);
        assert_eq!(map.get(&token2).unwrap().len(), 1);
    }

    #[test]
    fn test_bindings_to_map_groups_by_predicate() {
        use canopy::ThetaRole;

        if !data_available() {
            eprintln!("Skipping: Required data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let syntax = pipeline.syntax_provider.parse("John runs.").unwrap();

        // Create mock bindings with different predicate_token_ids
        let pred1 = TokenId(10);
        let pred2 = TokenId(20);

        let binding1 = RoleBinding::new(TokenId(1), ThetaRole::Agent, 0.9).with_predicate(pred1);

        let binding2 = RoleBinding::new(TokenId(2), ThetaRole::Theme, 0.8).with_predicate(pred2);

        let role_bindings = vec![binding1, binding2];
        let map = CanopyPipeline::bindings_to_map(&syntax, &role_bindings);

        // Verify each predicate maps to its own bindings
        assert!(map.contains_key(&pred1));
        assert!(map.contains_key(&pred2));
        assert_eq!(map.get(&pred1).unwrap().len(), 1);
        assert_eq!(map.get(&pred2).unwrap().len(), 1);
    }
}
