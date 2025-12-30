//! Canopy Demo - Semantic Analysis Pipeline
//!
//! Analyzes sentences from Moby Dick through the three-layer semantic pipeline.
//!
//! Run: cargo run --example demo --release

use canopy_events::{DependencyArc, EventComposer, SentenceAnalysis, SentenceMetadata};
use canopy_pipeline::{create_l1_analyzer_with_treebank, DiscourseProcessor};
use canopy_tokenizer::{DependencyRelation as L1Rel, SentenceAnalysisResult};
use canopy_treebank::types::DependencyRelation as L2Rel;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::fs;
use std::time::Instant;

const SENTENCE_COUNT: usize = 100;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\nCanopy Semantic Analysis Demo");
    println!("==============================\n");

    let start = Instant::now();

    // Load engines
    print!("Loading engines... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let l1_start = Instant::now();
    let l1 = create_l1_analyzer_with_treebank()?;
    let l1_time = l1_start.elapsed();
    let l2 = EventComposer::new()?;
    let mut l3 = DiscourseProcessor::new();
    println!("done ({:.0?})\n", l1_time);

    // Load corpus
    let text = fs::read_to_string("data/test-corpus/mobydick.txt")?;
    let sentences = extract_sentences(&text);
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut selected: Vec<&str> = sentences.iter().map(|s| s.as_str()).collect();
    selected.shuffle(&mut rng);
    let selected: Vec<&str> = selected.into_iter().take(SENTENCE_COUNT).collect();
    println!("Analyzing {} sentences from Moby Dick...\n", selected.len());

    // Analyze
    let mut events = 0;
    let mut participants = 0;
    let mut modal = 0;
    let mut negated = 0;
    let mut presup = 0;
    let mut with_pos = 0;
    let mut with_deps = 0;
    let mut samples: Vec<(String, SentenceAnalysisResult)> = Vec::new();
    let analysis_start = Instant::now();

    for sentence in &selected {
        let words = sentence.split_whitespace().count();
        if words == 0 || words > 50 {
            continue;
        }

        let l1_result = match l1.analyze_sentence(sentence) {
            Ok(r) if !r.tokens.is_empty() => r,
            _ => continue,
        };

        if l1_result.tokens.iter().any(|t| t.pos.is_some()) {
            with_pos += 1;
        }
        if !l1_result.dependencies.is_empty() {
            with_deps += 1;
        }

        // Collect samples for display
        if samples.len() < 5 && (4..=15).contains(&words) {
            samples.push((sentence.to_string(), l1_result.clone()));
        }

        let analysis = convert_l1_to_l2(sentence, &l1_result);
        if let Ok(result) = l2.compose_sentence(&analysis) {
            events += result.events.len();
            for ev in &result.events {
                participants += ev.event.participants.len();
                if ev.event.modality.is_some() {
                    modal += 1;
                }
                if !ev.polarity {
                    negated += 1;
                }
                presup += ev.presuppositions.len();
            }
            let _ = l3.process_sentence(sentence, &result);
        }
    }
    let analysis_time = analysis_start.elapsed();

    // Print metrics
    let stats = l1.get_statistics();
    let total = start.elapsed();

    println!("METRICS");
    println!("-------");
    println!("Sentences:      {}", selected.len());
    println!("Events:         {}", events);
    println!("Participants:   {}\n", participants);

    println!("Layer 1 Coverage:");
    println!(
        "  POS tags:     {}/{} ({:.0}%)",
        with_pos,
        selected.len(),
        with_pos as f64 / selected.len() as f64 * 100.0
    );
    println!(
        "  Dependencies: {}/{} ({:.0}%)\n",
        with_deps,
        selected.len(),
        with_deps as f64 / selected.len() as f64 * 100.0
    );

    println!("Semantic Features:");
    println!("  Modal:          {}", modal);
    println!("  Negated:        {}", negated);
    println!("  Presuppositions: {}\n", presup);

    println!("Performance:");
    println!("  Engine load:  {:.0?}", l1_time);
    println!(
        "  Analysis:     {:.1?} ({:.1}ms/sentence)",
        analysis_time,
        analysis_time.as_millis() as f64 / selected.len() as f64
    );
    println!("  Cache hit:    {:.1}%", stats.cache_hit_rate * 100.0);
    println!(
        "  Memory:       {:.2}MB / {}MB",
        stats.memory_usage.estimated_usage_mb, stats.memory_usage.budget_mb
    );
    println!("  Total:        {:.1?}\n", total);

    // Show samples
    println!("SAMPLE ANALYSIS");
    println!("---------------");
    for (i, (text, l1_result)) in samples.iter().enumerate() {
        println!("\n[{}] \"{}\"", i + 1, truncate(text, 60));
        let pos: Vec<String> = l1_result
            .tokens
            .iter()
            .map(|t| {
                format!(
                    "{}:{}",
                    t.original_word,
                    t.pos.map(|p| format!("{:?}", p)).unwrap_or("?".into())
                )
            })
            .collect();
        println!("  POS: {}", pos.join(" "));

        if !l1_result.dependencies.is_empty() {
            let deps: Vec<String> = l1_result
                .dependencies
                .iter()
                .filter_map(|d| {
                    let head = l1_result.tokens.get(d.head_idx)?.original_word.as_str();
                    let dep = l1_result
                        .tokens
                        .get(d.dependent_idx)?
                        .original_word
                        .as_str();
                    Some(format!("{}({}, {})", d.relation, head, dep))
                })
                .collect();
            println!("  Deps: {}", deps.join(", "));
        }

        let analysis = convert_l1_to_l2(text, l1_result);
        if let Ok(result) = l2.compose_sentence(&analysis) {
            for ev in result.events.iter().take(1) {
                println!("  Event: {}({})", ev.event.little_v, ev.event.predicate);
                for (role, entity) in &ev.event.participants {
                    println!("    {}: {}", role, entity.text);
                }
            }
        }
    }
    println!();

    Ok(())
}

