//! Performance Metrics and Monitoring for UDPipe 2.15
//!
//! This module provides comprehensive performance tracking and analysis
//! specifically designed for UDPipe 2.15 performance characteristics.
//!
//! ## UDPipe 2.15 Performance Strategy
//!
//! ```text
//! M3 (Current): Baseline measurement with UDPipe 2.15
//! M4:          Basic caching implementation
//! M5:          Full performance optimization
//! ```
//!
//! This module establishes baselines and provides the monitoring infrastructure
//! needed to measure cache effectiveness in future milestones.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive performance metrics for UDPipe 2.15 systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UDPipePerformanceMetrics {
    /// UDPipe version identifier
    pub version: String,

    /// Model file path
    pub model_path: Option<String>,

    /// Model loading time (one-time cost)
    pub model_load_time: Option<Duration>,

    /// Parsing performance by input size
    pub parsing_metrics: HashMap<InputSizeCategory, ParsingMetrics>,

    /// Cache performance (preparation for M4/M5)
    pub cache_metrics: CacheMetrics,

    /// Memory usage tracking
    pub memory_metrics: MemoryMetrics,

    /// Performance distribution analysis
    pub latency_distribution: LatencyDistribution,

    /// Warnings and performance violations
    pub warnings: Vec<PerformanceWarning>,
}

/// Input size categories for performance analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputSizeCategory {
    /// 1-10 words
    Small,
    /// 11-25 words
    Medium,
    /// 26-50 words
    Large,
    /// 51-100 words
    ExtraLarge,
    /// 100+ words
    Oversized,
}

impl InputSizeCategory {
    /// Categorize input based on word count
    pub fn from_word_count(words: usize) -> Self {
        match words {
            1..=10 => Self::Small,
            11..=25 => Self::Medium,
            26..=50 => Self::Large,
            51..=100 => Self::ExtraLarge,
            _ => Self::Oversized,
        }
    }

    /// Get expected performance threshold for this category
    pub fn performance_threshold_us(&self) -> u64 {
        match self {
            Self::Small => 100,       // Very fast for short inputs
            Self::Medium => 300,      // Should be under 300μs
            Self::Large => 500,       // Target threshold
            Self::ExtraLarge => 1000, // Acceptable for long inputs
            Self::Oversized => 2000,  // May need special handling
        }
    }
}

/// Performance metrics for a specific input size category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsingMetrics {
    /// Number of samples
    pub sample_count: usize,

    /// Total processing time across all samples
    pub total_time: Duration,

    /// Minimum processing time observed
    pub min_time: Duration,

    /// Maximum processing time observed
    pub max_time: Duration,

    /// Average processing time
    pub avg_time: Duration,

    /// 95th percentile processing time
    pub p95_time: Duration,

    /// 99th percentile processing time
    pub p99_time: Duration,

    /// Standard deviation of processing times
    pub std_dev: Duration,

    /// Number of times performance threshold was exceeded
    pub threshold_violations: usize,
}

impl Default for ParsingMetrics {
    fn default() -> Self {
        Self {
            sample_count: 0,
            total_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            avg_time: Duration::ZERO,
            p95_time: Duration::ZERO,
            p99_time: Duration::ZERO,
            std_dev: Duration::ZERO,
            threshold_violations: 0,
        }
    }
}

/// Cache performance metrics (preparation for M4/M5)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Cache hits
    pub hits: usize,

    /// Cache misses
    pub misses: usize,

    /// Cache evictions
    pub evictions: usize,

    /// Current cache size
    pub current_size: usize,

    /// Maximum cache size
    pub max_size: usize,

    /// Time saved due to cache hits
    pub time_saved: Duration,

    /// Average time for cache hit
    pub avg_hit_time: Duration,

    /// Average time for cache miss
    pub avg_miss_time: Duration,
}

impl CacheMetrics {
    /// Calculate cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Calculate cache effectiveness (time saved ratio)
    pub fn effectiveness(&self) -> f64 {
        if self.avg_miss_time.is_zero() {
            0.0
        } else {
            1.0 - (self.avg_hit_time.as_secs_f64() / self.avg_miss_time.as_secs_f64())
        }
    }
}

/// Memory usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,

    /// Average memory usage in bytes
    pub avg_memory_bytes: usize,

    /// Memory allocations
    pub allocations: usize,

    /// Memory deallocations
    pub deallocations: usize,

    /// Memory leaks detected
    pub leaks: usize,
}

/// Latency distribution analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyDistribution {
    /// Histogram buckets (microseconds -> count)
    pub histogram: HashMap<u64, usize>,

    /// Percentile values
    pub percentiles: HashMap<u8, Duration>, // P50, P90, P95, P99, etc.
}

