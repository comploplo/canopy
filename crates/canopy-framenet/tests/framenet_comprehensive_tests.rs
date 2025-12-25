//! Comprehensive tests for FrameNet functionality

use canopy_framenet::{
    utils, CoreType, DataLoader, Frame, FrameElement, FrameElementRealization, FrameNetAnalysis,
    FrameNetConfig, FrameNetEngine, FrameParser, FrameRelation, Lexeme, LexicalUnit,
    LexicalUnitRef, SemanticEngine, SemanticType, StatisticsProvider, SubcategorizationPattern,
    ValencePattern, ValenceUnit,
};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[cfg(test)]
mod framenet_tests {
    use super::*;

    // Helper function to create a test frame
    fn create_test_frame(id: &str, name: &str) -> Frame {
        Frame {
            id: id.to_string(),
            name: name.to_string(),
            created_by: Some("test".to_string()),
            created_date: Some("2023-01-01".to_string()),
            definition: format!("Test frame for {name}"),
            frame_elements: vec![
                FrameElement {
                    id: "1".to_string(),
                    name: "Agent".to_string(),
                    abbrev: "Agt".to_string(),
                    core_type: CoreType::Core,
                    bg_color: Some("FF0000".to_string()),
                    fg_color: Some("FFFFFF".to_string()),
                    created_by: Some("test".to_string()),
                    created_date: Some("2023-01-01".to_string()),
                    definition: "The entity that performs the action".to_string(),
                    semantic_types: vec![SemanticType {
                        name: "Sentient".to_string(),
                        id: "1".to_string(),
                    }],
                    fe_relations: vec![],
                },
                FrameElement {
                    id: "2".to_string(),
                    name: "Theme".to_string(),
                    abbrev: "Thm".to_string(),
                    core_type: CoreType::Core,
                    bg_color: Some("00FF00".to_string()),
                    fg_color: Some("000000".to_string()),
                    created_by: Some("test".to_string()),
                    created_date: Some("2023-01-01".to_string()),
                    definition: "The entity that is acted upon".to_string(),
                    semantic_types: vec![],
                    fe_relations: vec![],
                },
                FrameElement {
                    id: "3".to_string(),
                    name: "Manner".to_string(),
                    abbrev: "Man".to_string(),
                    core_type: CoreType::Peripheral,
                    bg_color: None,
                    fg_color: None,
                    created_by: Some("test".to_string()),
                    created_date: Some("2023-01-01".to_string()),
                    definition: "The manner of performing the action".to_string(),
                    semantic_types: vec![],
                    fe_relations: vec![],
                },
            ],
            frame_relations: vec![FrameRelation {
                relation_type: "Inheritance".to_string(),
                related_frame_id: "2".to_string(),
                related_frame_name: "Event".to_string(),
            }],
            lexical_units: vec![LexicalUnitRef {
                id: "1".to_string(),
                name: "give.v".to_string(),
                pos: "V".to_string(),
                status: "Created".to_string(),
            }],
        }
    }

    // Helper function to create test lexical unit
    fn create_test_lu(id: &str, name: &str, frame_name: &str) -> LexicalUnit {
        LexicalUnit {
            id: id.to_string(),
            name: name.to_string(),
            pos: "V".to_string(),
            status: "Created".to_string(),
            frame_id: "1".to_string(),
            frame_name: frame_name.to_string(),
            total_annotated: 50,
            definition: format!("Test lexical unit {name}"),
            lexemes: vec![Lexeme {
                pos: "V".to_string(),
                name: name.split('.').next().unwrap_or(name).to_string(),
                break_before: Some(false),
                headword: Some(true),
            }],
            valences: vec![ValencePattern {
                fe_name: "Agent".to_string(),
                total: 25,
                realizations: vec![FrameElementRealization {
                    grammatical_function: "Ext".to_string(),
                    phrase_type: "NP".to_string(),
                    count: 20,
                }],
            }],
            subcategorization: vec![SubcategorizationPattern {
                id: "1".to_string(),
                total: 30,
                valence_units: vec![ValenceUnit {
                    fe: "Agent".to_string(),
                    pt: "NP".to_string(),
                    gf: "Ext".to_string(),
                }],
            }],
        }
    }

