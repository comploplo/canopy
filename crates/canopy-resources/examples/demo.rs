//! Demo of the Canopy semantic analysis pipeline.
//!
//! Run with: cargo run -p canopy-resources --example demo --release

use canopy::{
    ConfidenceDisambiguator, Disambiguator, GardenPathDetector, IncrementalProcessor,
    IncrementalState, MinSurprisalDisambiguator, Surprisal, UniformSurprisalModel,
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

/// Section 3.5: Pattern matching demonstration
fn demo_pattern_matching(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 3.5 UD TREEBANK PATTERN MATCHING (VerbNet-Aware)                │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let examples = [
        "Mary gave John a book.",
        "The chef prepared the meal.",
        "John runs quickly.",
        "She broke the window.",
    ];

    println!("  Dependency Pattern Matching:");
    println!("  ────────────────────────────\n");

    for sentence in examples {
        println!("  \"{sentence}\"");
        let analysis = pipeline.analyze(sentence)?;

        // Get patterns for all verbs in the syntax
        let patterns = pipeline
            .syntax_provider()
            .get_patterns_for_syntax(&analysis.syntax);

        for token in &analysis.syntax.tokens {
            if let Some(pattern) = patterns.get(&token.id) {
                let vn_class = pattern.verbnet_class.as_deref().unwrap_or("(default SVO)");
                println!(
                    "    {} → VerbNet: {} ({:.0}% confidence)",
                    token.lemma,
                    vn_class,
                    pattern.confidence * 100.0
                );
                println!(
                    "      Expected args: {} (required: {})",
                    pattern.arguments.len(),
                    pattern.required_arguments().count()
                );

                // Show theta role hints
                let hints: Vec<_> = pattern
                    .arguments
                    .iter()
                    .filter_map(|arg| {
                        arg.role_hint
                            .as_ref()
                            .map(|r| format!("{:?}→{:?}", arg.dep_rel, r))
                    })
                    .collect();
                if !hints.is_empty() {
                    println!("      Role hints: {}", hints.join(", "));
                }
            }
        }
        println!();
    }

    // Show pattern matcher statistics
    if let Some(stats) = pipeline.syntax_provider().pattern_stats() {
        println!("  Pattern Matcher Statistics:");
        println!("  ───────────────────────────");
        println!(
            "    Cache hit rate:   {:.1}%",
            stats.cache_hit_rate() * 100.0
        );
        println!("    Cache hits:       {}", stats.cache_hits);
        println!("    Cache misses:     {}", stats.cache_misses);
        println!("    Lemma hits:       {}", stats.lemma_hits);
        println!("    VerbNet synth:    {}", stats.verbnet_synth);
        println!("    Default fallback: {}", stats.default_fallbacks);
    }

    println!();
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
        if let Some(relevance) = &sent.relevance {
            let question = relevance.question.as_deref().unwrap_or("(no active QUD)");
            println!("      Relevance: {:?} (QUD: {})", relevance.level, question);
        }
        if sent.validations.is_empty() {
            println!("      Validation: accepted");
        } else {
            for report in &sent.validations {
                if let Some(message) = &report.message {
                    println!("      Validation: {:?} ({})", report.status, message);
                } else {
                    println!("      Validation: {:?}", report.status);
                }
            }
        }
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
fn demo_qud_validation(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 5. QUD + VALIDATION DIALOGUE                                   │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    let dialogue = [
        ("Analyst", "Why did the pump fail?"),
        ("Engineer", "The spare filters arrived today."),
        (
            "Engineer",
            "The pump failed because the coolant overheated.",
        ),
        ("Engineer", "Actually, the pump did not fail."),
    ];

    let transcript = dialogue
        .iter()
        .map(|(_, utt)| *utt)
        .collect::<Vec<_>>()
        .join(" ");
    let doc = pipeline.analyze_document(&transcript)?;

    println!("  Dialogue diagnostics:\n");
    for ((speaker, utterance), sent) in dialogue.iter().zip(doc.sentences.iter()) {
        println!("    {speaker}: \"{utterance}\"");
        if let Some(relevance) = &sent.relevance {
            let qud = relevance.question.as_deref().unwrap_or("(no active QUD)");
            println!("      Relevance: {:?} (QUD: {qud})", relevance.level);
        } else {
            println!("      Relevance: (discourse disabled)");
        }

        if sent.validations.is_empty() {
            println!("      Validation: accepted");
        } else {
            for report in &sent.validations {
                if let Some(message) = &report.message {
                    println!("      Validation: {:?} ({})", report.status, message);
                } else {
                    println!("      Validation: {:?}", report.status);
                }
            }
        }
        println!();
    }

    Ok(())
}

/// Section 6: Discourse structure analysis
fn demo_discourse_structure(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    use canopy::Presupposition;

    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 6. DISCOURSE STRUCTURE (Moves, Coherence, Presuppositions)      │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // A narrative with clear discourse structure
    let text = "The old house stood on a hill. It had been empty for years. \
                Then one day a stranger arrived. He opened the creaky door carefully. \
                But the key didn't fit.";

    println!("  Narrative: \"{text}\"\n");

    let doc = pipeline.analyze_document(text)?;

    println!("  Discourse Move Analysis:");
    println!("  ─────────────────────────\n");

    for (i, sent) in doc.sentences.iter().enumerate() {
        print!("    [{}] \"{}\"", i + 1, sent.text);
        if let Some(ref dm) = sent.discourse_move {
            println!(" → {:?} ({:.0}% conf)", dm.move_type, dm.confidence * 100.0);
        } else {
            println!();
        }
    }

    println!("\n  Coherence Relations:");
    println!("  ─────────────────────\n");

    for (i, sent) in doc.sentences.iter().enumerate().skip(1) {
        if let Some(ref coh) = sent.coherence {
            println!(
                "    [{} → {}] {:?} ({:.0}% conf, signal: {:?})",
                i,
                i + 1,
                coh.relation,
                coh.confidence * 100.0,
                coh.primary_signal
            );
        }
    }

    println!("\n  Presuppositions Detected:");
    println!("  ──────────────────────────\n");

    let mut any_presuppositions = false;
    for (i, sent) in doc.sentences.iter().enumerate() {
        for presup in &sent.presuppositions {
            any_presuppositions = true;
            let desc = match &presup.presupposition {
                Presupposition::Existential { description, .. } => {
                    format!("Existential: \"{description}\"")
                }
                Presupposition::Factive { verb, proposition } => {
                    format!("Factive: {verb} → \"{proposition}\"")
                }
                Presupposition::Iterative { trigger, .. } => {
                    format!("Iterative: \"{trigger}\"")
                }
                Presupposition::Change {
                    verb, prior_state, ..
                } => {
                    format!("Change: {verb} (prior: {prior_state:?})")
                }
                Presupposition::Cleft { focus, .. } => {
                    format!("Cleft: \"{focus}\"")
                }
            };
            println!("    [{}] {} ({:?})", i + 1, desc, presup.status);
        }
    }
    if !any_presuppositions {
        println!("    (No explicit presupposition triggers detected)");
    }

    println!();
    Ok(())
}

/// Section 7: Underspecified semantic analysis
fn demo_underspecified(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 7. UNDERSPECIFIED ANALYSIS (Preserving Ambiguity)               │");
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

/// Section 8: Surprisal trace demonstration
fn demo_surprisal() {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 8. SURPRISAL TRACE (Information-Theoretic Processing)           │");
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

    let lm = UniformSurprisalModel::default();
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

/// Section 9: Disambiguator comparison
fn demo_disambiguators(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 9. DISAMBIGUATOR COMPARISON                                     │");
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

/// Section 10: Moby Dick benchmark
fn demo_moby_dick(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 10. MOBY DICK BENCHMARK (Full Novel Analysis)                   │");
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

/// Section 11: Logic Layer - Query Answering and Reasoning
fn demo_logic_layer(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    use canopy::kernel::logic::{ClosedWorldReasoner, Proposition, Query, Reasoner};
    use canopy::ThetaRole;

    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 11. LOGIC LAYER (Query Answering & Reasoning)                   │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    // Analyze a document to build DRS
    let text = "John gave Mary a book. She read it carefully.";
    println!("  Document: \"{text}\"\n");

    let doc = pipeline.analyze_document(text)?;

    if let Some(ref drs) = doc.drs {
        let reasoner = ClosedWorldReasoner::new();

        // 1. Consistency checking
        println!("  Consistency Check:");
        println!("  ──────────────────");
        let consistency = reasoner.check_consistent(drs);
        println!(
            "    Consistent: {} (conflicts: {})\n",
            consistency.consistent,
            consistency.conflicts.len()
        );

        // 2. Yes/No query - existence check
        println!("  Existence Queries:");
        println!("  ──────────────────");
        let exists_query = Query::exists("give");
        let result = reasoner.answer(drs, &exists_query);
        println!(
            "    \"Is there a giving event?\" → {}",
            if result.is_yes() { "Yes" } else { "No" }
        );

        let exists_query = Query::exists("run");
        let result = reasoner.answer(drs, &exists_query);
        println!(
            "    \"Is there a running event?\" → {}\n",
            if result.is_yes() { "Yes" } else { "No" }
        );

        // 3. Wh-question
        println!("  Wh-Question (Who gave?):");
        println!("  ─────────────────────────");
        let wh_query = Query::wh("give", ThetaRole::Agent);
        let result = reasoner.answer(drs, &wh_query);
        println!("    Answers: {} found", result.answers.len());
        for answer in &result.answers {
            for (var, binding) in &answer.bindings {
                println!("      {var} = \"{}\"", binding.text);
            }
        }
        println!();

        // 4. Entailment checking
        println!("  Entailment Check:");
        println!("  ──────────────────");
        let prop = Proposition::simple("give", ThetaRole::Agent, "John");
        let entailment = reasoner.entails(drs, &prop);
        println!(
            "    \"John is the agent of a giving\" → {:?}",
            entailment.entailed
        );
        if let Some(ref explanation) = entailment.explanation {
            println!("    Explanation: {}", explanation.summary);
        }
        println!();

        // 5. What-happened query
        println!("  What-Happened Query:");
        println!("  ─────────────────────");
        let what_query = Query::what_happened(Some("John".to_string()));
        let result = reasoner.answer(drs, &what_query);
        println!(
            "    \"What did John do?\" → {} event(s)",
            result.answers.len()
        );
        for answer in &result.answers {
            if let Some(binding) = answer.bindings.get("?event") {
                println!("      • {}", binding.text);
            }
        }
    } else {
        println!("  [DRS not available - discourse analysis may be disabled]");
    }

    println!();
    Ok(())
}

/// Section 12: Derivation Trace - Explainable Semantics
fn demo_derivation_trace(pipeline: &CanopyPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 12. DERIVATION TRACE (Explainable Semantics)                    │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    println!("  Single sentence with trace:");
    println!("  ─────────────────────────────\n");

    let (_, trace) =
        pipeline.analyze_with_trace("The chef carefully prepared the elegant meal.")?;
    println!("{}", trace.to_text());

    println!("\n  Document analysis with discourse trace:");
    println!("  ────────────────────────────────────────\n");

    let (_, doc_trace) = pipeline
        .analyze_document_with_trace("A man entered the room. He looked around nervously.")?;
    println!("{}", doc_trace.to_text());

    Ok(())
}

/// Section 13: TAM - Tense, Aspect, Modality and Temporal Reasoning
fn demo_tam() {
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│ 13. TAM - Tense, Aspect, Modality & Temporal Reasoning          │");
    println!("└─────────────────────────────────────────────────────────────────┘\n");

    demo_tam_temporal_frames();
    demo_tam_allen_algebra();
    demo_tam_modal_reasoning();
    demo_tam_counterfactuals();
    println!();
}

fn demo_tam_temporal_frames() {
    use canopy::kernel::discourse::AspectualViewpoint;

    println!("  Reichenbachian Temporal Frames:");
    println!("  ────────────────────────────────\n");

    let examples = [
        (
            "John ran.",
            "Past",
            "E < R = S",
            AspectualViewpoint::Perfective,
        ),
        (
            "John was running.",
            "Past Progressive",
            "E ○ R < S",
            AspectualViewpoint::Progressive,
        ),
        (
            "John has left.",
            "Present Perfect",
            "E < R = S",
            AspectualViewpoint::Perfect,
        ),
    ];

    for (sentence, tense_label, formula, viewpoint) in &examples {
        println!("    \"{sentence}\"");
        println!("      Tense: {tense_label}");
        println!("      Temporal: {formula}");
        println!("      Aspect: {viewpoint:?}\n");
    }
}

fn demo_tam_allen_algebra() {
    use canopy::kernel::logic::{AllenRelation, TemporalConstraint, TemporalReasoner};
    use canopy::ReferentId;

    println!("  Temporal Reasoning (Allen Interval Algebra):");
    println!("  ──────────────────────────────────────────────\n");

    let mut temporal = TemporalReasoner::new();
    let e1 = ReferentId::new(0);
    let e2 = ReferentId::new(1);
    let e3 = ReferentId::new(2);

    temporal.add_constraint(TemporalConstraint::new(
        e1,
        e2,
        AllenRelation::Before,
        "narration",
    ));
    temporal.add_constraint(TemporalConstraint::new(
        e2,
        e3,
        AllenRelation::Before,
        "narration",
    ));

    println!("    Narrative: \"John entered. Mary stood up. They shook hands.\"");
    println!("    Constraints:");
    println!("      e1(enter) < e2(stand-up)  [narration]");
    println!("      e2(stand-up) < e3(shake)  [narration]");

    let result = temporal.check_consistency();
    println!("\n    Consistency: {}", result.is_consistent);
    println!("    Inferred: {} new constraints", result.inferred.len());
    for inf in &result.inferred {
        println!(
            "      {:?} {:?} {:?} [{}]",
            inf.from, inf.relation, inf.to, inf.source
        );
    }
}

fn demo_tam_modal_reasoning() {
    use canopy::core::{ModalFlavor, ModalForce};
    use canopy::kernel::discourse::{AccessibilityType, WorldId};
    use canopy::kernel::logic::ModalReasoner;

    println!("\n  Modal Reasoning (Kripke Semantics):");
    println!("  ─────────────────────────────────────\n");

    let mut modal = ModalReasoner::new();
    let w0 = WorldId::ACTUAL;
    let w1 = modal.create_world();
    modal.make_accessible(w0, w1, AccessibilityType::Epistemic);
    modal.get_world_mut(&w1).unwrap().add_fact("raining");

    println!("    World Model:");
    println!("      w0 (actual): {{}}");
    println!("      w1 (epistemic): {{raining}}");
    println!("      w0 --[epistemic]--> w1");

    let eval =
        modal.evaluate_modal_fact(ModalForce::Possibility, ModalFlavor::Epistemic, "raining");
    println!("\n    \"It might be raining\" (◇_epistemic raining)");
    println!("      Holds: {}", eval.holds);
    println!("      Witness worlds: {:?}", eval.witness_worlds);
}

fn demo_tam_counterfactuals() {
    use canopy::kernel::discourse::{AccessibilityType, WorldId};
    use canopy::kernel::logic::{CounterfactualModal, ModalReasoner};

    println!("\n  Counterfactual Reasoning (Lewis Semantics):");
    println!("  ─────────────────────────────────────────────\n");

    let mut modal = ModalReasoner::new();
    let actual = WorldId::ACTUAL;

    modal
        .get_world_mut(&actual)
        .unwrap()
        .add_fact("john_stayed");
    modal.get_world_mut(&actual).unwrap().add_fact("mary_happy");

    let cf_world = modal.create_world();
    modal.make_accessible(actual, cf_world, AccessibilityType::Similarity);
    modal
        .get_world_mut(&cf_world)
        .unwrap()
        .add_fact("john_left");
    modal.get_world_mut(&cf_world).unwrap().add_fact("mary_sad");

    println!("    \"If John had left, Mary would be sad\"");
    println!("    Actual: {{john_stayed, mary_happy}}");
    println!("    Closest where antecedent holds: {{john_left, mary_sad}}\n");

    let cf_eval =
        modal.evaluate_counterfactual_facts("john_left", "mary_sad", CounterfactualModal::Would);

    println!("    Antecedent: john_left");
    println!("    Consequent: mary_sad");
    println!("    Result: {} ({})", cf_eval.holds, cf_eval.explanation);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║           CANOPY - Semantic Analysis Pipeline                    ║");
    println!("║     Linguistically-Grounded NLP with Real Semantic Resources     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    demo_tokenization();
    let pipeline = init_pipeline()?;
    demo_analysis(&pipeline)?;
    demo_pattern_matching(&pipeline)?;
    demo_document(&pipeline)?;
    demo_qud_validation(&pipeline)?;
    demo_discourse_structure(&pipeline)?;
    demo_underspecified(&pipeline)?;
    demo_surprisal();
    demo_disambiguators(&pipeline)?;
    demo_moby_dick(&pipeline)?;
    demo_logic_layer(&pipeline)?;
    demo_derivation_trace(&pipeline)?;
    demo_tam();

    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                        Demo Complete                             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
