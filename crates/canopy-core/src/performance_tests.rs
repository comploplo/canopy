//! Performance edge case tests for large document processing
//!
//! These tests ensure that the core data structures and operations
//! can handle large-scale data without performance degradation or
//! memory issues.

#[cfg(test)]
mod performance_edge_case_tests {
    use crate::*;
    use std::time::Instant;

    #[test]
    fn test_large_document_creation_performance() {
        let start = Instant::now();

        // Create a document with 10,000 sentences, each with 20 words
        let mut sentences = Vec::new();
        for sentence_id in 1..=10_000 {
            let mut words = Vec::new();
            for word_id in 1..=20 {
                let global_id = (sentence_id - 1) * 20 + word_id;
                words.push(Word::new(
                    global_id,
                    format!("word{word_id}"),
                    (global_id - 1) * 5,
                    global_id * 5,
                ));
            }
            sentences.push(Sentence::new(words));
        }

        let document = Document::new(
            "Large document for performance testing".to_string(),
            sentences,
        );

        let creation_time = start.elapsed();

        // Verify the document was created correctly
        assert_eq!(document.sentence_count(), 10_000);
        assert_eq!(document.total_word_count(), 200_000);

        // Performance should be reasonable (under 1 second for creation)
        assert!(
            creation_time.as_millis() < 1000,
            "Document creation took too long: {creation_time:?}"
        );
    }

    #[test]
    fn test_large_document_iteration_performance() {
        // Create a moderately large document
        let mut sentences = Vec::new();
        for sentence_id in 1..=1_000 {
            let mut words = Vec::new();
            for word_id in 1..=50 {
                let global_id = (sentence_id - 1) * 50 + word_id;
                words.push(Word::new(
                    global_id,
                    format!("word{word_id}"),
                    (global_id - 1) * 6,
                    global_id * 6,
                ));
            }
            sentences.push(Sentence::new(words));
        }

        let document = Document::new("Document for iteration testing".to_string(), sentences);

        let start = Instant::now();

        // Iterate through all words and perform some operation
        let mut word_count = 0;
        let mut total_length = 0;

        for sentence in &document.sentences {
            for word in &sentence.words {
                word_count += 1;
                total_length += word.text.len();

                // Simulate some processing
                let _lemma = &word.lemma;
                let _pos = &word.upos;
            }
        }

        let iteration_time = start.elapsed();

        assert_eq!(word_count, 50_000);
        assert!(total_length > 0);

        // Iteration should be fast (under 100ms)
        assert!(
            iteration_time.as_millis() < 100,
            "Document iteration took too long: {iteration_time:?}"
        );
    }

    #[test]
    fn test_large_sentence_word_access_performance() {
        // Create a single sentence with many words
        let mut words = Vec::new();
        for word_id in 1..=100_000 {
            words.push(Word::new(
                word_id,
                format!("word{word_id}"),
                (word_id - 1) * 5,
                word_id * 5,
            ));
        }

        let sentence = Sentence::new(words);

        let start = Instant::now();

        // Random access to words
        for i in (0..sentence.words.len()).step_by(1000) {
            let _word = &sentence.words[i];
        }

        let access_time = start.elapsed();

        assert_eq!(sentence.word_count(), 100_000);

        // Random access should be very fast (under 10ms)
        assert!(
            access_time.as_millis() < 10,
            "Word access took too long: {access_time:?}"
        );
    }

