//! Base engine implementation providing common functionality

use super::{
    cache::EngineCache,
    error::EngineResult,
    stats::{EngineStats, PerformanceMetrics},
    traits::{EngineConfig, SemanticResult},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Core trait that engines must implement for domain-specific analysis
pub trait EngineCore<Input, Output>: Send + Sync
where
    Input: Clone + Debug,
    Output: Clone + Debug,
{
    /// Perform the actual analysis (without caching/stats handling)
    ///
    /// # Errors
    /// Returns an error if analysis fails.
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
    Simple(String),
    Typed(String, String),
    Compound(Vec<String>),
}

impl std::fmt::Display for CacheKeyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheKeyFormat::Simple(key) => write!(f, "{key}"),
            CacheKeyFormat::Typed(prefix, key) => write!(f, "{prefix}:{key}"),
            CacheKeyFormat::Compound(parts) => write!(f, "{}", parts.join("|")),
        }
    }
}

/// Convert usize to f32 safely for statistics (saturates at `u16::MAX` for lossless conversion).
#[inline]
#[must_use]
fn count_as_f32(n: usize) -> f32 {
    f32::from(u16::try_from(n).unwrap_or(u16::MAX))
}

/// Confidence calculation utilities
pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    #[must_use]
    pub fn from_match_count(matches: usize, total_possible: usize) -> f32 {
        if total_possible == 0 {
            0.0
        } else {
            (count_as_f32(matches) / count_as_f32(total_possible)).min(1.0)
        }
    }

    #[must_use]
    pub fn from_coverage(covered: usize, total: usize) -> f32 {
        if total == 0 {
            0.0
        } else {
            (count_as_f32(covered) / count_as_f32(total)).min(1.0)
        }
    }

    #[must_use]
    pub fn weighted_average(scores_and_weights: &[(f32, f32)]) -> f32 {
        if scores_and_weights.is_empty() {
            return 0.0;
        }

        let total_weight: f32 = scores_and_weights.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return 0.0;
        }

        scores_and_weights
            .iter()
            .map(|(score, weight)| score * weight)
            .sum::<f32>()
            / total_weight
    }
}

/// Quality metrics for analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub accuracy: f32,
    pub coverage: f32,
    pub avg_confidence: f32,
    pub high_confidence_count: usize,
    pub total_queries: usize,
}

impl QualityMetrics {
    #[must_use]
    pub fn new() -> Self {
        Self {
            accuracy: 0.0,
            coverage: 0.0,
            avg_confidence: 0.0,
            high_confidence_count: 0,
            total_queries: 0,
        }
    }

    pub fn update(&mut self, confidence: f32) {
        self.total_queries += 1;
        let prev_count = count_as_f32(self.total_queries - 1);
        let curr_count = count_as_f32(self.total_queries);
        self.avg_confidence = ((self.avg_confidence * prev_count) + confidence) / curr_count;

        if confidence >= 0.8 {
            self.high_confidence_count += 1;
        }
        self.coverage = count_as_f32(self.high_confidence_count) / curr_count;
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Base engine providing common functionality for all semantic engines
pub struct BaseEngine<Input, Output>
where
    Input: Clone + Debug + Hash + Eq + Send + Sync,
    Output: Clone + Debug + Send + Sync,
{
    cache: Arc<Mutex<EngineCache<String, Output>>>,
    stats: Arc<Mutex<EngineStats>>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
    quality_metrics: Arc<Mutex<QualityMetrics>>,
    config: EngineConfig,
    _phantom: PhantomData<Input>,
}

impl<Input, Output> BaseEngine<Input, Output>
where
    Input: Clone + Debug + Hash + Eq + Send + Sync,
    Output: Clone + Debug + Send + Sync,
{
    #[must_use]
    pub fn new(config: EngineConfig, engine_name: String) -> Self {
        Self {
            cache: Arc::new(Mutex::new(EngineCache::new(config.cache_capacity))),
            stats: Arc::new(Mutex::new(EngineStats::new(engine_name))),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
            quality_metrics: Arc::new(Mutex::new(QualityMetrics::new())),
            config,
            _phantom: PhantomData,
        }
    }

    /// Analyze input using the provided engine core
    ///
    /// # Errors
    /// Returns an error if analysis fails.
    pub fn analyze<E>(&self, input: &Input, engine_core: &E) -> EngineResult<SemanticResult<Output>>
    where
        E: EngineCore<Input, Output>,
    {
        let start_time = Instant::now();
        let cache_key = engine_core.generate_cache_key(input);

        if let Ok(mut stats) = self.stats.lock() {
            stats.performance.total_queries += 1;
        }

        if self.config.enable_cache {
            if let Ok(cache) = self.cache.lock() {
                if let Some(cached_output) = cache.get(&cache_key) {
                    if let Ok(mut stats) = self.stats.lock() {
                        stats.cache.hits += 1;
                    }
                    let confidence = engine_core.calculate_confidence(input, &cached_output);
                    return Ok(SemanticResult::cached(cached_output, confidence));
                }
            }
        }

        if let Ok(mut stats) = self.stats.lock() {
            stats.cache.misses += 1;
        }

        let output = engine_core.perform_analysis(input)?;
        let confidence = engine_core.calculate_confidence(input, &output);
        let processing_time = super::micros_to_u64(start_time.elapsed().as_micros());

        if self.config.enable_cache && confidence >= self.config.confidence_threshold {
            if let Ok(cache) = self.cache.lock() {
                cache.insert(cache_key, output.clone());
            }
        }

        if self.config.enable_metrics {
            if let Ok(mut perf) = self.performance_metrics.lock() {
                perf.record_query(processing_time);
            }
            if let Ok(mut quality) = self.quality_metrics.lock() {
                quality.update(confidence);
            }
        }

        Ok(SemanticResult::new(
            output,
            confidence,
            false,
            processing_time,
        ))
    }

    #[must_use]
    pub fn get_stats(&self) -> EngineStats {
        self.stats
            .lock()
            .map_or_else(|_| EngineStats::new("Unknown".to_string()), |s| s.clone())
    }

    #[must_use]
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics
            .lock()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn get_quality_metrics(&self) -> QualityMetrics {
        self.quality_metrics
            .lock()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    pub fn clear_cache(&self) {
        if let Ok(cache) = self.cache.lock() {
            cache.clear();
        }
    }

    #[must_use]
    pub fn cache_stats(&self) -> super::cache::CacheStats {
        self.cache
            .lock()
            .map_or_else(|_| super::cache::CacheStats::empty(), |c| c.stats())
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
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_format() {
        let simple = CacheKeyFormat::Simple("word".to_string());
        assert_eq!(simple.to_string(), "word");

        let typed = CacheKeyFormat::Typed("verb".to_string(), "run".to_string());
        assert_eq!(typed.to_string(), "verb:run");
    }

    #[test]
    fn test_confidence_calculator() {
        assert!((ConfidenceCalculator::from_match_count(5, 10) - 0.5).abs() < f32::EPSILON);
        assert!((ConfidenceCalculator::from_coverage(8, 10) - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_quality_metrics() {
        let mut metrics = QualityMetrics::new();
        metrics.update(0.9);
        metrics.update(0.5);
        assert_eq!(metrics.total_queries, 2);
        assert!((metrics.avg_confidence - 0.7).abs() < f32::EPSILON);
    }
}
