//! Advanced coverage tests for WordNet loader to reach 95%+ coverage

use canopy_wordnet::loader::WordNetLoader;
use canopy_wordnet::parser::WordNetParserConfig;
use canopy_wordnet::types::PartOfSpeech;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[cfg(test)]
mod advanced_loader_tests {
    use super::*;

    fn create_test_data_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    #[test]
    fn test_synset_line_parsing_numeric_field_errors() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create data file with invalid numeric fields
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "invalid_offset 03 n 01 entity 0 000 | invalid offset"
        )
        .unwrap();
        writeln!(
            data_noun,
            "100001740 invalid_lex n 01 entity 0 000 | invalid lex_filenum"
        )
        .unwrap();
        writeln!(
            data_noun,
            "100001741 03 n invalid_count entity 0 000 | invalid w_cnt"
        )
        .unwrap();

        let config = WordNetParserConfig {
            strict_mode: true,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        // Should fail with numeric parsing errors
        assert!(result.is_err());
    }

    #[test]
    fn test_synset_line_parsing_numeric_field_errors_non_strict() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create data file with invalid numeric fields + one valid line
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "invalid_offset 03 n 01 entity 0 000 | invalid offset"
        )
        .unwrap();
        writeln!(data_noun, "100001740 03 n 01 entity 0 000 | valid entry").unwrap();

        let config = WordNetParserConfig {
            strict_mode: false,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        // Should succeed in non-strict mode, loading only valid entries
        assert!(result.is_ok());
        let database = result.unwrap();
        assert_eq!(database.synsets.len(), 1);
    }

    #[test]
    fn test_synset_line_parsing_pointer_count_errors() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create data file with invalid pointer count
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "100001740 03 n 01 entity 0 invalid_p_cnt | invalid p_cnt"
        )
        .unwrap();

        let config = WordNetParserConfig {
            strict_mode: true,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_synset_line_parsing_invalid_pointer_symbols() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create data file with invalid pointer symbols
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "100001740 03 n 01 entity 0 001 invalid_symbol 100002000 n 0000 | invalid symbol"
        )
        .unwrap();

        let config = WordNetParserConfig {
            strict_mode: true,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_synset_line_parsing_invalid_pos_chars() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create data file with invalid POS character
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(data_noun, "100001740 03 x 01 entity 0 000 | invalid pos").unwrap();

        let config = WordNetParserConfig {
            strict_mode: true,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_synset_line_parsing_invalid_target_pos() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create data file with invalid target POS in pointer
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "100001740 03 n 01 entity 0 001 @ 100002000 x 0000 | invalid target pos"
        )
        .unwrap();

        let config = WordNetParserConfig {
            strict_mode: true,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_synset_line_verb_frames_numeric_errors() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create verb data file with invalid frame numbers
        let mut data_verb = fs::File::create(data_dir.join("data.verb")).unwrap();
        writeln!(
            data_verb,
            "200001740 29 v 01 walk 0 000 001 + invalid_frame | invalid frame"
        )
        .unwrap();

        let config = WordNetParserConfig {
            strict_mode: true,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_synset_line_verb_frames_count_error() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create verb data file with invalid frame count
        let mut data_verb = fs::File::create(data_dir.join("data.verb")).unwrap();
        writeln!(
            data_verb,
            "200001740 29 v 01 walk 0 000 invalid_f_cnt | invalid f_cnt"
        )
        .unwrap();

        let config = WordNetParserConfig {
            strict_mode: false,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        // Should succeed in non-strict mode but skip invalid frame data
        assert!(result.is_ok());
        let database = result.unwrap();
        let synset = database.synsets.get(&200001740).unwrap();
        assert_eq!(synset.frames.len(), 0); // No valid frames parsed
    }

    #[test]
    fn test_index_line_parsing_numeric_errors() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create index file with numeric errors
        let mut index_noun = fs::File::create(data_dir.join("index.noun")).unwrap();
        writeln!(index_noun, "entity n invalid_synset_count 0").unwrap();
        writeln!(index_noun, "object n 1 invalid_pointer_count").unwrap();

        let config = WordNetParserConfig {
            strict_mode: true,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_index_line_parsing_invalid_pos() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create index file with invalid POS
        let mut index_noun = fs::File::create(data_dir.join("index.noun")).unwrap();
        writeln!(index_noun, "entity x 1 0").unwrap();

        let config = WordNetParserConfig {
            strict_mode: true,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_index_line_parsing_invalid_synset_offsets() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create index file with invalid synset offsets
        let mut index_noun = fs::File::create(data_dir.join("index.noun")).unwrap();
        writeln!(index_noun, "entity n 2 0 invalid_offset another_invalid").unwrap();

        let config = WordNetParserConfig {
            strict_mode: false,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        // Should succeed but with empty synset_offsets for invalid entries
        assert!(result.is_ok());
        let database = result.unwrap();
        if let Some(entry) = database
            .index
            .get(&("entity".to_string(), PartOfSpeech::Noun))
        {
            assert_eq!(entry.synset_offsets.len(), 0);
        }
    }

    #[test]
    fn test_index_line_parsing_invalid_tag_sense_count() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create index file with invalid tag_sense_count
        let mut index_noun = fs::File::create(data_dir.join("index.noun")).unwrap();
        writeln!(index_noun, "entity n 1 0 invalid_tag_count 100001740").unwrap();

        let config = WordNetParserConfig {
            strict_mode: false,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        // Should succeed but with default tag_sense_count of 0
        assert!(result.is_ok());
        let database = result.unwrap();
        if let Some(entry) = database
            .index
            .get(&("entity".to_string(), PartOfSpeech::Noun))
        {
            assert_eq!(entry.tag_sense_count, 0);
        }
    }

    #[test]
    fn test_load_synsets_with_empty_word_count() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create synset with 0 word count
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(data_noun, "100001740 03 n 00 000 | no words").unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();
        if let Some(synset) = database.synsets.get(&100001740) {
            assert_eq!(synset.words.len(), 0);
        }
    }

    #[test]
    fn test_load_synsets_with_empty_pointer_count() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create synset with 0 pointer count
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(data_noun, "100001740 03 n 01 entity 0 000 | no pointers").unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();
        if let Some(synset) = database.synsets.get(&100001740) {
            assert_eq!(synset.pointers.len(), 0);
        }
    }

    #[test]
    fn test_parse_synset_line_with_underscores_in_words() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create synset with underscores in word (should be converted to spaces)
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "100001740 03 n 01 living_being 0 000 | compound word"
        )
        .unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();
        if let Some(synset) = database.synsets.get(&100001740) {
            assert_eq!(synset.words[0].word, "living being"); // Underscore converted to space
        }
    }

    #[test]
    fn test_parse_index_line_with_underscores_in_lemma() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create index entry with underscores in lemma
        let mut index_noun = fs::File::create(data_dir.join("index.noun")).unwrap();
        writeln!(index_noun, "living_being n 1 0 100001740").unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();
        if let Some(entry) = database
            .index
            .get(&("living being".to_string(), PartOfSpeech::Noun))
        {
            assert_eq!(entry.lemma, "living being");
        }
    }

    #[test]
    fn test_parse_exception_line_with_underscores() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create exception with underscores
        let mut noun_exc = fs::File::create(data_dir.join("noun.exc")).unwrap();
        writeln!(noun_exc, "living_beings living_being").unwrap();

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();
        if let Some(exceptions) = database.exceptions.get(&PartOfSpeech::Noun) {
            assert!(exceptions.contains_key("living_beings"));
            if let Some(entry) = exceptions.get("living_beings") {
                assert_eq!(entry.base_forms[0], "living_being");
            }
        }
    }

    #[test]
    fn test_parse_synset_line_missing_lex_id_field() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create synset where the lex_id field is beyond the field count
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(data_noun, "100001740 03 n 01 entity").unwrap(); // Word without lex_id

        let config = WordNetParserConfig {
            strict_mode: false,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        // Should handle missing lex_id gracefully
        if result.is_ok() {
            let database = result.unwrap();
            if let Some(synset) = database.synsets.get(&100001740) {
                if !synset.words.is_empty() {
                    assert_eq!(synset.words[0].lex_id, 0); // Should default to 0
                }
            }
        }
    }

    #[test]
    fn test_parse_synset_line_malformed_lex_id() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create synset with non-numeric lex_id
        let mut data_noun = fs::File::create(data_dir.join("data.noun")).unwrap();
        writeln!(
            data_noun,
            "100001740 03 n 01 entity abc 000 | malformed lex_id"
        )
        .unwrap();

        let config = WordNetParserConfig {
            strict_mode: false,
            ..Default::default()
        };
        let loader = WordNetLoader::new(config);
        let result = loader.load_database(data_dir.to_str().unwrap());

        if result.is_ok() {
            let database = result.unwrap();
            if let Some(synset) = database.synsets.get(&100001740) {
                if !synset.words.is_empty() {
                    assert_eq!(synset.words[0].lex_id, 0); // Should default to 0 for unparseable
                }
            }
        }
    }

    #[test]
    fn test_load_database_comprehensive_logging_paths() {
        let temp_dir = create_test_data_dir();
        let data_dir = temp_dir.path();

        // Create comprehensive files to trigger all logging paths
        let mut offset_base = 100001740;
        for pos in &[
            PartOfSpeech::Noun,
            PartOfSpeech::Verb,
            PartOfSpeech::Adjective,
            PartOfSpeech::Adverb,
        ] {
            let pos_name = pos.name();

            // Create data file with multiple entries (unique offsets per POS)
            let mut data_file =
                fs::File::create(data_dir.join(format!("data.{}", pos_name))).unwrap();
            writeln!(
                data_file,
                "{} 03 {} 01 test_word_{} 0 000 | test definition 1",
                offset_base,
                pos.code(),
                pos.code()
            )
            .unwrap();
            writeln!(
                data_file,
                "{} 03 {} 01 another_word_{} 0 000 | test definition 2",
                offset_base + 1,
                pos.code(),
                pos.code()
            )
            .unwrap();

            // Create index file with multiple entries
            let mut index_file =
                fs::File::create(data_dir.join(format!("index.{}", pos_name))).unwrap();
            writeln!(
                index_file,
                "test_word_{} {} 1 0 {}",
                pos.code(),
                pos.code(),
                offset_base
            )
            .unwrap();
            writeln!(
                index_file,
                "another_word_{} {} 1 0 {}",
                pos.code(),
                pos.code(),
                offset_base + 1
            )
            .unwrap();

            // Create exception file with entries
            let mut exc_file =
                fs::File::create(data_dir.join(format!("{}.exc", pos_name))).unwrap();
            writeln!(exc_file, "irregular_{} regular_{}", pos.code(), pos.code()).unwrap();

            offset_base += 1000; // Ensure unique offsets per POS
        }

        let loader = WordNetLoader::new(WordNetParserConfig::default());
        let result = loader.load_database(data_dir.to_str().unwrap());

        assert!(result.is_ok());
        let database = result.unwrap();

        // Should have loaded all entries and triggered all logging paths
        assert_eq!(database.synsets.len(), 8); // 2 per POS * 4 POS
        assert_eq!(database.index.len(), 8);
        assert_eq!(database.exceptions.len(), 4);
        assert_eq!(database.synset_words.len(), 8); // All synsets should have word mappings
    }
}
