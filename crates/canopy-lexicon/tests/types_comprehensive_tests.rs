use canopy_lexicon::types::*;
use std::collections::HashMap;

// Comprehensive tests for lexicon types to improve coverage from 146/235 to 95%+

#[test]
fn test_word_class_type_string_conversions() {
    let types_and_strings = vec![
        (WordClassType::StopWords, "stop-words"),
        (WordClassType::Negation, "negation"),
        (WordClassType::DiscourseMarkers, "discourse-markers"),
        (WordClassType::Quantifiers, "quantifiers"),
        (WordClassType::Temporal, "temporal"),
        (WordClassType::Modal, "modal"),
        (WordClassType::Pronouns, "pronouns"),
        (WordClassType::Prepositions, "prepositions"),
        (WordClassType::Conjunctions, "conjunctions"),
        (WordClassType::Intensifiers, "intensifiers"),
        (WordClassType::HedgeWords, "hedge-words"),
        (WordClassType::Sentiment, "sentiment"),
        (WordClassType::Functional, "functional"),
    ];

    for (word_type, string_repr) in types_and_strings {
        assert_eq!(word_type.as_str(), string_repr);
        assert_eq!(
            WordClassType::parse_str(string_repr),
            Some(word_type.clone())
        );
    }

    // Test invalid strings
    assert_eq!(WordClassType::parse_str("invalid-type"), None);
    assert_eq!(WordClassType::parse_str(""), None);
}

#[test]
fn test_pattern_type_string_conversions() {
    let types_and_strings = vec![
        (PatternType::Prefix, "prefix"),
        (PatternType::Suffix, "suffix"),
        (PatternType::Infix, "infix"),
        (PatternType::WholeWord, "whole-word"),
        (PatternType::Phrase, "phrase"),
    ];

    for (pattern_type, string_repr) in types_and_strings {
        assert_eq!(pattern_type.as_str(), string_repr);
        assert_eq!(
            PatternType::parse_str(string_repr),
            Some(pattern_type.clone())
        );
    }

    assert_eq!(PatternType::parse_str("invalid-pattern"), None);
}

#[test]
fn test_property_value_conversions() {
    let string_prop = PropertyValue::String("test".to_string());
    let bool_prop = PropertyValue::Boolean(true);
    let int_prop = PropertyValue::Integer(42);
    let float_prop = PropertyValue::Float(3.14);

    // Test as_string()
    assert_eq!(string_prop.as_string(), Some("test"));
    assert_eq!(bool_prop.as_string(), None);
    assert_eq!(int_prop.as_string(), None);
    assert_eq!(float_prop.as_string(), None);

    // Test as_bool()
    assert_eq!(string_prop.as_bool(), None);
    assert_eq!(bool_prop.as_bool(), Some(true));
    assert_eq!(int_prop.as_bool(), None);
    assert_eq!(float_prop.as_bool(), None);

    // Test as_int()
    assert_eq!(string_prop.as_int(), None);
    assert_eq!(bool_prop.as_int(), None);
    assert_eq!(int_prop.as_int(), Some(42));
    assert_eq!(float_prop.as_int(), None);

    // Test as_float()
    assert_eq!(string_prop.as_float(), None);
    assert_eq!(bool_prop.as_float(), None);
    assert_eq!(int_prop.as_float(), None);
    assert_eq!(float_prop.as_float(), Some(3.14));
}

#[test]
fn test_lexicon_word_creation_and_matching() {
    let mut word = LexiconWord::new("test".to_string());
    assert_eq!(word.word, "test");
    assert!(word.variants.is_empty());
    assert_eq!(word.pos, None);
    assert_eq!(word.confidence, 1.0);
    assert_eq!(word.frequency, None);
    assert_eq!(word.context, None);

    // Test matching
    assert!(word.matches("test"));
    assert!(word.matches("TEST"));
    assert!(word.matches("Test"));
    assert!(!word.matches("testing"));

    // Add variants
    word.variants.push("variant1".to_string());
    word.variants.push("variant2".to_string());

    assert!(word.matches("variant1"));
    assert!(word.matches("VARIANT1"));
    assert!(word.matches("variant2"));
    assert!(!word.matches("variant3"));

    // Test with additional properties
    word.pos = Some("NOUN".to_string());
    word.confidence = 0.9;
    word.frequency = Some(100);
    word.context = Some("formal".to_string());

    assert_eq!(word.pos.as_ref().unwrap(), "NOUN");
    assert_eq!(word.confidence, 0.9);
    assert_eq!(word.frequency.unwrap(), 100);
    assert_eq!(word.context.as_ref().unwrap(), "formal");
}

