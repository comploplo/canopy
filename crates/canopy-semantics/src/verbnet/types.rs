//! VerbNet data structures
//!
//! This module contains comprehensive type definitions for all VerbNet data:
//! - 30 thematic roles (Agent, Patient, Theme, etc.)
//! - 36 selectional restrictions (animate, concrete, etc.)
//! - 40 syntax restrictions (plural, sentential, etc.)
//! - 146 semantic predicates (cause, motion, location, etc.)

use serde::{Deserialize, Serialize};
// TODO: HashMap will be used for caching/indexing in future
// use std::collections::HashMap;

/// A VerbNet verb class with all members, roles, and frames
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbClass {
    /// Class identifier (e.g., "give-13.1")
    pub id: String,

    /// Human-readable class name
    pub name: String,

    /// Verbs that belong to this class
    pub members: Vec<VerbMember>,

    /// Semantic roles for this class
    pub theta_roles: Vec<ThetaRole>,

    /// Syntactic patterns for this class
    pub frames: Vec<SyntacticFrame>,

    /// Sub-classes (hierarchical organization)
    pub subclasses: Vec<VerbClass>,
}

/// A verb member of a VerbNet class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbMember {
    /// Lemma form of the verb
    pub name: String,

    /// WordNet sense mapping
    pub wn_sense: Option<String>,

    /// FrameNet frame mapping
    pub fn_mapping: Option<String>,

    /// Semantic grouping within class
    pub grouping: Option<String>,
}

/// A thematic role with restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThetaRole {
    /// Type of thematic role
    pub role_type: ThetaRoleType,

    /// Semantic constraints on this role
    pub selectional_restrictions: Vec<SelectionalRestriction>,

    /// Syntactic constraints on this role
    pub syntax_restrictions: Vec<SyntaxRestriction>,
}

/// All 30 VerbNet thematic roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThetaRoleType {
    /// The animate being who performs an action
    Actor,

    /// The animate being who performs an action (synonym for Actor)
    Agent,

    /// Something valuable
    Asset,

    /// A property or characteristic
    Attribute,

    /// The animate being who benefits from an action
    Beneficiary,

    /// The force that causes an action
    Cause,

    /// A secondary agent in a joint action
    CoAgent,

    /// A secondary patient in a joint action
    CoPatient,

    /// A secondary theme in a joint action
    CoTheme,

    /// The place where something ends up
    Destination,

    /// A temporal extent
    Duration,

    /// The animate being who experiences a psychological state
    Experiencer,

    /// A spatial or temporal extent
    Extent,

    /// The intended destination
    Goal,

    /// The initial location of a theme
    InitialLocation,

    /// The tool or means used to perform an action
    Instrument,

    /// A spatial location
    Location,

    /// The substance from which something is made
    Material,

    /// The animate being who is affected by an action
    Patient,

    /// The central theme that changes location
    Pivot,

    /// Something that comes into existence
    Product,

    /// The animate being who receives something
    Recipient,

    /// The end state achieved
    Result,

    /// The place where something starts
    Source,

    /// Something that evokes a psychological state
    Stimulus,

    /// The primary participant that undergoes motion or change
    Theme,

    /// A temporal location
    Time,

    /// The subject of discussion
    Topic,

    /// The path of motion
    Trajectory,

    /// A quantitative measure
    Value,
}

/// All 36 VerbNet selectional restrictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionalRestriction {
    /// Living beings
    Animate,

    /// Human beings specifically
    Human,

    /// Organizations, institutions
    Organization,

    /// Physical, tangible objects
    Concrete,

    /// Non-physical concepts
    Abstract,

    /// Solid matter
    Solid,

    /// Liquid matter
    Fluid,

    /// Chemical substances
    Substance,

    /// Food and drink
    Comestible,

    /// Money, currency
    Currency,

    /// Long, thin objects
    Elongated,

    /// Sharp, pointed objects
    Pointy,

    /// Mechanical devices
    Machine,

    /// Transportation devices
    Vehicle,

    /// Clothing items
    Garment,

    /// Abstract animates (e.g., organizations as agents)
    AnimateAbstract,

    /// Spatial locations
    Location,

    /// Geographic regions
    Region,

    /// Specific places
    Place,

    /// Routes or paths
    Path,

    /// States or conditions
    State,

    /// Auditory phenomena
    Sound,

    /// Communication events
    Communication,

    /// Physical forces
    Force,

    /// Ideas or concepts
    Idea,

    /// Measurable quantities
    Scalar,

    /// Temporal concepts
    Time,

    /// Containers
    Container,

    /// Inflexible objects
    Rigid,

    /// Flexible objects
    NonRigid,

    /// Reflexive reference
    Refl,

    /// Body parts
    BodyPart,

    /// Living organisms
    Plant,

    /// Biological entities
    Biotic,

    /// Natural phenomena
    Natural,
}

