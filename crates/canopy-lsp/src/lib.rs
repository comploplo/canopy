//! Canopy LSP: Language Server Protocol implementation for linguistic analysis
//!
//! This crate provides the LSP server that integrates all canopy components
//! using dependency injection to avoid circular dependencies.

pub mod diagnostics;
pub mod handlers;
pub mod lsp_backend; // TODO: Implement proper LSP server with tower-lsp
pub mod server;
pub mod verbnet_test; // VerbNet integration test

// Re-export main functions for testing
pub use crate::main_impl::create_default_configs;

// Main implementation module
mod main_impl {
    use std::error::Error;

    /// Create default server configurations
    pub fn create_default_configs() -> (
        canopy_core::layer1parser::Layer1HelperConfig,
        canopy_core::layer1parser::SemanticConfig,
    ) {
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

        (parser_config, semantic_config)
    }
}

// Coverage improvement tests for M3
#[cfg(test)]
mod error_handling_tests;

#[cfg(test)]
mod tests;

// Quick coverage boost for 70% target
#[cfg(test)]
mod quick_coverage_boost;

// Comprehensive diagnostics tests for M3 coverage
#[cfg(test)]
mod diagnostics_tests;

// LSP backend stub tests for M3 coverage
#[cfg(test)]
mod lsp_backend_tests;

// LSP main functionality tests for M3 coverage
#[cfg(test)]
mod main_tests;

// Final coverage push tests to reach 70%
#[cfg(test)]
mod coverage_final_push;

// LSP main.rs coverage tests for 0% coverage files
#[cfg(test)]
mod lsp_main_coverage_tests;

use canopy_core::AnalysisResult;
use canopy_core::layer1parser::{Layer1ParserHandler, SemanticAnalysisHandler};

/// Main LSP server factory that creates a fully configured canopy server
pub struct CanopyLspServerFactory;

impl CanopyLspServerFactory {
    /// Create a new canopy server with dependency injection
    pub fn create_server() -> AnalysisResult<impl server::CanopyServer> {
        // Create the layer handlers
        let parser_handler = Layer1ParserHandler::new();
        let semantic_handler = SemanticAnalysisHandler::new();

        // Inject dependencies into the core server
        let server = server::DefaultCanopyServer::new(parser_handler, semantic_handler);

        Ok(server)
    }

    /// Create a server with custom configuration
    pub fn create_server_with_config(
        parser_config: canopy_core::layer1parser::Layer1HelperConfig,
        semantic_config: canopy_core::layer1parser::SemanticConfig,
    ) -> AnalysisResult<impl server::CanopyServer> {
        let parser_handler = Layer1ParserHandler::with_config(parser_config);
        let semantic_handler = SemanticAnalysisHandler::with_config(semantic_config);

        let server = server::DefaultCanopyServer::new(parser_handler, semantic_handler);

        Ok(server)
    }
}

/// Integration point that resolves the circular dependency issue
///
/// This module can import from both canopy-parser and canopy-semantics
/// and coordinate between them without creating cycles.
pub mod integration {
    // Legacy imports commented out for semantic-first architecture
    // use canopy_parser::{Layer1Parser, UDPipeEngine};

