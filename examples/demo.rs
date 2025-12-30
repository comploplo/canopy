//! Canopy Demo - Three-Layer Semantic Analysis Pipeline
//!
//! Shows how text flows through the semantic analysis layers:
//! - Layer 1: Tokenization, POS, dependencies, semantic features
//! - Layer 2: Event composition with participants and modality
//! - Layer 3: Discourse representation with entity tracking
//!
//! Run: cargo run --example demo --release

use canopy_events::{DependencyArc, EventComposer, SentenceAnalysis, SentenceMetadata};
use canopy_pipeline::{create_l1_analyzer_with_treebank, DiscourseProcessor};
use canopy_tokenizer::{DependencyRelation as L1Rel, SentenceAnalysisResult};
use canopy_treebank::types::DependencyRelation as L2Rel;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!();
    println!("CANOPY - Semantic Analysis Pipeline");
    println!("====================================");
    println!();

    // Initialize engines
    print!("Loading semantic engines... ");
    std::io::Write::flush(&mut std::io::stdout())?;
    let start = Instant::now();
    let l1 = create_l1_analyzer_with_treebank()?;
    let l2 = EventComposer::new()?;
    let mut l3 = DiscourseProcessor::new();
    println!("done ({:.0?})\n", start.elapsed());

    // =========================================================================
    // PART 1: Basic Analysis
    // =========================================================================
    println!("PART 1: Basic Semantic Analysis");
    println!("--------------------------------\n");

    let basic_sentences = [
        ("Simple event", "The captain saw the whale."),
        ("Modal verb", "Ahab must find Moby Dick."),
        ("Negation", "The crew did not believe him."),
        ("Communication", "Ishmael called him a madman."),
    ];

    for (label, sentence) in basic_sentences {
        analyze_sentence(&l1, &l2, &mut l3, label, sentence)?;
    }

    print_discourse_state(&l3, "After basic sentences");

    // =========================================================================
    // PART 2: Advanced Features
    // =========================================================================
    println!("\nPART 2: Advanced Semantic Features");
    println!("-----------------------------------\n");

    // Reset discourse for fresh context
    l3 = DiscourseProcessor::new();

    let advanced_sentences = [
        ("Passive voice", "The whale was spotted by the lookout."),
        ("Possibility modal", "The ship might sink in the storm."),
        ("Necessity modal", "Every sailor must follow orders."),
        ("Factive verb", "The captain knew that danger awaited."),
        ("Causative", "The storm made the sailors afraid."),
    ];

    for (label, sentence) in advanced_sentences {
        analyze_sentence(&l1, &l2, &mut l3, label, sentence)?;
    }

    print_discourse_state(&l3, "After advanced sentences");

    // =========================================================================
    // PART 3: Multi-Sentence Discourse
    // =========================================================================
    println!("\nPART 3: Multi-Sentence Discourse");
    println!("---------------------------------\n");

    // Reset for narrative
    l3 = DiscourseProcessor::new();

    let narrative = [
        "Captain Ahab stood on the deck.",
        "He stared at the horizon.",
        "The white whale had escaped him before.",
        "This time would be different.",
    ];

    println!("Narrative:");
    for (i, sentence) in narrative.iter().enumerate() {
        println!("  {}. {}", i + 1, sentence);
    }
    println!();

    for sentence in narrative {
        let l1_result = l1.analyze_sentence(sentence)?;
        let analysis = convert_l1_to_l2(sentence, &l1_result);
        if let Ok(events) = l2.compose_sentence(&analysis) {
            let _ = l3.process_sentence(sentence, &events);
        }
    }

    print_full_discourse(&l3);

    // =========================================================================
    // PART 4: Performance (Moby Dick corpus)
    // =========================================================================
    println!("\nPART 4: Performance (Moby Dick)");
    println!("-------------------------------\n");

    // Load sentences from Moby Dick
    let moby_sentences = load_moby_dick_sentences()?;
    println!("Loaded {} sentences from Moby Dick\n", moby_sentences.len());

    // Show 5 example sentences with full analysis
    println!("Example sentences:");
    println!();
    l3 = DiscourseProcessor::new();
    for (i, sentence) in moby_sentences.iter().take(5).enumerate() {
        analyze_sentence(&l1, &l2, &mut l3, &format!("Example {}", i + 1), sentence)?;
    }

    // Performance test on 100 sentences
    println!("Performance benchmark (100 sentences):");
    let test_sentences: Vec<_> = moby_sentences.iter().take(100).collect();

    let perf_start = Instant::now();
    let mut total_events = 0;
    let mut total_entities = 0;
    for sentence in &test_sentences {
        let l1_result = l1.analyze_sentence(sentence)?;
        let analysis = convert_l1_to_l2(sentence, &l1_result);
        if let Ok(events) = l2.compose_sentence(&analysis) {
            total_events += events.events.len();
            for event in &events.events {
                total_entities += event.event.participants.len();
            }
        }
    }
    let perf_elapsed = perf_start.elapsed();

    println!(
        "  Analyzed {} sentences in {:?}",
        test_sentences.len(),
        perf_elapsed
    );
    println!(
        "  Average: {:.2}ms per sentence",
        perf_elapsed.as_secs_f64() * 1000.0 / test_sentences.len() as f64
    );
    println!("  Events extracted: {}", total_events);
    println!("  Participants found: {}", total_entities);
    println!();

    Ok(())
}

