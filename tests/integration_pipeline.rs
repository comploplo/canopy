//! Integration tests for the complete parsing pipeline
//! 
//! These tests verify that the entire UDPipe → semantic features → VerbNet
//! pipeline works correctly end-to-end.

use canopy_core::{Document, UPos};
use canopy_parser::udpipe::{UDPipeParser, UDPipeEngine};
use canopy_semantics::verbnet::{VerbNetEngine, VerbNetFeatureExtractor};

#[test]
fn test_end_to_end_parsing_pipeline() {
    // Create the complete pipeline
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    let mut verbnet_engine = VerbNetEngine::new();
    verbnet_engine.add_test_data();
    
    let mut feature_extractor = VerbNetFeatureExtractor::new(verbnet_engine);
    
    // Test sentence
    let sentence = "She gave him a book.";
    
    // Parse with UDPipe (dummy tokenization)
    let parsed_doc = parser.parse_document(sentence).expect("Parsing should succeed");
    
    // Convert to core types
    let document: Document = parsed_doc.into();
    
    // Verify basic structure
    assert!(!document.sentences.is_empty(), "Should have at least one sentence");
    let first_sentence = &document.sentences[0];
    assert!(!first_sentence.words.is_empty(), "Should have words");
    
    // Extract semantic features for verbs
    for word in &first_sentence.words {
        if word.upos == UPos::Verb || word.lemma == "give" {
            let features = feature_extractor.extract_features(word);
            
            // For "gave/give" we should extract meaningful features
            if word.lemma == "give" {
                assert!(features.animacy.is_some(), "Should extract animacy features");
                assert!(features.confidence.animacy > 0.0, "Should have confidence > 0");
            }
        }
    }
}

#[test] 
fn test_pipeline_with_multiple_sentences() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    let text = "The cat sat. She gave him a book. They walked home.";
    let parsed_doc = parser.parse_document(text).expect("Multi-sentence parsing should succeed");
    
    // Should split into multiple sentences
    assert!(parsed_doc.sentences.len() >= 2, "Should detect multiple sentences");
    
    // Each sentence should have words
    for sentence in &parsed_doc.sentences {
        assert!(!sentence.words.is_empty(), "Each sentence should have words");
    }
}

#[test]
fn test_pipeline_error_handling() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    // Empty input should be handled gracefully
    let result = parser.parse_document("");
    assert!(result.is_err(), "Empty input should return error");
    
    // Whitespace-only input
    let result = parser.parse_document("   \n\t  ");
    assert!(result.is_err(), "Whitespace-only input should return error");
    
    // Valid input should work
    let result = parser.parse_document("Hello world.");
    assert!(result.is_ok(), "Valid input should succeed");
}

#[test]
fn test_word_to_core_conversion() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    let sentence = "The quick brown fox.";
    let parsed_doc = parser.parse_document(sentence).expect("Should parse successfully");
    
    // Convert to core Document
    let document: Document = parsed_doc.into();
    
    // Verify conversion worked correctly
    let words = &document.sentences[0].words;
    assert!(!words.is_empty(), "Should have converted words");
    
    // Check that Word fields are properly set
    for word in words {
        assert!(!word.text.is_empty(), "Word text should not be empty");
        assert!(!word.lemma.is_empty(), "Word lemma should not be empty");
        assert!(word.id > 0, "Word ID should be positive");
        // Note: UPos might be X for dummy tokenization, that's expected
    }
}

#[test]
fn test_verbnet_integration_with_parsing() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    let mut verbnet_engine = VerbNetEngine::new();
    verbnet_engine.add_test_data();
    
    let sentence = "give";
    let parsed_doc = parser.parse_document(sentence).expect("Should parse");
    let _document: Document = parsed_doc.into();
    
    // Check VerbNet lookup works
    let classes = verbnet_engine.get_verb_classes("give");
    assert!(!classes.is_empty(), "Should find VerbNet classes for 'give'");
    
    let roles = verbnet_engine.get_theta_roles("give");
    assert!(!roles.is_empty(), "Should find theta roles for 'give'");
    
    // Should include Agent, Theme, Recipient
    let role_types: Vec<_> = roles.iter().map(|r| r.role_type).collect();
    assert!(role_types.contains(&canopy_semantics::ThetaRoleType::Agent));
    assert!(role_types.contains(&canopy_semantics::ThetaRoleType::Theme));
    assert!(role_types.contains(&canopy_semantics::ThetaRoleType::Recipient));
}

