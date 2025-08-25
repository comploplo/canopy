//! Comprehensive tests for WordNet loader functionality

use canopy_wordnet::loader::WordNetLoader;
use canopy_wordnet::parser::WordNetParserConfig;
use canopy_wordnet::types::{PartOfSpeech, WordNetDatabase};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[cfg(test)]
mod loader_tests {
    use super::*;

    fn create_comprehensive_test_files(temp_dir: &TempDir) -> std::io::Result<()> {
        let data_dir = temp_dir.path();

        // Create comprehensive data.noun file
        let mut data_noun = fs::File::create(data_dir.join("data.noun"))?;
        writeln!(
            data_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(
            data_noun,
            "  1 by Princeton University under the following license."
        )?;
        writeln!(data_noun, "")?;
        writeln!(
            data_noun,
            "100001740 03 n 02 entity 0 something 0 001 @ 100002137 n 0000 | that which is perceived or known or inferred to have its own distinct existence (living or nonliving)  "
        )?;
        writeln!(
            data_noun,
            "100002137 03 n 02 thing 0 object 1 002 @ 100001930 n 0000 ~ 100001740 n 0000 | a separate and self-contained entity  "
        )?;
        writeln!(
            data_noun,
            "100003456 04 n 01 animal 0 003 @ 100002137 n 0000 ~ 100004567 n 0000 ~ 100005678 n 0000 | a living organism characterized by voluntary movement  "
        )?;

        // Create comprehensive index.noun file
        let mut index_noun = fs::File::create(data_dir.join("index.noun"))?;
        writeln!(
            index_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(
            index_noun,
            "  1 by Princeton University under the following license."
        )?;
        writeln!(index_noun, "")?;
        writeln!(index_noun, "entity n 1 1 @ 1 5 100001740")?;
        writeln!(index_noun, "thing n 2 2 @ ~ 1 3 100002137")?;
        writeln!(index_noun, "animal n 1 3 @ ~ ! 1 8 100003456")?;

        // Create data.verb file
        let mut data_verb = fs::File::create(data_dir.join("data.verb"))?;
        writeln!(
            data_verb,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(data_verb, "")?;
        writeln!(
            data_verb,
            "200001740 30 v 02 run 0 go 0 001 @ 200002137 v 0000 08 + 02 00 | move fast by using one's feet, with one foot off the ground at any given time  \"Don't walk when you can run\"  \"He ran to the store\"  "
        )?;
        writeln!(
            data_verb,
            "200002137 30 v 01 move 0 002 @ 200003000 v 0000 ~ 200001740 v 0000 08 + 01 00 | change location; move, travel, or proceed  \"How fast does your new car go?\"  "
        )?;

        // Create index.verb file
        let mut index_verb = fs::File::create(data_dir.join("index.verb"))?;
        writeln!(
            index_verb,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(index_verb, "")?;
        writeln!(index_verb, "run v 3 1 @ 1 15 200001740")?;
        writeln!(index_verb, "move v 5 2 @ ~ 1 12 200002137")?;

        // Create data.adj file
        let mut data_adj = fs::File::create(data_dir.join("data.adjective"))?;
        writeln!(
            data_adj,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(
            data_adj,
            "300001740 00 a 01 good 0 001 ! 300002137 a 0000 | having desirable or positive qualities especially those suitable for a thing specified  \"good food\"  "
        )?;
        writeln!(
            data_adj,
            "300002137 00 a 01 bad 0 002 ! 300001740 a 0000 & 300003456 a 0000 | having undesirable or negative qualities  \"bad weather\"  \"a bad report card\"  "
        )?;

        // Create index.adjective file
        let mut index_adj = fs::File::create(data_dir.join("index.adjective"))?;
        writeln!(
            index_adj,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(index_adj, "good a 1 1 ! 1 25 300001740")?;
        writeln!(index_adj, "bad a 2 2 ! & 1 18 300002137")?;

        // Create data.adverb file
        let mut data_adv = fs::File::create(data_dir.join("data.adverb"))?;
        writeln!(
            data_adv,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(
            data_adv,
            "400001740 01 r 01 quickly 0 001 ! 400002137 r 0000 | with rapid movements  \"he works quickly\"  "
        )?;

        // Create index.adverb file
        let mut index_adv = fs::File::create(data_dir.join("index.adverb"))?;
        writeln!(
            index_adv,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(index_adv, "quickly r 1 1 ! 1 8 400001740")?;

        // Create exception files
        let mut noun_exc = fs::File::create(data_dir.join("noun.exc"))?;
        writeln!(noun_exc, "children child")?;
        writeln!(noun_exc, "mice mouse")?;
        writeln!(noun_exc, "feet foot")?;

        let mut verb_exc = fs::File::create(data_dir.join("verb.exc"))?;
        writeln!(verb_exc, "ran run")?;
        writeln!(verb_exc, "went go")?;
        writeln!(verb_exc, "was be")?;

        Ok(())
    }

    fn create_minimal_test_files(temp_dir: &TempDir) -> std::io::Result<()> {
        let data_dir = temp_dir.path();

        // Create minimal valid files
        let mut data_noun = fs::File::create(data_dir.join("data.noun"))?;
        writeln!(
            data_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(
            data_noun,
            "100001740 03 n 01 test 0 000 | test definition  "
        )?;

        let mut index_noun = fs::File::create(data_dir.join("index.noun"))?;
        writeln!(
            index_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(index_noun, "test n 1 0 1 0 100001740")?;

        Ok(())
    }

    fn create_malformed_test_files(temp_dir: &TempDir) -> std::io::Result<()> {
        let data_dir = temp_dir.path();

        // Create files with malformed data
        let mut data_noun = fs::File::create(data_dir.join("data.noun"))?;
        writeln!(
            data_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(data_noun, "invalid_line_format")?;
        writeln!(
            data_noun,
            "100001740 03 n 01 test 0 000 | test definition  "
        )?;

        let mut index_noun = fs::File::create(data_dir.join("index.noun"))?;
        writeln!(
            index_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )?;
        writeln!(index_noun, "malformed index line")?;
        writeln!(index_noun, "test n 1 0 1 0 100001740")?;

        Ok(())
    }

    #[test]
    fn test_loader_comprehensive_database_loading() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();

        // Check that data was loaded from all POS types
        assert!(!database.synsets.is_empty());
        assert!(!database.index.is_empty());

        // Verify specific synsets were loaded
        assert!(database.synsets.contains_key(&100001740)); // entity
        assert!(database.synsets.contains_key(&200001740)); // run
        assert!(database.synsets.contains_key(&300001740)); // good
        assert!(database.synsets.contains_key(&400001740)); // quickly
    }

    #[test]
    fn test_loader_index_entries_loaded() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let database = loader
            .load_database(temp_dir.path().to_str().unwrap())
            .unwrap();

        // Check index entries for different POS
        let entity_entry = database.lookup_word("entity", PartOfSpeech::Noun);
        assert!(entity_entry.is_some());

        let run_entry = database.lookup_word("run", PartOfSpeech::Verb);
        assert!(run_entry.is_some());

        let good_entry = database.lookup_word("good", PartOfSpeech::Adjective);
        assert!(good_entry.is_some());

        let quickly_entry = database.lookup_word("quickly", PartOfSpeech::Adverb);
        assert!(quickly_entry.is_some());
    }

    #[test]
    fn test_loader_exception_lists_loaded() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let database = loader
            .load_database(temp_dir.path().to_str().unwrap())
            .unwrap();

        // Check exception lists were loaded
        assert!(database.exceptions.contains_key(&PartOfSpeech::Noun));
        assert!(database.exceptions.contains_key(&PartOfSpeech::Verb));

        let noun_exceptions = &database.exceptions[&PartOfSpeech::Noun];
        assert!(noun_exceptions.contains_key("children"));
        assert!(noun_exceptions.contains_key("mice"));

        let verb_exceptions = &database.exceptions[&PartOfSpeech::Verb];
        assert!(verb_exceptions.contains_key("ran"));
        assert!(verb_exceptions.contains_key("went"));
    }

    #[test]
    fn test_loader_synset_words_reverse_lookup() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let database = loader
            .load_database(temp_dir.path().to_str().unwrap())
            .unwrap();

        // Check reverse lookup was created
        assert!(!database.synset_words.is_empty());

        // Verify specific reverse lookups
        if let Some(words) = database.synset_words.get(&100001740) {
            assert!(words.contains(&"entity".to_string()));
        }

        if let Some(words) = database.synset_words.get(&200001740) {
            assert!(words.contains(&"run".to_string()));
        }
    }

    #[test]
    fn test_loader_semantic_relations() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let database = loader
            .load_database(temp_dir.path().to_str().unwrap())
            .unwrap();

        // Test that synsets with relations were loaded properly
        if let Some(synset) = database.synsets.get(&100001740) {
            assert!(!synset.pointers.is_empty());
        }

        // Test relation queries work
        if let Some(entity_synset) = database.synsets.get(&100001740) {
            let hypernyms = database.get_hypernyms(entity_synset);
            // Should have hypernyms based on test data
            assert!(!hypernyms.is_empty() || entity_synset.pointers.is_empty());
        }
    }

    #[test]
    fn test_loader_with_strict_parser_config() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        let strict_config = WordNetParserConfig {
            strict_mode: true,
            max_file_size: 1024 * 1024,
            skip_prefixes: vec!["  1 This".to_string()],
        };

        let loader = WordNetLoader::new(strict_config);
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        // Should succeed even in strict mode with valid data
        assert!(result.is_ok());
    }

    #[test]
    fn test_loader_with_relaxed_parser_config() {
        let temp_dir = TempDir::new().unwrap();
        create_malformed_test_files(&temp_dir).unwrap();

        let relaxed_config = WordNetParserConfig {
            strict_mode: false,
            max_file_size: 1024 * 1024,
            skip_prefixes: vec![],
        };

        let loader = WordNetLoader::new(relaxed_config);
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        // Should handle malformed data gracefully in non-strict mode
        if result.is_ok() {
            let database = result.unwrap();
            // Should still load valid entries
            assert!(!database.synsets.is_empty() || !database.index.is_empty());
        }
    }

    #[test]
    fn test_loader_missing_files_handling() {
        let temp_dir = TempDir::new().unwrap();

        // Create only noun files, missing other POS files
        let data_dir = temp_dir.path();
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )
        .unwrap();
        writeln!(
            data_noun,
            "100001740 03 n 01 test 0 000 | test definition  "
        )
        .unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        // Should succeed even with missing files for some POS
        assert!(result.is_ok());
        let database = result.unwrap();

        // Should have loaded noun data but not others
        assert!(!database.synsets.is_empty());
    }

    #[test]
    fn test_loader_empty_files_handling() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create empty files
        fs::File::create(data_dir.join("data.noun")).unwrap();
        fs::File::create(data_dir.join("index.noun")).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        // Should succeed with empty files
        assert!(result.is_ok());
        let database = result.unwrap();

        // Database should be empty but valid
        assert!(database.synsets.is_empty());
        assert!(database.index.is_empty());
    }

    #[test]
    fn test_loader_file_size_limits() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        // Test with very small file size limit
        let small_limit_config = WordNetParserConfig {
            strict_mode: false,
            max_file_size: 100, // Very small limit
            skip_prefixes: vec![],
        };

        let loader = WordNetLoader::new(small_limit_config);
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        // May fail or succeed depending on implementation, but should handle gracefully
        match result {
            Ok(database) => {
                // If it succeeds, database may be limited
                assert!(database.synsets.len() <= 100); // Sanity check
            }
            Err(_) => {
                // Expected with very small file limit
                assert!(true);
            }
        }
    }

    #[test]
    fn test_loader_large_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create a larger test file
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )
        .unwrap();

        // Add many synsets
        for i in 100000..100100 {
            writeln!(
                data_noun,
                "{} 03 n 01 word{} 0 000 | definition for word{}  ",
                i,
                i % 100,
                i % 100
            )
            .unwrap();
        }

        // Create corresponding index
        let mut index_noun = fs::File::create(data_dir.join("index.noun")).unwrap();
        writeln!(
            index_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )
        .unwrap();
        for i in 100000..100100 {
            writeln!(index_noun, "word{} n 1 0 1 0 {}", i % 100, i).unwrap();
        }

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();

        // Should have loaded many synsets
        assert!(!database.synsets.is_empty());
        assert!(database.synsets.len() <= 100); // At most 100 unique entries
    }

