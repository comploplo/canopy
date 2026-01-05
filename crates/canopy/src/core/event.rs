//! Event decomposition primitives.
//!
//! Based on Neo-Davidsonian event semantics (Parsons 1990) and
//! little v theory (Hale & Keyser 1993, Pylkkänen 2008).

use super::ThetaRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Little v types for event decomposition.
///
/// Each variant captures a distinct aspectual/causal flavor:
/// - CAUSE: external causation ("John broke the vase")
/// - BECOME: change of state ("The vase broke")
/// - BE: stative ("John is tall")
/// - DO: activities ("John ran")
/// - GO: motion ("John went home")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LittleV {
    /// CAUSE(causer, BECOME(theme, state))
    Cause {
        causer: Entity,
        caused: Box<LittleV>,
    },

    /// BECOME(theme, state)
    Become { theme: Entity, result: State },

    /// BE(theme, state)
    Be { theme: Entity, state: State },

    /// DO(agent, action)
    Do { agent: Entity, action: String },

    /// GO(theme, path)
    Go { theme: Entity, path: Path },

    /// HAVE(possessor, possessee)
    Have {
        possessor: Entity,
        possessee: Entity,
        kind: PossessionType,
    },

    /// EXPERIENCE(experiencer, stimulus)
    Experience {
        experiencer: Entity,
        stimulus: Entity,
        kind: PsychType,
    },
}

impl LittleV {
    /// Get the external argument (if any).
    #[must_use]
    pub fn external_argument(&self) -> Option<&Entity> {
        match self {
            LittleV::Cause { causer, .. } => Some(causer),
            LittleV::Do { agent, .. } => Some(agent),
            LittleV::Go { theme, .. } => Some(theme),
            LittleV::Have { possessor, .. } => Some(possessor),
            LittleV::Experience { experiencer, .. } => Some(experiencer),
            LittleV::Become { .. } | LittleV::Be { .. } => None,
        }
    }

    /// Check if this introduces an event variable.
    #[must_use]
    pub const fn is_eventive(&self) -> bool {
        !matches!(self, LittleV::Be { .. })
    }

    /// Get the aspectual class.
    #[must_use]
    pub const fn aspectual_class(&self) -> AspectualClass {
        match self {
            LittleV::Be { .. } | LittleV::Have { .. } | LittleV::Experience { .. } => {
                AspectualClass::State
            }
            LittleV::Do { .. } => AspectualClass::Activity,
            LittleV::Become { .. } => AspectualClass::Achievement,
            LittleV::Cause { .. } | LittleV::Go { .. } => AspectualClass::Accomplishment,
        }
    }
}

/// Entity reference for event participants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier.
    pub id: usize,
    /// Surface text.
    pub text: String,
    /// Semantic number.
    pub number: Option<SemanticNumber>,
    /// Distributivity for plurals.
    pub distributivity: Option<Distributivity>,
}

impl Entity {
    /// Create a new entity.
    #[must_use]
    pub fn new(id: usize, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
            number: None,
            distributivity: None,
        }
    }

    /// Create with semantic number.
    #[must_use]
    pub fn with_number(mut self, number: SemanticNumber) -> Self {
        self.number = Some(number);
        self
    }
}

/// State predication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub predicate: String,
    pub polarity: bool,
}

impl State {
    /// Create a positive state.
    #[must_use]
    pub fn positive(predicate: impl Into<String>) -> Self {
        Self {
            predicate: predicate.into(),
            polarity: true,
        }
    }

    /// Create a negative state.
    #[must_use]
    pub fn negative(predicate: impl Into<String>) -> Self {
        Self {
            predicate: predicate.into(),
            polarity: false,
        }
    }
}

/// Path for motion events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub source: Option<Entity>,
    pub goal: Option<Entity>,
    pub route: Option<Entity>,
}

impl Path {
    /// Create a path with just a goal.
    #[must_use]
    pub fn to_goal(goal: Entity) -> Self {
        Self {
            source: None,
            goal: Some(goal),
            route: None,
        }
    }

    /// Create a path from source to goal.
    #[must_use]
    pub fn from_to(source: Entity, goal: Entity) -> Self {
        Self {
            source: Some(source),
            goal: Some(goal),
            route: None,
        }
    }
}

/// Possession relationship types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PossessionType {
    /// Legal ownership.
    Ownership,
    /// Temporary possession.
    Temporary,
    /// Kinship relation.
    Kinship,
    /// Part-whole relation.
    PartWhole,
}

/// Psychological predicate types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PsychType {
    /// Subject-experiencer: "John fears spiders"
    SubjectExp,
    /// Object-experiencer: "Spiders frighten John"
    ObjectExp,
    /// Psych-state: "John is afraid"
    PsychState,
}

