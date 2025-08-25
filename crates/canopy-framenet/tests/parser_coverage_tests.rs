//! Tests for FrameNet XML parser to achieve coverage targets

use canopy_engine::XmlResource;
use canopy_framenet::types::*;
use quick_xml::Reader;
use std::io::Cursor;

#[test]
fn test_parse_complete_frame() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<frame xmlns="http://framenet.icsi.berkeley.edu" ID="139" name="Giving" cBy="MJE" cDate="03/15/2002">
  <definition>&lt;def-root&gt;A &lt;fen&gt;Donor&lt;/fen&gt; gives a &lt;fen&gt;Theme&lt;/fen&gt; to a &lt;fen&gt;Recipient&lt;/fen&gt;.&lt;/def-root&gt;</definition>
  <FE abbrev="Donor" coreType="Core" cDate="03/15/2002" ID="1052" name="Donor" bgColor="FF0000" fgColor="FFFFFF" cBy="MJE">
    <definition>&lt;def-root&gt;The person that begins in control of the Theme.&lt;/def-root&gt;</definition>
    <semType name="Sentient" abbrev="Sen" ID="5"/>
  </FE>
  <FE abbrev="Theme" coreType="Core" cDate="03/15/2002" ID="1053" name="Theme" bgColor="0000FF" fgColor="FFFFFF">
    <definition>&lt;def-root&gt;The object that is given.&lt;/def-root&gt;</definition>
    <feRelation type="Requires" relatedFE="Donor" relatedFrame="139"/>
  </FE>
  <FE abbrev="Recipient" coreType="Peripheral" cDate="03/15/2002" ID="1054" name="Recipient">
    <definition>&lt;def-root&gt;The person that receives the Theme.&lt;/def-root&gt;</definition>
  </FE>
  <FE abbrev="Time" coreType="Extra-Thematic" cDate="03/15/2002" ID="1055" name="Time">
    <definition>&lt;def-root&gt;When the giving occurs.&lt;/def-root&gt;</definition>
  </FE>
  <frameRelation type="Inherits" relatedFrame="Transfer" relatedFrameName="Transfer"/>
  <frameRelation type="Uses" relatedFrame="Getting" relatedFrameName="Getting"/>
  <lexUnit ID="4289" name="give.v" POS="V" status="Finished"/>
  <lexUnit ID="4290" name="donate.v" POS="V" status="Created"/>
</frame>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let frame = Frame::parse_xml(&mut reader).unwrap();

    // Test frame attributes
    assert_eq!(frame.id, "139");
    assert_eq!(frame.name, "Giving");
    assert_eq!(frame.created_by, Some("MJE".to_string()));
    assert_eq!(frame.created_date, Some("03/15/2002".to_string()));

    // Test cleaned definition
    assert_eq!(frame.definition, "A Donor gives a Theme to a Recipient.");

    // Test frame elements
    assert_eq!(frame.frame_elements.len(), 4);

    let donor_fe = frame
        .frame_elements
        .iter()
        .find(|fe| fe.name == "Donor")
        .unwrap();
    assert_eq!(donor_fe.id, "1052");
    assert_eq!(donor_fe.abbrev, "Donor");
    assert_eq!(donor_fe.core_type, CoreType::Core);
    assert_eq!(donor_fe.bg_color, Some("FF0000".to_string()));
    assert_eq!(donor_fe.fg_color, Some("FFFFFF".to_string()));
    assert_eq!(donor_fe.created_by, Some("MJE".to_string()));
    assert_eq!(donor_fe.created_date, Some("03/15/2002".to_string()));
    assert_eq!(
        donor_fe.definition,
        "The person that begins in control of the Theme."
    );
    assert_eq!(donor_fe.semantic_types.len(), 1);
    assert_eq!(donor_fe.semantic_types[0].name, "Sentient");
    assert_eq!(donor_fe.semantic_types[0].id, "5");

    let theme_fe = frame
        .frame_elements
        .iter()
        .find(|fe| fe.name == "Theme")
        .unwrap();
    assert_eq!(theme_fe.core_type, CoreType::Core);
    assert_eq!(theme_fe.fe_relations.len(), 1);
    assert_eq!(theme_fe.fe_relations[0].relation_type, "Requires");
    assert_eq!(theme_fe.fe_relations[0].related_fe, "Donor");
    assert_eq!(theme_fe.fe_relations[0].related_frame, "139");

    let recipient_fe = frame
        .frame_elements
        .iter()
        .find(|fe| fe.name == "Recipient")
        .unwrap();
    assert_eq!(recipient_fe.core_type, CoreType::Peripheral);

    let time_fe = frame
        .frame_elements
        .iter()
        .find(|fe| fe.name == "Time")
        .unwrap();
    assert_eq!(time_fe.core_type, CoreType::ExtraThematic);

    // Test frame relations
    assert_eq!(frame.frame_relations.len(), 2);

    let inherits_rel = frame
        .frame_relations
        .iter()
        .find(|rel| rel.relation_type == "Inherits")
        .unwrap();
    assert_eq!(inherits_rel.related_frame_id, "Transfer");
    assert_eq!(inherits_rel.related_frame_name, "Transfer");

    let uses_rel = frame
        .frame_relations
        .iter()
        .find(|rel| rel.relation_type == "Uses")
        .unwrap();
    assert_eq!(uses_rel.related_frame_id, "Getting");
    assert_eq!(uses_rel.related_frame_name, "Getting");

    // Test lexical unit references
    assert_eq!(frame.lexical_units.len(), 2);

    let give_lu = frame
        .lexical_units
        .iter()
        .find(|lu| lu.name == "give.v")
        .unwrap();
    assert_eq!(give_lu.id, "4289");
    assert_eq!(give_lu.pos, "V");
    assert_eq!(give_lu.status, "Finished");

    let donate_lu = frame
        .lexical_units
        .iter()
        .find(|lu| lu.name == "donate.v")
        .unwrap();
    assert_eq!(donate_lu.id, "4290");
    assert_eq!(donate_lu.status, "Created");
}

