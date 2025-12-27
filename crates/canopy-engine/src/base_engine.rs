//! Base engine implementation providing common functionality
//!
//! This module provides the `BaseEngine` struct that handles common operations
//! like caching, statistics tracking, and performance metrics for all semantic engines.

use crate::{
    cache::EngineCache,
    stats::{EngineStats, PerformanceMetrics},
    EngineConfig, EngineResult, QualityMetrics, SemanticResult,
};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Core trait that engines must implement for domain-specific analysis
pub trait EngineCore<Input, Output>: Send + Sync
where
    Input: Clone + Debug,
    Output: Clone + Debug,
{
    /// Perform the actual analysis (without caching/stats handling)
    fn perform_analysis(&self, input: &Input) -> EngineResult<Output>;

    /// Calculate confidence score for the analysis result
    fn calculate_confidence(&self, input: &Input, output: &Output) -> f32;

    /// Generate cache key for the input
    fn generate_cache_key(&self, input: &Input) -> String;

    /// Get the engine's name for identification
    fn engine_name(&self) -> &'static str;

    /// Get the engine's version
    fn engine_version(&self) -> &'static str;

    /// Check if the engine is properly initialized
    fn is_initialized(&self) -> bool;
}

/// Standard cache key formats used across engines
#[derive(Debug, Clone)]
pub enum CacheKeyFormat {
    /// Simple key: "word"
    Simple(String),
    /// Typed key: "verb:walk"
    Typed(String, String),
    /// Compound key: "word|pos|context"
    Compound(Vec<String>),
}

impl std::fmt::Display for CacheKeyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheKeyFormat::Simple(key) => write!(f, "{key}"),
            CacheKeyFormat::Typed(prefix, key) => write!(f, "{prefix}:{key}"),
            CacheKeyFormat::Compound(parts) => {
                let joined = parts.join("|");
                write!(f, "{joined}")
            }
        }
    }
}

/// Confidence calculation utilities
pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    /// Calculate confidence based on match count
    pub fn from_match_count(matches: usize, total_possible: usize) -> f32 {
        if total_possible == 0 {
            0.0
        } else {
            (matches as f32 / total_possible as f32).min(1.0)
        }
    }

    /// Calculate confidence based on coverage percentage
    pub fn from_coverage(covered: usize, total: usize) -> f32 {
        if total == 0 {
            0.0
        } else {
            (covered as f32 / total as f32).min(1.0)
        }
    }

    /// Calculate weighted average of confidence scores
    pub fn weighted_average(scores_and_weights: &[(f32, f32)]) -> f32 {
        if scores_and_weights.is_empty() {
            return 0.0;
        }

        let total_weight: f32 = scores_and_weights.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return 0.0;
        }

        let weighted_sum: f32 = scores_and_weights
            .iter()
            .map(|(score, weight)| score * weight)
            .sum();

        weighted_sum / total_weight
    }

    /// Apply confidence boost based on multiple sources agreeing
    pub fn apply_agreement_boost(base_confidence: f32, agreement_factor: f32) -> f32 {
        let boost = agreement_factor * 0.1; // Max 10% boost
        (base_confidence + boost).min(1.0)
    }
}

/// Base engine providing common functionality for all semantic engines
pub struct BaseEngine<Input, Output>
where
    Input: Clone + Debug + Hash + Eq + Send + Sync,
    Output: Clone + Debug + Send + Sync,
{
    /// Result cache
    cache: Arc<Mutex<EngineCache<String, Output>>>,
    /// Engine statistics
    stats: Arc<Mutex<EngineStats>>,
    /// Performance metrics
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Quality metrics
    quality_metrics: Arc<Mutex<QualityMetrics>>,
    /// Configuration
    config: EngineConfig,
    /// Phantom data to satisfy type parameter usage
    _phantom: PhantomData<Input>,
}

