//! Test VerbNet integration to ensure it's working correctly

use canopy_semantics::verbnet::VerbNetEngine;

/// Test that VerbNet is working and returns meaningful results
pub fn test_verbnet_integration() {
    println!("Testing VerbNet integration...");

    // Create VerbNet engine with test data
    let mut verbnet_engine = VerbNetEngine::new();
    verbnet_engine.add_test_data();

    // Test with common verbs
    let test_verbs = ["run", "walk", "eat", "see", "give", "put"];

    for verb in &test_verbs {
        println!("\n--- Testing verb: '{verb}' ---");

        // Get verb classes
        let verb_classes = verbnet_engine.get_verb_classes(verb);
        println!("  VerbNet classes: {}", verb_classes.len());

        for (i, class) in verb_classes.iter().enumerate() {
            println!("    Class {}: {} ({})", i + 1, class.id, class.name);
        }

        // Get theta roles
        let theta_roles = verbnet_engine.get_theta_roles(verb);
        println!("  Theta roles: {}", theta_roles.len());

        for role in &theta_roles {
            println!("    {role:?}");
        }

        // Get semantic predicates
        let predicates = verbnet_engine.get_semantic_predicates(verb);
        println!("  Semantic predicates: {}", predicates.len());

        for predicate in &predicates {
            println!("    {predicate:?}");
        }

        // Get aspectual info
        let aspectual_info = verbnet_engine.infer_aspectual_class(verb);
        println!("  Aspectual class: {aspectual_info:?}");

        // Get syntactic frames
        let frames = verbnet_engine.get_syntactic_frames(verb);
        println!("  Syntactic frames: {}", frames.len());
    }

    // Print overall statistics
    let stats = verbnet_engine.get_statistics();
    println!("\n--- VerbNet Statistics ---");
    println!("Total classes: {}", stats.total_classes);
    println!("Total verbs: {}", stats.total_verbs);
    println!("Total theta roles: {}", stats.total_roles);
    println!("Total predicates: {}", stats.total_predicates);
    println!(
        "Total selectional restrictions: {}",
        stats.total_restrictions
    );
}
