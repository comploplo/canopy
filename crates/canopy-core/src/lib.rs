// canopy-core: Core linguistic types and utilities for canopy

#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::private_intra_doc_links)]

//! # Canopy Core
//!
//! Core linguistic types and utilities for the canopy project - a high-performance
//! linguistic analysis LSP server implementing formal semantic theory in Rust.
//!
//! This crate provides the foundational types for representing linguistic structures,
//! including words, sentences, documents, and theta roles for semantic analysis.
//! It also provides the dependency injection architecture for coordinating between
//! different analysis layers.
//!
//! ## Key Components
//!
//! - [`ThetaRole`]: Semantic roles for argument structure analysis
//! - [`Word`]: Basic word representation with morphological features
//! - [`Sentence`]: Collections of words with positional information
//! - [`Document`]: Complete text documents with sentence boundaries
//! - [`layer1parser`]: Integration helpers bridging parser and semantics
//!
//! ## Example
//!
//! ```rust
//! use canopy_core::{Word, Sentence, ThetaRole};
//!
//! // Create words for "John gives Mary a book"
//! let words = vec![
//!     Word::new(1, "John".to_string(), 0, 4),
//!     Word::new(2, "gives".to_string(), 5, 10),
//!     Word::new(3, "Mary".to_string(), 11, 15),
//!     Word::new(4, "a".to_string(), 16, 17),
//!     Word::new(5, "book".to_string(), 18, 22),
//! ];
//!
//! let sentence = Sentence::new(words);
//! assert_eq!(sentence.word_count(), 5);
//!
//! // Check theta role properties
//! assert!(ThetaRole::Agent.is_core_argument());
//! assert_eq!(ThetaRole::all().len(), 19);
//! ```

pub mod layer1parser;
pub mod paths;
pub mod treebank_loader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[cfg(test)]
use proptest_derive::Arbitrary;

/// Core error types for canopy analysis
#[derive(Error, Debug)]
pub enum CanopyError {
    #[error("parsing failed: {context}")]
    ParseError { context: String },

    #[error("semantic analysis failed: {0}")]
    SemanticError(String),

    #[error("LSP protocol error: {0}")]
    LspError(String),
}

/// Result type for canopy operations
pub type AnalysisResult<T> = Result<T, CanopyError>;

/// Theta roles for semantic analysis (ported from Python V1 system)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum ThetaRole {
    Agent,
    Patient,
    Theme,
    Experiencer,
    Recipient,
    Benefactive,
    Instrument,
    Comitative,
    Location,
    Source,
    Goal,
    Direction,
    Temporal,
    Frequency,
    Measure,
    Cause,
    Manner,
    ControlledSubject,
    Stimulus,
}

impl ThetaRole {
    /// Returns all possible theta roles
    #[must_use]
    pub fn all() -> &'static [ThetaRole] {
        &[
            ThetaRole::Agent,
            ThetaRole::Patient,
            ThetaRole::Theme,
            ThetaRole::Experiencer,
            ThetaRole::Recipient,
            ThetaRole::Benefactive,
            ThetaRole::Instrument,
            ThetaRole::Comitative,
            ThetaRole::Location,
            ThetaRole::Source,
            ThetaRole::Goal,
            ThetaRole::Direction,
            ThetaRole::Temporal,
            ThetaRole::Frequency,
            ThetaRole::Measure,
            ThetaRole::Cause,
            ThetaRole::Manner,
            ThetaRole::ControlledSubject,
            ThetaRole::Stimulus,
        ]
    }

    /// Check if this is a core argument role (Agent, Patient, Theme, etc.)
    #[must_use]
    pub fn is_core_argument(&self) -> bool {
        matches!(
            self,
            ThetaRole::Agent
                | ThetaRole::Patient
                | ThetaRole::Theme
                | ThetaRole::Experiencer
                | ThetaRole::Recipient
        )
    }
}

/// Universal part-of-speech tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UPos {
    Adj,   // adjective
    Adp,   // adposition
    Adv,   // adverb
    Aux,   // auxiliary
    Cconj, // coordinating conjunction
    Det,   // determiner
    Intj,  // interjection
    Noun,  // noun
    Num,   // numeral
    Part,  // particle
    Pron,  // pronoun
    Propn, // proper noun
    Punct, // punctuation
    Sconj, // subordinating conjunction
    Sym,   // symbol
    Verb,  // verb
    X,     // other
}

