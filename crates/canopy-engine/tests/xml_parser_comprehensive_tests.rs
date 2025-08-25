//! Comprehensive tests for XML parser functionality

use canopy_engine::{
    xml_parser::{utils, XmlParser, XmlParserConfig, XmlResource},
    EngineError, EngineResult,
};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use std::fs;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use tempfile::{NamedTempFile, TempDir};

#[cfg(test)]
mod tests {
    use super::*;

    // Test resource for comprehensive testing
    #[derive(Debug, Clone, PartialEq)]
    struct TestResource {
        pub id: String,
        pub name: String,
        pub value: i32,
        pub optional_field: Option<String>,
    }

    impl XmlResource for TestResource {
        fn parse_xml<R: BufRead>(reader: &mut Reader<R>) -> EngineResult<Self> {
            let mut buf = Vec::new();
            let mut id = String::new();
            let mut name = String::new();
            let mut value = 0;
            let mut optional_field = None;

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) => match e.name() {
                        QName(b"id") => {
                            id = utils::extract_text_content(reader, &mut buf, b"id")?;
                        }
                        QName(b"name") => {
                            name = utils::extract_text_content(reader, &mut buf, b"name")?;
                        }
                        QName(b"value") => {
                            let value_str =
                                utils::extract_text_content(reader, &mut buf, b"value")?;
                            value = value_str.parse().map_err(|e| {
                                EngineError::data_load(format!("Invalid value: {}", e))
                            })?;
                        }
                        QName(b"optional") => {
                            optional_field =
                                Some(utils::extract_text_content(reader, &mut buf, b"optional")?);
                        }
                        _ => {}
                    },
                    Ok(Event::End(ref e)) if e.name() == QName(b"test") => {
                        break;
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(EngineError::data_load(format!("XML error: {}", e))),
                    _ => {}
                }
                buf.clear();
            }

