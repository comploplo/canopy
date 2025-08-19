//! Little v decomposition following Larson and Hale & Keyser
//!
//! This module implements event decomposition using little v heads:
//! - Larson (1988): "On the double object construction"
//! - Hale & Keyser (1993): "On argument structure and the lexical expression of syntactic relations"
//! - Kratzer (1996): "Severing the external argument from its verb"
//! - Pylkkänen (2002): "Introducing arguments"
//!
//! The basic insight is that verbs are decomposed into functional heads
//! that introduce semantic primitives: CAUSE, BECOME, DO, BE, GO, HAVE.

use crate::ThetaRoleType;
use crate::events::{Event, Participant};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Little v head types following semantic decomposition literature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LittleVType {
    /// CAUSE: introduces external causation
    /// - Semantics: [x CAUSE [y BECOME z]]
    /// - Example: "John broke the vase" = John CAUSE [vase BECOME broken]
    Cause,

    /// BECOME: introduces change of state
    /// - Semantics: [x BECOME y]
    /// - Example: "The ice melted" = ice BECOME [liquid]
    Become,

    /// DO: introduces agentive activity
    /// - Semantics: [x DO y]
    /// - Example: "John danced" = John DO [dance]
    Do,

    /// BE: introduces states
    /// - Semantics: [x BE y]
    /// - Example: "John is tall" = John BE [tall]
    Be,

    /// GO: introduces directed motion
    /// - Semantics: [x GO Path]
    /// - Example: "John went home" = John GO [to home]
    Go,

    /// HAVE: introduces possession/containment
    /// - Semantics: [x HAVE y]
    /// - Example: "John has a book" = John HAVE [book]
    Have,

    /// GET: introduces acquisition or achievement
    /// - Semantics: [x GET y] (= x CAUSE [x HAVE y])
    /// - Example: "John got a book" = John GET [book]
    Get,

    /// PUT: introduces causation of location
    /// - Semantics: [x PUT y LOC] (= x CAUSE [y BE-AT LOC])
    /// - Example: "John put the book on the table"
    Put,
}

/// Little v shell structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LittleVShell {
    /// The little v head
    pub v_head: LittleVType,

    /// External argument (specifier of vP)
    pub external_argument: Option<Participant>,

    /// Complement (VP or another vP)
    pub complement: VPComplement,

    /// Semantic features of this v head
    pub features: LittleVFeatures,
}

/// VP complement of little v
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VPComplement {
    /// Simple VP with verb and internal arguments
    SimpleVP {
        verb: String,
        internal_arguments: Vec<Participant>,
    },

    /// Nested vP (for complex decomposition)
    NestedVP(Box<LittleVShell>),

    /// Resultative phrase
    ResultativePhrase {
        result_predicate: String,
        result_argument: Participant,
    },

    /// Prepositional phrase (for locative/directional)
    PrepositionalPhrase {
        preposition: String,
        object: Participant,
    },
}

/// Semantic features of little v heads
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LittleVFeatures {
    /// Does this v introduce an external argument?
    pub introduces_external_arg: bool,

    /// Is this v causative?
    pub causative: bool,

    /// Is this v inchoative (change of state)?
    pub inchoative: bool,

    /// Is this v stative?
    pub stative: bool,

    /// Voice features
    pub voice: Option<VoiceFeatures>,
}

/// Voice features following Kratzer (1996)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceFeatures {
    /// Voice type
    pub voice_type: VoiceType,

    /// Does Voice introduce an external argument?
    pub introduces_agent: bool,

    /// Passive morphology present?
    pub passive_morphology: bool,
}

/// Voice types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceType {
    /// Active voice (agent in subject position)
    Active,

    /// Passive voice (agent in by-phrase or suppressed)
    Passive,

    /// Middle voice (agent suppressed, no passive morphology)
    Middle,

    /// Reflexive voice
    Reflexive,

    /// Reciprocal voice
    Reciprocal,
}

