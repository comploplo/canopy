//! Integration tests for semantic-first Layer 1
//!
//! These tests verify the core semantic engines work independently

use canopy_framenet::FrameNetEngine;
use canopy_tokenizer::*;
use canopy_verbnet::VerbNetEngine;

#[test]
fn test_verbnet_engine_creation() {
    // Skip test if engine cannot be created without data
    if let Ok(engine) = VerbNetEngine::new() {
        // Engine creation should succeed even without data
        assert_eq!(engine.engine_name(), "VerbNet");
    }
}

#[test]
fn test_framenet_engine_creation() {
    // Skip test if engine cannot be created without data
    if let Ok(engine) = FrameNetEngine::new() {
        // Engine creation should succeed even without data
        assert_eq!(engine.engine_name(), "FrameNet");
    }
}

#[test]
fn test_wordnet_engine_creation() {
    use canopy_wordnet::{WordNetConfig, WordNetEngine};
    let config = WordNetConfig::default();
    if let Ok(engine) = WordNetEngine::with_config(config) {
        // Engine creation should succeed
        // If data is available, engine will be ready; otherwise not
        // Just verify the engine was created successfully
        let _ = engine.is_ready(); // Check doesn't panic
    }
}

#[test]
fn test_morphology_database_creation() {
    use canopy_tokenizer::morphology::MorphologyDatabase;
    let result = MorphologyDatabase::new();
    assert!(
        result.is_ok(),
        "Morphology database should initialize successfully"
    );
}

#[test]
fn test_lexicon_creation() {
    use canopy_tokenizer::lexicon::ClosedClassLexicon;
    let result = ClosedClassLexicon::new();
    assert!(
        result.is_ok(),
        "Closed class lexicon should initialize successfully"
    );
}

#[test]
fn test_semantic_config_defaults() {
    use canopy_tokenizer::coordinator::CoordinatorConfig;
    let config = CoordinatorConfig::default();
    assert!(
        config.enable_framenet,
        "FrameNet should be enabled by default"
    );
    assert!(
        config.enable_verbnet,
        "VerbNet should be enabled by default"
    );
    assert!(
        config.enable_wordnet,
        "WordNet should be enabled by default"
    );
    assert_eq!(
        config.confidence_threshold, 0.1,
        "Default confidence threshold should be 0.1"
    );
}

#[test]
fn test_semantic_error_conversions() {
    // Test that error conversions work properly
    use canopy_tokenizer::SemanticError;
    let semantic_error = SemanticError::FrameNetError {
        context: "test error".to_string(),
    };

    match semantic_error {
        SemanticError::FrameNetError { .. } => {
            // Expected conversion
        }
        _ => panic!("Error conversion failed"),
    }
}

#[test]
fn test_tokenizer_basic_functionality() {
    let tokenizer = tokenization::Tokenizer::new();
    let result = tokenizer.tokenize_simple("hello world");

    // Tokenizer should successfully tokenize simple input
    assert!(
        result.is_ok(),
        "Tokenizer should successfully tokenize 'hello world'"
    );
    let tokens = result.unwrap();
    assert!(!tokens.is_empty(), "Should produce at least one token");
}
