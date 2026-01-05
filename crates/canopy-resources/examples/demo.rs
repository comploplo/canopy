//! Demo of the Canopy semantic analysis pipeline.
//!
//! Run with: cargo run -p canopy-resources --example demo --release

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

/// Section 5: Moby Dick benchmark
fn demo_moby_dick(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 5. MOBY DICK BENCHMARK (Full Novel Analysis)                    │");
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
    demo_moby_dick(&pipeline)?;

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                        Demo Complete                             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