fn analyze_sentence(
    l1: &canopy_tokenizer::SemanticCoordinator,
    l2: &EventComposer,
    l3: &mut DiscourseProcessor,
    label: &str,
    sentence: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("[{}]", label);
    println!("\"{}\"", sentence);
    println!();

    // Layer 1
    let l1_result = match l1.analyze_sentence(sentence) {
        Ok(r) => r,
        Err(e) => {
            println!("  Error: {}\n", e);
            return Ok(());
        }
    };

    print!("  L1: ");
    print_tokens(&l1_result);

    if !l1_result.dependencies.is_empty() {
        print!("      ");
        print_deps(&l1_result);
    }

    print_sentence_features(&l1_result);
    print_verbnet(&l1_result);

    // Layer 2
    let analysis = convert_l1_to_l2(sentence, &l1_result);
    let l2_result = match l2.compose_sentence(&analysis) {
        Ok(r) => r,
        Err(e) => {
            println!("  L2: Error - {}\n", e);
            return Ok(());
        }
    };

    print!("  L2: ");
    print_events(&l2_result);

    // Layer 3
    let referents = l3.process_sentence(sentence, &l2_result)?;
    print!("  L3: ");
    print_referents(&referents, l3);

    println!();
    Ok(())
}

fn print_tokens(result: &SentenceAnalysisResult) {
    let tokens: Vec<String> = result
        .tokens
        .iter()
        .filter(|t| !t.original_word.chars().all(|c| c.is_ascii_punctuation()))
        .map(|t| format!("{}/{}", t.original_word, pos_str(t)))
        .collect();
    println!("{}", tokens.join("  "));
}

fn print_deps(result: &SentenceAnalysisResult) {
    let deps: Vec<String> = result
        .dependencies
        .iter()
        .filter(|d| {
            matches!(
                d.relation,
                L1Rel::NominalSubject | L1Rel::Object | L1Rel::IndirectObject
            )
        })
        .filter_map(|d| {
            let head = result.tokens.get(d.head_idx)?;
            let dep = result.tokens.get(d.dependent_idx)?;
            Some(format!(
                "{}--{:?}-->{}",
                dep.original_word, d.relation, head.original_word
            ))
        })
        .take(3)
        .collect();
    if !deps.is_empty() {
        println!("{}", deps.join("  "));
    }
}

fn print_sentence_features(result: &SentenceAnalysisResult) {
    let mut features = Vec::new();
    if result.metadata.is_passive {
        features.push("PASSIVE");
    }
    if result.metadata.is_negated {
        features.push("NEGATED");
    }
    if result.metadata.is_interrogative {
        features.push("QUESTION");
    }
    if result.metadata.is_imperative {
        features.push("IMPERATIVE");
    }
    if !features.is_empty() {
        println!("      [{}]", features.join(", "));
    }
}

fn print_verbnet(result: &SentenceAnalysisResult) {
    for token in &result.tokens {
        if token.pos == Some(canopy_core::UPos::Verb) {
            if let Some(ref vn) = token.verbnet {
                if !vn.verb_classes.is_empty() {
                    println!(
                        "      VerbNet: {} -> {}",
                        token.lemma, vn.verb_classes[0].id
                    );
                    return;
                }
            }
        }
    }
}

fn print_events(result: &canopy_events::ComposedEvents) {
    if result.events.is_empty() {
        println!("(no events)");
        return;
    }

    let event = &result.events[0];
    let polarity = if event.polarity { "" } else { " [NEG]" };

    // Event type
    print!(
        "{} : {}{}",
        event.event.little_v, event.event.predicate, polarity
    );

    // Modality on same line if present
    if let Some(ref modality) = event.event.modality {
        print!(" [{:?}]", modality.force);
    }
    println!();

    // Participants
    for (role, entity) in &event.event.participants {
        println!("      {} = \"{}\"", role, entity.text);
    }

    // Presuppositions
    if !event.presuppositions.is_empty() {
        let triggers: Vec<String> = event
            .presuppositions
            .iter()
            .map(|p| format!("{:?}", p.trigger_type))
            .collect();
        println!("      presupposes: {}", triggers.join(", "));
    }
}