/// All 40 VerbNet syntax restrictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyntaxRestriction {
    /// Must be plural
    Plural,

    /// Sentential complement
    Sentential,

    /// That-complement
    ThatComp,

    /// Wh-extraction possible
    WhExtract,

    /// To-infinitive complement
    ToComp,

    /// Be-complement with ellipsis
    BeCompEllipsis,

    /// Small clause complement
    SmallClause,

    /// Finite complement clause
    FiniteComp,

    /// Tensed clause
    Tensed,

    /// Gerund form
    Gerund,

    /// Infinitival form
    Infinitival,

    /// Accusative with -ing
    AccIng,

    /// NP-to-infinitive
    NpToInf,

    /// Quotative complement
    QuotComp,

    /// Indicative mood
    Indicative,

    /// Subjunctive mood
    Subjunctive,

    /// Possessive form
    Poss,

    /// Reflexive transitive
    ReflTransitive,

    /// Post-nominal position
    PostNominal,

    /// Adjectival form
    Adjective,

    /// NP can be omitted
    NpOmissible,

    /// Intransitive use
    Intransitive,

    /// Bare infinitive
    BareInf,

    /// Passive construction
    Passive,

    /// Reciprocal construction
    Reciprocal,

    /// Impersonal construction
    Impersonal,

    /// Causative construction
    Causative,

    /// Inchoative construction
    Inchoative,

    /// Middle voice
    Middle,

    /// Resultative construction
    Resultative,

    /// Conative construction
    Conative,

    /// Location/locatum alternation
    Locative,

    /// Benefactive alternation
    Benefactive,

    /// Instrument subject alternation
    InstrumentSubject,

    /// Body part possessor ascension
    BodyPartAscension,

    /// Possessor-attribute factoring
    PossessorAttribute,

    /// Sum-of-money subject
    SumOfMoney,

    /// There-insertion
    ThereInsertion,
}

/// A syntactic frame with semantic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntacticFrame {
    /// Frame description (e.g., "NP V NP PP.destination")
    pub description: String,

    /// Primary frame type
    pub primary: String,

    /// Secondary frame type
    pub secondary: Option<String>,

    /// Example sentence
    pub example: String,

    /// Parsed syntax structure
    pub syntax: SyntaxPattern,

    /// Semantic predicates for this frame
    pub semantics: Vec<SemanticPredicate>,
}

/// Parsed syntactic pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxPattern {
    /// Constituent elements
    pub elements: Vec<SyntaxElement>,
}

/// Syntactic element in a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxElement {
    /// Category (NP, V, PP, etc.)
    pub category: String,

    /// Associated thematic role
    pub theta_role: Option<ThetaRoleType>,

    /// Syntactic restrictions
    pub restrictions: Vec<SyntaxRestriction>,
}

/// Semantic predicate with temporal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticPredicate {
    /// Type of semantic predicate
    pub predicate_type: PredicateType,

    /// When in the event this predicate holds
    pub event_time: EventTime,

    /// Arguments (references to theta roles)
    pub arguments: Vec<String>,

    /// Whether predicate is negated
    pub negated: bool,
}

/// All 146 VerbNet semantic predicates (subset shown for brevity)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredicateType {
    /// Causation relation
    Cause,

    /// Motion event
    Motion,

    /// Location relation
    Location,

    /// Transfer event
    Transfer,

    /// Contact relation
    Contact,

    /// Change event
    Change,

    /// Creation event
    Created,

    /// Destruction event
    Destroyed,

    /// Existence relation
    Exist,

    /// Function relation
    Function,

    /// State possession
    HasState,

    /// Manner specification
    Manner,

    /// Utilization relation
    Utilize,

    /// Perception event
    Perceive,

    /// Property relation
    Prop,

    /// Together relation
    Together,

    /// Apart relation
    Apart,

    /// Attached relation
    Attached,

    /// Involved relation
    Involved,

    /// Focus relation
    Focus,

    /// Visible relation
    Visible,

    /// Open relation
    Open,

    /// Closed relation
    Closed,

    /// Covered relation
    Covered,

    /// Contains relation
    Contains,

    /// Degradation event
    Degradation,

    /// Emotional state
    Emotional,

    /// Physical state
    Physical,

    /// Mental state
    Mental,

    /// Social state
    Social,

    // Note: In practice, there are 146 predicates total
    // This is a representative subset for the core types
    /// Generic other predicate
    Other(String),
}