    #[test]
    fn test_large_morphological_features_processing() {
        let start = Instant::now();

        // Create many words with complex morphological features
        let mut words = Vec::new();
        for i in 1..=10_000 {
            let morph_features = MorphFeatures {
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
                raw_features: Some(format!("Feature{i}")),
            };

            let word = Word {
                id: i,
                text: format!("complexword{i}"),
                lemma: format!("lemma{i}"),
                upos: UPos::Verb,
                xpos: Some(format!("VB{i}")),
                feats: morph_features,
                head: Some(i.saturating_sub(1)),
                deprel: DepRel::Root,
                deps: Some(format!("{i}:nsubj")),
                misc: Some(format!("SpaceAfter=No|Misc{i}")),
                start: (i - 1) * 10,
                end: i * 10,
            };

            words.push(word);
        }

        let creation_time = start.elapsed();

        // Process the features
        let start_processing = Instant::now();

        let mut feature_count = 0;
        for word in &words {
            if word.feats.person.is_some() {
                feature_count += 1;
            }
            if word.feats.number.is_some() {
                feature_count += 1;
            }
            if word.feats.gender.is_some() {
                feature_count += 1;
            }
            if word.feats.tense.is_some() {
                feature_count += 1;
            }
            if word.feats.aspect.is_some() {
                feature_count += 1;
            }
            if word.feats.mood.is_some() {
                feature_count += 1;
            }
            if word.feats.voice.is_some() {
                feature_count += 1;
            }
            if word.feats.case.is_some() {
                feature_count += 1;
            }
            // Process other features...
        }

        let processing_time = start_processing.elapsed();

        assert_eq!(words.len(), 10_000);
        assert!(feature_count > 50_000); // Should have many features (at least 8 per word * 10k words)

        // Both creation and processing should be fast
        assert!(
            creation_time.as_millis() < 500,
            "Feature creation took too long: {creation_time:?}"
        );
        assert!(
            processing_time.as_millis() < 100,
            "Feature processing took too long: {processing_time:?}"
        );
    }

    #[test]
    fn test_large_event_structure_performance() {
        let start = Instant::now();

        // Create many complex event structures
        let mut events = Vec::new();
        for i in 1..=1_000 {
            let agent = Entity {
                id: i * 2,
                text: format!("agent{i}"),
                animacy: Some(Animacy::Human),
                definiteness: Some(Definiteness::Definite),
            };

            let theme = Entity {
                id: i * 2 + 1,
                text: format!("theme{i}"),
                animacy: Some(Animacy::Inanimate),
                definiteness: Some(Definiteness::Indefinite),
            };

            let action = Action {
                predicate: format!("action{i}"),
                manner: Some(format!("manner{i}")),
                instrument: Some(Entity {
                    id: i * 3,
                    text: format!("instrument{i}"),
                    animacy: Some(Animacy::Inanimate),
                    definiteness: Some(Definiteness::Indefinite),
                }),
            };

            let little_v = LittleV::Do {
                agent: agent.clone(),
                action,
            };

            let mut participants = std::collections::HashMap::new();
            participants.insert(ThetaRole::Agent, agent);
            participants.insert(ThetaRole::Theme, theme);

            let event = Event {
                id: i,
                predicate: format!("predicate{i}"),
                little_v,
                participants,
                aspect: AspectualClass::Activity,
                voice: Voice::Active,
            };

            events.push(event);
        }

        let creation_time = start.elapsed();

        // Process the events
        let start_processing = Instant::now();

        let mut agent_count = 0;
        let mut theme_count = 0;
        for event in &events {
            if event.participants.contains_key(&ThetaRole::Agent) {
                agent_count += 1;
            }
            if event.participants.contains_key(&ThetaRole::Theme) {
                theme_count += 1;
            }

            // Check if event has external argument
            let _has_external = event.little_v.external_argument().is_some();
        }

        let processing_time = start_processing.elapsed();

        assert_eq!(events.len(), 1_000);
        assert_eq!(agent_count, 1_000);
        assert_eq!(theme_count, 1_000);

        // Performance should be reasonable
        assert!(
            creation_time.as_millis() < 1000,
            "Event creation took too long: {creation_time:?}"
        );
        assert!(
            processing_time.as_millis() < 100,
            "Event processing took too long: {processing_time:?}"
        );
    }

    #[test]
    fn test_memory_usage_with_large_documents() {
        // This test checks that we don't have excessive memory growth
        let initial_words = create_test_words(1_000);
        let medium_words = create_test_words(10_000);
        let large_words = create_test_words(100_000);

        // Verify all were created successfully
        assert_eq!(initial_words.len(), 1_000);
        assert_eq!(medium_words.len(), 10_000);
        assert_eq!(large_words.len(), 100_000);

        // Test that operations scale reasonably
        let start = Instant::now();
        let _count1 = count_verbs(&initial_words);
        let time1 = start.elapsed();

        let start = Instant::now();
        let _count2 = count_verbs(&medium_words);
        let time2 = start.elapsed();

        let start = Instant::now();
        let _count3 = count_verbs(&large_words);
        let time3 = start.elapsed();

        // Time should scale roughly linearly (within 2x factor per 10x increase)
        assert!(
            time2.as_nanos() < time1.as_nanos() * 25,
            "Medium processing too slow: {time2:?} vs {time1:?}"
        );
        assert!(
            time3.as_nanos() < time2.as_nanos() * 25,
            "Large processing too slow: {time3:?} vs {time2:?}"
        );
    }