/// Person values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDPerson {
    First,  // 1
    Second, // 2
    Third,  // 3
}

/// Number values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDNumber {
    Singular, // Sing
    Plural,   // Plur
    Dual,     // Dual
}

/// Gender values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDGender {
    Masculine, // Masc
    Feminine,  // Fem
    Neuter,    // Neut
}

/// Animacy values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDAnimacy {
    Animate,   // Anim
    Inanimate, // Inan
}

/// Case values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDCase {
    Nominative,   // Nom
    Accusative,   // Acc
    Genitive,     // Gen
    Dative,       // Dat
    Instrumental, // Ins
    Locative,     // Loc
    Vocative,     // Voc
    Ablative,     // Abl
}

/// Definiteness values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDDefiniteness {
    Definite,   // Def
    Indefinite, // Ind
    Specific,   // Spec
    Unspecific, // Nspec
}

/// Tense values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDTense {
    Past,    // Past
    Present, // Pres
    Future,  // Fut
}

/// Aspect values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDAspect {
    Perfective,   // Perf
    Imperfective, // Imp
}

/// Mood values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDMood {
    Indicative,  // Ind
    Imperative,  // Imp
    Conditional, // Cnd
    Subjunctive, // Sub
}

/// Voice values for Universal Dependencies morphology (distinct from semantic Voice)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDVoice {
    Active,  // Act
    Passive, // Pass
    Middle,  // Mid
}

/// Degree values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDDegree {
    Positive,    // Pos
    Comparative, // Cmp
    Superlative, // Sup
}

/// `VerbForm` values for Universal Dependencies morphology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum UDVerbForm {
    Finite,             // Fin
    Infinitive,         // Inf
    Participle,         // Part
    Gerund,             // Ger
    ConverbalAdverbial, // Conv
}

/// Morphological features following Universal Dependencies specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MorphFeatures {
    /// Person: first, second, third
    pub person: Option<UDPerson>,
    /// Number: singular, plural, dual
    pub number: Option<UDNumber>,
    /// Gender: masculine, feminine, neuter
    pub gender: Option<UDGender>,
    /// Animacy: animate, inanimate (morphological)
    pub animacy: Option<UDAnimacy>,
    /// Case: nominative, accusative, genitive, etc.
    pub case: Option<UDCase>,
    /// Definiteness: definite, indefinite (morphological)
    pub definiteness: Option<UDDefiniteness>,
    /// Tense: present, past, future
    pub tense: Option<UDTense>,
    /// Aspect: perfective, imperfective
    pub aspect: Option<UDAspect>,
    /// Mood: indicative, subjunctive, imperative
    pub mood: Option<UDMood>,
    /// Voice: active, passive (morphological)
    pub voice: Option<UDVoice>,
    /// Degree: positive, comparative, superlative
    pub degree: Option<UDDegree>,
    /// `VerbForm`: finite, infinitive, participle, gerund
    pub verbform: Option<UDVerbForm>,
    /// Raw features string for features not covered above
    pub raw_features: Option<String>,
}

/// Universal Dependencies dependency relations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum DepRel {
    /// Clausal argument of noun, adjective, or verb
    Acl,
    /// Adverbial clause modifier
    Advcl,
    /// Adverbial modifier
    Advmod,
    /// Adjectival modifier
    Amod,
    /// Appositional modifier
    Appos,
    /// Auxiliary
    Aux,
    /// Passive auxiliary
    AuxPass,
    /// Case marking
    Case,
    /// Coordinating conjunction
    Cc,
    /// Clausal complement
    Ccomp,
    /// Classifier
    Clf,
    /// Compound
    Compound,
    /// Conjunct
    Conj,
    /// Copula
    Cop,
    /// Clausal subject
    Csubj,
    /// Clausal passive subject
    CsubjPass,
    /// Dependent
    Dep,
    /// Determiner
    Det,
    /// Discourse element
    Discourse,
    /// Dislocated elements
    Dislocated,
    /// Expletive
    Expl,
    /// Fixed multiword expression
    Fixed,
    /// Flat multiword expression
    Flat,
    /// Goes with
    Goeswith,
    /// Indirect object
    Iobj,
    /// List
    List,
    /// Marker
    Mark,
    /// Negation modifier
    Neg,
    /// Nominal modifier
    Nmod,
    /// Nominal subject
    Nsubj,
    /// Passive nominal subject
    NsubjPass,
    /// Numeric modifier
    Nummod,
    /// Direct object
    Obj,
    /// Oblique nominal
    Obl,
    /// Orphan
    Orphan,
    /// Parataxis
    Parataxis,
    /// Punctuation
    Punct,
    /// Reparandum
    Reparandum,
    /// Root
    Root,
    /// Vocative
    Vocative,
    /// Open clausal complement
    Xcomp,
    /// Other/unknown relation
    Other(String),
}

