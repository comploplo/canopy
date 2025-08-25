//! Base engine infrastructure for semantic analysis
//!
//! This crate provides common traits, caching mechanisms, and utilities
//! that are shared across all semantic engines (VerbNet, FrameNet, WordNet).
//!
//! # Features
//!
//! - **Common Traits**: `SemanticEngine`, `CachedEngine`, `StatisticsProvider`
//! - **High-Performance Caching**: LRU cache with performance metrics
//! - **Error Handling**: Unified error types and conversion traits
//! - **Statistics**: Common statistics collection and reporting
//! - **Parallel Processing**: Optional parallel query support

use serde::{Deserialize, Serialize};

pub mod cache;
pub mod error;
pub mod parallel;
pub mod stats;
pub mod traits;
pub mod xml_parser;

// Re-export main types for convenience
pub use cache::{CacheKey, CacheStats, EngineCache};
pub use error::{EngineError, EngineResult};
pub use parallel::ParallelProcessor;
pub use stats::{EngineStats, PerformanceMetrics};
pub use traits::{CachedEngine, DataLoader, SemanticEngine, StatisticsProvider};
pub use xml_parser::{XmlParser, XmlParserConfig, XmlResource};

/// Common configuration for all engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Enable caching
    pub enable_cache: bool,
    /// Cache capacity (number of entries)
    pub cache_capacity: usize,
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    /// Enable parallel processing
    pub enable_parallel: bool,
    /// Maximum number of parallel threads
    pub max_threads: usize,
    /// Confidence threshold for results
    pub confidence_threshold: f32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_capacity: 10000,
            enable_metrics: true,
            enable_parallel: false,
            max_threads: 4,
            confidence_threshold: 0.5,
        }
    }
}

/// Base result type for semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticResult<T> {
    /// Analysis results
    pub data: T,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Whether result came from cache
    pub from_cache: bool,
    /// Processing time in microseconds
    pub processing_time_us: u64,
}

impl<T> SemanticResult<T> {
    /// Create a new semantic result
    pub fn new(data: T, confidence: f32, from_cache: bool, processing_time_us: u64) -> Self {
        Self {
            data,
            confidence,
            from_cache,
            processing_time_us,
        }
    }

    /// Create a result with high confidence
    pub fn with_high_confidence(data: T, processing_time_us: u64) -> Self {
        Self::new(data, 0.95, false, processing_time_us)
    }

    /// Create a cached result
    pub fn cached(data: T, confidence: f32) -> Self {
        Self::new(data, confidence, true, 0)
    }
}

/// Quality metrics for analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall accuracy score
    pub accuracy: f32,
    /// Coverage percentage
    pub coverage: f32,
    /// Average confidence of results
    pub avg_confidence: f32,
    /// Number of high-confidence results
    pub high_confidence_count: usize,
    /// Total number of queries
    pub total_queries: usize,
}

impl QualityMetrics {
    /// Create new quality metrics
    pub fn new() -> Self {
        Self {
            accuracy: 0.0,
            coverage: 0.0,
            avg_confidence: 0.0,
            high_confidence_count: 0,
            total_queries: 0,
        }
    }

    /// Update metrics with a new result
    pub fn update(&mut self, confidence: f32) {
        self.total_queries += 1;

        // Update average confidence using running average
        self.avg_confidence = ((self.avg_confidence * (self.total_queries - 1) as f32)
            + confidence)
            / self.total_queries as f32;

        // Count high confidence results (>= 0.8)
        if confidence >= 0.8 {
            self.high_confidence_count += 1;
        }

        // Update coverage (percentage of successful queries)
        self.coverage = self.high_confidence_count as f32 / self.total_queries as f32;
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert!(config.enable_cache);
        assert_eq!(config.cache_capacity, 10000);
        assert!(config.enable_metrics);
        assert!(!config.enable_parallel);
    }

    #[test]
    fn test_semantic_result_creation() {
        let result = SemanticResult::new(vec!["test"], 0.8, false, 100);
        assert_eq!(result.confidence, 0.8);
        assert!(!result.from_cache);
        assert_eq!(result.processing_time_us, 100);
    }

    #[test]
    fn test_quality_metrics_update() {
        let mut metrics = QualityMetrics::new();

        metrics.update(0.9);
        assert_eq!(metrics.total_queries, 1);
        assert_eq!(metrics.avg_confidence, 0.9);
        assert_eq!(metrics.high_confidence_count, 1);

        metrics.update(0.5);
        assert_eq!(metrics.total_queries, 2);
        assert_eq!(metrics.avg_confidence, 0.7);
        assert_eq!(metrics.high_confidence_count, 1);
    }
}
