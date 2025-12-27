//! FrameNet XML parser
//!
//! This module implements parsing of FrameNet XML files using the
//! canopy-engine XML infrastructure.

use crate::types::*;
use canopy_engine::{EngineError, EngineResult, XmlResource};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use std::io::BufRead;
use tracing::{debug, trace};

/// FrameNet XML parser helper
pub struct FrameParser;

impl XmlResource for Frame {
    fn parse_xml<R: BufRead>(reader: &mut Reader<R>) -> EngineResult<Self> {
        let mut buf = Vec::new();
        let mut frame = Frame {
            id: String::new(),
            name: String::new(),
            created_by: None,
            created_date: None,
            definition: String::new(),
            frame_elements: Vec::new(),
            frame_relations: Vec::new(),
            lexical_units: Vec::new(),
        };

        // Parse root frame element
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name() {
                        QName(b"frame") => {
                            // Extract frame attributes
                            if let Some(id) = get_attribute(e, "ID") {
                                frame.id = id;
                            }
                            if let Some(name) = get_attribute(e, "name") {
                                frame.name = name;
                                debug!("Parsing FrameNet frame: {}", frame.name);
                            }
                            if let Some(created_by) = get_attribute(e, "cBy") {
                                frame.created_by = Some(created_by);
                            }
                            if let Some(created_date) = get_attribute(e, "cDate") {
                                frame.created_date = Some(created_date);
                            }
                        }
                        QName(b"definition") => {
                            frame.definition =
                                extract_text_content(reader, &mut buf, b"definition")?;
                            // Clean up XML entities in definition
                            frame.definition = clean_definition(&frame.definition);
                        }
                        QName(b"FE") => {
                            let mut fe_buf = Vec::new();
                            let fe = parse_frame_element(reader, &mut fe_buf, e)?;
                            frame.frame_elements.push(fe);
                        }
                        QName(b"frameRelation") => {
                            let mut rel_buf = Vec::new();
                            let relation = parse_frame_relation(reader, &mut rel_buf, e)?;
                            frame.frame_relations.push(relation);
                        }
                        QName(b"lexUnit") => {
                            let mut lu_buf = Vec::new();
                            let lu_ref = parse_lexical_unit_ref(reader, &mut lu_buf, e)?;
                            frame.lexical_units.push(lu_ref);
                        }
                        _ => {
                            // Skip unknown elements
                            trace!("Skipping unknown element: {:?}", e.name());
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    match e.name() {
                        QName(b"frameRelation") => {
                            // Handle self-closing frameRelation elements
                            let relation_type = get_attribute(e, "type").unwrap_or_default();
                            let related_frame_id =
                                get_attribute(e, "relatedFrame").unwrap_or_default();
                            let related_frame_name =
                                get_attribute(e, "relatedFrameName").unwrap_or_default();
                            frame.frame_relations.push(FrameRelation {
                                relation_type,
                                related_frame_id,
                                related_frame_name,
                            });
                        }
                        QName(b"lexUnit") => {
                            // Handle self-closing lexUnit elements
                            let id = get_attribute(e, "ID").unwrap_or_default();
                            let name = get_attribute(e, "name").unwrap_or_default();
                            let pos = get_attribute(e, "POS").unwrap_or_default();
                            let status = get_attribute(e, "status").unwrap_or_default();
                            frame.lexical_units.push(LexicalUnitRef {
                                id,
                                name,
                                pos,
                                status,
                            });
                        }
                        _ => {
                            trace!("Skipping unknown empty element: {:?}", e.name());
                        }
                    }
                }
                Ok(Event::End(ref e)) if e.name() == QName(b"frame") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(EngineError::data_load(format!("XML parsing error: {e}")));
                }
                _ => {}
            }
            buf.clear();
        }

        // Validate that we got required fields
        if frame.id.is_empty() {
            return Err(EngineError::data_load(
                "Frame missing required ID attribute".to_string(),
            ));
        }
        if frame.name.is_empty() {
            return Err(EngineError::data_load(
                "Frame missing required name attribute".to_string(),
            ));
        }

        debug!(
            "Successfully parsed FrameNet frame: {} (ID: {})",
            frame.name, frame.id
        );
        Ok(frame)
    }

    fn root_element() -> &'static str {
        "frame"
    }
}