/// Performance warning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceWarning {
    /// Processing time exceeded threshold
    ThresholdExceeded {
        actual_us: u64,
        threshold_us: u64,
        input_size: InputSizeCategory,
    },

    /// Memory usage exceeded expected bounds
    MemoryExceeded {
        actual_bytes: usize,
        expected_bytes: usize,
    },

    /// Model loading took unexpectedly long
    SlowModelLoad { load_time_ms: u64 },

    /// Cache performance is poor
    PoorCachePerformance {
        hit_rate: f64,
        expected_hit_rate: f64,
    },

    /// UDPipe 2.15 specific warning
    UDPipe2Performance {
        message: String,
        recommendation: String,
    },
}

/// Performance tracker for UDPipe 2.15 systems
pub struct PerformanceTracker {
    /// Accumulated metrics
    metrics: UDPipePerformanceMetrics,
}

impl PerformanceTracker {
    /// Create new performance tracker
    pub fn new(version: &str, model_path: Option<&str>) -> Self {
        Self {
            metrics: UDPipePerformanceMetrics {
                version: version.to_string(),
                model_path: model_path.map(|s| s.to_string()),
                model_load_time: None,
                parsing_metrics: HashMap::new(),
                cache_metrics: CacheMetrics::default(),
                memory_metrics: MemoryMetrics::default(),
                latency_distribution: LatencyDistribution::default(),
                warnings: Vec::new(),
            },
        }
    }

    /// Start tracking a parsing operation
    pub fn start_parsing(&mut self) -> ParsingSession {
        ParsingSession {
            start_time: Instant::now(),
            tracker: self,
        }
    }

    /// Record model loading time
    pub fn record_model_load_time(&mut self, load_time: Duration) {
        self.metrics.model_load_time = Some(load_time);

        // Warn if model loading is slow
        if load_time.as_millis() > 1000 {
            self.metrics
                .warnings
                .push(PerformanceWarning::SlowModelLoad {
                    load_time_ms: load_time.as_millis() as u64,
                });
        }
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self, time_saved: Duration) {
        self.metrics.cache_metrics.hits += 1;
        self.metrics.cache_metrics.time_saved += time_saved;
    }

    /// Record cache miss
    pub fn record_cache_miss(&mut self, processing_time: Duration) {
        self.metrics.cache_metrics.misses += 1;

        // Update average miss time
        let total_misses = self.metrics.cache_metrics.misses;
        self.metrics.cache_metrics.avg_miss_time = (self.metrics.cache_metrics.avg_miss_time
            * (total_misses - 1) as u32
            + processing_time)
            / total_misses as u32;
    }

    /// Get current performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            version: self.metrics.version.clone(),
            total_samples: self
                .metrics
                .parsing_metrics
                .values()
                .map(|m| m.sample_count)
                .sum(),
            avg_processing_time: self.calculate_overall_average(),
            threshold_violations: self
                .metrics
                .parsing_metrics
                .values()
                .map(|m| m.threshold_violations)
                .sum(),
            cache_hit_rate: self.metrics.cache_metrics.hit_rate(),
            warnings_count: self.metrics.warnings.len(),
            performance_grade: self.calculate_performance_grade(),
        }
    }

    /// Calculate overall average processing time
    fn calculate_overall_average(&self) -> Duration {
        let total_time: Duration = self
            .metrics
            .parsing_metrics
            .values()
            .map(|m| m.total_time)
            .sum();
        let total_samples: usize = self
            .metrics
            .parsing_metrics
            .values()
            .map(|m| m.sample_count)
            .sum();

        if total_samples == 0 {
            Duration::ZERO
        } else {
            total_time / total_samples as u32
        }
    }

    /// Calculate performance grade
    fn calculate_performance_grade(&self) -> PerformanceGrade {
        let avg_time = self.calculate_overall_average();
        let violation_rate = if self.metrics.parsing_metrics.is_empty() {
            0.0
        } else {
            let total_violations: usize = self
                .metrics
                .parsing_metrics
                .values()
                .map(|m| m.threshold_violations)
                .sum();
            let total_samples: usize = self
                .metrics
                .parsing_metrics
                .values()
                .map(|m| m.sample_count)
                .sum();
            total_violations as f64 / total_samples as f64
        };

        match (avg_time.as_micros(), violation_rate) {
            (0..=500, r) if r < 0.05 => PerformanceGrade::Excellent,
            (0..=500, r) if r < 0.10 => PerformanceGrade::Good,
            (501..=1000, r) if r < 0.20 => PerformanceGrade::Acceptable,
            (1001..=2000, _) => PerformanceGrade::NeedsOptimization,
            _ => PerformanceGrade::Poor,
        }
    }

    /// Add UDPipe 2.15 specific warning
    pub fn add_udpipe2_warning(&mut self, message: &str, recommendation: &str) {
        self.metrics
            .warnings
            .push(PerformanceWarning::UDPipe2Performance {
                message: message.to_string(),
                recommendation: recommendation.to_string(),
            });
    }

    /// Get metrics
    pub fn get_metrics(&self) -> &UDPipePerformanceMetrics {
        &self.metrics
    }
}

