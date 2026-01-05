//! `PropBank` semantic role labeling engine implementation
//!
//! This module provides the main `PropBankEngine` that implements semantic role labeling
//! using `PropBank` framesets and predicate-argument structures.

use super::config::PropBankConfig;
use super::parser::{PropBankData, PropBankParser, PropBankStats};
use super::types::{PropBankAnalysis, PropBankPredicate, SemanticRole};
use crate::engine::{
    count_to_f32, micros_to_u64, BaseEngine, CacheKeyFormat, CacheStats, CachedEngine,
    EngineConfigurable, EngineCore, EngineError, EngineResult, EngineStats, ParallelEngine,
    PerformanceMetrics, SemanticEngine, SemanticResult, StatisticsProvider,
};
use canopy::ThetaRole;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::info;

/// Input type for `PropBank` analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropBankInput {
    pub word: String,
}

impl Hash for PropBankInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word.hash(state);
    }
}

/// `PropBank` semantic role labeling engine
#[derive(Debug)]
pub struct PropBankEngine {
    /// Base engine handling cache, stats, and metrics
    base_engine: BaseEngine<PropBankInput, PropBankAnalysis>,
    /// `PropBank` data loaded from framesets
    data: Arc<PropBankData>,
    /// PropBank-specific configuration
    config: PropBankConfig,
}

impl PropBankEngine {
    /// Create a new `PropBank` engine with default configuration
    ///
    /// # Errors
    /// Returns an error if configuration is invalid or data cannot be loaded.
    pub fn new() -> EngineResult<Self> {
        let config = PropBankConfig::default();
        Self::with_config(config)
    }

    /// Create a `PropBank` engine with custom configuration
    ///
    /// # Errors
    /// Returns an error if configuration is invalid or data cannot be loaded.
    pub fn with_config(config: PropBankConfig) -> EngineResult<Self> {
        info!("Initializing PropBank engine");

        // Validate configuration
        config.validate().map_err(EngineError::data_load)?;

        // Load PropBank data
        let mut parser = PropBankParser::new(config.clone())?;
        let mut data = parser.load_data()?;
        data.update_stats();

        info!(
            "PropBank engine initialized with {} predicates from {} framesets",
            data.stats.total_predicates, data.stats.total_framesets
        );

        // Convert PropBankConfig to EngineConfig using trait
        let engine_config = config.to_engine_config();

        Ok(Self {
            base_engine: BaseEngine::new(engine_config, "PropBank".to_string()),
            data: Arc::new(data),
            config,
        })
    }

    /// Analyze a predicate with its arguments
    ///
    /// This is a specialized lookup by lemma+sense. For general word analysis, use `analyze_word`.
    ///
    /// # Errors
    /// Returns an error if the predicate is not found in the database.
    pub fn analyze_predicate(
        &self,
        lemma: &str,
        sense: &str,
    ) -> EngineResult<SemanticResult<PropBankPredicate>> {
        let start_time = std::time::Instant::now();
        let roleset = format!("{lemma}.{sense}");

        // Direct lookup
        if let Some(predicate) = self.data.predicates.get(&roleset) {
            let confidence = Self::calculate_predicate_confidence(predicate);
            return Ok(SemanticResult::new(
                predicate.clone(),
                confidence,
                false,
                micros_to_u64(start_time.elapsed().as_micros()),
            ));
        }

        // Try fuzzy matching if enabled
        if self.config.enable_fuzzy_matching() {
            let query_lower = lemma.to_lowercase();
            let fuzzy_matches: Vec<&PropBankPredicate> = self
                .data
                .predicates
                .values()
                .filter(|predicate| {
                    let lemma_lower = predicate.lemma.to_lowercase();
                    lemma_lower.contains(&query_lower) || query_lower.contains(&lemma_lower)
                })
                .collect();

            if let Some(best_match) = fuzzy_matches.first() {
                let confidence = Self::calculate_predicate_confidence(best_match) * 0.8;
                return Ok(SemanticResult::new(
                    (*best_match).clone(),
                    confidence,
                    false,
                    micros_to_u64(start_time.elapsed().as_micros()),
                ));
            }
        }

        Err(EngineError::analysis(
            format!("PropBank predicate not found: {roleset}"),
            "predicate lookup",
        ))
    }

    /// Analyze a word for all possible predicates
    ///
    /// Uses `BaseEngine` for caching and statistics tracking.
    ///
    /// # Errors
    /// Returns an error if analysis fails or confidence is below threshold.
    pub fn analyze_word(&self, word: &str) -> EngineResult<SemanticResult<PropBankAnalysis>> {
        let input = PropBankInput {
            word: word.to_string(),
        };
        self.base_engine.analyze(&input, self)
    }