fn print_referents(referents: &[canopy_discourse::ReferentId], processor: &DiscourseProcessor) {
    let drs = processor.drs();
    let entity_count = drs.universe.iter().filter(|(_, r)| !r.is_event).count();

    if !referents.is_empty() {
        let refs: Vec<String> = referents.iter().map(|r| format!("e{}", r.0)).collect();
        println!(
            "new events: {}  |  total entities: {}",
            refs.join(", "),
            entity_count
        );
    } else {
        println!("total entities: {}", entity_count);
    }
}

fn print_discourse_state(processor: &DiscourseProcessor, label: &str) {
    let drs = processor.drs();
    let entities: Vec<String> = drs
        .universe
        .iter()
        .filter(|(_, r)| !r.is_event)
        .take(8)
        .map(|(id, r)| {
            let name = r.name.as_deref().unwrap_or("?");
            format!("x{}:{}", id.0, name)
        })
        .collect();

    println!("--- {} ---", label);
    println!("Discourse entities: {}", entities.join(", "));
    if drs.universe.iter().filter(|(_, r)| !r.is_event).count() > 8 {
        println!("  (and more...)");
    }
    println!();
}

fn print_full_discourse(processor: &DiscourseProcessor) {
    let drs = processor.drs();

    println!("Discourse Representation Structure (DRS):");
    println!();

    // Entities
    println!("  Universe (Entities):");
    for (id, referent) in drs.universe.iter().filter(|(_, r)| !r.is_event) {
        let name = referent.name.as_deref().unwrap_or("?");
        let props: Vec<&str> = referent.properties.keys().map(|s| s.as_str()).collect();
        if props.is_empty() {
            println!("    x{}: {}", id.0, name);
        } else {
            println!("    x{}: {} [{}]", id.0, name, props.join(", "));
        }
    }

    // Events
    println!();
    println!("  Universe (Events):");
    for (id, referent) in drs.universe.iter().filter(|(_, r)| r.is_event) {
        let pred = referent.name.as_deref().unwrap_or("?");
        println!("    e{}: {}", id.0, pred);
    }

    // Conditions
    if !drs.conditions.is_empty() {
        println!();
        println!("  Conditions:");
        for (i, condition) in drs.conditions.iter().take(5).enumerate() {
            println!("    {}: {:?}", i + 1, condition);
        }
        if drs.conditions.len() > 5 {
            println!("    ... ({} more)", drs.conditions.len() - 5);
        }
    }

    println!();
}

fn pos_str(token: &canopy_tokenizer::Layer1SemanticResult) -> &'static str {
    token
        .pos
        .map(|p| match p {
            canopy_core::UPos::Noun => "N",
            canopy_core::UPos::Verb => "V",
            canopy_core::UPos::Adj => "Adj",
            canopy_core::UPos::Adv => "Adv",
            canopy_core::UPos::Propn => "NNP",
            canopy_core::UPos::Pron => "Pro",
            canopy_core::UPos::Det => "Det",
            canopy_core::UPos::Adp => "P",
            canopy_core::UPos::Aux => "Aux",
            canopy_core::UPos::Cconj => "CC",
            canopy_core::UPos::Sconj => "SC",
            _ => "?",
        })
        .unwrap_or("?")
}

/// Load sentences from Moby Dick corpus, filtering for good analysis candidates
fn load_moby_dick_sentences() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::fs;

    let path = "data/test-corpus/mobydick.txt";
    let content = fs::read_to_string(path)?;

    // Skip header/preamble (starts at "CALL me Ishmael")
    let text_start = content.find("CALL me Ishmael").unwrap_or(0);
    let text = &content[text_start..];

    // Split into sentences (simple heuristic: split on . ! ?)
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let sentence = current.trim().to_string();
            // Filter for good sentences:
            // - 5-15 words (good for demo)
            // - Contains a verb-like word
            // - No weird characters
            let word_count = sentence.split_whitespace().count();
            if (5..=15).contains(&word_count)
                && sentence.chars().all(|c| c.is_ascii() || c == '\'')
                && !sentence.contains('*')
                && !sentence.contains('[')
            {
                sentences.push(sentence);
            }
            current.clear();
        }
    }

    // Shuffle deterministically for reproducible results
    // Use simple selection based on position
    let mut selected: Vec<String> = sentences
        .into_iter()
        .enumerate()
        .filter(|(i, _)| i % 3 == 0) // Take every 3rd sentence for variety
        .map(|(_, s)| s)
        .take(200) // Keep pool of 200
        .collect();

    // Return first 100 for analysis
    selected.truncate(100);
    Ok(selected)
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
