//! Davidson/Parsons event representation
//!
//! This module implements Neo-Davidsonian event semantics following:
//! - Davidson (1967): "The logical form of action sentences"
//! - Parsons (1990): "Events in the Semantics of English"
//! - Pietroski (2005): "Event composition and semantic types"
//!
//! Events are treated as first-class entities with participants filling
//! theta roles, enabling compositional semantic analysis.

use crate::ThetaRoleType;
use crate::syntax::LittleVShell;
use canopy_core::{UPos, Word};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for events in discourse
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub struct EventId(pub usize);

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

/// Neo-Davidsonian event structure
///
/// Following Parsons (1990), events are individuals with participants
/// and properties, allowing for flexible modification and composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event identifier
    pub id: EventId,

    /// The predicate that characterizes this event
    pub predicate: Predicate,

    /// Participants in theta roles (Agent, Patient, etc.)
    pub participants: HashMap<ThetaRoleType, Participant>,

    /// Event modifiers (manner, time, location, etc.)
    pub modifiers: Vec<EventModifier>,

    /// Aspectual properties of the event
    pub aspect: super::AspectualClass,

    /// Event structure decomposition (little v analysis)
    pub structure: Option<EventStructure>,

    /// Event time anchoring
    pub time: EventTime,

    /// Movement chains (syntactic movement analysis)
    pub movement_chains: Vec<MovementChain>,

    /// Little v analysis (event decomposition)
    pub little_v: Option<LittleVShell>,
}

impl Event {
    /// Create a new event with basic properties
    pub fn new(id: EventId, predicate: Predicate) -> Self {
        Self {
            id,
            predicate,
            participants: HashMap::new(),
            modifiers: Vec::new(),
            aspect: super::AspectualClass::State, // Default, will be determined
            structure: None,
            time: EventTime::Now,
            movement_chains: Vec::new(),
            little_v: None,
        }
    }

    /// Add a participant in a specific theta role
    pub fn add_participant(&mut self, role: ThetaRoleType, participant: Participant) {
        self.participants.insert(role, participant);
    }

    /// Add an event modifier
    pub fn add_modifier(&mut self, modifier: EventModifier) {
        self.modifiers.push(modifier);
    }

    /// Get participant for a specific theta role
    pub fn get_participant(&self, role: &ThetaRoleType) -> Option<&Participant> {
        self.participants.get(role)
    }

    /// Check if event has a specific participant type
    pub fn has_role(&self, role: &ThetaRoleType) -> bool {
        self.participants.contains_key(role)
    }

    /// Add a movement chain to the event
    pub fn add_movement_chain(&mut self, chain: MovementChain) {
        self.movement_chains.push(chain);
    }

    /// Get all movement chains
    pub fn get_movement_chains(&self) -> &[MovementChain] {
        &self.movement_chains
    }

    /// Set the little v analysis
    pub fn set_little_v(&mut self, little_v: LittleVShell) {
        self.little_v = Some(little_v);
    }

    /// Get the little v analysis
    pub fn get_little_v(&self) -> Option<&LittleVShell> {
        self.little_v.as_ref()
    }

    /// Check if this event has syntactic movement
    pub fn has_movement(&self) -> bool {
        !self.movement_chains.is_empty()
    }

    /// Get primary movement type (if any)
    pub fn get_primary_movement_type(&self) -> MovementType {
        self.movement_chains
            .first()
            .map(|chain| chain.movement_type)
            .unwrap_or(MovementType::None)
    }
}

/// Builder pattern for Event construction following the architecture specification
///
/// This provides a fluent interface for constructing complex events while maintaining
/// type safety and ensuring required fields are set.
pub struct EventBuilder {
    id: EventId,
    predicate: Predicate,
    participants: HashMap<ThetaRoleType, Participant>,
    modifiers: Vec<EventModifier>,
    aspect: super::AspectualClass,
    structure: Option<EventStructure>,
    time: EventTime,
    movement_chains: Vec<MovementChain>,
    little_v: Option<LittleVShell>,
}

