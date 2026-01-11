//! Integration tests for LSP handlers
//!
//! These tests use a real CanopyPipeline to test the handlers.

use canopy_lsp::analysis::{extract_sentences, AnalysisCache, PositionMapper};
use canopy_lsp::state::{CachedDocument, DocumentState, SentenceSpan};
use canopy_lsp::tokens::encoder::SemanticTokenEncoder;
use canopy_lsp::tokens::legend::{semantic_token_legend, ThetaTokenType, TokenModifier};
use canopy_resources::CanopyPipeline;
use tower_lsp::lsp_types::*;

/// Test document state operations
#[test]
fn test_document_state_lifecycle() {
    let state = DocumentState::new();
    let uri = Url::parse("file:///test.txt").unwrap();

    // Open document
    state.open(
        uri.clone(),
        "The cat sat on the mat.".to_string(),
        1,
        "plaintext".to_string(),
    );

    assert!(state.contains(&uri));
    assert_eq!(state.len(), 1);

    // Get document
    let doc = state.get(&uri).unwrap();
    assert_eq!(doc.content, "The cat sat on the mat.");
    assert_eq!(doc.version, 1);
    assert_eq!(doc.language_id, "plaintext");
    drop(doc);

    // Update document
    {
        let mut doc = state.get_mut(&uri).unwrap();
        doc.update_content("The dog ran.".to_string(), 2);
    }

    let doc = state.get(&uri).unwrap();
    assert_eq!(doc.content, "The dog ran.");
    assert_eq!(doc.version, 2);
    drop(doc);

    // Close document
    state.close(&uri);
    assert!(!state.contains(&uri));
    assert!(state.is_empty());
}

/// Test position mapping with real text
#[test]
fn test_position_mapper_multiline() {
    let text = "First line.\nSecond line.\nThird line.";
    let mapper = PositionMapper::new(text);

    // First line
    let pos = mapper.byte_to_position(0).unwrap();
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 0);

    // "First" ends at byte 5
    let pos = mapper.byte_to_position(5).unwrap();
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 5);

    // Second line starts at byte 12
    let pos = mapper.byte_to_position(12).unwrap();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.character, 0);

    // Third line
    let pos = mapper.byte_to_position(25).unwrap();
    assert_eq!(pos.line, 2);
    assert_eq!(pos.character, 0);
}

/// Test sentence extraction for plain text
#[test]
fn test_sentence_extraction_plaintext() {
    let text = "Hello world. This is a test. Final sentence.";
    let sentences = extract_sentences(text, "plaintext");

    assert_eq!(sentences.len(), 3);
    assert_eq!(sentences[0].text, "Hello world.");
    assert_eq!(sentences[1].text, "This is a test.");
    assert_eq!(sentences[2].text, "Final sentence.");
}

/// Test sentence extraction for markdown
#[test]
fn test_sentence_extraction_markdown() {
    let text = "# Header\n\nThis is a paragraph. With two sentences.\n\n- List item.";
    let sentences = extract_sentences(text, "markdown");

    // Should extract sentences from paragraph and list item
    assert!(!sentences.is_empty());
    // Verify at least one sentence is extracted
    assert!(sentences.iter().any(|s| s.text.contains("paragraph")));
}

/// Test semantic token encoder
#[test]
fn test_semantic_token_encoder() {
    let mut encoder = SemanticTokenEncoder::new();

    // Add some tokens
    encoder.push(0, 0, 3, ThetaTokenType::Agent, &[]);
    encoder.push(
        0,
        4,
        4,
        ThetaTokenType::Predicate,
        &[TokenModifier::HighConfidence],
    );
    encoder.push(0, 9, 5, ThetaTokenType::Patient, &[]);

    assert!(!encoder.is_empty());

    let tokens = encoder.build();
    // Should have 3 tokens, each with 5 values
    assert_eq!(tokens.len(), 3);
}

