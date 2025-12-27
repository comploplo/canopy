//! Tests for LexiconDatabase methods to achieve coverage targets

use canopy_lexicon::types::{
    ClassificationType, LexiconDatabase, LexiconPattern, LexiconWord, PatternType, PropertyValue,
    WordClass, WordClassType,
};
use regex::Regex;
use std::collections::HashMap;

fn create_test_database() -> LexiconDatabase {
    let mut database = LexiconDatabase::new();

    // Create test word class with patterns
    let mut negation_class = WordClass::new(
        "negation".to_string(),
        "Negation Words".to_string(),
        WordClassType::Negation,
        "Words indicating negation".to_string(),
    );

    negation_class.words.push(LexiconWord {
        word: "not".to_string(),
        variants: vec!["n't".to_string()],
        pos: Some("ADV".to_string()),
        confidence: 0.95,
        frequency: Some(1000),
        context: Some("negation".to_string()),
    });

    // Add pattern for negation prefixes
    let pattern = LexiconPattern {
        id: "neg-prefix".to_string(),
        pattern_type: PatternType::Prefix,
        regex: Regex::new(r"^(un|dis|in|non)").unwrap(),
        regex_str: "^(un|dis|in|non)".to_string(),
        description: "Negative prefix pattern".to_string(),
        confidence: 0.8,
        examples: vec!["unhappy".to_string(), "disagree".to_string()],
    };
    negation_class.patterns.push(pattern);

    // Add another word class for stop words
    let mut stop_class = WordClass::new(
        "stopwords".to_string(),
        "Stop Words".to_string(),
        WordClassType::StopWords,
        "Common function words".to_string(),
    );

    stop_class.words.push(LexiconWord {
        word: "the".to_string(),
        variants: vec![],
        pos: Some("DET".to_string()),
        confidence: 1.0,
        frequency: Some(5000),
        context: None,
    });

    database.word_classes.push(negation_class);
    database.word_classes.push(stop_class);
    database.build_indices();

    database
}

#[test]
fn test_analyze_patterns() {
    let database = create_test_database();

    // Test pattern matching for negative prefixes
    let matches = database.analyze_patterns("unhappy");
    assert!(!matches.is_empty());

    let pattern_match = &matches[0];
    assert_eq!(pattern_match.pattern_type, PatternType::Prefix);
    assert_eq!(pattern_match.matched_text, "un");
    assert_eq!(pattern_match.confidence, 0.8);

    // Test pattern matching for words that don't match
    let matches = database.analyze_patterns("happy");
    assert!(matches.is_empty());

    // Test multiple pattern matching
    let matches = database.analyze_patterns("disagree");
    assert!(!matches.is_empty());
    assert_eq!(matches[0].matched_text, "dis");
}

#[test]
fn test_get_classes_by_type() {
    let database = create_test_database();

    // Test getting negation word classes
    let negation_classes = database.get_classes_by_type(&WordClassType::Negation);
    assert_eq!(negation_classes.len(), 1);
    assert_eq!(negation_classes[0].id, "negation");

    // Test getting stop word classes
    let stop_classes = database.get_classes_by_type(&WordClassType::StopWords);
    assert_eq!(stop_classes.len(), 1);
    assert_eq!(stop_classes[0].id, "stopwords");

    // Test getting non-existent word class type
    let discourse_classes = database.get_classes_by_type(&WordClassType::DiscourseMarkers);
    assert!(discourse_classes.is_empty());
}

#[test]
fn test_classify_word_with_priorities() {
    let mut database = create_test_database();

    // Set different priorities for word classes
    database.word_classes[0].priority = 3; // negation
    database.word_classes[1].priority = 1; // stop words
    database.build_indices();

    // Test word classification
    let classifications = database.classify_word("not");
    assert!(!classifications.is_empty());

    let classification = &classifications[0];
    assert_eq!(classification.word_class_type, WordClassType::Negation);
    assert_eq!(classification.matched_word, "not");
    assert_eq!(classification.input_word, "not");
    assert_eq!(classification.confidence, 0.95);
    assert_eq!(
        classification.classification_type,
        ClassificationType::ExactMatch
    );
}

