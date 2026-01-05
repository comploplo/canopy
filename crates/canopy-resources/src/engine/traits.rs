//! Common traits for semantic engines

use super::cache::CacheStats;
use super::error::EngineResult;
use super::stats::{EngineStats, PerformanceMetrics};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::Path;

/// Base result type for semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticResult<T> {
    pub data: T,
    pub confidence: f32,
    pub from_cache: bool,
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

/// Core trait for all semantic engines
pub trait SemanticEngine: Send + Sync + Debug {
    /// Type of data this engine analyzes
    type Input: Clone + Debug;
    /// Type of results this engine produces
    type Output: Clone + Debug;
    /// Engine-specific configuration
    type Config: Clone + Debug;

    /// Analyze input and return semantic results
    ///
    /// # Errors
    /// Returns an error if analysis fails.
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
    fn cache_stats(&self) -> CacheStats;

    /// Set cache capacity
    fn set_cache_capacity(&mut self, capacity: usize);
}

/// Trait for engines that provide statistics
pub trait StatisticsProvider: SemanticEngine {
    /// Get comprehensive statistics about the engine
    fn statistics(&self) -> EngineStats;

    /// Get performance metrics
    fn performance_metrics(&self) -> PerformanceMetrics;
}

/// Trait for engines that can load data from external sources
pub trait DataLoader: SemanticEngine {
    /// Load data from a directory
    ///
    /// # Errors
    /// Returns an error if data cannot be loaded from the specified path.
    fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()>;

    /// Load test data for development/testing
    ///
    /// # Errors
    /// Returns an error if test data cannot be loaded.
    fn load_test_data(&mut self) -> EngineResult<()>;

    /// Reload data from the current source
    ///
    /// # Errors
    /// Returns an error if data cannot be reloaded.
    fn reload(&mut self) -> EngineResult<()>;

    /// Get information about the loaded data
    fn data_info(&self) -> DataInfo;
}

/// Information about loaded data
#[derive(Debug, Clone)]
pub struct DataInfo {
    pub source: String,
    pub entry_count: usize,
    pub format_version: String,
    pub loaded_at: std::time::SystemTime,
    pub checksum: Option<String>,
}

impl DataInfo {
    /// Create new data info
    #[must_use]
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
    #[must_use]
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
    ///
    /// # Errors
    /// Returns an error if any analysis in the batch fails.
    fn analyze_batch(
        &self,
        inputs: &[Self::Input],
    ) -> EngineResult<Vec<SemanticResult<Self::Output>>>;

    /// Set the number of parallel threads
    fn set_thread_count(&mut self, count: usize);

    /// Get the current thread count
    fn thread_count(&self) -> usize;
}

/// Common configuration for all engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub enable_cache: bool,
    pub cache_capacity: usize,
    pub enable_metrics: bool,
    pub enable_parallel: bool,
    pub max_threads: usize,
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

/// Trait for engine-specific configs that can be converted to `EngineConfig`
pub trait EngineConfigurable {
    /// Whether caching is enabled
    fn enable_cache(&self) -> bool;

    /// Cache capacity (number of entries)
    fn cache_capacity(&self) -> usize;

    /// Minimum confidence threshold for results
    fn confidence_threshold(&self) -> f32;

    /// Whether to enable metrics collection (default: true)
    fn enable_metrics(&self) -> bool {
        true
    }

    /// Whether to enable parallel processing (default: false)
    fn enable_parallel(&self) -> bool {
        false
    }

    /// Maximum parallel threads (default: 4)
    fn max_threads(&self) -> usize {
        4
    }

    /// Convert to `EngineConfig` using trait methods
    fn to_engine_config(&self) -> EngineConfig {
        EngineConfig {
            enable_cache: self.enable_cache(),
            cache_capacity: self.cache_capacity(),
            enable_metrics: self.enable_metrics(),
            enable_parallel: self.enable_parallel(),
            max_threads: self.max_threads(),
            confidence_threshold: self.confidence_threshold(),
        }
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
        assert!(info.is_fresh(3600));
    }

    #[test]
    fn test_semantic_result_creation() {
        let result = SemanticResult::new(vec!["test"], 0.8, false, 100);
        assert!((result.confidence - 0.8).abs() < f32::EPSILON);
        assert!(!result.from_cache);
        assert_eq!(result.processing_time_us, 100);
    }

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert!(config.enable_cache);
        assert_eq!(config.cache_capacity, 10000);
    }
}