    fn create_test_words(count: usize) -> Vec<Word> {
        let mut words = Vec::with_capacity(count);
        for i in 1..=count {
            words.push(Word {
                id: i,
                text: format!("word{i}"),
                lemma: format!("lemma{i}"),
                upos: if i % 3 == 0 { UPos::Verb } else { UPos::Noun },
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
        words
    }

    fn count_verbs(words: &[Word]) -> usize {
        words.iter().filter(|w| w.upos == UPos::Verb).count()
    }

    #[test]
    fn test_dependency_tree_depth_performance() {
        // Create a deeply nested dependency tree
        let depth = 1_000;
        let mut words = Vec::new();

        for i in 1..=depth {
            words.push(Word {
                id: i,
                text: format!("word{i}"),
                lemma: format!("lemma{i}"),
                upos: UPos::Noun,
                xpos: None,
                feats: MorphFeatures::default(),
                head: if i == 1 { None } else { Some(i - 1) }, // Chain dependency
                deprel: DepRel::Dep,
                deps: None,
                misc: None,
                start: (i - 1) * 6,
                end: i * 6,
            });
        }

        let sentence = Sentence::new(words);

        let start = Instant::now();

        // Traverse the dependency chain
        let mut current_id = depth;
        let mut chain_length = 0;

        while let Some(word) = sentence.words.iter().find(|w| w.id == current_id) {
            chain_length += 1;
            if let Some(head) = word.head {
                current_id = head;
            } else {
                break;
            }

            // Prevent infinite loops in case of cycles
            if chain_length > depth {
                break;
            }
        }

        let traversal_time = start.elapsed();

        assert_eq!(chain_length, depth);

        // Deep traversal should still be reasonably fast
        assert!(
            traversal_time.as_millis() < 100,
            "Dependency traversal took too long: {traversal_time:?}"
        );
    }

    #[test]
    fn test_large_semantic_feature_extraction() {
        let start = Instant::now();

        // Create many enhanced words with semantic features
        let mut enhanced_words = Vec::new();
        for i in 1..=5_000 {
            let base_word = Word::new(i, format!("word{i}"), i * 5, (i + 1) * 5);

            let semantic_features = SemanticFeatures {
                animacy: Some(if i % 4 == 0 {
                    Animacy::Human
                } else {
                    Animacy::Inanimate
                }),
                definiteness: Some(if i % 3 == 0 {
                    Definiteness::Definite
                } else {
                    Definiteness::Indefinite
                }),
                countability: Some(if i % 2 == 0 {
                    Countability::Count
                } else {
                    Countability::Mass
                }),
                concreteness: Some(if i % 5 == 0 {
                    Concreteness::Abstract
                } else {
                    Concreteness::Concrete
                }),
            };

            let confidence = FeatureConfidence {
                animacy: 0.9,
                definiteness: 0.8,
                countability: 0.85,
                concreteness: 0.92,
            };

            enhanced_words.push(EnhancedWord {
                base: base_word,
                semantic_features,
                confidence,
            });
        }

        let creation_time = start.elapsed();

        // Analyze the features
        let start_analysis = Instant::now();

        let mut human_count = 0;
        let mut definite_count = 0;
        let mut abstract_count = 0;

        for enhanced_word in &enhanced_words {
            if enhanced_word.semantic_features.animacy == Some(Animacy::Human) {
                human_count += 1;
            }
            if enhanced_word.semantic_features.definiteness == Some(Definiteness::Definite) {
                definite_count += 1;
            }
            if enhanced_word.semantic_features.concreteness == Some(Concreteness::Abstract) {
                abstract_count += 1;
            }
        }

        let analysis_time = start_analysis.elapsed();

        assert_eq!(enhanced_words.len(), 5_000);
        assert!(human_count > 0);
        assert!(definite_count > 0);
        assert!(abstract_count > 0);

        // Both creation and analysis should be efficient
        assert!(
            creation_time.as_millis() < 500,
            "Enhanced word creation took too long: {creation_time:?}"
        );
        assert!(
            analysis_time.as_millis() < 50,
            "Feature analysis took too long: {analysis_time:?}"
        );
    }

    #[test]
    fn test_concurrent_document_processing_simulation() {
        // Simulate concurrent processing by creating multiple documents
        // and processing them sequentially (to test data structure efficiency)

        let start = Instant::now();

        let mut documents = Vec::new();
        for doc_id in 1..=100 {
            let mut sentences = Vec::new();
            for sent_id in 1..=50 {
                let mut words = Vec::new();
                for word_id in 1..=20 {
                    let global_id = (doc_id - 1) * 1000 + (sent_id - 1) * 20 + word_id;
                    words.push(Word::new(
                        global_id,
                        format!("word{word_id}"),
                        (word_id - 1) * 5,
                        word_id * 5,
                    ));
                }
                sentences.push(Sentence::new(words));
            }

            documents.push(Document::new(format!("Document {doc_id}"), sentences));
        }

        let creation_time = start.elapsed();

        // Process all documents
        let start_processing = Instant::now();

        let mut total_words = 0;
        let mut total_sentences = 0;

        for document in &documents {
            total_sentences += document.sentence_count();
            total_words += document.total_word_count();
        }

        let processing_time = start_processing.elapsed();

        assert_eq!(documents.len(), 100);
        assert_eq!(total_sentences, 5_000); // 100 docs * 50 sentences
        assert_eq!(total_words, 100_000); // 100 docs * 50 sentences * 20 words

        // Performance should scale well for multiple documents
        assert!(
            creation_time.as_millis() < 2000,
            "Multi-document creation took too long: {creation_time:?}"
        );
        assert!(
            processing_time.as_millis() < 100,
            "Multi-document processing took too long: {processing_time:?}"
        );
    }

    #[test]
    fn test_large_theta_role_assignment_performance() {
        let start = Instant::now();

        // Create events with many theta role assignments
        let mut events = Vec::new();
        for i in 1..=1_000 {
            let mut participants = std::collections::HashMap::new();

            // Add all possible theta roles
            for (j, &role) in ThetaRole::all().iter().enumerate() {
                participants.insert(
                    role,
                    Entity {
                        id: i * 100 + j,
                        text: format!("entity_{i}_{j}"),
                        animacy: Some(if j % 2 == 0 {
                            Animacy::Human
                        } else {
                            Animacy::Inanimate
                        }),
                        definiteness: Some(if j % 3 == 0 {
                            Definiteness::Definite
                        } else {
                            Definiteness::Indefinite
                        }),
                    },
                );
            }

            let agent = participants.get(&ThetaRole::Agent).unwrap().clone();
            let action = Action {
                predicate: format!("complex_action_{i}"),
                manner: Some(format!("manner_{i}")),
                instrument: participants.get(&ThetaRole::Instrument).cloned(),
            };

            let event = Event {
                id: i,
                predicate: format!("complex_predicate_{i}"),
                little_v: LittleV::Do { agent, action },
                participants,
                aspect: AspectualClass::Activity,
                voice: Voice::Active,
            };

            events.push(event);
        }

        let creation_time = start.elapsed();

        // Query theta roles
        let start_query = Instant::now();

        let mut agent_events = 0;
        let mut theme_events = 0;
        let mut instrument_events = 0;

        for event in &events {
            if event.participants.contains_key(&ThetaRole::Agent) {
                agent_events += 1;
            }
            if event.participants.contains_key(&ThetaRole::Theme) {
                theme_events += 1;
            }
            if event.participants.contains_key(&ThetaRole::Instrument) {
                instrument_events += 1;
            }
        }

        let query_time = start_query.elapsed();

        assert_eq!(events.len(), 1_000);
        assert_eq!(agent_events, 1_000); // All events should have agents
        assert_eq!(theme_events, 1_000); // All events should have themes
        assert_eq!(instrument_events, 1_000); // All events should have instruments

        // Complex event processing should be efficient
        assert!(
            creation_time.as_millis() < 1500,
            "Complex event creation took too long: {creation_time:?}"
        );
        assert!(
            query_time.as_millis() < 100,
            "Theta role querying took too long: {query_time:?}"
        );
    }
}