impl XmlResource for LexicalUnit {
    fn parse_xml<R: BufRead>(reader: &mut Reader<R>) -> EngineResult<Self> {
        let mut buf = Vec::new();
        let mut lexical_unit = LexicalUnit {
            id: String::new(),
            name: String::new(),
            pos: String::new(),
            status: String::new(),
            frame_id: String::new(),
            frame_name: String::new(),
            total_annotated: 0,
            definition: String::new(),
            lexemes: Vec::new(),
            valences: Vec::new(),
            subcategorization: Vec::new(),
        };

        // Parse root lexUnit element
        loop {
            let event = reader
                .read_event_into(&mut buf)
                .map_err(|e| EngineError::data_load(format!("XML parsing error: {e}")))?;
            match event {
                Event::Start(ref e) | Event::Empty(ref e) => {
                    match e.name() {
                        QName(b"lexUnit") => {
                            // Extract LU attributes
                            if let Some(id) = get_attribute(e, "ID") {
                                lexical_unit.id = id;
                            }
                            if let Some(name) = get_attribute(e, "name") {
                                lexical_unit.name = name;
                                debug!("Parsing FrameNet lexical unit: {}", lexical_unit.name);
                            }
                            if let Some(pos) = get_attribute(e, "POS") {
                                lexical_unit.pos = pos;
                            }
                            if let Some(status) = get_attribute(e, "status") {
                                lexical_unit.status = status;
                            }
                            if let Some(frame) = get_attribute(e, "frame") {
                                lexical_unit.frame_name = frame;
                            }
                            if let Some(frame_id) = get_attribute(e, "frameID") {
                                lexical_unit.frame_id = frame_id;
                            }
                            if let Some(total) = get_attribute(e, "totalAnnotated") {
                                lexical_unit.total_annotated = total.parse().unwrap_or(0);
                            }
                        }
                        QName(b"definition") => {
                            lexical_unit.definition =
                                extract_text_content(reader, &mut buf, b"definition")?;
                        }
                        QName(b"lexeme") => {
                            // Handle both self-closing and regular lexeme tags
                            let pos = get_attribute(e, "POS").unwrap_or_default();
                            let name = get_attribute(e, "name").unwrap_or_default();
                            let break_before = get_attribute(e, "breakBefore").map(|s| s == "true");
                            let headword = get_attribute(e, "headword").map(|s| s == "true");

                            let lexeme = Lexeme {
                                pos,
                                name,
                                break_before,
                                headword,
                            };
                            lexical_unit.lexemes.push(lexeme);

                            // For non-self-closing tags, skip to the end
                            if matches!(event, Event::Start(_)) {
                                skip_element(reader, &mut buf, b"lexeme")?;
                            }
                        }
                        QName(b"valences") => {
                            lexical_unit.valences = parse_valences(reader, &mut buf)?;
                        }
                        QName(b"subCorpus") => {
                            // Skip subcorpus data for now
                            skip_element(reader, &mut buf, b"subCorpus")?;
                        }
                        _ => {
                            trace!("Skipping unknown element: {:?}", e.name());
                        }
                    }
                }
                Event::End(ref e) if e.name() == QName(b"lexUnit") => {
                    break;
                }
                Event::Eof => break,
                _ => {} // Skip other events
            }
            buf.clear();
        }

        // Validate required fields
        if lexical_unit.id.is_empty() {
            return Err(EngineError::data_load(
                "LexicalUnit missing required ID attribute".to_string(),
            ));
        }

        debug!(
            "Successfully parsed FrameNet lexical unit: {} (ID: {})",
            lexical_unit.name, lexical_unit.id
        );
        Ok(lexical_unit)
    }

    fn root_element() -> &'static str {
        "lexUnit"
    }
}

