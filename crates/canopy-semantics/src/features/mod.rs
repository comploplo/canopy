//! Semantic feature extraction using VerbNet data directly
//!
//! This module leverages VerbNet's rich semantic information:
//! - Selectional restrictions for participant features
//! - Semantic predicates (146+ types) for event classification
//! - No complex inference - just use what VerbNet provides

pub mod predicates;
pub mod verbnet_features;

pub use predicates::*;
pub use verbnet_features::*;