    /// Core analysis logic without caching (used by `EngineCore` trait)
    /// Uses O(1) lemma index lookup instead of O(n) predicate filtering
    fn perform_word_analysis(&self, word: &str) -> EngineResult<PropBankAnalysis> {
        let mut analysis = PropBankAnalysis::new(word.to_string());

        // O(1) lookup using lemma index
        let matching_predicates: Vec<&PropBankPredicate> = self
            .data
            .lemma_index
            .get(word)
            .map(|roleset_ids| {
                roleset_ids
                    .iter()
                    .filter_map(|id| self.data.predicates.get(id))
                    .collect()
            })
            .unwrap_or_default();

        if matching_predicates.is_empty() {
            // Try fuzzy matching if enabled (this remains O(n) but only for misses)
            if self.config.enable_fuzzy_matching() {
                let query_lower = word.to_lowercase();
                let fuzzy_matches: Vec<&PropBankPredicate> = self
                    .data
                    .predicates
                    .values()
                    .filter(|predicate| {
                        let lemma_lower = predicate.lemma.to_lowercase();
                        lemma_lower.contains(&query_lower) || query_lower.contains(&lemma_lower)
                    })
                    .collect();

                for predicate in fuzzy_matches {
                    analysis.add_alternative(predicate.clone());
                }
            }
        } else {
            // Use the most common sense as primary, others as alternatives
            let primary_predicate = matching_predicates
                .iter()
                .find(|p| p.sense == "01") // Prefer .01 sense
                .or_else(|| matching_predicates.first())
                .unwrap();

            let confidence = Self::calculate_predicate_confidence(primary_predicate);
            analysis = PropBankAnalysis::with_predicate(
                word.to_string(),
                (*primary_predicate).clone(),
                confidence,
            );

            // Add other senses as alternatives
            for predicate in matching_predicates.iter().skip(1) {
                analysis.add_alternative((*predicate).clone());
            }
        }

        // Calculate final confidence
        analysis.calculate_confidence();

        // Filter by confidence threshold
        if analysis.confidence < self.config.min_confidence {
            return Err(EngineError::analysis(
                format!(
                    "PropBank analysis confidence {} below threshold {}",
                    analysis.confidence, self.config.min_confidence
                ),
                "confidence threshold",
            ));
        }

        Ok(analysis)
    }

