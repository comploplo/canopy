//! Event Composition Demo - Full Layer 1 → Layer 2 Pipeline
//!
//! This example demonstrates the complete Canopy pipeline:
//! 1. Layer 1: Semantic analysis (VerbNet, FrameNet, WordNet)
//! 2. Layer 2: Event composition (Neo-Davidsonian events with theta roles)
//!
//! IMPORTANT: Performance timing shows HONEST end-to-end metrics:
//! - Engine loading: One-time startup cost (~100-500ms)
//! - Per-sentence L1 analysis: ~5-50ms per sentence (real semantic lookup)
//! - Per-sentence L2 composition: ~50-100μs per sentence (mostly in-memory)
//! - Total per-sentence: ~5-50ms (dominated by L1 semantic analysis)
//!
//! Run with: cargo run -p canopy-pipeline --example event_composition_demo

use canopy_core::UPos;
use canopy_events::{DependencyArc, EventComposer, SentenceAnalysis};
use canopy_pipeline::create_l1_analyzer;
use canopy_tokenizer::coordinator::Layer1SemanticResult;
use canopy_treebank::types::DependencyRelation;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Canopy Event Composition Demo ===\n");
    println!("NOTE: This demo shows HONEST end-to-end timing.\n");

    // Step 1: Create Layer 1 analyzer with all engines
    println!("Loading semantic engines (one-time startup cost)...");
    let start = Instant::now();
    let l1_analyzer = create_l1_analyzer()?;
    let engine_load_time = start.elapsed();
    println!("Engines loaded in {:?}\n", engine_load_time);

    // Step 2: Create Layer 2 event composer
    let composer = EventComposer::new()?;

    // Test sentences with their dependency structures
    let test_cases = vec![
        // Simple transitive: "John broke the vase"
        TestCase {
            sentence: "John broke the vase",
            words: vec!["John", "broke", "the", "vase"],
            dependencies: vec![
                (1, 0, DependencyRelation::NominalSubject), // John <- broke
                (1, 3, DependencyRelation::Object),         // vase <- broke
                (3, 2, DependencyRelation::Determiner),     // the <- vase
            ],
        },
        // Ditransitive: "Mary gave John a book"
        TestCase {
            sentence: "Mary gave John a book",
            words: vec!["Mary", "gave", "John", "a", "book"],
            dependencies: vec![
                (1, 0, DependencyRelation::NominalSubject), // Mary <- gave
                (1, 2, DependencyRelation::IndirectObject), // John <- gave
                (1, 4, DependencyRelation::Object),         // book <- gave
                (4, 3, DependencyRelation::Determiner),     // a <- book
            ],
        },
        // Psych verb: "The child fears the dark"
        TestCase {
            sentence: "The child fears the dark",
            words: vec!["The", "child", "fears", "the", "dark"],
            dependencies: vec![
                (2, 1, DependencyRelation::NominalSubject), // child <- fears
                (2, 4, DependencyRelation::Object),         // dark <- fears
                (1, 0, DependencyRelation::Determiner),     // The <- child
                (4, 3, DependencyRelation::Determiner),     // the <- dark
            ],
        },
        // Motion verb: "The runner walked quickly"
        TestCase {
            sentence: "The runner walked quickly",
            words: vec!["The", "runner", "walked", "quickly"],
            dependencies: vec![
                (2, 1, DependencyRelation::NominalSubject), // runner <- walked
                (2, 3, DependencyRelation::AdverbialModifier), // quickly <- walked
                (1, 0, DependencyRelation::Determiner),     // The <- runner
            ],
        },
    ];

    // Track total timing
    let mut total_l1_time = Duration::ZERO;
    let mut total_l2_time = Duration::ZERO;
    let mut sentence_count = 0;

    // Process each test case
    for test in test_cases {
        sentence_count += 1;
        let sentence_start = Instant::now();

        println!("Sentence: \"{}\"", test.sentence);
        println!("{:-<60}", "");

        // Run Layer 1 semantic analysis on each word
        let l1_start = Instant::now();
        let mut tokens: Vec<Layer1SemanticResult> = Vec::new();
        for word in &test.words {
            let result = l1_analyzer.analyze(word)?;
            tokens.push(result);
        }

        // Add POS tags based on position heuristics (simple demo)
        for (i, token) in tokens.iter_mut().enumerate() {
            token.pos = infer_pos(&test.dependencies, i, &test.words[i]);
        }
        let l1_time = l1_start.elapsed();
        total_l1_time += l1_time;

        // Build dependency arcs
        let deps: Vec<DependencyArc> = test
            .dependencies
            .iter()
            .map(|(head, dep, rel)| DependencyArc::new(*head, *dep, rel.clone()))
            .collect();

        // Create SentenceAnalysis
        let analysis =
            SentenceAnalysis::new(test.sentence.to_string(), tokens).with_dependencies(deps);

        // Run Layer 2 event composition
        let l2_start = Instant::now();
        let result = composer.compose_sentence(&analysis)?;
        let l2_time = l2_start.elapsed();
        total_l2_time += l2_time;

        let total_sentence_time = sentence_start.elapsed();

        // Display HONEST timing results
        println!("Timing Breakdown:");
        println!("  Layer 1 (semantic analysis): {:?}", l1_time);
        println!("  Layer 2 (event composition): {:?}", l2_time);
        println!("  Total end-to-end:            {:?}", total_sentence_time);
        println!("Events Composed: {}", result.events.len());

        for (i, event) in result.events.iter().enumerate() {
            println!("\n  Event {}: {}", i + 1, event.event.predicate);
            println!("    LittleV: {:?}", event.event.little_v);
            println!("    Voice: {:?}", event.event.voice);
            println!("    VerbNet Source: {:?}", event.verbnet_source);
            println!(
                "    Decomposition Confidence: {:.2}",
                event.decomposition_confidence
            );

            if !event.event.participants.is_empty() {
                println!("    Participants:");
                for (role, entity) in &event.event.participants {
                    println!("      - {:?}: \"{}\"", role, entity.text);
                }
            }
        }

        if !result.unbound_entities.is_empty() {
            println!("\n  Unbound Entities:");
            for entity in &result.unbound_entities {
                println!("    - \"{}\": {:?}", entity.text, entity.reason);
            }
        }

        println!("\n");
    }

    // Print summary with honest performance metrics
    println!("=== Performance Summary ===");
    println!("Engine load time (one-time):  {:?}", engine_load_time);
    println!("Sentences processed:          {}", sentence_count);
    println!("Total L1 time:                {:?}", total_l1_time);
    println!("Total L2 time:                {:?}", total_l2_time);
    println!(
        "Average per sentence:         {:?}",
        (total_l1_time + total_l2_time) / sentence_count as u32
    );
    println!();
    println!("IMPORTANT: The per-sentence time is dominated by L1 semantic analysis");
    println!("(VerbNet, FrameNet, WordNet lookups), which takes 5-50ms per sentence.");
    println!("L2 event composition is fast (~50-100μs) but cannot work without L1.");
    println!();
    println!("=== Demo Complete ===");
    Ok(())
}