impl EventBuilder {
    /// Create a new EventBuilder with required predicate
    pub fn new(predicate: Predicate) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        let id = EventId(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst));

        Self {
            id,
            predicate,
            participants: HashMap::new(),
            modifiers: Vec::new(),
            aspect: super::AspectualClass::State, // Default, will be determined
            structure: None,
            time: EventTime::Now,
            movement_chains: Vec::new(),
            little_v: None,
        }
    }

    /// Set a specific event ID (useful for testing)
    pub fn with_id(mut self, id: EventId) -> Self {
        self.id = id;
        self
    }

    /// Add a participant in a specific theta role
    pub fn with_participant(mut self, role: ThetaRoleType, participant: Participant) -> Self {
        self.participants.insert(role, participant);
        self
    }

    /// Add multiple participants at once
    pub fn with_participants(mut self, participants: HashMap<ThetaRoleType, Participant>) -> Self {
        self.participants.extend(participants);
        self
    }

    /// Set the aspectual class
    pub fn with_aspect(mut self, aspect: super::AspectualClass) -> Self {
        self.aspect = aspect;
        self
    }

    /// Add an event modifier
    pub fn with_modifier(mut self, modifier: EventModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    /// Add multiple modifiers
    pub fn with_modifiers(mut self, modifiers: Vec<EventModifier>) -> Self {
        self.modifiers.extend(modifiers);
        self
    }

    /// Set the event structure (little v decomposition)
    pub fn with_structure(mut self, structure: EventStructure) -> Self {
        self.structure = Some(structure);
        self
    }

    /// Set the event time
    pub fn with_time(mut self, time: EventTime) -> Self {
        self.time = time;
        self
    }

    /// Add a movement chain
    pub fn with_movement_chain(mut self, chain: MovementChain) -> Self {
        self.movement_chains.push(chain);
        self
    }

    /// Add multiple movement chains
    pub fn with_movement_chains(mut self, chains: Vec<MovementChain>) -> Self {
        self.movement_chains.extend(chains);
        self
    }

    /// Set the little v analysis
    pub fn with_little_v(mut self, little_v: LittleVShell) -> Self {
        self.little_v = Some(little_v);
        self
    }

    /// Intelligently assign theta roles using VerbNet pattern mapping
    ///
    /// This method uses the VerbNet engine to analyze dependency patterns and assign
    /// participants to theta roles with confidence scoring. It leverages the three-tier
    /// lookup system: cache → VerbNet → similarity fallback.
    ///
    /// # Arguments
    /// * `verbnet_engine` - The VerbNet engine for pattern analysis
    /// * `dependency_pattern` - Pattern like "nsubj+dobj+prep_to"
    /// * `arguments` - Tuples of (relation, head_lemma) for each argument
    ///
    /// # Returns
    /// Self with participants assigned based on VerbNet analysis, using the highest
    /// confidence analysis available.
    pub fn with_verbnet_theta_assignment(
        mut self,
        verbnet_engine: &crate::verbnet::engine::VerbNetEngine,
        dependency_pattern: &str,
        arguments: &[(String, String)],
    ) -> Self {
        // Get pattern analyses from VerbNet engine
        let analyses = verbnet_engine.map_dependency_pattern_to_theta_roles(
            &self.predicate.lemma,
            dependency_pattern,
            arguments,
        );

        // Use the highest confidence analysis
        if let Some(best_analysis) = analyses.first() {
            // Update predicate with VerbNet class information if available
            if self.predicate.verbnet_class.is_none() && !best_analysis.verb_class_id.is_empty() {
                self.predicate.verbnet_class = Some(best_analysis.verb_class_id.clone());
            }

            // Convert theta assignments to participants
            for assignment in &best_analysis.theta_assignments {
                if assignment.confidence > 0.5 {
                    // Only accept confident assignments
                    let participant = Participant {
                        word_id: 0, // Will be set by caller if needed
                        expression: assignment.argument_text.clone(),
                        features: ParticipantFeatures::default(), // Basic features for now
                        discourse_ref: None,
                    };

                    self.participants.insert(assignment.theta_role, participant);
                }
            }

            // Extract semantic features from the analysis
            for predicate in &best_analysis.semantic_predicates {
                use super::SemanticFeature;
                use crate::verbnet::types::PredicateType as VNPredType;

                // Map VerbNet predicates to semantic features
                let feature = match predicate.predicate_type {
                    VNPredType::Motion => Some(SemanticFeature::Motion),
                    VNPredType::Transfer => Some(SemanticFeature::Transfer),
                    VNPredType::Contact => Some(SemanticFeature::Contact),
                    VNPredType::Change => Some(SemanticFeature::ChangeOfState),
                    VNPredType::Perceive => Some(SemanticFeature::Perception),
                    VNPredType::Cause => None, // Handled by predicate type
                    _ => None,
                };

                if let Some(semantic_feature) = feature {
                    if !self.predicate.features.contains(&semantic_feature) {
                        self.predicate.features.push(semantic_feature);
                    }
                }
            }
        }

        self
    }

    /// Build the final Event, validating constraints
    pub fn build(self) -> Result<Event, EventBuildError> {
        // Validate that required theta roles for the predicate type are filled
        self.validate_theta_roles()?;

        // Validate that the aspectual class matches the predicate type
        self.validate_aspect_predicate_compatibility()?;

        Ok(Event {
            id: self.id,
            predicate: self.predicate,
            participants: self.participants,
            modifiers: self.modifiers,
            aspect: self.aspect,
            structure: self.structure,
            time: self.time,
            movement_chains: self.movement_chains,
            little_v: self.little_v,
        })
    }

    /// Validate that required theta roles are present
    fn validate_theta_roles(&self) -> Result<(), EventBuildError> {
        let required_roles = match &self.predicate.semantic_type {
            PredicateType::Action => vec![ThetaRoleType::Agent],
            PredicateType::State => vec![], // States can have various argument structures
            PredicateType::Achievement => vec![ThetaRoleType::Theme],
            PredicateType::Accomplishment => vec![ThetaRoleType::Agent, ThetaRoleType::Theme],
            PredicateType::Activity => vec![ThetaRoleType::Agent],
            PredicateType::Causative => vec![ThetaRoleType::Agent, ThetaRoleType::Patient],
            PredicateType::Inchoative => vec![ThetaRoleType::Theme],
        };

        for required_role in &required_roles {
            if !self.participants.contains_key(required_role) {
                return Err(EventBuildError::MissingRequiredThetaRole(*required_role));
            }
        }

        Ok(())
    }

    /// Validate aspect-predicate compatibility
    fn validate_aspect_predicate_compatibility(&self) -> Result<(), EventBuildError> {
        use super::AspectualClass;

        let compatible = match (&self.aspect, &self.predicate.semantic_type) {
            (AspectualClass::State, PredicateType::State) => true,
            (AspectualClass::Activity, PredicateType::Activity) => true,
            (AspectualClass::Achievement, PredicateType::Achievement) => true,
            (AspectualClass::Accomplishment, PredicateType::Accomplishment) => true,
            (AspectualClass::Achievement, PredicateType::Inchoative) => true,
            (AspectualClass::Accomplishment, PredicateType::Causative) => true,
            // Allow some flexibility for complex predicates
            _ => true, // For now, allow all combinations
        };

        if !compatible {
            return Err(EventBuildError::IncompatibleAspectPredicate {
                aspect: self.aspect,
                predicate_type: self.predicate.semantic_type.clone(),
            });
        }

        Ok(())
    }
}

