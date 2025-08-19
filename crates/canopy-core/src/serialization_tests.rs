//! Comprehensive serialization round-trip tests for all core data structures
//!
//! These tests ensure that all serializable types can be properly serialized
//! and deserialized without data loss, which is crucial for LSP protocol
//! and data persistence functionality.

#![allow(clippy::single_component_path_imports)] // Allow for test convenience

#[cfg(test)]
mod serialization_round_trip_tests {
    use crate::*;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn test_theta_role_serialization() {
        // Test all theta role variants
        for &role in ThetaRole::all() {
            let json = serde_json::to_string(&role).expect("Failed to serialize ThetaRole");
            let deserialized: ThetaRole =
                serde_json::from_str(&json).expect("Failed to deserialize ThetaRole");
            assert_eq!(role, deserialized);
        }
    }

    #[test]
    fn test_upos_serialization() {
        let pos_tags = vec![
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

        for pos in pos_tags {
            let json = serde_json::to_string(&pos).expect("Failed to serialize UPos");
            let deserialized: UPos =
                serde_json::from_str(&json).expect("Failed to deserialize UPos");
            assert_eq!(pos, deserialized);
        }
    }

    #[test]
    fn test_morphological_feature_enums_serialization() {
        // Test UDPerson
        let persons = vec![UDPerson::First, UDPerson::Second, UDPerson::Third];
        for person in persons {
            let json = serde_json::to_string(&person).expect("Failed to serialize UDPerson");
            let deserialized: UDPerson =
                serde_json::from_str(&json).expect("Failed to deserialize UDPerson");
            assert_eq!(person, deserialized);
        }

        // Test UDNumber
        let numbers = vec![UDNumber::Singular, UDNumber::Plural, UDNumber::Dual];
        for number in numbers {
            let json = serde_json::to_string(&number).expect("Failed to serialize UDNumber");
            let deserialized: UDNumber =
                serde_json::from_str(&json).expect("Failed to deserialize UDNumber");
            assert_eq!(number, deserialized);
        }

        // Test UDGender
        let genders = vec![UDGender::Masculine, UDGender::Feminine, UDGender::Neuter];
        for gender in genders {
            let json = serde_json::to_string(&gender).expect("Failed to serialize UDGender");
            let deserialized: UDGender =
                serde_json::from_str(&json).expect("Failed to deserialize UDGender");
            assert_eq!(gender, deserialized);
        }

        // Test UDCase
        let cases = vec![
            UDCase::Nominative,
            UDCase::Accusative,
            UDCase::Genitive,
            UDCase::Dative,
            UDCase::Instrumental,
            UDCase::Locative,
            UDCase::Vocative,
            UDCase::Ablative,
        ];
        for case in cases {
            let json = serde_json::to_string(&case).expect("Failed to serialize UDCase");
            let deserialized: UDCase =
                serde_json::from_str(&json).expect("Failed to deserialize UDCase");
            assert_eq!(case, deserialized);
        }

        // Test UDTense
        let tenses = vec![UDTense::Past, UDTense::Present, UDTense::Future];
        for tense in tenses {
            let json = serde_json::to_string(&tense).expect("Failed to serialize UDTense");
            let deserialized: UDTense =
                serde_json::from_str(&json).expect("Failed to deserialize UDTense");
            assert_eq!(tense, deserialized);
        }

        // Test UDAspect
        let aspects = vec![UDAspect::Perfective, UDAspect::Imperfective];
        for aspect in aspects {
            let json = serde_json::to_string(&aspect).expect("Failed to serialize UDAspect");
            let deserialized: UDAspect =
                serde_json::from_str(&json).expect("Failed to deserialize UDAspect");
            assert_eq!(aspect, deserialized);
        }

        // Test UDMood
        let moods = vec![
            UDMood::Indicative,
            UDMood::Imperative,
            UDMood::Conditional,
            UDMood::Subjunctive,
        ];
        for mood in moods {
            let json = serde_json::to_string(&mood).expect("Failed to serialize UDMood");
            let deserialized: UDMood =
                serde_json::from_str(&json).expect("Failed to deserialize UDMood");
            assert_eq!(mood, deserialized);
        }

        // Test UDVoice
        let voices = vec![UDVoice::Active, UDVoice::Passive, UDVoice::Middle];
        for voice in voices {
            let json = serde_json::to_string(&voice).expect("Failed to serialize UDVoice");
            let deserialized: UDVoice =
                serde_json::from_str(&json).expect("Failed to deserialize UDVoice");
            assert_eq!(voice, deserialized);
        }

        // Test UDVerbForm
        let verb_forms = vec![
            UDVerbForm::Finite,
            UDVerbForm::Infinitive,
            UDVerbForm::Participle,
            UDVerbForm::Gerund,
            UDVerbForm::ConverbalAdverbial,
        ];
        for verb_form in verb_forms {
            let json = serde_json::to_string(&verb_form).expect("Failed to serialize UDVerbForm");
            let deserialized: UDVerbForm =
                serde_json::from_str(&json).expect("Failed to deserialize UDVerbForm");
            assert_eq!(verb_form, deserialized);
        }
    }

    #[test]
    fn test_morph_features_serialization() {
        // Test empty MorphFeatures
        let empty_features = MorphFeatures::default();
        let json =
            serde_json::to_string(&empty_features).expect("Failed to serialize MorphFeatures");
        let deserialized: MorphFeatures =
            serde_json::from_str(&json).expect("Failed to deserialize MorphFeatures");
        assert_eq!(empty_features, deserialized);

        // Test fully populated MorphFeatures
        let full_features = MorphFeatures {
            person: Some(UDPerson::Third),
            number: Some(UDNumber::Singular),
            gender: Some(UDGender::Masculine),
            animacy: Some(UDAnimacy::Animate),
            case: Some(UDCase::Nominative),
            definiteness: Some(UDDefiniteness::Definite),
            tense: Some(UDTense::Present),
            aspect: Some(UDAspect::Perfective),
            mood: Some(UDMood::Indicative),
            voice: Some(UDVoice::Active),
            degree: Some(UDDegree::Positive),
            verbform: Some(UDVerbForm::Finite),
            raw_features: Some("Animacy=Anim|Case=Nom".to_string()),
        };

        let json =
            serde_json::to_string(&full_features).expect("Failed to serialize MorphFeatures");
        let deserialized: MorphFeatures =
            serde_json::from_str(&json).expect("Failed to deserialize MorphFeatures");
        assert_eq!(full_features, deserialized);
    }

    #[test]
    fn test_deprel_serialization() {
        // Test all DepRel variants
        let deprels = vec![
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
            DepRel::Other("custom_relation".to_string()),
        ];

        for deprel in deprels {
            let json = serde_json::to_string(&deprel).expect("Failed to serialize DepRel");
            let deserialized: DepRel =
                serde_json::from_str(&json).expect("Failed to deserialize DepRel");
            assert_eq!(deprel, deserialized);
        }
    }

    #[test]
    fn test_word_serialization() {
        let word = Word {
            id: 1,
            text: "running".to_string(),
            lemma: "run".to_string(),
            upos: UPos::Verb,
            xpos: Some("VBG".to_string()),
            feats: MorphFeatures {
                tense: Some(UDTense::Present),
                aspect: Some(UDAspect::Perfective),
                verbform: Some(UDVerbForm::Participle),
                ..MorphFeatures::default()
            },
            head: Some(0),
            deprel: DepRel::Root,
            deps: Some("1:nsubj".to_string()),
            misc: Some("SpaceAfter=No".to_string()),
            start: 0,
            end: 7,
        };

        let json = serde_json::to_string(&word).expect("Failed to serialize Word");
        let deserialized: Word = serde_json::from_str(&json).expect("Failed to deserialize Word");
        assert_eq!(word, deserialized);
    }

    #[test]
    fn test_sentence_serialization() {
        let words = vec![
            Word::new(1, "The".to_string(), 0, 3),
            Word::new(2, "cat".to_string(), 4, 7),
            Word::new(3, "runs".to_string(), 8, 12),
        ];

        let sentence = Sentence::new(words);

        let json = serde_json::to_string(&sentence).expect("Failed to serialize Sentence");
        let deserialized: Sentence =
            serde_json::from_str(&json).expect("Failed to deserialize Sentence");
        assert_eq!(sentence, deserialized);
    }

    #[test]
    fn test_document_serialization() {
        let words1 = vec![
            Word::new(1, "The".to_string(), 0, 3),
            Word::new(2, "cat".to_string(), 4, 7),
            Word::new(3, "runs".to_string(), 8, 12),
        ];
        let words2 = vec![
            Word::new(4, "Dogs".to_string(), 14, 18),
            Word::new(5, "bark".to_string(), 19, 23),
        ];

        let sentences = vec![Sentence::new(words1), Sentence::new(words2)];

        let document = Document::new("The cat runs. Dogs bark.".to_string(), sentences);

        let json = serde_json::to_string(&document).expect("Failed to serialize Document");
        let deserialized: Document =
            serde_json::from_str(&json).expect("Failed to deserialize Document");
        assert_eq!(document, deserialized);
    }

    #[test]
    fn test_semantic_features_serialization() {
        // Test empty semantic features
        let empty_features = SemanticFeatures::default();
        let json =
            serde_json::to_string(&empty_features).expect("Failed to serialize SemanticFeatures");
        let deserialized: SemanticFeatures =
            serde_json::from_str(&json).expect("Failed to deserialize SemanticFeatures");
        assert_eq!(empty_features, deserialized);

        // Test populated semantic features
        let features = SemanticFeatures {
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
            countability: Some(Countability::Count),
            concreteness: Some(Concreteness::Concrete),
        };

        let json = serde_json::to_string(&features).expect("Failed to serialize SemanticFeatures");
        let deserialized: SemanticFeatures =
            serde_json::from_str(&json).expect("Failed to deserialize SemanticFeatures");
        assert_eq!(features, deserialized);
    }

    #[test]
    fn test_feature_confidence_serialization() {
        let confidence = FeatureConfidence {
            animacy: 0.95,
            definiteness: 0.80,
            countability: 0.75,
            concreteness: 0.90,
        };

        let json =
            serde_json::to_string(&confidence).expect("Failed to serialize FeatureConfidence");
        let deserialized: FeatureConfidence =
            serde_json::from_str(&json).expect("Failed to deserialize FeatureConfidence");
        assert_eq!(confidence, deserialized);
    }

    #[test]
    fn test_enhanced_word_serialization() {
        let base_word = Word::new(1, "cat".to_string(), 0, 3);
        let semantic_features = SemanticFeatures {
            animacy: Some(Animacy::Animal),
            definiteness: Some(Definiteness::Indefinite),
            countability: Some(Countability::Count),
            concreteness: Some(Concreteness::Concrete),
        };
        let confidence = FeatureConfidence {
            animacy: 0.95,
            definiteness: 0.80,
            countability: 0.90,
            concreteness: 0.85,
        };

        let enhanced_word = EnhancedWord {
            base: base_word,
            semantic_features,
            confidence,
        };

        let json = serde_json::to_string(&enhanced_word).expect("Failed to serialize EnhancedWord");
        let deserialized: EnhancedWord =
            serde_json::from_str(&json).expect("Failed to deserialize EnhancedWord");
        assert_eq!(enhanced_word, deserialized);
    }

    #[test]
    fn test_entity_serialization() {
        let entity = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let json = serde_json::to_string(&entity).expect("Failed to serialize Entity");
        let deserialized: Entity =
            serde_json::from_str(&json).expect("Failed to deserialize Entity");
        assert_eq!(entity, deserialized);
    }

    #[test]
    fn test_state_serialization() {
        let state = State {
            predicate: "tall".to_string(),
            polarity: true,
        };

        let json = serde_json::to_string(&state).expect("Failed to serialize State");
        let deserialized: State = serde_json::from_str(&json).expect("Failed to deserialize State");
        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_action_serialization() {
        let entity = Entity {
            id: 1,
            text: "hammer".to_string(),
            animacy: Some(Animacy::Inanimate),
            definiteness: Some(Definiteness::Indefinite),
        };

        let action = Action {
            predicate: "run".to_string(),
            manner: Some("quickly".to_string()),
            instrument: Some(entity),
        };

        let json = serde_json::to_string(&action).expect("Failed to serialize Action");
        let deserialized: Action =
            serde_json::from_str(&json).expect("Failed to deserialize Action");
        assert_eq!(action, deserialized);
    }

    #[test]
    fn test_path_serialization() {
        let source = Entity {
            id: 1,
            text: "home".to_string(),
            animacy: Some(Animacy::Inanimate),
            definiteness: Some(Definiteness::Definite),
        };

        let goal = Entity {
            id: 2,
            text: "store".to_string(),
            animacy: Some(Animacy::Inanimate),
            definiteness: Some(Definiteness::Definite),
        };

        let path = Path {
            source: Some(source),
            goal: Some(goal),
            route: None,
            direction: Some(Direction::North),
        };

        let json = serde_json::to_string(&path).expect("Failed to serialize Path");
        let deserialized: Path = serde_json::from_str(&json).expect("Failed to deserialize Path");
        assert_eq!(path, deserialized);
    }

    #[test]
    fn test_proposition_serialization() {
        let proposition = Proposition {
            content: "It is raining".to_string(),
            modality: Some(Modality::Possibility),
            polarity: true,
        };

        let json = serde_json::to_string(&proposition).expect("Failed to serialize Proposition");
        let deserialized: Proposition =
            serde_json::from_str(&json).expect("Failed to deserialize Proposition");
        assert_eq!(proposition, deserialized);
    }

    #[test]
    fn test_little_v_serialization() {
        let causer = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let theme = Entity {
            id: 2,
            text: "vase".to_string(),
            animacy: Some(Animacy::Inanimate),
            definiteness: Some(Definiteness::Indefinite),
        };

        // Test Cause variant
        let cause_v = LittleV::Cause {
            causer: causer.clone(),
            caused_predicate: "break".to_string(),
            caused_theme: theme.clone(),
        };

        let json = serde_json::to_string(&cause_v).expect("Failed to serialize LittleV::Cause");
        let deserialized: LittleV =
            serde_json::from_str(&json).expect("Failed to deserialize LittleV::Cause");
        assert_eq!(cause_v, deserialized);

        // Test Become variant
        let state = State {
            predicate: "broken".to_string(),
            polarity: true,
        };

        let become_v = LittleV::Become {
            theme: theme.clone(),
            result_state: state,
        };

        let json = serde_json::to_string(&become_v).expect("Failed to serialize LittleV::Become");
        let deserialized: LittleV =
            serde_json::from_str(&json).expect("Failed to deserialize LittleV::Become");
        assert_eq!(become_v, deserialized);

        // Test Experience variant
        let experience_v = LittleV::Experience {
            experiencer: causer.clone(),
            stimulus: theme.clone(),
            psych_type: PsychType::SubjectExp,
        };

        let json =
            serde_json::to_string(&experience_v).expect("Failed to serialize LittleV::Experience");
        let deserialized: LittleV =
            serde_json::from_str(&json).expect("Failed to deserialize LittleV::Experience");
        assert_eq!(experience_v, deserialized);

        // Test Have variant
        let have_v = LittleV::Have {
            possessor: causer.clone(),
            possessee: theme.clone(),
            possession_type: PossessionType::Legal,
        };

        let json = serde_json::to_string(&have_v).expect("Failed to serialize LittleV::Have");
        let deserialized: LittleV =
            serde_json::from_str(&json).expect("Failed to deserialize LittleV::Have");
        assert_eq!(have_v, deserialized);
    }

    #[test]
    fn test_event_serialization() {
        let agent = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let theme = Entity {
            id: 2,
            text: "book".to_string(),
            animacy: Some(Animacy::Inanimate),
            definiteness: Some(Definiteness::Indefinite),
        };

        let action = Action {
            predicate: "read".to_string(),
            manner: None,
            instrument: None,
        };

        let little_v = LittleV::Do {
            agent: agent.clone(),
            action,
        };

        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, agent);
        participants.insert(ThetaRole::Theme, theme);

        let event = Event {
            id: 1,
            predicate: "read".to_string(),
            little_v,
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
        };

        let json = serde_json::to_string(&event).expect("Failed to serialize Event");
        let deserialized: Event = serde_json::from_str(&json).expect("Failed to deserialize Event");
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_enum_serialization_edge_cases() {
        // Test all AspectualClass variants
        let aspectual_classes = vec![
            AspectualClass::State,
            AspectualClass::Activity,
            AspectualClass::Accomplishment,
            AspectualClass::Achievement,
        ];

        for aspect in aspectual_classes {
            let json = serde_json::to_string(&aspect).expect("Failed to serialize AspectualClass");
            let deserialized: AspectualClass =
                serde_json::from_str(&json).expect("Failed to deserialize AspectualClass");
            assert_eq!(aspect, deserialized);
        }

        // Test all Voice variants
        let voices = vec![Voice::Active, Voice::Passive, Voice::Middle];
        for voice in voices {
            let json = serde_json::to_string(&voice).expect("Failed to serialize Voice");
            let deserialized: Voice =
                serde_json::from_str(&json).expect("Failed to deserialize Voice");
            assert_eq!(voice, deserialized);
        }

        // Test all Direction variants
        let directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::Forward,
            Direction::Backward,
            Direction::Into,
            Direction::OutOf,
            Direction::Through,
            Direction::Around,
        ];

        for direction in directions {
            let json = serde_json::to_string(&direction).expect("Failed to serialize Direction");
            let deserialized: Direction =
                serde_json::from_str(&json).expect("Failed to deserialize Direction");
            assert_eq!(direction, deserialized);
        }

        // Test all Modality variants
        let modalities = vec![
            Modality::Necessity,
            Modality::Possibility,
            Modality::Obligation,
            Modality::Desire,
        ];

        for modality in modalities {
            let json = serde_json::to_string(&modality).expect("Failed to serialize Modality");
            let deserialized: Modality =
                serde_json::from_str(&json).expect("Failed to deserialize Modality");
            assert_eq!(modality, deserialized);
        }

        // Test all Animacy variants
        let animacies = vec![
            Animacy::Human,
            Animacy::Animal,
            Animacy::Plant,
            Animacy::Inanimate,
        ];

        for animacy in animacies {
            let json = serde_json::to_string(&animacy).expect("Failed to serialize Animacy");
            let deserialized: Animacy =
                serde_json::from_str(&json).expect("Failed to deserialize Animacy");
            assert_eq!(animacy, deserialized);
        }

        // Test all PsychType variants
        let psych_types = vec![
            PsychType::SubjectExp,
            PsychType::ObjectExp,
            PsychType::PsychState,
        ];

        for psych_type in psych_types {
            let json = serde_json::to_string(&psych_type).expect("Failed to serialize PsychType");
            let deserialized: PsychType =
                serde_json::from_str(&json).expect("Failed to deserialize PsychType");
            assert_eq!(psych_type, deserialized);
        }

        // Test all PossessionType variants
        let possession_types = vec![
            PossessionType::Legal,
            PossessionType::Temporary,
            PossessionType::Kinship,
            PossessionType::PartWhole,
        ];

        for possession_type in possession_types {
            let json = serde_json::to_string(&possession_type)
                .expect("Failed to serialize PossessionType");
            let deserialized: PossessionType =
                serde_json::from_str(&json).expect("Failed to deserialize PossessionType");
            assert_eq!(possession_type, deserialized);
        }
    }

    #[test]
    fn test_serialization_with_unicode_and_special_characters() {
        // Test Word with unicode characters
        let word = Word {
            id: 1,
            text: "café".to_string(),
            lemma: "café".to_string(),
            upos: UPos::Noun,
            xpos: Some("NN".to_string()),
            feats: MorphFeatures::default(),
            head: None,
            deprel: DepRel::Root,
            deps: None,
            misc: Some("unicode=true".to_string()),
            start: 0,
            end: 4,
        };

        let json = serde_json::to_string(&word).expect("Failed to serialize Word with unicode");
        let deserialized: Word =
            serde_json::from_str(&json).expect("Failed to deserialize Word with unicode");
        assert_eq!(word, deserialized);

        // Test Entity with emoji
        let entity = Entity {
            id: 1,
            text: "🏃‍♂️".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let json = serde_json::to_string(&entity).expect("Failed to serialize Entity with emoji");
        let deserialized: Entity =
            serde_json::from_str(&json).expect("Failed to deserialize Entity with emoji");
        assert_eq!(entity, deserialized);

        // Test with special characters
        let entity_special = Entity {
            id: 2,
            text: "\"quoted\" & 'apostrophe' <tag>".to_string(),
            animacy: Some(Animacy::Inanimate),
            definiteness: Some(Definiteness::Indefinite),
        };

        let json = serde_json::to_string(&entity_special)
            .expect("Failed to serialize Entity with special chars");
        let deserialized: Entity =
            serde_json::from_str(&json).expect("Failed to deserialize Entity with special chars");
        assert_eq!(entity_special, deserialized);
    }

    #[test]
    fn test_nested_structure_serialization() {
        // Create a complex nested structure with multiple levels
        let speaker = Entity {
            id: 1,
            text: "Mary".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let addressee = Entity {
            id: 2,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let proposition = Proposition {
            content: "It will rain tomorrow".to_string(),
            modality: Some(Modality::Possibility),
            polarity: true,
        };

        let say_v = LittleV::Say {
            speaker,
            addressee: Some(addressee),
            content: proposition,
        };

        let json =
            serde_json::to_string(&say_v).expect("Failed to serialize complex nested LittleV");
        let deserialized: LittleV =
            serde_json::from_str(&json).expect("Failed to deserialize complex nested LittleV");
        assert_eq!(say_v, deserialized);
    }

    #[test]
    fn test_large_data_structure_serialization() {
        // Test serialization with large amounts of data
        let mut words = Vec::new();
        for i in 1..=1000 {
            words.push(Word {
                id: i,
                text: format!("word{}", i),
                lemma: format!("lemma{}", i),
                upos: UPos::Noun,
                xpos: None,
                feats: MorphFeatures::default(),
                head: if i > 1 { Some(i - 1) } else { None },
                deprel: DepRel::Dep,
                deps: None,
                misc: None,
                start: (i - 1) * 5,
                end: i * 5,
            });
        }

        let sentence = Sentence::new(words);
        let document = Document::new("Large document with many words".to_string(), vec![sentence]);

        let json = serde_json::to_string(&document).expect("Failed to serialize large Document");
        let deserialized: Document =
            serde_json::from_str(&json).expect("Failed to deserialize large Document");
        assert_eq!(document, deserialized);
        assert_eq!(deserialized.total_word_count(), 1000);
    }

    #[test]
    fn test_serialization_format_stability() {
        // Test that serialization format is stable (important for API compatibility)
        let word = Word::new(1, "test".to_string(), 0, 4);
        let json = serde_json::to_string(&word).expect("Failed to serialize Word");

        // Check that essential fields are present in JSON
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"text\":\"test\""));
        assert!(json.contains("\"start\":0"));
        assert!(json.contains("\"end\":4"));

        // Test that deserialization works with the expected format
        let manual_json = r#"{"id":1,"text":"test","lemma":"test","upos":"X","xpos":null,"feats":{"person":null,"number":null,"gender":null,"animacy":null,"case":null,"definiteness":null,"tense":null,"aspect":null,"mood":null,"voice":null,"degree":null,"verbform":null,"raw_features":null},"head":null,"deprel":"Dep","deps":null,"misc":null,"start":0,"end":4}"#;
        let deserialized: Word =
            serde_json::from_str(manual_json).expect("Failed to deserialize manual JSON");
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.text, "test");
    }

    #[test]
    fn test_default_values_serialization() {
        // Test that default values serialize/deserialize correctly
        let default_morph = MorphFeatures::default();
        let json = serde_json::to_string(&default_morph)
            .expect("Failed to serialize default MorphFeatures");
        let deserialized: MorphFeatures =
            serde_json::from_str(&json).expect("Failed to deserialize default MorphFeatures");
        assert_eq!(default_morph, deserialized);
        assert!(deserialized.person.is_none());
        assert!(deserialized.number.is_none());

        let default_semantic = SemanticFeatures::default();
        let json = serde_json::to_string(&default_semantic)
            .expect("Failed to serialize default SemanticFeatures");
        let deserialized: SemanticFeatures =
            serde_json::from_str(&json).expect("Failed to deserialize default SemanticFeatures");
        assert_eq!(default_semantic, deserialized);
        assert!(deserialized.animacy.is_none());

        let default_confidence = FeatureConfidence::default();
        let json = serde_json::to_string(&default_confidence)
            .expect("Failed to serialize default FeatureConfidence");
        let deserialized: FeatureConfidence =
            serde_json::from_str(&json).expect("Failed to deserialize default FeatureConfidence");
        assert_eq!(default_confidence, deserialized);
        assert_eq!(deserialized.animacy, 0.0);
    }
}