/// Test semantic token legend
#[test]
fn test_semantic_token_legend_structure() {
    let legend = semantic_token_legend();

    // Should have all theta role types plus syntactic categories
    assert!(legend.token_types.len() >= 14); // At least the theta roles

    // Should have modifiers
    assert!(legend.token_modifiers.len() >= 3);
}

/// Test cached document staleness
#[test]
fn test_cached_document_staleness() {
    use std::time::{Duration, Instant};

    let mut doc = CachedDocument::new("Test content.".to_string(), 1, "plaintext".to_string());

    // Fresh document has no analysis
    assert!(doc.is_analysis_stale(1000));

    // Set analyzed_at to now
    doc.analyzed_at = Some(Instant::now());
    assert!(!doc.is_analysis_stale(1000));

    // Wait and check staleness (use very short time)
    std::thread::sleep(Duration::from_millis(10));
    assert!(doc.is_analysis_stale(5)); // 5ms threshold, slept 10ms
}

/// Test with real pipeline (requires data files)
#[test]
fn test_pipeline_analysis() {
    let pipeline = match CanopyPipeline::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping pipeline test: {e}");
            return;
        }
    };

    // Analyze a simple sentence
    let analysis = pipeline.analyze("The cat chased the mouse.").unwrap();

    // Should have tokens
    assert!(!analysis.syntax.tokens.is_empty());

    // Should identify some roles
    // (May or may not have role_bindings depending on verb coverage)
}

/// Test position mapper edge cases
#[test]
fn test_position_mapper_edge_cases() {
    // Empty text - byte 0 is still valid (start position)
    let mapper = PositionMapper::new("");
    // Empty text has one line starting at byte 0
    let pos = mapper.byte_to_position(0);
    assert!(pos.is_some());

    // Out of bounds for empty text
    assert!(mapper.byte_to_position(1).is_none());

    // Single character
    let mapper = PositionMapper::new("a");
    let pos = mapper.byte_to_position(0).unwrap();
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 0);

    // Position to byte
    let mapper = PositionMapper::new("Hello\nWorld");
    let byte = mapper.position_to_byte(Position::new(1, 0)).unwrap();
    assert_eq!(byte, 6); // After "Hello\n"
}

/// Test sentence span properties
#[test]
fn test_sentence_span_properties() {
    let span = SentenceSpan {
        text: "Test sentence.".to_string(),
        byte_start: 0,
        byte_end: 14,
        line_start: 0,
        line_end: 0,
    };

    assert_eq!(span.text.len(), 14);
    assert_eq!(span.byte_end - span.byte_start, 14);
}

/// Test document state with multiple documents
#[test]
fn test_document_state_multiple_docs() {
    let state = DocumentState::new();

    let uri1 = Url::parse("file:///doc1.txt").unwrap();
    let uri2 = Url::parse("file:///doc2.txt").unwrap();
    let uri3 = Url::parse("file:///doc3.txt").unwrap();

    state.open(
        uri1.clone(),
        "Doc 1".to_string(),
        1,
        "plaintext".to_string(),
    );
    state.open(uri2.clone(), "Doc 2".to_string(), 1, "markdown".to_string());
    state.open(uri3.clone(), "Doc 3".to_string(), 1, "rust".to_string());

    assert_eq!(state.len(), 3);
    assert!(state.contains(&uri1));
    assert!(state.contains(&uri2));
    assert!(state.contains(&uri3));

    // Close one
    state.close(&uri2);
    assert_eq!(state.len(), 2);
    assert!(!state.contains(&uri2));
}

/// Test encoder with modifiers
#[test]
fn test_encoder_with_multiple_modifiers() {
    let mut encoder = SemanticTokenEncoder::new();

    encoder.push(
        0,
        0,
        5,
        ThetaTokenType::Agent,
        &[TokenModifier::HighConfidence, TokenModifier::Ambiguous],
    );

    let tokens = encoder.build();
    assert_eq!(tokens.len(), 1);

    // Check modifier bits (HighConfidence = 1, Ambiguous = 4, combined = 5)
    assert_eq!(tokens[0].token_modifiers_bitset, 5);
}

