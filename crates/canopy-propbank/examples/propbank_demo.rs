//! PropBank semantic role labeling demo
//!
//! This example demonstrates the PropBank engine's capabilities including:
//! - Predicate-argument structure analysis
//! - Semantic role classification (ARG0-5, ARGM-*)
//! - Theta role mapping for cross-engine compatibility
//! - Fuzzy matching and confidence scoring

use canopy_engine::{CachedEngine, SemanticEngine, StatisticsProvider};
use canopy_propbank::{PropBankConfig, PropBankEngine};
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

fn create_demo_propbank_data() -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir
        .path()
        .join("propbank-release")
        .join("data")
        .join("google")
        .join("ewt");
    fs::create_dir_all(&data_dir)?;

    // Create comprehensive PropBank framesets
    let prop_content = r#"# PropBank Demo Data - Representative English Predicates

# Basic transitive verbs
give.01: ARG0:giver ARG1:thing_given ARG2:recipient ARGM-LOC:location ARGM-TMP:time
give.02: ARG0:giver ARG1:beneficiary ARG2:gift ARGM-MNR:manner
take.01: ARG0:taker ARG1:thing_taken ARG2:taken_from ARGM-LOC:location
take.02: ARG0:agent ARG1:theme ARG2:duration ARGM-MNR:manner
bring.01: ARG0:bringer ARG1:thing_brought ARG2:destination ARGM-DIR:direction
carry.01: ARG0:carrier ARG1:thing_carried ARG2:destination ARGM-MNR:manner

# Motion verbs
run.01: ARG0:runner ARGM-LOC:location ARGM-MNR:manner ARGM-DIR:direction
run.02: ARG0:agent ARG1:path ARGM-DIR:direction ARGM-TMP:time
walk.01: ARG0:walker ARG1:path ARGM-LOC:location ARGM-MNR:manner
move.01: ARG0:mover ARG1:thing_moved ARG2:destination ARGM-DIR:direction
travel.01: ARG0:traveler ARG1:path ARG2:destination ARGM-MNR:manner

# Cognitive verbs
think.01: ARG0:thinker ARG1:thought ARGM-CAU:cause ARGM-TMP:time
believe.01: ARG0:believer ARG1:belief ARGM-CAU:reason
know.01: ARG0:knower ARG1:thing_known ARGM-TMP:time
understand.01: ARG0:understander ARG1:thing_understood ARGM-MNR:manner

# Communication verbs
say.01: ARG0:speaker ARG1:utterance ARG2:hearer ARGM-TMP:time ARGM-LOC:location
tell.01: ARG0:teller ARG1:thing_told ARG2:listener ARGM-MNR:manner
speak.01: ARG0:speaker ARG1:topic ARG2:listener ARGM-LOC:location
explain.01: ARG0:explainer ARG1:thing_explained ARG2:listener ARGM-MNR:manner

# Creation verbs
make.01: ARG0:maker ARG1:thing_made ARG2:material ARGM-LOC:location ARGM-TMP:time
build.01: ARG0:builder ARG1:thing_built ARG2:material ARGM-LOC:location
create.01: ARG0:creator ARG1:thing_created ARG2:source ARGM-MNR:manner
write.01: ARG0:writer ARG1:thing_written ARG2:beneficiary ARGM-TMP:time

# Consumption/interaction verbs
eat.01: ARG0:eater ARG1:food ARGM-LOC:location ARGM-TMP:time ARGM-MNR:manner
drink.01: ARG0:drinker ARG1:beverage ARGM-LOC:location ARGM-TMP:time
read.01: ARG0:reader ARG1:thing_read ARGM-LOC:location ARGM-TMP:time
watch.01: ARG0:watcher ARG1:thing_watched ARGM-LOC:location ARGM-TMP:time

# Stative verbs
be.01: ARG0:theme ARG1:attribute ARGM-LOC:location ARGM-TMP:time
have.01: ARG0:haver ARG1:thing_had ARGM-TMP:time
exist.01: ARG0:thing_existing ARGM-LOC:location ARGM-TMP:time
sleep.01: ARG0:sleeper ARGM-TMP:duration ARGM-LOC:location

