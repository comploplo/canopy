//! Performance verification test to ensure <500μs analysis time
//! This test verifies that the complete Layer 2 analysis meets our performance requirement

use canopy_core::{DepRel, MorphFeatures, UDVerbForm, UPos, Word};
use canopy_semantics::{Layer2Analyzer, Layer2Config, PerformanceMode};
use std::time::{Duration, Instant};

fn create_test_word(
    id: usize,
    text: &str,
    lemma: &str,
    upos: UPos,
    head: usize,
    deprel: DepRel,
) -> Word {
    let mut word = Word {
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
    };

    // Add realistic morphological features for verbs
    if upos == UPos::Verb || upos == UPos::Aux {
        match lemma {
            "was" | "were" | "is" | "am" | "are" => {
                word.feats.verbform = Some(UDVerbForm::Finite);
            }
            "be" | "seem" | "appear" => {
                word.feats.verbform = Some(UDVerbForm::Infinitive);
            }
            _ => {}
        }
    }

    word
}

fn create_complex_test_sentence() -> Vec<Word> {
    // "John seems to believe Mary gave the book to Peter."
    // This sentence includes:
    // - Raising construction (seems)
    // - Control verb (believe)
    // - Ditransitive (gave)
    // - Multiple theta role assignments
    vec![
        create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
        create_test_word(2, "seems", "seem", UPos::Verb, 0, DepRel::Root),
        create_test_word(3, "to", "to", UPos::Part, 4, DepRel::Mark),
        create_test_word(4, "believe", "believe", UPos::Verb, 2, DepRel::Xcomp),
        create_test_word(5, "Mary", "Mary", UPos::Propn, 6, DepRel::Nsubj),
        create_test_word(6, "gave", "give", UPos::Verb, 4, DepRel::Ccomp),
        create_test_word(7, "the", "the", UPos::Det, 8, DepRel::Det),
        create_test_word(8, "book", "book", UPos::Noun, 6, DepRel::Obj),
        create_test_word(9, "to", "to", UPos::Adp, 10, DepRel::Case),
        create_test_word(10, "Peter", "Peter", UPos::Propn, 6, DepRel::Iobj),
    ]
}

#[test]
fn test_analysis_performance_under_500_microseconds() {
    // Test with different performance modes
    let test_configs = [
        ("Speed Mode", PerformanceMode::Speed),
        ("Balanced Mode", PerformanceMode::Balanced),
        ("Accuracy Mode", PerformanceMode::Accuracy),
    ];

    let test_sentence = create_complex_test_sentence();
    let warmup_iterations = 5;
    let benchmark_iterations = 100;

    for (mode_name, perf_mode) in test_configs {
        println!("\n=== Testing {mode_name} ===");

        let config = Layer2Config {
            performance_mode: perf_mode,
            ..Default::default()
        };

        // Warmup phase
        for _ in 0..warmup_iterations {
            let mut analyzer = Layer2Analyzer::with_config(config.clone());
            let _ = analyzer
                .analyze(test_sentence.clone())
                .expect("Analysis should succeed");
        }

        // Benchmark phase
        let mut total_time = Duration::new(0, 0);
        let mut successful_analyses = 0;

        for iteration in 0..benchmark_iterations {
            let mut analyzer = Layer2Analyzer::with_config(config.clone());

            let start_time = Instant::now();
            let result = analyzer.analyze(test_sentence.clone());
            let analysis_time = start_time.elapsed();

            match result {
                Ok(analysis) => {
                    successful_analyses += 1;
                    total_time += analysis_time;

                    // Verify we got a meaningful analysis
                    assert!(!analysis.words.is_empty(), "Analysis should contain words");
                    assert!(
                        !analysis.events.is_empty(),
                        "Analysis should contain events"
                    );

                    // Print detailed timing every 10th iteration
                    if iteration % 10 == 0 {
                        println!("Iteration {}: {}μs", iteration, analysis_time.as_micros());

                        // Print component breakdown from metrics
                        println!(
                            "  - Theta assignment: {}μs",
                            analysis.metrics.theta_assignment_time_us
                        );
                        println!(
                            "  - Event creation: {}μs",
                            analysis.metrics.event_creation_time_us
                        );
                        println!("  - Events created: {}", analysis.metrics.events_created);
                    }
                }
                Err(e) => {
                    panic!("Analysis failed at iteration {iteration}: {e:?}");
                }
            }
        }

        // Calculate statistics
        let avg_time = total_time / successful_analyses as u32;
        let avg_time_micros = avg_time.as_micros();

        println!("\n{mode_name} Results:");
        println!("  - Successful analyses: {successful_analyses}/{benchmark_iterations}");
        println!("  - Average time: {avg_time_micros}μs");
        let total_ms = total_time.as_millis();
        println!("  - Total time: {total_ms}ms");

        // CRITICAL PERFORMANCE REQUIREMENT: Must be under 500μs
        assert!(
            avg_time_micros < 500,
            "{mode_name} failed performance requirement: {avg_time_micros}μs >= 500μs"
        );

        // Also check that we're not extremely slow
        assert!(
            avg_time_micros < 2000,
            "{mode_name} is suspiciously slow: {avg_time_micros}μs (may indicate a performance regression)"
        );

        println!("  ✅ {mode_name} PASSED: {avg_time_micros}μs < 500μs");
    }
}