    fn create_test_xml_files(temp_dir: &TempDir) -> std::io::Result<()> {
        let frames_dir = temp_dir.path().join("frame");
        let lu_dir = temp_dir.path().join("lu");

        fs::create_dir_all(&frames_dir)?;
        fs::create_dir_all(&lu_dir)?;

        // Create a simple frame XML file
        let frame_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<frame xmlns="http://framenet.icsi.berkeley.edu" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" cBy="test" cDate="01/01/2023" name="Giving" ID="1">
    <definition>A frame about giving actions</definition>
    <FE bgColor="FF0000" fgColor="FFFFFF" coreType="Core" cBy="test" cDate="01/01/2023" abbrev="Agt" name="Agent" ID="1">
        <definition>The entity that gives</definition>
    </FE>
    <FE bgColor="00FF00" fgColor="000000" coreType="Core" cBy="test" cDate="01/01/2023" abbrev="Thm" name="Theme" ID="2">
        <definition>The entity that is given</definition>
    </FE>
    <lexUnit status="Created" POS="V" name="give.v" ID="1" lemmaID="1" cBy="test" cDate="01/01/2023" totalAnnotated="100">
        <definition>To transfer possession</definition>
        <lexeme POS="V" name="give" order="1" headword="true" breakBefore="false"/>
    </lexUnit>
    <frameRelation type="Inheritance">
        <relatedFrame ID="2">Event</relatedFrame>
    </frameRelation>
</frame>"#;

        let mut frame_file = fs::File::create(frames_dir.join("Giving.xml"))?;
        frame_file.write_all(frame_xml.as_bytes())?;

        // Create a simple LU XML file
        let lu_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexUnit xmlns="http://framenet.icsi.berkeley.edu" frame="Giving" frameID="1" POS="V" name="give.v" ID="1" cBy="test" cDate="01/01/2023" status="Created" totalAnnotated="100">
    <definition>To transfer possession of something to someone</definition>
    <lexeme POS="V" name="give" order="1" headword="true" breakBefore="false"/>
    <valences>
        <valence ID="1" total="50">
            <FE name="Agent" total="45"/>
            <FE name="Theme" total="50"/>
            <FE name="Recipient" total="40"/>
        </valence>
    </valences>
</lexUnit>"#;

        let mut lu_file = fs::File::create(lu_dir.join("give.v.xml"))?;
        lu_file.write_all(lu_xml.as_bytes())?;

        Ok(())
    }

    #[test]
    fn test_framenet_config_creation() {
        let config = FrameNetConfig::default();

        assert!(config.enable_cache);
        assert!(config.cache_capacity > 0);
        assert!(config.confidence_threshold >= 0.0);
        assert!(config.confidence_threshold <= 1.0);
    }

    #[test]
    fn test_framenet_config_custom() {
        let custom_config = FrameNetConfig {
            frames_path: "test/frames".to_string(),
            lexical_units_path: "test/lu".to_string(),
            enable_cache: false,
            cache_capacity: 500,
            confidence_threshold: 0.8,
            settings: std::collections::HashMap::new(),
        };

        assert!(!custom_config.enable_cache);
        assert_eq!(custom_config.cache_capacity, 500);
        assert_eq!(custom_config.confidence_threshold, 0.8);
        assert_eq!(custom_config.frames_path, "test/frames");
        assert_eq!(custom_config.lexical_units_path, "test/lu");
    }

    #[test]
    fn test_framenet_engine_creation() {
        // Engine auto-loads data on creation with proper path resolution
        let engine = match FrameNetEngine::new() {
            Ok(e) => e,
            Err(e) => {
                println!("Skipping test: FrameNet data not available: {}", e);
                return;
            }
        };

        // Engine should be loaded after creation
        assert!(engine.is_loaded());
        assert!(engine.is_initialized());

        // Test Debug trait
        let debug_str = format!("{:?}", engine);
        assert!(debug_str.contains("FrameNetEngine"));
    }