impl LittleVType {
    /// Get default semantic features for this v type
    pub fn default_features(&self) -> LittleVFeatures {
        match self {
            LittleVType::Cause => LittleVFeatures {
                introduces_external_arg: true,
                causative: true,
                inchoative: false,
                stative: false,
                voice: Some(VoiceFeatures {
                    voice_type: VoiceType::Active,
                    introduces_agent: true,
                    passive_morphology: false,
                }),
            },

            LittleVType::Become => LittleVFeatures {
                introduces_external_arg: false,
                causative: false,
                inchoative: true,
                stative: false,
                voice: None, // Inchoatives often lack external arguments
            },

            LittleVType::Do => LittleVFeatures {
                introduces_external_arg: true,
                causative: false,
                inchoative: false,
                stative: false,
                voice: Some(VoiceFeatures {
                    voice_type: VoiceType::Active,
                    introduces_agent: true,
                    passive_morphology: false,
                }),
            },

            LittleVType::Be => LittleVFeatures {
                introduces_external_arg: false, // Statives often lack agents
                causative: false,
                inchoative: false,
                stative: true,
                voice: None,
            },

            LittleVType::Go => LittleVFeatures {
                introduces_external_arg: true,
                causative: false,
                inchoative: false,
                stative: false,
                voice: Some(VoiceFeatures {
                    voice_type: VoiceType::Active,
                    introduces_agent: false, // Themes can move themselves
                    passive_morphology: false,
                }),
            },

            LittleVType::Have => LittleVFeatures {
                introduces_external_arg: true,
                causative: false,
                inchoative: false,
                stative: true,
                voice: Some(VoiceFeatures {
                    voice_type: VoiceType::Active,
                    introduces_agent: false, // Possessors aren't agents
                    passive_morphology: false,
                }),
            },

            LittleVType::Get => LittleVFeatures {
                introduces_external_arg: true,
                causative: true, // GET = CAUSE + HAVE
                inchoative: true,
                stative: false,
                voice: Some(VoiceFeatures {
                    voice_type: VoiceType::Active,
                    introduces_agent: true,
                    passive_morphology: false,
                }),
            },

            LittleVType::Put => LittleVFeatures {
                introduces_external_arg: true,
                causative: true, // PUT = CAUSE + BE-AT
                inchoative: false,
                stative: false,
                voice: Some(VoiceFeatures {
                    voice_type: VoiceType::Active,
                    introduces_agent: true,
                    passive_morphology: false,
                }),
            },
        }
    }

    /// Check if this v type can take a specific complement type
    pub fn compatible_with_complement(&self, complement: &VPComplement) -> bool {
        match (self, complement) {
            // CAUSE can take any complement
            (LittleVType::Cause, _) => true,

            // BECOME typically takes resultative phrases
            (LittleVType::Become, VPComplement::ResultativePhrase { .. }) => true,
            (LittleVType::Become, VPComplement::SimpleVP { .. }) => true,

            // DO takes activity VPs
            (LittleVType::Do, VPComplement::SimpleVP { .. }) => true,

            // BE takes predicative complements
            (LittleVType::Be, VPComplement::ResultativePhrase { .. }) => true,
            (LittleVType::Be, VPComplement::PrepositionalPhrase { .. }) => true,

            // GO takes directional phrases
            (LittleVType::Go, VPComplement::PrepositionalPhrase { .. }) => true,

            // HAVE takes simple objects
            (LittleVType::Have, VPComplement::SimpleVP { .. }) => true,

            // GET and PUT are complex
            (LittleVType::Get, _) => true,
            (LittleVType::Put, _) => true,

            _ => false,
        }
    }