/// Test encoder on different lines
#[test]
fn test_encoder_multiline() {
    let mut encoder = SemanticTokenEncoder::new();

    encoder.push(0, 0, 3, ThetaTokenType::Agent, &[]);
    encoder.push(1, 5, 4, ThetaTokenType::Patient, &[]);
    encoder.push(3, 0, 6, ThetaTokenType::Theme, &[]);

    let tokens = encoder.build();
    assert_eq!(tokens.len(), 3);

    // First token: delta_line=0, delta_start=0
    assert_eq!(tokens[0].delta_line, 0);
    assert_eq!(tokens[0].delta_start, 0);

    // Second token: delta_line=1, delta_start=5
    assert_eq!(tokens[1].delta_line, 1);
    assert_eq!(tokens[1].delta_start, 5);

    // Third token: delta_line=2, delta_start=0
    assert_eq!(tokens[2].delta_line, 2);
    assert_eq!(tokens[2].delta_start, 0);
}

/// Test analysis cache with real pipeline
#[test]
fn test_analysis_cache_with_pipeline() {
    let pipeline = match CanopyPipeline::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping cache test: {e}");
            return;
        }
    };

    let cache = AnalysisCache::new(10);

    // First call should be a miss
    let result1 = cache.get_or_analyze("The dog runs.", &pipeline);
    assert!(result1.is_ok());

    let stats1 = cache.stats();
    assert_eq!(stats1.misses, 1);
    assert_eq!(stats1.hits, 0);
    assert_eq!(stats1.size, 1);

    // Second call with same sentence should be a hit
    let result2 = cache.get_or_analyze("The dog runs.", &pipeline);
    assert!(result2.is_ok());

    let stats2 = cache.stats();
    assert_eq!(stats2.misses, 1);
    assert_eq!(stats2.hits, 1);
    assert_eq!(stats2.size, 1);

    // Different sentence should be a miss
    let result3 = cache.get_or_analyze("The cat sleeps.", &pipeline);
    assert!(result3.is_ok());

    let stats3 = cache.stats();
    assert_eq!(stats3.misses, 2);
    assert_eq!(stats3.hits, 1);
    assert_eq!(stats3.size, 2);

    // Verify hit rate
    assert!((stats3.hit_rate() - 33.33).abs() < 1.0);
}

/// Test cache LRU eviction
#[test]
fn test_cache_lru_eviction() {
    let pipeline = match CanopyPipeline::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping LRU test: {e}");
            return;
        }
    };

    // Small cache to trigger eviction
    let cache = AnalysisCache::new(2);

    // Add 3 sentences to a cache of size 2
    let _ = cache.get_or_analyze("Sentence one.", &pipeline);
    let _ = cache.get_or_analyze("Sentence two.", &pipeline);
    let _ = cache.get_or_analyze("Sentence three.", &pipeline);

    let stats = cache.stats();
    assert_eq!(stats.misses, 3);
    assert_eq!(stats.size, 2); // LRU should have evicted one
}

/// Test cache clear after use
#[test]
fn test_cache_clear_after_use() {
    let pipeline = match CanopyPipeline::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping clear test: {e}");
            return;
        }
    };

    let cache = AnalysisCache::new(10);

    // Add some entries
    let _ = cache.get_or_analyze("First sentence.", &pipeline);
    let _ = cache.get_or_analyze("Second sentence.", &pipeline);

    let stats_before = cache.stats();
    assert_eq!(stats_before.size, 2);

    // Clear and verify
    cache.clear();

    let stats_after = cache.stats();
    assert_eq!(stats_after.size, 0);
    assert_eq!(stats_after.hits, 0);
    assert_eq!(stats_after.misses, 0);

    // Cache miss after clear
    let _ = cache.get_or_analyze("First sentence.", &pipeline);
    let stats_final = cache.stats();
    assert_eq!(stats_final.misses, 1);
}
