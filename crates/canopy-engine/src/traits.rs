//! Common traits for semantic engines
//!
//! This module defines the core traits that all semantic engines should implement,
//! providing a consistent interface across VerbNet, FrameNet, WordNet, and future engines.

use crate::{EngineResult, EngineStats, SemanticResult};
use std::fmt::Debug;
use std::path::Path;

/// Core trait for all semantic engines
pub trait SemanticEngine: Send + Sync + Debug {
    /// Type of data this engine analyzes
    type Input: Clone + Debug;
    /// Type of results this engine produces
    type Output: Clone + Debug;
    /// Engine-specific configuration
    type Config: Clone + Debug;

    /// Analyze input and return semantic results
    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>>;

    /// Get the engine's name for identification
    fn name(&self) -> &'static str;

    /// Get the engine's version
    fn version(&self) -> &'static str;

    /// Check if the engine is properly initialized
    fn is_initialized(&self) -> bool;

    /// Get the engine's configuration
    fn config(&self) -> &Self::Config;
}

/// Trait for engines that support caching
pub trait CachedEngine: SemanticEngine {
    /// Clear all cached data
    fn clear_cache(&self);

    /// Get cache statistics
    fn cache_stats(&self) -> crate::CacheStats;

    /// Set cache capacity
    fn set_cache_capacity(&mut self, capacity: usize);
}

/// Trait for engines that provide statistics
pub trait StatisticsProvider: SemanticEngine {
    /// Get comprehensive statistics about the engine
    fn statistics(&self) -> EngineStats;

    /// Get performance metrics
    fn performance_metrics(&self) -> crate::PerformanceMetrics;
}

/// Trait for engines that can load data from external sources
pub trait DataLoader: SemanticEngine {
    /// Load data from a directory
    fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()>;

    /// Load test data for development/testing
    fn load_test_data(&mut self) -> EngineResult<()>;

    /// Reload data from the current source
    fn reload(&mut self) -> EngineResult<()>;

    /// Get information about the loaded data
    fn data_info(&self) -> DataInfo;
}

/// Information about loaded data
#[derive(Debug, Clone)]
pub struct DataInfo {
    /// Source path or description
    pub source: String,
    /// Number of entries loaded
    pub entry_count: usize,
    /// Data format version
    pub format_version: String,
    /// Load timestamp
    pub loaded_at: std::time::SystemTime,
    /// Data checksum for integrity verification
    pub checksum: Option<String>,
}

impl DataInfo {
    /// Create new data info
    pub fn new(source: String, entry_count: usize) -> Self {
        Self {
            source,
            entry_count,
            format_version: "1.0".to_string(),
            loaded_at: std::time::SystemTime::now(),
            checksum: None,
        }
    }

    /// Check if data is fresh (loaded recently)
    pub fn is_fresh(&self, max_age_seconds: u64) -> bool {
        if let Ok(elapsed) = self.loaded_at.elapsed() {
            elapsed.as_secs() <= max_age_seconds
        } else {
            false
        }
    }
}

/// Trait for engines that support parallel processing
pub trait ParallelEngine: SemanticEngine {
    /// Analyze multiple inputs in parallel
    fn analyze_batch(
        &self,
        inputs: &[Self::Input],
    ) -> EngineResult<Vec<SemanticResult<Self::Output>>>;

    /// Set the number of parallel threads
    fn set_thread_count(&mut self, count: usize);

    /// Get the current thread count
    fn thread_count(&self) -> usize;
}

/// Trait for engines that support confidence scoring
pub trait ConfidenceEngine: SemanticEngine {
    /// Get confidence score for a specific analysis
    fn confidence_score(&self, input: &Self::Input, output: &Self::Output) -> f32;

    /// Filter results by confidence threshold
    fn filter_by_confidence(&self, results: Vec<Self::Output>, threshold: f32)
        -> Vec<Self::Output>;

    /// Get confidence distribution statistics
    fn confidence_distribution(&self) -> ConfidenceDistribution;
}

/// Confidence distribution statistics
#[derive(Debug, Clone)]
pub struct ConfidenceDistribution {
    /// Number of high confidence results (>= 0.8)
    pub high_confidence: usize,
    /// Number of medium confidence results (0.5-0.8)
    pub medium_confidence: usize,
    /// Number of low confidence results (< 0.5)
    pub low_confidence: usize,
    /// Average confidence score
    pub average: f32,
    /// Standard deviation of confidence scores
    pub std_dev: f32,
}

impl ConfidenceDistribution {
    /// Create new confidence distribution
    pub fn new() -> Self {
        Self {
            high_confidence: 0,
            medium_confidence: 0,
            low_confidence: 0,
            average: 0.0,
            std_dev: 0.0,
        }
    }

    /// Total number of results
    pub fn total(&self) -> usize {
        self.high_confidence + self.medium_confidence + self.low_confidence
    }

    /// Percentage of high confidence results
    pub fn high_confidence_rate(&self) -> f32 {
        if self.total() == 0 {
            0.0
        } else {
            self.high_confidence as f32 / self.total() as f32
        }
    }
}

impl Default for ConfidenceDistribution {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_info_creation() {
        let info = DataInfo::new("test_data".to_string(), 100);
        assert_eq!(info.source, "test_data");
        assert_eq!(info.entry_count, 100);
        assert_eq!(info.format_version, "1.0");
        assert!(info.is_fresh(3600)); // Should be fresh within an hour
    }

    #[test]
    fn test_confidence_distribution() {
        let dist = ConfidenceDistribution {
            high_confidence: 80,
            medium_confidence: 15,
            low_confidence: 5,
            average: 0.85,
            std_dev: 0.1,
        };

        assert_eq!(dist.total(), 100);
        assert_eq!(dist.high_confidence_rate(), 0.8);
    }
}
