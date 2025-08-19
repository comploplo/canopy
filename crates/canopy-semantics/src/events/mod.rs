//! Event structure module
//!
//! This module implements event-based semantic representations drawing from:
//! - Davidson (1967): Events as arguments in logical form
//! - Parsons (1990): Neo-Davidsonian event semantics
//! - Pietroski (2005): Event composition and modification
//!
//! The event structures combine Davidson's insights with Minimalist syntax,
//! providing a bridge between syntactic derivations and semantic interpretation.

pub mod aspect;
pub mod composition;
pub mod event_semantics;
pub mod example_usage;

pub use aspect::*;
pub use composition::*;
pub use event_semantics::{
    Animacy, CaseType, ChainLink, Concreteness, Definiteness, Event, EventBuildError, EventBuilder,
    EventId, EventModifier, EventStructure, EventTime, LandingSite, ModifierType, MovementChain,
    MovementFeature, MovementType as EventMovementType, Number, Participant, ParticipantFeatures,
    Predicate, PredicateType, SemanticFeature,
};

#[cfg(test)]
mod tests;
