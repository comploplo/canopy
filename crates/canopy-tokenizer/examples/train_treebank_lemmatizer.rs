//! Example of training TreebankLemmatizer with real UD English-EWT data
//!
//! This example shows how to train the TreebankLemmatizer on actual treebank data
//! and demonstrates its performance compared to SimpleLemmatizer.

use canopy_tokenizer::{Lemmatizer, SimpleLemmatizer, TreebankLemmatizer};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let training_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-train.conllu");

    if !training_path.exists() {
        println!("Training data not found at {:?}", training_path);
        println!("Please ensure UD English-EWT data is available");
        return Ok(());
    }

    println!("Training TreebankLemmatizer on UD English-EWT data...");
    let start_time = Instant::now();

    let mut treebank_lemmatizer = TreebankLemmatizer::new()?;
    treebank_lemmatizer.train_from_file(training_path)?;

    let training_time = start_time.elapsed();
    let stats = treebank_lemmatizer.get_stats();

    println!("\n=== Training Results ===");
    println!("Training time: {:.2}s", training_time.as_secs_f32());
    println!("Total mappings processed: {}", stats.total_mappings);
    println!("Unique words learned: {}", stats.unique_words);
    println!("Direct mappings stored: {}", stats.direct_mappings);
    println!("Suffix rules learned: {}", stats.suffix_rules);

    // Compare with SimpleLemmatizer
    let simple_lemmatizer = SimpleLemmatizer::new()?;

    println!("\n=== Performance Comparison ===");
    let test_words = vec![
        "running", "children", "gave", "better", "women", "feet", "dogs", "cats", "walking",
        "jumped", "books", "houses",
    ];

    println!(
        "{:<12} {:<12} {:<12} {:<12} {:<12}",
        "Word", "Treebank", "TreeConf", "Simple", "SimpleConf"
    );
    println!("{:-<60}", "");

    for word in &test_words {
        let (treebank_lemma, treebank_conf) = treebank_lemmatizer.lemmatize_with_confidence(word);
        let (simple_lemma, simple_conf) = simple_lemmatizer.lemmatize_with_confidence(word);

        println!(
            "{:<12} {:<12} {:<12.2} {:<12} {:<12.2}",
            word, treebank_lemma, treebank_conf, simple_lemma, simple_conf
        );
    }

    // Performance benchmark
    println!("\n=== Speed Benchmark ===");
    let benchmark_words: Vec<String> = test_words
        .iter()
        .cycle()
        .take(1000)
        .map(|s| s.to_string())
        .collect();

    let start = Instant::now();
    let _treebank_results: Vec<String> = benchmark_words
        .iter()
        .map(|w| treebank_lemmatizer.lemmatize(w))
        .collect();
    let treebank_time = start.elapsed();

    let start = Instant::now();
    let _simple_results: Vec<String> = benchmark_words
        .iter()
        .map(|w| simple_lemmatizer.lemmatize(w))
        .collect();
    let simple_time = start.elapsed();

    println!(
        "TreebankLemmatizer: {:.2}μs per word ({:.0} words/sec)",
        treebank_time.as_micros() as f32 / 1000.0,
        1000.0 / treebank_time.as_secs_f32()
    );

    println!(
        "SimpleLemmatizer:   {:.2}μs per word ({:.0} words/sec)",
        simple_time.as_micros() as f32 / 1000.0,
        1000.0 / simple_time.as_secs_f32()
    );

    println!(
        "Speed ratio: {:.1}x",
        treebank_time.as_secs_f32() / simple_time.as_secs_f32()
    );

    Ok(())
}
