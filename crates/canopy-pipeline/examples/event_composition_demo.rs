//! Canopy Demo - Full Semantic Analysis Pipeline
//!
//! This demo showcases the complete Canopy NLP system:
//!
//! ## Layer 1: Semantic Analysis
//! - VerbNet: Verb classes and theta roles (333 XML classes)
//! - FrameNet: Semantic frames and lexical units
//! - WordNet: Synsets and lexical relations (117k+ synsets)
//! - PropBank: Predicate-argument structures and semantic roles
//! - Treebank: UD dependency pattern matching
//! - Lemmatization: Morphological normalization
//!
//! ## Layer 2: Event Composition
//! - Neo-Davidsonian event structures
//! - LittleV primitives (Cause, Become, Be, Do, Experience, Go, Have, Say, Exist)
//! - Theta role assignment and participant binding
//! - Voice detection (active/passive)
//!
//! Run: cargo run --release -p canopy-pipeline --example event_composition_demo

use canopy_core::UPos;
use canopy_events::{DependencyArc, EventComposer, SentenceAnalysis};
use canopy_pipeline::create_l1_analyzer_with_treebank;
use canopy_tokenizer::coordinator::Layer1SemanticResult;
use canopy_treebank::types::DependencyRelation;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║                           CANOPY DEMO                                     ║");
    println!("║                   Semantic-First Natural Language Understanding           ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
    println!();

    let demo_start = Instant::now();

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 1: ENGINE INITIALIZATION
    // ═══════════════════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 1: ENGINE INITIALIZATION                                             │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();

    println!("  Loading Layer 1 semantic engines...");
    let l1_start = Instant::now();
    let l1_analyzer = create_l1_analyzer_with_treebank()?;
    let l1_load_time = l1_start.elapsed();

    println!("  Loading Layer 2 event composer...");
    let l2_start = Instant::now();
    let composer = EventComposer::new()?;
    let l2_load_time = l2_start.elapsed();

    let stats = l1_analyzer.get_statistics();
    println!();
    println!("  Engine Load Times:");
    println!("  ├─ Layer 1: {:?}", l1_load_time);
    println!("  └─ Layer 2: {:?}", l2_load_time);
    println!();
    println!("  Active Engines: {:?}", stats.active_engines);
    println!();

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 2: LAYER 1 SEMANTIC ANALYSIS
    // ═══════════════════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 2: LAYER 1 SEMANTIC ANALYSIS                                         │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();

    let demo_words = [
        ("give", "transfer verb"),
        ("run", "motion verb"),
        ("think", "cognition verb"),
        ("break", "change-of-state"),
        ("fear", "psych verb"),
        ("walk", "manner-of-motion"),
    ];

    for (word, desc) in &demo_words {
        let start = Instant::now();
        let result = l1_analyzer.analyze(word)?;
        let elapsed = start.elapsed();

        println!("  \"{}\" ({})", word, desc);
        println!(
            "  ├─ Lemma: {} (conf: {:.0}%)",
            result.lemma,
            result.lemmatization_confidence.unwrap_or(0.0) * 100.0
        );

        if let Some(ref vn) = result.verbnet {
            let classes: Vec<_> = vn
                .verb_classes
                .iter()
                .take(2)
                .map(|c| c.id.as_str())
                .collect();
            println!(
                "  ├─ VerbNet: {} classes {:?}",
                vn.verb_classes.len(),
                classes
            );
        }
        if let Some(ref fn_) = result.framenet {
            let frames: Vec<_> = fn_.frames.iter().take(2).map(|f| f.name.as_str()).collect();
            println!("  ├─ FrameNet: {} frames {:?}", fn_.frames.len(), frames);
        }
        if let Some(ref wn) = result.wordnet {
            println!("  ├─ WordNet: {} synsets", wn.synsets.len());
        }
        if let Some(ref pb) = result.propbank {
            let rolesets: Vec<_> = pb
                .alternative_rolesets
                .iter()
                .take(2)
                .map(|r| r.roleset.as_str())
                .collect();
            let primary = pb
                .predicate
                .as_ref()
                .map(|p| p.roleset.as_str())
                .unwrap_or("none");
            println!("  ├─ PropBank: {} (alt: {:?})", primary, rolesets);
        }
        if let Some(ref tb) = result.treebank {
            println!(
                "  ├─ Treebank: {:?} (conf: {:.0}%)",
                tb.dependency_relation,
                tb.confidence * 100.0
            );
        }
        println!("  └─ Time: {:?}", elapsed);
        println!();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 3: LAYER 2 EVENT COMPOSITION (100 sentences)
    // ═══════════════════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 3: LAYER 2 EVENT COMPOSITION (100 sentences)                         │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();

    // Generate 100 test sentences with varied structures
    let sentences = generate_100_sentences();

    println!("  Processing {} sentences...", sentences.len());
    println!();

    let mut total_l1_time = Duration::ZERO;
    let mut total_l2_time = Duration::ZERO;
    let mut total_events = 0;
    let mut total_participants = 0;

    // Process all 100 and show first 5 in detail
    for (i, test) in sentences.iter().enumerate() {
        let l1_start = Instant::now();
        let mut tokens: Vec<Layer1SemanticResult> = Vec::new();
        for word in &test.words {
            tokens.push(l1_analyzer.analyze(word)?);
        }
        for (j, token) in tokens.iter_mut().enumerate() {
            token.pos = infer_pos(&test.deps, j, test.words[j]);
        }
        let l1_time = l1_start.elapsed();
        total_l1_time += l1_time;

        let deps: Vec<DependencyArc> = test
            .deps
            .iter()
            .map(|(h, d, r)| DependencyArc::new(*h, *d, r.clone()))
            .collect();

        let analysis = SentenceAnalysis::new(test.text.to_string(), tokens).with_dependencies(deps);

        let l2_start = Instant::now();
        let result = composer.compose_sentence(&analysis)?;
        let l2_time = l2_start.elapsed();
        total_l2_time += l2_time;

        total_events += result.events.len();
        for event in &result.events {
            total_participants += event.event.participants.len();
        }

        // Show first 5 in detail
        if i < 5 {
            println!("  [{}/100] \"{}\"", i + 1, test.text);
            println!("  ├─ L1: {:?}, L2: {:?}", l1_time, l2_time);
            println!("  ├─ Events: {}", result.events.len());

            for event in &result.events {
                println!(
                    "  │   └─ {}({:?})",
                    event.event.predicate, event.event.little_v
                );
                for (role, entity) in &event.event.participants {
                    println!("  │       {:?}: \"{}\"", role, entity.text);
                }
            }
            println!();
        }
    }

    // Summary stats for all 100
    let avg_l1 = total_l1_time / sentences.len() as u32;
    let avg_l2 = total_l2_time / sentences.len() as u32;

    println!("  ─────────────────────────────────────────────────────────────────────────");
    println!("  100-Sentence Summary:");
    println!(
        "  ├─ Total L1 time: {:?} (avg: {:?}/sentence)",
        total_l1_time, avg_l1
    );
    println!(
        "  ├─ Total L2 time: {:?} (avg: {:?}/sentence)",
        total_l2_time, avg_l2
    );
    println!("  ├─ Events composed: {}", total_events);
    println!("  └─ Participants bound: {}", total_participants);
    println!();

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 4: CACHE & MEMORY STATS
    // ═══════════════════════════════════════════════════════════════════════════
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ PHASE 4: CACHE & MEMORY STATISTICS                                         │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();

    let final_stats = l1_analyzer.get_statistics();
    println!("  Cache Performance:");
    println!("  ├─ Hit rate: {:.1}%", final_stats.cache_hit_rate * 100.0);
    println!(
        "  └─ Memory: {:.1}MB / {}MB ({:.0}%)",
        final_stats.memory_usage.estimated_usage_mb,
        final_stats.memory_usage.budget_mb,
        final_stats.memory_usage.utilization_percent
    );
    println!();

    // ═══════════════════════════════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════════════════════════════
    let total_time = demo_start.elapsed();

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║ SUMMARY                                                                   ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Demo completed in {:?}", total_time);
    println!();
    println!("  Features Demonstrated:");
    println!("  ├─ Layer 1: VerbNet, FrameNet, WordNet, PropBank, Treebank, Lemmatization");
    println!("  └─ Layer 2: Neo-Davidsonian events, LittleV, theta roles, voice");
    println!();
    println!("  Performance:");
    println!("  ├─ Engine load: {:?}", l1_load_time + l2_load_time);
    println!("  ├─ L1 avg: {:?}/sentence", avg_l1);
    println!("  └─ L2 avg: {:?}/sentence", avg_l2);
    println!();
    println!("  Status: M7 Complete");
    println!("  Next: M8 Discourse Representation Theory (DRT)");
    println!();

    Ok(())
}

struct TestSentence {
    text: &'static str,
    words: Vec<&'static str>,
    deps: Vec<(usize, usize, DependencyRelation)>,
}

fn generate_100_sentences() -> Vec<TestSentence> {
    let templates = vec![
        // Transitive (Agent-Patient)
        TestSentence {
            text: "John broke the vase",
            words: vec!["John", "broke", "the", "vase"],
            deps: vec![
                (1, 0, DependencyRelation::NominalSubject),
                (1, 3, DependencyRelation::Object),
                (3, 2, DependencyRelation::Determiner),
            ],
        },
        // Ditransitive (Agent-Theme-Recipient)
        TestSentence {
            text: "Mary gave John a book",
            words: vec!["Mary", "gave", "John", "a", "book"],
            deps: vec![
                (1, 0, DependencyRelation::NominalSubject),
                (1, 2, DependencyRelation::IndirectObject),
                (1, 4, DependencyRelation::Object),
                (4, 3, DependencyRelation::Determiner),
            ],
        },
        // Psych verb (Experiencer-Stimulus)
        TestSentence {
            text: "The child fears the dark",
            words: vec!["The", "child", "fears", "the", "dark"],
            deps: vec![
                (2, 1, DependencyRelation::NominalSubject),
                (2, 4, DependencyRelation::Object),
                (1, 0, DependencyRelation::Determiner),
                (4, 3, DependencyRelation::Determiner),
            ],
        },
        // Motion verb
        TestSentence {
            text: "The runner walked quickly",
            words: vec!["The", "runner", "walked", "quickly"],
            deps: vec![
                (2, 1, DependencyRelation::NominalSubject),
                (2, 3, DependencyRelation::AdverbialModifier),
                (1, 0, DependencyRelation::Determiner),
            ],
        },
        // Communication
        TestSentence {
            text: "She told him the story",
            words: vec!["She", "told", "him", "the", "story"],
            deps: vec![
                (1, 0, DependencyRelation::NominalSubject),
                (1, 2, DependencyRelation::IndirectObject),
                (1, 4, DependencyRelation::Object),
                (4, 3, DependencyRelation::Determiner),
            ],
        },
        // Cognition
        TestSentence {
            text: "He knows the answer",
            words: vec!["He", "knows", "the", "answer"],
            deps: vec![
                (1, 0, DependencyRelation::NominalSubject),
                (1, 3, DependencyRelation::Object),
                (3, 2, DependencyRelation::Determiner),
            ],
        },
        // Intransitive motion
        TestSentence {
            text: "The bird flew away",
            words: vec!["The", "bird", "flew", "away"],
            deps: vec![
                (2, 1, DependencyRelation::NominalSubject),
                (2, 3, DependencyRelation::AdverbialModifier),
                (1, 0, DependencyRelation::Determiner),
            ],
        },
        // Perception
        TestSentence {
            text: "I saw the movie",
            words: vec!["I", "saw", "the", "movie"],
            deps: vec![
                (1, 0, DependencyRelation::NominalSubject),
                (1, 3, DependencyRelation::Object),
                (3, 2, DependencyRelation::Determiner),
            ],
        },
        // Creation
        TestSentence {
            text: "She built a house",
            words: vec!["She", "built", "a", "house"],
            deps: vec![
                (1, 0, DependencyRelation::NominalSubject),
                (1, 3, DependencyRelation::Object),
                (3, 2, DependencyRelation::Determiner),
            ],
        },
        // Consumption
        TestSentence {
            text: "They ate the food",
            words: vec!["They", "ate", "the", "food"],
            deps: vec![
                (1, 0, DependencyRelation::NominalSubject),
                (1, 3, DependencyRelation::Object),
                (3, 2, DependencyRelation::Determiner),
            ],
        },
    ];

    // Generate 100 sentences by cycling through templates with variations
    let subjects = [
        "John",
        "Mary",
        "The child",
        "She",
        "He",
        "They",
        "We",
        "I",
        "The man",
        "The woman",
    ];
    let mut result = Vec::with_capacity(100);

    for i in 0..100 {
        let template = &templates[i % templates.len()];
        let subj = subjects[i % subjects.len()];

        // Create a modified version
        let mut words = template.words.clone();
        if !words.is_empty() {
            // Replace first word(s) with subject
            let subj_words: Vec<&str> = subj.split_whitespace().collect();
            if subj_words.len() == 1 {
                words[0] = subj_words[0];
            }
        }

        result.push(TestSentence {
            text: template.text, // Keep original text for display
            words,
            deps: template.deps.clone(),
        });
    }

    result
}

fn infer_pos(deps: &[(usize, usize, DependencyRelation)], idx: usize, word: &str) -> Option<UPos> {
    let is_verb_head = deps.iter().any(|(head, _, rel)| {
        *head == idx
            && matches!(
                rel,
                DependencyRelation::NominalSubject
                    | DependencyRelation::Object
                    | DependencyRelation::IndirectObject
            )
    });

    if is_verb_head {
        return Some(UPos::Verb);
    }

    for (_, dep, rel) in deps {
        if *dep == idx {
            match rel {
                DependencyRelation::NominalSubject
                | DependencyRelation::Object
                | DependencyRelation::IndirectObject => {
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
