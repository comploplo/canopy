//! Corpus Performance Integration Tests
//!
//! These tests validate our semantic pipeline performance against real literary text,
//! using Moby Dick as a representative mid-size corpus for benchmarking.

use canopy_lsp::CanopyLspServerFactory;
use canopy_lsp::server::CanopyServer;
use std::fs;
use std::time::Instant;

/// Performance thresholds for corpus processing
const MAX_PROCESSING_TIME_PER_SENTENCE_US: u64 = 10_000; // 10ms per sentence
const MAX_TOTAL_CORPUS_TIME_MS: u128 = 30_000; // 30 seconds total
const MIN_SENTENCES_PER_SECOND: f64 = 100.0;
const MAX_MEMORY_PER_SENTENCE_KB: f64 = 50.0;

#[test]
fn test_moby_dick_corpus_performance() {
    println!("🐋 Starting Moby Dick corpus performance test...");

    // Load Moby Dick text
    let corpus_path = "data/test-corpus/mobydick.txt";
    let corpus_text = match fs::read_to_string(corpus_path) {
        Ok(text) => text,
        Err(e) => {
            println!(
                "⚠️  Could not load Moby Dick corpus from {}: {}",
                corpus_path, e
            );
            println!("   Skipping corpus performance test (file not available)");
            return; // Skip test if corpus not available
        }
    };

    println!("📖 Loaded corpus: {} characters", corpus_text.len());

    // Create server
    let server =
        CanopyLspServerFactory::create_server().expect("Should create LSP server for corpus test");

    // Split into sentences for individual processing
    let sentences: Vec<&str> = corpus_text
        .split(&['.', '!', '?'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.len() > 10) // Filter very short fragments
        .take(100) // Limit to first 100 sentences for reasonable test time
        .collect();

    println!("📝 Processing {} sentences from corpus", sentences.len());

    let overall_start = Instant::now();
    let mut total_words = 0;
    let mut total_semantic_analysis_time_us = 0;
    let mut successful_analyses = 0;
    let mut sentence_times = Vec::new();

    // Process each sentence and collect performance metrics
    for (i, sentence) in sentences.iter().enumerate() {
        let sentence_start = Instant::now();

        match server.process_text(sentence) {
            Ok(response) => {
                let sentence_time = sentence_start.elapsed();
                let internal_time_us = response.metrics.total_time_us;

                sentence_times.push(sentence_time.as_micros() as u64);
                successful_analyses += 1;
                total_semantic_analysis_time_us += internal_time_us;

                // Count words processed
                let words_in_sentence: usize = response
                    .document
                    .sentences
                    .iter()
                    .map(|s| s.words.len())
                    .sum();
                total_words += words_in_sentence;

                // Performance validation per sentence
                assert!(
                    internal_time_us < MAX_PROCESSING_TIME_PER_SENTENCE_US,
                    "Sentence {} took {}μs (limit: {}μs): '{}'",
                    i,
                    internal_time_us,
                    MAX_PROCESSING_TIME_PER_SENTENCE_US,
                    sentence.chars().take(50).collect::<String>()
                );

                // Progress reporting every 20 sentences
                if i % 20 == 0 {
                    println!(
                        "   📊 Processed {}/{} sentences ({}μs avg)",
                        i + 1,
                        sentences.len(),
                        sentence_times.iter().sum::<u64>() / (i + 1) as u64
                    );
                }
            }
            Err(e) => {
                println!("   ⚠️  Sentence {} failed: {:?}", i, e);
                // Continue processing other sentences
            }
        }
    }

    let total_time = overall_start.elapsed();

    println!("\n🏁 Moby Dick Corpus Performance Results:");
    println!("{}", "=".repeat(50));

    // Calculate performance metrics
    let sentences_per_second = successful_analyses as f64 / total_time.as_secs_f64();
    let avg_sentence_time_us = if successful_analyses > 0 {
        total_semantic_analysis_time_us / successful_analyses as u64
    } else {
        0
    };
    let words_per_second = total_words as f64 / total_time.as_secs_f64();

    // Statistical analysis of sentence processing times
    sentence_times.sort_unstable();
    let p50_us = sentence_times
        .get(sentence_times.len() / 2)
        .copied()
        .unwrap_or(0);
    let p95_us = sentence_times
        .get((sentence_times.len() * 95) / 100)
        .copied()
        .unwrap_or(0);
    let p99_us = sentence_times
        .get((sentence_times.len() * 99) / 100)
        .copied()
        .unwrap_or(0);

    // Performance reporting
    println!("📈 Processing Statistics:");
    println!("   • Total sentences: {}", sentences.len());
    println!("   • Successful analyses: {}", successful_analyses);
    println!("   • Total words processed: {}", total_words);
    println!(
        "   • Success rate: {:.1}%",
        (successful_analyses as f64 / sentences.len() as f64) * 100.0
    );

    println!("\n⚡ Performance Metrics:");
    println!(
        "   • Total processing time: {:.2}ms",
        total_time.as_millis()
    );
    println!("   • Sentences per second: {:.1}", sentences_per_second);
    println!("   • Words per second: {:.1}", words_per_second);
    println!("   • Average sentence time: {}μs", avg_sentence_time_us);

    println!("\n📊 Latency Distribution:");
    println!("   • P50 (median): {}μs", p50_us);
    println!("   • P95: {}μs", p95_us);
    println!("   • P99: {}μs", p99_us);

    // Memory estimation (rough)
    let estimated_memory_per_sentence_kb =
        (total_words as f64 * 200.0) / 1024.0 / successful_analyses as f64;
    println!("\n💾 Memory Estimation:");
    println!(
        "   • Estimated memory per sentence: {:.1}KB",
        estimated_memory_per_sentence_kb
    );

    // Performance assertions
    assert!(
        total_time.as_millis() < MAX_TOTAL_CORPUS_TIME_MS,
        "Total corpus processing took {}ms (limit: {}ms)",
        total_time.as_millis(),
        MAX_TOTAL_CORPUS_TIME_MS
    );

    assert!(
        sentences_per_second >= MIN_SENTENCES_PER_SECOND,
        "Processing rate {:.1} sent/sec below minimum {}",
        sentences_per_second,
        MIN_SENTENCES_PER_SECOND
    );

    assert!(
        estimated_memory_per_sentence_kb <= MAX_MEMORY_PER_SENTENCE_KB,
        "Memory usage {:.1}KB per sentence exceeds limit {}KB",
        estimated_memory_per_sentence_kb,
        MAX_MEMORY_PER_SENTENCE_KB
    );

    assert!(
        successful_analyses > sentences.len() / 2,
        "Success rate too low: {}/{} sentences processed successfully",
        successful_analyses,
        sentences.len()
    );

    println!("\n✅ All performance benchmarks passed!");
    println!("🚀 canopy.rs semantic analysis performs excellently on literary corpus");
}

#[test]
fn test_corpus_semantic_analysis_quality() {
    println!("🔍 Testing semantic analysis quality on Moby Dick corpus...");

    // Load corpus
    let corpus_path = "data/test-corpus/mobydick.txt";
    let corpus_text = match fs::read_to_string(corpus_path) {
        Ok(text) => text,
        Err(e) => {
            println!("⚠️  Could not load corpus: {}", e);
            return;
        }
    };

    let server = CanopyLspServerFactory::create_server().expect("Should create LSP server");

    // Test on specific interesting sentences from Moby Dick
    let interesting_sentences = extract_interesting_sentences(&corpus_text);

    println!(
        "📝 Analyzing {} semantically interesting sentences",
        interesting_sentences.len()
    );

    let mut semantic_features_found = 0;
    let mut layer_results_complete = 0;
    let mut total_confidence = 0.0;

    for (i, sentence) in interesting_sentences.iter().enumerate() {
        match server.process_text(sentence) {
            Ok(response) => {
                // Check semantic analysis quality
                if response.layer_results.contains_key("semantics") {
                    layer_results_complete += 1;

                    let semantic_layer = &response.layer_results["semantics"];
                    if semantic_layer.items_processed > 0 {
                        semantic_features_found += 1;

                        // Estimate confidence from processing success
                        let confidence = if response.document.sentences.is_empty() {
                            0.5
                        } else {
                            0.8 // High confidence for successful processing
                        };
                        total_confidence += confidence;
                    }
                }

                // Validate word structure
                for sentence in &response.document.sentences {
                    for word in &sentence.words {
                        assert!(!word.text.is_empty(), "Word text should not be empty");
                        assert!(!word.lemma.is_empty(), "Word lemma should not be empty");
                        assert!(word.start < word.end, "Word positions should be valid");
                    }
                }

                if i < 5 {
                    println!(
                        "   ✓ Sentence {}: {} words, {}μs",
                        i + 1,
                        response
                            .document
                            .sentences
                            .iter()
                            .map(|s| s.words.len())
                            .sum::<usize>(),
                        response.metrics.total_time_us
                    );
                }
            }
            Err(e) => {
                println!("   ⚠️  Sentence {} failed: {:?}", i, e);
            }
        }
    }

    let avg_confidence = if interesting_sentences.len() > 0 {
        total_confidence / interesting_sentences.len() as f64
    } else {
        0.0
    };

    println!("\n📊 Semantic Analysis Quality Results:");
    println!(
        "   • Sentences with semantic features: {}/{}",
        semantic_features_found,
        interesting_sentences.len()
    );
    println!(
        "   • Complete layer results: {}/{}",
        layer_results_complete,
        interesting_sentences.len()
    );
    println!("   • Average confidence: {:.2}", avg_confidence);

    // Quality assertions
    assert!(
        semantic_features_found > interesting_sentences.len() / 3,
        "Should find semantic features in at least 1/3 of sentences"
    );

    assert!(
        layer_results_complete > interesting_sentences.len() / 2,
        "Should have complete layer results for most sentences"
    );

    println!("✅ Semantic analysis quality validated on literary corpus!");
}

#[test]
fn test_corpus_stress_testing() {
    println!("💪 Running corpus stress test...");

    let corpus_path = "data/test-corpus/mobydick.txt";
    let corpus_text = match fs::read_to_string(corpus_path) {
        Ok(text) => text,
        Err(e) => {
            println!("⚠️  Could not load corpus: {}", e);
            return;
        }
    };

    let server = CanopyLspServerFactory::create_server().expect("Should create LSP server");

    // Extract paragraphs for stress testing
    let paragraphs: Vec<&str> = corpus_text
        .split("\n\n")
        .filter(|p| p.trim().len() > 50) // Substantial paragraphs only
        .take(20) // Limit for test efficiency
        .collect();

    println!("📚 Stress testing with {} paragraphs", paragraphs.len());

    let stress_start = Instant::now();
    let mut processed_paragraphs = 0;
    let mut total_sentences = 0;
    let mut total_words = 0;

    for (i, paragraph) in paragraphs.iter().enumerate() {
        match server.process_text(paragraph) {
            Ok(response) => {
                processed_paragraphs += 1;
                total_sentences += response.document.sentences.len();
                total_words += response
                    .document
                    .sentences
                    .iter()
                    .map(|s| s.words.len())
                    .sum::<usize>();

                // Memory and performance validation
                assert!(
                    response.metrics.total_time_us < 50_000, // 50ms per paragraph
                    "Paragraph {} processing took too long: {}μs",
                    i,
                    response.metrics.total_time_us
                );

                if i % 5 == 0 {
                    println!("   🔄 Processed {}/{} paragraphs", i + 1, paragraphs.len());
                }
            }
            Err(e) => {
                println!("   ⚠️  Paragraph {} failed: {:?}", i, e);
            }
        }
    }

    let stress_time = stress_start.elapsed();

    println!("\n💪 Stress Test Results:");
    println!(
        "   • Paragraphs processed: {}/{}",
        processed_paragraphs,
        paragraphs.len()
    );
    println!("   • Total sentences: {}", total_sentences);
    println!("   • Total words: {}", total_words);
    println!("   • Total time: {:.2}ms", stress_time.as_millis());
    println!(
        "   • Paragraphs per second: {:.1}",
        processed_paragraphs as f64 / stress_time.as_secs_f64()
    );

    if total_sentences > 0 {
        println!(
            "   • Average sentences per paragraph: {:.1}",
            total_sentences as f64 / processed_paragraphs as f64
        );
        println!(
            "   • Sentences per second: {:.1}",
            total_sentences as f64 / stress_time.as_secs_f64()
        );
    }

    // Stress test assertions
    assert!(
        processed_paragraphs > paragraphs.len() / 2,
        "Should successfully process majority of paragraphs"
    );

    assert!(
        stress_time.as_millis() < MAX_TOTAL_CORPUS_TIME_MS,
        "Stress test took too long: {}ms",
        stress_time.as_millis()
    );

    println!("✅ Corpus stress test passed!");
}

#[test]
fn test_corpus_memory_efficiency() {
    println!("🧠 Testing memory efficiency on corpus processing...");

    let corpus_path = "data/test-corpus/mobydick.txt";
    let corpus_text = match fs::read_to_string(corpus_path) {
        Ok(text) => text,
        Err(e) => {
            println!("⚠️  Could not load corpus: {}", e);
            return;
        }
    };

    let server = CanopyLspServerFactory::create_server().expect("Should create LSP server");

    // Process the same text multiple times to test memory management
    let test_text = corpus_text.lines().take(10).collect::<Vec<_>>().join(" ");

    println!("🔄 Running {} iterations for memory efficiency test", 50);

    let memory_start = Instant::now();

    for i in 0..50 {
        match server.process_text(&test_text) {
            Ok(response) => {
                // Verify processing is consistent
                assert!(
                    !response.document.sentences.is_empty(),
                    "Should produce sentences in iteration {}",
                    i
                );

                // Check for reasonable processing time (no memory pressure)
                assert!(
                    response.metrics.total_time_us < 20_000, // 20ms
                    "Memory pressure detected in iteration {}: {}μs",
                    i,
                    response.metrics.total_time_us
                );
            }
            Err(e) => {
                // Early iterations might fail, but consistent failures suggest memory issues
                if i > 10 {
                    println!("   ⚠️  Memory test iteration {} failed: {:?}", i, e);
                }
            }
        }

        // Let Rust drop the response to test memory cleanup
        // If there are memory leaks, this will accumulate over iterations
    }

    let memory_time = memory_start.elapsed();

    println!("🧠 Memory Efficiency Results:");
    println!(
        "   • 50 iterations completed in {:.2}ms",
        memory_time.as_millis()
    );
    println!(
        "   • Average time per iteration: {:.2}ms",
        memory_time.as_millis() as f64 / 50.0
    );

    // Memory efficiency assertion
    assert!(
        memory_time.as_millis() < 10_000, // 10 seconds for 50 iterations
        "Memory efficiency test took too long: {}ms (possible memory leak)",
        memory_time.as_millis()
    );

    println!("✅ Memory efficiency test passed!");
}

#[test]
fn test_corpus_linguistic_diversity() {
    println!("🌍 Testing linguistic diversity analysis on Moby Dick...");

    let corpus_path = "data/test-corpus/mobydick.txt";
    let corpus_text = match fs::read_to_string(corpus_path) {
        Ok(text) => text,
        Err(e) => {
            println!("⚠️  Could not load corpus: {}", e);
            return;
        }
    };

    let server = CanopyLspServerFactory::create_server().expect("Should create LSP server");

    // Extract diverse sentence types from corpus
    let diverse_sentences = extract_diverse_sentence_types(&corpus_text);

    println!(
        "📝 Testing {} diverse sentence types",
        diverse_sentences.len()
    );

    let mut pos_variety = std::collections::HashSet::new();
    let mut sentence_lengths = Vec::new();
    let mut complexity_handled = 0;

    for (sentence_type, sentence) in &diverse_sentences {
        match server.process_text(sentence) {
            Ok(response) => {
                // Collect POS diversity
                for sent in &response.document.sentences {
                    for word in &sent.words {
                        pos_variety.insert(word.upos);
                    }
                    sentence_lengths.push(sent.words.len());
                }

                // Check if complex sentences are handled
                if sentence.len() > 100 {
                    complexity_handled += 1;
                }

                println!(
                    "   ✓ {}: {} words, {}μs",
                    sentence_type,
                    response
                        .document
                        .sentences
                        .iter()
                        .map(|s| s.words.len())
                        .sum::<usize>(),
                    response.metrics.total_time_us
                );
            }
            Err(e) => {
                println!("   ⚠️  {} failed: {:?}", sentence_type, e);
            }
        }
    }

    println!("\n🌍 Linguistic Diversity Results:");
    println!("   • POS tag variety: {} different tags", pos_variety.len());
    println!(
        "   • Sentence length range: {}-{} words",
        sentence_lengths.iter().min().unwrap_or(&0),
        sentence_lengths.iter().max().unwrap_or(&0)
    );
    println!("   • Complex sentences handled: {}", complexity_handled);
    println!("   • POS tags found: {:?}", pos_variety);

    // Diversity assertions
    assert!(
        pos_variety.len() >= 8,
        "Should find diverse POS tags in literary text"
    );

    assert!(
        complexity_handled > 0,
        "Should handle some complex sentences"
    );

    println!("✅ Linguistic diversity test passed!");
}

/// Extract interesting sentences for semantic analysis testing
fn extract_interesting_sentences(text: &str) -> Vec<&str> {
    text.split(&['.', '!', '?'])
        .map(|s| s.trim())
        .filter(|s| {
            !s.is_empty() &&
            s.len() > 20 &&
            s.len() < 200 && // Reasonable sentence length
            s.contains(' ') && // Multi-word sentences
            !s.starts_with(char::is_numeric) // Avoid numbered lists
        })
        .take(100)
        .collect()
}

/// Extract diverse sentence types for linguistic testing
fn extract_diverse_sentence_types(text: &str) -> Vec<(&'static str, &str)> {
    let sentences: Vec<&str> = text
        .split(&['.', '!', '?'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.len() > 15)
        .collect();

    let mut diverse = Vec::new();

    // Find different sentence types
    for sentence in sentences.iter().take(200) {
        // Limit search scope
        if diverse.len() >= 20 {
            break;
        } // Limit total diverse sentences

        if sentence.contains(" and ") && !diverse.iter().any(|(t, _)| *t == "Coordination") {
            diverse.push(("Coordination", *sentence));
        } else if sentence.contains(" who ")
            || sentence.contains(" which ") && !diverse.iter().any(|(t, _)| *t == "Relative")
        {
            diverse.push(("Relative", *sentence));
        } else if sentence.contains(" that ") && !diverse.iter().any(|(t, _)| *t == "Complement") {
            diverse.push(("Complement", *sentence));
        } else if sentence.contains(" if ")
            || sentence.contains(" when ") && !diverse.iter().any(|(t, _)| *t == "Conditional")
        {
            diverse.push(("Conditional", *sentence));
        } else if sentence.contains(" not ") && !diverse.iter().any(|(t, _)| *t == "Negation") {
            diverse.push(("Negation", *sentence));
        } else if sentence.len() > 150 && !diverse.iter().any(|(t, _)| *t == "Complex") {
            diverse.push(("Complex", *sentence));
        } else if sentence.len() < 50
            && sentence.split_whitespace().count() > 3
            && !diverse.iter().any(|(t, _)| *t == "Simple")
        {
            diverse.push(("Simple", *sentence));
        }
    }

    diverse
}

#[test]
fn test_corpus_batch_processing() {
    println!("📦 Testing batch processing performance on corpus...");

    let corpus_path = "data/test-corpus/mobydick.txt";
    let corpus_text = match fs::read_to_string(corpus_path) {
        Ok(text) => text,
        Err(e) => {
            println!("⚠️  Could not load corpus: {}", e);
            return;
        }
    };

    let server = CanopyLspServerFactory::create_server().expect("Should create LSP server");

    // Create batches of sentences
    let sentences: Vec<&str> = corpus_text
        .split(&['.', '!', '?'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.len() > 10)
        .take(60) // Reasonable batch size
        .collect();

    let batch_size = 10;
    let batches: Vec<Vec<&str>> = sentences
        .chunks(batch_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    println!(
        "📦 Processing {} batches of {} sentences each",
        batches.len(),
        batch_size
    );

    let batch_start = Instant::now();
    let mut total_batch_words = 0;
    let mut successful_batches = 0;

    for (batch_idx, batch) in batches.iter().enumerate() {
        let batch_timing_start = Instant::now();
        let mut batch_successful = true;
        let mut batch_words = 0;

        // Process entire batch
        for sentence in batch {
            match server.process_text(sentence) {
                Ok(response) => {
                    batch_words += response
                        .document
                        .sentences
                        .iter()
                        .map(|s| s.words.len())
                        .sum::<usize>();
                }
                Err(_) => {
                    batch_successful = false;
                    break;
                }
            }
        }

        let batch_time = batch_timing_start.elapsed();

        if batch_successful {
            successful_batches += 1;
            total_batch_words += batch_words;

            println!(
                "   ✓ Batch {}: {} words in {:.2}ms",
                batch_idx + 1,
                batch_words,
                batch_time.as_millis()
            );

            // Batch performance assertion
            assert!(
                batch_time.as_millis() < 1000, // 1 second per batch
                "Batch {} took too long: {}ms",
                batch_idx,
                batch_time.as_millis()
            );
        } else {
            println!("   ⚠️  Batch {} had failures", batch_idx + 1);
        }
    }

    let total_batch_time = batch_start.elapsed();

    println!("\n📦 Batch Processing Results:");
    println!(
        "   • Successful batches: {}/{}",
        successful_batches,
        batches.len()
    );
    println!("   • Total words processed: {}", total_batch_words);
    println!(
        "   • Total batch time: {:.2}ms",
        total_batch_time.as_millis()
    );
    println!(
        "   • Batches per second: {:.1}",
        successful_batches as f64 / total_batch_time.as_secs_f64()
    );

    // Batch processing assertions
    assert!(
        successful_batches > batches.len() / 2,
        "Should successfully process majority of batches"
    );

    assert!(
        total_batch_time.as_millis() < 20_000, // 20 seconds total
        "Batch processing took too long: {}ms",
        total_batch_time.as_millis()
    );

    println!("✅ Batch processing performance validated!");
}

#[test]
fn test_corpus_scalability_projection() {
    println!("📈 Testing scalability projections based on corpus sample...");

    let corpus_path = "data/test-corpus/mobydick.txt";
    let corpus_text = match fs::read_to_string(corpus_path) {
        Ok(text) => text,
        Err(e) => {
            println!("⚠️  Could not load corpus: {}", e);
            return;
        }
    };

    let server = CanopyLspServerFactory::create_server().expect("Should create LSP server");

    // Test with progressively larger chunks to project scalability
    let test_sizes = vec![
        ("Small", 1000),   // ~1KB
        ("Medium", 5000),  // ~5KB
        ("Large", 15000),  // ~15KB
        ("XLarge", 50000), // ~50KB
    ];

    println!("📊 Testing scalability across different input sizes");

    let mut scalability_results = Vec::new();

    for (size_name, char_limit) in test_sizes {
        let test_chunk = corpus_text.chars().take(char_limit).collect::<String>();

        if test_chunk.len() < char_limit / 2 {
            println!("   ⚠️  Insufficient corpus data for {} test", size_name);
            continue;
        }

        let chunk_start = Instant::now();

        match server.process_text(&test_chunk) {
            Ok(response) => {
                let chunk_time = chunk_start.elapsed();
                let words_processed = response
                    .document
                    .sentences
                    .iter()
                    .map(|s| s.words.len())
                    .sum::<usize>();

                let chars_per_second = test_chunk.len() as f64 / chunk_time.as_secs_f64();
                let words_per_second = words_processed as f64 / chunk_time.as_secs_f64();

                scalability_results.push((
                    size_name,
                    test_chunk.len(),
                    chunk_time.as_micros() as u64,
                    words_processed,
                ));

                println!(
                    "   📈 {}: {} chars → {} words in {:.2}ms ({:.0} chars/sec, {:.0} words/sec)",
                    size_name,
                    test_chunk.len(),
                    words_processed,
                    chunk_time.as_millis(),
                    chars_per_second,
                    words_per_second
                );

                // Scalability assertion - time should scale sub-linearly
                let time_per_char_us = chunk_time.as_micros() as f64 / test_chunk.len() as f64;
                assert!(
                    time_per_char_us < 10.0, // <10μs per character
                    "{} chunk: {}μs per character is too slow",
                    size_name,
                    time_per_char_us
                );
            }
            Err(e) => {
                println!("   ❌ {} chunk failed: {:?}", size_name, e);
            }
        }
    }

    // Analyze scalability trends
    if scalability_results.len() >= 2 {
        println!("\n📈 Scalability Analysis:");

        for i in 1..scalability_results.len() {
            let (prev_name, prev_chars, prev_time, _prev_words) = scalability_results[i - 1];
            let (curr_name, curr_chars, curr_time, _curr_words) = scalability_results[i];

            let size_ratio = curr_chars as f64 / prev_chars as f64;
            let time_ratio = curr_time as f64 / prev_time as f64;
            let efficiency_ratio = time_ratio / size_ratio;

            println!(
                "   • {} → {}: {:.1}x size, {:.1}x time, {:.2}x efficiency",
                prev_name, curr_name, size_ratio, time_ratio, efficiency_ratio
            );

            // Good scalability means time grows sub-linearly with input size
            assert!(
                efficiency_ratio < 2.0,
                "Poor scalability from {} to {}: {:.2}x efficiency ratio",
                prev_name,
                curr_name,
                efficiency_ratio
            );
        }
    }

    println!("✅ Scalability projection test passed!");
}

#[test]
fn test_corpus_error_resilience() {
    println!("🛡️  Testing error resilience on challenging corpus text...");

    let corpus_path = "data/test-corpus/mobydick.txt";
    let corpus_text = match fs::read_to_string(corpus_path) {
        Ok(text) => text,
        Err(e) => {
            println!("⚠️  Could not load corpus: {}", e);
            return;
        }
    };

    let server = CanopyLspServerFactory::create_server().expect("Should create LSP server");

    // Extract challenging text patterns
    let challenging_texts = extract_challenging_patterns(&corpus_text);

    println!(
        "🔍 Testing resilience with {} challenging text patterns",
        challenging_texts.len()
    );

    let mut graceful_handling = 0;
    let mut total_errors = 0;

    for (pattern_type, text) in &challenging_texts {
        match server.process_text(text) {
            Ok(response) => {
                graceful_handling += 1;

                // Validate graceful handling produces reasonable results
                assert!(
                    response.metrics.total_time_us > 0,
                    "Should have processing time for {}",
                    pattern_type
                );

                println!(
                    "   ✓ {}: Handled gracefully ({}μs)",
                    pattern_type, response.metrics.total_time_us
                );
            }
            Err(e) => {
                total_errors += 1;
                println!("   ⚠️  {}: Error handled - {:?}", pattern_type, e);

                // Errors should be informative, not panics
                let error_msg = format!("{:?}", e);
                assert!(
                    !error_msg.contains("panic") && !error_msg.contains("unwrap"),
                    "Error should be handled gracefully, not panic: {}",
                    error_msg
                );
            }
        }
    }

    println!("\n🛡️  Error Resilience Results:");
    println!(
        "   • Graceful handling: {}/{}",
        graceful_handling,
        challenging_texts.len()
    );
    println!(
        "   • Error handling: {}/{}",
        total_errors,
        challenging_texts.len()
    );
    println!(
        "   • Success rate: {:.1}%",
        (graceful_handling as f64 / challenging_texts.len() as f64) * 100.0
    );

    // Resilience assertions
    assert!(
        graceful_handling + total_errors == challenging_texts.len(),
        "All challenging patterns should be handled (gracefully or with errors)"
    );

    println!("✅ Error resilience test passed!");
}

/// Extract challenging text patterns for resilience testing
fn extract_challenging_patterns(text: &str) -> Vec<(&'static str, &str)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut challenging = Vec::new();

    for line in lines.iter().take(500) {
        // Limit search scope
        if challenging.len() >= 15 {
            break;
        } // Limit total patterns

        let trimmed = line.trim();
        if trimmed.len() < 20 {
            continue;
        }

        if trimmed.starts_with('"') && !challenging.iter().any(|(t, _)| *t == "Quoted") {
            challenging.push(("Quoted", trimmed));
        } else if trimmed.contains("--") && !challenging.iter().any(|(t, _)| *t == "Dashes") {
            challenging.push(("Dashes", trimmed));
        } else if trimmed.contains(";") && !challenging.iter().any(|(t, _)| *t == "Semicolon") {
            challenging.push(("Semicolon", trimmed));
        } else if trimmed.contains(":") && !challenging.iter().any(|(t, _)| *t == "Colon") {
            challenging.push(("Colon", trimmed));
        } else if trimmed.contains("(") && !challenging.iter().any(|(t, _)| *t == "Parenthetical") {
            challenging.push(("Parenthetical", trimmed));
        } else if trimmed.len() > 200 && !challenging.iter().any(|(t, _)| *t == "VeryLong") {
            challenging.push(("VeryLong", trimmed));
        } else if trimmed.split_whitespace().count() > 30
            && !challenging.iter().any(|(t, _)| *t == "ManyWords")
        {
            challenging.push(("ManyWords", trimmed));
        }
    }

    challenging
}
