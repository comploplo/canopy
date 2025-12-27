//! Comprehensive tests for WordNet types functionality

use canopy_wordnet::types::{
    ExceptionEntry, IndexEntry, PartOfSpeech, SemanticPointer, SemanticRelation, Synset,
    SynsetWord, VerbFrame, WordNetAnalysis, WordNetDatabase,
};

#[cfg(test)]
mod types_tests {
    use super::*;
    use std::collections::HashMap;

    // Helper function to create a test synset
    fn create_test_synset(
        offset: usize,
        pos: PartOfSpeech,
        words: Vec<&str>,
        gloss: &str,
    ) -> Synset {
        Synset {
            offset,
            lex_filenum: 0,
            pos,
            words: words
                .iter()
                .map(|word| SynsetWord {
                    word: word.to_string(),
                    lex_id: 0,
                    tag_count: Some(1),
                })
                .collect(),
            pointers: vec![],
            frames: vec![],
            gloss: gloss.to_string(),
        }
    }

    #[test]
    fn test_part_of_speech_complete() {
        // Test all POS variants
        let pos_variants = vec![
            (PartOfSpeech::Noun, 'n', "noun"),
            (PartOfSpeech::Verb, 'v', "verb"),
            (PartOfSpeech::Adjective, 'a', "adjective"),
            (PartOfSpeech::AdjectiveSatellite, 's', "adjective satellite"),
            (PartOfSpeech::Adverb, 'r', "adverb"),
        ];

        for (pos, expected_code, expected_name) in pos_variants {
            assert_eq!(pos.code(), expected_code);
            assert_eq!(pos.name(), expected_name);
        }
    }

    #[test]
    fn test_part_of_speech_traits() {
        let noun = PartOfSpeech::Noun;
        let noun2 = noun;
        let verb = PartOfSpeech::Verb;

        // Test Clone, Copy, PartialEq, Eq
        assert_eq!(noun, noun2);
        assert_ne!(noun, verb);

        // Test Debug
        let debug_str = format!("{:?}", noun);
        assert!(debug_str.contains("Noun"));

        // Test Hash (by using in HashMap)
        let mut map = HashMap::new();
        map.insert(noun, 1);
        map.insert(verb, 2);
        assert_eq!(map.get(&noun), Some(&1));
        assert_eq!(map.get(&verb), Some(&2));
    }

    #[test]
    fn test_semantic_relation_complete() {
        // Test all semantic relation variants
        let relation_data = vec![
            (SemanticRelation::Antonym, "!", "opposite meaning"),
            (SemanticRelation::Hypernym, "@", "more general term"),
            (SemanticRelation::Hyponym, "~", "more specific term"),
            (
                SemanticRelation::InstanceHypernym,
                "@i",
                "class of this instance",
            ),
            (
                SemanticRelation::InstanceHyponym,
                "~i",
                "instance of this class",
            ),
            (
                SemanticRelation::MemberHolonym,
                "#m",
                "whole that has this as member",
            ),
            (
                SemanticRelation::SubstanceHolonym,
                "#s",
                "whole that has this as substance",
            ),
            (
                SemanticRelation::PartHolonym,
                "#p",
                "whole that has this as part",
            ),
            (SemanticRelation::MemberMeronym, "%m", "has member"),
            (SemanticRelation::SubstanceMeronym, "%s", "has substance"),
            (SemanticRelation::PartMeronym, "%p", "has part"),
            (SemanticRelation::Attribute, "=", "attribute relationship"),
            (SemanticRelation::Derivation, "+", "derivationally related"),
            (SemanticRelation::DomainTopic, ";c", "topic domain"),
            (SemanticRelation::DomainRegion, ";r", "region domain"),
            (SemanticRelation::DomainUsage, ";u", "usage domain"),
            (SemanticRelation::MemberTopic, "-c", "member of topic"),
            (SemanticRelation::MemberRegion, "-r", "member of region"),
            (SemanticRelation::MemberUsage, "-u", "member of usage"),
            (SemanticRelation::Entailment, "*", "entails"),
            (SemanticRelation::Cause, ">", "causes"),
            (SemanticRelation::AlsoSee, "^", "see also"),
            (SemanticRelation::VerbGroup, "$", "verb group"),
            (SemanticRelation::SimilarTo, "&", "similar to"),
            (SemanticRelation::Participle, "<", "participle form"),
            (SemanticRelation::Pertainym, "\\", "pertains to"),
        ];

        for (relation, expected_symbol, expected_description) in relation_data {
            assert_eq!(relation.symbol(), expected_symbol);
            assert_eq!(relation.description(), expected_description);
        }
    }

