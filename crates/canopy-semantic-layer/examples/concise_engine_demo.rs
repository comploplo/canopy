//! Concise Semantic Engine Data Demo
//!
//! Shows the essential data extracted from each engine in a clean, readable format.

use canopy_semantic_layer::{coordinator::CoordinatorConfig, SemanticCoordinator};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    println!("🔍 Semantic Engine Data Extraction");
    println!("==================================");

    let config = CoordinatorConfig {
        enable_verbnet: true,
        enable_framenet: true,
        enable_wordnet: true,
        enable_lexicon: true,
        graceful_degradation: true,
        confidence_threshold: 0.01,
        ..CoordinatorConfig::default()
    };

    let coordinator = SemanticCoordinator::new(config)?;
    let stats = coordinator.get_statistics();
    println!("✅ Engines loaded: {:?}\n", stats.active_engines);

    let test_words = ["give", "break", "walk", "teacher", "beautiful"];

    for word in test_words {
        println!("📝 \"{}\"", word);

        match coordinator.analyze(word) {
            Ok(result) => {
                print_engines_summary(&result);
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

fn print_engines_summary(result: &canopy_semantic_layer::coordinator::Layer1SemanticResult) {
    // VerbNet summary
    if let Some(ref vn) = result.verbnet {
        if !vn.verb_classes.is_empty() {
            print!("   🏷️  VerbNet: ");
            for (i, class) in vn.verb_classes.iter().take(2).enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", class.class_name);
                if !class.themroles.is_empty() {
                    let roles: Vec<_> = class
                        .themroles
                        .iter()
                        .take(3)
                        .map(|r| r.role_type.as_str())
                        .collect();
                    print!(" ({}", roles.join("/"));
                    if class.themroles.len() > 3 {
                        print!("+{}more", class.themroles.len() - 3);
                    }
                    print!(")");
                }
            }
            if vn.verb_classes.len() > 2 {
                print!(" +{} more", vn.verb_classes.len() - 2);
            }
            println!();
        }
    }

    // FrameNet summary
    if let Some(ref fn_result) = result.framenet {
        if !fn_result.frames.is_empty() {
            print!("   🖼️  FrameNet: ");
            for (i, frame) in fn_result.frames.iter().take(3).enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", frame.name);
                let core_elements: Vec<_> = frame
                    .frame_elements
                    .iter()
                    .filter(|e| matches!(e.core_type, canopy_framenet::CoreType::Core))
                    .take(3)
                    .map(|e| e.name.as_str())
                    .collect();
                if !core_elements.is_empty() {
                    print!(" ({})", core_elements.join("/"));
                }
            }
            if fn_result.frames.len() > 3 {
                print!(" +{} more", fn_result.frames.len() - 3);
            }
            println!();
        }
    }

    // WordNet summary
    if let Some(ref wn) = result.wordnet {
        if !wn.synsets.is_empty() {
            print!("   📚 WordNet: ");
            for (i, synset) in wn.synsets.iter().take(3).enumerate() {
                if i > 0 {
                    print!(", ");
                }
                let def = synset.definition();
                let short_def = if def.len() > 40 {
                    format!("{}...", &def[..37])
                } else {
                    def.to_string()
                };
                print!("{:?}:{}", synset.pos, short_def);
            }
            if wn.synsets.len() > 3 {
                print!(" +{} more", wn.synsets.len() - 3);
            }
            println!();
        }
    }

    // Quick stats
    let vn_count = result
        .verbnet
        .as_ref()
        .map(|v| v.verb_classes.len())
        .unwrap_or(0);
    let fn_count = result
        .framenet
        .as_ref()
        .map(|f| f.frames.len())
        .unwrap_or(0);
    let wn_count = result
        .wordnet
        .as_ref()
        .map(|w| w.synsets.len())
        .unwrap_or(0);
    let total = vn_count + fn_count + wn_count;

    if total > 0 {
        println!(
            "   📊 Total: {} semantic units (VN:{}, FN:{}, WN:{}) | Confidence: {:.2}",
            total, vn_count, fn_count, wn_count, result.confidence
        );
    } else {
        println!("   ❌ No semantic data found");
    }
}