    #[test]
    fn test_framenet_engine_with_invalid_config() {
        // Config with invalid paths should fail to create engine
        let config = FrameNetConfig {
            frames_path: "nonexistent/frames".to_string(),
            lexical_units_path: "nonexistent/lu".to_string(),
            enable_cache: false,
            cache_capacity: 100,
            confidence_threshold: 0.5,
            settings: std::collections::HashMap::new(),
        };

        // Engine creation should fail with invalid paths
        let result = FrameNetEngine::with_config(config);
        assert!(
            result.is_err(),
            "Engine creation should fail with invalid paths"
        );
    }

    #[test]
    fn test_frame_creation_and_methods() {
        let frame = create_test_frame("1", "Giving");

        assert_eq!(frame.id, "1");
        assert_eq!(frame.name, "Giving");
        assert_eq!(frame.frame_elements.len(), 3);
        assert_eq!(frame.frame_relations.len(), 1);
        assert_eq!(frame.lexical_units.len(), 1);

        // Test core elements
        let core_elements = frame.core_elements();
        assert_eq!(core_elements.len(), 2);

        // Test peripheral elements
        let peripheral_elements = frame.peripheral_elements();
        assert_eq!(peripheral_elements.len(), 1);

        // Test element lookup
        assert!(frame.has_frame_element("Agent"));
        assert!(frame.has_frame_element("Theme"));
        assert!(!frame.has_frame_element("NonExistent"));

        let agent_fe = frame.get_frame_element("Agent");
        assert!(agent_fe.is_some());
        assert_eq!(agent_fe.unwrap().name, "Agent");
    }

    #[test]
    fn test_frame_element_types() {
        let frame = create_test_frame("1", "Test");

        let agent = &frame.frame_elements[0];
        let theme = &frame.frame_elements[1];
        let manner = &frame.frame_elements[2];

        // Test core type checks
        assert!(agent.is_core());
        assert!(!agent.is_peripheral());
        assert!(!agent.is_extra_thematic());

        assert!(theme.is_core());
        assert!(!theme.is_peripheral());

        assert!(!manner.is_core());
        assert!(manner.is_peripheral());
        assert!(!manner.is_extra_thematic());
    }

    #[test]
    fn test_frame_element_colors() {
        let frame = create_test_frame("1", "Test");
        let agent = &frame.frame_elements[0];

        assert_eq!(agent.bg_color, Some("FF0000".to_string()));
        assert_eq!(agent.fg_color, Some("FFFFFF".to_string()));

        // Test color parsing via utils
        if let Some(bg_color) = &agent.bg_color {
            assert_eq!(utils::parse_fe_color(bg_color), Some((255, 0, 0)));
        }
        if let Some(fg_color) = &agent.fg_color {
            assert_eq!(utils::parse_fe_color(fg_color), Some((255, 255, 255)));
        }
    }

    #[test]
    fn test_lexical_unit_creation() {
        let lu = create_test_lu("1", "give.v", "Giving");

        assert_eq!(lu.id, "1");
        assert_eq!(lu.name, "give.v");
        assert_eq!(lu.frame_name, "Giving");
        assert_eq!(lu.total_annotated, 50);
        assert_eq!(lu.lexemes.len(), 1);
        assert_eq!(lu.valences.len(), 1);

        // Test base word extraction via utils
        assert_eq!(utils::extract_base_word(&lu.name), "give");

        // Test POS type
        assert_eq!(lu.pos, "V");
        assert!(lu.belongs_to_frame("Giving"));
    }

    #[test]
    fn test_framenet_analysis_creation() {
        let frames = vec![create_test_frame("1", "Giving")];
        let analysis = FrameNetAnalysis::new("give".to_string(), frames, 0.85);

        assert_eq!(analysis.input, "give");
        assert_eq!(analysis.confidence, 0.85);
        assert_eq!(analysis.frames.len(), 1);
        assert_eq!(analysis.frames[0].name, "Giving");

        // Test analysis methods
        assert!(!analysis.frames.is_empty());
        assert_eq!(analysis.frames.len(), 1);

        let primary_frame = analysis.primary_frame();
        assert!(primary_frame.is_some());
        assert_eq!(primary_frame.unwrap().name, "Giving");
    }