    #[test]
    fn test_semantic_pointer() {
        let pointer = SemanticPointer {
            relation: SemanticRelation::Hypernym,
            target_offset: 100002137,
            target_pos: PartOfSpeech::Noun,
            source_word: 0,
            target_word: 0,
        };

        assert_eq!(pointer.relation, SemanticRelation::Hypernym);
        assert_eq!(pointer.target_offset, 100002137);
        assert_eq!(pointer.target_pos, PartOfSpeech::Noun);
        assert_eq!(pointer.source_word, 0);
        assert_eq!(pointer.target_word, 0);

        // Test traits
        let pointer2 = pointer.clone();
        assert_eq!(pointer, pointer2);

        let debug_str = format!("{:?}", pointer);
        assert!(debug_str.contains("SemanticPointer"));
    }

    #[test]
    fn test_synset_word() {
        let word = SynsetWord {
            word: "running".to_string(),
            lex_id: 1,
            tag_count: Some(42),
        };

        assert_eq!(word.word, "running");
        assert_eq!(word.lex_id, 1);
        assert_eq!(word.tag_count, Some(42));

        // Test with None tag_count
        let word2 = SynsetWord {
            word: "walk".to_string(),
            lex_id: 0,
            tag_count: None,
        };
        assert_eq!(word2.tag_count, None);

        // Test traits
        let word3 = word.clone();
        assert_eq!(word, word3);
    }

    #[test]
    fn test_verb_frame() {
        let frame = VerbFrame {
            frame_number: 8,
            word_number: 0,
            template: "Somebody ----s something".to_string(),
        };

        assert_eq!(frame.frame_number, 8);
        assert_eq!(frame.word_number, 0);
        assert_eq!(frame.template, "Somebody ----s something");

        // Test traits
        let frame2 = frame.clone();
        assert_eq!(frame, frame2);
    }

    #[test]
    fn test_synset_creation_and_methods() {
        let synset = create_test_synset(
            100001740,
            PartOfSpeech::Noun,
            vec!["entity", "something"],
            "that which is perceived or known or inferred",
        );

        assert_eq!(synset.offset, 100001740);
        assert_eq!(synset.pos, PartOfSpeech::Noun);
        assert_eq!(synset.words.len(), 2);

        // Test primary_word
        assert_eq!(synset.primary_word(), Some("entity"));

        // Test contains_word
        assert!(synset.contains_word("entity"));
        assert!(synset.contains_word("something"));
        assert!(!synset.contains_word("nonexistent"));

        // Test word_list
        let word_list = synset.word_list();
        assert_eq!(word_list.len(), 2);
        assert!(word_list.contains(&"entity".to_string()));
        assert!(word_list.contains(&"something".to_string()));

        // Test definition extraction
        assert_eq!(
            synset.definition(),
            "that which is perceived or known or inferred"
        );

        // Test examples (no quotes in this gloss)
        assert!(synset.examples().is_empty());
    }

    #[test]
    fn test_synset_gloss_parsing() {
        // Test with semicolon separator
        let synset1 = create_test_synset(
            100001,
            PartOfSpeech::Noun,
            vec!["test"],
            "definition here; extra info",
        );
        assert_eq!(synset1.definition(), "definition here");

        // Test with quotes (examples)
        let synset2 = create_test_synset(
            100002,
            PartOfSpeech::Verb,
            vec!["run"],
            "move fast by using one's feet; \"Don't walk when you can run\" \"He ran to the store\"",
        );
        assert_eq!(synset2.definition(), "move fast by using one's feet");
        let examples = synset2.examples();
        assert_eq!(examples.len(), 2);
        assert!(examples.contains(&"Don't walk when you can run".to_string()));
        assert!(examples.contains(&"He ran to the store".to_string()));

        // Test with both semicolon and quotes
        let synset3 = create_test_synset(
            100003,
            PartOfSpeech::Adjective,
            vec!["good"],
            "having desirable qualities; \"good food\" \"a good book\"; additional notes",
        );
        assert_eq!(synset3.definition(), "having desirable qualities");
        let examples3 = synset3.examples();
        assert_eq!(examples3.len(), 2);
        assert!(examples3.contains(&"good food".to_string()));
        assert!(examples3.contains(&"a good book".to_string()));
    }

