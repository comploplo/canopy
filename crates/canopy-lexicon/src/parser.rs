//! XML parser for Canopy Lexicon data
//!
//! This module handles parsing of the lexicon XML files using the canopy-engine
//! XML infrastructure to load word classes, patterns, and metadata.

use crate::types::{
    LexiconDatabase, LexiconPattern, LexiconWord, PatternType, PropertyValue, WordClass,
    WordClassType,
};
use canopy_engine::{EngineError, EngineResult, XmlResource};
use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::name::QName;
use std::io::BufRead;

/// Lexicon XML resource for parsing
#[derive(Debug, Clone)]
pub struct LexiconXmlResource {
    pub database: LexiconDatabase,
}

impl XmlResource for LexiconXmlResource {
    fn parse_xml<R: BufRead>(reader: &mut Reader<R>) -> EngineResult<Self> {
        let mut database = LexiconDatabase::new();
        let mut buf = Vec::new();
        let mut current_word_class: Option<WordClass> = None;
        let mut current_pattern_data: Option<(String, PatternType, String, String)> = None; // (id, type, description, regex)
        let mut current_examples: Vec<String> = Vec::new();
        let mut in_metadata = false;
        let mut in_word_classes = false;
        let mut in_word_class = false;
        let mut in_words = false;
        let mut in_patterns = false;
        let mut in_pattern = false;
        let mut in_examples = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name() {
                        QName(b"lexicon") => {
                            // Parse lexicon attributes
                            for attr in e.attributes() {
                                let attr = attr.map_err(|e| {
                                    EngineError::data_load(format!(
                                        "Failed to parse attribute: {e}"
                                    ))
                                })?;

                                match attr.key {
                                    QName(b"version") => {
                                        database.version = String::from_utf8(attr.value.to_vec())
                                            .map_err(|e| {
                                            EngineError::data_load(format!("Invalid version: {e}"))
                                        })?;
                                    }
                                    QName(b"language") => {
                                        database.language = String::from_utf8(attr.value.to_vec())
                                            .map_err(|e| {
                                                EngineError::data_load(format!(
                                                    "Invalid language: {e}"
                                                ))
                                            })?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        QName(b"metadata") => {
                            in_metadata = true;
                        }
                        QName(b"title") if in_metadata => {
                            database.title = parse_text_content(reader, &mut buf, b"title")?;
                        }
                        QName(b"description") if in_metadata => {
                            database.description =
                                parse_text_content(reader, &mut buf, b"description")?;
                        }
                        QName(b"created") if in_metadata => {
                            database.created = parse_text_content(reader, &mut buf, b"created")?;
                        }
                        QName(b"author") if in_metadata => {
                            database.author = parse_text_content(reader, &mut buf, b"author")?;
                        }
                        QName(b"license") if in_metadata => {
                            database.license = parse_text_content(reader, &mut buf, b"license")?;
                        }
                        QName(b"word-classes") => {
                            in_word_classes = true;
                        }
                        QName(b"word-class") if in_word_classes => {
                            in_word_class = true;
                            current_word_class = Some(parse_word_class_start(e)?);
                        }
                        QName(b"description") if in_word_class => {
                            if let Some(ref mut word_class) = current_word_class {
                                word_class.description =
                                    parse_text_content(reader, &mut buf, b"description")?;
                            }
                        }
                        QName(b"properties") if in_word_class => {
                            // Properties will be handled by property elements
                        }
                        QName(b"property") if in_word_class => {
                            if let Some(ref mut word_class) = current_word_class {
                                parse_property(e, word_class)?;
                            }
                        }
                        QName(b"words") if in_word_class => {
                            in_words = true;
                        }
                        QName(b"word") if in_words => {
                            // Parse word attributes first
                            let mut pos = None;
                            let mut confidence = 1.0f32;
                            let mut frequency = None;
                            let mut context = None;

                            for attr in e.attributes() {
                                let attr = attr.map_err(|e| {
                                    EngineError::data_load(format!(
                                        "Failed to parse word attribute: {e}"
                                    ))
                                })?;
                                match attr.key.as_ref() {
                                    b"pos" => {
                                        let pos_str =
                                            std::str::from_utf8(&attr.value).map_err(|e| {
                                                EngineError::data_load(format!(
                                                    "Invalid UTF-8 in pos: {e}"
                                                ))
                                            })?;
                                        pos = Some(pos_str.to_string());
                                    }
                                    b"confidence" => {
                                        let conf_str =
                                            std::str::from_utf8(&attr.value).map_err(|e| {
                                                EngineError::data_load(format!(
                                                    "Invalid UTF-8 in confidence: {e}"
                                                ))
                                            })?;
                                        confidence = conf_str.parse().map_err(|e| {
                                            EngineError::data_load(format!(
                                                "Invalid confidence number: {e}"
                                            ))
                                        })?;
                                    }
                                    b"frequency" => {
                                        let freq_str =
                                            std::str::from_utf8(&attr.value).map_err(|e| {
                                                EngineError::data_load(format!(
                                                    "Invalid UTF-8 in frequency: {e}"
                                                ))
                                            })?;
                                        frequency = Some(freq_str.parse().map_err(|e| {
                                            EngineError::data_load(format!(
                                                "Invalid frequency number: {e}"
                                            ))
                                        })?);
                                    }
                                    b"context" => {
                                        let context_str = std::str::from_utf8(&attr.value)
                                            .map_err(|e| {
                                                EngineError::data_load(format!(
                                                    "Invalid UTF-8 in context: {e}"
                                                ))
                                            })?;
                                        context = Some(context_str.to_string());
                                    }
                                    _ => {} // Ignore unknown attributes
                                }
                            }

                            // Parse text content and create word immediately
                            let word_text = parse_text_content(reader, &mut buf, b"word")?;
                            let word = LexiconWord {
                                word: word_text,
                                variants: Vec::new(),
                                pos,
                                confidence,
                                frequency,
                                context,
                            };

                            // Add directly to word class
                            if let Some(ref mut word_class) = current_word_class {
                                word_class.words.push(word);
                            }
                        }
                        QName(b"patterns") if in_word_class => {
                            in_patterns = true;
                        }
                        QName(b"pattern") if in_patterns => {
                            in_pattern = true;
                            current_pattern_data = Some(parse_pattern_start(e)?);
                        }
                        QName(b"regex") if in_pattern => {
                            if let Some((_, _, _, ref mut regex)) = current_pattern_data {
                                *regex = parse_text_content(reader, &mut buf, b"regex")?;
                            }
                        }
                        QName(b"description") if in_pattern => {
                            if let Some((_, _, ref mut description, _)) = current_pattern_data {
                                *description =
                                    parse_text_content(reader, &mut buf, b"description")?;
                            }
                        }
                        QName(b"examples") if in_pattern => {
                            in_examples = true;
                            current_examples.clear();
                        }
                        QName(b"example") if in_examples => {
                            let example = parse_text_content(reader, &mut buf, b"example")?;
                            current_examples.push(example);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => match e.name() {
                    QName(b"metadata") => {
                        in_metadata = false;
                    }
                    QName(b"word-classes") => {
                        in_word_classes = false;
                    }
                    QName(b"word-class") => {
                        if let Some(word_class) = current_word_class.take() {
                            database.word_classes.push(word_class);
                        }
                        in_word_class = false;
                    }
                    QName(b"words") => {
                        in_words = false;
                    }
                    QName(b"patterns") => {
                        in_patterns = false;
                    }
                    QName(b"pattern") => {
                        if let (
                            Some((id, pattern_type, description, regex)),
                            Some(ref mut word_class),
                        ) = (current_pattern_data.take(), current_word_class.as_mut())
                        {
                            match LexiconPattern::new(id, pattern_type, regex, description) {
                                Ok(mut pattern) => {
                                    pattern.examples = current_examples.clone();
                                    word_class.patterns.push(pattern);
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to create pattern: {}", e);
                                }
                            }
                        }
                        in_pattern = false;
                    }
                    QName(b"examples") => {
                        in_examples = false;
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(EngineError::data_load(format!("XML parsing error: {e}"))),
                _ => {}
            }
            buf.clear();
        }

        // Build indices for fast lookup
        database.build_indices();

        Ok(LexiconXmlResource { database })
    }

    fn validate(&self) -> EngineResult<()> {
        if self.database.word_classes.is_empty() {
            return Err(EngineError::data_load(
                "No word classes found in lexicon".to_string(),
            ));
        }

        // Validate that all patterns compile
        for word_class in &self.database.word_classes {
            for pattern in &word_class.patterns {
                // Pattern regex is already validated during creation
                if pattern.examples.is_empty() {
                    tracing::warn!("Pattern {} has no examples", pattern.id);
                }
            }
        }

        Ok(())
    }

    fn root_element() -> &'static str {
        "lexicon"
    }
}

/// Parse text content from XML element
fn parse_text_content<R: BufRead>(
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
            _ => {} // Skip other events
        }
        buf.clear();
    }

    Ok(content.trim().to_string())
}

/// Parse word class start tag
fn parse_word_class_start(start: &quick_xml::events::BytesStart) -> EngineResult<WordClass> {
    let mut id = String::new();
    let mut name = String::new();
    let mut word_class_type = WordClassType::Functional;
    let mut priority = 1u8;

    for attr in start.attributes() {
        let attr = attr.map_err(|e| {
            EngineError::data_load(format!("Failed to parse word-class attribute: {e}"))
        })?;

        match attr.key {
            QName(b"id") => {
                id = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid id: {e}")))?;
            }
            QName(b"name") => {
                name = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid name: {e}")))?;
            }
            QName(b"type") => {
                let type_str = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid type: {e}")))?;
                word_class_type = WordClassType::parse_str(&type_str).ok_or_else(|| {
                    EngineError::data_load(format!("Unknown word class type: {type_str}"))
                })?;
            }
            QName(b"priority") => {
                let priority_str = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid priority: {e}")))?;
                priority = priority_str
                    .parse()
                    .map_err(|e| EngineError::data_load(format!("Invalid priority number: {e}")))?;
            }
            _ => {}
        }
    }

    if id.is_empty() {
        return Err(EngineError::data_load(
            "Word class missing required id attribute".to_string(),
        ));
    }

    let mut word_class = WordClass::new(id, name, word_class_type, String::new());
    word_class.priority = priority;

    Ok(word_class)
}

/// Parse property element
fn parse_property(
    start: &quick_xml::events::BytesStart,
    word_class: &mut WordClass,
) -> EngineResult<()> {
    let mut name = String::new();
    let mut value = String::new();
    let mut prop_type = String::from("string");

    for attr in start.attributes() {
        let attr = attr.map_err(|e| {
            EngineError::data_load(format!("Failed to parse property attribute: {e}"))
        })?;

        match attr.key {
            QName(b"name") => {
                name = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid property name: {e}")))?;
            }
            QName(b"value") => {
                value = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid property value: {e}")))?;
            }
            QName(b"type") => {
                prop_type = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid property type: {e}")))?;
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return Err(EngineError::data_load(
            "Property missing required name attribute".to_string(),
        ));
    }

    let property_value = match prop_type.as_str() {
        "boolean" => {
            let bool_val = value
                .parse::<bool>()
                .map_err(|e| EngineError::data_load(format!("Invalid boolean value: {e}")))?;
            PropertyValue::Boolean(bool_val)
        }
        "integer" => {
            let int_val = value
                .parse::<i64>()
                .map_err(|e| EngineError::data_load(format!("Invalid integer value: {e}")))?;
            PropertyValue::Integer(int_val)
        }
        "float" => {
            let float_val = value
                .parse::<f64>()
                .map_err(|e| EngineError::data_load(format!("Invalid float value: {e}")))?;
            PropertyValue::Float(float_val)
        }
        _ => PropertyValue::String(value),
    };

    word_class.properties.insert(name, property_value);
    Ok(())
}

/// Parse pattern start tag
fn parse_pattern_start(
    start: &quick_xml::events::BytesStart,
) -> EngineResult<(String, PatternType, String, String)> {
    let mut id = String::new();
    let mut pattern_type = PatternType::WholeWord;
    let mut confidence = 0.8f32;

    for attr in start.attributes() {
        let attr = attr.map_err(|e| {
            EngineError::data_load(format!("Failed to parse pattern attribute: {e}"))
        })?;

        match attr.key {
            QName(b"id") => {
                id = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid pattern id: {e}")))?;
            }
            QName(b"type") => {
                let type_str = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid pattern type: {e}")))?;
                pattern_type = PatternType::parse_str(&type_str).ok_or_else(|| {
                    EngineError::data_load(format!("Unknown pattern type: {type_str}"))
                })?;
            }
            QName(b"confidence") => {
                let conf_str = String::from_utf8(attr.value.to_vec()).map_err(|e| {
                    EngineError::data_load(format!("Invalid pattern confidence: {e}"))
                })?;
                confidence = conf_str.parse().map_err(|e| {
                    EngineError::data_load(format!("Invalid confidence number: {e}"))
                })?;
            }
            _ => {}
        }
    }

    if id.is_empty() {
        return Err(EngineError::data_load(
            "Pattern missing required id attribute".to_string(),
        ));
    }

    Ok((id, pattern_type, confidence.to_string(), String::new()))
}