# Change of state verbs
break.01: ARG0:breaker ARG1:thing_broken ARGM-LOC:location ARGM-MNR:manner
fix.01: ARG0:fixer ARG1:thing_fixed ARGM-LOC:location ARGM-TMP:time
open.01: ARG0:opener ARG1:thing_opened ARGM-MNR:manner
close.01: ARG0:closer ARG1:thing_closed ARGM-MNR:manner

# Emotional verbs
love.01: ARG0:lover ARG1:thing_loved ARGM-CAU:reason
like.01: ARG0:liker ARG1:thing_liked ARGM-CAU:reason
hate.01: ARG0:hater ARG1:thing_hated ARGM-CAU:reason
fear.01: ARG0:fearer ARG1:thing_feared ARGM-CAU:reason

# Complex predicates with many arguments
teach.01: ARG0:teacher ARG1:student ARG2:subject ARGM-LOC:location ARGM-TMP:time ARGM-MNR:manner
sell.01: ARG0:seller ARG1:thing_sold ARG2:buyer ARG3:price ARGM-LOC:location ARGM-TMP:time
buy.01: ARG0:buyer ARG1:thing_bought ARG2:seller ARG3:price ARGM-LOC:location ARGM-TMP:time
send.01: ARG0:sender ARG1:thing_sent ARG2:recipient ARG3:instrument ARGM-DIR:direction ARGM-TMP:time"#;

    fs::write(data_dir.join("demo_framesets.prop"), prop_content)?;

    // Create a CoNLL-U format .gold_skel file with PropBank annotations
    let gold_skel_content = r#"# sent_id = demo_1
# text = John gave Mary a beautiful book yesterday at the library.
1	John	John	PROPN	NNP	Number=Sing	2	nsubj	2:ARG0	SpacesAfter=\n
2	gave	give	VERB	VBD	Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin	0	root	0:pred	pred=give.01
3	Mary	Mary	PROPN	NNP	Number=Sing	2	iobj	2:ARG2	SpacesAfter=\n
4	a	a	DET	DT	Definite=Ind|PronType=Art	6	det	_	SpacesAfter=\n
5	beautiful	beautiful	ADJ	JJ	Degree=Pos	6	amod	_	SpacesAfter=\n
6	book	book	NOUN	NN	Number=Sing	2	obj	2:ARG1	SpacesAfter=\n
7	yesterday	yesterday	ADV	RB	_	2	advmod	2:ARGM-TMP	SpacesAfter=\n
8	at	at	ADP	IN	_	10	case	_	SpacesAfter=\n
9	the	the	DET	DT	Definite=Def|PronType=Art	10	det	_	SpacesAfter=\n
10	library	library	NOUN	NN	Number=Sing	2	obl	2:ARGM-LOC	SpaceAfter=No
11	.	.	PUNCT	.	_	2	punct	_	SpacesAfter=\n

# sent_id = demo_2
# text = She runs quickly in the park every morning.
1	She	she	PRON	PRP	Case=Nom|Gender=Fem|Number=Sing|Person=3|PronType=Prs	2	nsubj	2:ARG0	SpacesAfter=\n
2	runs	run	VERB	VBZ	Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin	0	root	0:pred	pred=run.01
3	quickly	quickly	ADV	RB	_	2	advmod	2:ARGM-MNR	SpacesAfter=\n
4	in	in	ADP	IN	_	6	case	_	SpacesAfter=\n
5	the	the	DET	DT	Definite=Def|PronType=Art	6	det	_	SpacesAfter=\n
6	park	park	NOUN	NN	Number=Sing	2	obl	2:ARGM-LOC	SpacesAfter=\n
7	every	every	DET	DT	_	8	det	_	SpacesAfter=\n
8	morning	morning	NOUN	NN	Number=Sing	2	obl:tmod	2:ARGM-TMP	SpaceAfter=No
9	.	.	PUNCT	.	_	2	punct	_	SpacesAfter=\n

# sent_id = demo_3
# text = The teacher explained the concept to students clearly.
1	The	the	DET	DT	Definite=Def|PronType=Art	2	det	_	SpacesAfter=\n
2	teacher	teacher	NOUN	NN	Number=Sing	3	nsubj	3:ARG0	SpacesAfter=\n
3	explained	explain	VERB	VBD	Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin	0	root	0:pred	pred=explain.01
4	the	the	DET	DT	Definite=Def|PronType=Art	5	det	_	SpacesAfter=\n
5	concept	concept	NOUN	NN	Number=Sing	3	obj	3:ARG1	SpacesAfter=\n
6	to	to	ADP	IN	_	7	case	_	SpacesAfter=\n
7	students	student	NOUN	NNS	Number=Plur	3	obl	3:ARG2	SpacesAfter=\n
8	clearly	clearly	ADV	RB	_	3	advmod	3:ARGM-MNR	SpaceAfter=No
9	.	.	PUNCT	.	_	3	punct	_	SpacesAfter=\n