    #[test]
    fn test_framenet_utils_extract_base_word() {
        assert_eq!(utils::extract_base_word("give.v"), "give");
        assert_eq!(utils::extract_base_word("run_away.v"), "run_away");
        assert_eq!(utils::extract_base_word("simple"), "simple");
        assert_eq!(utils::extract_base_word("complex.a.with.dots"), "complex");
    }

    #[test]
    fn test_framenet_utils_lu_matches_word() {
        assert!(utils::lu_matches_word("give.v", "give"));
        assert!(utils::lu_matches_word("GIVE.V", "give"));
        assert!(utils::lu_matches_word("give.v", "GIVE"));
        assert!(!utils::lu_matches_word("take.v", "give"));
        assert!(!utils::lu_matches_word("give.n", "giving"));
    }

    #[test]
    fn test_framenet_utils_frame_operations() {
        let frame = create_test_frame("1", "Giving");

        // Test frame has element
        assert!(utils::frame_has_element(&frame, "Agent"));
        assert!(utils::frame_has_element(&frame, "Theme"));
        assert!(!utils::frame_has_element(&frame, "NonExistent"));

        // Test get core elements
        let core_elements = utils::get_core_elements(&frame);
        assert_eq!(core_elements.len(), 2);
        assert!(core_elements.iter().any(|fe| fe.name == "Agent"));
        assert!(core_elements.iter().any(|fe| fe.name == "Theme"));
        assert!(!core_elements.iter().any(|fe| fe.name == "Manner"));
    }

    #[test]
    fn test_framenet_utils_lu_operations() {
        let lus = vec![
            create_test_lu("1", "give.v", "Giving"),
            create_test_lu("2", "donate.v", "Giving"),
            create_test_lu("3", "take.v", "Taking"),
        ];

        // Test filter LUs by frame
        let giving_lus = utils::filter_lus_by_frame(&lus, "Giving");
        assert_eq!(giving_lus.len(), 2);
        assert!(giving_lus.iter().any(|lu| lu.name == "give.v"));
        assert!(giving_lus.iter().any(|lu| lu.name == "donate.v"));

        let taking_lus = utils::filter_lus_by_frame(&lus, "Taking");
        assert_eq!(taking_lus.len(), 1);
        assert_eq!(taking_lus[0].name, "take.v");
    }

    #[test]
    fn test_framenet_utils_most_annotated_lu() {
        let mut lu1 = create_test_lu("1", "give.v", "Giving");
        let mut lu2 = create_test_lu("2", "donate.v", "Giving");
        let mut lu3 = create_test_lu("3", "present.v", "Giving");

        lu1.total_annotated = 100;
        lu2.total_annotated = 150;
        lu3.total_annotated = 75;

        let lus = vec![lu1, lu2, lu3];
        let most_annotated = utils::most_annotated_lu(&lus);

        assert!(most_annotated.is_some());
        assert_eq!(most_annotated.unwrap().name, "donate.v");
        assert_eq!(most_annotated.unwrap().total_annotated, 150);
    }

    #[test]
    fn test_framenet_utils_color_parsing() {
        // Test valid colors
        assert_eq!(utils::parse_fe_color("FF0000"), Some((255, 0, 0)));
        assert_eq!(utils::parse_fe_color("00FF00"), Some((0, 255, 0)));
        assert_eq!(utils::parse_fe_color("0000FF"), Some((0, 0, 255)));
        assert_eq!(utils::parse_fe_color("FFFFFF"), Some((255, 255, 255)));
        assert_eq!(utils::parse_fe_color("000000"), Some((0, 0, 0)));

        // Test invalid colors
        assert_eq!(utils::parse_fe_color("invalid"), None);
        assert_eq!(utils::parse_fe_color("FF"), None);
        assert_eq!(utils::parse_fe_color("FFGGBB"), None);
        assert_eq!(utils::parse_fe_color(""), None);
    }

