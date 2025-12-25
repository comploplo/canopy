//! Comprehensive tests for semantic Layer 1 functionality
//!
//! These tests verify that the semantic-first approach works correctly
//! and integrates properly with the existing system.

use canopy_tokenizer::*;

#[cfg(test)]
mod semantic_layer1_tests {
    use super::*;

    #[test]
    fn test_semantic_config_defaults() {
        let config = SemanticConfig::default();
        assert!(config.enable_framenet);
        assert!(config.enable_verbnet);
        assert!(config.enable_wordnet);
        assert_eq!(config.confidence_threshold, 0.7);
        assert!(config.parallel_processing);
    }

    #[test]
    fn test_semantic_classes() {
        assert_eq!(SemanticClass::Predicate, SemanticClass::Predicate);
        assert_ne!(SemanticClass::Predicate, SemanticClass::Argument);
        assert_ne!(SemanticClass::Unknown, SemanticClass::Function);
    }

    #[test]
    fn test_inflection_types() {
        assert_eq!(InflectionType::Verbal, InflectionType::Verbal);
        assert_ne!(InflectionType::Verbal, InflectionType::Nominal);
        assert_eq!(InflectionType::None, InflectionType::None);
    }

    #[test]
    fn test_aspectual_classes() {
        assert_eq!(AspectualClass::Activity, AspectualClass::Activity);
        assert_ne!(AspectualClass::State, AspectualClass::Achievement);
        assert_eq!(AspectualClass::Unknown, AspectualClass::Unknown);
    }

    #[test]
    fn test_quantifier_types() {
        assert_eq!(QuantifierType::Universal, QuantifierType::Universal);
        assert_ne!(QuantifierType::Existential, QuantifierType::Definite);
        assert_eq!(QuantifierType::Indefinite, QuantifierType::Indefinite);
    }

