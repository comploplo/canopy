//! Additional tests specifically targeting uncovered FrameNet engine functionality

use canopy_framenet::{DataLoader, FrameNetConfig, FrameNetEngine};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[cfg(test)]
mod engine_coverage_tests {
    use super::*;

    fn create_test_frames_xml() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "Giving.xml",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<frame xmlns="http://framenet.icsi.berkeley.edu" ID="139" name="Giving">
    <definition>A frame about giving actions</definition>
    <FE ID="1052" name="Donor" abbrev="Donor" coreType="Core">
        <definition>The giver</definition>
    </FE>
    <FE ID="1053" name="Theme" abbrev="Theme" coreType="Core">
        <definition>The thing given</definition>
    </FE>
    <FE ID="1054" name="Recipient" abbrev="Rec" coreType="Core">
        <definition>The receiver</definition>
    </FE>
</frame>"#,
            ),
            (
                "Event.xml",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<frame xmlns="http://framenet.icsi.berkeley.edu" ID="88" name="Event">
    <definition>A basic event frame</definition>
    <FE ID="2001" name="Event" abbrev="Evt" coreType="Core">
        <definition>The event that occurs</definition>
    </FE>
</frame>"#,
            ),
        ]
    }

    fn create_test_lus_xml() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "give.v.xml",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<lexUnit xmlns="http://framenet.icsi.berkeley.edu" frame="Giving" frameID="139" POS="V" name="give.v" ID="2477" status="Finished_Initial" totalAnnotated="100">
    <definition>To transfer possession</definition>
    <lexeme POS="V" name="give"/>
