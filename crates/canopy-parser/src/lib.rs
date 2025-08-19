//! Canopy Parser: UDPipe integration for Universal Dependencies parsing
//!
//! This crate provides a safe Rust interface to UDPipe for tokenization,
//! POS tagging, lemmatization, and dependency parsing.

#![allow(clippy::uninlined_format_args)] // Allow old format style in this crate
#![allow(clippy::field_reassign_with_default)] // Allow field assignment after default()
#![allow(clippy::useless_vec)] // Allow vec! usage for clarity

pub mod benchmarks;
pub mod evaluation;
pub mod layer1;
pub mod memory;
pub mod metrics;
pub mod pipeline;
pub mod udpipe;

// Re-export main types for convenience
pub use layer1::{EnhancedWord, Layer1Config, Layer1Error, Layer1Parser};
pub use pipeline::{
    create_udpipe2_pipeline, CanopyPipeline, PipelineConfig, PipelineError, PipelineMetrics,
};
pub use udpipe::{ParseError, UDPipeEngine, UDPipeParser};
pub use udpipe::{ParsedDocument, ParsedSentence, ParsedWord};

// Re-export memory utilities
pub use memory::{BoundedWordBuilder, MemoryConfig, MemoryStats, ObjectPool, StringPool, VecPool};

// Re-export evaluation utilities
pub use evaluation::{AccuracyMetrics, CorpusEvaluator, GoldStandardSentence, GoldStandardWord};

// Re-export performance metrics
pub use metrics::{
    InputSizeCategory, PerformanceGrade, PerformanceSummary, PerformanceTracker,
    UDPipePerformanceMetrics,
};

// Re-export benchmarking utilities
pub use benchmarks::{BenchmarkResult, BenchmarkSuite};

// Re-export canopy-core types
pub use canopy_core::{Document, Sentence, ThetaRole, UPos, Word};

/// Parser trait for universal parsing interface
pub trait Parser {
    type Error;
    type Document;

    /// Parse text into a structured document
    fn parse(&self, text: &str) -> Result<Self::Document, Self::Error>;
}

impl Parser for UDPipeParser {
    type Error = ParseError;
    type Document = ParsedDocument;

    fn parse(&self, text: &str) -> Result<Self::Document, Self::Error> {
        self.parse_document(text)
    }
}

#[cfg(test)]
mod layer1_latency_test;

#[cfg(test)]
mod golden_tests;

// Include coverage improvement tests for M3
#[cfg(test)]
mod coverage_improvement_tests;
#[cfg(test)]
mod quick_coverage_tests;
#[cfg(test)]
mod simple_coverage_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_trait() {
        // Test that our Parser trait is correctly implemented
        // We can't create an actual UDPipeParser without an engine file,
        // but we can verify the trait bounds compile
        #[allow(dead_code)]
        fn test_parser<P: Parser>(parser: P, text: &str) -> Result<P::Document, P::Error> {
            parser.parse(text)
        }

        // This test just verifies the trait compiles correctly
        #[allow(clippy::assertions_on_constants)]
        assert!(true);
    }
}

// Include coverage tests for lib.rs 0% coverage
#[cfg(test)]
mod lib_coverage_tests;