#[test]
fn test_parse_complete_lexical_unit() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexUnit xmlns="http://framenet.icsi.berkeley.edu" ID="4289" name="give.v" POS="V" status="Finished" frame="Giving" frameID="139" totalAnnotated="150">
  <definition>To transfer possession of something to someone.</definition>
  <lexeme POS="V" name="give" headword="true" breakBefore="false"/>
  <lexeme POS="PART" name="up" headword="false" breakBefore="true"/>
  <valences>
    <FERealization total="120">
      <FE name="Donor"/>
      <pattern total="100">
        <valenceUnit GF="Ext" PT="NP"/>
      </pattern>
      <pattern total="20">
        <valenceUnit GF="Gen" PT="Poss"/>
      </pattern>
    </FERealization>
    <FERealization total="90">
      <FE name="Theme"/>
      <pattern total="90">
        <valenceUnit GF="Obj" PT="NP"/>
      </pattern>
    </FERealization>
  </valences>
  <subCorpus name="test">
    <!-- Skip subcorpus content -->
  </subCorpus>
</lexUnit>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let lu = LexicalUnit::parse_xml(&mut reader).unwrap();

    // Test LU attributes
    assert_eq!(lu.id, "4289");
    assert_eq!(lu.name, "give.v");
    assert_eq!(lu.pos, "V");
    assert_eq!(lu.status, "Finished");
    assert_eq!(lu.frame_name, "Giving");
    assert_eq!(lu.frame_id, "139");
    assert_eq!(lu.total_annotated, 150);
    assert_eq!(
        lu.definition,
        "To transfer possession of something to someone."
    );

    // Test lexemes
    assert_eq!(lu.lexemes.len(), 2);

    let give_lexeme = lu.lexemes.iter().find(|lex| lex.name == "give").unwrap();
    assert_eq!(give_lexeme.pos, "V");
    assert_eq!(give_lexeme.headword, Some(true));
    assert_eq!(give_lexeme.break_before, Some(false));

    let up_lexeme = lu.lexemes.iter().find(|lex| lex.name == "up").unwrap();
    assert_eq!(up_lexeme.pos, "PART");
    assert_eq!(up_lexeme.headword, Some(false));
    assert_eq!(up_lexeme.break_before, Some(true));

    // Test valences
    assert_eq!(lu.valences.len(), 2);

    let donor_valence = lu.valences.iter().find(|v| v.fe_name == "Donor").unwrap();
    assert_eq!(donor_valence.total, 120);
    assert_eq!(donor_valence.realizations.len(), 2);

    let ext_realization = donor_valence
        .realizations
        .iter()
        .find(|r| r.grammatical_function == "Ext")
        .unwrap();
    assert_eq!(ext_realization.phrase_type, "NP");
    assert_eq!(ext_realization.count, 100);

    let gen_realization = donor_valence
        .realizations
        .iter()
        .find(|r| r.grammatical_function == "Gen")
        .unwrap();
    assert_eq!(gen_realization.phrase_type, "Poss");
    assert_eq!(gen_realization.count, 20);

    let theme_valence = lu.valences.iter().find(|v| v.fe_name == "Theme").unwrap();
    assert_eq!(theme_valence.total, 90);
    assert_eq!(theme_valence.realizations.len(), 1);
    assert_eq!(theme_valence.realizations[0].grammatical_function, "Obj");
    assert_eq!(theme_valence.realizations[0].phrase_type, "NP");
    assert_eq!(theme_valence.realizations[0].count, 90);
}

