//! Test VerbNet integration to ensure it's working correctly

use canopy_engine::StatisticsProvider;
use canopy_verbnet::VerbNetEngine;

/// Test that VerbNet is working and returns meaningful results
pub fn test_verbnet_integration() {
    println!("Testing VerbNet integration...");

    // Create VerbNet engine with test data
    let mut verbnet_engine = VerbNetEngine::new();

    // Test with common verbs
    let test_verbs = ["run", "walk", "eat", "see", "give", "put"];

    for verb in &test_verbs {
        println!("\n--- Testing verb: '{verb}' ---");

        // Get verb analysis
        match verbnet_engine.analyze_verb(verb) {
            Ok(analysis) => {
                println!("  VerbNet classes: {}", analysis.data.verb_classes.len());

                for (i, class) in analysis.data.verb_classes.iter().enumerate() {
                    println!("    Class {}: {} ({})", i + 1, class.id, class.class_name);
                }

                // Get theta roles from analysis
                println!(
                    "  Theta role assignments: {}",
                    analysis.data.theta_role_assignments.len()
                );
                for assignment in &analysis.data.theta_role_assignments {
                    println!("    {assignment:?}");
                }

                // Get semantic predicates from analysis
                println!(
                    "  Semantic predicates: {}",
                    analysis.data.semantic_predicates.len()
                );
                for predicate in &analysis.data.semantic_predicates {
                    println!("    {predicate:?}");
                }

                println!("  Confidence: {:.2}", analysis.confidence);
                println!("  From cache: {}", analysis.from_cache);
                println!("  Processing time: {}μs", analysis.processing_time_us);
            }
            Err(e) => {
                println!("  Error analyzing verb '{verb}': {e}");
            }
        }
    }

    // Print overall statistics
    let stats = verbnet_engine.statistics();
    println!("\n--- VerbNet Statistics ---");
    println!("Total entries: {}", stats.data.total_entries);
    println!("Unique keys: {}", stats.data.unique_keys);
    println!("Total queries: {}", stats.performance.total_queries);
    println!("Cache hits: {}", stats.cache.hits);
    println!("Cache misses: {}", stats.cache.misses);
    println!(
        "Average query time: {}μs",
        stats.performance.avg_query_time_us
    );
}
