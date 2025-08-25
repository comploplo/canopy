//! Statistics and performance metrics for semantic engines
//!
//! This module provides comprehensive statistics collection and reporting
//! capabilities for all semantic engines.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Comprehensive engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    /// Engine name
    pub engine_name: String,
    /// Data statistics
    pub data: DataStats,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Quality metrics
    pub quality: QualityStats,
    /// Cache statistics
    pub cache: crate::CacheStats,
}

impl EngineStats {
    /// Create new engine statistics
    pub fn new(engine_name: String) -> Self {
        Self {
            engine_name,
            data: DataStats::default(),
            performance: PerformanceMetrics::default(),
            quality: QualityStats::default(),
            cache: crate::CacheStats::empty(),
        }
    }
}

/// Statistics about loaded data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStats {
    /// Total number of entries loaded
    pub total_entries: usize,
    /// Number of unique keys/lemmas
    pub unique_keys: usize,
    /// Data format version
    pub format_version: String,
    /// Size of data in memory (bytes)
    pub memory_size_bytes: usize,
    /// Data source path or description
    pub data_source: String,
    /// Load timestamp
    pub loaded_at: SystemTime,
}

impl Default for DataStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            unique_keys: 0,
            format_version: "1.0".to_string(),
            memory_size_bytes: 0,
            data_source: "unknown".to_string(),
            loaded_at: SystemTime::now(),
        }
    }
}

/// Performance metrics for engine operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total number of queries processed
    pub total_queries: u64,
    /// Average query time in microseconds
    pub avg_query_time_us: f64,
    /// Minimum query time in microseconds
    pub min_query_time_us: u64,
    /// Maximum query time in microseconds
    pub max_query_time_us: u64,
    /// 95th percentile query time in microseconds
    pub p95_query_time_us: u64,
    /// 99th percentile query time in microseconds
    pub p99_query_time_us: u64,
    /// Queries per second
    pub queries_per_second: f64,
    /// Total processing time across all queries
    pub total_processing_time_ms: u64,
    /// Engine uptime in seconds
    pub uptime_secs: u64,
    /// Start time as timestamp
    start_time_secs: u64,
    /// Query times for percentile calculations
    query_times: Vec<u64>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        Self {
            total_queries: 0,
            avg_query_time_us: 0.0,
            min_query_time_us: u64::MAX,
            max_query_time_us: 0,
            p95_query_time_us: 0,
            p99_query_time_us: 0,
            queries_per_second: 0.0,
            total_processing_time_ms: 0,
            uptime_secs: 0,
            start_time_secs: now,
            query_times: Vec::new(),
        }
    }

    /// Record a query execution time
    pub fn record_query(&mut self, duration_us: u64) {
        self.total_queries += 1;
        self.total_processing_time_ms += duration_us / 1000;

        // Update min/max
        self.min_query_time_us = self.min_query_time_us.min(duration_us);
        self.max_query_time_us = self.max_query_time_us.max(duration_us);

        // Store for percentile calculation (limit to last 10k queries)
        self.query_times.push(duration_us);
        if self.query_times.len() > 10000 {
            self.query_times.remove(0);
        }

        // Update averages
        self.avg_query_time_us = (self.avg_query_time_us * (self.total_queries - 1) as f64
            + duration_us as f64)
            / self.total_queries as f64;

        // Update uptime and QPS
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        self.uptime_secs = now.saturating_sub(self.start_time_secs);

        if self.uptime_secs > 0 {
            self.queries_per_second = self.total_queries as f64 / self.uptime_secs as f64;
        }

        // Calculate percentiles
        self.calculate_percentiles();
    }

    /// Calculate 95th and 99th percentiles
    fn calculate_percentiles(&mut self) {
        if self.query_times.is_empty() {
            return;
        }

        let mut sorted_times = self.query_times.clone();
        sorted_times.sort_unstable();

        let len = sorted_times.len();
        let p95_index = (len as f64 * 0.95) as usize;
        let p99_index = (len as f64 * 0.99) as usize;

        self.p95_query_time_us = sorted_times
            .get(p95_index.min(len - 1))
            .copied()
            .unwrap_or(0);
        self.p99_query_time_us = sorted_times
            .get(p99_index.min(len - 1))
            .copied()
            .unwrap_or(0);
    }

    /// Check if performance is meeting targets
    pub fn is_performing_well(&self) -> bool {
        // Good performance: avg < 1ms, p95 < 5ms, QPS > 100
        self.avg_query_time_us < 1000.0
            && self.p95_query_time_us < 5000
            && self.queries_per_second > 100.0
    }

    /// Get performance grade (A, B, C, D, F)
    pub fn performance_grade(&self) -> char {
        if self.avg_query_time_us < 100.0 && self.queries_per_second > 1000.0 {
            'A' // Excellent
        } else if self.avg_query_time_us < 500.0 && self.queries_per_second > 500.0 {
            'B' // Good
        } else if self.avg_query_time_us < 1000.0 && self.queries_per_second > 100.0 {
            'C' // Acceptable
        } else if self.avg_query_time_us < 5000.0 && self.queries_per_second > 10.0 {
            'D' // Poor
        } else {
            'F' // Failing
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Quality statistics for analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStats {
    /// Overall accuracy score (0.0-1.0)
    pub accuracy: f32,
    /// Coverage percentage (0.0-1.0)
    pub coverage: f32,
    /// Average confidence of results (0.0-1.0)
    pub avg_confidence: f32,
    /// Confidence distribution
    pub confidence_distribution: ConfidenceDistribution,
    /// Number of successful analyses
    pub successful_analyses: u64,
    /// Number of failed analyses
    pub failed_analyses: u64,
    /// Quality trends over time
    pub trends: QualityTrends,
}

impl Default for QualityStats {
    fn default() -> Self {
        Self {
            accuracy: 0.0,
            coverage: 0.0,
            avg_confidence: 0.0,
            confidence_distribution: ConfidenceDistribution::default(),
            successful_analyses: 0,
            failed_analyses: 0,
            trends: QualityTrends::default(),
        }
    }
}

/// Confidence score distribution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfidenceDistribution {
    /// High confidence results (>= 0.8)
    pub high: u64,
    /// Medium confidence results (0.5-0.8)
    pub medium: u64,
    /// Low confidence results (< 0.5)
    pub low: u64,
    /// Histogram of confidence scores (10 buckets)
    pub histogram: [u64; 10],
}

impl ConfidenceDistribution {
    /// Record a confidence score
    pub fn record(&mut self, confidence: f32) {
        if confidence >= 0.8 {
            self.high += 1;
        } else if confidence >= 0.5 {
            self.medium += 1;
        } else {
            self.low += 1;
        }

        // Update histogram
        let bucket = ((confidence * 10.0) as usize).min(9);
        self.histogram[bucket] += 1;
    }

    /// Get total number of recorded scores
    pub fn total(&self) -> u64 {
        self.high + self.medium + self.low
    }

    /// Get high confidence rate
    pub fn high_confidence_rate(&self) -> f32 {
        if self.total() == 0 {
            0.0
        } else {
            self.high as f32 / self.total() as f32
        }
    }
}

/// Quality trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrends {
    /// Recent accuracy measurements (last 100 queries)
    pub recent_accuracy: Vec<f32>,
    /// Recent confidence measurements (last 100 queries)
    pub recent_confidence: Vec<f32>,
    /// Trend direction (-1: declining, 0: stable, 1: improving)
    pub trend_direction: i8,
    /// Trend strength (0.0-1.0)
    pub trend_strength: f32,
}