"#;

    fs::write(
        data_dir.join("demo_annotations.gold_skel"),
        gold_skel_content,
    )?;
    Ok(temp_dir)
}

fn print_predicate_analysis(engine: &PropBankEngine, lemma: &str, sense: &str) {
    println!("\n🔍 Analyzing predicate: {lemma}.{sense}");
    println!("{}", "─".repeat(50));

    match engine.analyze_predicate(lemma, sense) {
        Ok(result) => {
            let predicate = &result.data;
            println!("📋 Roleset: {}", predicate.roleset);
            println!("💯 Confidence: {:.2}", result.confidence);
            println!("⏱️  Processing time: {}μs", result.processing_time_us);
            println!("🎯 Arguments ({}):", predicate.arguments.len());

            // Group arguments by type
            let core_args = predicate.get_core_arguments();
            let modifiers = predicate.get_modifiers();

            if !core_args.is_empty() {
                println!("  📌 Core Arguments:");
                for arg in core_args {
                    let theta_role = arg
                        .role
                        .to_theta_role()
                        .map(|r| format!(" → {r:?}"))
                        .unwrap_or_default();
                    println!(
                        "    • {}: {}{}",
                        arg.role.to_propbank_label(),
                        arg.description,
                        theta_role
                    );
                }
            }

            if !modifiers.is_empty() {
                println!("  🏷️  Modifiers:");
                for arg in modifiers {
                    let theta_role = arg
                        .role
                        .to_theta_role()
                        .map(|r| format!(" → {r:?}"))
                        .unwrap_or_default();
                    println!(
                        "    • {}: {}{}",
                        arg.role.to_propbank_label(),
                        arg.description,
                        theta_role
                    );
                }
            }

            // Show theta roles for cross-engine compatibility
            if let Ok(theta_roles) = engine.get_theta_roles(lemma, sense) {
                if !theta_roles.is_empty() {
                    println!("  🔗 Theta Roles: {theta_roles:?}");
                }
            }
        }
        Err(e) => {
            println!("❌ Error: {e:?}");
        }
    }
}

fn print_word_analysis(engine: &PropBankEngine, word: &str) {
    println!("\n🔎 Analyzing word: '{word}'");
    println!("{}", "─".repeat(50));

    match engine.analyze_word(word) {
        Ok(result) => {
            let analysis = &result.data;
            println!("📊 Analysis confidence: {:.2}", analysis.confidence);
            println!("⏱️  Processing time: {}μs", result.processing_time_us);
            println!("🎯 Arguments found: {}", analysis.argument_count);

            if let Some(ref predicate) = analysis.predicate {
                println!(
                    "🏆 Primary predicate: {} (sense: {})",
                    predicate.lemma, predicate.sense
                );
                println!("  💬 Description: {}", predicate.definition);
                if !predicate.arguments.is_empty() {
                    println!("  📋 Role structure:");
                    for arg in &predicate.arguments {
                        println!(
                            "    • {}: {}",
                            arg.role.to_propbank_label(),
                            arg.description
                        );
                    }
                }
            }

            if !analysis.alternative_rolesets.is_empty() {
                println!(
                    "🔄 Alternative senses ({}):",
                    analysis.alternative_rolesets.len()
                );
                for alt in &analysis.alternative_rolesets {
                    println!(
                        "  • {}.{}: {} args",
                        alt.lemma,
                        alt.sense,
                        alt.arguments.len()
                    );
                }
            }

            if !analysis.theta_roles.is_empty() {
                println!("🔗 Compatible theta roles: {:?}", analysis.theta_roles);
            }
        }
        Err(e) => {
            println!("❌ Error: {e:?}");
        }
    }
}