#[test]
fn test_lexicon_pattern_creation_and_matching() {
    let pattern = LexiconPattern::new(
        "test-pattern".to_string(),
        PatternType::Suffix,
        r"ing$".to_string(),
        "Words ending with 'ing'".to_string(),
    )
    .unwrap();

    assert_eq!(pattern.id, "test-pattern");
    assert_eq!(pattern.pattern_type, PatternType::Suffix);
    assert_eq!(pattern.regex_str, r"ing$");
    assert_eq!(pattern.description, "Words ending with 'ing'");
    assert_eq!(pattern.confidence, 0.8);
    assert!(pattern.examples.is_empty());

    // Test pattern matching
    assert!(pattern.matches("running"));
    assert!(pattern.matches("walking"));
    assert!(!pattern.matches("run"));
    assert!(!pattern.matches("singer"));

    // Test extract_match
    assert_eq!(pattern.extract_match("running"), Some("ing".to_string()));
    assert_eq!(pattern.extract_match("walking"), Some("ing".to_string()));
    assert_eq!(pattern.extract_match("run"), None);
}

#[test]
fn test_lexicon_pattern_invalid_regex() {
    let result = LexiconPattern::new(
        "invalid-pattern".to_string(),
        PatternType::Prefix,
        r"[invalid".to_string(), // Invalid regex
        "Invalid pattern".to_string(),
    );

    assert!(result.is_err());
}

#[test]
fn test_word_class_creation_and_functionality() {
    let mut word_class = WordClass::new(
        "negation-words".to_string(),
        "Negation Words".to_string(),
        WordClassType::Negation,
        "Words that negate meaning".to_string(),
    );

    assert_eq!(word_class.id, "negation-words");
    assert_eq!(word_class.name, "Negation Words");
    assert_eq!(word_class.word_class_type, WordClassType::Negation);
    assert_eq!(word_class.description, "Words that negate meaning");
    assert_eq!(word_class.priority, 1);
    assert!(word_class.properties.is_empty());
    assert!(word_class.words.is_empty());
    assert!(word_class.patterns.is_empty());

    // Test class type checks
    assert!(!word_class.is_stop_words());
    assert!(word_class.modifies_polarity());
    assert!(!word_class.provides_discourse_structure());

    // Add words
    word_class.words.push(LexiconWord::new("not".to_string()));
    word_class.words.push(LexiconWord::new("never".to_string()));

    // Test contains_word
    assert!(word_class.contains_word("not").is_some());
    assert!(word_class.contains_word("Never").is_some()); // Case insensitive
    assert!(word_class.contains_word("always").is_none());

    // Add properties
    word_class
        .properties
        .insert("polarity-strength".to_string(), PropertyValue::Float(0.9));

    assert!(word_class.get_property("polarity-strength").is_some());
    assert!(word_class.get_property("nonexistent").is_none());

    // Add patterns
    let pattern = LexiconPattern::new(
        "un-prefix".to_string(),
        PatternType::Prefix,
        r"^un\w+".to_string(),
        "Un- prefix pattern".to_string(),
    )
    .unwrap();
    word_class.patterns.push(pattern);

    let matches = word_class.matches_pattern("unhappy");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].id, "un-prefix");

    let no_matches = word_class.matches_pattern("happy");
    assert!(no_matches.is_empty());
}

#[test]
fn test_lexicon_database_comprehensive() {
    let mut db = LexiconDatabase::new();

    // Test default values
    assert!(db.title.is_empty());
    assert_eq!(db.version, "1.0");
    assert_eq!(db.language, "en");

    let default_db = LexiconDatabase::default();
    assert_eq!(default_db.version, "1.0");

    // Create comprehensive test data
    let mut negation_class = WordClass::new(
        "negation".to_string(),
        "Negation".to_string(),
        WordClassType::Negation,
        "Negation words".to_string(),
    );
    negation_class.priority = 3;

    let mut not_word = LexiconWord::new("not".to_string());
    not_word.confidence = 0.95;
    not_word.context = Some("formal".to_string());
    negation_class.words.push(not_word);

    let mut never_word = LexiconWord::new("never".to_string());
    never_word.variants.push("nevr".to_string());
    negation_class.words.push(never_word);

    negation_class.properties.insert(
        "polarity".to_string(),
        PropertyValue::String("negative".to_string()),
    );

    let pattern = LexiconPattern::new(
        "un-prefix".to_string(),
        PatternType::Prefix,
        r"^un\w+".to_string(),
        "Un- prefix".to_string(),
    )
    .unwrap();
    negation_class.patterns.push(pattern);

    db.word_classes.push(negation_class);
    db.build_indices();

    // Test classification
    let classifications = db.classify_word("not");
    assert_eq!(classifications.len(), 1);

    let classification = &classifications[0];
    assert_eq!(classification.word_class_type, WordClassType::Negation);
    assert_eq!(classification.confidence, 0.95);
    assert_eq!(
        classification.classification_type,
        ClassificationType::ExactMatch
    );

    // Test pattern analysis
    let matches = db.analyze_patterns("unhappy");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_id, "un-prefix");

    // Test statistics
    let stats = db.stats();
    assert_eq!(stats.total_word_classes, 1);
    assert_eq!(stats.total_words, 2);
    assert_eq!(stats.total_patterns, 1);
}