/// Parse a frame element (FE)
fn parse_frame_element<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_tag: &quick_xml::events::BytesStart,
) -> EngineResult<FrameElement> {
    let mut fe = FrameElement {
        id: String::new(),
        name: String::new(),
        abbrev: String::new(),
        core_type: CoreType::Core,
        bg_color: None,
        fg_color: None,
        created_by: None,
        created_date: None,
        definition: String::new(),
        semantic_types: Vec::new(),
        fe_relations: Vec::new(),
    };

    // Extract FE attributes
    if let Some(id) = get_attribute(start_tag, "ID") {
        fe.id = id;
    }
    if let Some(name) = get_attribute(start_tag, "name") {
        fe.name = name;
    }
    if let Some(abbrev) = get_attribute(start_tag, "abbrev") {
        fe.abbrev = abbrev;
    }
    if let Some(core_type) = get_attribute(start_tag, "coreType") {
        fe.core_type = match core_type.as_str() {
            "Core" => CoreType::Core,
            "Peripheral" => CoreType::Peripheral,
            "Extra-Thematic" => CoreType::ExtraThematic,
            _ => CoreType::Core,
        };
    }
    if let Some(bg_color) = get_attribute(start_tag, "bgColor") {
        fe.bg_color = Some(bg_color);
    }
    if let Some(fg_color) = get_attribute(start_tag, "fgColor") {
        fe.fg_color = Some(fg_color);
    }
    if let Some(created_by) = get_attribute(start_tag, "cBy") {
        fe.created_by = Some(created_by);
    }
    if let Some(created_date) = get_attribute(start_tag, "cDate") {
        fe.created_date = Some(created_date);
    }

    // Parse FE content
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                QName(b"definition") => {
                    fe.definition = extract_text_content(reader, buf, b"definition")?;
                    fe.definition = clean_definition(&fe.definition);
                }
                QName(b"semType") => {
                    let mut sem_buf = Vec::new();
                    let sem_type = parse_semantic_type(reader, &mut sem_buf, e)?;
                    fe.semantic_types.push(sem_type);
                }
                QName(b"feRelation") => {
                    let mut rel_buf = Vec::new();
                    let relation = parse_fe_relation(reader, &mut rel_buf, e)?;
                    fe.fe_relations.push(relation);
                }
                _ => {
                    trace!("Skipping unknown FE element: {:?}", e.name());
                }
            },
            Ok(Event::Empty(ref e)) => {
                match e.name() {
                    QName(b"semType") => {
                        // Handle self-closing semType elements
                        let name = get_attribute(e, "name").unwrap_or_default();
                        let id = get_attribute(e, "ID").unwrap_or_default();
                        fe.semantic_types.push(SemanticType { name, id });
                    }
                    QName(b"feRelation") => {
                        // Handle self-closing feRelation elements
                        let relation_type = get_attribute(e, "type").unwrap_or_default();
                        let related_fe = get_attribute(e, "relatedFE").unwrap_or_default();
                        let related_frame = get_attribute(e, "relatedFrame").unwrap_or_default();
                        fe.fe_relations.push(FrameElementRelation {
                            relation_type,
                            related_fe,
                            related_frame,
                        });
                    }
                    _ => {
                        trace!("Skipping unknown empty FE element: {:?}", e.name());
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name() == QName(b"FE") => {
                break;
            }
            Ok(Event::Eof) => {
                return Err(EngineError::data_load(
                    "Unexpected end of file while parsing FE".to_string(),
                ));
            }
            Err(e) => {
                return Err(EngineError::data_load(format!(
                    "XML parsing error in FE: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(fe)
}

/// Parse a semantic type
fn parse_semantic_type<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_tag: &quick_xml::events::BytesStart,
) -> EngineResult<SemanticType> {
    let name = get_attribute(start_tag, "name").unwrap_or_default();
    let id = get_attribute(start_tag, "ID").unwrap_or_default();

    // Skip to end of element
    skip_element(reader, buf, b"semType")?;

    Ok(SemanticType { name, id })
}

/// Parse a frame relation
fn parse_frame_relation<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_tag: &quick_xml::events::BytesStart,
) -> EngineResult<FrameRelation> {
    let relation_type = get_attribute(start_tag, "type").unwrap_or_default();
    let related_frame_id = get_attribute(start_tag, "relatedFrame").unwrap_or_default();
    let related_frame_name = get_attribute(start_tag, "relatedFrameName").unwrap_or_default();

    skip_element(reader, buf, b"frameRelation")?;

    Ok(FrameRelation {
        relation_type,
        related_frame_id,
        related_frame_name,
    })
}

/// Parse a frame element relation
fn parse_fe_relation<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_tag: &quick_xml::events::BytesStart,
) -> EngineResult<FrameElementRelation> {
    let relation_type = get_attribute(start_tag, "type").unwrap_or_default();
    let related_fe = get_attribute(start_tag, "relatedFE").unwrap_or_default();
    let related_frame = get_attribute(start_tag, "relatedFrame").unwrap_or_default();

    skip_element(reader, buf, b"feRelation")?;

    Ok(FrameElementRelation {
        relation_type,
        related_fe,
        related_frame,
    })
}

/// Parse a lexical unit reference
fn parse_lexical_unit_ref<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_tag: &quick_xml::events::BytesStart,
) -> EngineResult<LexicalUnitRef> {
    let id = get_attribute(start_tag, "ID").unwrap_or_default();
    let name = get_attribute(start_tag, "name").unwrap_or_default();
    let pos = get_attribute(start_tag, "POS").unwrap_or_default();
    let status = get_attribute(start_tag, "status").unwrap_or_default();

    skip_element(reader, buf, b"lexUnit")?;

    Ok(LexicalUnitRef {
        id,
        name,
        pos,
        status,
    })
}

/// Parse a lexeme
#[allow(dead_code)]
fn parse_lexeme<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_tag: &quick_xml::events::BytesStart,
) -> EngineResult<Lexeme> {
    let pos = get_attribute(start_tag, "POS").unwrap_or_default();
    let name = get_attribute(start_tag, "name").unwrap_or_default();
    let break_before = get_attribute(start_tag, "breakBefore").map(|s| s == "true");
    let headword = get_attribute(start_tag, "headword").map(|s| s == "true");

    skip_element(reader, buf, b"lexeme")?;

    Ok(Lexeme {
        pos,
        name,
        break_before,
        headword,
    })
}

/// Parse valences section
fn parse_valences<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
) -> EngineResult<Vec<ValencePattern>> {
    let mut valences = Vec::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                QName(b"FERealization") => {
                    let mut val_buf = Vec::new();
                    let valence = parse_valence_pattern(reader, &mut val_buf, e)?;
                    valences.push(valence);
                }
                _ => {
                    trace!("Skipping unknown valences element: {:?}", e.name());
                }
            },
            Ok(Event::End(ref e)) if e.name() == QName(b"valences") => {
                break;
            }
            Ok(Event::Eof) => {
                return Err(EngineError::data_load(
                    "Unexpected end of file while parsing valences".to_string(),
                ));
            }
            Err(e) => {
                return Err(EngineError::data_load(format!(
                    "XML parsing error in valences: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(valences)
}

/// Parse a valence pattern
fn parse_valence_pattern<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_tag: &quick_xml::events::BytesStart,
) -> EngineResult<ValencePattern> {
    let total = get_attribute(start_tag, "total")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut fe_name = String::new();
    let mut realizations = Vec::new();

    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                QName(b"FE") => {
                    if let Some(name) = get_attribute(e, "name") {
                        fe_name = name;
                    }
                    skip_element(reader, buf, b"FE")?;
                }
                QName(b"pattern") => {
                    let mut real_buf = Vec::new();
                    let realization = parse_fe_realization(reader, &mut real_buf, e)?;
                    realizations.push(realization);
                }
                _ => {
                    trace!("Skipping unknown valence pattern element: {:?}", e.name());
                }
            },
            Ok(Event::Empty(ref e)) => match e.name() {
                QName(b"FE") => {
                    if let Some(name) = get_attribute(e, "name") {
                        fe_name = name;
                    }
                }
                _ => {
                    trace!(
                        "Skipping unknown empty valence pattern element: {:?}",
                        e.name()
                    );
                }
            },
            Ok(Event::End(ref e)) if e.name() == QName(b"FERealization") => {
                break;
            }
            Ok(Event::Eof) => {
                return Err(EngineError::data_load(
                    "Unexpected end of file while parsing valence pattern".to_string(),
                ));
            }
            Err(e) => {
                return Err(EngineError::data_load(format!(
                    "XML parsing error in valence pattern: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(ValencePattern {
        fe_name,
        total,
        realizations,
    })
}

/// Parse a frame element realization
fn parse_fe_realization<R: BufRead>(
    reader: &mut Reader<R>,
    buf: &mut Vec<u8>,
    start_tag: &quick_xml::events::BytesStart,
) -> EngineResult<FrameElementRealization> {
    let count = get_attribute(start_tag, "total")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut grammatical_function = String::new();
    let mut phrase_type = String::new();

    // Parse pattern elements
    loop {
        match reader.read_event_into(buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                QName(b"valenceUnit") => {
                    if let Some(gf) = get_attribute(e, "GF") {
                        grammatical_function = gf;
                    }
                    if let Some(pt) = get_attribute(e, "PT") {
                        phrase_type = pt;
                    }
                    skip_element(reader, buf, b"valenceUnit")?;
                }
                _ => {
                    trace!("Skipping unknown pattern element: {:?}", e.name());
                }
            },
            Ok(Event::Empty(ref e)) => match e.name() {
                QName(b"valenceUnit") => {
                    if let Some(gf) = get_attribute(e, "GF") {
                        grammatical_function = gf;
                    }
                    if let Some(pt) = get_attribute(e, "PT") {
                        phrase_type = pt;
                    }
                }
                _ => {
                    trace!("Skipping unknown empty pattern element: {:?}", e.name());
                }
            },
            Ok(Event::End(ref e)) if e.name() == QName(b"pattern") => {
                break;
            }
            Ok(Event::Eof) => {
                return Err(EngineError::data_load(
                    "Unexpected end of file while parsing FE realization".to_string(),
                ));
            }
            Err(e) => {
                return Err(EngineError::data_load(format!(
                    "XML parsing error in FE realization: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(FrameElementRealization {
        grammatical_function,
        phrase_type,
        count,
    })
}

/// Extract attribute value from XML start tag
fn get_attribute(element: &quick_xml::events::BytesStart, attr_name: &str) -> Option<String> {
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

/// Extract text content from an XML element
fn extract_text_content<R: BufRead>(
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

/// Skip to the end of the current element
fn skip_element<R: BufRead>(
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

/// Clean FrameNet definition text (remove XML entities, etc.)
fn clean_definition(definition: &str) -> String {
    definition
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        // Remove FrameNet markup tags like <def-root>, <fen>, <ex>, <t>, <fex>
        .replace("<def-root>", "")
        .replace("</def-root>", "")
        .replace("<fen>", "")
        .replace("</fen>", "")
        .replace("<ex>", "")
        .replace("</ex>", "")
        .replace("<t>", "")
        .replace("</t>", "")
        .replace("<fex", "")
        .replace("</fex>", "")
        // Remove attributes from fex tags (simplified approach)
        .split('<')
        .next()
        .unwrap_or(definition)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::Reader;

    #[test]
    fn test_parse_simple_frame() {
        let xml = r#"<?xml version="1.0"?>
        <frame ID="139" name="Giving">
            <definition>&lt;def-root&gt;A frame about giving&lt;/def-root&gt;</definition>
            <FE ID="1052" name="Donor" abbrev="Donor" coreType="Core" bgColor="FF0000" fgColor="FFFFFF">
                <definition>&lt;def-root&gt;The giver&lt;/def-root&gt;</definition>
            </FE>
            <FE ID="1053" name="Recipient" abbrev="Rec" coreType="Core" bgColor="0000FF" fgColor="FFFFFF">
                <definition>&lt;def-root&gt;The receiver&lt;/def-root&gt;</definition>
            </FE>
        </frame>"#;

        let mut reader = Reader::from_str(xml);
        let frame = Frame::parse_xml(&mut reader).unwrap();

        assert_eq!(frame.id, "139");
        assert_eq!(frame.name, "Giving");
        assert_eq!(frame.definition, "A frame about giving");
        assert_eq!(frame.frame_elements.len(), 2);
        assert_eq!(frame.frame_elements[0].name, "Donor");
        assert_eq!(frame.frame_elements[1].name, "Recipient");
        assert!(frame.frame_elements[0].is_core());
        assert!(frame.frame_elements[1].is_core());
    }

    #[test]
    fn test_clean_definition() {
        let definition =
            "&lt;def-root&gt;A &lt;fen&gt;Donor&lt;/fen&gt; gives something.&lt;/def-root&gt;";
        let cleaned = clean_definition(definition);
        assert_eq!(cleaned, "A Donor gives something.");
    }

    #[test]
    fn test_parse_lexical_unit() {
        let xml = r#"<?xml version="1.0"?>
        <lexUnit ID="2477" name="divest.v" POS="V" status="Finished_Initial" frame="Emptying" frameID="58" totalAnnotated="11">
            <definition>COD: deprive or dispossess someone or something of</definition>
            <lexeme POS="V" name="divest"/>
        </lexUnit>"#;

        let mut reader = Reader::from_str(xml);
        let lu = LexicalUnit::parse_xml(&mut reader).unwrap();

        assert_eq!(lu.id, "2477");
        assert_eq!(lu.name, "divest.v");
        assert_eq!(lu.pos, "V");
        assert_eq!(lu.status, "Finished_Initial");
        assert_eq!(lu.frame_name, "Emptying");
        assert_eq!(lu.frame_id, "58");
        assert_eq!(lu.total_annotated, 11);
        assert_eq!(lu.lexemes.len(), 1);
        assert_eq!(lu.lexemes[0].name, "divest");
    }

    #[test]
    fn test_core_type_parsing() {
        assert_eq!(
            match "Core".to_string().as_str() {
                "Core" => CoreType::Core,
                "Peripheral" => CoreType::Peripheral,
                "Extra-Thematic" => CoreType::ExtraThematic,
                _ => CoreType::Core,
            },
            CoreType::Core
        );
    }
}