</lexUnit>"#,
            ),
            (
                "donate.v.xml",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<lexUnit xmlns="http://framenet.icsi.berkeley.edu" frame="Giving" frameID="139" POS="V" name="donate.v" ID="2478" status="Created" totalAnnotated="25">
    <definition>To give as a donation</definition>
    <lexeme POS="V" name="donate"/>
</lexUnit>"#,
            ),
            (
                "gift.v.xml",
                r#"<?xml version="1.0" encoding="UTF-8"?>
<lexUnit xmlns="http://framenet.icsi.berkeley.edu" frame="Giving" frameID="139" POS="V" name="gift.v" ID="2479" status="Finished_Initial" totalAnnotated="50">
    <definition>To present as a gift</definition>
    <lexeme POS="V" name="gift"/>
</lexUnit>"#,
            ),
        ]
    }

    fn create_test_data_files(temp_dir: &TempDir) -> std::io::Result<()> {
        let frames_dir = temp_dir.path().join("frame");
        let lu_dir = temp_dir.path().join("lu");

        fs::create_dir_all(&frames_dir)?;
        fs::create_dir_all(&lu_dir)?;

        // Create frame files
        for (filename, content) in create_test_frames_xml() {
            let mut file = fs::File::create(frames_dir.join(filename))?;
            file.write_all(content.as_bytes())?;
        }

        // Create LU files
        for (filename, content) in create_test_lus_xml() {
            let mut file = fs::File::create(lu_dir.join(filename))?;
            file.write_all(content.as_bytes())?;
        }

        Ok(())
    }

    #[test]
    fn test_engine_getter_methods() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        // Test get_frame method (uncovered line 249)
        let frame = engine.get_frame("139");
        assert!(frame.is_some());
        assert_eq!(frame.unwrap().name, "Giving");

        // Test get_frame with non-existent ID
        let non_existent = engine.get_frame("999");
        assert!(non_existent.is_none());

        // Test get_frame_by_name method (uncovered lines 253-255)
        let frame_by_name = engine.get_frame_by_name("Giving");
        assert!(frame_by_name.is_some());
        assert_eq!(frame_by_name.unwrap().id, "139");

        // Test get_frame_by_name with non-existent name
        let non_existent_name = engine.get_frame_by_name("NonExistent");
        assert!(non_existent_name.is_none());

        // Test get_lexical_unit method (uncovered lines 259-260)
        let lu = engine.get_lexical_unit("2477");
        assert!(lu.is_some());
        assert_eq!(lu.unwrap().name, "give.v");

        // Test get_lexical_unit with non-existent ID
        let non_existent_lu = engine.get_lexical_unit("9999");
        assert!(non_existent_lu.is_none());
    }

    #[test]
    fn test_confidence_calculation_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        // Test scenario that triggers line 178: (0, 0) case
        let empty_result = engine.analyze_text("nonexistentthing").unwrap();
        assert_eq!(empty_result.confidence, 0.0);

        // Test scenario that triggers line 180: (1, n) where n > 1
        // This tests "One frame, multiple LUs" case
        let multi_lu_result = engine.analyze_text("give donate gift").unwrap();
        // Should find multiple LUs for the Giving frame
        if !multi_lu_result.data.frames.is_empty() {
            // This should exercise the confidence calculation paths
            assert!(multi_lu_result.confidence > 0.0);
        }

        // Test scenario that could trigger line 181: (n, 1) where n > 1
        // This is "Multiple frames, one LU" - harder to create but we can test the logic exists
        let result = engine.analyze_text("give").unwrap();
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_duplicate_lu_handling() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        // Test a query that could potentially match the same LU multiple ways
        // This should exercise line 143: duplicate prevention logic
        let result = engine.analyze_text("give give").unwrap();

        // Even though we searched for "give give", duplicates should be removed
        let lu_ids: std::collections::HashSet<String> = result
            .data
            .lexical_units
            .iter()
            .map(|lu| lu.id.clone())
            .collect();

        // Should not have duplicate LUs
        assert_eq!(lu_ids.len(), result.data.lexical_units.len());
    }

    #[test]
    fn test_debug_logging_paths() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        // This should trigger the debug logging at lines 109-110
        let result = engine.analyze_text("give").unwrap();

        // Verify the analysis worked (which means debug logging was executed)
        assert!(!result.data.frames.is_empty());
        assert!(result.confidence > 0.0);

        // Test with multiple words to ensure various code paths are hit
        let multi_result = engine.analyze_text("give donate").unwrap();
        assert!(multi_result.confidence > 0.0);
    }

    #[test]
    fn test_lu_quality_bonus_calculation() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        // Test with LUs that have different annotation counts and statuses
        let result = engine.analyze_text("give").unwrap();

        // The give.v LU has status "Finished_Initial" and totalAnnotated="100"
        // This should get both annotation score bonus and status bonus
        // Testing that confidence calculation includes quality bonuses
        assert!(result.confidence > 0.5); // Should be reasonably high due to quality

        // Test with donate.v which has lower annotation count and different status
        let donate_result = engine.analyze_text("donate").unwrap();
        assert!(donate_result.confidence > 0.0);

        // give.v should have higher confidence than donate.v due to better quality metrics
        if !result.data.frames.is_empty() && !donate_result.data.frames.is_empty() {
            // This exercises the LU quality bonus calculation paths
            assert!(result.confidence >= donate_result.confidence);
        }
    }

    #[test]
    fn test_mixed_directory_loading() {
        let temp_dir = TempDir::new().unwrap();

        // Create files directly in temp_dir (not in frame/lu subdirs)
        // This should trigger the mixed directory loading path
        for (filename, content) in create_test_frames_xml() {
            let mut file = fs::File::create(temp_dir.path().join(filename)).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        for (filename, content) in create_test_lus_xml() {
            let mut file = fs::File::create(temp_dir.path().join(filename)).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }

        let mut engine = FrameNetEngine::new();

        // This should trigger load_mixed_directory method
        let result = engine.load_from_directory(temp_dir.path());
        assert!(result.is_ok());

        // Verify data was loaded
        assert!(engine.is_loaded());
        assert!(!engine.get_all_frames().is_empty());
        assert!(!engine.get_all_lexical_units().is_empty());
    }

    #[test]
    fn test_search_methods_coverage() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        // Test search_frames method
        let giving_frames = engine.search_frames("giving");
        assert!(!giving_frames.is_empty());
        assert!(giving_frames.iter().any(|f| f.name == "Giving"));

        // Test search with definition content
        let event_frames = engine.search_frames("event");
        assert!(event_frames.iter().any(|f| f.name == "Event"));

        // Test search_lexical_units method
        let give_lus = engine.search_lexical_units("give");
        assert!(!give_lus.is_empty());
        assert!(give_lus.iter().any(|lu| lu.name == "give.v"));

        // Test search by frame name
        let giving_lus = engine.search_lexical_units("Giving");
        assert!(giving_lus.len() >= 3); // Should find give.v, donate.v, gift.v
    }

    #[test]
    fn test_word_based_matching_paths() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut engine = FrameNetEngine::new();
        engine.load_from_directory(temp_dir.path()).unwrap();

        // Test multi-word input to exercise word-based matching
        let result = engine.analyze_text("I want to give you a gift").unwrap();

        // The word-based matching should have been exercised even if no LUs are found
        // Just verify the analysis completed successfully
        assert!(result.confidence >= 0.0);

        // Test simpler case that should definitely match
        let simple_result = engine.analyze_text("give").unwrap();
        assert!(simple_result.confidence > 0.0);

        // This exercises the word-based matching loop and base word extraction
        if !simple_result.data.lexical_units.is_empty() {
            let lu_names: Vec<String> = simple_result
                .data
                .lexical_units
                .iter()
                .map(|lu| lu.name.clone())
                .collect();
            assert!(lu_names.iter().any(|name| name.contains("give")));
        }
    }

    #[test]
    fn test_cache_behavior_coverage() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut config = FrameNetConfig::default();
        config.enable_cache = true;
        config.cache_capacity = 100;

        let mut engine = FrameNetEngine::with_config(config);
        engine.load_from_directory(temp_dir.path()).unwrap();

        // First query - should be a cache miss
        let result1 = engine.analyze_text("give").unwrap();
        assert!(!result1.from_cache);

        // Second identical query - should be a cache hit
        let result2 = engine.analyze_text("give").unwrap();
        assert!(result2.from_cache);

        // Results should be identical
        assert_eq!(result1.data.input, result2.data.input);
        assert_eq!(result1.confidence, result2.confidence);
    }

    #[test]
    fn test_parallel_loading_paths() {
        let temp_dir = TempDir::new().unwrap();
        create_test_data_files(&temp_dir).unwrap();

        let mut engine = FrameNetEngine::new();

        // This should exercise both frame and LU loading paths
        let result = engine.load_from_directory(temp_dir.path());
        assert!(result.is_ok());

        // Verify indices were built
        assert!(!engine.get_all_frames().is_empty());
        assert!(!engine.get_all_lexical_units().is_empty());

        // Verify frame name index works
        let frame_by_name = engine.get_frame_by_name("Giving");
        assert!(frame_by_name.is_some());

        // Test that analysis works after loading
        let analysis = engine.analyze_text("give");
        assert!(analysis.is_ok());
    }
}
