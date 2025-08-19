//! Simple coverage tests for parser utilities
//!
//! These tests target uncovered utility functions and edge cases to improve
//! overall test coverage without requiring complex setup.

use crate::metrics::*;

#[cfg(test)]
mod simple_coverage_tests {
    use super::*;

    #[test]
    fn test_input_size_category_edge_cases() {
        // Test edge cases for input size categorization
        assert_eq!(
            InputSizeCategory::from_word_count(0),
            InputSizeCategory::Oversized
        );
        assert_eq!(
            InputSizeCategory::from_word_count(1),
            InputSizeCategory::Small
        );
        assert_eq!(
            InputSizeCategory::from_word_count(10),
            InputSizeCategory::Small
        );
        assert_eq!(
            InputSizeCategory::from_word_count(11),
            InputSizeCategory::Medium
        );
        assert_eq!(
            InputSizeCategory::from_word_count(25),
            InputSizeCategory::Medium
        );
        assert_eq!(
            InputSizeCategory::from_word_count(26),
            InputSizeCategory::Large
        );
        assert_eq!(
            InputSizeCategory::from_word_count(50),
            InputSizeCategory::Large
        );
        assert_eq!(
            InputSizeCategory::from_word_count(51),
            InputSizeCategory::ExtraLarge
        );
        assert_eq!(
            InputSizeCategory::from_word_count(100),
            InputSizeCategory::ExtraLarge
        );
        assert_eq!(
            InputSizeCategory::from_word_count(101),
            InputSizeCategory::Oversized
        );
        assert_eq!(
            InputSizeCategory::from_word_count(1000),
            InputSizeCategory::Oversized
        );
    }

    #[test]
    fn test_performance_threshold_values() {
        // Test performance threshold for each category
        assert_eq!(InputSizeCategory::Small.performance_threshold_us(), 100);
        assert_eq!(InputSizeCategory::Medium.performance_threshold_us(), 300);
        assert_eq!(InputSizeCategory::Large.performance_threshold_us(), 500);
        assert_eq!(
            InputSizeCategory::ExtraLarge.performance_threshold_us(),
            1000
        );
        assert_eq!(
            InputSizeCategory::Oversized.performance_threshold_us(),
            2000
        );
    }

    #[test]
    fn test_parsing_metrics_default() {
        // Test default implementation for ParsingMetrics
        let metrics = ParsingMetrics::default();
        assert_eq!(metrics.sample_count, 0);
        assert_eq!(metrics.total_time, std::time::Duration::ZERO);
        assert_eq!(metrics.min_time, std::time::Duration::MAX);
        assert_eq!(metrics.max_time, std::time::Duration::ZERO);
        assert_eq!(metrics.avg_time, std::time::Duration::ZERO);
        assert_eq!(metrics.p95_time, std::time::Duration::ZERO);
        assert_eq!(metrics.p99_time, std::time::Duration::ZERO);
        assert_eq!(metrics.std_dev, std::time::Duration::ZERO);
        assert_eq!(metrics.threshold_violations, 0);
    }

    #[test]
    fn test_cache_metrics_edge_cases() {
        // Test cache metrics with edge cases
        let mut metrics = CacheMetrics::default();

        // Test with no hits or misses
        assert_eq!(metrics.hit_rate(), 0.0);

        // Test with only hits
        metrics.hits = 10;
        metrics.misses = 0;
        assert_eq!(metrics.hit_rate(), 1.0);

        // Test with only misses
        metrics.hits = 0;
        metrics.misses = 10;
        assert_eq!(metrics.hit_rate(), 0.0);

        // Test with mixed hits and misses
        metrics.hits = 7;
        metrics.misses = 3;
        assert_eq!(metrics.hit_rate(), 0.7);
    }

    #[test]
    fn test_performance_grade_descriptions() {
        // Test PerformanceGrade descriptions and emojis
        assert_eq!(
            PerformanceGrade::Excellent.description(),
            "Excellent performance - meets all targets"
        );
        assert_eq!(
            PerformanceGrade::Good.description(),
            "Good performance - minor threshold violations"
        );
        assert_eq!(
            PerformanceGrade::Acceptable.description(),
            "Acceptable performance - some optimization needed"
        );
        assert_eq!(
            PerformanceGrade::NeedsOptimization.description(),
            "Performance needs optimization - consider caching"
        );
        assert_eq!(
            PerformanceGrade::Poor.description(),
            "Poor performance - requires immediate attention"
        );

        // Test emojis
        assert_eq!(PerformanceGrade::Excellent.emoji(), "🚀");
        assert_eq!(PerformanceGrade::Good.emoji(), "✅");
        assert_eq!(PerformanceGrade::Acceptable.emoji(), "⚖️");
        assert_eq!(PerformanceGrade::NeedsOptimization.emoji(), "⚠️");
        assert_eq!(PerformanceGrade::Poor.emoji(), "❌");
    }

