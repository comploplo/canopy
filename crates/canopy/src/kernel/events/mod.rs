//! Layer 2: Event Composition
//!
//! Neo-Davidsonian event composition from Layer 1 semantic analysis.
//! Maps semantic predicates to `LittleV` event primitives and binds
//! syntactic dependents to semantic participants.
//!
//! # Architecture
//!
//! ```text
//! AnnotatedSyntax (from SyntaxProvider)
//!         ↓
//! RoleBindings (from RoleProvider)
//!         ↓
//!    EventComposer
//!         ↓
//!   ComposedEvents
//! ```
//!
//! # Event Decomposition
//!
//! Predicates are decomposed into `LittleV` primitives:
//!
//! | Pattern | LittleV | Example |
//! |---------|---------|---------|
//! | Causative | Cause(x, Become(y)) | "John broke the vase" |
//! | Motion | Go(x, path) | "John walked home" |
//! | Transfer | Cause(x, Have(y, z)) | "John gave Mary a book" |
//! | Stative | Be(x, state) | "John is tall" |
//! | Psych | Experience(x, y) | "John fears spiders" |

mod compose;
mod types;

pub use compose::{EventComposer, EventComposerConfig};
pub use types::{
    ComposedEvent, ComposedEvents, DependencyArc, LittleVType, Participant, PresupposedContent,
    Presupposition, PresuppositionTrigger, SentenceAnalysis, SentenceMetadata, UnbindingReason,
    UnboundParticipant,
};