impl std::str::FromStr for DepRel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "acl" => DepRel::Acl,
            "advcl" => DepRel::Advcl,
            "advmod" => DepRel::Advmod,
            "amod" => DepRel::Amod,
            "appos" => DepRel::Appos,
            "aux" => DepRel::Aux,
            "aux:pass" => DepRel::AuxPass,
            "case" => DepRel::Case,
            "cc" => DepRel::Cc,
            "ccomp" => DepRel::Ccomp,
            "clf" => DepRel::Clf,
            "compound" => DepRel::Compound,
            "conj" => DepRel::Conj,
            "cop" => DepRel::Cop,
            "csubj" => DepRel::Csubj,
            "csubj:pass" => DepRel::CsubjPass,
            "dep" => DepRel::Dep,
            "det" => DepRel::Det,
            "discourse" => DepRel::Discourse,
            "dislocated" => DepRel::Dislocated,
            "expl" => DepRel::Expl,
            "fixed" => DepRel::Fixed,
            "flat" => DepRel::Flat,
            "goeswith" => DepRel::Goeswith,
            "iobj" => DepRel::Iobj,
            "list" => DepRel::List,
            "mark" => DepRel::Mark,
            "neg" => DepRel::Neg,
            "nmod" => DepRel::Nmod,
            "nsubj" => DepRel::Nsubj,
            "nsubj:pass" => DepRel::NsubjPass,
            "nummod" => DepRel::Nummod,
            "obj" => DepRel::Obj,
            "obl" => DepRel::Obl,
            "orphan" => DepRel::Orphan,
            "parataxis" => DepRel::Parataxis,
            "punct" => DepRel::Punct,
            "reparandum" => DepRel::Reparandum,
            "root" => DepRel::Root,
            "vocative" => DepRel::Vocative,
            "xcomp" => DepRel::Xcomp,
            _ => DepRel::Other(s.to_string()),
        })
    }
}

impl DepRel {
    /// Parse a dependency relation string into a `DepRel` enum
    #[must_use]
    pub fn from_str_simple(s: &str) -> Self {
        s.parse().unwrap_or_else(|()| DepRel::Other(s.to_string()))
    }
}

/// Enhanced word with extracted semantic features (Layer 1.5)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnhancedWord {
    /// Base word information from `UDPipe`
    pub base: Word,
    /// Semantic features extracted by feature extraction pipeline
    pub semantic_features: SemanticFeatures,
    /// Confidence scores for extracted features
    pub confidence: FeatureConfidence,
}

/// Semantic features extracted from morphosyntactic analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SemanticFeatures {
    /// Animacy classification (human > animal > plant > inanimate)
    pub animacy: Option<Animacy>,
    /// Definiteness from determiners and context
    pub definiteness: Option<Definiteness>,
    /// Count/mass distinction for nouns
    pub countability: Option<Countability>,
    /// Concrete vs abstract distinction
    pub concreteness: Option<Concreteness>,
}

/// Countability distinction for nouns
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Countability {
    /// Count nouns: "book", "cat", "idea"
    Count,
    /// Mass nouns: "water", "sand", "information"
    Mass,
    /// Nouns that can be both: "paper" (count) vs "paper" (mass)
    Dual,
}

/// Concreteness classification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Concreteness {
    /// Physical entities: "book", "cat", "building"
    Concrete,
    /// Mental/abstract concepts: "idea", "love", "democracy"
    Abstract,
    /// Events and processes: "meeting", "running", "explosion"
    Eventive,
}

/// Confidence scores for extracted features (0.0 to 1.0)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureConfidence {
    pub animacy: f32,
    pub definiteness: f32,
    pub countability: f32,
    pub concreteness: f32,
}

/// Enhanced word representation with full Universal Dependencies information (Layer 1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Word {
    pub id: usize,
    pub text: String,
    pub lemma: String,
    pub upos: UPos,
    /// Language-specific POS tag
    pub xpos: Option<String>,
    /// Morphological features
    pub feats: MorphFeatures,
    pub head: Option<usize>,
    pub deprel: DepRel,
    /// Enhanced dependencies (graph structure)
    pub deps: Option<String>,
    /// Miscellaneous information (SpaceAfter=No, etc.)
    pub misc: Option<String>,
    pub start: usize,
    pub end: usize,
}