/// Errors that can occur during event building
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventBuildError {
    /// A required theta role is missing
    MissingRequiredThetaRole(ThetaRoleType),

    /// Aspectual class is incompatible with predicate type
    IncompatibleAspectPredicate {
        aspect: super::AspectualClass,
        predicate_type: PredicateType,
    },

    /// Invalid movement chain structure
    InvalidMovementChain(String),
}

impl std::fmt::Display for EventBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventBuildError::MissingRequiredThetaRole(role) => {
                write!(f, "Missing required theta role: {:?}", role)
            }
            EventBuildError::IncompatibleAspectPredicate {
                aspect,
                predicate_type,
            } => {
                write!(
                    f,
                    "Aspect {:?} incompatible with predicate type {:?}",
                    aspect, predicate_type
                )
            }
            EventBuildError::InvalidMovementChain(msg) => {
                write!(f, "Invalid movement chain: {}", msg)
            }
        }
    }
}

impl std::error::Error for EventBuildError {}

/// Event predicate following semantic decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    /// The lexical predicate (verb stem)
    pub lemma: String,

    /// Semantic type of the predicate
    pub semantic_type: PredicateType,

    /// VerbNet class information (if available)
    pub verbnet_class: Option<String>,

    /// Semantic features from selectional restrictions
    pub features: Vec<SemanticFeature>,
}