fn convert_l1_to_l2(text: &str, l1: &SentenceAnalysisResult) -> SentenceAnalysis {
    let deps: Vec<DependencyArc> = l1
        .dependencies
        .iter()
        .map(|d| {
            let rel = match d.relation {
                L1Rel::NominalSubject => L2Rel::NominalSubject,
                L1Rel::Object => L2Rel::Object,
                L1Rel::IndirectObject => L2Rel::IndirectObject,
                L1Rel::Oblique => L2Rel::Oblique,
                L1Rel::AdverbialModifier => L2Rel::AdverbialModifier,
                L1Rel::AdjectivalModifier => L2Rel::AdjectivalModifier,
                L1Rel::Determiner => L2Rel::Determiner,
                L1Rel::Auxiliary => L2Rel::Auxiliary,
                L1Rel::Root => L2Rel::Root,
                L1Rel::Other => L2Rel::Other("other".into()),
            };
            DependencyArc::with_confidence(d.head_idx, d.dependent_idx, rel, d.confidence)
        })
        .collect();

    let meta = SentenceMetadata {
        is_passive: l1.metadata.is_passive,
        is_interrogative: l1.metadata.is_interrogative,
        is_negated: l1.metadata.is_negated,
        is_imperative: l1.metadata.is_imperative,
        ..Default::default()
    };

    SentenceAnalysis::new(text.to_string(), l1.tokens.clone())
        .with_dependencies(deps)
        .with_metadata(meta)
}

fn extract_sentences(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().skip(970).collect();
    let story = lines.join(" ");
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in story.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let s = current.trim().to_string();
            let words = s.split_whitespace().count();
            if (3..=40).contains(&words) && !s.starts_with("CHAPTER") {
                sentences.push(s);
            }
            current.clear();
        }
    }
    sentences
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