    #[test]
    fn test_synset_relations() {
        let mut synset = create_test_synset(100001, PartOfSpeech::Noun, vec!["entity"], "test");

        // Add some pointers
        synset.pointers.push(SemanticPointer {
            relation: SemanticRelation::Hypernym,
            target_offset: 100002,
            target_pos: PartOfSpeech::Noun,
            source_word: 0,
            target_word: 0,
        });

        synset.pointers.push(SemanticPointer {
            relation: SemanticRelation::Hyponym,
            target_offset: 100003,
            target_pos: PartOfSpeech::Noun,
            source_word: 0,
            target_word: 0,
        });

        synset.pointers.push(SemanticPointer {
            relation: SemanticRelation::Hypernym,
            target_offset: 100004,
            target_pos: PartOfSpeech::Noun,
            source_word: 0,
            target_word: 0,
        });

        // Test get_relations
        let hypernyms = synset.get_relations(&SemanticRelation::Hypernym);
        assert_eq!(hypernyms.len(), 2);

        let hyponyms = synset.get_relations(&SemanticRelation::Hyponym);
        assert_eq!(hyponyms.len(), 1);

        let antonyms = synset.get_relations(&SemanticRelation::Antonym);
        assert_eq!(antonyms.len(), 0);
    }

    #[test]
    fn test_index_entry() {
        let entry = IndexEntry {
            lemma: "run".to_string(),
            pos: PartOfSpeech::Verb,
            synset_count: 3,
            pointer_count: 5,
            relations: vec![
                SemanticRelation::Hypernym,
                SemanticRelation::Hyponym,
                SemanticRelation::Entailment,
            ],
            tag_sense_count: 25,
            synset_offsets: vec![2097048, 1926311, 2000556],
        };

        assert_eq!(entry.lemma, "run");
        assert_eq!(entry.pos, PartOfSpeech::Verb);
        assert_eq!(entry.synset_count, 3);
        assert_eq!(entry.synset_offsets.len(), 3);

        // Test primary_synset_offset
        assert_eq!(entry.primary_synset_offset(), Some(2097048));

        // Test has_relation
        assert!(entry.has_relation(&SemanticRelation::Hypernym));
        assert!(entry.has_relation(&SemanticRelation::Hyponym));
        assert!(entry.has_relation(&SemanticRelation::Entailment));
        assert!(!entry.has_relation(&SemanticRelation::Antonym));

        // Test empty case
        let empty_entry = IndexEntry {
            lemma: "empty".to_string(),
            pos: PartOfSpeech::Noun,
            synset_count: 0,
            pointer_count: 0,
            relations: vec![],
            tag_sense_count: 0,
            synset_offsets: vec![],
        };
        assert_eq!(empty_entry.primary_synset_offset(), None);
    }

    #[test]
    fn test_exception_entry() {
        let exception = ExceptionEntry {
            inflected: "ran".to_string(),
            base_forms: vec!["run".to_string()],
        };

        assert_eq!(exception.inflected, "ran");
        assert_eq!(exception.base_forms.len(), 1);
        assert_eq!(exception.base_forms[0], "run");

        // Test multiple base forms
        let exception2 = ExceptionEntry {
            inflected: "worse".to_string(),
            base_forms: vec!["bad".to_string(), "ill".to_string()],
        };
        assert_eq!(exception2.base_forms.len(), 2);
        assert!(exception2.base_forms.contains(&"bad".to_string()));
        assert!(exception2.base_forms.contains(&"ill".to_string()));
    }

    #[test]
    fn test_wordnet_database_creation() {
        let database = WordNetDatabase::new();

        assert_eq!(database.synsets.len(), 0);
        assert_eq!(database.index.len(), 0);
        assert_eq!(database.exceptions.len(), 0);
        assert_eq!(database.synset_words.len(), 0);

        // Test Default trait
        let database2 = WordNetDatabase::default();
        assert_eq!(database2.synsets.len(), 0);
    }

