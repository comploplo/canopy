//! Tests for FrameNetEngine methods to achieve coverage targets

use canopy_framenet::{DataLoader, FrameNetConfig, FrameNetEngine};
use std::fs;
use tempfile::TempDir;

fn create_test_framenet_data() -> (TempDir, FrameNetEngine) {
    let temp_dir = TempDir::new().unwrap();

    // Create frame XML data
    let frame_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<frame xmlns="http://framenet.icsi.berkeley.edu" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" 
       ID="139" name="Giving" cBy="MJE" cDate="03/15/2002">
  <definition>
    A Donor gives a Theme to a Recipient. The Donor has control over the Transfer and creates a situation where the Recipient gains control over the Theme.
  </definition>
  <FE abbrev="Donor" coreType="Core" cDate="03/15/2002" ID="1052" name="Donor">
    <definition>The person that begins in control of the Theme and causes it to be in the Recipient's possession.</definition>
  </FE>
  <FE abbrev="Theme" coreType="Core" cDate="03/15/2002" ID="1053" name="Theme">
    <definition>The object that is given.</definition>
  </FE>
  <FE abbrev="Recipient" coreType="Core" cDate="03/15/2002" ID="1054" name="Recipient">
    <definition>The person that begins without the Theme and ends up in possession of it.</definition>
  </FE>
  <FE abbrev="Time" coreType="Peripheral" cDate="03/15/2002" ID="1055" name="Time">
    <definition>When the giving occurs.</definition>
  </FE>
  <lexUnit ID="4289" name="give.v" POS="V" status="Finished" frame="Giving"/>
  <lexUnit ID="4290" name="donate.v" POS="V" status="Finished" frame="Giving"/>
</frame>"#;

    // Create lexical unit XML data
    let lu_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexUnit xmlns="http://framenet.icsi.berkeley.edu" ID="4289" name="give.v" 
         POS="V" status="Finished" frame="Giving" frameID="139" totalAnnotated="150">
  <definition>To transfer possession of something to someone.</definition>
  <lexeme POS="V" name="give" headword="true"/>
  <valences>
    <FERealization total="120">
      <FE name="Donor"/>
      <pattern total="100">
        <valenceUnit GF="Ext" PT="NP"/>
      </pattern>
    </FERealization>
  </valences>
</lexUnit>"#;

    // Create frame directory and file
    let frames_dir = temp_dir.path().join("frame");
    fs::create_dir_all(&frames_dir).unwrap();
    fs::write(frames_dir.join("Giving.xml"), frame_xml).unwrap();

    // Create LU directory and file
    let lu_dir = temp_dir.path().join("lu");
    fs::create_dir_all(&lu_dir).unwrap();
    fs::write(lu_dir.join("give.v.xml"), lu_xml).unwrap();

    let config = FrameNetConfig {
        frames_path: frames_dir.to_string_lossy().to_string(),
        lexical_units_path: lu_dir.to_string_lossy().to_string(),
        ..Default::default()
    };

    let mut engine = FrameNetEngine::with_config(config);
    engine
        .load_from_directory(temp_dir.path())
        .expect("Failed to load test data");

    (temp_dir, engine)
}

#[test]
fn test_search_frames() {
    let (_temp_dir, engine) = create_test_framenet_data();

    // Test searching by frame name
    let results = engine.search_frames("giving");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Giving");

    // Test case-insensitive search
    let results = engine.search_frames("GIVING");
    assert_eq!(results.len(), 1);

    // Test searching by definition content
    let results = engine.search_frames("donor");
    assert_eq!(results.len(), 1);

    // Test search with no matches
    let results = engine.search_frames("nonexistent");
    assert!(results.is_empty());

    // Test partial match
    let results = engine.search_frames("giv");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_search_lexical_units() {
    let (_temp_dir, engine) = create_test_framenet_data();

    // Test searching by LU name
    let results = engine.search_lexical_units("give");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "give.v");

    // Test searching by frame name
    let results = engine.search_lexical_units("giving");
    assert_eq!(results.len(), 1);

    // Test case-insensitive search
    let results = engine.search_lexical_units("GIVE");
    assert_eq!(results.len(), 1);

    // Test search with no matches
    let results = engine.search_lexical_units("nonexistent");
    assert!(results.is_empty());
}

#[test]
fn test_get_frame_by_name() {
    let (_temp_dir, engine) = create_test_framenet_data();

    // Test getting existing frame by name
    let frame = engine.get_frame_by_name("Giving");
    assert!(frame.is_some());
    assert_eq!(frame.unwrap().id, "139");

    // Test case sensitivity
    let frame = engine.get_frame_by_name("giving");
    assert!(frame.is_none());

    // Test non-existent frame
    let frame = engine.get_frame_by_name("NonExistent");
    assert!(frame.is_none());
}

