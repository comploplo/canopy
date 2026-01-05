//! Demo of the Canopy semantic analysis pipeline.
//!
//! Run with: cargo run -p canopy-resources --example demo --release

use canopy::{
    ConfidenceDisambiguator, Disambiguator, GardenPathDetector, IncrementalProcessor,
    IncrementalState, MinSurprisalDisambiguator, Surprisal, UniformLanguageModel,
};
use canopy_resources::{CanopyPipeline, Tokenizer, UnicodeTokenizer};
use std::time::Instant;

/// Section 1: Demo Unicode tokenization (UAX #29)
fn demo_tokenization() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 1. UNICODE TOKENIZATION (UAX #29 Standard)                      │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let tokenizer = UnicodeTokenizer::from_ewt().unwrap_or_else(|_| UnicodeTokenizer::new());
    let examples = [
        ("Contractions", "I don't think she'll go."),
        ("Unicode text", "Café résumé naïve"),
        ("Mixed script", "Hello 世界! Привет мир!"),
        ("Emoji", "Great job! 🎉👏 Keep going 💪"),
    ];

    for (label, text) in examples {
        println!("  {label}:");
        println!("    Input:  \"{text}\"");
        let tokens = tokenizer.tokenize(text);
        print!("    Tokens: ");
        for token in &tokens {
            if token.is_split() {
                print!("[{}]* ", token.form);
            } else {
                print!("[{}] ", token.form);
            }
        }
        println!();
        if let Some(t) = tokens.iter().find(|t| !t.is_split()) {
            let slice = &text[t.byte_span.0..t.byte_span.1];
            println!(
                "    Span check: byte_span {:?} → \"{}\" ✓",
                t.byte_span, slice
            );
        }
        println!();
    }
}

/// Section 2: Initialize and time pipeline
fn init_pipeline() -> Result<CanopyPipeline, Box<dyn std::error::Error>> {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 2. SEMANTIC PIPELINE (VerbNet + FrameNet + WordNet + PropBank)  │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    println!("  Loading semantic resources...");
    let start = Instant::now();
    let pipeline = CanopyPipeline::new()?;
    println!("  ✓ Pipeline initialized in {:?}\n", start.elapsed());
    Ok(pipeline)
}

/// Section 3: Show semantic analysis examples
fn demo_analysis(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 3. SEMANTIC ANALYSIS                                            │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let examples = [
        ("Simple action", "John runs quickly."),
        ("Ditransitive", "Mary gave John a book."),
        ("Passive voice", "The window was broken by the ball."),
        (
            "Complex event",
            "The chef carefully prepared the elegant meal.",
        ),
    ];

    for (label, sentence) in examples {
        println!("  {label}: \"{sentence}\"");
        let start = Instant::now();
        let analysis = pipeline.analyze(sentence)?;
        let elapsed = start.elapsed();

        print!("    Syntax: ");
        for token in &analysis.syntax.tokens {
            print!("{}:{:?} ", token.form, token.upos);
        }
        println!();

        if !analysis.decompositions.is_empty() {
            println!("    Predicates:");
            for decomp in &analysis.decompositions {
                let roles: Vec<_> = decomp
                    .expected_roles
                    .iter()
                    .map(|r| format!("{r:?}"))
                    .collect();
                println!(
                    "      • {} → {:?} [{}]",
                    decomp.sense_id,
                    decomp.little_v_type,
                    roles.join(", ")
                );
            }
        }
        if !analysis.role_bindings.is_empty() {
            println!("    Role bindings:");
            for binding in &analysis.role_bindings {
                if let Some(token) = analysis.syntax.get_token(binding.token_id) {
                    println!(
                        "      • \"{}\" → {:?} ({:.0}% confidence)",
                        token.form,
                        binding.role,
                        binding.confidence * 100.0
                    );
                }
            }
        }
        if let Some(events) = &analysis.events {
            println!("    Events: {} composed", events.events.len());
        }
        println!("    Time: {elapsed:?}\n");
    }
    Ok(())
}

