//! Layer 1 + Real UDPipe Integration Latency Tests
//!
//! This module tests the performance of the real UDPipe integration in Layer 1.

#![allow(clippy::useless_vec)] // Allow vec usage in tests for clarity

#[cfg(test)]
mod tests {
    use crate::layer1::{Layer1Config, Layer1Parser};
    use crate::udpipe::UDPipeEngine;
    use std::time::Instant;

    #[test]
    fn test_layer1_basic_model_latency() {
        println!("\n🚀 Layer 1 + UDPipe Model Latency Test");

        // Create test engine (uses real model if available, enhanced tokenization fallback)
        let engine = UDPipeEngine::for_testing();
        let parser = Layer1Parser::new(engine);

        let test_sentences = vec![
            "The cat sat.",
            "She gave him a book.",
            "The quick brown fox jumps over the lazy dog.",
            "John loves Mary and she loves him too.",
            "In the beginning was the Word, and the Word was with God.",
        ];

        for (i, sentence) in test_sentences.iter().enumerate() {
            println!("\nTest {}: \"{}\"", i + 1, sentence);

            // Warm-up run
            let _ = parser
                .parse_document(sentence)
                .expect("Parse should succeed");

            // Timed runs
            let mut times = Vec::new();
            for _ in 0..5 {
                let start = Instant::now();
                let result = parser
                    .parse_document(sentence)
                    .expect("Parse should succeed");
                let duration = start.elapsed();
                times.push(duration.as_nanos() as f64);

                if i == 0 {
                    // Show first result in detail
                    println!("  Words parsed: {}", result.len());
                    for word in result.iter().take(3) {
                        println!(
                            "    {}: {} [{:?}] -> {}",
                            word.word.id, word.word.text, word.word.upos, word.word.lemma
                        );
                    }
                    if result.len() > 3 {
                        println!("    ... and {} more", result.len() - 3);
                    }
                }
            }

            let avg_ns = times.iter().sum::<f64>() / times.len() as f64;
            let min_ns = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_ns = times.iter().fold(0.0_f64, |a, &b| a.max(b));

            println!("  Performance:");
            println!("    Average: {:.1}μs", avg_ns / 1000.0);
            println!("    Min:     {:.1}μs", min_ns / 1000.0);
            println!("    Max:     {:.1}μs", max_ns / 1000.0);
            println!("    Characters: {}", sentence.len());
            println!(
                "    μs/char: {:.3}",
                (avg_ns / 1000.0) / sentence.len() as f64
            );

            // Validate performance target
            let target_us = 500.0;
            let actual_us = avg_ns / 1000.0;
            if actual_us < target_us {
                println!("    ✅ Under 500μs target ({actual_us:.1}μs)");
            } else {
                println!("    ⚠️  Over 500μs target ({actual_us:.1}μs)");
            }
        }
    }

    #[test]
    fn test_layer1_real_model_latency() {
        println!("\n🚀 Layer 1 + Real UDPipe Model Latency Test");

        // Try to load real model
        let model_path = "/Users/gabe/projects/canopy/models/test.model";

        if !std::path::Path::new(model_path).exists() {
            println!("⚠️  Real model not found at {model_path}");
            println!("   Skipping real UDPipe latency test");
            return;
        }

        println!("Loading real UDPipe model from: {model_path}");
        let engine = UDPipeEngine::load(model_path).expect("Model should load");

        // Create parser with debug configuration
        let config = Layer1Config {
            enable_features: true,
            max_sentence_length: 200,
            debug: false, // Disable debug for cleaner output
        };
        let parser = Layer1Parser::with_config(engine, config);

        let test_sentences = vec![
            "The cat sat on the mat.",
            "She gave him a beautiful book yesterday.",
            "The quick brown fox jumps over the lazy dog.",
            "Complex sentence with subordinate clauses that test parsing accuracy.",
        ];

        for (i, sentence) in test_sentences.iter().enumerate() {
            println!("\nReal Model Test {}: \"{}\"", i + 1, sentence);

            // Multiple timed runs to get average
            let mut times = Vec::new();
            for _ in 0..3 {
                let start = Instant::now();
                let result = parser
                    .parse_document(sentence)
                    .expect("Parse should succeed");
                let duration = start.elapsed();
                times.push((duration, result));
            }

            let avg_duration =
                times.iter().map(|(d, _)| d.as_nanos() as f64).sum::<f64>() / times.len() as f64;
            let result = &times[0].1; // Use first result for analysis

            println!("  Parse time: {:.1}μs", avg_duration / 1000.0);
            println!("  Words parsed: {}", result.len());

            // Show enhanced features
            let enhanced_words: Vec<_> = result
                .iter()
                .filter(|w| w.animacy.is_some() || w.concreteness.is_some())
                .collect();

            if !enhanced_words.is_empty() {
                println!("  Enhanced features:");
                for word in enhanced_words {
                    println!(
                        "    {}: {:?} animacy, {:?} concreteness",
                        word.word.text, word.animacy, word.concreteness
                    );
                }
            }

            // Check for real morphological features from UDPipe
            let features_count = result
                .iter()
                .filter(|w| {
                    w.word.feats.person.is_some()
                        || w.word.feats.number.is_some()
                        || w.word.feats.gender.is_some()
                        || w.word.feats.raw_features.is_some()
                })
                .count();
            println!("  Words with morphological features: {features_count}");

            // Performance metrics
            let chars = sentence.len();
            let words = result.len();
            println!("  Performance:");
            println!("    μs/char: {:.3}", (avg_duration / 1000.0) / chars as f64);
            println!("    μs/word: {:.1}", (avg_duration / 1000.0) / words as f64);

            // Target validation (keep under 500μs per sentence)
            let target_us = 500.0;
            let actual_us = avg_duration / 1000.0;
            if actual_us < target_us {
                println!("    ✅ Under 500μs target ({actual_us:.1}μs)");
            } else {
                println!("    ⚠️  Over 500μs target ({actual_us:.1}μs)");
            }

            // Show sample parsing output
            if i == 0 {
                println!("  Sample parsing output:");
                for word in result.iter().take(5) {
                    println!(
                        "    {}: {} [{:?}] -> {} (head: {:?})",
                        word.word.id,
                        word.word.text,
                        word.word.upos,
                        word.word.lemma,
                        word.word.head
                    );
                }
            }
        }
    }