    #[test]
    fn test_framenet_utils_frame_relations() {
        let frame1 = create_test_frame("1", "Giving");
        let mut frame2 = create_test_frame("2", "Event");

        // Initially not related (except for the inheritance we set up)
        assert!(utils::frames_are_related(&frame1, &frame2));

        // Add reverse relation
        frame2.frame_relations.push(FrameRelation {
            relation_type: "Used_by".to_string(),
            related_frame_id: "1".to_string(),
            related_frame_name: "Giving".to_string(),
        });

        assert!(utils::frames_are_related(&frame1, &frame2));
        assert!(utils::frames_are_related(&frame2, &frame1));

        // Test unrelated frames
        let frame3 = create_test_frame("3", "Unrelated");
        assert!(!utils::frames_are_related(&frame1, &frame3));
    }

    #[test]
    fn test_core_type_serialization() {
        // Test that CoreType can be serialized/deserialized
        let core = CoreType::Core;
        let peripheral = CoreType::Peripheral;
        let extra_thematic = CoreType::ExtraThematic;

        // Test JSON serialization
        let core_json = serde_json::to_string(&core).unwrap();
        let peripheral_json = serde_json::to_string(&peripheral).unwrap();
        let extra_json = serde_json::to_string(&extra_thematic).unwrap();

        assert!(core_json.contains("Core"));
        assert!(peripheral_json.contains("Peripheral"));
        assert!(extra_json.contains("Extra-Thematic"));

        // Test deserialization
        let core_back: CoreType = serde_json::from_str(&core_json).unwrap();
        let peripheral_back: CoreType = serde_json::from_str(&peripheral_json).unwrap();
        let extra_back: CoreType = serde_json::from_str(&extra_json).unwrap();

        assert_eq!(core, core_back);
        assert_eq!(peripheral, peripheral_back);
        assert_eq!(extra_thematic, extra_back);
    }

    #[test]
    fn test_frame_parser_creation() {
        let parser = FrameParser;

        // Test that parser exists (FrameParser is a unit struct)
        // Since FrameParser doesn't implement Debug, we just test its existence
        let _parser = parser; // Use the parser to avoid unused variable warning
    }

    #[test]
    fn test_engine_with_loaded_data() {
        // Engine auto-loads data on creation
        let engine = match FrameNetEngine::new() {
            Ok(e) => e,
            Err(e) => {
                println!("Skipping test: FrameNet data not available: {}", e);
                return;
            }
        };

        // Engine should be loaded
        assert!(engine.is_loaded());
        assert!(engine.is_initialized());

        // Test get methods return data
        assert!(!engine.get_all_frames().is_empty());

        let stats = engine.statistics();
        assert!(stats.total_frames > 0, "Should have loaded frames");
    }

    #[test]
    fn test_engine_analyze_text() {
        // Engine auto-loads on creation
        let engine = match FrameNetEngine::new() {
            Ok(e) => e,
            Err(e) => {
                println!("Skipping test: FrameNet data not available: {}", e);
                return;
            }
        };

        // Analyzing text with loaded engine
        let result = engine.analyze_text("give");
        assert!(result.is_ok());

        let analysis = result.unwrap();
        // May or may not find frames depending on LU coverage
        assert!(analysis.confidence >= 0.0);
    }

    #[test]
    fn test_engine_statistics_tracking() {
        // Engine auto-loads on creation
        let engine = match FrameNetEngine::new() {
            Ok(e) => e,
            Err(e) => {
                println!("Skipping test: FrameNet data not available: {}", e);
                return;
            }
        };

        // Test that statistics reflect loaded data
        let initial_stats = engine.statistics();
        assert!(initial_stats.total_frames > 0, "Should have loaded frames");

        // Test that we can analyze text
        let result = engine.analyze_text("test");
        assert!(result.is_ok());

        // Verify statistics are still accessible after analysis
        let after_stats = engine.statistics();
        assert!(
            after_stats.total_frames > 0,
            "Should still have frames after analysis"
        );
    }

