//! # Canopy Discourse - Layer 3 DRT & Context Tracking
//!
//! This crate implements Discourse Representation Theory (DRT) for building
//! semantic representations that span multiple sentences. It takes Layer 2
//! event structures and constructs discourse-level meaning representations.
//!
//! ## Overview
//!
//! Layer 3 builds on top of Layer 2 event composition:
//!
//! ```text
//! Layer 2 (canopy-events)              Discourse Context
//!         ↓                                    ↓
//! ComposedEvents                       ReferentRegistry
//!         ↓                                    ↓
//!         └──────────────┬─────────────────────┘
//!                        ↓
//!               DiscourseContext
//!                        ↓
//!            Discourse Representation Structure (DRS)
//! ```
//!
//! ## Discourse Representation Structures
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
//! ## Usage
//!
//! ```rust,no_run
//! use canopy_discourse::{DiscourseContext, DiscourseConfig};
//! use canopy_discourse::referent::ReferentType;
//!
//! // Create a discourse context
//! let mut ctx = DiscourseContext::with_defaults();
//!
//! // Process first sentence
//! ctx.begin_sentence("A man walks.".to_string());
//! let man_id = ctx.introduce_referent("man".to_string(), ReferentType::Individual).unwrap();
//! ctx.end_sentence();
//!
//! // Process second sentence with pronoun
//! ctx.begin_sentence("He whistles.".to_string());
//! let resolved = ctx.resolve_pronoun("he");
//! ctx.end_sentence();
//!
//! // Get the DRS
//! let drs = ctx.drs();
//! println!("DRS has {} referents", drs.referent_count());
//! ```
//!
//! ## Key Components
//!
//! - [`Drs`]: Discourse Representation Structure with universe and conditions
//! - [`DiscourseContext`]: Manages discourse state across sentences
//! - [`DiscourseReferent`]: Entities and events that can be referred to
//! - [`ReferentRegistry`]: Tracks all active discourse referents
//!
//! ## Anaphora Resolution
//!
//! The system resolves pronouns using:
//! - **Recency**: Referents from recent sentences are preferred
//! - **Animacy**: Human referents are more salient for he/she
//! - **Gender/Number agreement**: Filters incompatible candidates
//!
//! ## References
//!
//! - Kamp, H. (1981). A theory of truth and semantic representation.
//! - Kamp, H., & Reyle, U. (1993). From Discourse to Logic.

pub mod context;
pub mod drs;
pub mod error;
pub mod gender;
pub mod logophoricity;
pub mod referent;
pub mod reflexivity;

// Re-export main types
pub use context::{ContextStatistics, DiscourseConfig, DiscourseContext, SentenceInfo};
pub use drs::{
    AttitudeType, Drs, DrsCondition, DrsId, SubordinateDrs, SubordinationRelation,
    TemporalRelationType,
};
pub use error::{DiscourseError, DiscourseResult};
pub use gender::{GenderLookup, GenderLookupError};
pub use logophoricity::{is_picture_noun, LogophoricContext, LogophoricDetector};
pub use referent::{
    classify_anaphor, is_personal_pronoun, is_pronoun, is_self_anaphor, AnaphorClassification,
    AnaphorType, DiscourseReferent, Gender, NumberFeature, Person, PropertyValue, ReferentId,
    ReferentRegistry, ReferentType,
};
pub use reflexivity::{ConditionBResult, PredicateAnalyzer};

// Re-export core types used in our API
pub use canopy_core::ThetaRole;
pub use canopy_events::ComposedEvent;