/// Section 4: Document analysis with discourse
fn demo_document(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 4. DOCUMENT ANALYSIS (Multi-Sentence + DRS)                     │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let document = "A man entered the room. He looked around nervously. The door slammed shut.";
    println!("  Document: \"{document}\"\n");

    let start = Instant::now();
    let doc = pipeline.analyze_document(document)?;
    let elapsed = start.elapsed();

    println!("  Sentences analyzed: {}", doc.sentence_count());
    for (i, sent) in doc.sentences.iter().enumerate() {
        println!(
            "    [{}] \"{}\" ({} tokens, {} events)",
            i + 1,
            sent.text,
            sent.syntax.tokens.len(),
            sent.event_count()
        );
    }

    if let Some(drs) = &doc.drs {
        println!("\n  Discourse Representation Structure (DRS):");
        println!("    Universe: {} discourse referents", drs.universe.len());
        println!("    Conditions: {} predicates", drs.conditions.len());
        if !drs.universe.is_empty() {
            print!("    Referents: ");
            for (id, referent) in drs.universe.iter().take(5) {
                print!("{id}:{:?} ", referent.referent_type);
            }
            if drs.universe.len() > 5 {
                print!("...");
            }
            println!();
        }
    }
    println!("\n  Total time: {elapsed:?}");
    Ok(())
}

/// Section 5: Underspecified semantic analysis
fn demo_underspecified(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 5. UNDERSPECIFIED ANALYSIS (Preserving Ambiguity)               │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let examples = [
        ("Lexical ambiguity", "The bank collapsed."),
        ("Scope ambiguity", "Every student read a book."),
        ("Pronoun ambiguity", "John told Bill he was tired."),
    ];

    for (label, sentence) in examples {
        println!("  {label}: \"{sentence}\"");
        let start = Instant::now();
        let underspec = pipeline.analyze_underspecified(sentence)?;
        let elapsed = start.elapsed();

        let summary = &underspec.ambiguity;
        println!("    Ambiguous: {}", underspec.is_ambiguous());
        println!("    Reading count: {}", underspec.reading_count());
        println!(
            "    Breakdown: lexical={}, structural={}, scope={}, referential={}",
            summary.lexical, summary.structural, summary.scope, summary.referential
        );

        if let Some(ref packed) = underspec.packed_events {
            println!(
                "    Packed events: {} choice points",
                packed.sense_choices.len()
            );
        }

        // Show resolved version
        let resolved = underspec.to_resolved();
        println!(
            "    Resolved: {} events, {} tokens",
            resolved.event_count(),
            resolved.syntax.tokens.len()
        );
        println!("    Time: {elapsed:?}\n");
    }

    Ok(())
}

/// Section 6: Surprisal trace demonstration
fn demo_surprisal() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 6. SURPRISAL TRACE (Information-Theoretic Processing)           │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let examples = [
        (
            "Normal sentence",
            vec!["The", "dog", "chased", "the", "cat"],
        ),
        (
            "Garden-path",
            vec!["The", "horse", "raced", "past", "the", "barn", "fell"],
        ),
    ];

    let lm = UniformLanguageModel::default();
    let processor = IncrementalProcessor::new();
    let detector = GardenPathDetector::with_threshold(10.0); // 10 bits threshold

    for (label, words) in examples {
        println!("  {label}: \"{}\"", words.join(" "));

        let mut state = IncrementalState::new();
        let mut surprisals = Vec::new();

        for (i, word) in words.iter().enumerate() {
            let token_id = canopy::TokenId::new(i);
            let surprisal = processor.process_word(&mut state, token_id, word, &lm);
            surprisals.push((word.to_string(), surprisal));
        }

        // Show surprisal trace
        print!("    Surprisal: ");
        for (word, surp) in &surprisals {
            print!("{}:{:.1}b ", word, surp.bits());
        }
        println!();

        // Check for garden-path
        let trace: Vec<Surprisal> = surprisals.iter().map(|(_, s)| *s).collect();
        if let Some(event) = detector.detect(&trace) {
            println!(
                "    Garden-path detected at word {} (surprisal: {:.1} bits)",
                event.word_index,
                event.surprisal.bits()
            );
        } else {
            println!("    No garden-path detected");
        }

        // Show entropy
        println!("    Final entropy: {:.2} bits", state.entropy());
        println!();
    }
}

