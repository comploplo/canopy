//! Basic coverage tests for pipeline traits module

use canopy_pipeline::traits::*;

#[test]
fn test_performance_mode() {
    let default_mode = PerformanceMode::default();
    assert_eq!(default_mode, PerformanceMode::Balanced);
    assert_ne!(PerformanceMode::Speed, PerformanceMode::Accuracy);
}

#[test]
fn test_model_type() {
    assert_eq!(ModelType::UDPipe12, ModelType::UDPipe12);
    assert_ne!(ModelType::UDPipe12, ModelType::UDPipe215);
}

#[test]
fn test_feature_set() {
    let feature_set = FeatureSet::default();
    assert!(feature_set.morphological.is_empty());
    assert!(feature_set.verbnet.is_none());
}

#[test]
fn test_cache_stats() {
    let stats = CacheStats::default();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
}

#[test]
fn test_metrics() {
    let metrics = Metrics::default();
    assert!(metrics.timings.is_empty());
    assert!(metrics.counts.is_empty());
}
