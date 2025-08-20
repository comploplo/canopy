//! Diagnostic test to understand what's actually happening in the performance test

use canopy_core::{DepRel, MorphFeatures, UPos, Word};
use canopy_semantics::Layer2Analyzer;
use std::time::{Duration, Instant};

fn create_test_word(
    id: usize,
    text: &str,
    lemma: &str,
    upos: UPos,
    head: usize,
    deprel: DepRel,
) -> Word {
    Word {
        id,
        text: text.to_string(),
        lemma: lemma.to_string(),
        upos,
        xpos: None,
        feats: MorphFeatures::default(),
        head: Some(head),
        deprel,
        deps: None,
        misc: None,
        start: 0,
        end: text.len(),
    }
}

#[test]
fn diagnostic_what_is_actually_happening() {
    println!("\n=== DIAGNOSTIC: What is the analyzer actually doing? ===");

    // Test with a sentence that should definitely hit VerbNet: "John gave Mary a book"
    let test_sentence = vec![
        create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
        create_test_word(2, "gave", "give", UPos::Verb, 0, DepRel::Root),
        create_test_word(3, "Mary", "Mary", UPos::Propn, 2, DepRel::Iobj),
        create_test_word(4, "a", "a", UPos::Det, 5, DepRel::Det),
        create_test_word(5, "book", "book", UPos::Noun, 2, DepRel::Obj),
    ];

    println!(
        "Input sentence: {:?}",
        test_sentence.iter().map(|w| &w.text).collect::<Vec<_>>()
    );

    // Test analysis with detailed output
    let mut analyzer = Layer2Analyzer::new();

    println!("\n--- Timing individual operations ---");
    let overall_start = Instant::now();

    let result = analyzer.analyze(test_sentence.clone());
    let overall_time = overall_start.elapsed();

    match result {
        Ok(analysis) => {
            println!("✅ Analysis succeeded in {}μs", overall_time.as_micros());

            println!("\n--- Analysis Results ---");
            println!("Words processed: {}", analysis.words.len());
            println!("Events created: {}", analysis.events.len());
            println!("Theta assignments: {}", analysis.theta_assignments.len());
            println!("Overall confidence: {:.2}", analysis.confidence);

            println!("\n--- Events Detail ---");
            for (i, event) in analysis.events.iter().enumerate() {
                println!("Event {}: {:?}", i, event.predicate);
                println!("  Participants: {:?}", event.participants.len());
                for (role, participant) in &event.participants {
                    println!("    {:?}: {}", role, participant.expression);
                }
            }

            println!("\n--- Theta Assignments Detail ---");
            for (event_id, assignments) in &analysis.theta_assignments {
                println!("Event {:?}: {} assignments", event_id, assignments.len());
                for (role, participant) in assignments {
                    println!("  {:?} -> {}", role, participant.expression);
                }
            }

            println!("\n--- Metrics Breakdown ---");
            println!("Total time: {}μs", analysis.metrics.total_time_us);
            println!(
                "Theta assignment: {}μs",
                analysis.metrics.theta_assignment_time_us
            );
            println!(
                "Event creation: {}μs",
                analysis.metrics.event_creation_time_us
            );
            println!(
                "Little v time: {:?}μs",
                analysis.metrics.little_v_decomposition_time_us
            );
            println!(
                "Events with little v: {}",
                analysis.metrics.events_with_little_v
            );
        }
        Err(e) => {
            println!("❌ Analysis failed: {e:?}");
        }
    }
}

#[test]
fn diagnostic_verbnet_loading() {
    println!("\n=== DIAGNOSTIC: Is VerbNet data actually loaded? ===");

    // Try to create analyzer and see if VerbNet engine works
    let mut analyzer = Layer2Analyzer::new();

    // Test with a very simple sentence first
    let simple_sentence = vec![
        create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
        create_test_word(2, "runs", "run", UPos::Verb, 0, DepRel::Root),
    ];

    println!("Testing simple sentence: John runs");
    let start = Instant::now();
    let result = analyzer.analyze(simple_sentence);
    let time = start.elapsed();

    match result {
        Ok(analysis) => {
            println!("Simple analysis: {}μs", time.as_micros());
            println!(
                "Events: {}, Theta assignments: {}",
                analysis.events.len(),
                analysis.theta_assignments.len()
            );
        }
        Err(e) => println!("Simple analysis failed: {e:?}"),
    }

    // Test with complex VerbNet-heavy sentence
    let complex_sentence = vec![
        create_test_word(1, "Mary", "Mary", UPos::Propn, 2, DepRel::Nsubj),
        create_test_word(2, "believes", "believe", UPos::Verb, 0, DepRel::Root), // Should be in VerbNet
        create_test_word(3, "John", "John", UPos::Propn, 4, DepRel::Nsubj),
        create_test_word(4, "gave", "give", UPos::Verb, 2, DepRel::Ccomp), // Ditransitive - definitely in VerbNet
        create_test_word(5, "Susan", "Susan", UPos::Propn, 4, DepRel::Iobj),
        create_test_word(6, "the", "the", UPos::Det, 7, DepRel::Det),
        create_test_word(7, "book", "book", UPos::Noun, 4, DepRel::Obj),
    ];

    println!("\nTesting complex sentence: Mary believes John gave Susan the book");
    let start = Instant::now();
    let result = analyzer.analyze(complex_sentence);
    let time = start.elapsed();

    match result {
        Ok(analysis) => {
            println!("Complex analysis: {}μs", time.as_micros());
            println!(
                "Events: {}, Theta assignments: {}",
                analysis.events.len(),
                analysis.theta_assignments.len()
            );

            // Print detailed results
            for event in &analysis.events {
                println!("Event predicate: {:?}", event.predicate.lemma);
            }
        }
        Err(e) => println!("Complex analysis failed: {e:?}"),
    }
}

#[test]
fn diagnostic_baseline_overhead() {
    println!("\n=== DIAGNOSTIC: Baseline timing overhead ===");

    // Measure just the timing overhead
    let iterations = 100;
    let mut total_time = Duration::new(0, 0);

    for _ in 0..iterations {
        let start = Instant::now();
        // Do absolutely nothing
        let _end = start.elapsed();
        total_time += _end;
    }

    let avg_overhead = total_time / iterations as u32;
    println!("Average timing overhead: {}ns", avg_overhead.as_nanos());

    // Measure very simple operation
    let mut total_simple = Duration::new(0, 0);
    for _ in 0..iterations {
        let start = Instant::now();
        let _array = [1, 2, 3, 4, 5]; // Simple allocation
        total_simple += start.elapsed();
    }

    let avg_simple = total_simple / iterations as u32;
    println!("Average simple allocation: {}ns", avg_simple.as_nanos());

    println!(
        "If analysis is ~1μs (1000ns), overhead is {:.1}%",
        avg_overhead.as_nanos() as f64 / 1000.0 * 100.0
    );
}
