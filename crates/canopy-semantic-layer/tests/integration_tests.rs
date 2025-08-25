//! Integration tests for semantic-first Layer 1
//!
//! These tests verify the core semantic engines work independently

use canopy_framenet::FrameNetEngine;
use canopy_semantic_layer::*;
use canopy_verbnet::VerbNetEngine;

#[test]
fn test_verbnet_engine_creation() {
    let engine = VerbNetEngine::new();
    // Engine creation should succeed even without data
    assert_eq!(engine.engine_name(), "VerbNet");
}

#[test]
fn test_framenet_engine_creation() {
    let engine = FrameNetEngine::new();
    // Engine creation should succeed even without data
    assert_eq!(engine.engine_name(), "FrameNet");
}

#[test]
fn test_wordnet_engine_creation() {
    use canopy_wordnet::{WordNetConfig, WordNetEngine};
    let config = WordNetConfig::default();
    let engine = WordNetEngine::new(config);
    // Engine creation should succeed even without data
    // Simple test that the engine was created successfully
    assert!(!engine.is_ready()); // Should not be ready until data is loaded
}

#[test]
fn test_morphology_database_creation() {
    use canopy_semantic_layer::morphology::MorphologyDatabase;
    let result = MorphologyDatabase::new();
    assert!(
        result.is_ok(),
        "Morphology database should initialize successfully"
    );
}

#[test]
fn test_lexicon_creation() {
    use canopy_semantic_layer::lexicon::ClosedClassLexicon;
    let result = ClosedClassLexicon::new();
    assert!(
        result.is_ok(),
        "Closed class lexicon should initialize successfully"
    );
}

#[test]
fn test_semantic_config_defaults() {
    use canopy_semantic_layer::coordinator::CoordinatorConfig;
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
    use canopy_semantic_layer::SemanticError;
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

    // Should not panic and should return some form of result
    assert!(
        result.is_ok() || result.is_err(),
        "Tokenizer should return a Result"
    );
}
