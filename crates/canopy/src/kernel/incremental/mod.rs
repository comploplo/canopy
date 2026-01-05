//! Incremental processing with surprisal-based disambiguation.
//!
//! This module implements surprisal theory (Hale 2001, Levy 2008) for
//! incremental left-to-right processing with probability-weighted readings.
//!
//! # Academic Foundations
//!
//! - **Surprisal Theory**: Processing difficulty at word w = -log P(w|context)
//! - **Entropy Reduction**: Processing reflects uncertainty reduction (Roark et al. 2009)
//! - **Garden-Path Detection**: High surprisal indicates reanalysis needed
//!
//! # Architecture
//!
//! ```text
//! Token Stream ──► IncrementalProcessor ──► IncrementalState
//!                        │                        │
//!                        ▼                        ▼
//!                  SurprisalModel           Surprisal Trace
//!                        │                   Beam of Readings
//!                        ▼                   Entropy
//!                  P(word|context)
//! ```

mod beam;
mod lm;
mod state;
mod surprisal;

pub use beam::{BeamSearch, BeamSearchConfig};
pub use lm::{SurprisalModel, UniformSurprisalModel};
pub use state::{IncrementalProcessor, IncrementalState, ReadingPrefix};
pub use surprisal::{GardenPathDetector, GardenPathEvent, Surprisal};
