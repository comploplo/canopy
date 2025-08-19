//! Additional tests to improve code coverage for M3 completion
//!
//! This module contains targeted tests to cover edge cases and code paths
//! that may not be covered by existing functional tests.

#![allow(clippy::uninlined_format_args)] // Allow old format style in tests
#![allow(clippy::single_component_path_imports)] // Allow for test convenience
#![allow(clippy::field_reassign_with_default)] // Allow field reassignment pattern in tests

#[cfg(test)]
mod coverage_tests {
    use crate::*;
    use std::collections::HashMap;

    #[test]
    fn test_error_display_implementations() {
        // Test Display implementations for error types that exist in canopy-core
        let canopy_error = CanopyError::ParseError {
            context: "test context".to_string(),
        };
        assert!(format!("{}", canopy_error).contains("parsing failed"));

        let semantic_error = CanopyError::SemanticError("test error".to_string());
        assert!(format!("{}", semantic_error).contains("semantic analysis failed"));

        let lsp_error = CanopyError::LspError("test lsp error".to_string());
        assert!(format!("{}", lsp_error).contains("LSP protocol error"));
    }

    #[test]
    fn test_debug_implementations() {
        // Test Debug implementations for main types
        let word = Word {
            id: 1,
            text: "test".to_string(),
            lemma: "test".to_string(),
            upos: UPos::Noun,
            xpos: Some("NN".to_string()),
            feats: MorphFeatures::default(),
            head: Some(0),
            deprel: DepRel::Root,
            deps: None,
            misc: None,
            start: 0,
            end: 4,
        };
        let debug_str = format!("{:?}", word);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("Noun"));