    // Legacy integration structs commented out for semantic-first architecture
    // /// Real Layer 1 handler that uses actual UDPipe and VerbNet
    // ///
    // /// This handler bridges the parser and semantics crates safely
    // /// by living in the LSP layer that depends on both.
    // pub struct RealLayer1Handler {
    //     udpipe_parser: Layer1Parser,
    //     verbnet_engine: VerbNetEngine,
    //     config: canopy_core::layer1parser::Layer1HelperConfig,
    // }
    //
    //     impl RealLayer1Handler {
    //         /// Create a real Layer 1 handler with actual engines
    //         pub fn new() -> AnalysisResult<Self> {
    //             // Initialize UDPipe
    //             let udpipe_engine = UDPipeEngine::for_testing(); // Use test engine (real model if available)
    //             let udpipe_parser = Layer1Parser::new(udpipe_engine);
    //
    //             // Initialize VerbNet with test data
    //             let mut verbnet_engine = VerbNetEngine::new();
    //             verbnet_engine.add_test_data(); // Add test verbs for development
    //
    //             Ok(Self {
    //                 udpipe_parser,
    //                 verbnet_engine,
    //                 config: canopy_core::layer1parser::Layer1HelperConfig::default(),
    //             })
    //         }
    //
    //         /// Process text using real UDPipe + VerbNet integration
    //         pub fn process_real(&self, text: &str) -> AnalysisResult<Vec<Word>> {
    //             // Step 1: Parse with UDPipe (basic features)
    //             let enhanced_words =
    //                 self.udpipe_parser
    //                     .parse_document(text)
    //                     .map_err(|e| CanopyError::ParseError {
    //                         context: format!("UDPipe parsing failed: {e:?}"),
    //                     })?;
    //
    //             // Step 2: Convert Layer1Parser::EnhancedWord to core::Word
    //             let words: Vec<Word> = enhanced_words
    //                 .into_iter()
    //                 .map(|enhanced| enhanced.word)
    //                 .collect();
    //
    //             // Step 3: Enhance with VerbNet features
    //             let verbnet_enhanced = self.enhance_with_verbnet(words)?;
    //
    //             Ok(verbnet_enhanced)
    //         }
    //
    //         /// Enhance words with VerbNet semantic features
    //         fn enhance_with_verbnet(&self, words: Vec<Word>) -> AnalysisResult<Vec<Word>> {
    //             let enhanced: Vec<Word> = words.into_iter().inspect(|word| {
    //                 // Use VerbNet for verb analysis
    //                 if word.upos == canopy_core::UPos::Verb {
    //                     // Get VerbNet classes for this verb
    //                     let verb_classes = self.verbnet_engine.get_verb_classes(&word.lemma);
    //
    //                     if !verb_classes.is_empty() {
    //                         // Get theta roles for this verb
    //                         let theta_roles = self.verbnet_engine.get_theta_roles(&word.lemma);
    //
    //                         // Get aspectual classification
    //                         let _aspectual_info = self.verbnet_engine.infer_aspectual_class(&word.lemma);
    //
    //                         // Get semantic predicates
    //                         let predicates = self.verbnet_engine.get_semantic_predicates(&word.lemma);
    //
    //                         // Store VerbNet analysis in word metadata (simplified for now)
    //                         // TODO: Extend Word type to properly store semantic analysis
    //                         if self.config.debug {
    //                             eprintln!("VerbNet analysis for '{}': {} classes, {} theta roles, {} predicates",
    //                                      word.lemma, verb_classes.len(), theta_roles.len(), predicates.len());
    //                         }
    //                     }
    //                 }
    //             }).collect();
    //
    //             Ok(enhanced)
    //         }
    //     }
    //
    //     /// Factory for creating real integrated handlers
    //     pub struct RealServerFactory;
    //
    //     impl RealServerFactory {
    //         /// Create a canopy server with real UDPipe and VerbNet integration
    //         pub fn create() -> AnalysisResult<impl server::CanopyServer> {
    //             let real_handler = RealLayer1Handler::new()?;
    //
    //             // For now, wrap the real handler in a bridge
    //             let parser_bridge = RealParserBridge::new(real_handler);
    //             let semantic_bridge = SemanticAnalysisHandler::new();
    //
    //             Ok(server::DefaultCanopyServer::new(
    //                 parser_bridge,
    //                 semantic_bridge,
    //             ))
    //         }
    //     }
    //
    //     /// Bridge that adapts RealLayer1Handler to the LayerHandler trait
    //     pub struct RealParserBridge {
    //         handler: RealLayer1Handler,
    //     }
    //
    //     impl RealParserBridge {
    //         pub fn new(handler: RealLayer1Handler) -> Self {
    //             Self { handler }
    //         }
    //     }
    //
    //     impl LayerHandler<String, Vec<Word>> for RealParserBridge {
    //         fn process(&self, input: String) -> AnalysisResult<Vec<Word>> {
    //             self.handler.process_real(&input)
    //         }
    //
    //         fn config(&self) -> &dyn canopy_core::layer1parser::LayerConfig {
    //             &self.handler.config
    //         }
    //
    //         fn health(&self) -> canopy_core::layer1parser::ComponentHealth {
    //             canopy_core::layer1parser::ComponentHealth {
    //                 name: "real_layer1_bridge".to_string(),
    //                 healthy: true, // TODO: Check actual health
    //                 last_error: None,
    //                 metrics: std::collections::HashMap::new(),
    //             }
    //         }
    // }
}

// Integration tests are now in the tests module