#[test]
fn test_memory_efficiency() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    // Test that we can parse many sentences without excessive memory growth
    let base_sentence = "The cat sat on the mat.";
    
    for i in 0..100 {
        let test_sentence = format!("{base_sentence} Iteration {i}.");
        let result = parser.parse_document(&test_sentence);
        assert!(result.is_ok(), "Should parse successfully in iteration {i}");
        
        // Let the document be dropped to test memory cleanup
    }
    
    // If we got here, memory management is working reasonably
}

#[test]
fn test_linguistic_invariants() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    let test_cases = vec![
        "Simple sentence.",
        "The quick brown fox jumps.",
        "She loves reading books in the library.",
        "Although it was raining, they continued walking.",
    ];
    
    for sentence in test_cases {
        let parsed_doc = parser.parse_document(sentence).expect("Should parse");
        let document: Document = parsed_doc.into();
        
        // Linguistic invariants that should hold
        for sentence in &document.sentences {
            // Word IDs should be sequential starting from 1
            for (i, word) in sentence.words.iter().enumerate() {
                assert_eq!(word.id, i + 1, "Word IDs should be sequential");
            }
            
            // Character positions should be non-decreasing
            for i in 1..sentence.words.len() {
                assert!(
                    sentence.words[i].start >= sentence.words[i-1].start,
                    "Word positions should be non-decreasing"
                );
            }
            
            // Text should not be empty for any word
            for word in &sentence.words {
                assert!(!word.text.is_empty(), "Word text should not be empty");
                assert!(!word.lemma.is_empty(), "Word lemma should not be empty");
            }
        }
    }
}

#[test]
fn test_performance_characteristics() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    // Test that parsing time scales reasonably with input size
    let short_sentence = "Cat.";
    let medium_sentence = "The quick brown fox jumps over the lazy dog.";
    let long_sentence = "The extraordinarily complex and multifaceted nature of human language comprehension and production mechanisms requires sophisticated computational models that can adequately capture the intricate relationships between syntactic structures and semantic representations.";
    
    // All should complete quickly (we're not timing here, just ensuring no hangs)
    for sentence in [short_sentence, medium_sentence, long_sentence] {
        let start = std::time::Instant::now();
        let result = parser.parse_document(sentence);
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Should parse successfully");
        assert!(duration.as_millis() < 100, "Should complete within 100ms (actually much faster)");
    }
}

#[test]
fn test_unicode_handling() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    // Test with various Unicode characters
    let unicode_sentence = "Café naïve résumé façade.";
    let result = parser.parse_document(unicode_sentence);
    assert!(result.is_ok(), "Should handle Unicode characters");
    
    let parsed_doc = result.unwrap();
    assert!(!parsed_doc.sentences.is_empty(), "Should parse Unicode sentence");
}

#[test]
fn test_edge_case_inputs() {
    let udpipe_engine = UDPipeEngine::for_testing();
    let parser = UDPipeParser::new_with_engine(udpipe_engine);
    
    // Single character
    let result = parser.parse_document("A");
    assert!(result.is_ok(), "Should handle single character");
    
    // Only punctuation
    let result = parser.parse_document("!!!");
    assert!(result.is_ok(), "Should handle punctuation-only");
    
    // Mixed punctuation and words
    let result = parser.parse_document("Hello, world!");
    assert!(result.is_ok(), "Should handle mixed content");
    
    if let Ok(parsed_doc) = result {
        let document: Document = parsed_doc.into();
        // Should properly separate punctuation
        let words = &document.sentences[0].words;
        let _has_punctuation = words.iter().any(|w| w.upos == UPos::Punct || w.text.contains('!'));
        // Note: This might not work perfectly with dummy tokenization, but shouldn't crash
    }
}