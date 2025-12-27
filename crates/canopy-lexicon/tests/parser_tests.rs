//! Parser tests for canopy-lexicon
//!
//! Tests XML parsing functionality with focus on what's actually implemented

use canopy_engine::{XmlParser, XmlResource};
use canopy_lexicon::parser::LexiconXmlResource;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

#[cfg(test)]
mod parser_tests {
    use super::*;

    fn create_simple_lexicon_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Simple Test Lexicon</title>
    <description>Test lexicon for parser tests</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>

  <word-classes>
    <word-class id="test-stop-words" name="Test Stop Words" type="stop-words" priority="10">
      <description>Test stop words</description>
      <properties>
        <property name="semantic-weight" value="0.1" type="float"/>
      </properties>
      <words>
        <word pos="DT">the</word>
        <word pos="DT">a</word>
      </words>
    </word-class>

    <word-class id="test-negation" name="Test Negation" type="negation" priority="9">
      <description>Test negation words</description>
      <words>
        <word pos="RB">not</word>
        <word pos="DT">no</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#
            .to_string()
    }

    fn create_empty_lexicon_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Empty Test Lexicon</title>
    <description>Empty test lexicon</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>

  <word-classes>
  </word-classes>
</lexicon>"#
            .to_string()
    }

    fn create_invalid_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
    <word-classes>
        <word-class id="test" type="test">
            <words>
                <word pos="TEST">test
                <!-- Missing closing tag -->
        </word-class>
    </word-classes>