    /// Get the theta roles typically associated with this v type
    pub fn typical_theta_roles(&self) -> Vec<ThetaRoleType> {
        match self {
            LittleVType::Cause => vec![ThetaRoleType::Agent, ThetaRoleType::Patient],
            LittleVType::Become => vec![ThetaRoleType::Theme],
            LittleVType::Do => vec![ThetaRoleType::Agent],
            LittleVType::Be => vec![ThetaRoleType::Theme],
            LittleVType::Go => vec![ThetaRoleType::Theme, ThetaRoleType::Goal],
            LittleVType::Have => vec![ThetaRoleType::Experiencer, ThetaRoleType::Theme],
            LittleVType::Get => vec![ThetaRoleType::Agent, ThetaRoleType::Theme],
            LittleVType::Put => vec![
                ThetaRoleType::Agent,
                ThetaRoleType::Theme,
                ThetaRoleType::Location,
            ],
        }
    }
}

/// Event decomposer using little v theory
#[derive(Debug)]
pub struct EventDecomposer {
    /// Decomposition rules for different verb types
    decomposition_rules: Vec<DecompositionRule>,
}

impl EventDecomposer {
    /// Create new event decomposer with default rules
    pub fn new() -> Self {
        Self {
            decomposition_rules: Self::default_decomposition_rules(),
        }
    }

    /// Decompose an event into little v structure
    pub fn decompose_event(&self, event: &Event) -> Option<LittleVShell> {
        // Find appropriate decomposition rule
        for rule in &self.decomposition_rules {
            if rule.matches_event(event) {
                return rule.apply_decomposition(event);
            }
        }

        // Default simple decomposition
        self.default_decomposition(event)
    }

    /// Default decomposition rules based on verb classes
    fn default_decomposition_rules() -> Vec<DecompositionRule> {
        vec![
            // Causative alternation: "John broke the vase" (causative side)
            DecompositionRule {
                pattern: VerbPattern::CausativeAlternation,
                v_structure: vec![LittleVType::Cause, LittleVType::Become],
                conditions: vec![
                    DecompositionCondition::HasTheme,
                    DecompositionCondition::HasAgent, // Must have agent for causative
                ],
            },
            // Transitive causative: "John broke the vase" (with Patient)
            DecompositionRule {
                pattern: VerbPattern::CausativeAlternation,
                v_structure: vec![LittleVType::Cause, LittleVType::Become],
                conditions: vec![
                    DecompositionCondition::HasPatient,
                    DecompositionCondition::HasAgent,
                ],
            },
            // Unaccusative: "The ice melted"
            DecompositionRule {
                pattern: VerbPattern::Unaccusative,
                v_structure: vec![LittleVType::Become],
                conditions: vec![
                    DecompositionCondition::HasTheme,
                    DecompositionCondition::NoAgent,
                ],
            },
            // Activity verbs: "John ran"
            DecompositionRule {
                pattern: VerbPattern::Activity,
                v_structure: vec![LittleVType::Do],
                conditions: vec![DecompositionCondition::HasAgent],
            },
            // Accomplishment: "John built a house"
            DecompositionRule {
                pattern: VerbPattern::Accomplishment,
                v_structure: vec![LittleVType::Cause, LittleVType::Become],
                conditions: vec![
                    DecompositionCondition::HasAgent,
                    DecompositionCondition::HasTheme,
                ],
            },
            // Motion verbs: "John went home"
            DecompositionRule {
                pattern: VerbPattern::Motion,
                v_structure: vec![LittleVType::Go],
                conditions: vec![
                    DecompositionCondition::HasTheme,
                    DecompositionCondition::HasGoal,
                ],
            },
            // Ditransitive: "John gave Mary a book"
            DecompositionRule {
                pattern: VerbPattern::Ditransitive,
                v_structure: vec![LittleVType::Cause, LittleVType::Have],
                conditions: vec![
                    DecompositionCondition::HasAgent,
                    DecompositionCondition::HasTheme,
                    DecompositionCondition::HasRecipient,
                ],
            },
        ]
    }

