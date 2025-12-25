//! PropBank semantic role labeling engine implementation
//!
//! This module provides the main PropBankEngine that implements semantic role labeling
//! using PropBank framesets and predicate-argument structures.

use crate::config::PropBankConfig;
use crate::parser::{PropBankData, PropBankParser, PropBankStats};
use crate::types::{PropBankAnalysis, PropBankPredicate, SemanticRole};
use canopy_core::ThetaRole;
use canopy_engine::traits::ParallelEngine;
use canopy_engine::{
    CacheStats, CachedEngine, EngineError, EngineResult, EngineStats, PerformanceMetrics,
    SemanticEngine, SemanticResult, StatisticsProvider,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::info;

/// PropBank semantic role labeling engine
#[derive(Debug)]
pub struct PropBankEngine {
    data: Arc<PropBankData>,
    config: PropBankConfig,
    cache: Arc<Mutex<HashMap<String, PropBankAnalysis>>>,
    stats: Arc<Mutex<EngineStats>>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl PropBankEngine {
    /// Create a new PropBank engine with default configuration
    pub fn new() -> EngineResult<Self> {
        let config = PropBankConfig::default();
        Self::with_config(config)
    }

    /// Create a PropBank engine with custom configuration
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

        // Initialize components
        let cache = Arc::new(Mutex::new(HashMap::new()));
        let stats = Arc::new(Mutex::new(EngineStats::new("PropBank".to_string())));
        let performance_metrics = Arc::new(Mutex::new(PerformanceMetrics::new()));

        Ok(Self {
            data: Arc::new(data),
            config,
            cache,
            stats,
            performance_metrics,
        })
    }

    /// Analyze a predicate with its arguments
    pub fn analyze_predicate(
        &self,
        lemma: &str,
        sense: &str,
    ) -> EngineResult<SemanticResult<PropBankPredicate>> {
        let start_time = std::time::Instant::now();
        let cache_key = format!("propbank:{}#{}", lemma.to_lowercase(), sense);

        // Check cache first
        if self.config.enable_cache {
            if let Ok(cache) = self.cache.lock() {
                if let Some(cached_analysis) = cache.get(&cache_key) {
                    if let Some(ref predicate) = cached_analysis.predicate {
                        return Ok(SemanticResult::cached(
                            predicate.clone(),
                            cached_analysis.confidence,
                        ));
                    }
                }
            }
        }

        // Look up predicate
        let roleset = format!("{lemma}.{sense}");
        let result = if let Some(predicate) = self.data.predicates.get(&roleset) {
            let confidence = self.calculate_predicate_confidence(predicate);

            SemanticResult::new(
                predicate.clone(),
                confidence,
                false,
                start_time.elapsed().as_micros() as u64,
            )
        } else {
            // Try fuzzy matching if enabled
            if self.config.enable_fuzzy_matching {
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
                    let confidence = self.calculate_predicate_confidence(best_match) * 0.8; // Lower confidence for fuzzy match

                    SemanticResult::new(
                        (*best_match).clone(),
                        confidence,
                        false,
                        start_time.elapsed().as_micros() as u64,
                    )
                } else {
                    return Err(EngineError::analysis(
                        format!("No PropBank predicate found for: {roleset}"),
                        "fuzzy matching",
                    ));
                }
            } else {
                return Err(EngineError::analysis(
                    format!("PropBank predicate not found: {roleset}"),
                    "predicate lookup",
                ));
            }
        };

        // Cache the analysis result
        if self.config.enable_cache {
            let analysis = PropBankAnalysis::with_predicate(
                format!("{lemma}#{sense}"),
                result.data.clone(),
                result.confidence,
            );
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(cache_key, analysis);
            }
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.performance.total_queries += 1;
            // Would need more sophisticated metrics tracking for timing stats
        }

        Ok(result)
    }

    /// Analyze a word for all possible predicates
    pub fn analyze_word(&self, word: &str) -> EngineResult<SemanticResult<PropBankAnalysis>> {
        let start_time = std::time::Instant::now();
        let cache_key = format!("propbank:{}", word.to_lowercase());

        // Check cache first
        if self.config.enable_cache {
            if let Ok(cache) = self.cache.lock() {
                if let Some(cached_analysis) = cache.get(&cache_key) {
                    return Ok(SemanticResult::cached(
                        cached_analysis.clone(),
                        cached_analysis.confidence,
                    ));
                }
            }
        }

        // Find all predicates matching this lemma
        let matching_predicates: Vec<&PropBankPredicate> = self
            .data
            .predicates
            .values()
            .filter(|pred| pred.lemma == word)
            .collect();

        let mut analysis = PropBankAnalysis::new(word.to_string());

        if matching_predicates.is_empty() {
            // Try fuzzy matching if enabled
            if self.config.enable_fuzzy_matching {
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

            let confidence = self.calculate_predicate_confidence(primary_predicate);
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

        let result = SemanticResult::new(
            analysis.clone(),
            analysis.confidence,
            false,
            start_time.elapsed().as_micros() as u64,
        );

        // Cache the result
        if self.config.enable_cache {
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(cache_key, analysis.clone());
            }
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.performance.total_queries += 1;
            // Would need more sophisticated metrics tracking for timing stats
        }

        Ok(result)
    }

    /// Get all predicates for a lemma
    pub fn get_framesets(&self, lemma: &str) -> Vec<&PropBankPredicate> {
        self.data
            .predicates
            .values()
            .filter(|pred| pred.lemma == lemma)
            .collect()
    }

    /// Get semantic roles for a specific predicate
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
    pub fn get_theta_roles(&self, lemma: &str, sense: &str) -> EngineResult<Vec<ThetaRole>> {
        let roles = self.get_semantic_roles(lemma, sense)?;
        Ok(roles
            .iter()
            .filter_map(|role| role.to_theta_role())
            .collect())
    }

    /// Calculate confidence for a predicate based on available information
    fn calculate_predicate_confidence(&self, predicate: &PropBankPredicate) -> f32 {
        let mut confidence = 0.7; // Base confidence

        // Boost confidence based on number of arguments
        let arg_count_boost = (predicate.arguments.len() as f32 * 0.05).min(0.2);
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

    /// Get PropBank statistics
    pub fn get_propbank_stats(&self) -> &PropBankStats {
        &self.data.stats
    }

    /// Batch analysis for multiple words
    pub fn analyze_batch(
        &self,
        words: &[&str],
    ) -> Vec<EngineResult<SemanticResult<PropBankAnalysis>>> {
        words.iter().map(|word| self.analyze_word(word)).collect()
    }

    /// Check if a predicate exists in the database
    pub fn has_predicate(&self, lemma: &str, sense: &str) -> bool {
        let roleset = format!("{lemma}.{sense}");
        self.data.predicates.contains_key(&roleset)
    }

    /// Get all available lemmas
    pub fn get_available_lemmas(&self) -> Vec<&String> {
        self.data.framesets.keys().collect()
    }

    /// Check if the engine supports parallel processing
    pub fn supports_parallel(&self) -> bool {
        true // PropBank engine implements ParallelEngine trait
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
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    fn cache_stats(&self) -> CacheStats {
        if let Ok(cache) = self.cache.lock() {
            CacheStats {
                hits: 0,   // Would need to track this separately
                misses: 0, // Would need to track this separately
                total_lookups: 0,
                hit_rate: 0.0,
                evictions: 0,
                current_size: cache.len(),
                has_ttl: false,
            }
        } else {
            CacheStats::empty()
        }
    }

    fn set_cache_capacity(&mut self, _capacity: usize) {
        // Note: PropBank config is immutable after creation
        // This would require a more complex design to support
    }
}

impl StatisticsProvider for PropBankEngine {
    fn statistics(&self) -> EngineStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            EngineStats::new("PropBank".to_string())
        }
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        if let Ok(metrics) = self.performance_metrics.lock() {
            metrics.clone()
        } else {
            PerformanceMetrics::new()
        }
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
                .map(|s| s.as_str())
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

    fn create_test_propbank_data() -> EngineResult<TempDir> {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir
            .path()
            .join("propbank-release")
            .join("data")
            .join("google")
            .join("ewt");
        fs::create_dir_all(&data_dir).unwrap();

        // Create a simple .prop file
        let prop_content = r#"give.01: ARG0:agent ARG1:theme ARG2:recipient
take.01: ARG0:agent ARG1:theme ARG2:source
run.01: ARG0:agent
run.02: ARG0:agent ARG1:path"#;

        fs::write(data_dir.join("test.prop"), prop_content).unwrap();
        Ok(temp_dir)
    }

    #[test]
    fn test_propbank_engine_creation() {
        let temp_dir = create_test_propbank_data().unwrap();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_predicate_analysis() {
        let temp_dir = create_test_propbank_data().unwrap();
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
        let temp_dir = create_test_propbank_data().unwrap();
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
        let temp_dir = create_test_propbank_data().unwrap();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config).unwrap();
        let query = "give".to_string();
        let result = engine.analyze(&query);

        assert!(result.is_ok());
    }

    #[test]
    fn test_statistics_provider() {
        let temp_dir = create_test_propbank_data().unwrap();
        let config = PropBankConfig::default()
            .with_data_path(temp_dir.path().join("propbank-release").join("data"));

        let engine = PropBankEngine::with_config(config).unwrap();
        let stats = engine.statistics();

        assert_eq!(stats.engine_name, "PropBank");
        assert!(stats.performance.total_queries >= 0);
    }

    #[test]
    fn test_theta_role_mapping() {
        let temp_dir = create_test_propbank_data().unwrap();
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