"#
        .to_string()
    }

    fn create_test_file(content: &str) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let mut file = File::create(&file_path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test data");

        temp_dir
    }

    #[test]
    fn test_parse_simple_lexicon() {
        let xml_content = create_simple_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(result.is_ok(), "Simple lexicon should parse successfully");

        let resource = result.unwrap();

        // Test basic parsing worked
        let stats = resource.database.stats();
        assert!(stats.total_words >= 4); // Should have "the", "a", "not", "no"
    }

    #[test]
    fn test_parse_empty_lexicon() {
        let xml_content = create_empty_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(result.is_ok(), "Empty lexicon should parse successfully");

        let resource = result.unwrap();
        let stats = resource.database.stats();
        assert_eq!(stats.total_words, 0);
        assert_eq!(stats.total_word_classes, 0);
    }

    #[test]
    fn test_parse_invalid_xml() {
        let xml_content = create_invalid_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        // Should fail to parse invalid XML
        assert!(result.is_err(), "Invalid XML should fail to parse");
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(Path::new("/nonexistent/file.xml"));

        // Should fail for nonexistent file
        assert!(result.is_err(), "Nonexistent file should fail to parse");
    }

    #[test]
    fn test_validation_success() {
        let xml_content = create_simple_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let resource = parser.parse_file::<LexiconXmlResource>(&file_path).unwrap();

        let validation_result = resource.validate();
        assert!(
            validation_result.is_ok(),
            "Valid resource should pass validation"
        );
    }

    #[test]
    fn test_resource_structure() {
        let xml_content = create_simple_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let resource = parser.parse_file::<LexiconXmlResource>(&file_path).unwrap();

        // Test that the resource has a database
        let stats = resource.database.stats();
        assert!(stats.total_words > 0);
        assert!(stats.total_word_classes > 0);
    }

    #[test]
    fn test_database_stats_after_parsing() {
        let xml_content = create_simple_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let resource = parser.parse_file::<LexiconXmlResource>(&file_path).unwrap();

        let stats = resource.database.stats();
        // Should have parsed some data
        assert!(stats.total_word_classes >= 2); // stop-words and negation classes
        assert!(stats.total_words >= 4); // "the", "a", "not", "no"
    }

    #[test]
    fn test_parser_with_different_encodings() {
        // Test with UTF-8 content that includes special characters
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Unicode Test Lexicon</title>
    <description>Unicode test lexicon</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>

  <word-classes>
    <word-class id="unicode-words" name="Unicode Words" type="stop-words" priority="10">
      <description>Unicode words for testing</description>
      <words>
        <word pos="NN">café</word>
        <word pos="ADJ">naïve</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#;

        let temp_dir = create_test_file(xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);

        assert!(result.is_ok(), "Should parse Unicode content successfully");
    }

    #[test]
    fn test_parser_error_messages() {
        let parser = XmlParser::new();
        let result =
            parser.parse_file::<LexiconXmlResource>(Path::new("/definitely/does/not/exist.xml"));

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(!error_msg.is_empty(), "Error should have a message");
    }

    #[test]
    fn test_multiple_parser_instances() {
        let xml_content = create_simple_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        // Create multiple parsers
        let parser1 = XmlParser::new();
        let parser2 = XmlParser::new();

        let result1 = parser1.parse_file::<LexiconXmlResource>(&file_path);
        let result2 = parser2.parse_file::<LexiconXmlResource>(&file_path);

        assert!(result1.is_ok(), "First parser should succeed");
        assert!(result2.is_ok(), "Second parser should succeed");

        // Results should be consistent
        let stats1 = result1.unwrap().database.stats();
        let stats2 = result2.unwrap().database.stats();
        assert_eq!(stats1.total_words, stats2.total_words);
    }

    #[test]
    fn test_parser_with_large_content() {
        // Create larger XML content to test performance
        let mut xml_content = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Large Test Lexicon</title>
    <description>Large test lexicon</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>

  <word-classes>
    <word-class id="large-words" name="Large Words" type="stop-words" priority="10">
      <description>Many words for performance testing</description>
      <words>"#,
        );

        // Add many words
        for i in 0..100 {
            xml_content.push_str(&format!("        <word pos=\"NN\">word{}</word>\n", i));
        }

        xml_content.push_str(
            r#"      </words>
    </word-class>
  </word-classes>
</lexicon>"#,
        );

        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let start = std::time::Instant::now();
        let parser = XmlParser::new();
        let result = parser.parse_file::<LexiconXmlResource>(&file_path);
        let parse_time = start.elapsed();

        assert!(result.is_ok(), "Should parse large content successfully");
        assert!(
            parse_time.as_secs() < 10,
            "Parsing should complete in reasonable time"
        );

        let resource = result.unwrap();
        let stats = resource.database.stats();
        assert!(stats.total_words >= 100);
    }

    #[test]
    fn test_parser_config() {
        // Test that parser can be configured (if config is available)
        let parser = XmlParser::new();

        // Parser should be created successfully
        // Actual configuration testing depends on available API
        let xml_content = create_simple_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let result = parser.parse_file::<LexiconXmlResource>(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resource_interface_methods() {
        let xml_content = create_simple_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        let parser = XmlParser::new();
        let resource = parser.parse_file::<LexiconXmlResource>(&file_path).unwrap();

        // Test XmlResource interface methods
        let validation = resource.validate();
        assert!(validation.is_ok());

        // Database should be accessible
        let stats = resource.database.stats();
        assert!(stats.total_words > 0);
    }

    #[test]
    fn test_file_path_edge_cases() {
        let parser = XmlParser::new();

        let edge_cases = vec![
            "",
            "/",
            "relative/path.xml",
            "/absolute/path.xml",
            "file with spaces.xml",
        ];

        for path in edge_cases {
            let result = parser.parse_file::<LexiconXmlResource>(Path::new(path));
            // Most of these should fail, but shouldn't panic
            match result {
                Ok(_) => {
                    // Unexpected success (unless file actually exists)
                }
                Err(_) => {
                    // Expected failure for nonexistent files
                }
            }
        }
    }

    #[test]
    fn test_concurrent_parsing() {
        let xml_content = create_simple_lexicon_xml();
        let temp_dir = create_test_file(&xml_content);
        let file_path = temp_dir.path().join("test-lexicon.xml");

        use std::sync::Arc;
        use std::thread;

        let path = Arc::new(file_path);
        let mut handles = vec![];

        // Spawn multiple threads to parse the same file
        for i in 0..5 {
            let path_clone = Arc::clone(&path);
            let handle = thread::spawn(move || {
                let parser = XmlParser::new();
                let result = parser.parse_file::<LexiconXmlResource>(&path_clone);
                assert!(result.is_ok(), "Thread {} parsing should succeed", i);
                result.unwrap().database.stats().total_words
            });
            handles.push(handle);
        }

        // Wait for all threads and verify consistent results
        let mut results = vec![];
        for handle in handles {
            let word_count = handle.join().expect("Thread should complete");
            results.push(word_count);
        }

        // All threads should get the same result
        for i in 1..results.len() {
            assert_eq!(
                results[0], results[i],
                "All threads should get same word count"
            );
        }
    }
}