struct TestCase {
    sentence: &'static str,
    words: Vec<&'static str>,
    dependencies: Vec<(usize, usize, DependencyRelation)>,
}

/// Simple POS inference based on dependency structure (for demo purposes)
fn infer_pos(deps: &[(usize, usize, DependencyRelation)], idx: usize, word: &str) -> Option<UPos> {
    // Check if this token is a verb head
    let is_verb_head = deps.iter().any(|(head, _, rel)| {
        *head == idx
            && matches!(
                rel,
                DependencyRelation::NominalSubject | DependencyRelation::Object
            )
    });

    if is_verb_head {
        return Some(UPos::Verb);
    }

    // Check dependency relations
    for (_, dep, rel) in deps {
        if *dep == idx {
            match rel {
                DependencyRelation::NominalSubject => {
                    // Heuristic: capitalized = proper noun, else noun
                    if word
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        return Some(UPos::Propn);
                    }
                    return Some(UPos::Noun);
                }
                DependencyRelation::Object | DependencyRelation::IndirectObject => {
                    if word
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        return Some(UPos::Propn);
                    }
                    return Some(UPos::Noun);
                }
                DependencyRelation::Determiner => return Some(UPos::Det),
                DependencyRelation::AdverbialModifier => return Some(UPos::Adv),
                _ => {}
            }
        }
    }

    None
}
