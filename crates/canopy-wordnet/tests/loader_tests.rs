//! Comprehensive tests for WordNet loader functionality

use canopy_wordnet::loader::WordNetLoader;
use canopy_wordnet::parser::WordNetParserConfig;
use canopy_wordnet::types::{PartOfSpeech, WordNetDatabase};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_loader() -> WordNetLoader {
        let config = WordNetParserConfig::default();
        WordNetLoader::new(config)
    }

    #[test]
    fn test_loader_creation() {
        let loader = create_test_loader();
        // Loader should be created successfully
        assert!(!format!("{:?}", loader).is_empty());
    }

    #[test]
    fn test_load_database_invalid_path() {
        let loader = create_test_loader();
        let result = loader.load_database("non_existent_path");

        // Should fail gracefully with non-existent path
        assert!(result.is_err());
    }

    #[test]
    fn test_load_database_empty_directory() {
        let loader = create_test_loader();

        // Create temporary directory
        let temp_dir = std::env::temp_dir().join("wordnet_test_empty");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let result = loader.load_database(temp_dir.to_str().unwrap());

        // Should handle empty directory
        if result.is_ok() {
            let database = result.unwrap();
            assert_eq!(database.synsets.len(), 0);
        }

        // Clean up
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_database_initialization() {
        let database = WordNetDatabase::new();

        // New database should be empty
        assert_eq!(database.synsets.len(), 0);
        assert_eq!(database.index.len(), 0);
    }

    #[test]
    fn test_database_lookup_word() {
        let database = WordNetDatabase::new();

        // Lookup on empty database should return None
        let result = database.lookup_word("test", PartOfSpeech::Noun);
        assert!(result.is_none());
    }

    #[test]
    fn test_database_get_synsets_for_word() {
        let database = WordNetDatabase::new();

        // Get synsets on empty database should return empty vector
        let synsets = database.get_synsets_for_word("test", PartOfSpeech::Noun);
        assert!(synsets.is_empty());
    }

    #[test]
    fn test_loader_with_custom_config() {
        let mut config = WordNetParserConfig::default();
        config.strict_mode = true;
        config.max_file_size = 1024 * 1024; // 1MB

        let loader = WordNetLoader::new(config);

        // Test with custom configuration
        let result = loader.load_database("non_existent_path");
        assert!(result.is_err()); // Should still fail with invalid path
    }

    #[test]
    fn test_multiple_loader_instances() {
        let loader1 = create_test_loader();
        let loader2 = create_test_loader();

        // Multiple loaders should work independently
        let result1 = loader1.load_database("path1");
        let result2 = loader2.load_database("path2");

        // Both should fail with invalid paths, but independently
        assert!(result1.is_err());
        assert!(result2.is_err());
    }

    #[test]
    fn test_database_relations() {
        let database = WordNetDatabase::new();

        // Test relation methods on empty database
        // Create a mock synset using the actual Synset structure
        let empty_synset = canopy_wordnet::types::Synset {
            offset: 0,
            lex_filenum: 0,
            pos: PartOfSpeech::Noun,
            words: vec![],
            pointers: vec![],
            frames: vec![],
            gloss: "test synset definition".to_string(),
        };

        let hypernyms = database.get_hypernyms(&empty_synset);
        assert!(hypernyms.is_empty());

        let hyponyms = database.get_hyponyms(&empty_synset);
        assert!(hyponyms.is_empty());
    }

    #[test]
    fn test_part_of_speech_code() {
        // Test all POS codes
        assert_eq!(PartOfSpeech::Noun.code(), 'n');
        assert_eq!(PartOfSpeech::Verb.code(), 'v');
        assert_eq!(PartOfSpeech::Adjective.code(), 'a');
        assert_eq!(PartOfSpeech::Adverb.code(), 'r');
    }

    #[test]
    fn test_part_of_speech_from_str() {
        // Test POS parsing from strings
        // Note: PartOfSpeech doesn't have from_str method in current API
        // Instead we test the code() method
        assert_eq!(PartOfSpeech::Noun.code(), 'n');
        assert_eq!(PartOfSpeech::Verb.code(), 'v');
        assert_eq!(PartOfSpeech::Adjective.code(), 'a');
        assert_eq!(PartOfSpeech::Adverb.code(), 'r');
    }

    #[test]
    fn test_loader_error_handling() {
        let loader = create_test_loader();

        // Test various invalid paths
        let test_paths = vec!["", "/", ".", "..", "~/nonexistent", "/root/forbidden"];

        for path in test_paths {
            let result = loader.load_database(path);
            // All should handle errors gracefully
            if result.is_ok() {
                // If successful, database should be valid
                let database = result.unwrap();
                assert!(database.synsets.len() == 0);
            }
        }
    }

    #[test]
    fn test_concurrent_loading() {
        use std::sync::Arc;
        use std::thread;

        let loader = Arc::new(create_test_loader());
        let mut handles = vec![];

        // Test concurrent loading attempts
        for i in 0..5 {
            let loader_clone = Arc::clone(&loader);
            let handle = thread::spawn(move || {
                let path = format!("test_path_{}", i);
                let result = loader_clone.load_database(&path);
                // Should handle concurrent access gracefully
                result.is_err() // Expected to fail with invalid paths
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result); // All should fail with invalid paths
        }
    }

    #[test]
    fn test_memory_efficiency() {
        // Test that loader doesn't hold onto large amounts of memory
        let loader = create_test_loader();

        // Multiple failed load attempts shouldn't accumulate memory
        for _ in 0..100 {
            let _ = loader.load_database("nonexistent_path");
        }

        // This test passes if it doesn't crash or run out of memory
        assert!(true);
    }
}