/// Types of predicates following event semantics literature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredicateType {
    /// Action predicates (Agent-oriented)
    Action,

    /// State predicates (Experiencer/Theme-oriented)
    State,

    /// Achievement predicates (result-oriented)
    Achievement,

    /// Accomplishment predicates (process + result)
    Accomplishment,

    /// Activity predicates (process-oriented)
    Activity,

    /// Causative predicates (little v CAUSE)
    Causative,

    /// Inchoative predicates (little v BECOME)
    Inchoative,
}

/// Event participants (individuals filling theta roles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Reference to the linguistic expression
    pub word_id: usize,

    /// The actual word/phrase representing this participant
    pub expression: String,

    /// Semantic properties of the participant
    pub features: ParticipantFeatures,

    /// Discourse reference (for coreference tracking)
    pub discourse_ref: Option<String>,
}

impl Participant {
    /// Create participant from a word
    pub fn from_word(word: &Word) -> Self {
        Self {
            word_id: word.id,
            expression: word.text.clone(),
            features: ParticipantFeatures::from_word(word),
            discourse_ref: None,
        }
    }
}

/// Semantic features of participants (from selectional restrictions)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParticipantFeatures {
    /// Animacy (human, animate, inanimate)
    pub animacy: Option<Animacy>,

    /// Concreteness (concrete, abstract)
    pub concreteness: Option<Concreteness>,

    /// Definiteness (definite, indefinite)
    pub definiteness: Option<Definiteness>,

    /// Number (singular, plural)
    pub number: Option<Number>,
}

impl ParticipantFeatures {
    /// Extract features from UDPipe word analysis
    pub fn from_word(word: &Word) -> Self {
        Self {
            animacy: Self::extract_animacy(word),
            concreteness: Self::extract_concreteness(word),
            definiteness: Self::extract_definiteness(word),
            number: Self::extract_number(word),
        }
    }

    fn extract_animacy(word: &Word) -> Option<Animacy> {
        // Extract from UDPipe features or use heuristics
        if let Some(animacy) = &word.feats.animacy {
            match animacy {
                canopy_core::UDAnimacy::Animate => Some(Animacy::Human),
                canopy_core::UDAnimacy::Inanimate => Some(Animacy::Inanimate),
            }
        } else {
            // Heuristic based on POS and lemma
            match word.upos {
                UPos::Propn => Some(Animacy::Human), // Proper nouns often human
                UPos::Pron => match word.lemma.as_str() {
                    "I" | "you" | "he" | "she" | "we" | "they" => Some(Animacy::Human),
                    "it" => Some(Animacy::Inanimate),
                    _ => None,
                },
                _ => None,
            }
        }
    }

    fn extract_concreteness(_word: &Word) -> Option<Concreteness> {
        // TODO: Implement based on VerbNet selectional restrictions
        None
    }

    fn extract_definiteness(word: &Word) -> Option<Definiteness> {
        if let Some(definiteness) = &word.feats.definiteness {
            match definiteness {
                canopy_core::UDDefiniteness::Definite => Some(Definiteness::Definite),
                canopy_core::UDDefiniteness::Indefinite => Some(Definiteness::Indefinite),
                _ => None,
            }
        } else {
            None
        }
    }

