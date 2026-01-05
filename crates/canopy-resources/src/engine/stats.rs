//! Statistics and performance metrics for semantic engines

use super::cache::CacheStats;
use serde::{Deserialize, Serialize};

/// Convert u64 to f64 for statistics (saturates at `u32::MAX` for lossless conversion).
#[inline]
fn stats_u64_to_f64(n: u64) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}
use std::time::{Duration, SystemTime};

/// Comprehensive engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub engine_name: String,
    pub data: DataStats,
    pub performance: PerformanceMetrics,
    pub cache: CacheStats,
}

impl EngineStats {
    /// Create new engine statistics
    #[must_use]
    pub fn new(engine_name: String) -> Self {
        Self {
            engine_name,
            data: DataStats::default(),
            performance: PerformanceMetrics::default(),
            cache: CacheStats::empty(),
        }
    }
}

/// Statistics about loaded data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStats {
    pub total_entries: usize,
    pub unique_keys: usize,
    pub format_version: String,
    pub memory_size_bytes: usize,
    pub data_source: String,
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
    pub total_queries: u64,
    pub avg_query_time_us: f64,
    pub min_query_time_us: u64,
    pub max_query_time_us: u64,
    pub p95_query_time_us: u64,
    pub p99_query_time_us: u64,
    pub queries_per_second: f64,
    pub total_processing_time_ms: u64,
    pub uptime_secs: u64,
    start_time_secs: u64,
    query_times: Vec<u64>,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    #[must_use]
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

        self.min_query_time_us = self.min_query_time_us.min(duration_us);
        self.max_query_time_us = self.max_query_time_us.max(duration_us);

        self.query_times.push(duration_us);
        if self.query_times.len() > 10000 {
            self.query_times.remove(0);
        }

        self.avg_query_time_us = (self.avg_query_time_us
            * stats_u64_to_f64(self.total_queries - 1)
            + stats_u64_to_f64(duration_us))
            / stats_u64_to_f64(self.total_queries);

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        self.uptime_secs = now.saturating_sub(self.start_time_secs);

        if self.uptime_secs > 0 {
            self.queries_per_second =
                stats_u64_to_f64(self.total_queries) / stats_u64_to_f64(self.uptime_secs);
        }

        self.calculate_percentiles();
    }

    fn calculate_percentiles(&mut self) {
        if self.query_times.is_empty() {
            return;
        }

        let mut sorted_times = self.query_times.clone();
        sorted_times.sort_unstable();

        let len = sorted_times.len();
        // Calculate percentile indices using integer arithmetic to avoid float casts
        let p95_index = len.saturating_mul(95) / 100;
        let p99_index = len.saturating_mul(99) / 100;

        self.p95_query_time_us = sorted_times
            .get(p95_index.min(len - 1))
            .copied()
            .unwrap_or(0);
        self.p99_query_time_us = sorted_times
            .get(p99_index.min(len - 1))
            .copied()
            .unwrap_or(0);
    }

    /// Get performance grade (A, B, C, D, F)
    #[must_use]
    pub fn performance_grade(&self) -> char {
        if self.avg_query_time_us < 100.0 && self.queries_per_second > 1000.0 {
            'A'
        } else if self.avg_query_time_us < 500.0 && self.queries_per_second > 500.0 {
            'B'
        } else if self.avg_query_time_us < 1000.0 && self.queries_per_second > 100.0 {
            'C'
        } else if self.avg_query_time_us < 5000.0 && self.queries_per_second > 10.0 {
            'D'
        } else {
            'F'
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_recording() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_query(500);
        metrics.record_query(1000);
        metrics.record_query(1500);

        assert_eq!(metrics.total_queries, 3);
        assert!((metrics.avg_query_time_us - 1000.0).abs() < f64::EPSILON);
        assert_eq!(metrics.min_query_time_us, 500);
        assert_eq!(metrics.max_query_time_us, 1500);
    }

    #[test]
    fn test_engine_stats_creation() {
        let stats = EngineStats::new("TestEngine".to_string());
        assert_eq!(stats.engine_name, "TestEngine");
    }

    #[test]
    fn test_performance_grade() {
        let mut metrics = PerformanceMetrics::new();
        // Initially F grade
        assert_eq!(metrics.performance_grade(), 'F');

        // Record enough queries to get different grades
        for _ in 0..1000 {
            metrics.record_query(50);
        }
        // With low latency and high throughput
        metrics.queries_per_second = 2000.0;
        metrics.avg_query_time_us = 50.0;
        assert_eq!(metrics.performance_grade(), 'A');

        metrics.avg_query_time_us = 300.0;
        metrics.queries_per_second = 600.0;
        assert_eq!(metrics.performance_grade(), 'B');

        metrics.avg_query_time_us = 800.0;
        metrics.queries_per_second = 200.0;
        assert_eq!(metrics.performance_grade(), 'C');

        metrics.avg_query_time_us = 3000.0;
        metrics.queries_per_second = 50.0;
        assert_eq!(metrics.performance_grade(), 'D');
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.total_queries, 0);
    }
}