impl QualityTrends {
    /// Record a new quality measurement
    pub fn record(&mut self, accuracy: f32, confidence: f32) {
        // Keep only last 100 measurements
        if self.recent_accuracy.len() >= 100 {
            self.recent_accuracy.remove(0);
            self.recent_confidence.remove(0);
        }

        self.recent_accuracy.push(accuracy);
        self.recent_confidence.push(confidence);

        // Calculate trend
        self.calculate_trend();
    }

    /// Calculate trend direction and strength
    fn calculate_trend(&mut self) {
        if self.recent_accuracy.len() < 10 {
            return;
        }

        // Simple linear regression on recent data
        let n = self.recent_accuracy.len();
        let recent = &self.recent_accuracy[n.saturating_sub(20)..];

        if recent.len() < 2 {
            return;
        }

        let x_sum: f32 = (0..recent.len()).map(|i| i as f32).sum();
        let y_sum: f32 = recent.iter().sum();
        let xy_sum: f32 = recent.iter().enumerate().map(|(i, &y)| i as f32 * y).sum();
        let x_sq_sum: f32 = (0..recent.len()).map(|i| (i as f32).powi(2)).sum();

        let n_f = recent.len() as f32;
        let slope = (n_f * xy_sum - x_sum * y_sum) / (n_f * x_sq_sum - x_sum.powi(2));

        // Determine trend direction
        if slope > 0.01 {
            self.trend_direction = 1; // Improving
        } else if slope < -0.01 {
            self.trend_direction = -1; // Declining
        } else {
            self.trend_direction = 0; // Stable
        }

        // Calculate trend strength
        self.trend_strength = slope.abs().min(1.0);
    }
}

