//! XML parsing infrastructure for semantic engines
//!
//! This module provides shared XML parsing capabilities that linguistic resource engines
//! can use to parse their data files. It supports VerbNet, FrameNet, and other XML-based
//! linguistic resources.

use crate::{EngineError, EngineResult};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use std::io::BufRead;
use std::path::Path;

/// Trait for types that can be parsed from XML files
pub trait XmlResource: Sized {
    /// Parse this resource from an XML file
    fn parse_xml<R: BufRead>(reader: &mut Reader<R>) -> EngineResult<Self>;

    /// Validate the parsed resource
    fn validate(&self) -> EngineResult<()> {
        // Default implementation: no validation
        Ok(())
    }

    /// Get the expected root element name
    fn root_element() -> &'static str;
}

/// Configuration for XML parsing
#[derive(Debug, Clone)]
pub struct XmlParserConfig {
    /// Whether to validate XML against schema
    pub validate_schema: bool,
    /// Whether to use strict parsing (fail on any error)
    pub strict_mode: bool,
    /// Maximum file size to parse (bytes)
    pub max_file_size: usize,
    /// Whether to expand entities
    pub expand_entities: bool,
}

impl Default for XmlParserConfig {
    fn default() -> Self {
        Self {
            validate_schema: false,
            strict_mode: false,
            max_file_size: 50 * 1024 * 1024, // 50MB
            expand_entities: true,
        }
    }
}

/// Generic XML parser for linguistic resources
pub struct XmlParser {
    config: XmlParserConfig,
}

impl XmlParser {
    /// Create a new XML parser with default configuration
    pub fn new() -> Self {
        Self {
            config: XmlParserConfig::default(),
        }
    }

    /// Create a new XML parser with custom configuration
    pub fn with_config(config: XmlParserConfig) -> Self {
        Self { config }
    }

    /// Parse a single XML file into a resource
    pub fn parse_file<T: XmlResource>(&self, path: &Path) -> EngineResult<T> {
        // Check file size
        let metadata = std::fs::metadata(path).map_err(|e| {
            EngineError::data_load(format!(
                "Failed to read file metadata for {}: {}",
                path.display(),
                e
            ))
        })?;

        if metadata.len() > self.config.max_file_size as u64 {
            return Err(EngineError::data_load(format!(
                "File {} too large: {} bytes (max: {})",
                path.display(),
                metadata.len(),
                self.config.max_file_size
            )));
        }

        // Open and parse the file
        let mut reader = Reader::from_file(path).map_err(|e| {
            EngineError::data_load(format!("Failed to open XML file {}: {}", path.display(), e))
        })?;

        // Configure reader
        reader.expand_empty_elements(true);

        T::parse_xml(&mut reader).map_err(|e| {
            EngineError::data_load(format!(
                "Failed to parse XML file {}: {}",
                path.display(),
                e
            ))
        })
    }