fn demo_semantic_role_patterns(engine: &PropBankEngine) {
    println!("\n{}", "=".repeat(70));
    println!("🎭 SEMANTIC ROLE PATTERN ANALYSIS");
    println!("{}", "=".repeat(70));

    let predicates = [
        ("give", "01"),
        ("take", "01"),
        ("run", "01"),
        ("think", "01"),
        ("make", "01"),
        ("explain", "01"),
    ];

    for (lemma, sense) in predicates {
        if let Ok(structure) = engine.get_argument_structure(lemma, sense) {
            println!("\n🏗️  {} argument structure:", structure.predicate);
            println!("  • Core arguments: {}", structure.core_argument_count);
            println!("  • Modifiers: {}", structure.modifier_count);
            println!("  • Total: {}", structure.total_arguments);
            println!("  • Theta roles: {:?}", structure.theta_roles);
        }
    }
}

fn demo_predicate_similarity(engine: &PropBankEngine) {
    println!("\n{}", "=".repeat(70));
    println!("🔗 PREDICATE SIMILARITY ANALYSIS");
    println!("{}", "=".repeat(70));

    let reference_predicates = [("give", "01"), ("run", "01"), ("make", "01")];

    for (lemma, sense) in reference_predicates {
        println!("\n🎯 Finding predicates similar to {lemma}.{sense}:");
        match engine.find_similar_predicates(lemma, sense) {
            Ok(similar) => {
                if similar.is_empty() {
                    println!("  No similar predicates found");
                } else {
                    println!("  Found {} similar predicates:", similar.len());
                    for pred in similar.iter().take(5) {
                        // Show top 5
                        println!(
                            "    • {}.{}: {} args",
                            pred.lemma,
                            pred.sense,
                            pred.arguments.len()
                        );
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Error: {e:?}");
            }
        }
    }
}

fn demo_fuzzy_matching(engine: &PropBankEngine) {
    println!("\n{}", "=".repeat(70));
    println!("🔍 FUZZY MATCHING CAPABILITIES");
    println!("{}", "=".repeat(70));

    let fuzzy_queries = ["giv", "runn", "mak", "explai", "xyz"];

    for query in fuzzy_queries {
        println!("\n🔎 Fuzzy search for: '{query}'");
        match engine.analyze_word(query) {
            Ok(result) => {
                if result.data.has_match() {
                    println!("  ✅ Found matches (confidence: {:.2})", result.confidence);
                    if let Some(ref pred) = result.data.predicate {
                        println!("    → {}.{}", pred.lemma, pred.sense);
                    }
                    for alt in &result.data.alternative_rolesets {
                        println!("    → {}.{} (alternative)", alt.lemma, alt.sense);
                    }
                } else {
                    println!("  ❌ No matches found");
                }
            }
            Err(e) => {
                println!("  ⚠️  Error: {e:?}");
            }
        }
    }
}

fn demo_performance_metrics(engine: &PropBankEngine) {
    println!("\n{}", "=".repeat(70));
    println!("📊 PERFORMANCE & STATISTICS");
    println!("{}", "=".repeat(70));

    // Get PropBank-specific statistics
    let propbank_stats = engine.get_propbank_stats();
    println!("\n📈 PropBank Data Statistics:");
    println!("  • Total framesets: {}", propbank_stats.total_framesets);
    println!("  • Total predicates: {}", propbank_stats.total_predicates);
    println!(
        "  • Avg arguments per predicate: {:.2}",
        propbank_stats.avg_arguments_per_predicate
    );
    println!(
        "  • .prop files processed: {}",
        propbank_stats.prop_files_processed
    );
    println!(
        "  • .gold_skel files processed: {}",
        propbank_stats.gold_skel_files_processed
    );

    // Engine statistics
    let stats = engine.statistics();
    println!("\n⚡ Engine Performance:");
    println!("  • Engine: {}", stats.engine_name);
    println!("  • Total queries: {}", stats.performance.total_queries);
    println!(
        "  • Avg query time: {:.2}μs",
        stats.performance.avg_query_time_us
    );
    println!(
        "  • Min query time: {}μs",
        stats.performance.min_query_time_us
    );
    println!(
        "  • Max query time: {}μs",
        stats.performance.max_query_time_us
    );

    // Cache statistics
    let cache_stats = engine.cache_stats();
    println!("\n💾 Cache Performance:");
    println!("  • Current size: {}", cache_stats.current_size);
    println!("  • Total lookups: {}", cache_stats.total_lookups);
    println!("  • Hit rate: {:.2}%", cache_stats.hit_rate * 100.0);
    println!("  • Evictions: {}", cache_stats.evictions);
}

fn demo_batch_processing(engine: &PropBankEngine) {
    println!("\n{}", "=".repeat(70));
    println!("🚀 BATCH PROCESSING PERFORMANCE");
    println!("{}", "=".repeat(70));

    let test_words = vec![
        "give", "take", "run", "walk", "think", "believe", "say", "make", "build", "eat", "read",
        "write", "love", "teach", "send", "break",
    ];

    println!("\n📦 Processing {} words in batch...", test_words.len());
    let start = Instant::now();
    let results = engine.analyze_batch(&test_words);
    let duration = start.elapsed();

    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.len() - successful;

    println!("⏱️  Total time: {duration:?}");
    println!("📊 Results: {successful} successful, {failed} failed");
    println!(
        "🚄 Rate: {:.0} words/sec",
        test_words.len() as f64 / duration.as_secs_f64()
    );
    println!(
        "⚡ Avg per word: {:.2}μs",
        duration.as_micros() as f64 / test_words.len() as f64
    );

    // Show a few sample results
    println!("\n📋 Sample results:");
    for (word, result) in test_words.iter().zip(results.iter()).take(5) {
        match result {
            Ok(analysis) => {
                println!(
                    "  ✅ {}: confidence {:.2}, {} args",
                    word, analysis.data.confidence, analysis.data.argument_count
                );
            }
            Err(e) => {
                println!("  ❌ {word}: {e:?}");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏛️  PropBank Semantic Role Labeling Demo");
    println!("{}", "=".repeat(70));
    println!("This demo showcases PropBank's predicate-argument structure analysis");
    println!("and semantic role labeling capabilities for natural language understanding.\n");

    // Create temporary PropBank data
    println!("🔧 Setting up demo PropBank data...");
    let temp_dir = create_demo_propbank_data()?;

    // Initialize PropBank engine
    let config = PropBankConfig::default()
        .with_data_path(temp_dir.path().join("propbank-release").join("data"))
        .with_prop_files(true)
        .with_gold_skel_files(true)
        .with_cache(true, 1000)
        .with_fuzzy_matching(true)
        .with_verbose(false);

    println!("⚡ Initializing PropBank engine...");
    let engine = PropBankEngine::with_config(config)?;

    if !engine.is_initialized() {
        eprintln!("❌ Failed to initialize PropBank engine");
        return Ok(());
    }

    println!("✅ PropBank engine initialized successfully!\n");

    // Demo 1: Individual predicate analysis
    println!("{}", "=".repeat(70));
    println!("🎯 INDIVIDUAL PREDICATE ANALYSIS");
    println!("{}", "=".repeat(70));

    let predicates = [
        ("give", "01"),
        ("give", "02"),
        ("take", "01"),
        ("run", "01"),
        ("think", "01"),
        ("explain", "01"),
    ];

    for (lemma, sense) in predicates {
        print_predicate_analysis(&engine, lemma, sense);
    }

    // Demo 2: Word analysis (all senses)
    println!("\n\n{}", "=".repeat(70));
    println!("📚 WORD ANALYSIS (ALL SENSES)");
    println!("{}", "=".repeat(70));

    let words = ["give", "run", "make", "love"];
    for word in words {
        print_word_analysis(&engine, word);
    }

    // Demo 3: Semantic role patterns
    demo_semantic_role_patterns(&engine);

    // Demo 4: Predicate similarity
    demo_predicate_similarity(&engine);

    // Demo 5: Fuzzy matching
    demo_fuzzy_matching(&engine);

    // Demo 6: Batch processing
    demo_batch_processing(&engine);

    // Demo 7: Performance metrics
    demo_performance_metrics(&engine);

    // Cleanup demo
    println!("\n{}", "=".repeat(70));
    println!("🎉 DEMO COMPLETED SUCCESSFULLY!");
    println!("{}", "=".repeat(70));
    println!("PropBank engine demonstrated:");
    println!("✅ Semantic role labeling (ARG0-5, ARGM-*)");
    println!("✅ Theta role mapping for cross-engine compatibility");
    println!("✅ Predicate-argument structure analysis");
    println!("✅ Fuzzy matching and confidence scoring");
    println!("✅ High-performance batch processing");
    println!("✅ Comprehensive caching and statistics");
    println!("\n🚀 Ready for integration with other Canopy engines!");

    Ok(())
}