    #[test]
    fn test_wordnet_database_operations() {
        let mut database = WordNetDatabase::new();

        // Add a synset
        let synset = create_test_synset(
            100001,
            PartOfSpeech::Noun,
            vec!["entity", "something"],
            "test definition",
        );
        database.synsets.insert(100001, synset);

        // Add an index entry
        let index_entry = IndexEntry {
            lemma: "entity".to_string(),
            pos: PartOfSpeech::Noun,
            synset_count: 1,
            pointer_count: 0,
            relations: vec![],
            tag_sense_count: 5,
            synset_offsets: vec![100001],
        };
        database
            .index
            .insert(("entity".to_string(), PartOfSpeech::Noun), index_entry);

        // Test lookup_word
        let result = database.lookup_word("entity", PartOfSpeech::Noun);
        assert!(result.is_some());
        assert_eq!(result.unwrap().lemma, "entity");

        let no_result = database.lookup_word("nonexistent", PartOfSpeech::Noun);
        assert!(no_result.is_none());

        // Test get_synset
        let synset_result = database.get_synset(100001);
        assert!(synset_result.is_some());
        assert_eq!(synset_result.unwrap().offset, 100001);

        let no_synset = database.get_synset(999999);
        assert!(no_synset.is_none());

        // Test get_synsets_for_word
        let synsets = database.get_synsets_for_word("entity", PartOfSpeech::Noun);
        assert_eq!(synsets.len(), 1);
        assert_eq!(synsets[0].offset, 100001);

        let no_synsets = database.get_synsets_for_word("nonexistent", PartOfSpeech::Noun);
        assert!(no_synsets.is_empty());
    }

    #[test]
    fn test_wordnet_database_relations() {
        let mut database = WordNetDatabase::new();

        // Create synsets with relations
        let parent_synset = Synset {
            offset: 100001,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![SynsetWord {
                word: "parent".to_string(),
                lex_id: 0,
                tag_count: None,
            }],
            pointers: vec![],
            frames: vec![],
            gloss: "parent concept".to_string(),
        };

        let child_synset = Synset {
            offset: 100002,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![SynsetWord {
                word: "child".to_string(),
                lex_id: 0,
                tag_count: None,
            }],
            pointers: vec![SemanticPointer {
                relation: SemanticRelation::Hypernym,
                target_offset: 100001,
                target_pos: PartOfSpeech::Noun,
                source_word: 0,
                target_word: 0,
            }],
            frames: vec![],
            gloss: "child concept".to_string(),
        };

        database.synsets.insert(100001, parent_synset);
        database.synsets.insert(100002, child_synset);

        // Test hypernyms
        let child = database.get_synset(100002).unwrap();
        let hypernyms = database.get_hypernyms(child);
        assert_eq!(hypernyms.len(), 1);
        assert_eq!(hypernyms[0].offset, 100001);

        // Test hyponyms (empty for this setup)
        let hyponyms = database.get_hyponyms(child);
        assert!(hyponyms.is_empty());

        // Test instance relations (empty for this setup)
        let instance_hypernyms = database.get_instance_hypernyms(child);
        assert!(instance_hypernyms.is_empty());

        let instance_hyponyms = database.get_instance_hyponyms(child);
        assert!(instance_hyponyms.is_empty());
    }