/// Active parsing session for tracking
pub struct ParsingSession<'a> {
    start_time: Instant,
    tracker: &'a mut PerformanceTracker,
}

impl<'a> ParsingSession<'a> {
    /// Complete the parsing session and record metrics
    pub fn complete(self, word_count: usize) {
        let processing_time = self.start_time.elapsed();
        let category = InputSizeCategory::from_word_count(word_count);
        let threshold = category.performance_threshold_us();

        // Get or create metrics for this category
        let metrics = self
            .tracker
            .metrics
            .parsing_metrics
            .entry(category)
            .or_default();

        // Update metrics
        metrics.sample_count += 1;
        metrics.total_time += processing_time;
        metrics.min_time = metrics.min_time.min(processing_time);
        metrics.max_time = metrics.max_time.max(processing_time);
        metrics.avg_time = metrics.total_time / metrics.sample_count as u32;

        // Check threshold violation
        if processing_time.as_micros() as u64 > threshold {
            metrics.threshold_violations += 1;

            self.tracker
                .metrics
                .warnings
                .push(PerformanceWarning::ThresholdExceeded {
                    actual_us: processing_time.as_micros() as u64,
                    threshold_us: threshold,
                    input_size: category,
                });
        }

        // Update latency distribution
        let bucket = (processing_time.as_micros() as u64 / 100) * 100; // 100μs buckets
        *self
            .tracker
            .metrics
            .latency_distribution
            .histogram
            .entry(bucket)
            .or_insert(0) += 1;
    }
}

/// Performance summary for reporting
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub version: String,
    pub total_samples: usize,
    pub avg_processing_time: Duration,
    pub threshold_violations: usize,
    pub cache_hit_rate: f64,
    pub warnings_count: usize,
    pub performance_grade: PerformanceGrade,
}

/// Performance grade classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceGrade {
    /// <500μs average, <5% violations
    Excellent,
    /// <500μs average, <10% violations
    Good,
    /// <1000μs average, <20% violations
    Acceptable,
    /// <2000μs average
    NeedsOptimization,
    /// >2000μs average
    Poor,
}

impl PerformanceGrade {
    /// Get grade description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Excellent => "Excellent performance - meets all targets",
            Self::Good => "Good performance - minor threshold violations",
            Self::Acceptable => "Acceptable performance - some optimization needed",
            Self::NeedsOptimization => "Performance needs optimization - consider caching",
            Self::Poor => "Poor performance - requires immediate attention",
        }
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Excellent => "🚀",
            Self::Good => "✅",
            Self::Acceptable => "⚖️",
            Self::NeedsOptimization => "⚠️",
            Self::Poor => "❌",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_size_categorization() {
        assert_eq!(
            InputSizeCategory::from_word_count(5),
            InputSizeCategory::Small
        );
        assert_eq!(
            InputSizeCategory::from_word_count(15),
            InputSizeCategory::Medium
        );
        assert_eq!(
            InputSizeCategory::from_word_count(35),
            InputSizeCategory::Large
        );
        assert_eq!(
            InputSizeCategory::from_word_count(75),
            InputSizeCategory::ExtraLarge
        );
        assert_eq!(
            InputSizeCategory::from_word_count(150),
            InputSizeCategory::Oversized
        );
    }

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new("2.15", Some("/path/to/model"));

        // Simulate parsing sessions - use fake timing instead of sleep for reliability
        let session = tracker.start_parsing();
        // Manually set a short time by completing immediately
        session.complete(5); // Small input - should be fast

        let session = tracker.start_parsing();
        std::thread::sleep(Duration::from_micros(600)); // Over 300μs threshold for medium
        session.complete(15); // Medium input (exceeds threshold)

        let summary = tracker.get_summary();
        assert_eq!(summary.total_samples, 2);
        // Allow for timing variations - the key is that violations occur
        assert!(summary.threshold_violations >= 1); // At least one exceeded threshold
        assert!(summary.threshold_violations <= 2); // But not more than our tests
        assert!(summary.avg_processing_time.as_micros() > 0);
    }

    #[test]
    fn test_cache_metrics() {
        let mut metrics = CacheMetrics::default();
        assert_eq!(metrics.hit_rate(), 0.0);

        metrics.hits = 8;
        metrics.misses = 2;
        assert_eq!(metrics.hit_rate(), 0.8);
    }

    #[test]
    fn test_performance_grade() {
        assert_eq!(PerformanceGrade::Excellent.emoji(), "🚀");
        assert_eq!(PerformanceGrade::Poor.emoji(), "❌");
        assert!(PerformanceGrade::Excellent
            .description()
            .contains("Excellent"));
    }
}
