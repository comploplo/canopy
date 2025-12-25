//! Integration tests for FrameNet engine with BaseEngine infrastructure
//!
//! These tests verify that FrameNet properly loads and uses real data.
//! With workspace-relative path resolution, engines find their data automatically.
//!
//! Uses singleton pattern to load FrameNet data once per test binary (~50s),
//! rather than reloading for each test (which would take 200+ seconds total).

use canopy_engine::{EngineCore, SemanticEngine};
use canopy_framenet::FrameNetEngine;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

// Shared engine singleton - loaded once per test binary
static SHARED_ENGINE: OnceCell<Mutex<FrameNetEngine>> = OnceCell::new();

fn shared_engine() -> &'static Mutex<FrameNetEngine> {
    SHARED_ENGINE.get_or_init(|| {
        eprintln!("🔧 Loading shared FrameNet engine (one-time)...");
        Mutex::new(FrameNetEngine::new().expect("FrameNet data required for tests"))
    })
}

#[test]
fn test_framenet_engine_baseengine_integration() {
    let engine = shared_engine().lock().unwrap();

    // With proper path resolution, engine should be initialized
    assert!(
        SemanticEngine::is_initialized(&*engine),
        "Engine should be initialized"
    );
    assert!(engine.is_loaded(), "Engine should be loaded");

    println!("📊 Engine Statistics:");
    let all_frames = engine.get_all_frames();
    let all_lus = engine.get_all_lexical_units();
    println!("  Frames: {}", all_frames.len());
    println!("  Lexical Units: {}", all_lus.len());
    assert!(!all_frames.is_empty(), "Should have loaded frames");

    // Test BaseEngine integration
    println!("🔧 BaseEngine Integration:");
    let stats = engine.statistics();
    println!("  Total frames: {}", stats.total_frames);
    println!("  Total lexical units: {}", stats.total_lexical_units);
    assert!(stats.total_frames > 0, "Should have loaded frames");

    let cache_stats = engine.cache_stats();
    println!("  Cache hits: {}", cache_stats.hits);
    println!("  Cache misses: {}", cache_stats.misses);

    // Test semantic analysis through BaseEngine
    let result = engine.analyze_text("give").unwrap();
    println!("🎯 Semantic Analysis Test:");
    println!("  Input: 'give'");
    println!("  Frames found: {}", result.data.frames.len());
    println!("  Confidence: {:.2}", result.confidence);
    println!("  From cache: {}", result.from_cache);

    if !result.data.frames.is_empty() {
        for frame in &result.data.frames {
            println!("  - Frame: {} ({})", frame.name, frame.id);
            println!("    Core elements: {}", frame.core_elements().len());
        }
    }
}

#[test]
fn test_framenet_engine_with_loaded_data() {
    let engine = shared_engine().lock().unwrap();

    // Engine auto-loads data on creation
    assert!(
        SemanticEngine::is_initialized(&*engine),
        "Engine should be initialized"
    );
    assert!(engine.is_loaded(), "Engine should be loaded");

    // Statistics should reflect loaded data
    let stats = engine.statistics();
    assert!(stats.total_frames > 0, "Should have frames");
}

#[test]
fn test_framenet_caching_behavior() {
    let engine = shared_engine().lock().unwrap();

    // Test cache behavior - capture initial state (may have activity from other tests)
    let initial_stats = engine.cache_stats();
    let initial_hits = initial_stats.hits;
    let initial_misses = initial_stats.misses;

    // First analysis (cache miss for this word)
    let result1 = engine.analyze_text("testing").unwrap();

    // Second analysis (should be cache hit)
    let result2 = engine.analyze_text("testing").unwrap();

    // Results should be consistent
    assert_eq!(result1.confidence, result2.confidence);
    assert_eq!(result1.data.frames.len(), result2.data.frames.len());

    // Cache should show activity
    let final_stats = engine.cache_stats();
    assert!(
        final_stats.hits > initial_hits || final_stats.misses > initial_misses,
        "Cache should show some activity"
    );

    // Second result should be from cache
    assert!(result2.from_cache, "Second lookup should be from cache");
}

#[test]
fn test_framenet_config_and_traits() {
    let engine = shared_engine().lock().unwrap();

    // Test trait implementations
    assert_eq!(engine.name(), "FrameNet");
    assert_eq!(engine.version(), "1.7");
    assert_eq!(engine.engine_name(), "FrameNet");
    assert_eq!(engine.engine_version(), "1.7");

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