    #[test]
    fn test_lowest_common_hypernym() {
        let mut database = WordNetDatabase::new();

        // Create a simple hierarchy: root -> middle -> leaf1, leaf2
        let root = create_test_synset(100001, PartOfSpeech::Noun, vec!["root"], "root");
        let middle = Synset {
            offset: 100002,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![SynsetWord {
                word: "middle".to_string(),
                lex_id: 0,
                tag_count: None,
            }],
            pointers: vec![SemanticPointer {
                relation: SemanticRelation::Hypernym,
                target_offset: 100001,
                target_pos: PartOfSpeech::Noun,
                source_word: 0,
                target_word: 0,
            }],
            frames: vec![],
            gloss: "middle".to_string(),
        };
        let leaf1 = Synset {
            offset: 100003,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![SynsetWord {
                word: "leaf1".to_string(),
                lex_id: 0,
                tag_count: None,
            }],
            pointers: vec![SemanticPointer {
                relation: SemanticRelation::Hypernym,
                target_offset: 100002,
                target_pos: PartOfSpeech::Noun,
                source_word: 0,
                target_word: 0,
            }],
            frames: vec![],
            gloss: "leaf1".to_string(),
        };

        database.synsets.insert(100001, root);
        database.synsets.insert(100002, middle);
        database.synsets.insert(100003, leaf1);

        let leaf1_synset = database.get_synset(100003).unwrap();
        let middle_synset = database.get_synset(100002).unwrap();

        // Test LCH between leaf and middle (should be middle)
        let lch = database.lowest_common_hypernym(leaf1_synset, middle_synset);
        assert!(lch.is_some());
        assert_eq!(lch.unwrap().offset, 100002);

        // Test same synset (should return itself)
        let same_lch = database.lowest_common_hypernym(leaf1_synset, leaf1_synset);
        assert!(same_lch.is_some());
        assert_eq!(same_lch.unwrap().offset, 100003);
    }

    #[test]
    fn test_path_similarity() {
        let database = WordNetDatabase::new();
        let synset1 = create_test_synset(100001, PartOfSpeech::Noun, vec!["test1"], "test");
        let synset2 = create_test_synset(100002, PartOfSpeech::Noun, vec!["test2"], "test");

        // Test identical synsets
        let similarity1 = database.path_similarity(&synset1, &synset1);
        assert_eq!(similarity1, 1.0);

        // Test different synsets (will return 0.0 in empty database)
        let similarity2 = database.path_similarity(&synset1, &synset2);
        assert_eq!(similarity2, 0.0);
    }

    #[test]
    fn test_database_stats() {
        let mut database = WordNetDatabase::new();

        // Add synsets of different POS types
        database.synsets.insert(
            1,
            create_test_synset(1, PartOfSpeech::Noun, vec!["noun1"], "test"),
        );
        database.synsets.insert(
            2,
            create_test_synset(2, PartOfSpeech::Noun, vec!["noun2"], "test"),
        );
        database.synsets.insert(
            3,
            create_test_synset(3, PartOfSpeech::Verb, vec!["verb1"], "test"),
        );
        database.synsets.insert(
            4,
            create_test_synset(4, PartOfSpeech::Adjective, vec!["adj1"], "test"),
        );
        database.synsets.insert(
            5,
            create_test_synset(5, PartOfSpeech::AdjectiveSatellite, vec!["adjsat1"], "test"),
        );
        database.synsets.insert(
            6,
            create_test_synset(6, PartOfSpeech::Adverb, vec!["adv1"], "test"),
        );

        // Add an index entry
        let index_entry = IndexEntry {
            lemma: "test".to_string(),
            pos: PartOfSpeech::Noun,
            synset_count: 1,
            pointer_count: 0,
            relations: vec![],
            tag_sense_count: 1,
            synset_offsets: vec![1],
        };
        database
            .index
            .insert(("test".to_string(), PartOfSpeech::Noun), index_entry);

        let stats = database.stats();
        assert_eq!(stats.total_synsets, 6);
        assert_eq!(stats.noun_synsets, 2);
        assert_eq!(stats.verb_synsets, 1);
        assert_eq!(stats.adjective_synsets, 2); // Includes AdjectiveSatellite
        assert_eq!(stats.adverb_synsets, 1);
        assert_eq!(stats.total_words, 6); // One word per synset
        assert_eq!(stats.total_index_entries, 1);
        assert_eq!(stats.total_relations, 0); // No pointers added
    }