            Ok(TestResource {
                id,
                name,
                value,
                optional_field,
            })
        }

        fn root_element() -> &'static str {
            "test"
        }

        fn validate(&self) -> EngineResult<()> {
            if self.id.is_empty() {
                return Err(EngineError::data_load("ID cannot be empty".to_string()));
            }
            if self.name.is_empty() {
                return Err(EngineError::data_load("Name cannot be empty".to_string()));
            }
            if self.value < 0 {
                return Err(EngineError::data_load(
                    "Value cannot be negative".to_string(),
                ));
            }
            Ok(())
        }
    }

    // XmlParserConfig Tests

    #[test]
    fn test_xml_parser_config_default() {
        let config = XmlParserConfig::default();

        assert!(!config.validate_schema);
        assert!(!config.strict_mode);
        assert_eq!(config.max_file_size, 50 * 1024 * 1024);
        assert!(config.expand_entities);
    }

    #[test]
    fn test_xml_parser_config_custom() {
        let config = XmlParserConfig {
            validate_schema: true,
            strict_mode: true,
            max_file_size: 1024,
            expand_entities: false,
        };

        assert!(config.validate_schema);
        assert!(config.strict_mode);
        assert_eq!(config.max_file_size, 1024);
        assert!(!config.expand_entities);
    }

    // XmlParser Tests

    #[test]
    fn test_xml_parser_new() {
        let parser = XmlParser::new();
        let config = parser.config();

        assert!(!config.validate_schema);
        assert!(!config.strict_mode);
        assert_eq!(config.max_file_size, 50 * 1024 * 1024);
        assert!(config.expand_entities);
    }

    #[test]
    fn test_xml_parser_default() {
        let parser1 = XmlParser::new();
        let parser2 = XmlParser::default();

        assert_eq!(
            parser1.config().validate_schema,
            parser2.config().validate_schema
        );
        assert_eq!(parser1.config().strict_mode, parser2.config().strict_mode);
        assert_eq!(
            parser1.config().max_file_size,
            parser2.config().max_file_size
        );
        assert_eq!(
            parser1.config().expand_entities,
            parser2.config().expand_entities
        );
    }

    #[test]
    fn test_xml_parser_with_config() {
        let config = XmlParserConfig {
            validate_schema: true,
            strict_mode: false,
            max_file_size: 2048,
            expand_entities: true,
        };

        let parser = XmlParser::with_config(config.clone());
        let parser_config = parser.config();

        assert_eq!(parser_config.validate_schema, config.validate_schema);
        assert_eq!(parser_config.strict_mode, config.strict_mode);
        assert_eq!(parser_config.max_file_size, config.max_file_size);
        assert_eq!(parser_config.expand_entities, config.expand_entities);
    }

    #[test]
    fn test_xml_parser_set_config() {
        let mut parser = XmlParser::new();

        let new_config = XmlParserConfig {
            validate_schema: true,
            strict_mode: true,
            max_file_size: 4096,
            expand_entities: false,
        };

        parser.set_config(new_config.clone());
        let parser_config = parser.config();

        assert_eq!(parser_config.validate_schema, new_config.validate_schema);
        assert_eq!(parser_config.strict_mode, new_config.strict_mode);
        assert_eq!(parser_config.max_file_size, new_config.max_file_size);
        assert_eq!(parser_config.expand_entities, new_config.expand_entities);
    }

    // File Parsing Tests

    #[test]
    fn test_parse_file_success() {
        let xml_content = r#"<?xml version="1.0"?>
        <test>
            <id>file-test-1</id>
            <name>File Test Resource</name>
            <value>100</value>
        </test>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let parser = XmlParser::new();
        let result: TestResource = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.id, "file-test-1");
        assert_eq!(result.name, "File Test Resource");
        assert_eq!(result.value, 100);
        assert_eq!(result.optional_field, None);
    }

    #[test]
    fn test_parse_file_with_optional_field() {
        let xml_content = r#"<?xml version="1.0"?>
        <test>
            <id>optional-test</id>
            <name>Optional Field Test</name>
            <value>42</value>
            <optional>Extra data</optional>
        </test>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let parser = XmlParser::new();
        let result: TestResource = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.id, "optional-test");
        assert_eq!(result.name, "Optional Field Test");
        assert_eq!(result.value, 42);
        assert_eq!(result.optional_field, Some("Extra data".to_string()));
    }

    #[test]
    fn test_parse_file_not_found() {
        let parser = XmlParser::new();
        let result: Result<TestResource, _> = parser.parse_file(Path::new("/nonexistent/file.xml"));

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read file metadata"));
    }

    #[test]
    fn test_parse_file_too_large() {
        let xml_content =
            "<?xml version=\"1.0\"?><test><id>big</id><name>Big File</name><value>1</value></test>";

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = XmlParserConfig {
            max_file_size: 10, // Very small limit
            ..Default::default()
        };

        let parser = XmlParser::with_config(config);
        let result: Result<TestResource, _> = parser.parse_file(temp_file.path());

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("File") && error_msg.contains("too large"));
    }

    // Directory Parsing Tests

    #[test]
    fn test_parse_directory_success() {
        let temp_dir = TempDir::new().unwrap();

        // Create test XML files
        let xml1 = r#"<?xml version="1.0"?><test><id>dir-1</id><name>Dir Test 1</name><value>10</value></test>"#;
        let xml2 = r#"<?xml version="1.0"?><test><id>dir-2</id><name>Dir Test 2</name><value>20</value></test>"#;

        fs::write(temp_dir.path().join("test1.xml"), xml1).unwrap();
        fs::write(temp_dir.path().join("test2.xml"), xml2).unwrap();

        let parser = XmlParser::new();
        let results: Vec<TestResource> = parser.parse_directory(temp_dir.path()).unwrap();

        assert_eq!(results.len(), 2);

        // Results could be in any order
        let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"dir-1"));
        assert!(ids.contains(&"dir-2"));
    }

    #[test]
    fn test_parse_directory_with_non_xml_files() {
        let temp_dir = TempDir::new().unwrap();

        let xml_content = r#"<?xml version="1.0"?><test><id>mixed-1</id><name>Mixed Test</name><value>5</value></test>"#;

        fs::write(temp_dir.path().join("test.xml"), xml_content).unwrap();
        fs::write(temp_dir.path().join("readme.txt"), "This is not XML").unwrap();
        fs::write(temp_dir.path().join("config.json"), "{}").unwrap();

        let parser = XmlParser::new();
        let results: Vec<TestResource> = parser.parse_directory(temp_dir.path()).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "mixed-1");
    }

    #[test]
    fn test_parse_directory_not_directory() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not a directory").unwrap();

        let parser = XmlParser::new();
        let result: Result<Vec<TestResource>, _> = parser.parse_directory(temp_file.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("is not a directory"));
    }

    #[test]
    fn test_parse_directory_no_xml_files() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("readme.txt"), "No XML here").unwrap();
        fs::write(temp_dir.path().join("data.json"), "{}").unwrap();

        let parser = XmlParser::new();
        let result: Result<Vec<TestResource>, _> = parser.parse_directory(temp_dir.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No valid XML files found"));
    }

    #[test]
    fn test_parse_directory_strict_mode() {
        let temp_dir = TempDir::new().unwrap();

        let good_xml =
            r#"<?xml version="1.0"?><test><id>good</id><name>Good</name><value>1</value></test>"#;
        let bad_xml = r#"<?xml version="1.0"?><test><id>bad</id><name>Bad</name><value>invalid</value></test>"#;

        fs::write(temp_dir.path().join("good.xml"), good_xml).unwrap();
        fs::write(temp_dir.path().join("bad.xml"), bad_xml).unwrap();

        let config = XmlParserConfig {
            strict_mode: true,
            ..Default::default()
        };

        let parser = XmlParser::with_config(config);
        let result: Result<Vec<TestResource>, _> = parser.parse_directory(temp_dir.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid value"));
    }

    // Pattern Parsing Tests

    #[test]
    fn test_parse_pattern_success() {
        let temp_dir = TempDir::new().unwrap();

        let xml1 = r#"<?xml version="1.0"?><test><id>pattern-1</id><name>Pattern Test 1</name><value>1</value></test>"#;
        let xml2 = r#"<?xml version="1.0"?><test><id>pattern-2</id><name>Pattern Test 2</name><value>2</value></test>"#;
        let xml3 = r#"<?xml version="1.0"?><test><id>other-3</id><name>Other Test 3</name><value>3</value></test>"#;

        fs::write(temp_dir.path().join("pattern_test1.xml"), xml1).unwrap();
        fs::write(temp_dir.path().join("pattern_test2.xml"), xml2).unwrap();
        fs::write(temp_dir.path().join("other_test3.xml"), xml3).unwrap();

        let parser = XmlParser::new();
        let results: Vec<TestResource> = parser.parse_pattern(temp_dir.path(), "pattern").unwrap();

        assert_eq!(results.len(), 2);

        let ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"pattern-1"));
        assert!(ids.contains(&"pattern-2"));
        assert!(!ids.contains(&"other-3"));
    }

    #[test]
    fn test_parse_pattern_no_matches() {
        let temp_dir = TempDir::new().unwrap();

        let xml_content = r#"<?xml version="1.0"?><test><id>nomatch</id><name>No Match</name><value>1</value></test>"#;
        fs::write(temp_dir.path().join("test.xml"), xml_content).unwrap();

        let parser = XmlParser::new();
        let results: Vec<TestResource> = parser.parse_pattern(temp_dir.path(), "pattern").unwrap();

        assert_eq!(results.len(), 0);
    }

    // Validation Tests

    #[test]
    fn test_parse_with_validation_success() {
        let temp_dir = TempDir::new().unwrap();

        let xml_content = r#"<?xml version="1.0"?><test><id>valid</id><name>Valid Resource</name><value>42</value></test>"#;
        fs::write(temp_dir.path().join("valid.xml"), xml_content).unwrap();

        let config = XmlParserConfig {
            validate_schema: true,
            strict_mode: false,
            ..Default::default()
        };

        let parser = XmlParser::with_config(config);
        let results: Vec<TestResource> = parser.parse_directory(temp_dir.path()).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "valid");
    }

    #[test]
    fn test_parse_with_validation_failure_non_strict() {
        let temp_dir = TempDir::new().unwrap();

        let invalid_xml =
            r#"<?xml version="1.0"?><test><id></id><name>Empty ID</name><value>1</value></test>"#;
        fs::write(temp_dir.path().join("invalid.xml"), invalid_xml).unwrap();

        let config = XmlParserConfig {
            validate_schema: true,
            strict_mode: false,
            ..Default::default()
        };

        let parser = XmlParser::with_config(config);
        let results: Vec<TestResource> = parser.parse_directory(temp_dir.path()).unwrap();

        // In non-strict mode, invalid resources should still be included
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_parse_with_validation_failure_strict() {
        let temp_dir = TempDir::new().unwrap();

        let invalid_xml =
            r#"<?xml version="1.0"?><test><id></id><name>Empty ID</name><value>1</value></test>"#;
        fs::write(temp_dir.path().join("invalid.xml"), invalid_xml).unwrap();

        let config = XmlParserConfig {
            validate_schema: true,
            strict_mode: true,
            ..Default::default()
        };

        let parser = XmlParser::with_config(config);
        let result: Result<Vec<TestResource>, _> = parser.parse_directory(temp_dir.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ID cannot be empty"));
    }

    // Utility Functions Tests

    #[test]
    fn test_extract_text_content_basic() {
        let xml = r#"<element>Simple text</element>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        reader.read_event_into(&mut buf).unwrap(); // Skip to start
        let content = utils::extract_text_content(&mut reader, &mut buf, b"element").unwrap();
        assert_eq!(content, "Simple text");
    }

    #[test]
    fn test_extract_text_content_with_whitespace() {
        let xml = r#"<element>  Trimmed text  </element>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        reader.read_event_into(&mut buf).unwrap();
        let content = utils::extract_text_content(&mut reader, &mut buf, b"element").unwrap();
        assert_eq!(content, "Trimmed text");
    }

    #[test]
    fn test_extract_text_content_empty() {
        let xml = r#"<element></element>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        reader.read_event_into(&mut buf).unwrap();
        let content = utils::extract_text_content(&mut reader, &mut buf, b"element").unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_extract_text_content_unexpected_eof() {
        let xml = r#"<element>Incomplete"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        reader.read_event_into(&mut buf).unwrap();
        let result = utils::extract_text_content(&mut reader, &mut buf, b"element");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unexpected end of file"));
    }

    #[test]
    fn test_get_attribute_success() {
        let xml = r#"<element id="123" name="test" flag="true"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(start)) | Ok(Event::Empty(start)) => {
                assert_eq!(utils::get_attribute(&start, "id"), Some("123".to_string()));
                assert_eq!(
                    utils::get_attribute(&start, "name"),
                    Some("test".to_string())
                );
                assert_eq!(
                    utils::get_attribute(&start, "flag"),
                    Some("true".to_string())
                );
                assert_eq!(utils::get_attribute(&start, "missing"), None);
            }
            _ => panic!("Expected start or empty event"),
        }
    }

    #[test]
    fn test_get_attribute_no_attributes() {
        let xml = r#"<element/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(start)) | Ok(Event::Empty(start)) => {
                assert_eq!(utils::get_attribute(&start, "any"), None);
            }
            _ => panic!("Expected start or empty event"),
        }
    }

    #[test]
    fn test_skip_element_simple() {
        let xml = r#"<root><skip>content to skip</skip><after>after skip</after></root>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        // Skip to root
        reader.read_event_into(&mut buf).unwrap();
        // Skip to skip element
        reader.read_event_into(&mut buf).unwrap();

        utils::skip_element(&mut reader, &mut buf, b"skip").unwrap();

        // Next event should be the start of 'after' element
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => assert_eq!(e.name(), QName(b"after")),
            _ => panic!("Expected start event for 'after' element"),
        }
    }

    #[test]
    fn test_skip_element_nested() {
        let xml = r#"<root><outer><inner><deep>content</deep></inner></outer><after>after skip</after></root>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        // Skip to root, then to outer
        reader.read_event_into(&mut buf).unwrap();
        reader.read_event_into(&mut buf).unwrap();

        utils::skip_element(&mut reader, &mut buf, b"outer").unwrap();

        // Next event should be the start of 'after' element
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => assert_eq!(e.name(), QName(b"after")),
            _ => panic!("Expected start event for 'after' element"),
        }
    }

    #[test]
    fn test_skip_element_unexpected_eof() {
        let xml = r#"<element>incomplete"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        reader.read_event_into(&mut buf).unwrap();
        let result = utils::skip_element(&mut reader, &mut buf, b"element");

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unexpected end of file"));
    }

    // Advanced XML Parsing Tests

    #[test]
    fn test_parse_malformed_xml() {
        let xml_content = r#"<?xml version="1.0"?><test><id>malformed<name>Missing close tag<value>1</value></test>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let parser = XmlParser::new();
        let result: Result<TestResource, _> = parser.parse_file(temp_file.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse XML file"));
    }

    #[test]
    fn test_parse_xml_with_entities() {
        let xml_content = r#"<?xml version="1.0"?>
        <test>
            <id>entity-test</id>
            <name>Test &amp; Resource with &lt;entities&gt;</name>
            <value>42</value>
        </test>"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(xml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let parser = XmlParser::new();
        let result: TestResource = parser.parse_file(temp_file.path()).unwrap();

        assert_eq!(result.id, "entity-test");
        assert_eq!(result.name, "Test & Resource with <entities>");
        assert_eq!(result.value, 42);
    }

    // XmlResource trait tests

    #[test]
    fn test_xml_resource_root_element() {
        assert_eq!(TestResource::root_element(), "test");
    }

    #[test]
    fn test_xml_resource_validation_success() {
        let resource = TestResource {
            id: "valid-id".to_string(),
            name: "Valid Name".to_string(),
            value: 42,
            optional_field: None,
        };

        assert!(resource.validate().is_ok());
    }

    #[test]
    fn test_xml_resource_validation_empty_id() {
        let resource = TestResource {
            id: "".to_string(),
            name: "Valid Name".to_string(),
            value: 42,
            optional_field: None,
        };

        let result = resource.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ID cannot be empty"));
    }

    #[test]
    fn test_xml_resource_validation_empty_name() {
        let resource = TestResource {
            id: "valid-id".to_string(),
            name: "".to_string(),
            value: 42,
            optional_field: None,
        };

        let result = resource.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Name cannot be empty"));
    }

    #[test]
    fn test_xml_resource_validation_negative_value() {
        let resource = TestResource {
            id: "valid-id".to_string(),
            name: "Valid Name".to_string(),
            value: -1,
            optional_field: None,
        };

        let result = resource.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Value cannot be negative"));
    }

    // Edge Case Tests

    #[test]
    fn test_parse_directory_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let parser = XmlParser::new();
        let result: Result<Vec<TestResource>, _> = parser.parse_directory(temp_dir.path());

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No valid XML files found"));
    }

    #[test]
    fn test_parse_pattern_empty_pattern() {
        let temp_dir = TempDir::new().unwrap();

        let xml_content =
            r#"<?xml version="1.0"?><test><id>any</id><name>Any</name><value>1</value></test>"#;
        fs::write(temp_dir.path().join("test.xml"), xml_content).unwrap();

        let parser = XmlParser::new();
        let results: Vec<TestResource> = parser.parse_pattern(temp_dir.path(), "").unwrap();

        // Empty pattern should match all files
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_configuration_immutability() {
        let original_config = XmlParserConfig {
            validate_schema: true,
            strict_mode: true,
            max_file_size: 1024,
            expand_entities: false,
        };

        let parser = XmlParser::with_config(original_config.clone());
        let config = parser.config();

        // Config should be readable but we can't modify it through the reference
        assert_eq!(config.validate_schema, original_config.validate_schema);
        assert_eq!(config.strict_mode, original_config.strict_mode);
        assert_eq!(config.max_file_size, original_config.max_file_size);
        assert_eq!(config.expand_entities, original_config.expand_entities);
    }
}