/// Aspectual classification (Vendler 1967).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AspectualClass {
    /// [-dynamic, -telic]: "know", "be tall"
    State,
    /// [+dynamic, -telic]: "run", "sing"
    Activity,
    /// [+dynamic, +telic, +durative]: "build a house"
    Accomplishment,
    /// [+dynamic, +telic, -durative]: "arrive", "die"
    Achievement,
}

/// Voice alternations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Voice {
    #[default]
    Active,
    Passive,
    Middle,
}

/// Semantic number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticNumber {
    Singular,
    Plural,
    Mass,
}

/// Distributivity for plural events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Distributivity {
    /// Single event: "The boys gathered"
    Collective,
    /// Multiple events: "The boys each ran"
    Distributive,
    /// Unspecified: "The boys laughed"
    Unspecified,
}

/// Modal force in Kratzerian semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModalForce {
    /// Universal: "must", "have to"
    Necessity,
    /// Existential: "can", "may"
    Possibility,
}

/// Modal flavor (conversational background).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModalFlavor {
    /// Knowledge-based: "He must be home"
    Epistemic,
    /// Obligation-based: "You must pay"
    Deontic,
    /// Ability-based: "She can swim"
    Circumstantial,
    /// Desire-based: "I want to go"
    Bouletic,
    /// Goal-based: "To win, you must train"
    Teleological,
}

/// Combined modality for events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventModality {
    pub force: ModalForce,
    pub flavor: ModalFlavor,
    pub auxiliary: Option<String>,
}

impl EventModality {
    /// Create a new event modality.
    #[must_use]
    pub fn new(force: ModalForce, flavor: ModalFlavor) -> Self {
        Self {
            force,
            flavor,
            auxiliary: None,
        }
    }
}

/// Proposition for embedded content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposition {
    pub content: String,
    pub polarity: bool,
}

/// Action for DO events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub predicate: String,
    pub manner: Option<String>,
}

/// Full event structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: usize,
    pub predicate: String,
    pub little_v: LittleV,
    pub participants: HashMap<ThetaRole, Entity>,
    pub aspect: AspectualClass,
    pub voice: Voice,
    pub modality: Option<EventModality>,
}

impl Event {
    /// Create a new event.
    #[must_use]
    pub fn new(id: usize, predicate: impl Into<String>, little_v: LittleV) -> Self {
        let aspect = little_v.aspectual_class();
        Self {
            id,
            predicate: predicate.into(),
            little_v,
            participants: HashMap::new(),
            aspect,
            voice: Voice::default(),
            modality: None,
        }
    }