impl Word {
    #[must_use]
    pub fn new(id: usize, text: String, start: usize, end: usize) -> Self {
        Self {
            id,
            lemma: text.to_lowercase(), // Simple lemma for now
            text,
            upos: UPos::X, // Will be determined by parser
            xpos: None,
            feats: MorphFeatures::default(),
            head: None,
            deprel: DepRel::Dep, // Default to generic dependency
            deps: None,
            misc: None,
            start,
            end,
        }
    }
}

/// Sentence containing multiple words
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sentence {
    pub words: Vec<Word>,
    pub start: usize,
    pub end: usize,
}

impl Sentence {
    #[must_use]
    pub fn new(words: Vec<Word>) -> Self {
        let start = words.first().map_or(0, |w| w.start);
        let end = words.last().map_or(0, |w| w.end);
        Self { words, start, end }
    }

    #[must_use]
    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}

/// Document containing multiple sentences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub sentences: Vec<Sentence>,
    pub text: String,
}

impl Document {
    #[must_use]
    pub fn new(text: String, sentences: Vec<Sentence>) -> Self {
        Self { sentences, text }
    }

    #[must_use]
    pub fn sentence_count(&self) -> usize {
        self.sentences.len()
    }

    #[must_use]
    pub fn total_word_count(&self) -> usize {
        self.sentences.iter().map(Sentence::word_count).sum()
    }
}

/// Little v types for event decomposition (Pylkkänen 2008, Hale & Keyser 1993)
///
/// Following current syntactic theory, verbal projections decompose into smaller
/// semantic primitives. Each `LittleV` captures a distinct aspectual and causal flavor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum LittleV {
    /// External causation: "John broke the vase" → CAUSE(John, BECOME(vase, broken))
    /// Introduces an external argument (agent/causer)
    Cause {
        causer: Entity,
        // TODO: Replace with Box<Event> for proper event decomposition
        // Currently simplified to avoid circular dependency during initial implementation
        caused_predicate: String,
        caused_theme: Entity,
    },

    /// Change of state: "The vase broke" → BECOME(vase, broken)
    /// No external causer, theme undergoes change
    Become { theme: Entity, result_state: State },

    /// States: "John is tall", "Mary knows French"
    /// No event progression, just predication
    Be { theme: Entity, state: State },

    /// Activities: "John ran", "Mary sang"
    /// Dynamic but atelic (no inherent endpoint)
    Do { agent: Entity, action: Action },

    /// Psychological predicates: "John fears spiders", "Mary loves chocolate"
    /// Experiencer-stimulus relationships
    Experience {
        experiencer: Entity,
        stimulus: Entity,
        psych_type: PsychType,
    },

    /// Motion/Transfer: "John went home", "The book went to Mary"
    /// Path-based semantics with source/goal structure
    Go { theme: Entity, path: Path },

    /// Possession: "John has a car", "Mary owns the house"
    /// Possessor-possessee relationships
    Have {
        possessor: Entity,
        possessee: Entity,
        possession_type: PossessionType,
    },

    /// Communication: "John said that P", "Mary told John that P"
    /// Speech act semantics with propositional content
    Say {
        speaker: Entity,
        addressee: Option<Entity>,
        content: Proposition,
    },

    /// Existentials: "There is a book on the table"
    /// Pure existence predication
    Exist {
        entity: Entity,
        location: Option<Entity>,
    },
}

/// Entity reference for event participants
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Entity {
    pub id: usize,
    pub text: String,
    pub animacy: Option<Animacy>,
    pub definiteness: Option<Definiteness>,
}

/// Event structure with little v decomposition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: usize,
    pub predicate: String,
    pub little_v: LittleV,
    pub participants: HashMap<ThetaRole, Entity>,
    pub aspect: AspectualClass,
    pub voice: Voice,
}

/// State predication for "be" and "become" structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct State {
    pub predicate: String,
    pub polarity: bool, // true = positive, false = negative
}

/// Action for dynamic "do" events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Action {
    pub predicate: String,
    pub manner: Option<String>,
    pub instrument: Option<Entity>,
}

/// Path structure for motion events (Talmy 2000)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Path {
    pub source: Option<Entity>,
    pub goal: Option<Entity>,
    pub route: Option<Entity>,
    pub direction: Option<Direction>,
}