#[test]
fn test_minimal_frame_parsing() {
    let xml = r#"<?xml version="1.0"?>
<frame ID="1" name="MinimalFrame">
  <definition>A minimal frame</definition>
</frame>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let frame = Frame::parse_xml(&mut reader).unwrap();

    assert_eq!(frame.id, "1");
    assert_eq!(frame.name, "MinimalFrame");
    assert_eq!(frame.definition, "A minimal frame");
    assert!(frame.frame_elements.is_empty());
    assert!(frame.frame_relations.is_empty());
    assert!(frame.lexical_units.is_empty());
    assert_eq!(frame.created_by, None);
    assert_eq!(frame.created_date, None);
}

#[test]
fn test_minimal_lexical_unit_parsing() {
    let xml = r#"<?xml version="1.0"?>
<lexUnit ID="1" name="test.v" POS="V" status="Created">
  <definition>A test word</definition>
</lexUnit>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let lu = LexicalUnit::parse_xml(&mut reader).unwrap();

    assert_eq!(lu.id, "1");
    assert_eq!(lu.name, "test.v");
    assert_eq!(lu.pos, "V");
    assert_eq!(lu.status, "Created");
    assert_eq!(lu.definition, "A test word");
    assert!(lu.lexemes.is_empty());
    assert!(lu.valences.is_empty());
    assert_eq!(lu.frame_id, "");
    assert_eq!(lu.frame_name, "");
    assert_eq!(lu.total_annotated, 0);
}

#[test]
fn test_error_handling_missing_frame_id() {
    let xml = r#"<?xml version="1.0"?>
<frame name="InvalidFrame">
  <definition>Frame without ID</definition>
</frame>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let result = Frame::parse_xml(&mut reader);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Frame missing required ID"));
}

#[test]
fn test_error_handling_missing_frame_name() {
    let xml = r#"<?xml version="1.0"?>
<frame ID="123">
  <definition>Frame without name</definition>
</frame>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let result = Frame::parse_xml(&mut reader);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Frame missing required name"));
}

#[test]
fn test_error_handling_missing_lu_id() {
    let xml = r#"<?xml version="1.0"?>
<lexUnit name="test.v" POS="V" status="Created">
  <definition>LU without ID</definition>
</lexUnit>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let result = LexicalUnit::parse_xml(&mut reader);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("LexicalUnit missing required ID"));
}

#[test]
fn test_malformed_xml_error() {
    let xml = r#"<?xml version="1.0"?>
<frame ID="123" name="Test">
  <definition>Unclosed definition
</frame>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let result = Frame::parse_xml(&mut reader);
    assert!(result.is_err());
}

#[test]
fn test_unexpected_eof_in_definition() {
    let xml = r#"<?xml version="1.0"?>
<frame ID="123" name="Test">
  <definition>Incomplete"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let result = Frame::parse_xml(&mut reader);
    assert!(result.is_err());
}