    /// Add a participant.
    pub fn add_participant(&mut self, role: ThetaRole, entity: Entity) {
        self.participants.insert(role, entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_little_v_cause() {
        let john = Entity::new(1, "John");
        let vase = Entity::new(2, "the vase");
        let broken = State::positive("broken");

        let inner = LittleV::Become {
            theme: vase,
            result: broken,
        };
        let cause = LittleV::Cause {
            causer: john.clone(),
            caused: Box::new(inner),
        };

        assert_eq!(
            cause.external_argument().map(|e| &e.text),
            Some(&"John".to_string())
        );
        assert!(cause.is_eventive());
        assert_eq!(cause.aspectual_class(), AspectualClass::Accomplishment);
    }

    #[test]
    fn test_entity() {
        let e = Entity::new(1, "the book").with_number(SemanticNumber::Singular);
        assert_eq!(e.number, Some(SemanticNumber::Singular));
    }

    #[test]
    fn test_path() {
        let boston = Entity::new(1, "Boston");
        let nyc = Entity::new(2, "New York");
        let path = Path::from_to(boston, nyc);
        assert!(path.source.is_some());
        assert!(path.goal.is_some());
    }

    #[test]
    fn test_event() {
        let john = Entity::new(1, "John");
        let little_v = LittleV::Do {
            agent: john.clone(),
            action: "run".to_string(),
        };
        let mut event = Event::new(1, "run", little_v);
        event.add_participant(ThetaRole::Agent, john);

        assert_eq!(event.participants.len(), 1);
        assert_eq!(event.aspect, AspectualClass::Activity);
    }

    #[test]
    fn test_state() {
        let pos = State::positive("happy");
        assert!(pos.polarity);
        assert_eq!(pos.predicate, "happy");

        let neg = State::negative("sad");
        assert!(!neg.polarity);
    }

    #[test]
    fn test_path_to_goal() {
        let nyc = Entity::new(1, "New York");
        let path = Path::to_goal(nyc);
        assert!(path.goal.is_some());
        assert!(path.source.is_none());
        assert!(path.route.is_none());
    }

    #[test]
    fn test_little_v_become() {
        let vase = Entity::new(1, "the vase");
        let broken = State::positive("broken");
        let become_event = LittleV::Become {
            theme: vase,
            result: broken,
        };

        assert!(become_event.external_argument().is_none());
        assert!(become_event.is_eventive());
        assert_eq!(become_event.aspectual_class(), AspectualClass::Achievement);
    }

    #[test]
    fn test_little_v_be() {
        let john = Entity::new(1, "John");
        let tall = State::positive("tall");
        let be = LittleV::Be {
            theme: john,
            state: tall,
        };

        assert!(be.external_argument().is_none());
        assert!(!be.is_eventive());
        assert_eq!(be.aspectual_class(), AspectualClass::State);
    }

    #[test]
    fn test_little_v_go() {
        let john = Entity::new(1, "John");
        let boston = Entity::new(2, "Boston");
        let path = Path::to_goal(boston);
        let go = LittleV::Go {
            theme: john.clone(),
            path,
        };

        assert_eq!(
            go.external_argument().map(|e| &e.text),
            Some(&"John".to_string())
        );
        assert!(go.is_eventive());
        assert_eq!(go.aspectual_class(), AspectualClass::Accomplishment);
    }

    #[test]
    fn test_little_v_have() {
        let john = Entity::new(1, "John");
        let book = Entity::new(2, "a book");
        let have = LittleV::Have {
            possessor: john.clone(),
            possessee: book,
            kind: PossessionType::Ownership,
        };

        assert_eq!(
            have.external_argument().map(|e| &e.text),
            Some(&"John".to_string())
        );
        assert!(have.is_eventive()); // Have is still eventive (has event variable)
        assert_eq!(have.aspectual_class(), AspectualClass::State);
    }

    #[test]
    fn test_little_v_experience() {
        let john = Entity::new(1, "John");
        let spiders = Entity::new(2, "spiders");
        let exp = LittleV::Experience {
            experiencer: john.clone(),
            stimulus: spiders,
            kind: PsychType::SubjectExp,
        };

        assert_eq!(
            exp.external_argument().map(|e| &e.text),
            Some(&"John".to_string())
        );
        assert!(exp.is_eventive()); // Experience is eventive (has event variable)
        assert_eq!(exp.aspectual_class(), AspectualClass::State);
    }

    #[test]
    fn test_event_modality() {
        let modality = EventModality::new(ModalForce::Necessity, ModalFlavor::Epistemic);
        assert_eq!(modality.force, ModalForce::Necessity);
        assert_eq!(modality.flavor, ModalFlavor::Epistemic);
        assert!(modality.auxiliary.is_none());
    }

    #[test]
    fn test_psych_type_variants() {
        assert_eq!(format!("{:?}", PsychType::SubjectExp), "SubjectExp");
        assert_eq!(format!("{:?}", PsychType::ObjectExp), "ObjectExp");
        assert_eq!(format!("{:?}", PsychType::PsychState), "PsychState");
    }

    #[test]
    fn test_possession_type_variants() {
        assert_eq!(format!("{:?}", PossessionType::Ownership), "Ownership");
        assert_eq!(format!("{:?}", PossessionType::Temporary), "Temporary");
        assert_eq!(format!("{:?}", PossessionType::Kinship), "Kinship");
        assert_eq!(format!("{:?}", PossessionType::PartWhole), "PartWhole");
    }

    #[test]
    fn test_modal_force_variants() {
        assert_eq!(format!("{:?}", ModalForce::Necessity), "Necessity");
        assert_eq!(format!("{:?}", ModalForce::Possibility), "Possibility");
    }

    #[test]
    fn test_modal_flavor_variants() {
        assert_eq!(format!("{:?}", ModalFlavor::Epistemic), "Epistemic");
        assert_eq!(format!("{:?}", ModalFlavor::Deontic), "Deontic");
        assert_eq!(
            format!("{:?}", ModalFlavor::Circumstantial),
            "Circumstantial"
        );
        assert_eq!(format!("{:?}", ModalFlavor::Bouletic), "Bouletic");
        assert_eq!(format!("{:?}", ModalFlavor::Teleological), "Teleological");
    }

    #[test]
    fn test_distributivity_variants() {
        assert_eq!(format!("{:?}", Distributivity::Collective), "Collective");
        assert_eq!(
            format!("{:?}", Distributivity::Distributive),
            "Distributive"
        );
        assert_eq!(format!("{:?}", Distributivity::Unspecified), "Unspecified");
    }

    #[test]
    fn test_aspectual_class_variants() {
        assert_eq!(format!("{:?}", AspectualClass::State), "State");
        assert_eq!(format!("{:?}", AspectualClass::Activity), "Activity");
        assert_eq!(format!("{:?}", AspectualClass::Achievement), "Achievement");
        assert_eq!(
            format!("{:?}", AspectualClass::Accomplishment),
            "Accomplishment"
        );
    }
}
