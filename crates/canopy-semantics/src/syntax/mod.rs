//! Syntactic theory module
//!
//! This module implements syntactic structures and operations following:
//! - Minimalist Program (Chomsky 1995, 2000, 2008)
//! - Government and Binding theory (Chomsky 1981)
//! - Movement theory (Lasnik 1999, Fox 2000)
//! - Control theory (Landau 2000, 2013)
//! - Voice and argument structure (Kratzer 1996, Alexiadou et al. 2015)
//!
//! Key components:
//! - Little v shells (Larson 1988, Hale & Keyser 1993)
//! - Movement chains and copy theory
//! - Control and PRO
//! - Voice alternations

pub mod little_v;
pub mod movement;
pub mod voice;
// Future modules for complex movement theory (M4+):
// pub mod chains;
// pub mod control;

// Re-export little_v types (excluding VoiceType to avoid conflict)
pub use little_v::{
    DecompositionCondition, DecompositionRule, EventDecomposer, LittleVFeatures, LittleVShell,
    LittleVType, VPComplement, VerbPattern, VoiceFeatures,
};

// Re-export voice types (this VoiceType is the main one)
pub use voice::*;

// Re-export movement types
pub use movement::*;