    /// Get all predicates for a lemma (uses O(1) index lookup)
    #[must_use]
    pub fn get_framesets(&self, lemma: &str) -> Vec<&PropBankPredicate> {
        self.data
            .lemma_index
            .get(lemma)
            .map(|roleset_ids| {
                roleset_ids
                    .iter()
                    .filter_map(|id| self.data.predicates.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get semantic roles for a specific predicate
    ///
    /// # Errors
    /// Returns an error if the predicate is not found in the database.
    pub fn get_semantic_roles(&self, lemma: &str, sense: &str) -> EngineResult<Vec<SemanticRole>> {
        let roleset = format!("{lemma}.{sense}");

        if let Some(predicate) = self.data.predicates.get(&roleset) {
            Ok(predicate
                .arguments
                .iter()
                .map(|arg| arg.role.clone())
                .collect())
        } else {
            Err(EngineError::analysis(
                format!("Predicate not found: {roleset}"),
                "semantic role lookup",
            ))
        }
    }

    /// Get theta roles for compatibility with other engines
    ///
    /// # Errors
    /// Returns an error if the predicate is not found in the database.
    pub fn get_theta_roles(&self, lemma: &str, sense: &str) -> EngineResult<Vec<ThetaRole>> {
        let roles = self.get_semantic_roles(lemma, sense)?;
        Ok(roles
            .iter()
            .filter_map(super::types::SemanticRole::to_theta_role)
            .collect())
    }

    /// Calculate confidence for a predicate based on available information
    fn calculate_predicate_confidence(predicate: &PropBankPredicate) -> f32 {
        let mut confidence = 0.7; // Base confidence

        // Boost confidence based on number of arguments
        let arg_count_boost = (count_to_f32(predicate.arguments.len()) * 0.05).min(0.2);
        confidence += arg_count_boost;

        // Boost confidence if predicate has a definition
        if !predicate.definition.is_empty() {
            confidence += 0.05;
        }

        // Boost confidence for common senses
        match predicate.sense.as_str() {
            "01" => confidence += 0.1, // Most common sense
            "02" => confidence += 0.05,
            _ => {}
        }

        confidence.min(0.95)
    }

    /// Get `PropBank` statistics
    #[must_use]
    pub fn get_propbank_stats(&self) -> &PropBankStats {
        &self.data.stats
    }

    /// Batch analysis for multiple words
    #[must_use]
    pub fn analyze_batch(
        &self,
        words: &[&str],
    ) -> Vec<EngineResult<SemanticResult<PropBankAnalysis>>> {
        words.iter().map(|word| self.analyze_word(word)).collect()
    }

    /// Check if a predicate exists in the database
    #[must_use]
    pub fn has_predicate(&self, lemma: &str, sense: &str) -> bool {
        let roleset = format!("{lemma}.{sense}");
        self.data.predicates.contains_key(&roleset)
    }

    /// Get all available lemmas
    #[must_use]
    pub fn get_available_lemmas(&self) -> Vec<&String> {
        self.data.framesets.keys().collect()
    }

    /// Check if the engine supports parallel processing
    #[must_use]
    pub fn supports_parallel(&self) -> bool {
        true // PropBank engine implements ParallelEngine trait
    }
}

/// Implementation of `EngineCore` trait for `BaseEngine` integration
impl EngineCore<PropBankInput, PropBankAnalysis> for PropBankEngine {
    fn perform_analysis(&self, input: &PropBankInput) -> EngineResult<PropBankAnalysis> {
        self.perform_word_analysis(&input.word)
    }

    fn calculate_confidence(&self, _input: &PropBankInput, output: &PropBankAnalysis) -> f32 {
        output.confidence
    }

    fn generate_cache_key(&self, input: &PropBankInput) -> String {
        CacheKeyFormat::Typed("propbank".to_string(), input.word.clone()).to_string()
    }

    fn engine_name(&self) -> &'static str {
        "PropBank"
    }

    fn engine_version(&self) -> &'static str {
        "1.0.0"
    }

    fn is_initialized(&self) -> bool {
        !self.data.predicates.is_empty()
    }
}

// Implement required traits
impl SemanticEngine for PropBankEngine {
    type Input = String;
    type Output = PropBankAnalysis;
    type Config = PropBankConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        self.analyze_word(input)
    }

    fn name(&self) -> &'static str {
        "PropBank"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn is_initialized(&self) -> bool {
        !self.data.predicates.is_empty()
    }

    fn config(&self) -> &Self::Config {
        &self.config
    }
}

impl CachedEngine for PropBankEngine {
    fn clear_cache(&self) {
        self.base_engine.clear_cache();
    }

    fn cache_stats(&self) -> CacheStats {
        self.base_engine.cache_stats()
    }

    fn set_cache_capacity(&mut self, _capacity: usize) {
        // BaseEngine doesn't support runtime capacity changes
    }
}

impl StatisticsProvider for PropBankEngine {
    fn statistics(&self) -> EngineStats {
        self.base_engine.get_stats()
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }
}

impl ParallelEngine for PropBankEngine {
    fn analyze_batch(
        &self,
        inputs: &[Self::Input],
    ) -> EngineResult<Vec<SemanticResult<Self::Output>>> {
        // Use existing analyze_batch method but convert the signature
        let batch_results = self.analyze_batch(
            inputs
                .iter()
                .map(std::string::String::as_str)
                .collect::<Vec<_>>()
                .as_slice(),
        );

        // Convert Vec<EngineResult<SemanticResult<PropBankAnalysis>>> to EngineResult<Vec<SemanticResult<PropBankAnalysis>>>
        let mut results = Vec::new();
        for result in batch_results {
            results.push(result?);
        }
        Ok(results)
    }

    fn set_thread_count(&mut self, _count: usize) {
        // PropBank engine doesn't currently support configurable threading
        // This would require architectural changes to support
    }

    fn thread_count(&self) -> usize {
        1 // Currently single-threaded
    }
}

// Additional specialized methods for PropBank
impl PropBankEngine {
    /// Find predicates that share semantic roles with the given predicate
    ///
    /// # Errors
    /// Returns an error if the predicate is not found in the database.
    pub fn find_similar_predicates(
        &self,
        lemma: &str,
        sense: &str,
    ) -> EngineResult<Vec<&PropBankPredicate>> {
        let roleset = format!("{lemma}.{sense}");
        let reference_predicate = self.data.predicates.get(&roleset).ok_or_else(|| {
            EngineError::analysis(
                format!("Predicate not found: {roleset}"),
                "predicate lookup",
            )
        })?;

        let reference_roles: Vec<_> = reference_predicate
            .arguments
            .iter()
            .map(|arg| &arg.role)
            .collect();

        let mut similar = Vec::new();

        for predicate in self.data.predicates.values() {
            if predicate.roleset == roleset {
                continue; // Skip the reference predicate itself
            }

            let predicate_roles: Vec<_> = predicate.arguments.iter().map(|arg| &arg.role).collect();

            // Calculate role similarity (simple intersection count)
            let common_roles = reference_roles
                .iter()
                .filter(|role| predicate_roles.contains(role))
                .count();

            // Consider similar if they share at least 2 roles
            if common_roles >= 2 {
                similar.push(predicate);
            }
        }

        Ok(similar)
    }