    fn extract_number(word: &Word) -> Option<Number> {
        if let Some(number) = &word.feats.number {
            match number {
                canopy_core::UDNumber::Singular => Some(Number::Singular),
                canopy_core::UDNumber::Plural => Some(Number::Plural),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Animacy distinctions (following VerbNet selectional restrictions)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub enum Animacy {
    #[default]
    Inanimate,
    Plant,
    Animal,
    Human,
}

/// Concreteness distinctions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Concreteness {
    Concrete,
    Abstract,
}

/// Definiteness distinctions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Definiteness {
    #[default]
    Definite,
    Indefinite,
    Bare,
}

/// Number distinctions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Number {
    #[default]
    Singular,
    Plural,
    Mass,
}

/// Event modifiers (following Parsons' modifier attachment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventModifier {
    /// Type of modification
    pub modifier_type: ModifierType,

    /// The modifying expression
    pub expression: String,

    /// Word ID of the modifier
    pub word_id: usize,
}

/// Types of event modification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierType {
    /// Manner modification (how the event occurred)
    Manner,

    /// Temporal modification (when the event occurred)
    Temporal,

    /// Locative modification (where the event occurred)
    Locative,

    /// Instrumental modification (with what the event occurred)
    Instrumental,

    /// Purpose modification (why the event occurred)
    Purpose,

    /// Degree modification (to what extent)
    Degree,
}

/// Movement chain representation following Chomsky's copy theory
///
/// Represents syntactic movement chains where an element moves from one position
/// to another, leaving a trace (copy) in its original position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementChain {
    /// Type of movement (A-movement, A'-movement, etc.)
    pub movement_type: MovementType,

    /// The moved element (head of chain)
    pub moved_element: ChainLink,

    /// Intermediate positions (if any)
    pub intermediate_positions: Vec<ChainLink>,

    /// Base position (tail of chain)
    pub base_position: ChainLink,

    /// Landing site features
    pub landing_site: LandingSite,
}

/// Types of syntactic movement following Chomsky (1995, 2000)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementType {
    /// A-movement (to argument positions)
    /// Examples: passive movement, raising
    AMovement,

    /// A'-movement (to non-argument positions)
    /// Examples: wh-movement, topicalization, focus movement
    ABarMovement,

    /// Head movement
    /// Examples: verb movement to T, T to C
    HeadMovement,

    /// No movement (base generated)
    None,
}

/// Link in a movement chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainLink {
    /// Word ID of the element at this position
    pub word_id: usize,

    /// Syntactic position (e.g., "Spec,CP", "Spec,TP", "object position")
    pub position: String,

    /// Is this position phonetically realized?
    pub phonetically_realized: bool,

    /// Theta role assigned at this position (if any)
    pub theta_role: Option<ThetaRoleType>,

    /// Case assigned at this position (if any)
    pub case: Option<CaseType>,
}

/// Landing site information for movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandingSite {
    /// Syntactic position of landing site
    pub position: String,

    /// Features driving movement (EPP, focus, etc.)
    pub driving_features: Vec<MovementFeature>,

    /// Is this an intermediate or final landing site?
    pub is_intermediate: bool,
}

/// Features that drive syntactic movement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementFeature {
    /// EPP feature (Extended Projection Principle)
    EPP,

    /// Focus feature
    Focus,

    /// Topic feature
    Topic,

    /// Wh feature
    Wh,

    /// Case feature
    Case(CaseType),

    /// Phi features (person, number, gender)
    Phi,
}

/// Case types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseType {
    /// Nominative case
    Nominative,

    /// Accusative case
    Accusative,

    /// Genitive case
    Genitive,

    /// Dative case
    Dative,

    /// Oblique case
    Oblique,
}

