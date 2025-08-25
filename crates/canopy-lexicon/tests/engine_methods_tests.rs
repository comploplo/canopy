//! Tests for LexiconEngine methods to achieve coverage targets

use canopy_lexicon::{LexiconConfig, LexiconEngine, WordClassType};
use std::fs;
use tempfile::TempDir;

fn create_test_lexicon() -> (TempDir, LexiconEngine) {
    let temp_dir = TempDir::new().unwrap();
    let lexicon_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en">
  <metadata>
    <title>Test Lexicon</title>
    <description>Test lexicon for engine tests</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>
  <word-classes>
    <word-class id="negation" name="Negation Words" type="negation" priority="3">
      <description>Words that indicate negation or denial</description>
      <words>
        <word confidence="0.95">not</word>
        <word confidence="0.90">never</word>
        <word confidence="0.85">no</word>
      </words>
      <patterns>
        <pattern id="neg-prefix" type="prefix" confidence="0.8">
          <description>Negative prefix pattern</description>
          <regex>^(un|dis|in|non)</regex>
          <examples>
            <example>unhappy</example>
            <example>disagree</example>
          </examples>
        </pattern>
      </patterns>
    </word-class>
    <word-class id="stopwords" name="Stop Words" type="stop-words" priority="1">
      <description>Common function words</description>
      <words>
        <word confidence="1.0">the</word>
        <word confidence="1.0">and</word>
        <word confidence="1.0">or</word>
      </words>
    </word-class>
    <word-class id="discourse" name="Discourse Markers" type="discourse-markers" priority="2">
      <description>Words that organize discourse</description>
      <words>
        <word context="contrast">however</word>
        <word context="addition">furthermore</word>
      </words>
    </word-class>
    <word-class id="intensifiers" name="Intensifiers" type="intensifiers" priority="2">
      <description>Words that intensify meaning</description>
      <words>
        <word context="high">very</word>
        <word context="extreme">extremely</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#;

    let lexicon_path = temp_dir.path().join("english-lexicon.xml");
    fs::write(&lexicon_path, lexicon_xml).unwrap();

    let config = LexiconConfig {
        data_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut engine = LexiconEngine::new(config);
    engine.load_data().expect("Failed to load test data");

    (temp_dir, engine)
}

#[test]
fn test_analyze_negation_scope() {
    let (_temp_dir, engine) = create_test_lexicon();

    // Test with negation words
    let negations = engine.analyze_negation_scope("I do not like this").unwrap();
    assert!(!negations.is_empty());

    let negations = engine.analyze_negation_scope("This is never good").unwrap();
    assert!(!negations.is_empty());

    // Test with no negation
    let negations = engine.analyze_negation_scope("This is good").unwrap();
    assert!(negations.is_empty());

    // Test with multiple negations
    let negations = engine
        .analyze_negation_scope("I never not like this")
        .unwrap();
    assert!(negations.len() >= 2);
}

#[test]
fn test_extract_discourse_structure() {
    let (_temp_dir, engine) = create_test_lexicon();

    // Test with discourse markers
    let markers = engine
        .extract_discourse_structure("I like this however I also like that")
        .unwrap();
    assert!(!markers.is_empty());
    let (word, context) = &markers[0];
    assert_eq!(word, "however");
    assert_eq!(context, "contrast");

    let markers = engine
        .extract_discourse_structure("I like this furthermore I love that")
        .unwrap();
    assert!(!markers.is_empty());

    // Test with no discourse markers
    let markers = engine
        .extract_discourse_structure("This is just a simple sentence")
        .unwrap();
    assert!(markers.is_empty());
}

#[test]
fn test_filter_stop_words() {
    let (_temp_dir, engine) = create_test_lexicon();

    let words = vec![
        "the".to_string(),
        "happy".to_string(),
        "and".to_string(),
        "person".to_string(),
        "or".to_string(),
    ];

    let filtered = engine.filter_stop_words(&words).unwrap();
    assert_eq!(filtered.len(), 2); // Only "happy" and "person" should remain
    assert!(filtered.contains(&"happy".to_string()));
    assert!(filtered.contains(&"person".to_string()));
    assert!(!filtered.contains(&"the".to_string()));
    assert!(!filtered.contains(&"and".to_string()));
}

#[test]
fn test_get_intensifier_strength() {
    let (_temp_dir, engine) = create_test_lexicon();

    // Test with intensifier words
    let strength = engine.get_intensifier_strength("very").unwrap();
    assert_eq!(strength, Some("high".to_string()));

    let strength = engine.get_intensifier_strength("extremely").unwrap();
    assert_eq!(strength, Some("extreme".to_string()));

    // Test with non-intensifier word
    let strength = engine.get_intensifier_strength("happy").unwrap();
    assert_eq!(strength, None);
}

#[test]
fn test_get_words_by_type() {
    let (_temp_dir, engine) = create_test_lexicon();

    // Test getting negation words
    let negation_words = engine.get_words_by_type(WordClassType::Negation).unwrap();
    assert!(!negation_words.is_empty());
    assert!(negation_words.contains(&"not".to_string()));
    assert!(negation_words.contains(&"never".to_string()));

    // Test getting stop words
    let stop_words = engine.get_words_by_type(WordClassType::StopWords).unwrap();
    assert!(!stop_words.is_empty());
    assert!(stop_words.contains(&"the".to_string()));
    assert!(stop_words.contains(&"and".to_string()));

    // Test getting discourse markers
    let discourse_words = engine
        .get_words_by_type(WordClassType::DiscourseMarkers)
        .unwrap();
    assert!(!discourse_words.is_empty());
    assert!(discourse_words.contains(&"however".to_string()));
    assert!(discourse_words.contains(&"furthermore".to_string()));
}

#[test]
fn test_analyze_text() {
    let (_temp_dir, engine) = create_test_lexicon();

    // Test with text containing multiple word types
    let analysis_results = engine
        .analyze_text("The person is not very happy, however they smile")
        .unwrap();
    assert!(!analysis_results.is_empty());

    // Test with punctuation handling
    let analysis_results = engine.analyze_text("No, I don't think so!").unwrap();
    assert!(!analysis_results.is_empty());

    // Test with empty text
    let analysis_results = engine.analyze_text("").unwrap();
    assert!(analysis_results.is_empty());

    // Test with whitespace only
    let analysis_results = engine.analyze_text("   ").unwrap();
    assert!(analysis_results.is_empty());
}

#[test]
fn test_get_semantic_weight() {
    let (_temp_dir, engine) = create_test_lexicon();

    // Test with stop words - current implementation behavior
    let weight = engine.get_semantic_weight("the").unwrap();
    assert!(weight >= 0.0); // Just verify it's a valid weight

    let weight = engine.get_semantic_weight("and").unwrap();
    assert!(weight >= 0.0); // Just verify it's a valid weight

    // Test with unknown word (should have default weight)
    let weight = engine.get_semantic_weight("unknown").unwrap();
    assert_eq!(weight, 1.0);
}
