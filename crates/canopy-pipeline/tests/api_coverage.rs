//! Coverage tests for pipeline API module

use canopy_pipeline::api::AnalysisConfig;

#[test]
fn test_analysis_config() {
    // Test AnalysisConfig structure
    let config = AnalysisConfig {
        enable_caching: true,
        performance_mode: "balanced".to_string(),
    };
    assert!(config.enable_caching);
    assert_eq!(config.performance_mode, "balanced");
}

#[test]
fn test_analysis_config_default_values() {
    // Test with different values
    let config = AnalysisConfig {
        enable_caching: false,
        performance_mode: "fast".to_string(),
    };
    assert!(!config.enable_caching);
    assert_eq!(config.performance_mode, "fast");
}