impl<Input, Output> BaseEngine<Input, Output>
where
    Input: Clone + Debug + Hash + Eq + Send + Sync,
    Output: Clone + Debug + Send + Sync,
{
    /// Create a new base engine with the given configuration
    pub fn new(config: EngineConfig, engine_name: String) -> Self {
        let cache = Arc::new(Mutex::new(EngineCache::new(config.cache_capacity)));
        let stats = Arc::new(Mutex::new(EngineStats::new(engine_name)));
        let performance_metrics = Arc::new(Mutex::new(PerformanceMetrics::new()));
        let quality_metrics = Arc::new(Mutex::new(QualityMetrics::new()));

        Self {
            cache,
            stats,
            performance_metrics,
            quality_metrics,
            config,
            _phantom: PhantomData,
        }
    }

    /// Analyze input using the provided engine core implementation
    pub fn analyze<E>(&self, input: &Input, engine_core: &E) -> EngineResult<SemanticResult<Output>>
    where
        E: EngineCore<Input, Output>,
    {
        let start_time = Instant::now();
        let cache_key = engine_core.generate_cache_key(input);

        // Update query stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.performance.total_queries += 1;
        }

        // Check cache first if enabled
        if self.config.enable_cache {
            if let Ok(cache) = self.cache.lock() {
                if let Some(cached_output) = cache.get(&cache_key) {
                    debug!(
                        engine = engine_core.engine_name(),
                        cache_key = %cache_key,
                        "Cache hit"
                    );

                    // Update cache hit stats
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.cache.hits += 1;
                    }

                    let confidence = engine_core.calculate_confidence(input, &cached_output);
                    return Ok(SemanticResult::cached(cached_output, confidence));
                }
            }
        }

        // Cache miss - perform actual analysis
        debug!(
            engine = engine_core.engine_name(),
            cache_key = %cache_key,
            "Cache miss, performing analysis"
        );

        if let Ok(mut stats) = self.stats.lock() {
            stats.cache.misses += 1;
        }

        // Perform the actual analysis
        let output = engine_core.perform_analysis(input)?;
        let confidence = engine_core.calculate_confidence(input, &output);
        let processing_time = start_time.elapsed().as_micros() as u64;

        // Store in cache if enabled and confidence is high enough
        if self.config.enable_cache && confidence >= self.config.confidence_threshold {
            if let Ok(cache) = self.cache.lock() {
                cache.insert(cache_key.clone(), output.clone());
            }
        }

        // Update performance metrics
        if self.config.enable_metrics {
            self.update_metrics(processing_time, confidence);
        }

        // Log performance warning if query is slow
        if processing_time > 10_000 {
            warn!(
                engine = engine_core.engine_name(),
                processing_time_us = processing_time,
                cache_key = %cache_key,
                "Slow query detected"
            );
        }

        Ok(SemanticResult::new(
            output,
            confidence,
            false,
            processing_time,
        ))
    }

    /// Update internal metrics with the latest query
    fn update_metrics(&self, processing_time: u64, confidence: f32) {
        // Update performance metrics
        if let Ok(mut perf) = self.performance_metrics.lock() {
            perf.record_query(processing_time);
        }

        // Update quality metrics
        if let Ok(mut quality) = self.quality_metrics.lock() {
            quality.update(confidence);
        }
    }

    /// Get current engine statistics
    pub fn get_stats(&self) -> EngineStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            EngineStats::new("Unknown".to_string())
        }
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        if let Ok(metrics) = self.performance_metrics.lock() {
            metrics.clone()
        } else {
            PerformanceMetrics::new()
        }
    }

    /// Get current quality metrics
    pub fn get_quality_metrics(&self) -> QualityMetrics {
        if let Ok(metrics) = self.quality_metrics.lock() {
            metrics.clone()
        } else {
            QualityMetrics::new()
        }
    }

    /// Clear all cached data
    pub fn clear_cache(&self) {
        if let Ok(cache) = self.cache.lock() {
            cache.clear();
        }
        info!("Cache cleared");
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> crate::CacheStats {
        if let Ok(cache) = self.cache.lock() {
            cache.stats()
        } else {
            crate::CacheStats::empty()
        }
    }

    /// Set cache capacity
    pub fn set_cache_capacity(&self, capacity: usize) {
        if let Ok(cache) = self.cache.lock() {
            // NOTE: Current EngineCache doesn't support resizing,
            // would need to recreate with new capacity
            drop(cache);
        }
        info!(capacity = capacity, "Cache capacity update requested");
    }

    /// Get engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Check if engine has processed any queries
    pub fn has_activity(&self) -> bool {
        if let Ok(stats) = self.stats.lock() {
            stats.performance.total_queries > 0
        } else {
            false
        }
    }
}

impl<Input, Output> Debug for BaseEngine<Input, Output>
where
    Input: Clone + Debug + Hash + Eq + Send + Sync,
    Output: Clone + Debug + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseEngine")
            .field("config", &self.config)
            .field("has_activity", &self.has_activity())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MockInput(String);

    #[derive(Debug, Clone)]
    struct MockOutput(String, f32);

    impl Hash for MockInput {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
    }

    impl PartialEq for MockInput {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    impl Eq for MockInput {}

    struct MockEngineCore;

    impl EngineCore<MockInput, MockOutput> for MockEngineCore {
        fn perform_analysis(&self, input: &MockInput) -> EngineResult<MockOutput> {
            Ok(MockOutput(format!("analyzed_{}", input.0), 0.8))
        }

        fn calculate_confidence(&self, _input: &MockInput, output: &MockOutput) -> f32 {
            output.1
        }

        fn generate_cache_key(&self, input: &MockInput) -> String {
            format!("mock:{}", input.0)
        }

        fn engine_name(&self) -> &'static str {
            "MockEngine"
        }

        fn engine_version(&self) -> &'static str {
            "1.0.0"
        }

        fn is_initialized(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_base_engine_creation() {
        let config = EngineConfig::default();
        let engine: BaseEngine<MockInput, MockOutput> =
            BaseEngine::new(config, "TestEngine".to_string());

        assert!(!engine.has_activity());
        assert_eq!(engine.config().cache_capacity, 10000);
    }

    #[test]
    fn test_analyze_with_cache() {
        let config = EngineConfig::default();
        let engine: BaseEngine<MockInput, MockOutput> =
            BaseEngine::new(config, "TestEngine".to_string());
        let core = MockEngineCore;

        let input = MockInput("test".to_string());

        // First call should perform analysis
        let result1 = engine.analyze(&input, &core).unwrap();
        assert!(!result1.from_cache);
        assert_eq!(result1.data.0, "analyzed_test");

        // Second call should hit cache
        let result2 = engine.analyze(&input, &core).unwrap();
        assert!(result2.from_cache);
    }

    #[test]
    fn test_confidence_calculator() {
        assert_eq!(ConfidenceCalculator::from_match_count(5, 10), 0.5);
        assert_eq!(ConfidenceCalculator::from_coverage(8, 10), 0.8);

        let scores = vec![(0.8, 1.0), (0.6, 2.0)];
        let avg = ConfidenceCalculator::weighted_average(&scores);
        assert!((avg - 0.6667).abs() < 0.001);
    }

    #[test]
    fn test_cache_key_format() {
        let simple = CacheKeyFormat::Simple("word".to_string());
        assert_eq!(simple.to_string(), "word");

        let typed = CacheKeyFormat::Typed("verb".to_string(), "run".to_string());
        assert_eq!(typed.to_string(), "verb:run");

        let compound = CacheKeyFormat::Compound(vec!["word".to_string(), "pos".to_string()]);
        assert_eq!(compound.to_string(), "word|pos");
    }
}
