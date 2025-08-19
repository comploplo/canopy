//! # VerbNet Integration Module
//!
//! This module provides integration with VerbNet 3.4 for verb class analysis,
//! theta role assignment, and semantic predicate extraction.
//!
//! ## Copyright Notice
//!
//! VerbNet 3.4 Copyright 2005 by University of Pennsylvania.
//! All rights reserved. Used under permissive license allowing
//! commercial use, modification, and distribution.
//!
//! ## Module Structure
//!
//! - `types`: Core VerbNet data structures
//! - `parser`: XML parsing functionality
//! - `engine`: Fast lookup engine with indices

pub mod engine;
pub mod feature_extraction;
pub mod integration_tests;
pub mod parser;
pub mod types;

// Re-export public API
pub use engine::VerbNetEngine;
pub use feature_extraction::{Animacy, Concreteness, VerbNetFeatureExtractor, VerbNetFeatures};
pub use types::*;
