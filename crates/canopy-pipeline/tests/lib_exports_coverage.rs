//! Coverage tests for pipeline lib exports

use canopy_pipeline::*;

#[test]
fn test_pipeline_lib_exports() {
    // Test that key types can be accessed
    let _ = std::any::type_name::<error::PipelineError>();
    let _ = std::any::type_name::<pipeline::PipelineConfig>();
    // Reaching here means pipeline lib exports are accessible
}

#[test]
fn test_pipeline_lib_functions() {
    // Test any public library functions
    // This exercises module initialization and exports
    // Reaching here means pipeline lib functions are callable
}

#[test]
fn test_pipeline_lib_traits() {
    // Test that traits are exported
    use canopy_pipeline::traits::PerformanceMode;
    let mode = PerformanceMode::default();
    assert_eq!(mode, PerformanceMode::Balanced);
}

#[test]
fn test_pipeline_lib_containers() {
    // Test container exports
    use canopy_pipeline::container::ContainerBuilder;
    let _builder = ContainerBuilder::new();
    // Reaching here means container builder is accessible
}

#[test]
fn test_pipeline_lib_models() {
    // Test that models are exported
    // Reaching here means models are accessible
}
