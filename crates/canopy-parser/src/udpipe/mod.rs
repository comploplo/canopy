//! UDPipe integration module for canopy-parser
//!
//! This module provides a safe Rust interface to the UDPipe C++ library for
//! Universal Dependencies parsing, including tokenization, POS tagging,
//! lemmatization, and dependency parsing.

// Include generated FFI bindings
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/udpipe_bindings.rs"));
}

pub mod engine;
pub mod wrapper;

pub use engine::UDPipeEngine;
pub use wrapper::{ParseError, ParsedDocument, ParsedSentence, ParsedWord, UDPipeParser};

// Re-export core types from canopy-core for convenience
pub use canopy_core::{Document, Sentence, ThetaRole, Word};

#[cfg(test)]
mod tests {
    // Tests for UDPipe module - imports added as needed

    #[test]
    fn test_ffi_available() {
        // Just ensure the FFI bindings compile
        // We'll add actual parsing tests once we have a model
    }
}

#[cfg(test)]
mod morphological_features_test;

#[cfg(test)]
mod real_parsing_tests;

// Coverage improvement tests for M3
#[cfg(test)]
mod ffi_comprehensive_tests;