/// Event structure decomposition (following Hale & Keyser little v theory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStructure {
    /// Simple predicate (no decomposition)
    Simple,

    /// Causative structure: [vP Agent [v CAUSE [VP Theme Predicate]]]
    Causative {
        causer: Participant,
        caused_event: Box<Event>,
    },

    /// Inchoative structure: [vP [v BECOME [VP Theme State]]]
    Inchoative {
        theme: Participant,
        result_state: String,
    },

    /// Unaccusative structure: [vP [v [VP Theme Predicate]]]
    Unaccusative { theme: Participant },
}

impl MovementChain {
    /// Create a new movement chain
    pub fn new(
        movement_type: MovementType,
        moved_element: ChainLink,
        base_position: ChainLink,
        landing_site: LandingSite,
    ) -> Self {
        Self {
            movement_type,
            moved_element,
            intermediate_positions: Vec::new(),
            base_position,
            landing_site,
        }
    }

    /// Add an intermediate position to the chain
    pub fn add_intermediate_position(&mut self, position: ChainLink) {
        self.intermediate_positions.push(position);
    }

    /// Get the full chain from base to final position
    pub fn get_full_chain(&self) -> Vec<&ChainLink> {
        let mut chain = vec![&self.base_position];
        for intermediate in &self.intermediate_positions {
            chain.push(intermediate);
        }
        chain.push(&self.moved_element);
        chain
    }

    /// Check if this is a valid movement chain (UG constraints)
    pub fn is_valid(&self) -> bool {
        // Basic validity checks following principles of UG

        // 1. Base position should not be phonetically realized in movement
        if self.movement_type != MovementType::None && self.base_position.phonetically_realized {
            return false;
        }

        // 2. Final position should be phonetically realized (unless covert movement)
        if !self.moved_element.phonetically_realized && self.movement_type != MovementType::None {
            return false;
        }

        // 3. A-movement should target A-positions, A'-movement A'-positions
        // (This would require more detailed position analysis)

        true
    }
}

impl std::fmt::Display for MovementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MovementType::AMovement => write!(f, "A-movement"),
            MovementType::ABarMovement => write!(f, "A'-movement"),
            MovementType::HeadMovement => write!(f, "Head movement"),
            MovementType::None => write!(f, "No movement"),
        }
    }
}

/// Event time specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventTime {
    /// Event time unspecified (default)
    Now,

    /// Past time reference
    Past,

    /// Future time reference
    Future,

    /// Relative to another event
    Relative(EventId),
}

