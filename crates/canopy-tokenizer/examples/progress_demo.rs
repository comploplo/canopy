//! Progress Bar Demo
//!
//! Shows the engine loading progress bars in action by running a fresh load.

use canopy_tokenizer::coordinator::{CoordinatorConfig, SemanticCoordinator};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Minimize logging to show progress bars better
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    println!("🚀 Engine Loading Progress Demo");
    println!("===============================\n");

    let config = CoordinatorConfig {
        enable_verbnet: true,
        enable_framenet: true,
        enable_wordnet: true,
        enable_lexicon: true,
        ..CoordinatorConfig::default()
    };

    // This will trigger the progress bars
    let coordinator = SemanticCoordinator::new(config)?;

    let stats = coordinator.get_statistics();
    println!("✨ Ready for semantic analysis!");
    println!("   Engines: {}", stats.active_engines.join(", "));
    println!(
        "   Memory: {:.1}MB cache allocated",
        stats.memory_usage.budget_mb
    );

    Ok(())
}
