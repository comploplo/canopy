//! UDPipe quality assessment test

use super::UDPipeEngine;
use canopy_core::UPos;

#[test]
fn test_udpipe_output_quality() {
    println!("=== UDPipe Output Quality Assessment ===\n");

    let engine = UDPipeEngine::for_testing();

    // Test cases covering various linguistic phenomena
    let test_cases = vec![
        // Basic sentences
        ("Simple present", "The cat sits on the mat."),
        ("Simple past", "John walked to the store."),
        ("Question", "Where did Mary go?"),
        // Verb forms and tense
        ("Progressive", "She is reading a book."),
        ("Perfect", "They have finished the work."),
        ("Modal", "You should study harder."),
        ("Passive", "The book was written by Shakespeare."),
        // Complex syntax
        ("Relative clause", "The man who called yesterday arrived."),
        ("Coordination", "John and Mary went to the park."),
        ("Subordination", "Because it rained, we stayed inside."),
        // Different word types
        ("Proper nouns", "Alice visited Paris in December."),
        ("Numbers", "I bought 3 apples for $2.50."),
        ("Contractions", "I can't believe it's working!"),
    ];

    let mut total_words = 0;
    let mut correct_verbs = 0;
    let mut total_verbs_expected = 0;
    let mut correct_nouns = 0;
    let mut total_nouns_expected = 0;
    let mut correct_dets = 0;
    let mut total_dets_expected = 0;

    for (description, sentence) in test_cases {
        println!("📝 {}: \"{}\"", description, sentence);

        match engine.parse(sentence) {
            Ok(result) => {
                println!("   ✅ Parsed {} words:", result.words.len());

                for word in &result.words {
                    println!("      {} → {} ({:?})", word.form, word.lemma, word.upos);
                    total_words += 1;

                    // Check specific word expectations
                    match word.form.to_lowercase().as_str() {
                        // Expected verbs
                        "sits" | "sit" | "walked" | "walk" | "go" | "went" | "is" | "reading"
                        | "read" | "have" | "finished" | "finish" | "should" | "study" | "was"
                        | "written" | "write" | "called" | "call" | "arrived" | "arrive"
                        | "stayed" | "stay" | "visited" | "visit" | "bought" | "buy"
                        | "believe" | "working" | "work" => {
                            total_verbs_expected += 1;
                            if matches!(word.upos, UPos::Verb | UPos::Aux) {
                                correct_verbs += 1;
                            } else {
                                println!("      ❌ Expected VERB but got {:?}", word.upos);
                            }
                        }

                        // Expected nouns
                        "cat" | "mat" | "john" | "store" | "mary" | "book" | "man"
                        | "yesterday" | "alice" | "paris" | "december" | "apples" => {
                            total_nouns_expected += 1;
                            if matches!(word.upos, UPos::Noun | UPos::Propn) {
                                correct_nouns += 1;
                            } else {
                                println!("      ❌ Expected NOUN but got {:?}", word.upos);
                            }
                        }

                        // Expected determiners
                        "the" | "a" | "an" => {
                            total_dets_expected += 1;
                            if matches!(word.upos, UPos::Det) {
                                correct_dets += 1;
                            } else {
                                println!("      ❌ Expected DET but got {:?}", word.upos);
                            }
                        }

                        _ => {}
                    }
                }

                // Check for reasonable distribution
                let verb_count = result
                    .words
                    .iter()
                    .filter(|w| matches!(w.upos, UPos::Verb | UPos::Aux))
                    .count();
                let noun_count = result
                    .words
                    .iter()
                    .filter(|w| matches!(w.upos, UPos::Noun | UPos::Propn))
                    .count();
                let punct_count = result
                    .words
                    .iter()
                    .filter(|w| matches!(w.upos, UPos::Punct))
                    .count();

                println!(
                    "   📊 Distribution: {} verbs, {} nouns, {} punct",
                    verb_count, noun_count, punct_count
                );

                // Flag if everything is punctuation (indicates parsing failure)
                if punct_count >= result.words.len() * 3 / 4 {
                    println!("   🚨 WARNING: Most words tagged as punctuation!");
                }

                // Flag if no verbs in a sentence that should have them
                if verb_count == 0 && sentence.len() > 10 {
                    println!("   ⚠️  WARNING: No verbs found in complete sentence");
                }
            }
            Err(e) => {
                println!("   ❌ Parse failed: {:?}", e);
            }
        }

        println!();
    }

    // Overall statistics
    println!("=== OVERALL QUALITY ASSESSMENT ===");
    println!("Total words processed: {}", total_words);

    if total_verbs_expected > 0 {
        let verb_accuracy = (correct_verbs as f64 / total_verbs_expected as f64) * 100.0;
        println!(
            "Verb accuracy: {}/{} ({:.1}%)",
            correct_verbs, total_verbs_expected, verb_accuracy
        );
    }

    if total_nouns_expected > 0 {
        let noun_accuracy = (correct_nouns as f64 / total_nouns_expected as f64) * 100.0;
        println!(
            "Noun accuracy: {}/{} ({:.1}%)",
            correct_nouns, total_nouns_expected, noun_accuracy
        );
    }

    if total_dets_expected > 0 {
        let det_accuracy = (correct_dets as f64 / total_dets_expected as f64) * 100.0;
        println!(
            "Determiner accuracy: {}/{} ({:.1}%)",
            correct_dets, total_dets_expected, det_accuracy
        );
    }

    let overall_accuracy = ((correct_verbs + correct_nouns + correct_dets) as f64
        / (total_verbs_expected + total_nouns_expected + total_dets_expected) as f64)
        * 100.0;
    println!("Overall POS accuracy: {:.1}%", overall_accuracy);

    // Quality assessment
    println!("\n=== QUALITY VERDICT ===");
    if overall_accuracy >= 90.0 {
        println!("🏆 EXCELLENT: UDPipe output quality is excellent");
    } else if overall_accuracy >= 75.0 {
        println!("✅ GOOD: UDPipe output quality is good for most applications");
    } else if overall_accuracy >= 60.0 {
        println!("⚠️  FAIR: UDPipe output quality is fair, some issues present");
    } else {
        println!("❌ POOR: UDPipe output quality needs improvement");
    }

    // Specific recommendations
    if total_verbs_expected > 0 && (correct_verbs as f64 / total_verbs_expected as f64) < 0.8 {
        println!("📝 Recommendation: Verb detection needs improvement");
    }
    if total_nouns_expected > 0 && (correct_nouns as f64 / total_nouns_expected as f64) < 0.8 {
        println!("📝 Recommendation: Noun detection needs improvement");
    }

    // Assert minimum quality standards
    assert!(
        overall_accuracy >= 50.0,
        "UDPipe quality is too low: {:.1}%",
        overall_accuracy
    );
    assert!(total_words > 50, "Not enough test data processed");
}