/// Section 7: Disambiguator comparison
fn demo_disambiguators(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 7. DISAMBIGUATOR COMPARISON                                     │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let sentence = "The bank collapsed.";
    println!("  Sentence: \"{sentence}\"\n");

    // Get underspecified analysis first
    let underspec = pipeline.analyze_underspecified(sentence)?;
    println!("  Underspecified: {} readings\n", underspec.reading_count());

    // Compare disambiguators
    let disambiguators: Vec<(&str, Box<dyn Disambiguator>)> = vec![
        ("MinSurprisal", Box::new(MinSurprisalDisambiguator)),
        ("Confidence", Box::new(ConfidenceDisambiguator)),
    ];

    for (name, disambiguator) in &disambiguators {
        let start = Instant::now();
        let result = pipeline.analyze_with_disambiguator(sentence, disambiguator.as_ref())?;
        let elapsed = start.elapsed();

        println!("  {name} disambiguator:");
        println!("    Events: {}", result.event_count());
        if !result.decompositions.is_empty() {
            for decomp in &result.decompositions {
                println!(
                    "      Sense: {} ({:.0}%)",
                    decomp.sense_id,
                    decomp.confidence * 100.0
                );
            }
        }
        println!("    Time: {elapsed:?}\n");
    }

    Ok(())
}

/// Section 8: Moby Dick benchmark
fn demo_moby_dick(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 8. MOBY DICK BENCHMARK (Full Novel Analysis)                    │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let moby_path = std::path::Path::new("data/test-corpus/mobydick.txt");
    if !moby_path.exists() {
        println!("  [Skipped: data/test-corpus/mobydick.txt not found]");
        return Ok(());
    }

    let moby_text = std::fs::read_to_string(moby_path)?;
    println!("  Source: data/test-corpus/mobydick.txt");
    println!(
        "  Size: {} chars, {} lines\n",
        moby_text.len(),
        moby_text.lines().count()
    );

    let sentences: Vec<&str> = moby_text
        .lines()
        .filter(|line| {
            let t = line.trim();
            !t.is_empty()
                && t.len() > 20
                && !t
                    .chars()
                    .all(|c| c.is_uppercase() || c.is_whitespace() || c == '.')
        })
        .collect();

    println!("  Analyzing {} lines from Moby Dick...\n", sentences.len());
    let start = Instant::now();
    let (mut tokens, mut predicates, mut events) = (0, 0, 0);

    for sentence in &sentences {
        if let Ok(analysis) = pipeline.analyze(sentence) {
            tokens += analysis.syntax.tokens.len();
            predicates += analysis.decompositions.len();
            events += analysis.event_count();
        }
    }

    let elapsed = start.elapsed();
    let count = sentences.len();
    let per = elapsed / u32::try_from(count).unwrap_or(u32::MAX);

    println!("  Results:");
    println!("    Lines analyzed:    {count}");
    println!("    Total tokens:      {tokens}");
    println!("    Total predicates:  {predicates}");
    println!("    Total events:      {events}");
    println!("\n  Performance:");
    println!("    Total time:        {elapsed:?}");
    println!("    Per line:          {per:?}");
    println!(
        "    Throughput:        {:.0} lines/sec",
        f64::from(u32::try_from(count).unwrap_or(u32::MAX)) / elapsed.as_secs_f64()
    );
    println!(
        "    Token rate:        {:.0} tokens/sec",
        f64::from(u32::try_from(tokens).unwrap_or(u32::MAX)) / elapsed.as_secs_f64()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           CANOPY - Semantic Analysis Pipeline                    ║");
    println!("║     Linguistically-Grounded NLP with Real Semantic Resources     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    demo_tokenization();
    let pipeline = init_pipeline()?;
    demo_analysis(&pipeline)?;
    demo_document(&pipeline)?;
    demo_underspecified(&pipeline)?;
    demo_surprisal();
    demo_disambiguators(&pipeline)?;
    demo_moby_dick(&pipeline)?;

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                        Demo Complete                             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