#[test]
// Enabled for M4 Phase 1 - performance comparison test with reasonable assertions
fn test_raising_vs_control_distinction_performance() {
    // Test that our raising detection doesn't significantly impact performance
    let raising_sentence = vec![
        // "John seems to be happy" (raising construction)
        create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
        create_test_word(2, "seems", "seem", UPos::Verb, 0, DepRel::Root),
        create_test_word(3, "to", "to", UPos::Part, 5, DepRel::Mark),
        create_test_word(4, "be", "be", UPos::Aux, 5, DepRel::Cop),
        create_test_word(5, "happy", "happy", UPos::Adj, 2, DepRel::Xcomp),
    ];

    let control_sentence = vec![
        // "John tries to be happy" (control construction)
        create_test_word(1, "John", "John", UPos::Propn, 2, DepRel::Nsubj),
        create_test_word(2, "tries", "try", UPos::Verb, 0, DepRel::Root),
        create_test_word(3, "to", "to", UPos::Part, 5, DepRel::Mark),
        create_test_word(4, "be", "be", UPos::Aux, 5, DepRel::Cop),
        create_test_word(5, "happy", "happy", UPos::Adj, 2, DepRel::Xcomp),
    ];

    let iterations = 50;

    // Test raising sentence
    let mut raising_total = Duration::new(0, 0);
    for _ in 0..iterations {
        let mut analyzer = Layer2Analyzer::new();
        let start = Instant::now();
        let result = analyzer
            .analyze(raising_sentence.clone())
            .expect("Raising analysis should succeed");
        raising_total += start.elapsed();

        // Verify the analysis succeeded (detailed movement checking would need access to movement analysis)
        assert!(
            !result.events.is_empty(),
            "Should create events for raising construction"
        );
    }

    // Test control sentence
    let mut control_total = Duration::new(0, 0);
    for _ in 0..iterations {
        let mut analyzer = Layer2Analyzer::new();
        let start = Instant::now();
        let result = analyzer
            .analyze(control_sentence.clone())
            .expect("Control analysis should succeed");
        control_total += start.elapsed();

        // Verify the analysis succeeded
        assert!(
            !result.events.is_empty(),
            "Should create events for control construction"
        );
    }

    let raising_avg = raising_total / iterations as u32;
    let control_avg = control_total / iterations as u32;

    println!("Raising vs Control Performance:");
    println!("  - Raising construction: {}μs", raising_avg.as_micros());
    println!("  - Control construction: {}μs", control_avg.as_micros());

    // Both should be well under 500μs
    assert!(
        raising_avg.as_micros() < 500,
        "Raising analysis too slow: {}μs",
        raising_avg.as_micros()
    );
    assert!(
        control_avg.as_micros() < 500,
        "Control analysis too slow: {}μs",
        control_avg.as_micros()
    );

    // The difference shouldn't be dramatic (within 2x)
    let ratio = std::cmp::max(raising_avg, control_avg).as_micros() as f64
        / std::cmp::min(raising_avg, control_avg).as_micros() as f64;
    assert!(ratio < 2.0, "Performance difference too large: {ratio}x");
}
