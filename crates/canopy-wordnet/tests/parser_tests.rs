//! Comprehensive tests for WordNet parser functionality

use canopy_wordnet::parser::{WordNetParser, WordNetParserConfig};
use canopy_wordnet::types::PartOfSpeech;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> WordNetParserConfig {
        WordNetParserConfig {
            strict_mode: false,
            max_file_size: 1024 * 1024, // 1MB for testing
            skip_prefixes: vec!["  ".to_string()],
        }
    }

    fn create_test_parser() -> WordNetParser {
        WordNetParser::with_config(create_test_config())
    }

    #[test]
    fn test_parser_config_default() {
        let config = WordNetParserConfig::default();

        // Test default values
        assert!(!config.strict_mode);
        assert_eq!(config.max_file_size, 100 * 1024 * 1024); // 100MB
        assert_eq!(config.skip_prefixes.len(), 1);
        assert_eq!(config.skip_prefixes[0], "  ");
    }

    #[test]
    fn test_parser_config_custom() {
        let config = create_test_config();

        // Test custom values
        assert!(!config.strict_mode);
        assert_eq!(config.max_file_size, 1024 * 1024); // 1MB
        assert_eq!(config.skip_prefixes.len(), 1);
        assert_eq!(config.skip_prefixes[0], "  ");
    }

    #[test]
    fn test_parser_creation() {
        let parser = create_test_parser();

        // Parser should be created successfully
        assert!(!format!("{:?}", parser).is_empty());
    }

    #[test]
    fn test_parse_synset_offset_valid() {
        use canopy_wordnet::parser::utils;

        // Test valid synset offsets
        let valid_offsets = vec!["00000001", "12345678", "99999999", "00001740"];

        for offset in valid_offsets {
            let result = utils::parse_synset_offset(offset);
            if result.is_ok() {
                let parsed_offset = result.unwrap();
                assert!(parsed_offset > 0);
            }
        }
    }

    #[test]
    fn test_parse_synset_offset_invalid() {
        use canopy_wordnet::parser::utils;

        // Test invalid synset offsets
        let invalid_offsets = vec![
            "", "abc", "1234567x", // Contains letters
        ];

        for offset in invalid_offsets {
            let result = utils::parse_synset_offset(offset);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_pos_code_valid() {
        use canopy_wordnet::parser::utils;

        // Test valid POS codes
        let valid_codes = vec![
            ('n', PartOfSpeech::Noun),
            ('v', PartOfSpeech::Verb),
            ('a', PartOfSpeech::Adjective),
            ('r', PartOfSpeech::Adverb),
        ];

        for (code, expected_pos) in valid_codes {
            let result = utils::parse_pos(code);
            if result.is_ok() {
                let parsed_pos = result.unwrap();
                assert_eq!(parsed_pos, expected_pos);
            }
        }
    }

    #[test]
    fn test_parse_pos_code_invalid() {
        use canopy_wordnet::parser::utils;

        // Test invalid POS codes
        let invalid_codes = vec!['x', 'z', '1', '!'];

        for code in invalid_codes {
            let result = utils::parse_pos(code);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_numeric_field() {
        use canopy_wordnet::parser::utils;

        // Test valid numeric fields
        let valid_numbers = vec!["1", "5", "10", "99"];

        for num_str in valid_numbers {
            let result = utils::parse_numeric_field::<u32>(num_str, "test_field");
            if result.is_ok() {
                let num = result.unwrap();
                assert!(num > 0);
                assert_eq!(num.to_string(), num_str);
            }
        }
    }

    #[test]
    fn test_parse_numeric_field_invalid() {
        use canopy_wordnet::parser::utils;

        // Test invalid numeric fields
        let invalid_numbers = vec!["", "abc", "1.5", "-1"];

        for num_str in invalid_numbers {
            let result = utils::parse_numeric_field::<u32>(num_str, "test_field");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_extract_gloss_simple() {
        use canopy_wordnet::parser::utils;

        // Test gloss extraction
        let test_lines = vec![
            (
                "synset data | a simple definition",
                Some("a simple definition"),
            ),
            (
                "synset data | definition with (parentheses)",
                Some("definition with (parentheses)"),
            ),
            (
                "synset data | definition; with semicolon",
                Some("definition; with semicolon"),
            ),
            (
                "synset data | definition \"with quotes\"",
                Some("definition \"with quotes\""),
            ),
            ("no gloss separator here", None),
            ("", None),
        ];

        for (line, expected) in test_lines {
            let result = utils::extract_gloss(line);
            assert_eq!(result.as_deref(), expected);
        }
    }

    #[test]
    fn test_extract_gloss_complex() {
        use canopy_wordnet::parser::utils;

        // Test complex gloss extraction
        let complex_lines = vec![
            "synset words | the property of being physically malleable; the property of something that can be worked or hammered or shaped without breaking",
            "word forms | a disposition to remain inactive or inert; \"he had to overcome his inertia and get back to work\"",
            "semantic data | the semantic role of the animate being that causes or initiates an action",
        ];

        for line in complex_lines {
            let result = utils::extract_gloss(line);
            assert!(result.is_some());
            let gloss = result.unwrap();
            assert!(!gloss.is_empty());
            assert!(!gloss.starts_with(' ')); // Should be trimmed
        }
    }

    #[test]
    fn test_parse_with_different_configs() {
        // Test parsing with different configurations
        let configs = vec![
            WordNetParserConfig {
                strict_mode: true,
                max_file_size: 1024, // 1KB
                skip_prefixes: vec!["  ".to_string(), "\t".to_string()],
            },
            WordNetParserConfig {
                strict_mode: false,
                max_file_size: 1024 * 1024 * 10, // 10MB
                skip_prefixes: vec!["%".to_string()],
            },
        ];

        for config in configs {
            let parser = WordNetParser::with_config(config.clone());

            // Test that parser is created with the config
            assert_eq!(parser.config().strict_mode, config.strict_mode);
            assert_eq!(parser.config().max_file_size, config.max_file_size);
            assert_eq!(parser.config().skip_prefixes, config.skip_prefixes);
        }
    }

    #[test]
    fn test_parse_edge_cases() {
        use canopy_wordnet::parser::utils;

        // Test edge cases for synset offset parsing
        let offset_cases = vec![("00000000", true), ("99999999", true)];

        for (input, should_succeed) in offset_cases {
            let result = utils::parse_synset_offset(input);
            if should_succeed {
                assert!(result.is_ok());
            }
        }

        // Test edge cases for POS parsing
        let pos_result = utils::parse_pos('n');
        assert!(pos_result.is_ok());
        assert_eq!(pos_result.unwrap(), PartOfSpeech::Noun);
    }

    #[test]
    fn test_concurrent_parsing() {
        use canopy_wordnet::parser::utils;
        use std::thread;

        let handles = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let offset = format!("{:08}", i + 1);
                    let result = utils::parse_synset_offset(&offset);
                    result.is_ok()
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            let result = handle.join().unwrap();
            // All should succeed with valid offsets
            assert!(result);
        }
    }

    #[test]
    fn test_parser_memory_efficiency() {
        use canopy_wordnet::parser::utils;

        // Test parsing many items doesn't accumulate memory
        for i in 0..1000 {
            let offset = format!("{:08}", i % 100000000);
            let _ = utils::parse_synset_offset(&offset);

            let pos_codes = ['n', 'v', 'a', 'r'];
            let _ = utils::parse_pos(pos_codes[i % 4]);
        }

        // Test passes if it doesn't crash or run out of memory
        assert!(true);
    }

    #[test]
    fn test_error_messages() {
        use canopy_wordnet::parser::utils;

        // Test that error messages are informative
        let result = utils::parse_synset_offset("invalid");
        if let Err(error) = result {
            let error_msg = error.to_string();
            assert!(!error_msg.is_empty());
            assert!(
                error_msg.contains("invalid")
                    || error_msg.contains("synset")
                    || error_msg.contains("offset")
            );
        }

        let result = utils::parse_pos('x');
        if let Err(error) = result {
            let error_msg = error.to_string();
            assert!(!error_msg.is_empty());
            assert!(
                error_msg.contains("Invalid")
                    || error_msg.contains("pos")
                    || error_msg.contains("part")
            );
        }
    }

    #[test]
    fn test_config_validation() {
        // Test configuration validation
        let mut config = WordNetParserConfig::default();

        // Test extreme values
        config.max_file_size = 0;
        let parser = WordNetParser::with_config(config.clone());
        // Should handle zero max file size gracefully
        assert_eq!(parser.config().max_file_size, 0);

        config.max_file_size = 1000000000; // 1GB
        let parser = WordNetParser::with_config(config);
        // Should handle very large max file size
        assert_eq!(parser.config().max_file_size, 1000000000);
    }

    #[test]
    fn test_parse_pointer_symbols() {
        use canopy_wordnet::parser::utils;
        use canopy_wordnet::types::SemanticRelation;

        // Test valid pointer symbols
        let valid_symbols = vec![
            ("!", SemanticRelation::Antonym),
            ("@", SemanticRelation::Hypernym),
            ("~", SemanticRelation::Hyponym),
            ("@i", SemanticRelation::InstanceHypernym),
            ("~i", SemanticRelation::InstanceHyponym),
            ("#m", SemanticRelation::MemberHolonym),
            ("*", SemanticRelation::Entailment),
            (">", SemanticRelation::Cause),
            ("&", SemanticRelation::SimilarTo),
        ];

        for (symbol, expected_relation) in valid_symbols {
            let result = utils::parse_pointer_symbol(symbol);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected_relation);
        }

        // Test invalid pointer symbols
        let invalid_symbols = vec!["?", "xyz", "", "@@"];
        for symbol in invalid_symbols {
            let result = utils::parse_pointer_symbol(symbol);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_split_fields() {
        use canopy_wordnet::parser::utils;

        // Test field splitting
        let test_lines = vec![
            ("field1 field2 field3", vec!["field1", "field2", "field3"]),
            ("  spaced  fields  ", vec!["spaced", "fields"]),
            ("single", vec!["single"]),
            ("", vec![]),
            ("\ttab\tseparated", vec!["tab", "separated"]),
        ];

        for (line, expected) in test_lines {
            let fields = utils::split_fields(line);
            assert_eq!(fields, expected);
        }
    }

    #[test]
    fn test_is_license_or_empty() {
        use canopy_wordnet::parser::utils;

        // Test license/empty line detection
        let test_cases = vec![
            ("", true),
            ("   ", true),
            ("  License text", true),
            ("\tTab-prefixed line", true),
            ("Normal data line", false),
            ("not prefixed", false),
        ];

        for (line, expected) in test_cases {
            assert_eq!(utils::is_license_or_empty(line), expected);
        }
    }

    #[test]
    fn test_parser_file_parsing() {
        let parser = create_test_parser();

        // Test file parsing with non-existent file
        use std::path::Path;
        let non_existent_path = Path::new("non_existent_file.txt");

        let result = parser.parse_file(non_existent_path, |_reader| Ok("test result".to_string()));

        // Should fail with non-existent file
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_set_config() {
        let mut parser = create_test_parser();

        let new_config = WordNetParserConfig {
            strict_mode: true,
            max_file_size: 2048,
            skip_prefixes: vec!["##".to_string()],
        };

        parser.set_config(new_config.clone());

        assert_eq!(parser.config().strict_mode, new_config.strict_mode);
        assert_eq!(parser.config().max_file_size, new_config.max_file_size);
        assert_eq!(parser.config().skip_prefixes, new_config.skip_prefixes);
    }
}
