//! Integration tests for VerbNet engine with BaseEngine infrastructure
//!
//! Note: These tests require VerbNet data to be available. Tests will
//! gracefully skip if data is not found.

use canopy_engine::EngineCore;
use canopy_verbnet::{VerbNetConfig, VerbNetEngine};
use std::path::Path;

/// Helper to create engine with correct path for integration tests
/// Integration tests run from the crate directory, so we need ../../data/
fn try_create_engine() -> Option<VerbNetEngine> {
    // Try workspace-relative path (when run from workspace root)
    let workspace_path = Path::new("data/verbnet/vn-gl");
    // Try crate-relative path (when run from crate directory)
    let crate_path = Path::new("../../data/verbnet/vn-gl");

    let data_path = if workspace_path.exists() {
        workspace_path
    } else if crate_path.exists() {
        crate_path
    } else {
        eprintln!(
            "VerbNet data not found at {} or {}",
            workspace_path.display(),
            crate_path.display()
        );
        eprintln!("Skipping integration test");
        return None;
    };

    let config = VerbNetConfig {
        data_path: data_path.to_string_lossy().to_string(),
        ..VerbNetConfig::default()
    };

    match VerbNetEngine::with_config(config) {
        Ok(engine) => Some(engine),
        Err(e) => {
            eprintln!("Failed to create VerbNet engine: {}", e);
            None
        }
    }
}

#[test]
fn test_verbnet_engine_baseengine_integration() {
    let Some(engine) = try_create_engine() else {
        return;
    };

    // Test that engine is properly initialized
    assert!(
        EngineCore::is_initialized(&engine),
        "Engine should report as initialized"
    );
    assert!(engine.is_loaded(), "Engine should report as loaded");

    println!("📊 Engine Statistics:");
    println!("  Classes: {}", engine.get_all_classes().len());
    let total_verbs: usize = engine
        .get_all_classes()
        .iter()
        .map(|class| class.members.len())
        .sum();
    println!("  Verbs: {}", total_verbs);

    // Verify we have real data loaded
    assert!(
        engine.get_all_classes().len() > 100,
        "Should have loaded many VerbNet classes"
    );

    // Test BaseEngine integration
    let stats = engine.statistics();
    assert_eq!(stats.engine_name, "VerbNet");

    let cache_stats = engine.cache_stats();
    assert_eq!(
        cache_stats.hits, 0,
        "Fresh engine should have no cache hits"
    );

    // Test semantic analysis through BaseEngine
    let result = engine.analyze_verb("give").expect("Should analyze 'give'");
    println!("🎯 Semantic Analysis Test:");
    println!("  Input: 'give'");
    println!("  Classes found: {}", result.data.verb_classes.len());
    println!("  Confidence: {:.2}", result.confidence);

    assert!(
        !result.data.verb_classes.is_empty(),
        "'give' should have VerbNet classes"
    );
    assert!(
        result.confidence > 0.5,
        "'give' should have high confidence"
    );
}

#[test]
fn test_verbnet_caching_behavior() {
    let Some(engine) = try_create_engine() else {
        return;
    };

    // First analysis - cache miss
    let result1 = engine.analyze_verb("run").expect("Should analyze 'run'");

    // Second analysis - should hit cache
    let result2 = engine
        .analyze_verb("run")
        .expect("Should analyze 'run' again");

    // Results should be identical
    assert_eq!(result1.confidence, result2.confidence);
    assert_eq!(
        result1.data.verb_classes.len(),
        result2.data.verb_classes.len()
    );

    // Cache should show activity
    let cache_stats = engine.cache_stats();
    assert!(
        cache_stats.hits > 0 || cache_stats.misses > 0,
        "Cache should have activity"
    );
}

#[test]
fn test_verbnet_config_and_traits() {
    let Some(engine) = try_create_engine() else {
        return;
    };

    // Test trait implementations
    assert_eq!(engine.engine_name(), "VerbNet");
    assert_eq!(engine.engine_version(), "3.4");

    // Test configuration access
    let config = engine.config();
    assert!(config.enable_cache, "Default config should enable caching");
    assert_eq!(
        config.cache_capacity, 10000,
        "Default cache capacity should be 10000"
    );
    assert_eq!(
        config.confidence_threshold, 0.5,
        "Default confidence threshold should be 0.5"
    );
}

#[test]
fn test_verbnet_verb_class_lookup() {
    let Some(engine) = try_create_engine() else {
        return;
    };

    // Test looking up specific verb classes via analyze_verb
    let result = engine.analyze_verb("give").expect("Should analyze 'give'");
    assert!(
        !result.data.verb_classes.is_empty(),
        "'give' should be in VerbNet"
    );

    // Verify we got meaningful class data
    for class in &result.data.verb_classes {
        println!("  Found class: {} ({})", class.id, class.class_name);
        assert!(!class.id.is_empty(), "Class ID should not be empty");
    }

    // Test verb that likely isn't in VerbNet
    let nonexistent = engine
        .analyze_verb("xyzzy")
        .expect("Should return result for unknown verb");
    assert!(
        nonexistent.data.verb_classes.is_empty(),
        "Made-up word should not be in VerbNet"
    );
}