    /// Apply default simple decomposition
    fn default_decomposition(&self, event: &Event) -> Option<LittleVShell> {
        // Simple heuristic based on participant structure
        let has_agent = event.has_role(&ThetaRoleType::Agent);
        let has_theme = event.has_role(&ThetaRoleType::Theme);
        let has_patient = event.has_role(&ThetaRoleType::Patient);

        let v_type = if has_agent && (has_theme || has_patient) {
            LittleVType::Cause // Transitive causative
        } else if has_agent {
            LittleVType::Do // Intransitive agentive
        } else if has_theme || has_patient {
            LittleVType::Become // Unaccusative
        } else {
            LittleVType::Be // Default stative
        };

        let external_arg = event.get_participant(&ThetaRoleType::Agent).cloned();

        let internal_args: Vec<Participant> = event
            .participants
            .iter()
            .filter_map(|(role, participant)| {
                if *role != ThetaRoleType::Agent {
                    Some(participant.clone())
                } else {
                    None
                }
            })
            .collect();

        Some(LittleVShell {
            v_head: v_type.clone(),
            external_argument: external_arg,
            complement: VPComplement::SimpleVP {
                verb: event.predicate.lemma.clone(),
                internal_arguments: internal_args,
            },
            features: v_type.default_features(),
        })
    }
}

/// Decomposition rule for specific verb patterns
#[derive(Debug, Clone)]
pub struct DecompositionRule {
    /// Pattern this rule matches
    pub pattern: VerbPattern,

    /// Little v structure to build
    pub v_structure: Vec<LittleVType>,

    /// Conditions that must be met
    pub conditions: Vec<DecompositionCondition>,
}

impl DecompositionRule {
    /// Check if this rule matches an event
    pub fn matches_event(&self, event: &Event) -> bool {
        self.conditions
            .iter()
            .all(|condition| condition.check(event))
    }

    /// Apply decomposition to create little v structure
    pub fn apply_decomposition(&self, event: &Event) -> Option<LittleVShell> {
        if self.v_structure.is_empty() {
            return None;
        }

        // For now, create simple structure with first v head
        // In full implementation, this would build complex nested structures
        let v_type = &self.v_structure[0];
        let external_arg = event.get_participant(&ThetaRoleType::Agent).cloned();

        let internal_args: Vec<Participant> = event
            .participants
            .iter()
            .filter_map(|(role, participant)| {
                if *role != ThetaRoleType::Agent {
                    Some(participant.clone())
                } else {
                    None
                }
            })
            .collect();

        Some(LittleVShell {
            v_head: v_type.clone(),
            external_argument: external_arg,
            complement: VPComplement::SimpleVP {
                verb: event.predicate.lemma.clone(),
                internal_arguments: internal_args,
            },
            features: v_type.default_features(),
        })
    }
}

/// Verb patterns for decomposition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerbPattern {
    /// Causative/inchoative alternation
    CausativeAlternation,

    /// Unaccusative verbs
    Unaccusative,

    /// Activity verbs
    Activity,

    /// Accomplishment verbs
    Accomplishment,

    /// Motion verbs
    Motion,

    /// Ditransitive verbs
    Ditransitive,

    /// Stative verbs
    Stative,
}

/// Conditions for decomposition rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecompositionCondition {
    /// Must have Agent role
    HasAgent,

    /// Must not have Agent role
    NoAgent,

    /// May have Agent role (optional)
    MayHaveAgent,

    /// Must have Theme role
    HasTheme,

    /// Must have Patient role
    HasPatient,

    /// Must have Goal role
    HasGoal,

    /// Must have Recipient role
    HasRecipient,

    /// Must have Location role
    HasLocation,
}

impl DecompositionCondition {
    /// Check if condition is satisfied by event
    pub fn check(&self, event: &Event) -> bool {
        match self {
            DecompositionCondition::HasAgent => event.has_role(&ThetaRoleType::Agent),
            DecompositionCondition::NoAgent => !event.has_role(&ThetaRoleType::Agent),
            DecompositionCondition::MayHaveAgent => true, // Always satisfied
            DecompositionCondition::HasTheme => event.has_role(&ThetaRoleType::Theme),
            DecompositionCondition::HasPatient => event.has_role(&ThetaRoleType::Patient),
            DecompositionCondition::HasGoal => event.has_role(&ThetaRoleType::Goal),
            DecompositionCondition::HasRecipient => event.has_role(&ThetaRoleType::Recipient),
            DecompositionCondition::HasLocation => event.has_role(&ThetaRoleType::Location),
        }
    }
}

