use canopy_parser::udpipe::UDPipeEngine;
use canopy_parser::layer1::{Layer1Parser, Layer1Config};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Layer 1 + Real UDPipe Integration Latency Test");
    println!("=" .repeat(60));

    // Test both dummy and real model configurations
    test_with_dummy_model()?;
    test_with_real_model()?;

    Ok(())
}

fn test_with_dummy_model() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Testing with Dummy UDPipe Model (Enhanced Tokenization)");
    println!("-".repeat(50));

    // Create dummy engine
    let engine = UDPipeEngine::for_testing();
    let parser = Layer1Parser::new(engine);

    let test_sentences = [
        "The cat sat.",
        "She gave him a book.",
        "The quick brown fox jumps over the lazy dog.",
        "John loves Mary and she loves him too.",
        "In the beginning was the Word, and the Word was with God.",
    ];

    for (i, sentence) in test_sentences.iter().enumerate() {
        println!("\nTest {}: \"{}\"", i + 1, sentence);

        // Warm-up run
        let _ = parser.parse_document(sentence)?;

        // Timed runs
        let mut times = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let result = parser.parse_document(sentence)?;
            let duration = start.elapsed();
            times.push(duration.as_nanos() as f64);

            if i == 0 {
                // Show first result in detail
                println!("  Words parsed: {}", result.len());
                for (j, word) in result.iter().take(3).enumerate() {
                    println!("    {}: {} [{}] -> {}",
                        word.word.id, word.word.text, word.word.upos, word.word.lemma);
                }
                if result.len() > 3 {
                    println!("    ... and {} more", result.len() - 3);
                }
            }
        }

        let avg_ns = times.iter().sum::<f64>() / times.len();
        let min_ns = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_ns = times.iter().fold(0.0, |a, &b| a.max(b));

        println!("  Performance:");
        println!("    Average: {:.1}μs", avg_ns / 1000.0);
        println!("    Min:     {:.1}μs", min_ns / 1000.0);
        println!("    Max:     {:.1}μs", max_ns / 1000.0);
        println!("    Characters: {}", sentence.len());
        println!("    μs/char: {:.3}", (avg_ns / 1000.0) / sentence.len() as f64);
    }

    Ok(())
}

fn test_with_real_model() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Testing with Real UDPipe Model");
    println!("-".repeat(50));

    // Try to load real model
    let model_path = "/Users/gabe/projects/canopy/models/test.model";

    if !std::path::Path::new(model_path).exists() {
        println!("⚠️  Real model not found at {}", model_path);
        println!("   Skipping real UDPipe latency test");
        return Ok(());
    }

    println!("Loading real UDPipe model from: {}", model_path);
    let engine = UDPipeEngine::load(model_path)?;

    // Create parser with debug configuration
    let config = Layer1Config {
        enable_features: true,
        max_sentence_length: 200,
        debug: true,
    };
    let parser = Layer1Parser::with_config(engine, config);

    let test_sentences = [
        "The cat sat on the mat.",
        "She gave him a beautiful book yesterday.",
        "The quick brown fox jumps over the lazy dog.",
        "Complex sentence with subordinate clauses that test parsing accuracy.",
    ];

    for (i, sentence) in test_sentences.iter().enumerate() {
        println!("\nReal Model Test {}: \"{}\"", i + 1, sentence);

        // Single timed run (real UDPipe might be slower)
        let start = Instant::now();
        let result = parser.parse_document(sentence)?;
        let duration = start.elapsed();

        println!("  Parse time: {:.1}μs", duration.as_nanos() as f64 / 1000.0);
        println!("  Words parsed: {}", result.len());
        println!("  Enhanced features:");

        for word in &result {
            if word.animacy.is_some() || word.concreteness.is_some() {
                println!("    {}: {:?} animacy, {:?} concreteness",
                    word.word.text, word.animacy, word.concreteness);
            }
        }

        // Check for real morphological features from UDPipe
        let features_count = result.iter()
            .filter(|w| !w.word.feats.features.is_empty())
            .count();
        println!("  Words with morphological features: {}", features_count);

        // Performance metrics
        let chars = sentence.len();
        let words = result.len();
        println!("  Performance:");
        println!("    μs/char: {:.3}", (duration.as_nanos() as f64 / 1000.0) / chars as f64);
        println!("    μs/word: {:.1}", (duration.as_nanos() as f64 / 1000.0) / words as f64);

        // Target validation (keep under 500μs per sentence)
        let target_us = 500.0;
        let actual_us = duration.as_nanos() as f64 / 1000.0;
        if actual_us < target_us {
            println!("    ✅ Under 500μs target ({:.1}μs)", actual_us);
        } else {
            println!("    ⚠️  Over 500μs target ({:.1}μs)", actual_us);
        }
    }

    Ok(())
}

fn test_linguistic_accuracy() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧠 Testing Linguistic Analysis Accuracy");
    println!("-".repeat(50));

    let engine = UDPipeEngine::for_testing();
    let parser = Layer1Parser::new(engine);

    let test_cases = vec![
        ("The dog barked.", vec!["dog"], vec!["barked"]),
        ("John gave Mary a book.", vec!["John", "Mary"], vec!["book"]),
        ("The computer crashed.", vec![], vec!["computer"]),
    ];

    for (sentence, expected_animate, expected_inanimate) in test_cases {
        println!("\nAnalyzing: \"{}\"", sentence);
        let result = parser.parse_document(sentence)?;

        for word in result {
            if let Some(animacy) = &word.animacy {
                println!("  {}: {:?}", word.word.text, animacy);
            }
        }
    }

    Ok(())
}