#[test]
fn test_database_stats() {
    let database = create_test_database();

    let stats = database.stats();
    assert_eq!(stats.total_word_classes, 2);
    assert_eq!(stats.total_words, 2); // "not" and "the"
    assert_eq!(stats.total_patterns, 1); // one negation pattern

    // Check words by type
    assert_eq!(
        *stats
            .words_by_type
            .get(&WordClassType::Negation)
            .unwrap_or(&0),
        1
    );
    assert_eq!(
        *stats
            .words_by_type
            .get(&WordClassType::StopWords)
            .unwrap_or(&0),
        1
    );
}

#[test]
fn test_word_class_utility_methods() {
    let database = create_test_database();
    let negation_class = &database.word_classes[0];

    // Test utility methods
    assert!(negation_class.modifies_polarity());
    assert!(!negation_class.is_stop_words());
    assert!(!negation_class.provides_discourse_structure());

    let stop_class = &database.word_classes[1];
    assert!(stop_class.is_stop_words());
    assert!(!stop_class.modifies_polarity());
    assert!(!stop_class.provides_discourse_structure());
}

#[test]
fn test_word_class_contains_word() {
    let database = create_test_database();
    let negation_class = &database.word_classes[0];

    // Test exact word match
    let result = negation_class.contains_word("not");
    assert!(result.is_some());
    assert_eq!(result.unwrap().word, "not");

    // Test variant match
    let result = negation_class.contains_word("n't");
    assert!(result.is_some());
    assert_eq!(result.unwrap().word, "not");

    // Test non-matching word
    let result = negation_class.contains_word("happy");
    assert!(result.is_none());
}

#[test]
fn test_word_class_pattern_matching() {
    let database = create_test_database();
    let negation_class = &database.word_classes[0];

    // Test pattern matching
    let patterns = negation_class.matches_pattern("unhappy");
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].id, "neg-prefix");

    // Test no pattern match
    let patterns = negation_class.matches_pattern("happy");
    assert!(patterns.is_empty());
}

#[test]
fn test_property_value_getters() {
    let mut properties = HashMap::new();
    properties.insert(
        "string_prop".to_string(),
        PropertyValue::String("test".to_string()),
    );
    properties.insert("bool_prop".to_string(), PropertyValue::Boolean(true));
    properties.insert("int_prop".to_string(), PropertyValue::Integer(42));
    properties.insert("float_prop".to_string(), PropertyValue::Float(2.72));

    // Test PropertyValue getters
    if let Some(PropertyValue::String(s)) = properties.get("string_prop") {
        assert_eq!(s, "test");
    }

    let string_val = properties.get("string_prop").unwrap();
    assert_eq!(string_val.as_string(), Some("test"));
    assert_eq!(string_val.as_bool(), None);

    let bool_val = properties.get("bool_prop").unwrap();
    assert_eq!(bool_val.as_bool(), Some(true));
    assert_eq!(bool_val.as_string(), None);

    let int_val = properties.get("int_prop").unwrap();
    assert_eq!(int_val.as_int(), Some(42));
    assert_eq!(int_val.as_float(), None);

    let float_val = properties.get("float_prop").unwrap();
    assert_eq!(float_val.as_float(), Some(2.72));
    assert_eq!(float_val.as_int(), None);
}

#[test]
fn test_pattern_type_string_conversion() {
    assert_eq!(PatternType::Prefix.as_str(), "prefix");
    assert_eq!(PatternType::Suffix.as_str(), "suffix");
    assert_eq!(PatternType::Infix.as_str(), "infix");
    assert_eq!(PatternType::WholeWord.as_str(), "whole-word");
    assert_eq!(PatternType::Phrase.as_str(), "phrase");

    assert_eq!(PatternType::parse_str("prefix"), Some(PatternType::Prefix));
    assert_eq!(PatternType::parse_str("suffix"), Some(PatternType::Suffix));
    assert_eq!(PatternType::parse_str("invalid"), None);
}

#[test]
fn test_word_class_type_string_conversion() {
    assert_eq!(WordClassType::Negation.as_str(), "negation");
    assert_eq!(WordClassType::StopWords.as_str(), "stop-words");
    assert_eq!(
        WordClassType::DiscourseMarkers.as_str(),
        "discourse-markers"
    );

    assert_eq!(
        WordClassType::parse_str("negation"),
        Some(WordClassType::Negation)
    );
    assert_eq!(
        WordClassType::parse_str("stop-words"),
        Some(WordClassType::StopWords)
    );
    assert_eq!(WordClassType::parse_str("invalid"), None);
}
