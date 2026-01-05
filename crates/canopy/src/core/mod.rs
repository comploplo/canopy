//! Core types for the Canopy semantic kernel.
//!
//! This module contains foundational types that are used throughout the kernel:
//! - Error types (`CanopyError`)
//! - Theta roles (`ThetaRole`)
//! - Event decomposition primitives (`LittleV`, `Entity`, `Event`)
//! - Syntactic categories (`UPos`, `DepRel`)
//!
//! These types are kernel-internal and don't depend on any external resources.

mod error;
mod event;
mod syntax;
mod theta;

pub use error::{CanopyError, CanopyResult};
pub use event::{
    Action, AspectualClass, Distributivity, Entity, Event, EventModality, LittleV, ModalFlavor,
    ModalForce, Path, PossessionType, Proposition, PsychType, SemanticNumber, State, Voice,
};
pub use syntax::{
    Case, Definiteness, DepRel, Gender, Mood, MorphFeatures, MorphVoice, Number, Person, Tense,
    UPos, VerbForm,
};
pub use theta::ThetaRole;