    #[test]
    fn test_performance_warning_variants() {
        // Test PerformanceWarning enum variants for coverage
        let threshold_warning = PerformanceWarning::ThresholdExceeded {
            actual_us: 600,
            threshold_us: 300,
            input_size: InputSizeCategory::Medium,
        };

        let memory_warning = PerformanceWarning::MemoryExceeded {
            actual_bytes: 1000000,
            expected_bytes: 500000,
        };

        let model_warning = PerformanceWarning::SlowModelLoad { load_time_ms: 2000 };

        let cache_warning = PerformanceWarning::PoorCachePerformance {
            hit_rate: 0.3,
            expected_hit_rate: 0.8,
        };

        let udpipe_warning = PerformanceWarning::UDPipe2Performance {
            message: "Performance degraded".to_string(),
            recommendation: "Consider caching".to_string(),
        };

        // Test debug formatting
        let threshold_debug = format!("{:?}", threshold_warning);
        let memory_debug = format!("{:?}", memory_warning);
        let model_debug = format!("{:?}", model_warning);
        let cache_debug = format!("{:?}", cache_warning);
        let udpipe_debug = format!("{:?}", udpipe_warning);

        assert!(threshold_debug.contains("ThresholdExceeded"));
        assert!(memory_debug.contains("MemoryExceeded"));
        assert!(model_debug.contains("SlowModelLoad"));
        assert!(cache_debug.contains("PoorCachePerformance"));
        assert!(udpipe_debug.contains("UDPipe2Performance"));
    }

    #[test]
    fn test_memory_metrics_default() {
        // Test MemoryMetrics default initialization
        let metrics = MemoryMetrics::default();
        assert_eq!(metrics.peak_memory_bytes, 0);
        assert_eq!(metrics.avg_memory_bytes, 0);
        assert_eq!(metrics.allocations, 0);
        assert_eq!(metrics.deallocations, 0);
        assert_eq!(metrics.leaks, 0);
    }

    #[test]
    fn test_latency_distribution_default() {
        // Test LatencyDistribution default initialization
        let distribution = LatencyDistribution::default();
        assert!(distribution.histogram.is_empty());
        assert!(distribution.percentiles.is_empty());
    }

    #[test]
    fn test_udpipe_performance_metrics_creation() {
        // Test UDPipePerformanceMetrics basic structure
        let metrics = UDPipePerformanceMetrics {
            version: "2.15".to_string(),
            model_path: Some("/test/model.udpipe".to_string()),
            model_load_time: Some(std::time::Duration::from_millis(500)),
            parsing_metrics: std::collections::HashMap::new(),
            cache_metrics: CacheMetrics::default(),
            memory_metrics: MemoryMetrics::default(),
            latency_distribution: LatencyDistribution::default(),
            warnings: vec![],
        };

        assert_eq!(metrics.version, "2.15");
        assert!(metrics.model_path.is_some());
        assert!(metrics.model_load_time.is_some());
        assert!(metrics.parsing_metrics.is_empty());
        assert!(metrics.warnings.is_empty());
    }

    #[test]
    fn test_performance_summary_creation() {
        // Test PerformanceSummary structure
        let summary = PerformanceSummary {
            version: "2.15".to_string(),
            total_samples: 100,
            avg_processing_time: std::time::Duration::from_micros(250),
            threshold_violations: 5,
            cache_hit_rate: 0.85,
            warnings_count: 2,
            performance_grade: PerformanceGrade::Good,
        };

        assert_eq!(summary.version, "2.15");
        assert_eq!(summary.total_samples, 100);
        assert_eq!(summary.threshold_violations, 5);
        assert!((summary.cache_hit_rate - 0.85).abs() < f64::EPSILON);
        assert_eq!(summary.warnings_count, 2);
        assert_eq!(summary.performance_grade, PerformanceGrade::Good);
    }
}