#[test]
fn test_word_classification_helper_methods() {
    let mut properties = HashMap::new();
    properties.insert("semantic-weight".to_string(), PropertyValue::Float(0.3));

    let classification = WordClassification {
        word_class_type: WordClassType::Negation,
        word_class_id: "neg1".to_string(),
        word_class_name: "Negation".to_string(),
        matched_word: "not".to_string(),
        input_word: "not".to_string(),
        confidence: 0.9,
        classification_type: ClassificationType::ExactMatch,
        context: None,
        properties,
    };

    // Test type checks
    assert!(classification.is_negation());
    assert!(!classification.is_stop_word());
    assert!(!classification.is_discourse_marker());
    assert!(!classification.is_quantifier());
    assert_eq!(classification.semantic_weight(), 0.3);

    // Test with default weight
    let stop_classification = WordClassification {
        word_class_type: WordClassType::StopWords,
        word_class_id: "stop1".to_string(),
        word_class_name: "Stop Words".to_string(),
        matched_word: "the".to_string(),
        input_word: "the".to_string(),
        confidence: 0.8,
        classification_type: ClassificationType::ExactMatch,
        context: None,
        properties: HashMap::new(),
    };

    assert!(stop_classification.is_stop_word());
    assert_eq!(stop_classification.semantic_weight(), 1.0); // Default
}

#[test]
fn test_lexicon_analysis_comprehensive() {
    let mut analysis = LexiconAnalysis::new("test input".to_string());

    assert_eq!(analysis.input, "test input");
    assert!(!analysis.has_results());
    assert_eq!(analysis.confidence, 0.0);

    // Add various classifications
    let negation_classification = WordClassification {
        word_class_type: WordClassType::Negation,
        word_class_id: "neg1".to_string(),
        word_class_name: "Negation".to_string(),
        matched_word: "not".to_string(),
        input_word: "not".to_string(),
        confidence: 0.9,
        classification_type: ClassificationType::ExactMatch,
        context: None,
        properties: HashMap::new(),
    };

    let stop_classification = WordClassification {
        word_class_type: WordClassType::StopWords,
        word_class_id: "stop1".to_string(),
        word_class_name: "Stop Words".to_string(),
        matched_word: "the".to_string(),
        input_word: "the".to_string(),
        confidence: 0.8,
        classification_type: ClassificationType::ExactMatch,
        context: None,
        properties: HashMap::new(),
    };

    let discourse_classification = WordClassification {
        word_class_type: WordClassType::DiscourseMarkers,
        word_class_id: "disc1".to_string(),
        word_class_name: "Discourse".to_string(),
        matched_word: "however".to_string(),
        input_word: "however".to_string(),
        confidence: 0.7,
        classification_type: ClassificationType::ExactMatch,
        context: None,
        properties: HashMap::new(),
    };

    analysis.classifications.push(negation_classification);
    analysis.classifications.push(stop_classification);
    analysis.classifications.push(discourse_classification);

    // Add pattern match
    let pattern_match = PatternMatch {
        word_class_type: WordClassType::Functional,
        word_class_id: "morph1".to_string(),
        pattern_id: "ing-suffix".to_string(),
        pattern_type: PatternType::Suffix,
        input_word: "running".to_string(),
        matched_text: "ing".to_string(),
        confidence: 0.8,
        description: "Present participle".to_string(),
    };

    analysis.pattern_matches.push(pattern_match);

    // Test functionality
    assert!(analysis.has_results());

    let negations = analysis.get_negations();
    assert_eq!(negations.len(), 1);
    assert_eq!(negations[0].matched_word, "not");

    let stop_words = analysis.get_stop_words();
    assert_eq!(stop_words.len(), 1);
    assert_eq!(stop_words[0].matched_word, "the");

    let discourse_markers = analysis.get_discourse_markers();
    assert_eq!(discourse_markers.len(), 1);
    assert_eq!(discourse_markers[0].matched_word, "however");

    // Test confidence calculation
    analysis.calculate_confidence();
    assert!((analysis.confidence - 0.8).abs() < 0.001); // (0.9+0.8+0.7+0.8)/4 = 0.8

    // Test empty analysis
    let mut empty_analysis = LexiconAnalysis::new("empty".to_string());
    empty_analysis.calculate_confidence();
    assert_eq!(empty_analysis.confidence, 0.0);
}
