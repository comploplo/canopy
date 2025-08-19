//! Additional utility tests to improve test coverage
//!
//! These tests target simple utility functions and edge cases that may not
//! be covered by the main test suite but are important for overall code quality.

use crate::*;

#[cfg(test)]
mod utility_coverage_tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        // Test document creation with new() method
        let doc = Document::new("test text".to_string(), vec![]);
        assert!(doc.sentences.is_empty());
        assert_eq!(doc.text, "test text");
    }

    #[test]
    fn test_sentence_creation() {
        // Test sentence creation with new() method
        let sent = Sentence::new(vec![]);
        assert!(sent.words.is_empty());
        assert_eq!(sent.words.len(), 0);
    }

    #[test]
    fn test_theta_role_debug_display() {
        // Test debug formatting for ThetaRole enum
        let roles = vec![
            ThetaRole::Agent,
            ThetaRole::Patient,
            ThetaRole::Theme,
            ThetaRole::Experiencer,
            ThetaRole::Benefactive,
            ThetaRole::Source,
            ThetaRole::Goal,
        ];

        for role in &roles {
            let debug_str = format!("{:?}", role);
            assert!(!debug_str.is_empty());
            assert!(debug_str.len() > 3); // Should be meaningful
        }

        // Test role equality
        assert_eq!(ThetaRole::Agent, ThetaRole::Agent);
        assert_ne!(ThetaRole::Agent, ThetaRole::Patient);
    }

    #[test]
    fn test_upos_variants() {
        // Test UPos enum variants for coverage
        let variants = vec![
            UPos::Noun,
            UPos::Verb,
            UPos::Adj,
            UPos::Adv,
            UPos::Pron,
            UPos::Det,
            UPos::Adp,
            UPos::Num,
            UPos::Cconj,
            UPos::Part,
            UPos::Punct,
            UPos::Sym,
            UPos::Intj,
            UPos::X,
            UPos::Aux,
            UPos::Cconj,
            UPos::Sconj,
        ];

        for variant in &variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_deprel_variants() {
        // Test DepRel enum variants for coverage
        let variants = vec![
            DepRel::Root,
            DepRel::Nsubj,
            DepRel::Obj,
            DepRel::Iobj,
            DepRel::Csubj,
            DepRel::Xcomp,
            DepRel::Ccomp,
            DepRel::Amod,
            DepRel::Nmod,
            DepRel::Advmod,
            DepRel::Acl,
            DepRel::Advcl,
            DepRel::Aux,
            DepRel::Cop,
            DepRel::Mark,
            DepRel::Det,
            DepRel::Clf,
            DepRel::Case,
            DepRel::Conj,
            DepRel::Cc,
            DepRel::Fixed,
            DepRel::Flat,
            DepRel::Compound,
            DepRel::List,
            DepRel::Parataxis,
            DepRel::Orphan,
            DepRel::Goeswith,
            DepRel::Reparandum,
            DepRel::Punct,
            DepRel::Appos,
            DepRel::Nummod,
            DepRel::Discourse,
            DepRel::Vocative,
            DepRel::Expl,
            DepRel::Dislocated,
            DepRel::Obl,
            DepRel::CsubjPass,
            DepRel::NsubjPass,
            DepRel::Other("unknown".to_string()),
        ];

        for variant in &variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_animacy_variants() {
        // Test Animacy enum variants
        let variants = vec![Animacy::Animal, Animacy::Human, Animacy::Inanimate];

        for variant in &variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_morphological_features_defaults() {
        // Test MorphFeatures with Default
        let morph = MorphFeatures::default();
        assert_eq!(morph.animacy, None);
        assert_eq!(morph.aspect, None);
        assert_eq!(morph.case, None);
        assert_eq!(morph.number, None);
        assert_eq!(morph.voice, None);
    }

    #[test]
    fn test_error_variants() {
        // Test CanopyError variants for coverage
        let parser_error = CanopyError::ParseError {
            context: "test error".to_string(),
        };
        let semantic_error = CanopyError::SemanticError("semantic issue".to_string());
        let lsp_error = CanopyError::LspError("lsp problem".to_string());

        // Test debug formatting
        let parser_debug = format!("{:?}", parser_error);
        let semantic_debug = format!("{:?}", semantic_error);
        let lsp_debug = format!("{:?}", lsp_error);

        assert!(parser_debug.contains("ParseError"));
        assert!(semantic_debug.contains("SemanticError"));
        assert!(lsp_debug.contains("LspError"));
    }

    #[test]
    fn test_semantic_feature_patterns() {
        // Test basic SemanticFeatures patterns
        let semantic_features = SemanticFeatures {
            animacy: Some(Animacy::Human),
            definiteness: None,
            countability: None,
            concreteness: None,
        };

        // Test debug formatting
        let features_debug = format!("{:?}", semantic_features);
        assert!(features_debug.contains("animacy"));
        assert!(features_debug.contains("Human"));
    }

    #[test]
    fn test_theta_role_count() {
        // Verify we have all expected theta roles using the official all() method
        let all_roles = ThetaRole::all();

        // Should have 19 roles total (from current Rust system)
        assert_eq!(all_roles.len(), 19);

        // Test that all roles are unique
        let mut role_set = std::collections::HashSet::new();
        for role in all_roles {
            assert!(role_set.insert(*role), "Duplicate role found: {:?}", role);
        }
    }

    #[test]
    fn test_little_v_variants() {
        // Test LittleV enum variants for coverage - using simplified instances
        use crate::{Action, Entity, PsychType, State};

        let entity = Entity {
            id: 1,
            text: "test".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: None,
        };
        let state = State {
            predicate: "happy".to_string(),
            polarity: true,
        };
        let action = Action {
            predicate: "run".to_string(),
            manner: None,
            instrument: None,
        };

        let variants = vec![
            LittleV::Cause {
                causer: entity.clone(),
                caused_predicate: "break".to_string(),
                caused_theme: entity.clone(),
            },
            LittleV::Become {
                theme: entity.clone(),
                result_state: state.clone(),
            },
            LittleV::Be {
                theme: entity.clone(),
                state: state,
            },
            LittleV::Do {
                agent: entity.clone(),
                action: action,
            },
            LittleV::Experience {
                experiencer: entity.clone(),
                stimulus: entity.clone(),
                psych_type: PsychType::SubjectExp,
            },
        ];

        for variant in &variants {
            let debug_str = format!("{:?}", variant);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_path_creation() {
        // Test Path struct creation and basic methods
        use crate::{Direction, Entity};

        let entity = Entity {
            id: 2,
            text: "location".to_string(),
            animacy: None,
            definiteness: None,
        };

        let path = Path {
            source: Some(entity.clone()),
            goal: Some(entity.clone()),
            route: None,
            direction: Some(Direction::Up),
        };

        assert!(path.source.is_some());
        assert!(path.goal.is_some());
        assert!(path.route.is_none());
        assert!(path.direction.is_some());
    }

    #[test]
    fn test_state_creation() {
        // Test State struct creation
        let state = State {
            predicate: "happy".to_string(),
            polarity: true,
        };

        assert_eq!(state.predicate, "happy");
        assert!(state.polarity);

        let negative_state = State {
            predicate: "sad".to_string(),
            polarity: false,
        };

        assert_eq!(negative_state.predicate, "sad");
        assert!(!negative_state.polarity);
    }
}