/// Propositional content for communication events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Proposition {
    pub content: String, // DRS representation (future: proper DRS type)
    pub modality: Option<Modality>,
    pub polarity: bool,
}

/// Psychological predicate subtypes (Belletti & Rizzi 1988)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum PsychType {
    /// Subject-experiencer: "John fears spiders"
    SubjectExp,
    /// Object-experiencer: "Spiders frighten John"
    ObjectExp,
    /// Psych-states: "John is afraid of spiders"
    PsychState,
}

/// Possession relationship types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum PossessionType {
    /// Legal ownership: "John owns the car"
    Legal,
    /// Temporary possession: "John has the keys"
    Temporary,
    /// Kinship: "John has a sister"
    Kinship,
    /// Part-whole: "The car has four wheels"
    PartWhole,
}

/// Aspectual classification (Vendler 1967)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum AspectualClass {
    /// [-dynamic, -telic]: "know", "love"
    State,
    /// [+dynamic, -telic]: "run", "sing"
    Activity,
    /// [+dynamic, +telic]: "paint a picture"
    Accomplishment,
    /// [-durative, +telic]: "arrive", "die"
    Achievement,
}

/// Voice alternations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Voice {
    Active,
    Passive,
    Middle, // "The door opened"
}

/// Spatial directions for path semantics
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    North,
    South,
    East,
    West,
    Forward,
    Backward,
    Into,
    OutOf,
    Through,
    Around,
}

/// Modal operators for propositions
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Modality {
    /// Must, have to
    Necessity,
    /// Can, may
    Possibility,
    /// Should, ought to
    Obligation,
    /// Want, wish
    Desire,
}

/// Animacy hierarchy for semantic feature analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Animacy {
    /// Humans: "John", "teacher"
    Human,
    /// Animals: "cat", "dog"
    Animal,
    /// Plants: "tree", "flower"
    Plant,
    /// Inanimate: "book", "table"
    Inanimate,
}

/// Definiteness for discourse reference
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Definiteness {
    /// "the book", proper nouns
    Definite,
    /// "a book", "some student"
    Indefinite,
    /// "books" (generic), "water"
    Generic,
}

impl LittleV {
    /// Get the external argument (if any) for this little v
    #[must_use]
    pub fn external_argument(&self) -> Option<&Entity> {
        match self {
            LittleV::Cause { causer, .. } => Some(causer),
            LittleV::Do { agent, .. } => Some(agent),
            LittleV::Experience { experiencer, .. } => Some(experiencer),
            LittleV::Go { theme, .. } => Some(theme),
            LittleV::Have { possessor, .. } => Some(possessor),
            LittleV::Say { speaker, .. } => Some(speaker),
            _ => None, // Become, Be, Exist have no external argument
        }
    }

    /// Check if this little v introduces an event variable
    #[must_use]
    pub fn is_eventive(&self) -> bool {
        !matches!(self, LittleV::Be { .. })
    }

    /// Get the aspectual class implied by this little v
    #[must_use]
    pub fn aspectual_class(&self) -> AspectualClass {
        match self {
            LittleV::Be { .. }
            | LittleV::Experience { .. }
            | LittleV::Have { .. }
            | LittleV::Exist { .. } => AspectualClass::State,
            LittleV::Do { .. } | LittleV::Say { .. } => AspectualClass::Activity,
            LittleV::Become { .. } => AspectualClass::Achievement,
            LittleV::Cause { .. } | LittleV::Go { .. } => AspectualClass::Accomplishment,
        }
    }
}

// Include coverage improvement tests for M3
#[cfg(test)]
mod coverage_boost_tests;
#[cfg(test)]
mod coverage_improvement_tests;
#[cfg(test)]
mod utility_coverage_tests;

// Include serialization round-trip tests for M3
#[cfg(test)]
mod serialization_tests;

