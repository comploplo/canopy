//! Integration tests for VerbNet engine against real data

use canopy_verbnet::{DataLoader, SemanticEngine, VerbNetEngine};
use std::path::Path;

#[test]
fn test_load_single_verbnet_file() {
    let mut engine = VerbNetEngine::new();

    // Test loading a single VerbNet XML file
    let test_file = Path::new("../../data/verbnet/vn-gl/give-13.1.xml");

    if !test_file.exists() {
        println!("VerbNet test data not found at: {}", test_file.display());
        println!("Skipping integration test");
        return;
    }

    // Try to load the file using the XML parser directly
    let result = engine.load_from_directory(test_file.parent().unwrap());

    match result {
        Ok(()) => {
            println!("✅ Successfully loaded VerbNet data");
            // Access VerbNet-specific stats directly
            println!(
                "Number of classes loaded: {}",
                engine.get_all_classes().len()
            );
            println!("Engine initialized: {}", engine.is_initialized());

            // Test that we can find some verbs
            if let Ok(analysis) = engine.analyze_verb("give") {
                println!("✅ Successfully analyzed 'give':");
                println!("  Classes found: {}", analysis.data.verb_classes.len());
                println!("  Confidence: {:.2}", analysis.confidence);

                for class in &analysis.data.verb_classes {
                    println!("  - Class: {} ({})", class.id, class.class_name);
                }
            } else {
                println!("❌ Failed to analyze 'give'");
            }

            // Test that the engine reports as loaded
            assert!(engine.is_loaded(), "Engine should report as loaded");
        }
        Err(e) => {
            println!("❌ Failed to load VerbNet data: {}", e);
            println!("This indicates issues with the XML parser implementation");
            panic!("VerbNet integration test failed: {}", e);
        }
    }
}

#[test]
fn test_verbnet_parser_with_real_data() {
    use canopy_engine::XmlParser;
    use canopy_verbnet::types::VerbClass;

    let test_file = Path::new("../../data/verbnet/vn-gl/give-13.1.xml");

    if !test_file.exists() {
        println!("VerbNet test data not found, skipping parser test");
        return;
    }

    let parser = XmlParser::new();
    let result = parser.parse_file::<VerbClass>(test_file);

    match result {
        Ok(verb_class) => {
            println!("✅ Successfully parsed VerbNet class: {}", verb_class.id);
            println!("  Class name: {}", verb_class.class_name);
            println!("  Members: {}", verb_class.members.len());
            println!("  Thematic roles: {}", verb_class.themroles.len());
            println!("  Frames: {}", verb_class.frames.len());

            // Verify basic structure
            assert!(!verb_class.id.is_empty(), "Class ID should not be empty");
            assert!(!verb_class.members.is_empty(), "Should have members");
            assert!(
                !verb_class.themroles.is_empty(),
                "Should have thematic roles"
            );

            // Check specific content for give-13.1
            if verb_class.id == "give-13.1" {
                assert!(
                    verb_class.members.iter().any(|m| m.name == "deal"),
                    "Should contain 'deal' member"
                );
                assert!(
                    verb_class.themroles.iter().any(|r| r.role_type == "Agent"),
                    "Should have Agent role"
                );
                assert!(
                    verb_class.themroles.iter().any(|r| r.role_type == "Theme"),
                    "Should have Theme role"
                );
                assert!(
                    verb_class
                        .themroles
                        .iter()
                        .any(|r| r.role_type == "Recipient"),
                    "Should have Recipient role"
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to parse VerbNet file: {}", e);
            panic!("VerbNet parser test failed: {}", e);
        }
    }
}

#[test]
fn test_multiple_verbnet_files() {
    use canopy_engine::XmlParser;
    use canopy_verbnet::types::VerbClass;

    let data_dir = Path::new("../../data/verbnet/vn-gl");

    if !data_dir.exists() {
        println!("VerbNet data directory not found, skipping multi-file test");
        return;
    }

    let parser = XmlParser::new();
    let test_files = ["give-13.1.xml", "run-51.3.2.xml", "put-9.1.xml"];

    let mut successful_parses = 0;
    let mut total_attempts = 0;

    for filename in &test_files {
        let filepath = data_dir.join(filename);
        if filepath.exists() {
            total_attempts += 1;
            match parser.parse_file::<VerbClass>(&filepath) {
                Ok(verb_class) => {
                    successful_parses += 1;
                    println!(
                        "✅ Parsed {}: {} (ID: {})",
                        filename, verb_class.class_name, verb_class.id
                    );
                }
                Err(e) => {
                    println!("❌ Failed to parse {}: {}", filename, e);
                }
            }
        } else {
            println!("⏭  Skipping {} (file not found)", filename);
        }
    }

    if total_attempts > 0 {
        let success_rate = (successful_parses as f32 / total_attempts as f32) * 100.0;
        println!(
            "Parse success rate: {:.1}% ({}/{})",
            success_rate, successful_parses, total_attempts
        );

        // Require at least 50% success rate for integration test to pass
        assert!(
            success_rate >= 50.0,
            "VerbNet parser success rate too low: {:.1}%",
            success_rate
        );
    } else {
        println!("No VerbNet test files found, skipping multi-file test");
    }
}
