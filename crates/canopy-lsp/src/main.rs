//! Canopy LSP Server Binary
//!
//! This is the main entry point for the canopy language server.

use canopy_lsp::{CanopyLspServerFactory, server::CanopyServer};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize basic logging
    println!("Initializing Canopy LSP Server...");

    // Create the canopy server with debug enabled
    let parser_config = canopy_core::layer1parser::Layer1HelperConfig {
        enable_udpipe: true,
        enable_basic_features: true,
        enable_verbnet: true,
        max_sentence_length: 100,
        debug: true, // Enable debug output
        confidence_threshold: 0.5,
    };

    let semantic_config = canopy_core::layer1parser::SemanticConfig {
        enable_theta_roles: true,
        enable_animacy: true,
        enable_definiteness: true,
        confidence_threshold: 0.6,
        debug: true, // Enable debug output
    };

    // Test VerbNet integration directly
    canopy_lsp::verbnet_test::test_verbnet_integration();

    // Create the basic server for now
    let server = CanopyLspServerFactory::create_server_with_config(parser_config, semantic_config)?;

    // Test the server
    println!("Canopy LSP Server starting...");

    let health = server.health();
    println!("Server health: {:?}", health);

    // Test processing with a verb to trigger VerbNet analysis
    let response = server.process_text("John runs quickly")?;
    println!(
        "Test processing: {} words processed in {}μs",
        response.document.total_word_count(),
        response.metrics.total_time_us
    );

    println!("Canopy LSP Server ready!");

    // TODO: Start actual LSP server with tower-lsp
    // For now, just keep the process alive
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    Ok(())
}