    #[test]
    fn test_layer1_linguistic_accuracy() {
        println!("\n🧠 Layer 1 Linguistic Analysis Accuracy Test");

        let engine = UDPipeEngine::for_testing();
        let parser = Layer1Parser::new(engine);

        let test_cases = vec![
            ("The dog barked loudly.", "dog should be animate"),
            (
                "The computer crashed yesterday.",
                "computer should be inanimate",
            ),
            (
                "John gave Mary a beautiful book.",
                "John and Mary should be animate",
            ),
        ];

        for (sentence, expectation) in test_cases {
            println!("\nAnalyzing: \"{sentence}\"");
            println!("Expectation: {expectation}");

            let result = parser
                .parse_document(sentence)
                .expect("Parse should succeed");

            println!("Results:");
            for word in result {
                if word.animacy.is_some() || word.concreteness.is_some() {
                    println!(
                        "  {}: animacy={:?}, concreteness={:?}",
                        word.word.text, word.animacy, word.concreteness
                    );
                }
            }
        }
    }

    #[test]
    fn test_layer1_performance_scaling() {
        println!("\n📈 Layer 1 Performance Scaling Test");

        let engine = UDPipeEngine::for_testing();
        let parser = Layer1Parser::new(engine);

        // Test different sentence lengths
        let test_cases = vec![
            ("Short", "Cat."),
            ("Medium", "The quick brown fox jumps over the lazy dog."),
            ("Long", "In a hole in the ground there lived a hobbit, not a nasty dirty wet hole filled with the ends of worms and an oozy smell."),
            ("Very Long", "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity, it was the season of Light, it was the season of Darkness."),
        ];

        for (name, sentence) in test_cases {
            let char_count = sentence.len();

            // Time multiple runs
            let mut times = Vec::new();
            for _ in 0..5 {
                let start = Instant::now();
                let result = parser
                    .parse_document(sentence)
                    .expect("Parse should succeed");
                let duration = start.elapsed();
                times.push((duration.as_nanos() as f64, result.len()));
            }

            let avg_ns = times.iter().map(|(ns, _)| *ns).sum::<f64>() / times.len() as f64;
            let word_count = times[0].1;

            println!("\n{} sentence:", name);
            println!("  Characters: {}", char_count);
            println!("  Words: {}", word_count);
            println!("  Time: {:.1}μs", avg_ns / 1000.0);
            println!("  μs/char: {:.3}", (avg_ns / 1000.0) / char_count as f64);
            println!("  μs/word: {:.1}", (avg_ns / 1000.0) / word_count as f64);

            // Check if performance degrades with length
            let efficiency = (avg_ns / 1000.0) / char_count as f64;
            if efficiency < 1.0 {
                println!("  ✅ Excellent efficiency ({:.3} μs/char)", efficiency);
            } else if efficiency < 5.0 {
                println!("  ✅ Good efficiency ({:.3} μs/char)", efficiency);
            } else {
                println!("  ⚠️  Consider optimization ({:.3} μs/char)", efficiency);
            }
        }
    }
}