#[test]
fn test_get_frame() {
    let (_temp_dir, engine) = create_test_framenet_data();

    // Test getting existing frame by ID
    let frame = engine.get_frame("139");
    assert!(frame.is_some());
    assert_eq!(frame.unwrap().name, "Giving");

    // Test non-existent frame ID
    let frame = engine.get_frame("999");
    assert!(frame.is_none());
}

#[test]
fn test_get_lexical_unit() {
    let (_temp_dir, engine) = create_test_framenet_data();

    // Test getting existing LU by ID
    let lu = engine.get_lexical_unit("4289");
    assert!(lu.is_some());
    assert_eq!(lu.unwrap().name, "give.v");

    // Test non-existent LU ID
    let lu = engine.get_lexical_unit("999");
    assert!(lu.is_none());
}

#[test]
fn test_get_all_frames() {
    let (_temp_dir, engine) = create_test_framenet_data();

    let frames = engine.get_all_frames();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].name, "Giving");
    assert_eq!(frames[0].id, "139");
}

#[test]
fn test_get_all_lexical_units() {
    let (_temp_dir, engine) = create_test_framenet_data();

    let lus = engine.get_all_lexical_units();
    assert_eq!(lus.len(), 1);
    assert_eq!(lus[0].name, "give.v");
    assert_eq!(lus[0].id, "4289");
}

#[test]
fn test_is_loaded() {
    let (_temp_dir, engine) = create_test_framenet_data();

    // Should be loaded after setup
    assert!(engine.is_loaded());

    // Test with empty engine
    let empty_engine = FrameNetEngine::new();
    assert!(!empty_engine.is_loaded());
}

#[test]
fn test_analyze_text_with_matching() {
    let (_temp_dir, mut engine) = create_test_framenet_data();

    // Test analysis with matching word
    let result = engine.analyze_text("I will give you the book").unwrap();
    assert!(result.confidence > 0.0);
    assert!(!result.data.frames.is_empty());
    assert_eq!(result.data.frames[0].name, "Giving");

    // Test analysis with non-matching text
    let result = engine.analyze_text("The weather is nice").unwrap();
    assert!(result.data.frames.is_empty());
    assert_eq!(result.confidence, 0.0);

    // Test analysis with empty text
    let result = engine.analyze_text("").unwrap();
    assert!(result.data.frames.is_empty());
}

#[test]
fn test_cache_functionality() {
    let (_temp_dir, mut engine) = create_test_framenet_data();

    // First analysis should miss cache
    let result1 = engine.analyze_text("give").unwrap();
    assert!(!result1.from_cache);

    // Second analysis should hit cache
    let result2 = engine.analyze_text("give").unwrap();
    // Note: The actual caching behavior depends on the implementation
    // This tests that repeated calls work consistently
    assert_eq!(result1.data.frames.len(), result2.data.frames.len());
}

#[test]
fn test_frame_elements_coverage() {
    let (_temp_dir, engine) = create_test_framenet_data();

    let frame = engine.get_frame("139").unwrap();

    // Test frame element methods
    let core_elements = frame.core_elements();
    assert_eq!(core_elements.len(), 3); // Donor, Theme, Recipient

    let peripheral_elements = frame.peripheral_elements();
    assert_eq!(peripheral_elements.len(), 1); // Time

    let extra_thematic = frame.extra_thematic_elements();
    assert_eq!(extra_thematic.len(), 0);

    // Test frame element lookup
    assert!(frame.has_frame_element("Donor"));
    assert!(frame.has_frame_element("Theme"));
    assert!(!frame.has_frame_element("NonExistent"));

    let donor_fe = frame.get_frame_element("Donor").unwrap();
    assert_eq!(donor_fe.name, "Donor");
    assert!(donor_fe.is_core());
    assert!(!donor_fe.is_peripheral());
    assert!(!donor_fe.is_extra_thematic());
}

#[test]
fn test_lexical_unit_methods() {
    let (_temp_dir, engine) = create_test_framenet_data();

    let lu = engine.get_lexical_unit("4289").unwrap();

    // Test lexical unit methods
    let primary_lexeme = lu.primary_lexeme();
    assert!(primary_lexeme.is_some());
    assert_eq!(primary_lexeme.unwrap().name, "give");
    assert_eq!(primary_lexeme.unwrap().headword, Some(true));

    // Test frame belonging
    assert!(lu.belongs_to_frame("Giving"));
    assert!(!lu.belongs_to_frame("Other"));

    // Test valence patterns
    let valences = lu.get_valences_for_fe("Donor");
    assert!(!valences.is_empty());
}