impl fmt::Display for LittleVShell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[vP ")?;

        if let Some(ext_arg) = &self.external_argument {
            write!(f, "{} ", ext_arg.expression)?;
        }

        write!(f, "[v {:?} ", self.v_head)?;

        match &self.complement {
            VPComplement::SimpleVP {
                verb,
                internal_arguments,
            } => {
                write!(f, "[VP {verb} ")?;
                for arg in internal_arguments {
                    write!(f, "{} ", arg.expression)?;
                }
                write!(f, "]")?;
            }
            VPComplement::NestedVP(nested) => {
                write!(f, "{nested}")?;
            }
            VPComplement::ResultativePhrase {
                result_predicate,
                result_argument,
            } => {
                write!(
                    f,
                    "[ResP {} {}]",
                    result_argument.expression, result_predicate
                )?;
            }
            VPComplement::PrepositionalPhrase {
                preposition,
                object,
            } => {
                write!(f, "[PP {} {}]", preposition, object.expression)?;
            }
        }

        write!(f, "]]")
    }
}

impl Default for EventDecomposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{
        AspectualClass, Event, EventId, EventTime, Predicate, PredicateType, SemanticFeature,
    };
    use std::collections::HashMap;

    fn create_test_participant(name: &str, id: usize) -> Participant {
        use canopy_core::{DepRel, MorphFeatures, UPos, Word};
        let word = Word {
            id,
            text: name.to_string(),
            lemma: name.to_string(),
            upos: UPos::Noun,
            xpos: None,
            feats: MorphFeatures::default(),
            head: Some(0),
            deprel: DepRel::Root,
            deps: None,
            misc: None,
            start: 0,
            end: name.len(),
        };
        Participant::from_word(&word)
    }

    #[test]
    fn test_little_v_features() {
        let cause_features = LittleVType::Cause.default_features();
        assert!(cause_features.introduces_external_arg);
        assert!(cause_features.causative);
        assert!(!cause_features.stative);

        let become_features = LittleVType::Become.default_features();
        assert!(!become_features.introduces_external_arg);
        assert!(!become_features.causative);
        assert!(become_features.inchoative);

        let be_features = LittleVType::Be.default_features();
        assert!(!be_features.introduces_external_arg);
        assert!(be_features.stative);
    }

    #[test]
    fn test_event_decomposition_transitive() {
        // Test "John broke the vase"
        let predicate = Predicate {
            lemma: "break".to_string(),
            semantic_type: PredicateType::Causative,
            verbnet_class: Some("break-45.1".to_string()),
            features: vec![SemanticFeature::ChangeOfState],
        };

        let mut event = Event {
            id: EventId(1),
            predicate,
            participants: HashMap::new(),
            modifiers: Vec::new(),
            aspect: AspectualClass::Achievement,
            structure: None,
            time: EventTime::Now,
            movement_chains: Vec::new(),
            little_v: None,
        };

        // Add participants
        event.add_participant(ThetaRoleType::Agent, create_test_participant("John", 1));
        event.add_participant(ThetaRoleType::Patient, create_test_participant("vase", 2));

        let decomposer = EventDecomposer::new();
        let shell = decomposer.decompose_event(&event).unwrap();

        assert_eq!(shell.v_head, LittleVType::Cause);
        assert!(shell.external_argument.is_some());
        assert_eq!(shell.external_argument.as_ref().unwrap().expression, "John");

        if let VPComplement::SimpleVP {
            verb,
            internal_arguments,
        } = &shell.complement
        {
            assert_eq!(verb, "break");
            assert_eq!(internal_arguments.len(), 1);
            assert_eq!(internal_arguments[0].expression, "vase");
        }
    }

    #[test]
    fn test_event_decomposition_unaccusative() {
        // Test "The ice melted"
        let predicate = Predicate {
            lemma: "melt".to_string(),
            semantic_type: PredicateType::Inchoative,
            verbnet_class: Some("melt-45.5".to_string()),
            features: vec![SemanticFeature::ChangeOfState],
        };

        let mut event = Event {
            id: EventId(1),
            predicate,
            participants: HashMap::new(),
            modifiers: Vec::new(),
            aspect: AspectualClass::Achievement,
            structure: None,
            time: EventTime::Now,
            movement_chains: Vec::new(),
            little_v: None,
        };

        // Add only Theme (no Agent for unaccusative)
        event.add_participant(ThetaRoleType::Theme, create_test_participant("ice", 1));

        let decomposer = EventDecomposer::new();
        let shell = decomposer.decompose_event(&event).unwrap();

        assert_eq!(shell.v_head, LittleVType::Become);
        assert!(shell.external_argument.is_none()); // No external argument
    }

    #[test]
    fn test_decomposition_rule_matching() {
        let rule = DecompositionRule {
            pattern: VerbPattern::CausativeAlternation,
            v_structure: vec![LittleVType::Cause, LittleVType::Become],
            conditions: vec![
                DecompositionCondition::HasTheme,
                DecompositionCondition::MayHaveAgent,
            ],
        };

        let predicate = Predicate {
            lemma: "break".to_string(),
            semantic_type: PredicateType::Causative,
            verbnet_class: None,
            features: vec![],
        };

        let mut event = Event {
            id: EventId(1),
            predicate,
            participants: HashMap::new(),
            modifiers: Vec::new(),
            aspect: AspectualClass::Achievement,
            structure: None,
            time: EventTime::Now,
            movement_chains: Vec::new(),
            little_v: None,
        };

        event.add_participant(ThetaRoleType::Theme, create_test_participant("vase", 1));

        assert!(rule.matches_event(&event));

        // Add agent - should still match (MayHaveAgent)
        event.add_participant(ThetaRoleType::Agent, create_test_participant("John", 2));
        assert!(rule.matches_event(&event));
    }

    #[test]
    fn test_theta_role_mapping() {
        let cause_roles = LittleVType::Cause.typical_theta_roles();
        assert!(cause_roles.contains(&ThetaRoleType::Agent));
        assert!(cause_roles.contains(&ThetaRoleType::Patient));

        let go_roles = LittleVType::Go.typical_theta_roles();
        assert!(go_roles.contains(&ThetaRoleType::Theme));
        assert!(go_roles.contains(&ThetaRoleType::Goal));

        let have_roles = LittleVType::Have.typical_theta_roles();
        assert!(have_roles.contains(&ThetaRoleType::Experiencer));
        assert!(have_roles.contains(&ThetaRoleType::Theme));
    }

    #[test]
    fn test_complement_compatibility() {
        let become_complement = VPComplement::ResultativePhrase {
            result_predicate: "broken".to_string(),
            result_argument: create_test_participant("vase", 1),
        };

        assert!(LittleVType::Become.compatible_with_complement(&become_complement));
        assert!(LittleVType::Be.compatible_with_complement(&become_complement));
        assert!(!LittleVType::Go.compatible_with_complement(&become_complement));

        let pp_complement = VPComplement::PrepositionalPhrase {
            preposition: "to".to_string(),
            object: create_test_participant("home", 1),
        };

        assert!(LittleVType::Go.compatible_with_complement(&pp_complement));
        assert!(LittleVType::Be.compatible_with_complement(&pp_complement));
    }

    #[test]
    fn test_shell_display() {
        let shell = LittleVShell {
            v_head: LittleVType::Cause,
            external_argument: Some(create_test_participant("John", 1)),
            complement: VPComplement::SimpleVP {
                verb: "break".to_string(),
                internal_arguments: vec![create_test_participant("vase", 2)],
            },
            features: LittleVType::Cause.default_features(),
        };

        let display = format!("{}", shell);
        assert!(display.contains("John"));
        assert!(display.contains("Cause"));
        assert!(display.contains("break"));
        assert!(display.contains("vase"));
    }
}
