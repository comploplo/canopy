//! XML parsing utilities for semantic engines

use super::error::{EngineError, EngineResult};
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Trait for types that can be parsed from XML resources
///
/// Implement this trait for data types that should be loadable from XML files.
pub trait XmlResource: Sized {
    /// Parse this resource from an XML file reader
    ///
    /// # Errors
    /// Returns an error if XML parsing fails.
    fn parse_xml<R: BufRead>(reader: &mut Reader<R>) -> EngineResult<Self>;

    /// Validate the parsed resource (default: no validation)
    ///
    /// # Errors
    /// Returns an error if validation fails.
    fn validate(&self) -> EngineResult<()> {
        Ok(())
    }

    /// Get the expected root element name
    fn root_element() -> &'static str;
}

/// Extract attribute value from XML element by name
#[must_use]
pub fn get_attribute(element: &BytesStart, attr_name: &str) -> Option<String> {
    element.attributes().find_map(|attr| {
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

/// Extract text content from an XML element until end tag
///
/// # Errors
/// Returns an error if XML parsing fails or EOF is encountered unexpectedly.
pub fn extract_text_content<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    end_tag: &[u8],
) -> EngineResult<String> {
    let mut content = String::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Text(e)) => {
                let text = e
                    .unescape()
                    .map_err(|e| EngineError::data_load(format!("Failed to decode text: {e}")))?;
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
            _ => {}
        }
        buf.clear();
    }

    Ok(content.trim().to_string())
}

/// Skip to the end of the current XML element
///
/// # Errors
/// Returns an error if XML parsing fails or EOF is encountered unexpectedly.
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

/// XML parser configuration
#[derive(Debug, Clone)]
pub struct XmlParserConfig {
    pub trim_text: bool,
    pub check_end_names: bool,
}

impl Default for XmlParserConfig {
    fn default() -> Self {
        Self {
            trim_text: true,
            check_end_names: true,
        }
    }
}

/// XML parser wrapper with common functionality
#[derive(Debug)]
pub struct XmlParser {
    pub config: XmlParserConfig,
}

