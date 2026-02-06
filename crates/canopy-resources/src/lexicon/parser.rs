//! XML parser for Canopy Lexicon data
//!
//! This module handles parsing of the lexicon XML files using the canopy-engine
//! XML infrastructure to load word classes, patterns, and metadata.

use super::types::{
    LexiconDatabase, LexiconPattern, LexiconWord, PatternType, PropertyValue, WordClass,
    WordClassType,
};
use crate::engine::{
    extract_text_content as parse_text_content, EngineError, EngineResult, XmlResource,
};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use std::io::BufRead;

/// Parse lexicon root attributes
fn parse_lexicon_attrs(
    e: &quick_xml::events::BytesStart,
    db: &mut LexiconDatabase,
) -> EngineResult<()> {
    for attr in e.attributes() {
        let attr =
            attr.map_err(|e| EngineError::data_load(format!("Failed to parse attribute: {e}")))?;
        match attr.key {
            QName(b"version") => {
                db.version = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid version: {e}")))?;
            }
            QName(b"language") => {
                db.language = String::from_utf8(attr.value.to_vec())
                    .map_err(|e| EngineError::data_load(format!("Invalid language: {e}")))?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Parse word element attributes and text
/// Extract a string attribute value from an XML attribute.
fn attr_string(value: &[u8], name: &str) -> EngineResult<String> {
    std::str::from_utf8(value)
        .map_err(|e| EngineError::data_load(format!("Invalid UTF-8 in {name}: {e}")))
        .map(String::from)
}

fn parse_word_element<R: BufRead>(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> EngineResult<LexiconWord> {
    let mut pos = None;
    let mut confidence = 1.0f32;
    let mut frequency = None;
    let mut context = None;
    let mut person = None;
    let mut number = None;
    let mut case = None;
    let mut gender = None;

    for attr in e.attributes() {
        let attr = attr
            .map_err(|e| EngineError::data_load(format!("Failed to parse word attribute: {e}")))?;
        match attr.key.as_ref() {
            b"pos" => pos = Some(attr_string(&attr.value, "pos")?),
            b"confidence" => {
                confidence = attr_string(&attr.value, "confidence")?
                    .parse()
                    .map_err(|e| EngineError::data_load(format!("Invalid confidence: {e}")))?;
            }
            b"frequency" => {
                frequency = Some(
                    attr_string(&attr.value, "frequency")?
                        .parse()
                        .map_err(|e| EngineError::data_load(format!("Invalid frequency: {e}")))?,
                );
            }
            b"context" => context = Some(attr_string(&attr.value, "context")?),
            b"person" => person = Some(attr_string(&attr.value, "person")?),
            b"number" => number = Some(attr_string(&attr.value, "number")?),
            b"case" => case = Some(attr_string(&attr.value, "case")?),
            b"gender" => gender = Some(attr_string(&attr.value, "gender")?),
            _ => {}
        }
    }
    let word_text = parse_text_content(reader, buf, b"word")?;
    Ok(LexiconWord {
        word: word_text,
        variants: Vec::new(),
        pos,
        confidence,
        frequency,
        context,
        person,
        number,
        case,
        gender,
    })
}

/// Parser location within document structure
#[derive(Default)]
struct DocumentLocation {
    in_metadata: bool,
    in_word_classes: bool,
    in_word_class: bool,
}

/// Location within word class content
#[derive(Default, PartialEq, Eq)]
enum WordClassSection {
    #[default]
    None,
    Words,
    Patterns,
}

/// Parser location within pattern element
#[derive(Default)]
struct PatternLocation {
    in_pattern: bool,
    in_examples: bool,
}

/// Parser state combining document and pattern locations
#[derive(Default)]
struct ParserState {
    doc: DocumentLocation,
    section: WordClassSection,
    pattern: PatternLocation,
}

/// Mutable parsing context to reduce function arguments.
struct ParseContext<'a> {
    db: &'a mut LexiconDatabase,
    current_wc: &'a mut Option<WordClass>,
    current_pattern: &'a mut Option<(String, PatternType, String, String)>,
    examples: &'a mut Vec<String>,
    state: &'a mut ParserState,
}

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
        let mut current_pattern_data: Option<(String, PatternType, String, String)> = None;
        let mut current_examples: Vec<String> = Vec::new();
        let mut state = ParserState::default();

        loop {
            buf.clear();
            let event = reader.read_event_into(&mut buf);

            let mut ctx = ParseContext {
                db: &mut database,
                current_wc: &mut current_word_class,
                current_pattern: &mut current_pattern_data,
                examples: &mut current_examples,
                state: &mut state,
            };

            match event {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    Self::handle_start_event(name, e, reader, &mut ctx)?;
                }
                Ok(Event::End(ref e)) => {
                    Self::handle_end_event(e, &mut ctx);
                }
                Ok(Event::Empty(ref e)) if ctx.state.doc.in_word_class => {
                    if e.name() == QName(b"property") {
                        if let Some(ref mut wc) = ctx.current_wc {
                            parse_property(e, wc)?;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(EngineError::data_load(format!("XML parsing error: {e}"))),
                _ => {}
            }
        }
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

impl LexiconXmlResource {
    fn handle_start_event<R: BufRead>(
        name: QName,
        e: &quick_xml::events::BytesStart,
        reader: &mut Reader<R>,
        ctx: &mut ParseContext<'_>,
    ) -> EngineResult<()> {
        let mut buf = Vec::new();
        match name {
            QName(b"lexicon") => parse_lexicon_attrs(e, ctx.db)?,
            QName(b"metadata") => ctx.state.doc.in_metadata = true,
            QName(b"title") if ctx.state.doc.in_metadata => {
                ctx.db.title = parse_text_content(reader, &mut buf, b"title")?;
            }
            QName(b"description") if ctx.state.doc.in_metadata => {
                ctx.db.description = parse_text_content(reader, &mut buf, b"description")?;
            }
            QName(b"created") if ctx.state.doc.in_metadata => {
                ctx.db.created = parse_text_content(reader, &mut buf, b"created")?;
            }
            QName(b"author") if ctx.state.doc.in_metadata => {
                ctx.db.author = parse_text_content(reader, &mut buf, b"author")?;
            }
            QName(b"license") if ctx.state.doc.in_metadata => {
                ctx.db.license = parse_text_content(reader, &mut buf, b"license")?;
            }
            QName(b"word-classes") => ctx.state.doc.in_word_classes = true,
            QName(b"word-class") if ctx.state.doc.in_word_classes => {
                ctx.state.doc.in_word_class = true;
                *ctx.current_wc = Some(parse_word_class_start(e)?);
            }
            QName(b"description") if ctx.state.doc.in_word_class => {
                if let Some(wc) = ctx.current_wc {
                    wc.description = parse_text_content(reader, &mut buf, b"description")?;
                }
            }
            QName(b"properties") if ctx.state.doc.in_word_class => {}
            QName(b"property") if ctx.state.doc.in_word_class => {
                if let Some(wc) = ctx.current_wc {
                    parse_property(e, wc)?;
                }
            }
            QName(b"words") if ctx.state.doc.in_word_class => {
                ctx.state.section = WordClassSection::Words;
            }
            QName(b"word") if ctx.state.section == WordClassSection::Words => {
                if let Some(wc) = ctx.current_wc {
                    wc.words.push(parse_word_element(e, reader, &mut buf)?);
                }
            }
            QName(b"patterns") if ctx.state.doc.in_word_class => {
                ctx.state.section = WordClassSection::Patterns;
            }
            QName(b"pattern") if ctx.state.section == WordClassSection::Patterns => {
                ctx.state.pattern.in_pattern = true;
                *ctx.current_pattern = Some(parse_pattern_start(e)?);
            }
            QName(b"regex") if ctx.state.pattern.in_pattern => {
                if let Some((_, _, _, ref mut regex)) = ctx.current_pattern {
                    *regex = parse_text_content(reader, &mut buf, b"regex")?;
                }
            }
            QName(b"description") if ctx.state.pattern.in_pattern => {
                if let Some((_, _, ref mut desc, _)) = ctx.current_pattern {
                    *desc = parse_text_content(reader, &mut buf, b"description")?;
                }
            }
            QName(b"examples") if ctx.state.pattern.in_pattern => {
                ctx.state.pattern.in_examples = true;
                ctx.examples.clear();
            }
            QName(b"example") if ctx.state.pattern.in_examples => {
                ctx.examples
                    .push(parse_text_content(reader, &mut buf, b"example")?);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_end_event(e: &quick_xml::events::BytesEnd, ctx: &mut ParseContext<'_>) {
        match e.name() {
            QName(b"metadata") => ctx.state.doc.in_metadata = false,
            QName(b"word-classes") => ctx.state.doc.in_word_classes = false,
            QName(b"word-class") => {
                if let Some(wc) = ctx.current_wc.take() {
                    ctx.db.word_classes.push(wc);
                }
                ctx.state.doc.in_word_class = false;
                ctx.state.section = WordClassSection::None;
            }
            QName(b"words" | b"patterns") => ctx.state.section = WordClassSection::None,
            QName(b"pattern") => {
                if let (Some((id, ptype, desc, regex)), Some(wc)) =
                    (ctx.current_pattern.take(), ctx.current_wc.as_mut())
                {
                    if let Ok(mut pat) = LexiconPattern::new(id, ptype, regex, desc) {
                        pat.examples.clone_from(ctx.examples);
                        wc.patterns.push(pat);
                    }
                }
                ctx.state.pattern.in_pattern = false;
            }
            QName(b"examples") => ctx.state.pattern.in_examples = false,
            _ => {}
        }
    }
}

// parse_text_content imported from canopy_engine::xml_utils (as extract_text_content)

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
