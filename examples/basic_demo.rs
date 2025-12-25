//! Canopy Semantic Analysis: Basic Demo
//!
//! This demo shows Canopy's semantic analysis on 100 random sentences from
//! Moby Dick, demonstrating the system's ability to process arbitrary text.

use canopy_events::{DependencyArc, EventComposer, SentenceAnalysis};
use canopy_tokenizer::coordinator::CoordinatorConfig;
use canopy_tokenizer::tokenization::Tokenizer;
use canopy_tokenizer::SemanticCoordinator;
use canopy_treebank::types::DependencyRelation;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::fs;
use std::sync::Arc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌲 Canopy Semantic Analysis - Moby Dick Demo");
    println!("=============================================\n");

    // Load Moby Dick text
    println!("📚 Loading Moby Dick...");
    let text = fs::read_to_string("data/test-corpus/mobydick.txt")?;

    // Extract sentences using tokenizer
    let tokenizer = Tokenizer::new();

    // Skip header/metadata and extract prose content
    // The book content starts after "CHAPTER 1"
    let content_start = text
        .find("CHAPTER 1")
        .or_else(|| text.find("LOOMINGS"))
        .unwrap_or(0);
    let content = &text[content_start..];

    // Extract sentences: split on sentence-ending punctuation
    // and filter to get good quality prose sentences
    let raw_sentences: Vec<&str> = content
        .split(|c| c == '.' || c == '!' || c == '?')
        .map(|s| s.trim())
        .filter(|s| {
            let word_count = s.split_whitespace().count();
            // Keep sentences with 8-25 words (reasonable prose length)
            // Exclude chapter headers, page numbers, and metadata
            word_count >= 8
                && word_count <= 25
                && !s.contains("CHAPTER")
                && !s.contains("MOBY-DICK")
                && !s.chars().any(|c| c.is_ascii_digit())
                && s.chars().filter(|c| c.is_alphabetic()).count() > 20
        })
        .collect();

    println!(
        "   ✅ Found {} prose sentences (8-25 words each)",
        raw_sentences.len()
    );

    // Select 100 random sentences with a fixed seed for reproducibility
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut selected: Vec<&str> = raw_sentences.clone();
    selected.shuffle(&mut rng);
    let sample_sentences: Vec<&str> = selected.into_iter().take(100).collect();

    println!("   📊 Selected 100 random sentences for analysis\n");

    // Configure semantic coordinator with real data
    let config = CoordinatorConfig {
        enable_verbnet: true,
        enable_framenet: true,
        enable_wordnet: true,
        enable_lexicon: false,
        enable_lemmatization: true,
        confidence_threshold: 0.3,
        l1_cache_memory_mb: 50,
        ..CoordinatorConfig::default()
    };

    println!("🔧 Initializing semantic engines...");
    let start = Instant::now();
    let mut coordinator = SemanticCoordinator::new(config)?;

    // Wire up treebank dependency analysis
    match canopy_treebank::TreebankEngine::with_config(canopy_treebank::engine::TreebankConfig::default()) {
        Ok(treebank_engine) => {
            coordinator.set_treebank_provider(Arc::new(treebank_engine));
            println!("   ✅ Treebank dependency analysis enabled");
        }
        Err(e) => {
            println!("   ⚠️  Treebank disabled: {}", e);
        }
    }

    println!("   ✅ Initialized in {}ms\n", start.elapsed().as_millis());

    // Analyze sentences
    println!("📊 Analyzing 100 Moby Dick sentences:");
    println!("{}", "=".repeat(60));

    let analysis_start = Instant::now();
    let mut total_words = 0;
    let mut total_features = 0;
    let mut sentences_with_semantic_data = 0;

    // Show first 5 sentences in detail, then summarize the rest
    for (i, sentence) in sample_sentences.iter().enumerate() {
        // Tokenize the sentence
        let tokens = match tokenizer.tokenize(sentence) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let content_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| !t.is_punctuation && t.is_content_word)
            .collect();

        let mut sentence_features = 0;

        // Analyze each content word
        for token in &content_tokens {
            // Use analyze() for arbitrary text (no POS tags available)
            if let Ok(result) = coordinator.analyze(&token.text) {
                if result.verbnet.is_some() {
                    sentence_features += 1;
                }
                if result.framenet.is_some() {
                    sentence_features += 1;
                }
                if result.wordnet.is_some() {
                    sentence_features += 1;
                }
                if result.treebank.is_some() {
                    sentence_features += 1;
                }
            }
        }

        total_words += content_tokens.len();
        total_features += sentence_features;

        if sentence_features > 0 {
            sentences_with_semantic_data += 1;
        }

        // Show details for first 5 sentences only
        if i < 5 {
            let truncated = if sentence.len() > 70 {
                format!("{}...", &sentence[..67])
            } else {
                sentence.to_string()
            };
            println!("\n🔍 [{}] \"{}\"", i + 1, truncated);
            println!(
                "   {} content words, {} semantic features",
                content_tokens.len(),
                sentence_features
            );

            // Show some semantic hits
            let mut shown = 0;
            for token in &content_tokens {
                if shown >= 3 {
                    break;
                }
                if let Ok(result) = coordinator.analyze(&token.text) {
                    let mut features = Vec::new();
                    if result.verbnet.is_some() {
                        features.push("VerbNet");
                    }
                    if result.framenet.is_some() {
                        features.push("FrameNet");
                    }
                    if result.wordnet.is_some() {
                        features.push("WordNet");
                    }
                    if result.treebank.is_some() {
                        features.push("Treebank");
                    }
                    if !features.is_empty() {
                        println!("   • {}: {}", token.text, features.join(", "));
                        shown += 1;
                    }
                }
            }
        }
    }

    let analysis_time = analysis_start.elapsed();

    // Summary statistics
    println!("\n{}", "=".repeat(60));
    println!("📈 Analysis Summary:");
    println!("   • Sentences analyzed: 100");
    println!(
        "   • Sentences with semantic data: {} ({:.1}%)",
        sentences_with_semantic_data,
        sentences_with_semantic_data as f64
    );
    println!("   • Total content words: {}", total_words);
    println!("   • Total semantic features: {}", total_features);
    println!(
        "   • Average features per sentence: {:.1}",
        total_features as f64 / 100.0
    );
    println!("   • Analysis time: {}ms", analysis_time.as_millis());
    println!(
        "   • Throughput: {:.0} words/sec",
        total_words as f64 / analysis_time.as_secs_f64()
    );

    // Cache statistics
    let stats = coordinator.get_statistics();
    println!("\n💾 Cache Performance:");
    println!("   • Hit rate: {:.1}%", stats.cache_hit_rate * 100.0);
    println!(
        "   • Memory: {:.1}MB / {}MB",
        stats.memory_usage.estimated_usage_mb, stats.memory_usage.budget_mb
    );

    // Layer 2: Event Composition Demo
    println!("\n{}", "=".repeat(60));
    println!("🎭 Layer 2: Event Composition Demo");
    println!("{}", "=".repeat(60));

    let event_composer = EventComposer::new()?;
    println!("   ✅ EventComposer initialized\n");

    // Show event composition for a few carefully selected sentences
    let demo_sentences = [
        ("John gives Mary a book", vec![
            ("John", "john", Some(canopy_core::UPos::Propn)),
            ("gives", "give", Some(canopy_core::UPos::Verb)),
            ("Mary", "mary", Some(canopy_core::UPos::Propn)),
            ("a", "a", Some(canopy_core::UPos::Det)),
            ("book", "book", Some(canopy_core::UPos::Noun)),
        ], vec![
            (1, 0, DependencyRelation::NominalSubject),  // John <- gives
            (1, 2, DependencyRelation::IndirectObject),  // Mary <- gives
            (1, 4, DependencyRelation::Object),          // book <- gives
        ]),
        ("The whale broke the boat", vec![
            ("The", "the", Some(canopy_core::UPos::Det)),
            ("whale", "whale", Some(canopy_core::UPos::Noun)),
            ("broke", "break", Some(canopy_core::UPos::Verb)),
            ("the", "the", Some(canopy_core::UPos::Det)),
            ("boat", "boat", Some(canopy_core::UPos::Noun)),
        ], vec![
            (2, 1, DependencyRelation::NominalSubject),  // whale <- broke
            (2, 4, DependencyRelation::Object),          // boat <- broke
        ]),
        ("Ahab fears the whale", vec![
            ("Ahab", "ahab", Some(canopy_core::UPos::Propn)),
            ("fears", "fear", Some(canopy_core::UPos::Verb)),
            ("the", "the", Some(canopy_core::UPos::Det)),
            ("whale", "whale", Some(canopy_core::UPos::Noun)),
        ], vec![
            (1, 0, DependencyRelation::NominalSubject),  // Ahab <- fears
            (1, 3, DependencyRelation::Object),          // whale <- fears
        ]),
    ];

    for (sentence_text, token_data, deps) in &demo_sentences {
        println!("📝 \"{}\"", sentence_text);

        // Build Layer 1 tokens
        let tokens: Vec<canopy_tokenizer::coordinator::Layer1SemanticResult> = token_data
            .iter()
            .map(|(word, lemma, pos)| {
                let mut result = canopy_tokenizer::coordinator::Layer1SemanticResult {
                    original_word: word.to_string(),
                    lemma: lemma.to_string(),
                    pos: *pos,
                    lemmatization_confidence: Some(0.9),
                    verbnet: None,
                    framenet: None,
                    wordnet: None,
                    lexicon: None,
                    treebank: None,
                    confidence: 0.85,
                    sources: vec![],
                    errors: vec![],
                };
                // Enrich with semantic data from coordinator
                if let Ok(enriched) = coordinator.analyze(lemma) {
                    result.verbnet = enriched.verbnet;
                    result.framenet = enriched.framenet;
                }
                result
            })
            .collect();

        // Build dependency arcs
        let dependency_arcs: Vec<DependencyArc> = deps
            .iter()
            .map(|(head, dep, rel)| DependencyArc::new(*head, *dep, rel.clone()))
            .collect();

        // Create sentence analysis
        let analysis = SentenceAnalysis::new(sentence_text.to_string(), tokens)
            .with_dependencies(dependency_arcs);

        // Compose events
        match event_composer.compose_sentence(&analysis) {
            Ok(composed) => {
                if composed.has_events() {
                    for event in &composed.events {
                        println!("   🎯 Event: {}", event.event.predicate);
                        println!("      LittleV: {:?}", event.event.little_v);
                        println!("      Voice: {:?}", event.event.voice);
                        println!("      Participants: {} bound", event.event.participants.len());
                        for (role, entity) in &event.event.participants {
                            println!("         • {:?} → \"{}\"", role, entity.text);
                        }
                        println!(
                            "      Confidence: {:.1}%",
                            event.overall_confidence() * 100.0
                        );
                    }
                } else {
                    println!("   ⚠️  No events composed");
                }
            }
            Err(e) => println!("   ❌ Error: {}", e),
        }
        println!();
    }

    println!("✨ Demo complete!");
    Ok(())
}