impl Default for QualityTrends {
    fn default() -> Self {
        Self {
            recent_accuracy: Vec::new(),
            recent_confidence: Vec::new(),
            trend_direction: 0,
            trend_strength: 0.0,
        }
    }
}

/// Engine health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineHealth {
    /// Overall health score (0.0-1.0)
    pub overall_score: f32,
    /// Performance health (0.0-1.0)
    pub performance_health: f32,
    /// Quality health (0.0-1.0)
    pub quality_health: f32,
    /// Cache health (0.0-1.0)
    pub cache_health: f32,
    /// Health status
    pub status: HealthStatus,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Health status levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl EngineHealth {
    /// Assess engine health from statistics
    pub fn assess(stats: &EngineStats) -> Self {
        // Calculate component health scores
        let performance_health = Self::assess_performance(&stats.performance);
        let quality_health = Self::assess_quality(&stats.quality);
        let cache_health = Self::assess_cache(&stats.cache);

        // Calculate overall score (weighted average)
        let overall_score = performance_health * 0.4 + quality_health * 0.4 + cache_health * 0.2;

        // Determine status
        let status = match overall_score {
            score if score >= 0.9 => HealthStatus::Excellent,
            score if score >= 0.8 => HealthStatus::Good,
            score if score >= 0.6 => HealthStatus::Fair,
            score if score >= 0.4 => HealthStatus::Poor,
            _ => HealthStatus::Critical,
        };

        // Generate recommendations
        let recommendations = Self::generate_recommendations(stats);

        Self {
            overall_score,
            performance_health,
            quality_health,
            cache_health,
            status,
            recommendations,
        }
    }

    fn assess_performance(perf: &PerformanceMetrics) -> f32 {
        let mut score: f32 = 1.0;

        // Penalize slow average query times
        if perf.avg_query_time_us > 1000.0 {
            score *= 0.8;
        }
        if perf.avg_query_time_us > 5000.0 {
            score *= 0.5;
        }

        // Penalize low QPS
        if perf.queries_per_second < 100.0 {
            score *= 0.8;
        }
        if perf.queries_per_second < 10.0 {
            score *= 0.5;
        }

        score.clamp(0.0, 1.0)
    }

    fn assess_quality(quality: &QualityStats) -> f32 {
        let mut score = quality.avg_confidence;

        // Boost score for high accuracy
        if quality.accuracy > 0.9 {
            score *= 1.1;
        }

        // Penalize low coverage
        if quality.coverage < 0.5 {
            score *= 0.8;
        }

        score.clamp(0.0, 1.0)
    }

    fn assess_cache(cache: &crate::CacheStats) -> f32 {
        if cache.total_lookups == 0 {
            return 1.0; // No cache usage yet
        }

        cache.hit_rate as f32
    }

    fn generate_recommendations(stats: &EngineStats) -> Vec<String> {
        let mut recommendations = Vec::new();

        if stats.performance.avg_query_time_us > 1000.0 {
            recommendations
                .push("Consider optimizing query processing for better latency".to_string());
        }

        if stats.cache.hit_rate < 0.7 {
            recommendations
                .push("Consider increasing cache size or reviewing cache strategy".to_string());
        }

        if stats.quality.avg_confidence < 0.7 {
            recommendations.push("Review data quality and analysis algorithms".to_string());
        }

        if stats.quality.coverage < 0.8 {
            recommendations.push("Consider expanding data sources to improve coverage".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_recording() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_query(500); // 0.5ms
        metrics.record_query(1000); // 1ms
        metrics.record_query(1500); // 1.5ms

        assert_eq!(metrics.total_queries, 3);
        assert_eq!(metrics.avg_query_time_us, 1000.0);
        assert_eq!(metrics.min_query_time_us, 500);
        assert_eq!(metrics.max_query_time_us, 1500);
    }

    #[test]
    fn test_confidence_distribution() {
        let mut dist = ConfidenceDistribution::default();

        dist.record(0.9); // High
        dist.record(0.7); // Medium
        dist.record(0.3); // Low

        assert_eq!(dist.high, 1);
        assert_eq!(dist.medium, 1);
        assert_eq!(dist.low, 1);
        assert_eq!(dist.total(), 3);
        assert!((dist.high_confidence_rate() - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_engine_health_assessment() {
        let stats = EngineStats::new("TestEngine".to_string());
        let health = EngineHealth::assess(&stats);

        assert!(health.overall_score >= 0.0 && health.overall_score <= 1.0);
        assert!(matches!(
            health.status,
            HealthStatus::Excellent
                | HealthStatus::Good
                | HealthStatus::Fair
                | HealthStatus::Poor
                | HealthStatus::Critical
        ));
    }
}