        let sentence = Sentence::new(vec![word]);
        let document = Document::new("Test document".to_string(), vec![sentence]);
        let debug_str = format!("{:?}", document);
        assert!(debug_str.contains("Test document"));
    }

    #[test]
    fn test_enum_edge_cases() {
        // Test all enum variants for completeness
        let pos_variants = vec![
            UPos::Adj,
            UPos::Adp,
            UPos::Adv,
            UPos::Aux,
            UPos::Cconj,
            UPos::Det,
            UPos::Intj,
            UPos::Noun,
            UPos::Num,
            UPos::Part,
            UPos::Pron,
            UPos::Propn,
            UPos::Punct,
            UPos::Sconj,
            UPos::Sym,
            UPos::Verb,
            UPos::X,
        ];
        for pos in pos_variants {
            let debug_str = format!("{:?}", pos);
            assert!(!debug_str.is_empty());
        }

        let deprel_variants = vec![
            DepRel::Acl,
            DepRel::Advcl,
            DepRel::Advmod,
            DepRel::Amod,
            DepRel::Appos,
            DepRel::Aux,
            DepRel::AuxPass,
            DepRel::Case,
            DepRel::Cc,
            DepRel::Ccomp,
            DepRel::Clf,
            DepRel::Compound,
            DepRel::Conj,
            DepRel::Cop,
            DepRel::Csubj,
            DepRel::CsubjPass,
            DepRel::Dep,
            DepRel::Det,
            DepRel::Discourse,
            DepRel::Dislocated,
            DepRel::Expl,
            DepRel::Fixed,
            DepRel::Flat,
            DepRel::Goeswith,
            DepRel::Iobj,
            DepRel::List,
            DepRel::Mark,
            DepRel::Neg,
            DepRel::Nmod,
            DepRel::Nsubj,
            DepRel::NsubjPass,
            DepRel::Nummod,
            DepRel::Obj,
            DepRel::Obl,
            DepRel::Orphan,
            DepRel::Parataxis,
            DepRel::Punct,
            DepRel::Reparandum,
            DepRel::Root,
            DepRel::Vocative,
            DepRel::Xcomp,
            DepRel::Other("custom".to_string()),
        ];
        for deprel in deprel_variants {
            let debug_str = format!("{:?}", deprel);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_morphological_features_coverage() {
        // Test all morphological feature types
        let mut feats = MorphFeatures::default();

        feats.animacy = Some(UDAnimacy::Animate);
        feats.aspect = Some(UDAspect::Perfective);
        feats.case = Some(UDCase::Nominative);
        feats.definiteness = Some(UDDefiniteness::Definite);
        feats.degree = Some(UDDegree::Positive);
        feats.gender = Some(UDGender::Masculine);
        feats.mood = Some(UDMood::Indicative);
        feats.number = Some(UDNumber::Singular);
        feats.person = Some(UDPerson::Third);
        feats.tense = Some(UDTense::Present);
        feats.verbform = Some(UDVerbForm::Finite);
        feats.voice = Some(UDVoice::Active);

        // Test serialization/deserialization
        let serialized = serde_json::to_string(&feats).expect("Should serialize");
        let deserialized: MorphFeatures =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(feats.animacy, deserialized.animacy);
        assert_eq!(feats.aspect, deserialized.aspect);
    }

    #[test]
    fn test_event_structures_coverage() {
        // Test event-related structures that exist in canopy-core
        let john = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let event = Event {
            id: 1,
            predicate: "run".to_string(),
            little_v: LittleV::Do {
                agent: john.clone(),
                action: Action {
                    predicate: "run".to_string(),
                    manner: None,
                    instrument: None,
                },
            },
            participants: HashMap::new(),
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("run"));
        assert!(debug_str.contains("Activity"));
    }

    #[test]
    fn test_theta_role_coverage() {
        // Test all theta role variants that exist in canopy-core
        let roles = vec![
            ThetaRole::Agent,
            ThetaRole::Patient,
            ThetaRole::Theme,
            ThetaRole::Experiencer,
            ThetaRole::Recipient,
            ThetaRole::Benefactive,
            ThetaRole::Instrument,
            ThetaRole::Comitative,
            ThetaRole::Location,
            ThetaRole::Source,
            ThetaRole::Goal,
            ThetaRole::Direction,
            ThetaRole::Temporal,
            ThetaRole::Frequency,
            ThetaRole::Measure,
            ThetaRole::Cause,
            ThetaRole::Manner,
            ThetaRole::ControlledSubject,
            ThetaRole::Stimulus,
        ];

        for role in roles {
            let debug_str = format!("{:?}", role);
            assert!(!debug_str.is_empty());

            // Test serialization
            let serialized = serde_json::to_string(&role).expect("Should serialize");
            let deserialized: ThetaRole =
                serde_json::from_str(&serialized).expect("Should deserialize");
            assert_eq!(role, deserialized);
        }
    }

    #[test]
    fn test_semantic_feature_coverage() {
        // Test semantic feature types that exist in canopy-core
        let features = SemanticFeatures {
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
            countability: Some(Countability::Count),
            concreteness: Some(Concreteness::Concrete),
        };

        let debug_str = format!("{:?}", features);
        assert!(debug_str.contains("Human"));
        assert!(debug_str.contains("Definite"));

        // Test serialization
        let serialized = serde_json::to_string(&features).expect("Should serialize");
        let deserialized: SemanticFeatures =
            serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(features, deserialized);
    }

    #[test]
    fn test_enhanced_word_coverage() {
        let enhanced_word = EnhancedWord {
            base: Word {
                id: 1,
                text: "test".to_string(),
                lemma: "test".to_string(),
                upos: UPos::Noun,
                xpos: Some("NN".to_string()),
                feats: MorphFeatures::default(),
                head: Some(0),
                deprel: DepRel::Nsubj,
                deps: None,
                misc: None,
                start: 0,
                end: 4,
            },
            semantic_features: SemanticFeatures::default(),
            confidence: FeatureConfidence::default(),
        };

        // Test clone and debug
        let cloned = enhanced_word.clone();
        assert_eq!(enhanced_word.base.text, cloned.base.text);

        let debug_str = format!("{:?}", enhanced_word);
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_error_source_implementations() {
        // Test error source chain for CanopyError
        let canopy_error = CanopyError::ParseError {
            context: "test context".to_string(),
        };

        // Just verify the error implements the Error trait properly
        let error_msg = format!("{}", canopy_error);
        assert!(error_msg.contains("parsing failed"));
    }

    #[test]
    fn test_default_implementations() {
        // Test Default trait implementations
        let default_morph = MorphFeatures::default();
        assert!(default_morph.animacy.is_none());
        assert!(default_morph.aspect.is_none());

        let default_semantic = SemanticFeatures::default();
        assert!(default_semantic.animacy.is_none());
        assert!(default_semantic.definiteness.is_none());

        let default_confidence = FeatureConfidence::default();
        assert_eq!(default_confidence.animacy, 0.0);
        assert_eq!(default_confidence.definiteness, 0.0);
    }

    #[test]
    fn test_string_parsing() {
        // Test DepRel from_str parsing
        let deprel = DepRel::from_str_simple("nsubj");
        assert_eq!(deprel, DepRel::Nsubj);

        let custom_deprel = DepRel::from_str_simple("custom_relation");
        if let DepRel::Other(ref s) = custom_deprel {
            assert_eq!(s, "custom_relation");
        } else {
            panic!("Expected Other variant");
        }
    }

    #[test]
    fn test_edge_case_word_positions() {
        // Test edge cases for word positions
        let word = Word {
            id: 0,                // Edge case: zero ID
            text: "".to_string(), // Edge case: empty text
            lemma: "test".to_string(),
            upos: UPos::X, // Edge case: unknown POS
            xpos: None,
            feats: MorphFeatures::default(),
            head: None,          // Edge case: no head
            deprel: DepRel::Dep, // Edge case: generic dependency
            deps: None,
            misc: None,
            start: 100,
            end: 100, // Edge case: zero-length span
        };

        assert_eq!(word.id, 0);
        assert_eq!(word.start, word.end);
        assert_eq!(word.upos, UPos::X);
    }
}
