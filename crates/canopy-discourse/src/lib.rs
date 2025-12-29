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
//! ## Temporal Reasoning (Allen's Interval Algebra)
//!
//! Full implementation of Allen's 13 temporal relations for event ordering:
//! - **Basic**: Before, Meets, Overlaps, Starts, During, Finishes, Equals
//! - **Inverse**: After, MetBy, OverlappedBy, StartedBy, Contains, FinishedBy
//!
//! Infers temporal relations from tense/aspect (Dowty 1986):
//! - Past perfect → Before (prior to reference time)
//! - State + Achievement → Overlaps (background)
//! - Achievement sequence → Meets (narrative progression)
//!
//! ## Centering Theory (Thematic Continuity)
//!
//! Tracks discourse topic using Grosz, Joshi & Weinstein (1995):
//! - **Cf (Forward-looking centers)**: Entities ranked by salience
//! - **Cb (Backward-looking center)**: Current discourse topic
//! - **Cp (Preferred center)**: Most salient entity
//!
//! Transition types ordered by coherence:
//! - **Continue**: Cb = prev_Cb = Cp (smooth continuation)
//! - **Retain**: Cb = prev_Cb ≠ Cp (topic retained but challenged)
//! - **SmoothShift**: Cb ≠ prev_Cb, Cb = Cp (smooth topic change)
//! - **RoughShift**: Cb ≠ prev_Cb ≠ Cp (abrupt topic change)
//!
//! ## Coherence Relations
//!
//! Detects discourse relations (Hobbs 1979, Asher & Lascarides 2003):
//! - **Causal**: Result ("John pushed Bill. He fell."), Explanation
//! - **Temporal**: Narration (sequential), Background (overlapping)
//! - **Similarity**: Parallel (same structure), Contrast (opposition)
//! - **Elaboration**: Detail, Exemplification
//!
//! Uses discourse markers ("however" → Contrast, "therefore" → Result),
//! VerbNet causatives, and shared referent analysis.
//!
//! ## Multi-Sentence Integration
//!
//! Builds rich representations across discourse:
//! - **Entity profiles**: Accumulated properties, aliases, event roles
//! - **Event chains**: Causal, temporal, thematic, protagonist-based
//! - **Prominence scoring**: Based on mentions, roles, discourse span
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
//! - [`TemporalReasoner`]: Allen's interval algebra for event ordering
//! - [`CenteringTracker`]: Topic continuity and transition detection
//! - [`CoherenceAnalyzer`]: Discourse relation inference
//! - [`SemanticIntegrator`]: Multi-sentence entity/event integration
//!
//! ## Anaphora Resolution
//!
//! The system resolves pronouns using:
//! - **Recency**: Referents from recent sentences are preferred
//! - **Animacy**: Human referents are more salient for he/she
//! - **Gender/Number agreement**: Filters incompatible candidates
//! - **Binding Theory**: Reuland (2011) and Charnavel (2019) constraints
//!
//! ## References
//!
//! - Kamp, H. (1981). A theory of truth and semantic representation.
//! - Kamp, H., & Reyle, U. (1993). From Discourse to Logic.
//! - Allen, J.F. (1983). Maintaining Knowledge about Temporal Intervals.
//! - Grosz, B., Joshi, A. & Weinstein, S. (1995). Centering.
//! - Hobbs, J. (1979). Coherence and Coreference.
//! - Asher, N. & Lascarides, A. (2003). Logics of Conversation.
//! - Reuland, E. (2011). Anaphora and Language Design.
//! - Charnavel, I. (2019). Exempt Anaphors and Logophoricity.

pub mod centering;
pub mod coherence;
pub mod context;
pub mod drs;
pub mod error;
pub mod gender;
pub mod integration;
pub mod logophoricity;
pub mod referent;
pub mod reflexivity;
pub mod temporal;

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

// Re-export temporal reasoning types
pub use temporal::{
    AllenRelation, ConstraintConfidence, TemporalConstraint, TemporalReasoner, Tense,
};

// Re-export centering theory types
pub use centering::{CenteringTracker, CenteringTransition, CfEntry, GrammaticalRole};

// Re-export coherence relation types
pub use coherence::{CoherenceAnalyzer, CoherenceRelation, DrsSegment};

// Re-export integration types
pub use integration::{ChainType, DiscourseSummary, EntityProfile, EventChain, SemanticIntegrator};

// Re-export core types used in our API
pub use canopy_core::ThetaRole;
pub use canopy_events::ComposedEvent;