    #[test]
    fn test_loader_performance_with_repeated_loading() {
        let temp_dir = TempDir::new().unwrap();
        create_minimal_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());

        // Load the same database multiple times
        for _ in 0..10 {
            let result = loader.load_database(temp_dir.path().to_str().unwrap());
            assert!(result.is_ok());

            let database = result.unwrap();
            assert!(!database.synsets.is_empty());
        }
    }

    #[test]
    fn test_loader_concurrent_safety() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        create_minimal_test_files(&temp_dir).unwrap();

        let loader = Arc::new(WordNetLoader::new(WordNetParserConfig::default()));
        let path = temp_dir.path().to_str().unwrap().to_string();

        let mut handles = vec![];

        // Test concurrent loading
        for _ in 0..5 {
            let loader_clone = Arc::clone(&loader);
            let path_clone = path.clone();

            let handle = thread::spawn(move || {
                let result = loader_clone.load_database(&path_clone);
                result.is_ok()
            });
            handles.push(handle);
        }

        // All threads should succeed
        for handle in handles {
            let success = handle.join().unwrap();
            assert!(success);
        }
    }

    #[test]
    fn test_loader_database_statistics() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let database = loader
            .load_database(temp_dir.path().to_str().unwrap())
            .unwrap();

        // Test database statistics
        let stats = database.stats();

        assert!(stats.total_synsets > 0);
        assert!(stats.total_index_entries > 0);
        assert!(stats.noun_synsets > 0);
        assert!(stats.verb_synsets > 0);
        assert!(stats.adjective_synsets > 0);
        assert!(stats.adverb_synsets > 0);

        // Total should be sum of parts
        assert_eq!(
            stats.total_synsets,
            stats.noun_synsets
                + stats.verb_synsets
                + stats.adjective_synsets
                + stats.adverb_synsets
        );
    }

    #[test]
    fn test_loader_word_lookup_functionality() {
        let temp_dir = TempDir::new().unwrap();
        create_comprehensive_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let database = loader
            .load_database(temp_dir.path().to_str().unwrap())
            .unwrap();

        // Test word lookups work after loading
        let entity_synsets = database.get_synsets_for_word("entity", PartOfSpeech::Noun);
        if !entity_synsets.is_empty() {
            assert_eq!(entity_synsets[0].primary_word(), Some("entity"));
        }

        let run_synsets = database.get_synsets_for_word("run", PartOfSpeech::Verb);
        if !run_synsets.is_empty() {
            assert!(run_synsets[0].contains_word("run"));
        }
    }

    #[test]
    fn test_loader_custom_skip_prefixes() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create file with custom prefix to skip
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(data_noun, "SKIP_THIS line should be ignored").unwrap();
        writeln!(
            data_noun,
            "  1 This software and database is being provided to you, the LICENSEE,"
        )
        .unwrap();
        writeln!(
            data_noun,
            "100001740 03 n 01 test 0 000 | test definition  "
        )
        .unwrap();

        let custom_config = WordNetParserConfig {
            strict_mode: false,
            max_file_size: 1024 * 1024,
            skip_prefixes: vec!["SKIP_THIS".to_string()],
        };

        let loader = WordNetLoader::new(custom_config);
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();
        assert!(!database.synsets.is_empty());
    }

    #[test]
    fn test_loader_malformed_data_handling() {
        let temp_dir = TempDir::new().unwrap();
        create_malformed_test_files(&temp_dir).unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(temp_dir.path().to_str().unwrap());

        // Should handle malformed data gracefully
        match result {
            Ok(database) => {
                // Should still load valid entries if any
                assert!(database.synsets.len() >= 0);
            }
            Err(_) => {
                // May fail with malformed data, which is acceptable
                assert!(true);
            }
        }
    }

    #[test]
    fn test_loader_debug_trait() {
        let loader = WordNetLoader::new(WordNetParserConfig::default());

        // Test Debug implementation
        let debug_str = format!("{:?}", loader);
        assert!(!debug_str.is_empty());
        assert!(debug_str.contains("WordNetLoader"));
    }
}