    #[test]
    fn test_engine_caching_behavior() {
        let mut config = FrameNetConfig::default();
        config.enable_cache = true;
        config.cache_capacity = 100;

        let mut engine = FrameNetEngine::with_config(config).unwrap();

        // First analysis
        let result1 = engine.analyze_text("test");
        assert!(result1.is_ok());

        // Second analysis should potentially use cache
        let result2 = engine.analyze_text("test");
        assert!(result2.is_ok());

        // Results should be consistent
        let analysis1 = result1.unwrap();
        let analysis2 = result2.unwrap();
        assert_eq!(analysis1.data.input, analysis2.data.input);
    }

    #[test]
    fn test_engine_with_real_data() {
        // Engine auto-loads on creation
        let engine = match FrameNetEngine::new() {
            Ok(e) => e,
            Err(e) => {
                println!("Skipping test: FrameNet data not available: {}", e);
                return;
            }
        };

        // Engine should be loaded
        assert!(engine.is_loaded());
        assert!(engine.is_initialized());

        // Should have loaded some data
        let all_frames = engine.get_all_frames();
        let all_lus = engine.get_all_lexical_units();
        assert!(
            !all_frames.is_empty() || !all_lus.is_empty(),
            "Should have loaded frames or lexical units"
        );

        // Test analysis with loaded data
        let analysis_result = engine.analyze_text("give");
        assert!(analysis_result.is_ok());
    }

    #[test]
    fn test_engine_load_invalid_directory() {
        // Engine auto-loads on creation
        let mut engine = match FrameNetEngine::new() {
            Ok(e) => e,
            Err(e) => {
                println!("Skipping test: FrameNet data not available: {}", e);
                return;
            }
        };

        // Engine is already loaded from creation
        assert!(engine.is_loaded());

        // Loading from invalid directory should fail
        let result = engine.load_from_directory("/nonexistent/path");
        assert!(result.is_err(), "Loading from invalid path should fail");

        // Engine may still be loaded from original load
        // (depends on implementation - loading failure doesn't unload)
    }

    #[test]
    fn test_semantic_type_operations() {
        let semantic_type = SemanticType {
            name: "Sentient".to_string(),
            id: "1".to_string(),
        };

        assert_eq!(semantic_type.name, "Sentient");
        assert_eq!(semantic_type.id, "1");

        // Test serialization
        let json = serde_json::to_string(&semantic_type).unwrap();
        let deserialized: SemanticType = serde_json::from_str(&json).unwrap();
        assert_eq!(semantic_type, deserialized);
    }

    #[test]
    fn test_frame_relations() {
        let relation = FrameRelation {
            relation_type: "Inheritance".to_string(),
            related_frame_id: "2".to_string(),
            related_frame_name: "Event".to_string(),
        };

        assert_eq!(relation.relation_type, "Inheritance");
        assert_eq!(relation.related_frame_id, "2");
        assert_eq!(relation.related_frame_name, "Event");

        // Test serialization
        let json = serde_json::to_string(&relation).unwrap();
        let deserialized: FrameRelation = serde_json::from_str(&json).unwrap();
        assert_eq!(relation, deserialized);
    }

    #[test]
    fn test_concurrent_engine_operations() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let engine = Arc::new(Mutex::new(FrameNetEngine::new().unwrap()));
        let mut handles = vec![];

        // Test concurrent analysis (simulated)
        for i in 0..5 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                let mut eng = engine_clone.lock().unwrap();
                let result = eng.analyze_text(&format!("test{}", i));
                result.is_ok()
            });
            handles.push(handle);
        }

        // All analyses should succeed
        for handle in handles {
            let success = handle.join().unwrap();
            assert!(success);
        }
    }

    #[test]
    fn test_valence_pattern_operations() {
        let lu = create_test_lu("1", "give.v", "Giving");

        assert_eq!(lu.valences.len(), 1);
        let valence = &lu.valences[0];
        assert_eq!(valence.fe_name, "Agent");
        assert_eq!(valence.total, 25);
    }
}