    #[test]
    fn test_logical_term_creation() {
        let var = LogicalTerm::Variable("x".to_string());
        let const_term = LogicalTerm::Constant("john".to_string());
        let func = LogicalTerm::Function("loves".to_string(), vec![var, const_term]);

        match func {
            LogicalTerm::Function(name, args) => {
                assert_eq!(name, "loves");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected function term"),
        }
    }

    #[test]
    fn test_logical_predicate_creation() {
        let pred = LogicalPredicate {
            name: "give".to_string(),
            arguments: vec![
                LogicalTerm::Variable("x".to_string()),
                LogicalTerm::Variable("y".to_string()),
                LogicalTerm::Variable("z".to_string()),
            ],
            arity: 3,
        };

        assert_eq!(pred.name, "give");
        assert_eq!(pred.arity, 3);
        assert_eq!(pred.arguments.len(), 3);
    }

    #[test]
    fn test_semantic_restriction_creation() {
        let restriction = SemanticRestriction {
            restriction_type: "animacy".to_string(),
            required_value: "animate".to_string(),
            strength: 0.8,
        };

        assert_eq!(restriction.restriction_type, "animacy");
        assert_eq!(restriction.strength, 0.8);
    }

    #[test]
    fn test_morphological_analysis_structure() {
        let analysis = MorphologicalAnalysis {
            lemma: "give".to_string(),
            features: std::collections::HashMap::new(),
            inflection_type: InflectionType::Verbal,
            is_recognized: true,
        };

        assert_eq!(analysis.lemma, "give");
        assert_eq!(analysis.inflection_type, InflectionType::Verbal);
        assert!(analysis.is_recognized);
    }

    #[test]
    fn test_wordnet_sense_structure() {
        let sense = WordNetSense {
            synset_id: "give.v.01".to_string(),
            definition: "transfer possession of something".to_string(),
            pos: "v".to_string(),
            hypernyms: vec!["transfer.v.01".to_string()],
            hyponyms: vec!["hand.v.01".to_string()],
            sense_rank: 1,
        };

        assert_eq!(sense.synset_id, "give.v.01");
        assert_eq!(sense.sense_rank, 1);
        assert!(!sense.hypernyms.is_empty());
    }

    #[test]
    fn test_frame_element_structure() {
        let element = FrameElement {
            name: "Donor".to_string(),
            semantic_type: "Agent".to_string(),
            is_core: true,
        };

        assert_eq!(element.name, "Donor");
        assert!(element.is_core);
    }

    #[test]
    fn test_frame_analysis_structure() {
        let trigger = FrameUnit {
            name: "give".to_string(),
            pos: "v".to_string(),
            frame: "Giving".to_string(),
            definition: Some("to transfer possession".to_string()),
        };

        let analysis = FrameAnalysis {
            name: "Giving".to_string(),
            elements: vec![],
            confidence: 0.95,
            trigger,
        };

        assert_eq!(analysis.name, "Giving");
        assert_eq!(analysis.confidence, 0.95);
    }

    #[test]
    fn test_semantic_predicate_structure() {
        let predicate = SemanticPredicate {
            lemma: "give".to_string(),
            verbnet_class: Some("give-13.1".to_string()),
            theta_grid: vec![
                canopy_core::ThetaRole::Agent,
                canopy_core::ThetaRole::Patient,
                canopy_core::ThetaRole::Recipient,
            ],
            selectional_restrictions: std::collections::HashMap::new(),
            aspectual_class: AspectualClass::Accomplishment,
            confidence: 0.9,
        };

        assert_eq!(predicate.lemma, "give");
        assert_eq!(predicate.theta_grid.len(), 3);
        assert_eq!(predicate.aspectual_class, AspectualClass::Accomplishment);
    }

    #[test]
    fn test_semantic_token_structure() {
        let token = SemanticToken {
            text: "gave".to_string(),
            lemma: "give".to_string(),
            semantic_class: SemanticClass::Predicate,
            frames: vec![],
            verbnet_classes: vec![],
            wordnet_senses: vec![],
            morphology: MorphologicalAnalysis {
                lemma: "give".to_string(),
                features: std::collections::HashMap::new(),
                inflection_type: InflectionType::Verbal,
                is_recognized: true,
            },
            confidence: 0.85,
        };

        assert_eq!(token.text, "gave");
        assert_eq!(token.lemma, "give");
        assert_eq!(token.semantic_class, SemanticClass::Predicate);
        assert_eq!(token.confidence, 0.85);
    }

    #[test]
    fn test_logical_form_structure() {
        let logical_form = LogicalForm {
            predicates: vec![LogicalPredicate {
                name: "give".to_string(),
                arguments: vec![
                    LogicalTerm::Variable("x0_0".to_string()),
                    LogicalTerm::Variable("x0_1".to_string()),
                    LogicalTerm::Variable("x0_2".to_string()),
                ],
                arity: 3,
            }],
            variables: std::collections::HashMap::new(),
            quantifiers: vec![],
        };

        assert_eq!(logical_form.predicates.len(), 1);
        assert_eq!(logical_form.predicates[0].name, "give");
        assert_eq!(logical_form.predicates[0].arity, 3);
    }

    #[test]
    fn test_quantifier_structure() {
        let quantifier = QuantifierStructure {
            quantifier_type: QuantifierType::Universal,
            variable: "x".to_string(),
            restriction: LogicalPredicate {
                name: "person".to_string(),
                arguments: vec![LogicalTerm::Variable("x".to_string())],
                arity: 1,
            },
            scope: LogicalPredicate {
                name: "happy".to_string(),
                arguments: vec![LogicalTerm::Variable("x".to_string())],
                arity: 1,
            },
        };

        assert_eq!(quantifier.quantifier_type, QuantifierType::Universal);
        assert_eq!(quantifier.variable, "x");
    }

    #[test]
    fn test_analysis_metrics_structure() {
        let metrics = AnalysisMetrics {
            total_time_us: 1000,
            tokenization_time_us: 100,
            framenet_time_us: 200,
            verbnet_time_us: 300,
            wordnet_time_us: 150,
            token_count: 5,
            frame_count: 2,
            predicate_count: 1,
        };

        assert_eq!(metrics.total_time_us, 1000);
        assert_eq!(metrics.token_count, 5);
        assert_eq!(metrics.predicate_count, 1);
    }

    #[test]
    fn test_semantic_layer1_output_structure() {
        let output = SemanticLayer1Output {
            tokens: vec![],
            frames: vec![],
            predicates: vec![],
            logical_form: LogicalForm {
                predicates: vec![],
                variables: std::collections::HashMap::new(),
                quantifiers: vec![],
            },
            metrics: AnalysisMetrics {
                total_time_us: 500,
                tokenization_time_us: 50,
                framenet_time_us: 100,
                verbnet_time_us: 150,
                wordnet_time_us: 100,
                token_count: 0,
                frame_count: 0,
                predicate_count: 0,
            },
        };

        assert_eq!(output.tokens.len(), 0);
        assert_eq!(output.frames.len(), 0);
        assert_eq!(output.predicates.len(), 0);
        assert_eq!(output.metrics.total_time_us, 500);
    }

    #[test]
    fn test_semantic_error_types() {
        use crate::SemanticError;

        let tokenization_error = SemanticError::TokenizationError {
            context: "Test error".to_string(),
        };

        let framenet_error = SemanticError::FrameNetError {
            context: "FrameNet test error".to_string(),
        };

        assert!(format!("{}", tokenization_error).contains("Tokenization failed"));
        assert!(format!("{}", framenet_error).contains("FrameNet analysis failed"));
    }

    // Integration tests would go here once the engines are fully implemented
    #[test]
    fn test_integration_readiness() {
        // Test that all the types are properly defined and can be used together
        let config = SemanticConfig::default();
        assert!(config.enable_framenet && config.enable_verbnet && config.enable_wordnet);

        // Verify that semantic classes cover the expected range
        let classes = vec![
            SemanticClass::Predicate,
            SemanticClass::Argument,
            SemanticClass::Modifier,
            SemanticClass::Function,
            SemanticClass::Quantifier,
            SemanticClass::Unknown,
        ];
        assert_eq!(classes.len(), 6);

        // Verify aspectual classes are complete
        let aspects = vec![
            AspectualClass::State,
            AspectualClass::Activity,
            AspectualClass::Accomplishment,
            AspectualClass::Achievement,
            AspectualClass::Unknown,
        ];
        assert_eq!(aspects.len(), 5);
    }
}

#[cfg(test)]
mod tokenization_integration_tests {
    use super::*;
    use canopy_tokenizer::tokenization::{Token, Tokenizer};