// Include performance edge case tests for M3
#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_theta_role_core_arguments() {
        // Test all core arguments
        assert!(ThetaRole::Agent.is_core_argument());
        assert!(ThetaRole::Patient.is_core_argument());
        assert!(ThetaRole::Theme.is_core_argument());
        assert!(ThetaRole::Experiencer.is_core_argument());
        assert!(ThetaRole::Recipient.is_core_argument());

        // Test all non-core arguments
        assert!(!ThetaRole::Benefactive.is_core_argument());
        assert!(!ThetaRole::Instrument.is_core_argument());
        assert!(!ThetaRole::Comitative.is_core_argument());
        assert!(!ThetaRole::Location.is_core_argument());
        assert!(!ThetaRole::Source.is_core_argument());
        assert!(!ThetaRole::Goal.is_core_argument());
        assert!(!ThetaRole::Direction.is_core_argument());
        assert!(!ThetaRole::Temporal.is_core_argument());
        assert!(!ThetaRole::Frequency.is_core_argument());
        assert!(!ThetaRole::Measure.is_core_argument());
        assert!(!ThetaRole::Cause.is_core_argument());
        assert!(!ThetaRole::Manner.is_core_argument());
        assert!(!ThetaRole::ControlledSubject.is_core_argument());
        assert!(!ThetaRole::Stimulus.is_core_argument());
    }

    #[test]
    fn test_theta_role_count() {
        // Ensure we have exactly 19 theta roles as in Python V1
        assert_eq!(ThetaRole::all().len(), 19);
    }

    #[test]
    fn test_all_theta_roles_covered() {
        // Test that all theta roles are present and unique
        let all_roles = ThetaRole::all();

        // Check each role is present
        assert!(all_roles.contains(&ThetaRole::Agent));
        assert!(all_roles.contains(&ThetaRole::Patient));
        assert!(all_roles.contains(&ThetaRole::Theme));
        assert!(all_roles.contains(&ThetaRole::Experiencer));
        assert!(all_roles.contains(&ThetaRole::Recipient));
        assert!(all_roles.contains(&ThetaRole::Benefactive));
        assert!(all_roles.contains(&ThetaRole::Instrument));
        assert!(all_roles.contains(&ThetaRole::Comitative));
        assert!(all_roles.contains(&ThetaRole::Location));
        assert!(all_roles.contains(&ThetaRole::Source));
        assert!(all_roles.contains(&ThetaRole::Goal));
        assert!(all_roles.contains(&ThetaRole::Direction));
        assert!(all_roles.contains(&ThetaRole::Temporal));
        assert!(all_roles.contains(&ThetaRole::Frequency));
        assert!(all_roles.contains(&ThetaRole::Measure));
        assert!(all_roles.contains(&ThetaRole::Cause));
        assert!(all_roles.contains(&ThetaRole::Manner));
        assert!(all_roles.contains(&ThetaRole::ControlledSubject));
        assert!(all_roles.contains(&ThetaRole::Stimulus));
    }

    #[test]
    fn test_word_creation() {
        let word = Word::new(1, "Test".to_string(), 0, 4);
        assert_eq!(word.text, "Test");
        assert_eq!(word.lemma, "test"); // Should be lowercase
        assert_eq!(word.start, 0);
        assert_eq!(word.end, 4);
        assert_eq!(word.id, 1);
        assert_eq!(word.upos, UPos::X); // Default value
        assert_eq!(word.xpos, None);
        assert_eq!(word.feats, MorphFeatures::default());
        assert_eq!(word.head, None);
        assert_eq!(word.deprel, DepRel::Dep); // Default value
    }

    #[test]
    fn test_sentence_creation() {
        let words = vec![
            Word::new(1, "The".to_string(), 0, 3),
            Word::new(2, "cat".to_string(), 4, 7),
        ];
        let sentence = Sentence::new(words);
        assert_eq!(sentence.word_count(), 2);
        assert_eq!(sentence.start, 0);
        assert_eq!(sentence.end, 7);
    }

    #[test]
    fn test_empty_sentence() {
        // Test edge case of empty sentence
        let sentence = Sentence::new(vec![]);
        assert_eq!(sentence.word_count(), 0);
        assert_eq!(sentence.start, 0); // Default value for empty
        assert_eq!(sentence.end, 0); // Default value for empty
    }

    #[test]
    fn test_document_methods() {
        // Test document creation and methods
        let words1 = vec![Word::new(1, "Hello".to_string(), 0, 5)];
        let words2 = vec![
            Word::new(2, "world".to_string(), 6, 11),
            Word::new(3, "test".to_string(), 12, 16),
        ];
        let sentences = vec![Sentence::new(words1), Sentence::new(words2)];
        let doc = Document::new("Hello world test".to_string(), sentences);

        assert_eq!(doc.sentence_count(), 2);
        assert_eq!(doc.total_word_count(), 3); // 1 + 2 words
        assert_eq!(doc.text, "Hello world test");
    }

    #[test]
    fn test_empty_document() {
        // Test edge case of empty document
        let doc = Document::new("".to_string(), vec![]);
        assert_eq!(doc.sentence_count(), 0);
        assert_eq!(doc.total_word_count(), 0);
        assert_eq!(doc.text, "");
    }

    #[test]
    fn test_little_v_external_arguments() {
        let john = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let action = Action {
            predicate: "run".to_string(),
            manner: None,
            instrument: None,
        };

        let do_v = LittleV::Do {
            agent: john.clone(),
            action,
        };
        assert_eq!(do_v.external_argument().unwrap().text, "John");

        let state = State {
            predicate: "tall".to_string(),
            polarity: true,
        };

        let be_v = LittleV::Be {
            theme: john.clone(),
            state,
        };
        assert!(be_v.external_argument().is_none());
    }

    #[test]
    fn test_aspectual_classification() {
        let john = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        // States
        let be_v = LittleV::Be {
            theme: john.clone(),
            state: State {
                predicate: "tall".to_string(),
                polarity: true,
            },
        };
        assert_eq!(be_v.aspectual_class(), AspectualClass::State);

        // Activities
        let do_v = LittleV::Do {
            agent: john.clone(),
            action: Action {
                predicate: "run".to_string(),
                manner: None,
                instrument: None,
            },
        };
        assert_eq!(do_v.aspectual_class(), AspectualClass::Activity);

        // Achievements
        let become_v = LittleV::Become {
            theme: john.clone(),
            result_state: State {
                predicate: "awake".to_string(),
                polarity: true,
            },
        };
        assert_eq!(become_v.aspectual_class(), AspectualClass::Achievement);
    }

    #[test]
    fn test_psych_predicate_types() {
        let john = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let spiders = Entity {
            id: 2,
            text: "spiders".to_string(),
            animacy: Some(Animacy::Animal),
            definiteness: Some(Definiteness::Generic),
        };

        // Subject-experiencer: "John fears spiders"
        let fear_v = LittleV::Experience {
            experiencer: john.clone(),
            stimulus: spiders.clone(),
            psych_type: PsychType::SubjectExp,
        };

        assert_eq!(fear_v.external_argument().unwrap().text, "John");
        assert_eq!(fear_v.aspectual_class(), AspectualClass::State);
        assert!(fear_v.is_eventive()); // Experience is eventive
    }

    #[test]
    fn test_all_little_v_variants() {
        let john = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let book = Entity {
            id: 2,
            text: "book".to_string(),
            animacy: Some(Animacy::Inanimate),
            definiteness: Some(Definiteness::Indefinite),
        };

        // Test Cause variant
        let cause_v = LittleV::Cause {
            causer: john.clone(),
            caused_predicate: "break".to_string(),
            caused_theme: book.clone(),
        };
        assert_eq!(cause_v.external_argument().unwrap().text, "John");
        assert_eq!(cause_v.aspectual_class(), AspectualClass::Accomplishment);
        assert!(cause_v.is_eventive());

        // Test Say variant
        let say_v = LittleV::Say {
            speaker: john.clone(),
            addressee: Some(book.clone()),
            content: Proposition {
                content: "hello".to_string(),
                modality: None,
                polarity: true,
            },
        };
        assert_eq!(say_v.external_argument().unwrap().text, "John");
        assert_eq!(say_v.aspectual_class(), AspectualClass::Activity);
        assert!(say_v.is_eventive());

        // Test Exist variant
        let exist_v = LittleV::Exist {
            entity: book.clone(),
            location: None,
        };
        assert!(exist_v.external_argument().is_none());
        assert_eq!(exist_v.aspectual_class(), AspectualClass::State);
        assert!(exist_v.is_eventive()); // Exist is eventive

        // Test Be variant (non-eventive)
        let be_v = LittleV::Be {
            theme: john.clone(),
            state: State {
                predicate: "tall".to_string(),
                polarity: true,
            },
        };
        assert!(be_v.external_argument().is_none());
        assert_eq!(be_v.aspectual_class(), AspectualClass::State);
        assert!(!be_v.is_eventive()); // Be is not eventive
    }

    // Golden tests for deterministic output validation
    #[test]
    fn test_theta_roles_golden() {
        let all_roles = ThetaRole::all();
        insta::assert_debug_snapshot!(all_roles);
    }

    #[test]
    fn test_little_v_golden() {
        let john = Entity {
            id: 1,
            text: "John".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        let mary = Entity {
            id: 2,
            text: "Mary".to_string(),
            animacy: Some(Animacy::Human),
            definiteness: Some(Definiteness::Definite),
        };

        // Test different LittleV types
        let examples = vec![
            LittleV::Do {
                agent: john.clone(),
                action: Action {
                    predicate: "run".to_string(),
                    manner: None,
                    instrument: None,
                },
            },
            LittleV::Have {
                possessor: john.clone(),
                possessee: mary.clone(),
                possession_type: PossessionType::Kinship,
            },
            LittleV::Go {
                theme: john.clone(),
                path: Path {
                    source: None,
                    goal: Some(mary.clone()),
                    route: None,
                    direction: Some(Direction::Forward),
                },
            },
        ];

        insta::assert_debug_snapshot!(examples);
    }

    #[test]
    fn test_document_structure_golden() {
        let words = vec![
            Word::new(1, "John".to_string(), 0, 4),
            Word::new(2, "gives".to_string(), 5, 10),
            Word::new(3, "Mary".to_string(), 11, 15),
            Word::new(4, "a".to_string(), 16, 17),
            Word::new(5, "book".to_string(), 18, 22),
        ];
        let sentence = Sentence::new(words);
        let document = Document::new("John gives Mary a book".to_string(), vec![sentence]);

        insta::assert_debug_snapshot!(document);
    }

    // Property-based tests for linguistic invariants
    proptest! {
        #[test]
        fn prop_word_positions_are_ordered(
            words in prop::collection::vec(
                (1usize..100, "[a-zA-Z]+", 0usize..100),
                1..10
            ).prop_map(|mut word_data| {
                // Ensure words are positioned in order
                word_data.sort_by_key(|(_, _, start)| *start);
                let mut pos = 0;
                word_data.into_iter().enumerate().map(|(i, (_, text, _))| {
                    let start = pos;
                    let end = pos + text.len();
                    pos = end + 1; // Add space between words
                    Word::new(i + 1, text, start, end)
                }).collect::<Vec<_>>()
            })
        ) {
            let sentence = Sentence::new(words.clone());

            // Invariant: sentence boundaries should span all words
            if !words.is_empty() {
                prop_assert_eq!(sentence.start, words.first().unwrap().start);
                prop_assert_eq!(sentence.end, words.last().unwrap().end);
            }

            // Invariant: word count should match
            prop_assert_eq!(sentence.word_count(), words.len());
        }

        #[test]
        fn prop_theta_role_consistency(role in any::<ThetaRole>()) {
            // Invariant: all theta roles should be in the all() list
            prop_assert!(ThetaRole::all().contains(&role));

            // Invariant: core argument classification should be stable
            let is_core = role.is_core_argument();
            prop_assert_eq!(is_core, role.is_core_argument());
        }

        #[test]
        fn prop_document_word_count_sum(
            sentences in prop::collection::vec(
                prop::collection::vec(
                    (1usize..100, "[a-zA-Z]+", 0usize..20),
                    1..5
                ).prop_map(|words| {
                    Sentence::new(
                        words.into_iter().enumerate().map(|(i, (_, text, offset))| {
                            Word::new(i + 1, text.clone(), offset, offset + text.len())
                        }).collect()
                    )
                }),
                1..5
            )
        ) {
            let _total_text_len: usize = sentences.iter()
                .flat_map(|s| &s.words)
                .map(|w| w.text.len())
                .sum();
            let document = Document::new("dummy".to_string(), sentences);

            // Invariant: total word count should equal sum of sentence word counts
            let total_words: usize = document.sentences.iter()
                .map(super::Sentence::word_count)
                .sum();
            prop_assert_eq!(document.total_word_count(), total_words);

            // Invariant: sentence count should match
            prop_assert_eq!(document.sentence_count(), document.sentences.len());
        }

        #[test]
        fn prop_serialization_roundtrip(
            text in "[a-zA-Z ]{1,50}",
            word_count in 1usize..10
        ) {
            // Create a simple document
            let words: Vec<Word> = (0..word_count).map(|i| {
                Word::new(i + 1, format!("word{i}"), i * 5, (i + 1) * 5 - 1)
            }).collect();
            let sentence = Sentence::new(words);
            let original = Document::new(text, vec![sentence]);

            // Serialize and deserialize
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: Document = serde_json::from_str(&json).unwrap();

            // Invariant: roundtrip should preserve all data
            prop_assert_eq!(original, deserialized);
        }
    }
}
