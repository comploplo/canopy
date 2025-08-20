//! Debug test to check if VerbNet is actually working

use canopy_semantics::verbnet::VerbNetEngine;

#[test]
fn test_verbnet_engine_directly() {
    println!("\n=== TESTING VERBNET ENGINE DIRECTLY ===");

    let mut engine = VerbNetEngine::new_with_test_data();

    // Test 1: Simple VerbNet lookup for "give"
    println!("\n--- Test 1: Basic VerbNet lookup for 'give' ---");
    let result = engine.lookup_with_context(
        "give",
        "subj obj iobj", // Basic ditransitive pattern
        &[
            ("subj".to_string(), "John".to_string()),
            ("obj".to_string(), "book".to_string()),
            ("iobj".to_string(), "Mary".to_string()),
        ],
        "active",
        "present",
    );

    match result {
        Ok(lookup_result) => {
            println!("✅ VerbNet lookup succeeded!");
            println!(
                "Theta assignments: {} options",
                lookup_result.theta_assignments.len()
            );
            println!(
                "Semantic predicates: {} predicates",
                lookup_result.semantic_predicates.len()
            );
            println!(
                "Selectional restrictions: {} restrictions",
                lookup_result.selectional_restrictions.len()
            );

            // Print details
            for (i, (theta_roles, confidence)) in lookup_result.theta_assignments.iter().enumerate()
            {
                println!(
                    "  Assignment {}: {} roles, confidence: {:.3}",
                    i,
                    theta_roles.len(),
                    confidence
                );
                for role in theta_roles {
                    println!(
                        "    {:?} (restrictions: {})",
                        role.role_type,
                        role.selectional_restrictions.len()
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ VerbNet lookup failed: {e:?}");
        }
    }

    // Test 2: Check if VerbNet XML data is loaded
    println!("\n--- Test 2: Check VerbNet data loading ---");
    let verb_classes = engine.get_verb_classes("give");
    println!("✅ Found {} verb classes for 'give'", verb_classes.len());
    for class in &verb_classes {
        println!("  Class: {}", class.id);
    }

    // Test 3: Test with other common verbs
    println!("\n--- Test 3: Testing other verbs ---");
    let test_verbs = ["run", "break", "seem", "believe"];
    for verb in test_verbs {
        let classes = engine.get_verb_classes(verb);
        println!("✅ '{}': {} classes", verb, classes.len());
        for class in &classes {
            println!("  {}", class.id);
        }
    }
}
