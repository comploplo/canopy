//! Basic tests for benchmarks.rs module structures

use canopy_pipeline::benchmarks::*;

#[cfg(test)]
mod benchmarks_tests {
    use super::*;

    #[test]
    fn test_benchmark_config_creation() {
        let config = BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 10,
            test_sentences: vec!["John runs".to_string(), "Mary walks".to_string()],
        };

        assert_eq!(config.iterations, 100);
        assert_eq!(config.warmup_iterations, 10);
        assert_eq!(config.test_sentences.len(), 2);
    }

    #[test]
    fn test_benchmark_results_creation() {
        let results = BenchmarkResults {
            total_time: std::time::Duration::from_millis(1000),
            avg_time_per_text: std::time::Duration::from_millis(20),
            throughput_texts_per_sec: 50.0,
            memory_usage_mb: 10.5,
        };

        assert_eq!(results.total_time.as_millis(), 1000);
        assert_eq!(results.avg_time_per_text.as_millis(), 20);
        assert_eq!(results.throughput_texts_per_sec, 50.0);
        assert_eq!(results.memory_usage_mb, 10.5);
    }

    #[test]
    fn test_performance_profile_creation() {
        let profile = PerformanceProfile {
            model_name: "test_profile".to_string(),
            avg_latency_ms: 17.5,
            p95_latency_ms: 25.0,
            p99_latency_ms: 30.0,
            throughput: 50.0,
            memory_usage: 10.5,
        };

        assert_eq!(profile.model_name, "test_profile");
        assert_eq!(profile.avg_latency_ms, 17.5);
        assert_eq!(profile.p95_latency_ms, 25.0);
        assert_eq!(profile.p99_latency_ms, 30.0);
        assert_eq!(profile.throughput, 50.0);
        assert_eq!(profile.memory_usage, 10.5);
    }

    #[test]
    fn test_model_comparison_creation() {
        let comparison = ModelComparison {
            model1: "UDPipe 1.2".to_string(),
            model2: "UDPipe 2.15".to_string(),
            performance_ratio: 1.18,
            accuracy_comparison: Some(AccuracyComparison {
                model1_accuracy: 0.92,
                model2_accuracy: 0.94,
                difference: 0.02,
            }),
        };

        assert_eq!(comparison.model1, "UDPipe 1.2");
        assert_eq!(comparison.model2, "UDPipe 2.15");
        assert_eq!(comparison.performance_ratio, 1.18);
        assert!(comparison.accuracy_comparison.is_some());
        if let Some(acc) = comparison.accuracy_comparison {
            assert_eq!(acc.difference, 0.02);
        }
    }

    #[test]
    fn test_accuracy_comparison_creation() {
        let accuracy = AccuracyComparison {
            model1_accuracy: 0.90,
            model2_accuracy: 0.92,
            difference: 0.02,
        };

        assert_eq!(accuracy.model1_accuracy, 0.90);
        assert_eq!(accuracy.model2_accuracy, 0.92);
        assert_eq!(accuracy.difference, 0.02);
    }

    #[test]
    fn test_run_model_comparison_panics() {
        // Test that the unimplemented function panics
        let result = std::panic::catch_unwind(|| {
            run_model_comparison(vec!["model_a".to_string(), "model_b".to_string()])
        });
        assert!(result.is_err());
    }
}
