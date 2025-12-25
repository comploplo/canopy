//! # Canopy Events - Layer 2 Event Composition
//!
//! This crate provides Neo-Davidsonian event composition from Layer 1 semantic analysis.
//! It maps VerbNet semantic predicates to LittleV event primitives and binds syntactic
//! dependents to semantic participants.
//!
//! ## Overview
//!
//! Layer 2 takes the output of Layer 1 (semantic analysis of individual words) and
//! dependency patterns from the treebank to compose sentence-level event structures.
//!
//! ```text
//! Layer 1 (canopy-tokenizer)          Treebank (canopy-treebank)
//!         ↓                                    ↓
//! Layer1SemanticResult              DependencyPattern
//!         ↓                                    ↓
//!         └──────────────┬─────────────────────┘
//!                        ↓
//!               EventComposer
//!                        ↓
//!               ComposedEvents
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use canopy_events::{EventComposer, SentenceAnalysis};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create the composer
//! let composer = EventComposer::new()?;
//!
//! // Create sentence analysis from Layer 1 output
//! let analysis = SentenceAnalysis::new(
//!     "John gave Mary a book".to_string(),
//!     vec![], // Layer 1 tokens would go here
//! );
//!
//! // Compose events
//! let events = composer.compose_sentence(&analysis)?;
//!
//! println!("Composed {} events", events.events.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Event Decomposition
//!
//! VerbNet semantic predicates are mapped to LittleV primitives:
//!
//! | VerbNet | LittleV | Example |
//! |---------|---------|---------|
//! | `cause` | Cause | "John broke the vase" |
//! | `motion` | Go | "John walked home" |
//! | `transfer` | Cause(Have) | "John gave Mary a book" |
//! | `state` | Be | "John is tall" |
//! | `experience` | Experience | "John fears spiders" |
//!
//! ## Participant Binding
//!
//! Universal Dependencies are mapped to theta roles:
//!
//! | Dependency | Theta Role |
//! |------------|------------|
//! | `nsubj` | Agent, Experiencer |
//! | `obj` | Patient, Theme |
//! | `iobj` | Recipient, Benefactive |
//! | `obl` | Location, Instrument |

pub mod binding;
pub mod composer;
pub mod confidence;
pub mod config;
pub mod decomposition;
pub mod error;
pub mod sentence_builder;
pub mod types;

// Re-export main types
pub use composer::EventComposer;
pub use config::EventComposerConfig;
pub use error::{EventError, EventResult};
pub use sentence_builder::{
    SentenceAnalysisBuilder, extract_dependency_arcs, extract_metadata, layer1_tokens_from_parsed,
};
pub use types::{
    ComposedEvent, ComposedEvents, DecomposedEvent, DependencyArc, LittleVType, PredicateInfo,
    SentenceAnalysis, SentenceMetadata, UnbindingReason, UnboundEntity,
};

// Re-export core types for convenience
pub use canopy_core::{
    Action, Animacy, AspectualClass, Definiteness, Entity, Event, LittleV, Path, PossessionType,
    Proposition, PsychType, State, ThetaRole, Voice,
};
