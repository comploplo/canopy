//! Shared XML parsing utilities
//!
//! Common helper functions for parsing XML files across semantic engines.
//! Used by VerbNet, FrameNet, Lexicon, and other XML-based parsers.

use crate::{EngineError, EngineResult};
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;
use std::io::BufRead;

/// Extract attribute value from XML element by name
///
/// # Arguments
/// * `element` - The XML start tag to search
/// * `attr_name` - Name of the attribute to find
///
/// # Returns
/// * `Some(String)` - The attribute value if found
/// * `None` - If attribute doesn't exist or can't be decoded
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
/// Reads all text events until the matching end tag is found.
/// Handles text unescaping and concatenation.
///
/// # Arguments
/// * `reader` - The XML reader
/// * `buf` - Buffer for reading events
/// * `end_tag` - The end tag name to look for (e.g., b"definition")
///
/// # Returns
/// * `Ok(String)` - The trimmed text content
/// * `Err` - If EOF reached before end tag or parsing error
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
            _ => {} // Skip other events (comments, processing instructions, etc.)
        }
        buf.clear();
    }

    Ok(content.trim().to_string())
}

/// Skip to the end of the current XML element
///
/// Handles nested elements with the same name by tracking depth.
/// Useful for skipping over elements you don't need to parse.
///
/// # Arguments
/// * `reader` - The XML reader
/// * `buf` - Buffer for reading events
/// * `element_name` - The element name to skip (e.g., b"annotation")
///
/// # Returns
/// * `Ok(())` - Successfully skipped to end of element
/// * `Err` - If EOF reached before end tag or parsing error
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_get_attribute() {
        let xml = r#"<element id="test-id" name="Test Name"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        if let Ok(Event::Start(start)) | Ok(Event::Empty(start)) = reader.read_event_into(&mut buf)
        {
            assert_eq!(get_attribute(&start, "id"), Some("test-id".to_string()));
            assert_eq!(get_attribute(&start, "name"), Some("Test Name".to_string()));
            assert_eq!(get_attribute(&start, "missing"), None);
        } else {
            panic!("Expected start or empty element");
        }
    }

    #[test]
    fn test_extract_text_content() {
        let xml = r#"<root><definition>Some text content here</definition></root>"#;
        let cursor = Cursor::new(xml);
        let mut reader = Reader::from_reader(cursor);
        let mut buf = Vec::new();

        // Skip to the definition element
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
    fn test_skip_element() {
        let xml = r#"<root><skip><nested><deep>content</deep></nested></skip><next/></root>"#;
        let cursor = Cursor::new(xml);
        let mut reader = Reader::from_reader(cursor);
        let mut buf = Vec::new();

        // Skip to the skip element
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) if e.name() == QName(b"skip") => break,
                Ok(Event::Eof) => panic!("Unexpected EOF"),
                _ => {}
            }
            buf.clear();
        }

        skip_element(&mut reader, &mut buf, b"skip").unwrap();

        // Verify we're at the next element
        buf.clear();
        if let Ok(Event::Empty(e)) = reader.read_event_into(&mut buf) {
            assert_eq!(e.name(), QName(b"next"));
        } else {
            panic!("Expected empty next element");
        }
    }
}
