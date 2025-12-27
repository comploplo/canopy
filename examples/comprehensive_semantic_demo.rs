//! Comprehensive Canopy Semantic Analysis Demo
//!
//! This demo showcases all Canopy semantic engines working together:
//! - VerbNet: Verb classification and theta roles
//! - FrameNet: Frame semantic analysis
//! - PropBank: Semantic role labeling (SRL)
//! - WordNet: Lexical semantic relations
//! - Treebank: Gold-standard UD parses
//! - Semantic Coordinator: Unified analysis pipeline
//!
//! ⚠️  CURRENT LIMITATION: Treebank-based parsing only
//!    Uses gold-standard UD English-EWT parses from ~16,600 sentences
//!    Future: Pattern synthesis for arbitrary sentences (v0.8+)

use canopy_core::treebank_loader::TreebankSentenceLoader;
use canopy_engine::traits::{CachedEngine, StatisticsProvider};
use canopy_propbank::{PropBankConfig, PropBankEngine};
use canopy_tokenizer::{SemanticCoordinator, coordinator::CoordinatorConfig};
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

// Create demo PropBank data for the integrated demo
fn create_demo_propbank_data() -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir
        .path()
        .join("propbank-release")
        .join("data")
        .join("google")
        .join("ewt");
    fs::create_dir_all(&data_dir)?;

    // PropBank framesets that align with common English verbs
    let prop_content = r#"# PropBank Demo Data - Common English Predicates

# Basic transitive verbs
give.01: ARG0:giver ARG1:thing_given ARG2:recipient ARGM-LOC:location ARGM-TMP:time
take.01: ARG0:taker ARG1:thing_taken ARG2:taken_from ARGM-LOC:location
run.01: ARG0:runner ARGM-LOC:location ARGM-MNR:manner ARGM-DIR:direction
think.01: ARG0:thinker ARG1:thought ARGM-CAU:cause ARGM-TMP:time
know.01: ARG0:knower ARG1:thing_known ARGM-TMP:time
chase.01: ARG0:chaser ARG1:thing_chased ARGM-LOC:location
sleep.01: ARG0:sleeper ARGM-LOC:location ARGM-TMP:time"#;

    fs::write(data_dir.join("demo_framesets.prop"), prop_content)?;
    Ok(temp_dir)
}

#[derive(Debug, Clone)]
struct SentenceAnalysisResult {
    sentence_id: String,
    sentence_text: String,
    word_count: usize,
    duration: std::time::Duration,
    semantic_features: usize,
    verbnet_found: bool,
    framenet_found: bool,
    wordnet_found: bool,
    propbank_predicates: Vec<String>,
}