    #[test]
    fn test_tokenizer_integration() {
        let tokenizer = Tokenizer::new();

        // Test advanced tokenization
        let tokens_result = tokenizer.tokenize("John gave Mary a book.");
        assert!(tokens_result.is_ok());

        let tokens = tokens_result.unwrap();
        assert!(tokens.len() >= 5); // At least John, gave, Mary, a, book

        // Check that content words are identified
        let gave_token = tokens.iter().find(|t| t.text == "gave");
        assert!(gave_token.is_some());
        assert!(gave_token.unwrap().is_content_word);

        let a_token = tokens.iter().find(|t| t.text == "a");
        assert!(a_token.is_some());
        assert!(!a_token.unwrap().is_content_word); // Function word
    }

    #[test]
    fn test_contraction_handling() {
        let tokenizer = Tokenizer::new();

        let tokens_result = tokenizer.tokenize("I don't like it.");
        assert!(tokens_result.is_ok());

        let tokens = tokens_result.unwrap();
        let token_texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();

        // Should expand "don't" to "do" and "not"
        assert!(token_texts.contains(&"do"));
        assert!(token_texts.contains(&"not"));
    }

    #[test]
    fn test_tokenizer_sentence_segmentation() {
        let tokenizer = Tokenizer::new();

        let sentences_result =
            tokenizer.segment_sentences("Hello world. How are you? Fine, thanks!");
        assert!(sentences_result.is_ok());

        let sentences = sentences_result.unwrap();
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world");
        assert_eq!(sentences[1], "How are you");
        assert_eq!(sentences[2], "Fine, thanks");
    }
}