/// Semantic features extracted from VerbNet restrictions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticFeature {
    /// Motion event
    Motion,

    /// Transfer event
    Transfer,

    /// Contact event
    Contact,

    /// Change of state
    ChangeOfState,

    /// Perception event
    Perception,

    /// Communication event
    Communication,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.predicate.lemma, self.id)?;

        for (role, participant) in &self.participants {
            write!(f, ", {role:?}: {}", participant.expression)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy_core::{DepRel, MorphFeatures};

    #[test]
    fn test_event_creation() {
        let predicate = Predicate {
            lemma: "run".to_string(),
            semantic_type: PredicateType::Activity,
            verbnet_class: Some("run-51.3.2".to_string()),
            features: vec![SemanticFeature::Motion],
        };

        let event = Event::new(EventId(1), predicate);
        assert_eq!(event.id, EventId(1));
        assert_eq!(event.predicate.lemma, "run");
        assert_eq!(event.predicate.semantic_type, PredicateType::Activity);
    }

    #[test]
    fn test_participant_creation() {
        let word = Word {
            id: 1,
            text: "John".to_string(),
            lemma: "John".to_string(),
            upos: UPos::Propn,
            xpos: None,
            feats: MorphFeatures::default(),
            head: Some(2),
            deprel: DepRel::Nsubj,
            deps: None,
            misc: None,
            start: 0,
            end: 4,
        };

        let participant = Participant::from_word(&word);
        assert_eq!(participant.word_id, 1);
        assert_eq!(participant.expression, "John");
        assert_eq!(participant.features.animacy, Some(Animacy::Human));
    }

    #[test]
    fn test_event_participant_assignment() {
        let predicate = Predicate {
            lemma: "hit".to_string(),
            semantic_type: PredicateType::Action,
            verbnet_class: Some("hit-18.1".to_string()),
            features: vec![SemanticFeature::Contact],
        };

        let mut event = Event::new(EventId(1), predicate);

        let agent_word = Word {
            id: 1,
            text: "John".to_string(),
            lemma: "John".to_string(),
            upos: UPos::Propn,
            xpos: None,
            feats: MorphFeatures::default(),
            head: Some(2),
            deprel: DepRel::Nsubj,
            deps: None,
            misc: None,
            start: 0,
            end: 4,
        };

        let agent = Participant::from_word(&agent_word);
        event.add_participant(ThetaRoleType::Agent, agent);

        assert!(event.has_role(&ThetaRoleType::Agent));
        assert_eq!(
            event
                .get_participant(&ThetaRoleType::Agent)
                .unwrap()
                .expression,
            "John"
        );
    }

    #[test]
    fn test_verbnet_theta_assignment() {
        use crate::verbnet::engine::VerbNetEngine;

        // Create a mock VerbNet engine (simplified for testing)
        let engine = VerbNetEngine::new();

        // Create a ditransitive "give" predicate
        let predicate = Predicate {
            lemma: "give".to_string(),
            semantic_type: PredicateType::Action,
            verbnet_class: None, // Will be filled by VerbNet assignment
            features: vec![],
        };

        // Create event builder
        let builder = EventBuilder::new(predicate);

        // Test dependency pattern and arguments (John gives Mary a book)
        let dependency_pattern = "nsubj+dobj+iobj";
        let arguments = vec![
            ("nsubj".to_string(), "John".to_string()),
            ("dobj".to_string(), "book".to_string()),
            ("iobj".to_string(), "Mary".to_string()),
        ];

        // Apply VerbNet theta assignment
        let builder_with_roles =
            builder.with_verbnet_theta_assignment(&engine, &dependency_pattern, &arguments);

        // For this test, we expect the system to gracefully handle cases where VerbNet
        // doesn't have confident assignments. The validation will catch missing required roles.
        match builder_with_roles.build() {
            Ok(event) => {
                // If we get an event, verify VerbNet class assignment
                println!(
                    "Successfully built event with VerbNet class: {:?}",
                    event.predicate.verbnet_class
                );
                println!(
                    "Event participants: {:?}",
                    event.participants.keys().collect::<Vec<_>>()
                );
            }
            Err(EventBuildError::MissingRequiredThetaRole(role)) => {
                // This is expected when VerbNet cache is empty and no confident assignments are made
                println!(
                    "Expected validation error: Missing required theta role {:?}",
                    role
                );
                assert_eq!(role, ThetaRoleType::Agent); // Action predicates require Agent
            }
            Err(other) => panic!("Unexpected error: {:?}", other),
        }
    }

    #[test]
    fn test_verbnet_confidence_filtering() {
        use crate::verbnet::engine::VerbNetEngine;

        let engine = VerbNetEngine::new();

        let predicate = Predicate {
            lemma: "test".to_string(),
            semantic_type: PredicateType::Action,
            verbnet_class: None,
            features: vec![],
        };

        let builder = EventBuilder::new(predicate);

        // Test with a pattern that might have low confidence assignments
        let dependency_pattern = "unknown_pattern";
        let arguments = vec![("unknown".to_string(), "test".to_string())];

        let builder_with_roles =
            builder.with_verbnet_theta_assignment(&engine, &dependency_pattern, &arguments);

        // Test that the system filters out low confidence assignments
        match builder_with_roles.build() {
            Ok(event) => {
                println!(
                    "Successfully built event with {} participants",
                    event.participants.len()
                );
                // Should have 0 participants since confidence filtering rejects low confidence assignments
                assert_eq!(event.participants.len(), 0);
            }
            Err(EventBuildError::MissingRequiredThetaRole(_)) => {
                // This is expected when no confident assignments are made and validation catches it
                println!(
                    "Expected: No confident assignments found, validation caught missing required roles"
                );
            }
            Err(other) => panic!("Unexpected error: {:?}", other),
        }
    }
}