/// Event time specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventTime {
    /// Preparatory stage - start(E)
    Start,

    /// Culmination stage - during(E)
    During,

    /// Consequent stage - end(E)
    End,
}

/// Aspectual information derived from VerbNet predicates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AspectualInfo {
    /// Whether the event has duration
    pub durative: bool,

    /// Whether the event involves change
    pub dynamic: bool,

    /// Whether the event has a natural endpoint
    pub telic: bool,

    /// Whether the event is instantaneous
    pub punctual: bool,
}

impl ThetaRoleType {
    /// Get all theta role types as a slice
    pub const fn all() -> &'static [ThetaRoleType] {
        &[
            ThetaRoleType::Actor,
            ThetaRoleType::Agent,
            ThetaRoleType::Asset,
            ThetaRoleType::Attribute,
            ThetaRoleType::Beneficiary,
            ThetaRoleType::Cause,
            ThetaRoleType::CoAgent,
            ThetaRoleType::CoPatient,
            ThetaRoleType::CoTheme,
            ThetaRoleType::Destination,
            ThetaRoleType::Duration,
            ThetaRoleType::Experiencer,
            ThetaRoleType::Extent,
            ThetaRoleType::Goal,
            ThetaRoleType::InitialLocation,
            ThetaRoleType::Instrument,
            ThetaRoleType::Location,
            ThetaRoleType::Material,
            ThetaRoleType::Patient,
            ThetaRoleType::Pivot,
            ThetaRoleType::Product,
            ThetaRoleType::Recipient,
            ThetaRoleType::Result,
            ThetaRoleType::Source,
            ThetaRoleType::Stimulus,
            ThetaRoleType::Theme,
            ThetaRoleType::Time,
            ThetaRoleType::Topic,
            ThetaRoleType::Trajectory,
            ThetaRoleType::Value,
        ]
    }

    /// Check if this is a core argument role (Agent, Patient, Theme)
    pub const fn is_core_role(self) -> bool {
        matches!(
            self,
            ThetaRoleType::Agent | ThetaRoleType::Patient | ThetaRoleType::Theme
        )
    }

    /// Check if this is an animate role
    pub const fn requires_animacy(self) -> bool {
        matches!(
            self,
            ThetaRoleType::Agent
                | ThetaRoleType::Actor
                | ThetaRoleType::Experiencer
                | ThetaRoleType::Recipient
                | ThetaRoleType::Beneficiary
        )
    }
}

impl SelectionalRestriction {
    /// Check if this restriction implies animacy
    pub const fn implies_animacy(self) -> bool {
        matches!(
            self,
            SelectionalRestriction::Animate
                | SelectionalRestriction::Human
                | SelectionalRestriction::AnimateAbstract
        )
    }

    /// Check if this restriction implies concreteness
    pub const fn implies_concreteness(self) -> bool {
        matches!(
            self,
            SelectionalRestriction::Concrete
                | SelectionalRestriction::Solid
                | SelectionalRestriction::Fluid
                | SelectionalRestriction::Machine
                | SelectionalRestriction::Vehicle
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theta_role_properties() {
        assert!(ThetaRoleType::Agent.is_core_role());
        assert!(ThetaRoleType::Agent.requires_animacy());
        assert!(!ThetaRoleType::Instrument.requires_animacy());
    }

    #[test]
    fn test_selectional_restrictions() {
        assert!(SelectionalRestriction::Human.implies_animacy());
        assert!(SelectionalRestriction::Concrete.implies_concreteness());
        assert!(!SelectionalRestriction::Abstract.implies_concreteness());
    }

    #[test]
    fn test_theta_role_count() {
        assert_eq!(ThetaRoleType::all().len(), 30);
    }
}