fn analyze_sentence_comprehensive(
    coordinator: &mut SemanticCoordinator,
    propbank_engine: &PropBankEngine,
    loader: &TreebankSentenceLoader,
    sentence_id: &str,
) -> Result<SentenceAnalysisResult, Box<dyn std::error::Error>> {
    let start = Instant::now();

    // Get treebank sentence
    let sentence = loader
        .get_sentence(sentence_id)
        .ok_or_else(|| format!("Sentence '{}' not found in treebank", sentence_id))?;

    // Convert to words with gold-standard parses
    let words = loader.convert_to_words(sentence)?;

    // Analyze each word for semantic features
    let mut semantic_features = 0;
    let mut verbnet_found = false;
    let mut framenet_found = false;
    let mut wordnet_found = false;
    let mut propbank_predicates = Vec::new();

    for word in &words {
        // Coordinator analysis
        if let Ok(result) = coordinator.analyze(&word.lemma) {
            if result.verbnet.is_some() {
                semantic_features += 1;
                verbnet_found = true;
            }
            if result.framenet.is_some() {
                semantic_features += 1;
                framenet_found = true;
            }
            if result.wordnet.is_some() {
                semantic_features += 1;
                wordnet_found = true;
            }
        }

        // PropBank analysis
        if let Ok(pb_result) = propbank_engine.analyze_word(&word.lemma)
            && pb_result.data.has_match()
            && pb_result.confidence > 0.3
        {
            propbank_predicates.push(word.lemma.clone());
        }
    }

    let duration = start.elapsed();

    Ok(SentenceAnalysisResult {
        sentence_id: sentence_id.to_string(),
        sentence_text: sentence.text.clone(),
        word_count: words.len(),
        duration,
        semantic_features,
        verbnet_found,
        framenet_found,
        wordnet_found,
        propbank_predicates,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌟 Canopy Comprehensive Semantic Analysis Demo");
    println!("{}", "=".repeat(80));
    println!("Showcasing integrated semantic engines:");
    println!("• VerbNet: Verb classification and theta roles");
    println!("• FrameNet: Frame semantic analysis");
    println!("• PropBank: Semantic role labeling (SRL)");
    println!("• WordNet: Lexical semantic relations");
    println!("• Treebank: Gold-standard UD dependency parsing");
    println!("• Semantic Coordinator: Unified analysis pipeline\n");

    // 1. Load treebank data
    println!("📚 1. Loading UD English-EWT treebank...");
    let loader = TreebankSentenceLoader::new()?;
    println!(
        "✅ Loaded: {} dev + {} train + {} test sentences",
        loader.dev_count(),
        loader.train_count(),
        loader.test_count()
    );

    // 2. Initialize PropBank engine
    println!("\n🔧 2. Setting up PropBank engine...");
    let temp_dir = create_demo_propbank_data()?;
    let propbank_config = PropBankConfig::default()
        .with_data_path(temp_dir.path().join("propbank-release").join("data"))
        .with_prop_files(true)
        .with_cache(true, 1000)
        .with_fuzzy_matching(true);

    let propbank_engine = PropBankEngine::with_config(propbank_config)?;
    println!(
        "✅ PropBank engine initialized with {} predicates",
        propbank_engine.get_propbank_stats().total_predicates
    );

    // 3. Initialize Semantic Coordinator
    println!("\n🔧 3. Setting up Semantic Coordinator...");
    let coord_config = CoordinatorConfig {
        enable_verbnet: true,
        enable_framenet: true,
        enable_wordnet: true,
        enable_lexicon: false,
        enable_treebank: true,
        enable_lemmatization: true,
        use_advanced_lemmatization: false,
        confidence_threshold: 0.3,
        l1_cache_memory_mb: 100,
        use_treebank_lemmas: true,
        lemma_confidence_threshold: 0.5,
        enable_shared_lemma_cache: true,
        cache_capacity: 2000,
        enable_cache_warmup: false,
        cache_warmup_common_words: false,
    };

    let init_start = Instant::now();
    let mut coordinator = SemanticCoordinator::new(coord_config)?;

    // Add treebank support
    match canopy_treebank::TreebankEngine::with_config(
        canopy_treebank::engine::TreebankConfig::default(),
    ) {
        Ok(treebank_engine) => {
            use std::sync::Arc;
            coordinator.set_treebank_provider(Arc::new(treebank_engine));
            println!("✅ Treebank dependency analysis enabled");
        }
        Err(e) => {
            println!(
                "⚠️  Treebank initialization failed: {}. Continuing without dependency analysis.",
                e
            );
        }
    }

    let init_time = init_start.elapsed();
    println!(
        "✅ Semantic Coordinator initialized in {}ms",
        init_time.as_millis()
    );

    // 4. Analyze diverse canonical sentences from treebank
    let demo_sentences = vec![
        "canonical-001", // John gave Mary a book.
        "canonical-002", // The cat is sleeping.
        "canonical-003", // They are running quickly.
        "canonical-004", // She knows the answer.
        "canonical-005", // We saw the movie yesterday.
        "canonical-006", // He will come tomorrow.
        "canonical-007", // Mary ran quickly.
        "canonical-008", // The cat is sleeping.
        "canonical-009", // Birds fly south.
        "canonical-010", // The dog chased the cat.
        "canonical-011", // Water freezes at zero.
        "canonical-012", // They are running.
        "canonical-013", // I think she knows.
        "canonical-014", // The sun rises.
        "canonical-015", // Children play games.
    ];

    println!("\n{}", "=".repeat(80));
    println!("🚀 COMPREHENSIVE SEMANTIC ANALYSIS");
    println!("{}", "=".repeat(80));
    println!(
        "Analyzing {} treebank sentences with gold-standard parses...\n",
        demo_sentences.len()
    );

    // Analyze all sentences and collect results
    let mut results = Vec::new();
    for sentence_id in &demo_sentences {
        match analyze_sentence_comprehensive(
            &mut coordinator,
            &propbank_engine,
            &loader,
            sentence_id,
        ) {
            Ok(result) => results.push(result),
            Err(e) => println!("⚠️  Failed to analyze '{}': {}", sentence_id, e),
        }
    }

    if results.is_empty() {
        println!("❌ No sentences were successfully analyzed");
        return Ok(());
    }

    // Show top 5 most semantically rich sentences
    results.sort_by_key(|r| std::cmp::Reverse(r.semantic_features));

    println!("🌟 TOP 5 MOST SEMANTICALLY RICH SENTENCES");
    println!("{}", "=".repeat(80));
    for (i, result) in results.iter().take(5).enumerate() {
        println!(
            "\n{}. 📝 {} - \"{}\" ({:.1}ms)",
            i + 1,
            result.sentence_id,
            result.sentence_text,
            result.duration.as_millis()
        );
        println!(
            "   📊 {} words, {} semantic features",
            result.word_count, result.semantic_features
        );
        println!(
            "   🎯 Engines: VerbNet:{} FrameNet:{} WordNet:{}",
            if result.verbnet_found { "✓" } else { "✗" },
            if result.framenet_found { "✓" } else { "✗" },
            if result.wordnet_found { "✓" } else { "✗" }
        );
        if !result.propbank_predicates.is_empty() {
            println!(
                "   🏛️  PropBank: {} predicates ({})",
                result.propbank_predicates.len(),
                result.propbank_predicates.join(", ")
            );
        }
    }

    // Performance statistics
    println!("\n\n{}", "=".repeat(80));
    println!("📊 PERFORMANCE & STATISTICS SUMMARY");
    println!("{}", "=".repeat(80));

    let total_sentences = results.len();
    let total_time: std::time::Duration = results.iter().map(|r| r.duration).sum();
    let avg_time = total_time / total_sentences as u32;
    let min_time = results.iter().map(|r| r.duration).min().unwrap();
    let max_time = results.iter().map(|r| r.duration).max().unwrap();

    println!("\n⏱️  Timing Analysis ({} sentences):", total_sentences);
    println!("   • Total time: {:.1}ms", total_time.as_millis());
    println!("   • Average: {:.1}ms per sentence", avg_time.as_millis());
    println!(
        "   • Range: {:.1}ms - {:.1}ms",
        min_time.as_millis(),
        max_time.as_millis()
    );
    println!(
        "   • Throughput: {:.0} sentences/sec",
        1000.0 / avg_time.as_millis() as f64
    );

    // Semantic coverage
    let total_features: usize = results.iter().map(|r| r.semantic_features).sum();
    let total_words: usize = results.iter().map(|r| r.word_count).sum();
    println!("\n📈 Semantic Coverage:");
    println!("   • Total words analyzed: {}", total_words);
    println!("   • Total semantic features: {}", total_features);
    println!(
        "   • Features per word: {:.2}",
        total_features as f64 / total_words as f64
    );

    // Coordinator statistics
    let coord_stats = coordinator.get_statistics();
    println!("\n🎯 Semantic Coordinator Statistics:");
    println!("   • Active engines: {:?}", coord_stats.active_engines);
    println!(
        "   • Cache hit rate: {:.1}%",
        coord_stats.cache_hit_rate * 100.0
    );
    println!(
        "   • Memory usage: {:.1}MB / {}MB ({:.1}%)",
        coord_stats.memory_usage.estimated_usage_mb,
        coord_stats.memory_usage.budget_mb,
        coord_stats.memory_usage.utilization_percent
    );

    // PropBank statistics
    let propbank_stats = propbank_engine.get_propbank_stats();
    let pb_engine_stats = propbank_engine.statistics();
    println!("\n🏛️  PropBank Engine Statistics:");
    println!(
        "   • Total predicates loaded: {}",
        propbank_stats.total_predicates
    );
    println!(
        "   • Average arguments per predicate: {:.2}",
        propbank_stats.avg_arguments_per_predicate
    );
    println!(
        "   • Queries processed: {}",
        pb_engine_stats.performance.total_queries
    );
    println!(
        "   • Average query time: {:.2}μs",
        pb_engine_stats.performance.avg_query_time_us
    );

    // Cache statistics
    let cache_stats = propbank_engine.cache_stats();
    println!("   • Cache size: {}", cache_stats.current_size);
    println!("   • Total lookups: {}", cache_stats.total_lookups);

    // Cross-engine compatibility demo
    println!("\n{}", "=".repeat(80));
    println!("🔗 CROSS-ENGINE COMPATIBILITY DEMONSTRATION");
    println!("{}", "=".repeat(80));

    let test_verbs = ["give", "run", "think", "chase"];
    for verb in &test_verbs {
        println!("\n🎯 Analyzing verb: '{}'", verb);

        // PropBank analysis
        match propbank_engine.analyze_word(verb) {
            Ok(result) => {
                if let Some(ref predicate) = result.data.predicate {
                    println!(
                        "   🏛️  PropBank: {} (sense: {})",
                        predicate.lemma, predicate.sense
                    );

                    // Show theta role mapping for compatibility
                    let theta_roles: Vec<_> = predicate
                        .arguments
                        .iter()
                        .filter_map(|arg| arg.role.to_theta_role())
                        .collect();
                    if !theta_roles.is_empty() {
                        println!("      🔗 Maps to theta roles: {:?}", theta_roles);
                        println!(
                            "      ✅ Compatible with VerbNet and FrameNet theta role systems"
                        );
                    }
                }
            }
            Err(_) => {
                println!("   🏛️  PropBank: No analysis available");
            }
        }

        // Coordinator analysis for comparison
        match coordinator.analyze(verb) {
            Ok(result) => {
                if let Some(ref verbnet) = result.verbnet {
                    println!("   🎭 VerbNet: {} classes", verbnet.verb_classes.len());
                }
                if let Some(ref framenet) = result.framenet {
                    println!("   🖼️  FrameNet: {} frames", framenet.frames.len());
                }
                if let Some(ref wordnet) = result.wordnet {
                    println!("   📚 WordNet: {} synsets", wordnet.synsets.len());
                }
            }
            Err(_) => {
                println!("   🎯 Coordinator: Limited analysis available");
            }
        }
    }

    // Final Summary
    println!("\n{}", "=".repeat(80));
    println!("🎉 DEMO COMPLETED SUCCESSFULLY!");
    println!("{}", "=".repeat(80));

    println!("\n✅ Demonstrated Capabilities:");
    println!("   • Multi-engine semantic analysis pipeline");
    println!("   • Treebank-based gold-standard parsing");
    println!("   • PropBank semantic role labeling integration");
    println!("   • Cross-engine theta role compatibility");
    println!("   • High-performance caching and statistics");

    println!("\n🚀 Production Readiness:");
    if coord_stats.active_engines.len() >= 3 {
        println!(
            "   • EXCELLENT: {} semantic engines active",
            coord_stats.active_engines.len()
        );
        println!("   • Rich semantic features available");
    } else {
        println!(
            "   • PARTIAL: {} semantic engines active",
            coord_stats.active_engines.len()
        );
        println!("   • Load VerbNet/FrameNet/WordNet data for full capability");
    }

    if coord_stats.cache_hit_rate > 0.3 {
        println!(
            "   • Cache performance: GOOD ({:.1}% hit rate)",
            coord_stats.cache_hit_rate * 100.0
        );
    } else {
        println!("   • Cache performance: COLD START (expected for demo)");
    }

    println!("\n⚠️  CURRENT LIMITATION:");
    println!("   Canopy currently supports UD treebank sentences only.");
    println!("   Available: ~16,600 gold-standard UD English-EWT sentences");
    println!("   Future: VerbNet pattern synthesis for arbitrary sentences (v0.8+)");

    println!("\n💡 The Canopy semantic analysis framework successfully integrates");
    println!("   multiple complementary semantic engines into a unified,");
    println!("   high-performance natural language understanding system!");

    Ok(())
}