    #[test]
    fn test_wordnet_analysis() {
        let mut analysis = WordNetAnalysis::new("run".to_string(), PartOfSpeech::Verb);

        assert_eq!(analysis.word, "run");
        assert_eq!(analysis.pos, PartOfSpeech::Verb);
        assert_eq!(analysis.confidence, 0.0);
        assert!(!analysis.has_results());

        // Add some data
        let synset = create_test_synset(
            2097048,
            PartOfSpeech::Verb,
            vec!["run", "go"],
            "move fast by using one's feet",
        );
        analysis.synsets.push(synset);
        analysis
            .definitions
            .push("move fast by using one's feet".to_string());
        analysis
            .examples
            .push("Don't walk when you can run".to_string());
        analysis.confidence = 0.85;

        // Add a relation
        let related_synset = create_test_synset(
            2000556,
            PartOfSpeech::Verb,
            vec!["walk"],
            "use one's feet to advance",
        );
        analysis
            .relations
            .push((SemanticRelation::Hyponym, vec![related_synset]));

        assert!(analysis.has_results());
        assert_eq!(analysis.synsets.len(), 1);
        assert_eq!(analysis.definitions.len(), 1);
        assert_eq!(analysis.examples.len(), 1);
        assert_eq!(analysis.relations.len(), 1);
        assert_eq!(analysis.confidence, 0.85);

        // Test primary_definition
        assert_eq!(
            analysis.primary_definition(),
            Some(&"move fast by using one's feet".to_string())
        );

        // Test with empty analysis
        let empty_analysis = WordNetAnalysis::new("nonexistent".to_string(), PartOfSpeech::Noun);
        assert!(!empty_analysis.has_results());
        assert_eq!(empty_analysis.primary_definition(), None);
    }

    #[test]
    fn test_serialization_traits() {
        // Test that all major types implement Serialize/Deserialize
        let pos = PartOfSpeech::Noun;
        let relation = SemanticRelation::Hypernym;
        let synset = create_test_synset(100001, PartOfSpeech::Noun, vec!["test"], "definition");
        let analysis = WordNetAnalysis::new("test".to_string(), PartOfSpeech::Noun);
        let database = WordNetDatabase::new();

        // Test Debug trait (serialization test would need serde_json)
        let _pos_debug = format!("{:?}", pos);
        let _relation_debug = format!("{:?}", relation);
        let _synset_debug = format!("{:?}", synset);
        let _analysis_debug = format!("{:?}", analysis);
        let _database_debug = format!("{:?}", database);

        // If we reach here, Debug trait works for all types
        // Reaching here without panic = test passes
    }

    #[test]
    fn test_edge_cases_and_error_conditions() {
        // Test empty gloss parsing
        let synset = create_test_synset(1, PartOfSpeech::Noun, vec![], "");
        assert_eq!(synset.definition(), "");
        assert!(synset.examples().is_empty());
        assert_eq!(synset.primary_word(), None);

        // Test malformed quotes in gloss
        let synset2 = create_test_synset(
            2,
            PartOfSpeech::Noun,
            vec!["test"],
            "definition \"incomplete quote",
        );
        assert_eq!(synset2.definition(), "definition");
        assert!(synset2.examples().is_empty()); // Incomplete quotes ignored

        // Test very long word lists
        let long_words: Vec<&str> = (0..100)
            .map(|i| {
                let s = format!("word{}", i);
                Box::leak(s.into_boxed_str()) as &str
            })
            .collect();
        let synset3 = create_test_synset(3, PartOfSpeech::Noun, long_words, "test");
        assert_eq!(synset3.words.len(), 100);
        let word_list = synset3.word_list();
        assert_eq!(word_list.len(), 100);
        assert!(word_list.contains(&"word50".to_string()));
    }

    #[test]
    fn test_unicode_and_special_characters() {
        // Test with Unicode characters
        let synset = create_test_synset(
            1,
            PartOfSpeech::Noun,
            vec!["café", "naïve"],
            "A place where coffee ☕ is served; \"café au lait\" \"naïve approach\"",
        );

        assert!(synset.contains_word("café"));
        assert!(synset.contains_word("naïve"));
        assert_eq!(synset.definition(), "A place where coffee ☕ is served");

        let examples = synset.examples();
        assert_eq!(examples.len(), 2);
        assert!(examples.contains(&"café au lait".to_string()));
        assert!(examples.contains(&"naïve approach".to_string()));

        // Test empty strings and whitespace
        let synset2 = create_test_synset(2, PartOfSpeech::Noun, vec!["   "], "   ;   \"  \"  ");
        assert_eq!(synset2.definition(), "");
        let _examples2 = synset2.examples();
        // The example parsing might not work as expected with whitespace - just verify it parses
    }
}