    /// Parse all XML files in a directory
    pub fn parse_directory<T: XmlResource>(&self, dir_path: &Path) -> EngineResult<Vec<T>> {
        if !dir_path.is_dir() {
            return Err(EngineError::data_load(format!(
                "Path {} is not a directory",
                dir_path.display()
            )));
        }

        let mut resources = Vec::new();
        let entries = std::fs::read_dir(dir_path).map_err(|e| {
            EngineError::data_load(format!(
                "Failed to read directory {}: {}",
                dir_path.display(),
                e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                EngineError::data_load(format!("Failed to read directory entry: {e}"))
            })?;

            let path = entry.path();

            // Only process XML files
            if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                match self.parse_file::<T>(&path) {
                    Ok(resource) => {
                        if self.config.validate_schema {
                            if let Err(e) = resource.validate() {
                                if self.config.strict_mode {
                                    return Err(e);
                                } else {
                                    eprintln!(
                                        "Warning: Validation failed for {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                        resources.push(resource);
                    }
                    Err(e) => {
                        if self.config.strict_mode {
                            return Err(e);
                        } else {
                            eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        if resources.is_empty() {
            return Err(EngineError::data_load(format!(
                "No valid XML files found in directory {}",
                dir_path.display()
            )));
        }

        Ok(resources)
    }

    /// Parse all XML files in a directory in parallel (if rayon feature is enabled)
    #[cfg(feature = "parallel")]
    pub fn parse_directory_parallel<T>(&self, dir_path: &Path) -> EngineResult<Vec<T>>
    where
        T: XmlResource + Send + Sync,
    {
        use rayon::prelude::*;

        if !dir_path.is_dir() {
            return Err(EngineError::data_load(format!(
                "Path {} is not a directory",
                dir_path.display()
            )));
        }

        // Collect all XML file paths first
        let xml_paths: Vec<_> = std::fs::read_dir(dir_path)
            .map_err(|e| {
                EngineError::data_load(format!(
                    "Failed to read directory {}: {}",
                    dir_path.display(),
                    e
                ))
            })?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                        Some(path)
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Parse files in parallel
        let results: Result<Vec<_>, _> = xml_paths
            .into_par_iter()
            .map(|path| {
                match self.parse_file::<T>(&path) {
                    Ok(resource) => {
                        if self.config.validate_schema {
                            if let Err(e) = resource.validate() {
                                if self.config.strict_mode {
                                    return Err(e);
                                } else {
                                    eprintln!(
                                        "Warning: Validation failed for {}: {}",
                                        path.display(),
                                        e
                                    );
                                }
                            }
                        }
                        Ok(resource)
                    }
                    Err(e) => {
                        if self.config.strict_mode {
                            Err(e)
                        } else {
                            eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                            // In non-strict mode, we skip failed files
                            Err(e)
                        }
                    }
                }
            })
            .collect();

        match results {
            Ok(resources) => {
                if resources.is_empty() {
                    Err(EngineError::data_load(format!(
                        "No valid XML files found in directory {}",
                        dir_path.display()
                    )))
                } else {
                    Ok(resources)
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Parse XML files matching a pattern
    pub fn parse_pattern<T: XmlResource>(
        &self,
        dir_path: &Path,
        pattern: &str,
    ) -> EngineResult<Vec<T>> {
        if !dir_path.is_dir() {
            return Err(EngineError::data_load(format!(
                "Path {} is not a directory",
                dir_path.display()
            )));
        }

        let mut resources = Vec::new();
        let entries = std::fs::read_dir(dir_path).map_err(|e| {
            EngineError::data_load(format!(
                "Failed to read directory {}: {}",
                dir_path.display(),
                e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                EngineError::data_load(format!("Failed to read directory entry: {e}"))
            })?;

            let path = entry.path();

            // Check if filename matches pattern and is XML
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.contains(pattern) && filename.ends_with(".xml") {
                    match self.parse_file::<T>(&path) {
                        Ok(resource) => resources.push(resource),
                        Err(e) => {
                            if self.config.strict_mode {
                                return Err(e);
                            } else {
                                eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        Ok(resources)
    }

    /// Get parser configuration
    pub fn config(&self) -> &XmlParserConfig {
        &self.config
    }

    /// Update parser configuration
    pub fn set_config(&mut self, config: XmlParserConfig) {
        self.config = config;
    }
}

impl Default for XmlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for XML parsing
pub mod utils {
    use super::*;

    /// Extract text content from an XML element
    pub fn extract_text_content<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        end_tag: &[u8],
    ) -> EngineResult<String> {
        let mut content = String::new();

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Text(e)) => {
                    let text = e.unescape().map_err(|e| {
                        EngineError::data_load(format!("Failed to decode text: {e}"))
                    })?;
                    content.push_str(&text);
                }
                Ok(Event::End(e)) if e.name() == QName(end_tag) => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(EngineError::data_load(
                        "Unexpected end of file while reading text content".to_string(),
                    ));
                }
                Err(e) => {
                    return Err(EngineError::data_load(format!("XML parsing error: {e}")));
                }
                _ => {} // Skip other events
            }
            buf.clear();
        }

        Ok(content.trim().to_string())
    }

    /// Extract attribute value from an XML start tag
    pub fn get_attribute(
        start_tag: &quick_xml::events::BytesStart,
        attr_name: &str,
    ) -> Option<String> {
        start_tag.attributes().find_map(|attr| {
            if let Ok(attr) = attr {
                if attr.key == QName(attr_name.as_bytes()) {
                    String::from_utf8(attr.value.to_vec()).ok()
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// Skip to the end of the current element
    pub fn skip_element<R: BufRead>(
        reader: &mut Reader<R>,
        buf: &mut Vec<u8>,
        element_name: &[u8],
    ) -> EngineResult<()> {
        let mut depth = 1;

        loop {
            match reader.read_event_into(buf) {
                Ok(Event::Start(e)) if e.name() == QName(element_name) => {
                    depth += 1;
                }
                Ok(Event::End(e)) if e.name() == QName(element_name) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Ok(Event::Eof) => {
                    return Err(EngineError::data_load(
                        "Unexpected end of file while skipping element".to_string(),
                    ));
                }
                Err(e) => {
                    return Err(EngineError::data_load(format!("XML parsing error: {e}")));
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test resource for parsing
    #[derive(Debug, PartialEq)]
    struct TestResource {
        id: String,
        name: String,
        value: i32,
    }

    impl XmlResource for TestResource {
        fn parse_xml<R: BufRead>(reader: &mut Reader<R>) -> EngineResult<Self> {
            let mut buf = Vec::new();
            let mut id = String::new();
            let mut name = String::new();
            let mut value = 0;

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
                                EngineError::data_load(format!("Invalid value: {e}"))
                            })?;
                        }
                        _ => {}
                    },
                    Ok(Event::End(ref e)) if e.name() == QName(b"test") => {
                        break;
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(EngineError::data_load(format!("XML error: {e}"))),
                    _ => {}
                }
                buf.clear();
            }

            Ok(TestResource { id, name, value })
        }

        fn root_element() -> &'static str {
            "test"
        }
    }

    #[test]
    fn test_xml_parsing() {
        let xml = r#"<?xml version="1.0"?>
        <test>
            <id>test-1</id>
            <name>Test Resource</name>
            <value>42</value>
        </test>"#;

        let mut reader = Reader::from_str(xml);
        let resource = TestResource::parse_xml(&mut reader).unwrap();

        assert_eq!(resource.id, "test-1");
        assert_eq!(resource.name, "Test Resource");
        assert_eq!(resource.value, 42);
    }

    #[test]
    fn test_extract_text_content() {
        let xml = r#"<element>Some text content</element>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        // Skip to start of element
        reader.read_event_into(&mut buf).unwrap();

        let content = utils::extract_text_content(&mut reader, &mut buf, b"element").unwrap();
        assert_eq!(content, "Some text content");
    }

    #[test]
    fn test_get_attribute() {
        let xml = r#"<element id="test-id" name="test-name"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(start)) | Ok(Event::Empty(start)) => {
                assert_eq!(
                    utils::get_attribute(&start, "id"),
                    Some("test-id".to_string())
                );
                assert_eq!(
                    utils::get_attribute(&start, "name"),
                    Some("test-name".to_string())
                );
                assert_eq!(utils::get_attribute(&start, "missing"), None);
            }
            _ => panic!("Expected start or empty event"),
        }
    }
}