impl XmlParser {
    /// Create a new XML parser with default config
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: XmlParserConfig::default(),
        }
    }

    /// Create a new XML parser with custom config
    #[must_use]
    pub fn with_config(config: XmlParserConfig) -> Self {
        Self { config }
    }

    /// Create a `quick_xml` Reader for a file
    ///
    /// # Errors
    /// Returns an error if the file cannot be opened.
    pub fn create_reader<P: AsRef<Path>>(&self, path: P) -> EngineResult<Reader<BufReader<File>>> {
        let file = File::open(path.as_ref()).map_err(|e| {
            EngineError::io(
                format!("Failed to open XML file: {}", path.as_ref().display()),
                e,
            )
        })?;

        let reader = BufReader::new(file);
        let mut xml_reader = Reader::from_reader(reader);

        xml_reader.config_mut().trim_text(self.config.trim_text);
        xml_reader.config_mut().check_end_names = self.config.check_end_names;

        Ok(xml_reader)
    }

    /// Parse an XML file into the specified type
    ///
    /// # Errors
    /// Returns an error if the file cannot be opened or parsed.
    pub fn parse_file<T: XmlResource>(&self, path: &Path) -> EngineResult<T> {
        let mut reader = self.create_reader(path)?;
        T::parse_xml(&mut reader)
    }

    /// Parse all XML files in a directory
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read.
    pub fn parse_directory<T: XmlResource + std::fmt::Debug>(
        &self,
        path: &Path,
    ) -> EngineResult<Vec<T>> {
        let mut results = Vec::new();

        for entry in std::fs::read_dir(path).map_err(|e| {
            EngineError::io(format!("Failed to read directory: {}", path.display()), e)
        })? {
            let entry =
                entry.map_err(|e| EngineError::io("Failed to read entry".to_string(), e))?;
            let file_path = entry.path();

            if file_path.extension().and_then(|s| s.to_str()) == Some("xml") {
                match self.parse_file::<T>(&file_path) {
                    Ok(item) => results.push(item),
                    Err(e) => {
                        tracing::warn!("Failed to parse {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Parse all XML files in a directory using parallel processing
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read.
    #[cfg(feature = "parallel")]
    pub fn parse_directory_parallel<T: XmlResource + std::fmt::Debug + Send>(
        &self,
        path: &Path,
    ) -> EngineResult<Vec<T>> {
        use rayon::prelude::*;

        let files: Vec<_> = std::fs::read_dir(path)
            .map_err(|e| EngineError::io(format!("Failed to read directory: {:?}", path), e))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("xml"))
            .collect();

        let results: Vec<T> = files
            .par_iter()
            .filter_map(|file_path| match self.parse_file::<T>(file_path) {
                Ok(item) => Some(item),
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", file_path, e);
                    None
                }
            })
            .collect();

        Ok(results)
    }
}

impl Default for XmlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an XML source (file or embedded data)
#[derive(Debug, Clone)]
pub struct XmlSource {
    pub path: Option<String>,
    pub content: Option<String>,
}

impl XmlSource {
    /// Create from a file path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: Some(path.as_ref().to_string_lossy().into_owned()),
            content: None,
        }
    }

    /// Create from embedded content
    #[must_use]
    pub fn from_content(content: String) -> Self {
        Self {
            path: None,
            content: Some(content),
        }
    }

    /// Read the XML content
    ///
    /// # Errors
    /// Returns an error if no source is specified or the file cannot be read.
    pub fn read(&self) -> EngineResult<String> {
        if let Some(content) = &self.content {
            return Ok(content.clone());
        }

        if let Some(path) = &self.path {
            return std::fs::read_to_string(path)
                .map_err(|e| EngineError::io(format!("Failed to read XML: {path}"), e));
        }

        Err(EngineError::config("No XML source specified"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_get_attribute() {
        let xml = r#"<element id="test-id" name="Test Name"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        if let Ok(Event::Start(start) | Event::Empty(start)) = reader.read_event_into(&mut buf) {
            assert_eq!(get_attribute(&start, "id"), Some("test-id".to_string()));
            assert_eq!(get_attribute(&start, "name"), Some("Test Name".to_string()));
            assert_eq!(get_attribute(&start, "missing"), None);
        } else {
            panic!("Expected start or empty element");
        }
    }

    #[test]
    fn test_extract_text_content() {
        let xml = r"<root><definition>Some text content here</definition></root>";
        let cursor = Cursor::new(xml);
        let mut reader = Reader::from_reader(cursor);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"definition") => break,
                Ok(Event::Eof) => panic!("Unexpected EOF"),
                _ => {}
            }
            buf.clear();
        }

        let content = extract_text_content(&mut reader, &mut buf, b"definition").unwrap();
        assert_eq!(content, "Some text content here");
    }

    #[test]
    fn test_xml_source() {
        let source = XmlSource::from_content("<root/>".to_string());
        assert_eq!(source.read().unwrap(), "<root/>");
    }

    #[test]
    fn test_xml_parser_default() {
        let parser = XmlParser::default();
        assert!(parser.config.trim_text);
        assert!(parser.config.check_end_names);
    }

    #[test]
    fn test_xml_parser_with_config() {
        let config = XmlParserConfig {
            trim_text: false,
            check_end_names: false,
        };
        let parser = XmlParser::with_config(config);
        assert!(!parser.config.trim_text);
        assert!(!parser.config.check_end_names);
    }

    #[test]
    fn test_xml_source_from_path() {
        let source = XmlSource::from_path("/some/path.xml");
        assert_eq!(source.path, Some("/some/path.xml".to_string()));
        assert!(source.content.is_none());
    }

    #[test]
    fn test_xml_source_no_source() {
        let source = XmlSource {
            path: None,
            content: None,
        };
        assert!(source.read().is_err());
    }

    #[test]
    fn test_skip_element() {
        let xml = r"<root><skip>content to skip</skip></root>";
        let cursor = Cursor::new(xml);
        let mut reader = Reader::from_reader(cursor);
        let mut buf = Vec::new();

        // Find the <skip> element
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"skip") => break,
                Ok(Event::Eof) => panic!("Unexpected EOF"),
                _ => {}
            }
            buf.clear();
        }

        // Skip to the end of the <skip> element
        skip_element(&mut reader, &mut buf, b"skip").unwrap();

        // We should now be at </root>
        buf.clear();
        if let Ok(Event::End(e)) = reader.read_event_into(&mut buf) {
            assert_eq!(e.name(), QName(b"root"));
        } else {
            panic!("Expected </root>");
        }
    }

    #[test]
    fn test_xml_parser_config_default() {
        let config = XmlParserConfig::default();
        assert!(config.trim_text);
        assert!(config.check_end_names);
    }
}