    /// Get argument structure summary for a predicate
    ///
    /// # Errors
    /// Returns an error if the predicate is not found in the database.
    pub fn get_argument_structure(
        &self,
        lemma: &str,
        sense: &str,
    ) -> EngineResult<ArgumentStructure> {
        let roleset = format!("{lemma}.{sense}");
        let predicate = self.data.predicates.get(&roleset).ok_or_else(|| {
            EngineError::analysis(
                format!("Predicate not found: {roleset}"),
                "predicate lookup",
            )
        })?;

        let core_args = predicate.get_core_arguments();
        let modifiers = predicate.get_modifiers();

        Ok(ArgumentStructure {
            predicate: roleset,
            core_argument_count: core_args.len(),
            modifier_count: modifiers.len(),
            total_arguments: predicate.arguments.len(),
            theta_roles: predicate
                .arguments
                .iter()
                .filter_map(|arg| arg.role.to_theta_role())
                .collect(),
        })
    }
}

/// Summary of argument structure for a predicate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentStructure {
    pub predicate: String,
    pub core_argument_count: usize,
    pub modifier_count: usize,
    pub total_arguments: usize,
    pub theta_roles: Vec<ThetaRole>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_propbank_data() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir
            .path()
            .join("propbank-release")
            .join("data")
            .join("google")
            .join("ewt");
        fs::create_dir_all(&data_dir).unwrap();

        // Create a .prop file in CoNLL-style PropBank format
        // Format: file sent_idx token_idx tagger lemma roleset ----- args...
        let prop_content = r"ewt/test.xml.parse 0 2 gold give give.01 ----- 0:1-ARG0 2:0-rel 3:1-ARG1 5:1-ARG2
ewt/test.xml.parse 1 3 gold take take.01 ----- 0:1-ARG0 3:0-rel 4:1-ARG1 6:1-ARG2
ewt/test.xml.parse 2 1 gold run run.01 ----- 0:1-ARG0 1:0-rel
ewt/test.xml.parse 3 2 gold run run.02 ----- 0:1-ARG0 2:0-rel 3:1-ARG1";

        fs::write(data_dir.join("test.prop"), prop_content).unwrap();
        temp_dir
    }

    #[test]
    fn test_propbank_engine_creation() {
        let temp_dir = create_test_propbank_data();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_predicate_analysis() {
        let temp_dir = create_test_propbank_data();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config).unwrap();
        let result = engine.analyze_predicate("give", "01");

        assert!(result.is_ok());
        let predicate = result.unwrap();
        assert_eq!(predicate.data.lemma, "give");
        assert_eq!(predicate.data.sense, "01");
        assert_eq!(predicate.data.arguments.len(), 3);
    }

    #[test]
    fn test_word_analysis() {
        let temp_dir = create_test_propbank_data();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config).unwrap();
        let result = engine.analyze_word("run");

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.data.has_match());
        assert!(!analysis.data.alternative_rolesets.is_empty());
    }

    #[test]
    fn test_semantic_engine_trait() {
        let temp_dir = create_test_propbank_data();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config).unwrap();
        let query = "give".to_string();
        let result = engine.analyze(&query);

        assert!(result.is_ok());
    }

    #[test]
    fn test_statistics_provider() {
        let temp_dir = create_test_propbank_data();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config).unwrap();
        let stats = engine.statistics();

        assert_eq!(stats.engine_name, "PropBank");
        // stats.performance.total_queries is unsigned, always >= 0
    }

    #[test]
    fn test_theta_role_mapping() {
        let temp_dir = create_test_propbank_data();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config).unwrap();
        let theta_roles = engine.get_theta_roles("give", "01");

        assert!(theta_roles.is_ok());
        let roles = theta_roles.unwrap();
        assert!(roles.contains(&ThetaRole::Agent));
        assert!(roles.contains(&ThetaRole::Patient));
    }
}
