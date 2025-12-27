use canopy_verbnet::VerbNetEngine;

fn main() {
    println!("🔍 Debugging VerbNet data loading...");

    // Try to create VerbNet engine with real data
    match VerbNetEngine::new() {
        Ok(engine) => {
            println!("✅ VerbNet engine created successfully!");
            let _stats = engine.statistics();
            println!("   Engine initialized and ready");

            // Test analysis
            match engine.analyze_verb("give") {
                Ok(result) => {
                    println!("✅ Analysis successful for 'give':");
                    println!("   Confidence: {}", result.confidence);
                    println!("   Classes found: {}", result.data.verb_classes.len());
                }
                Err(e) => println!("❌ Analysis failed: {}", e),
            }
        }
        Err(e) => {
            println!("❌ VerbNet engine creation failed: {}", e);

            // Check if data directory exists
            let data_path = "data/verbnet/vn-gl";
            if std::path::Path::new(data_path).exists() {
                println!("📁 Data directory exists: {}", data_path);

                // Count XML files
                match std::fs::read_dir(data_path) {
                    Ok(entries) => {
                        let xml_count = entries
                            .filter_map(Result::ok)
                            .filter(|entry| {
                                entry
                                    .path()
                                    .extension()
                                    .and_then(|ext| ext.to_str())
                                    .map(|ext| ext == "xml")
                                    .unwrap_or(false)
                            })
                            .count();
                        println!("📄 Found {} XML files in directory", xml_count);

                        // Try to read a sample file
                        if xml_count > 0 {
                            if let Ok(entries) = std::fs::read_dir(data_path) {
                                for entry in entries.filter_map(Result::ok) {
                                    if entry.path().extension().and_then(|ext| ext.to_str())
                                        == Some("xml")
                                    {
                                        println!("📋 Sample file: {}", entry.path().display());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => println!("❌ Could not read directory: {}", e),
                }
            } else {
                println!("📁 Data directory does not exist: {}", data_path);
            }
        }
    }
}
