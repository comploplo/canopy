//! Layer 3: Discourse Representation Theory (DRT)
//!
//! Implements Kamp's DRT for building semantic representations that span
//! multiple sentences. Takes Layer 2 event structures and constructs
//! discourse-level meaning representations.
//!
//! # Architecture
//!
//! ```text
//! Layer 2 Events              Discourse Context
//!        ↓                           ↓
//! ComposedEvents              ReferentRegistry
//!        ↓                           ↓
//!        └────────────┬──────────────┘
//!                     ↓
//!              DiscourseContext
//!                     ↓
//!          Discourse Representation Structure (DRS)
//! ```
//!
//! # Discourse Representation Structures
//!
//! A DRS consists of:
//! - **Universe**: A set of discourse referents (entities and events)
//! - **Conditions**: Predicates and relations over those referents
//!
//! For example, "A man walks. He whistles." produces:
//!
//! ```text
//! [ x, e1, e2 |
//!   man(x),
//!   walk(e1),
//!   agent(e1, x),
//!   whistle(e2),
//!   agent(e2, x)
//! ]
//! ```
//!
//! The pronoun "he" is resolved to the same referent `x` as "a man".
//!
//! # Kernel Purity
//!
//! This module contains NO word-level knowledge. The kernel:
//! - Receives composed events from Layer 2
//! - Builds DRS from those events
//! - Resolves anaphora using structural/syntactic constraints
//!
//! Word-level knowledge (gender lookup, animacy, etc.) comes from providers.

mod binding;
mod coherence;
mod context;
mod drs;
mod moves;
mod presupposition;
mod qud;
mod referent;
mod relevance;
mod validation;

pub use binding::{
    AnaphorType, BindingConstraint, BindingResult, PronounResolver, UnderspecBinding,
};
pub use coherence::{
    CoherenceClassification, CoherenceClassifier, CoherenceEdge, CoherenceGraph, CoherenceRelation,
    CoherenceSignal, SentenceData, SentenceReferents,
};
pub use context::{DiscourseConfig, DiscourseContext};
pub use drs::{
    Drs, DrsCondition, DrsId, Label, SubordinateDrs, SubordinationConstraint,
    SubordinationConstraintType, SubordinationRelation, UdrsBuilder, UnderspecDrs,
};
pub use moves::{DiscourseMove, MoveClassification, MoveClassifier, QuestionType};
pub use presupposition::{
    Presupposition, PresuppositionDetector, PresuppositionManager, PresuppositionStatus,
    PriorState, TrackedPresupposition,
};
pub use qud::{
    PartialAnswer, QudIssue, QudOrigin, QudReport, QudReportEntry, QudStack, QudStatus, QudTree,
    QudTreeInfo, QudTreeNode, QudUpdate, QudUpdateAction,
};
pub use referent::{
    DiscourseReferent, Gender, NumberFeature, Person, ReferentId, ReferentRegistry, ReferentType,
};
pub use relevance::{RelevanceAlignment, RelevanceLevel, RelevanceReport};
pub use validation::{ValidationEngine, ValidationReport, ValidationStatus};
