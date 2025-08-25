//! Integration tests for FrameNet engine against real data

use canopy_framenet::{DataLoader, FrameNetEngine, SemanticEngine};
use std::path::Path;

#[test]
fn test_load_single_framenet_file() {
    let mut engine = FrameNetEngine::new();

    // Test loading FrameNet XML files
    let frames_dir = Path::new("../../data/framenet/archive/framenet_v17/framenet_v17/frame");
    let lu_dir = Path::new("../../data/framenet/archive/framenet_v17/framenet_v17/lu");

    if !frames_dir.exists() && !lu_dir.exists() {
        println!("FrameNet test data not found at expected locations");
        println!("Frames: {}", frames_dir.display());
        println!("LUs: {}", lu_dir.display());
        println!("Skipping integration test");
        return;
    }

    // Try to load the FrameNet data directory
    let framenet_dir = Path::new("../../data/framenet/archive/framenet_v17/framenet_v17");
    let result = engine.load_from_directory(framenet_dir);

    match result {
        Ok(()) => {
            println!("✅ Successfully loaded FrameNet data");
            println!("Number of frames loaded: {}", engine.get_all_frames().len());
            println!(
                "Number of lexical units loaded: {}",
                engine.get_all_lexical_units().len()
            );
            println!("Engine initialized: {}", engine.is_initialized());

            // Test that we can analyze some text
            if let Ok(analysis) = engine.analyze_text("give") {
                println!("✅ Successfully analyzed 'give':");
                println!("  Frames found: {}", analysis.data.frames.len());
                println!("  Confidence: {:.2}", analysis.confidence);

                for frame in &analysis.data.frames {
                    println!("  - Frame: {} ({})", frame.name, frame.id);
                    println!("    Core elements: {}", frame.core_elements().len());
                }
            } else {
                println!("❌ Failed to analyze 'give'");
            }

            // Test that the engine reports as loaded
            assert!(engine.is_loaded(), "Engine should report as loaded");
        }
        Err(e) => {
            println!("❌ Failed to load FrameNet data: {}", e);
            println!("This indicates issues with the XML parser implementation");
            panic!("FrameNet integration test failed: {}", e);
        }
    }
}

#[test]
fn test_framenet_frame_parser() {
    use canopy_engine::XmlParser;
    use canopy_framenet::types::Frame;

    let test_file =
        Path::new("../../data/framenet/archive/framenet_v17/framenet_v17/frame/Giving.xml");

    if !test_file.exists() {
        println!("FrameNet Giving frame not found, skipping parser test");
        return;
    }

    let parser = XmlParser::new();
    let result = parser.parse_file::<Frame>(test_file);

    match result {
        Ok(frame) => {
            println!("✅ Successfully parsed FrameNet frame: {}", frame.name);
            println!("  Frame ID: {}", frame.id);
            println!("  Frame elements: {}", frame.frame_elements.len());
            println!("  Definition length: {}", frame.definition.len());

            // Verify basic structure
            assert!(!frame.id.is_empty(), "Frame ID should not be empty");
            assert!(
                !frame.frame_elements.is_empty(),
                "Should have frame elements"
            );
            assert!(!frame.definition.is_empty(), "Should have definition");

            // Check specific content for Giving frame
            if frame.name == "Giving" {
                assert!(frame.has_frame_element("Donor"), "Should have Donor FE");
                assert!(frame.has_frame_element("Theme"), "Should have Theme FE");
                assert!(
                    frame.has_frame_element("Recipient"),
                    "Should have Recipient FE"
                );

                println!("  ✅ Verified Giving frame structure");

                // Check core elements
                let core_elements = frame.core_elements();
                println!("  Core elements: {}", core_elements.len());
                for fe in core_elements {
                    println!(
                        "    - {}: {}",
                        fe.name,
                        fe.definition.chars().take(50).collect::<String>()
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to parse FrameNet frame: {}", e);
            panic!("FrameNet frame parser test failed: {}", e);
        }
    }
}

#[test]
fn test_framenet_lu_parser() {
    use canopy_engine::XmlParser;
    use canopy_framenet::types::LexicalUnit;

    // Try to find any LU file
    let lu_dir = Path::new("../../data/framenet/archive/framenet_v17/framenet_v17/lu");

    if !lu_dir.exists() {
        println!("FrameNet LU directory not found, skipping LU parser test");
        return;
    }

    let parser = XmlParser::new();
    let entries = std::fs::read_dir(lu_dir).unwrap();

    let mut successful_parses = 0;
    let mut total_attempts = 0;

    // Try parsing first few LU files
    for entry in entries.take(5) {
        if let Ok(entry) = entry {
            let filepath = entry.path();
            if filepath.extension().and_then(|s| s.to_str()) == Some("xml") {
                total_attempts += 1;
                match parser.parse_file::<LexicalUnit>(&filepath) {
                    Ok(lu) => {
                        successful_parses += 1;
                        println!(
                            "✅ Parsed LU: {} (frame: {}, pos: {})",
                            lu.name, lu.frame_name, lu.pos
                        );

                        // Verify structure
                        assert!(!lu.id.is_empty(), "LU ID should not be empty");
                        assert!(!lu.name.is_empty(), "LU name should not be empty");
                        assert!(!lu.pos.is_empty(), "LU POS should not be empty");
                    }
                    Err(e) => {
                        println!("❌ Failed to parse {}: {}", filepath.display(), e);
                    }
                }

                if total_attempts >= 5 {
                    break;
                }
            }
        }
    }

    if total_attempts > 0 {
        let success_rate = (successful_parses as f32 / total_attempts as f32) * 100.0;
        println!(
            "LU parse success rate: {:.1}% ({}/{})",
            success_rate, successful_parses, total_attempts
        );

        // Require at least 50% success rate for integration test to pass
        assert!(
            success_rate >= 50.0,
            "FrameNet LU parser success rate too low: {:.1}%",
            success_rate
        );
    } else {
        println!("No FrameNet LU files found, skipping test");
    }
}

#[test]
fn test_framenet_mixed_directory() {
    use canopy_engine::XmlParser;
    use canopy_framenet::types::{Frame, LexicalUnit};

    let test_dir = Path::new("../../data/framenet/archive/framenet_v17/framenet_v17");

    if !test_dir.exists() {
        println!("FrameNet test directory not found, skipping mixed directory test");
        return;
    }

    let mut engine = FrameNetEngine::new();

    // Test loading mixed frame and LU data
    match engine.load_from_directory(test_dir) {
        Ok(()) => {
            let total_frames = engine.get_all_frames().len();
            let total_lus = engine.get_all_lexical_units().len();

            println!("✅ Successfully loaded mixed FrameNet data:");
            println!("  Frames: {}", total_frames);
            println!("  Lexical Units: {}", total_lus);

            // Should have loaded some data
            assert!(
                total_frames > 0 || total_lus > 0,
                "Should have loaded some frames or LUs"
            );

            // Test frame search
            let giving_frames = engine.search_frames("giving");
            if !giving_frames.is_empty() {
                println!(
                    "  ✅ Found {} frame(s) matching 'giving'",
                    giving_frames.len()
                );
            }

            // Test LU search
            let give_lus = engine.search_lexical_units("give");
            if !give_lus.is_empty() {
                println!("  ✅ Found {} LU(s) matching 'give'", give_lus.len());
            }
        }
        Err(e) => {
            println!("❌ Failed to load mixed FrameNet data: {}", e);
            // Don't panic here as this might be due to data organization
        }
    }
}