#[test]
fn test_core_type_variations() {
    let xml = r#"<?xml version="1.0"?>
<frame ID="123" name="CoreTypeTest">
  <definition>Testing core types</definition>
  <FE ID="1" name="Core" coreType="Core">
    <definition>Core element</definition>
  </FE>
  <FE ID="2" name="Peripheral" coreType="Peripheral">
    <definition>Peripheral element</definition>
  </FE>
  <FE ID="3" name="ExtraThematic" coreType="Extra-Thematic">
    <definition>Extra-thematic element</definition>
  </FE>
  <FE ID="4" name="Unknown" coreType="UnknownType">
    <definition>Unknown type defaults to Core</definition>
  </FE>
</frame>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let frame = Frame::parse_xml(&mut reader).unwrap();

    assert_eq!(frame.frame_elements.len(), 4);

    let core_fe = frame
        .frame_elements
        .iter()
        .find(|fe| fe.name == "Core")
        .unwrap();
    assert_eq!(core_fe.core_type, CoreType::Core);

    let peripheral_fe = frame
        .frame_elements
        .iter()
        .find(|fe| fe.name == "Peripheral")
        .unwrap();
    assert_eq!(peripheral_fe.core_type, CoreType::Peripheral);

    let extra_fe = frame
        .frame_elements
        .iter()
        .find(|fe| fe.name == "ExtraThematic")
        .unwrap();
    assert_eq!(extra_fe.core_type, CoreType::ExtraThematic);

    let unknown_fe = frame
        .frame_elements
        .iter()
        .find(|fe| fe.name == "Unknown")
        .unwrap();
    assert_eq!(unknown_fe.core_type, CoreType::Core); // defaults to Core
}

#[test]
fn test_self_closing_lexeme_tags() {
    let xml = r#"<?xml version="1.0"?>
<lexUnit ID="123" name="test.v" POS="V" status="Created">
  <definition>Test with self-closing lexemes</definition>
  <lexeme POS="V" name="test" headword="true"/>
  <lexeme POS="PART" name="out"/>
</lexUnit>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let lu = LexicalUnit::parse_xml(&mut reader).unwrap();

    assert_eq!(lu.lexemes.len(), 2);

    let test_lexeme = lu.lexemes.iter().find(|lex| lex.name == "test").unwrap();
    assert_eq!(test_lexeme.pos, "V");
    assert_eq!(test_lexeme.headword, Some(true));

    let out_lexeme = lu.lexemes.iter().find(|lex| lex.name == "out").unwrap();
    assert_eq!(out_lexeme.pos, "PART");
    assert_eq!(out_lexeme.headword, None);
}

#[test]
fn test_root_elements() {
    assert_eq!(Frame::root_element(), "frame");
    assert_eq!(LexicalUnit::root_element(), "lexUnit");
}

#[test]
fn test_parse_invalid_total_annotated() {
    let xml = r#"<?xml version="1.0"?>
<lexUnit ID="123" name="test.v" POS="V" status="Created" totalAnnotated="not_a_number">
  <definition>Test with invalid totalAnnotated</definition>
</lexUnit>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let lu = LexicalUnit::parse_xml(&mut reader).unwrap();
    assert_eq!(lu.total_annotated, 0); // defaults to 0 for invalid parse
}

#[test]
fn test_empty_valence_patterns() {
    let xml = r#"<?xml version="1.0"?>
<lexUnit ID="123" name="test.v" POS="V" status="Created">
  <definition>Test with empty valences</definition>
  <valences>
  </valences>
</lexUnit>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let lu = LexicalUnit::parse_xml(&mut reader).unwrap();
    assert!(lu.valences.is_empty());
}

#[test]
fn test_complex_definition_cleaning() {
    let xml = r#"<?xml version="1.0"?>
<frame ID="123" name="TestFrame">
  <definition>&lt;def-root&gt;A &lt;fen&gt;complex&lt;/fen&gt; definition with &lt;ex&gt;examples&lt;/ex&gt; and &lt;t&gt;targets&lt;/t&gt; &amp;amp; entities.&lt;/def-root&gt;</definition>
</frame>"#;

    let cursor = Cursor::new(xml);
    let mut reader = Reader::from_reader(cursor);

    let frame = Frame::parse_xml(&mut reader).unwrap();

    // Definition should be cleaned of FrameNet markup
    assert!(!frame.definition.contains("<def-root>"));
    assert!(!frame.definition.contains("<fen>"));
    assert!(!frame.definition.contains("<ex>"));
    assert!(!frame.definition.contains("<t>"));
    assert!(frame.definition.contains("complex"));
    assert!(frame.definition.contains("& entities"));
}
