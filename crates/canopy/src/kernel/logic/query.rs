//! Query types for logical reasoning over DRS.
//!
//! Defines the different types of queries that can be answered against
//! a discourse representation.

use crate::core::ThetaRole;
use crate::kernel::discourse::{QudIssue, ReferentId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A query against a DRS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Query {
    /// Yes/no question: "Did John leave?"
    YesNo {
        /// The proposition to check.
        proposition: Proposition,
    },

    /// Wh-question: "Who left?"
    WhQuestion {
        /// The predicate to search for.
        predicate: String,
        /// The role to extract (who = Agent, what = Theme, etc.).
        target_role: ThetaRole,
        /// Additional constraints on the query.
        constraints: Vec<Constraint>,
    },

    /// What-happened question: "What did John do?"
    WhatHappened {
        /// The agent to query about (if any).
        agent: Option<String>,
    },

    /// Existence check: "Is there a book?"
    Exists {
        /// Description/predicate to check for.
        predicate: String,
    },
}

impl Query {
    /// Create a yes/no query for a simple predicate with one participant.
    #[must_use]
    pub fn yes_no(
        predicate: impl Into<String>,
        entity: impl Into<String>,
        role: ThetaRole,
    ) -> Self {
        let mut participants = HashMap::new();
        participants.insert(role, Term::Constant(entity.into()));
        Self::YesNo {
            proposition: Proposition {
                predicate: predicate.into(),
                participants,
                polarity: true,
            },
        }
    }

    /// Create a yes/no query with multiple participants.
    #[must_use]
    pub fn yes_no_full(proposition: Proposition) -> Self {
        Self::YesNo { proposition }
    }

    /// Create a wh-question.
    #[must_use]
    pub fn wh(predicate: impl Into<String>, target_role: ThetaRole) -> Self {
        Self::WhQuestion {
            predicate: predicate.into(),
            target_role,
            constraints: Vec::new(),
        }
    }

    /// Create a wh-question with constraints.
    #[must_use]
    pub fn wh_with_constraints(
        predicate: impl Into<String>,
        target_role: ThetaRole,
        constraints: Vec<Constraint>,
    ) -> Self {
        Self::WhQuestion {
            predicate: predicate.into(),
            target_role,
            constraints,
        }
    }

    /// Create a what-happened query.
    #[must_use]
    pub fn what_happened(agent: Option<String>) -> Self {
        Self::WhatHappened { agent }
    }

    /// Create an existence query.
    #[must_use]
    pub fn exists(predicate: impl Into<String>) -> Self {
        Self::Exists {
            predicate: predicate.into(),
        }
    }
}

/// A proposition that can be checked against the DRS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposition {
    /// The event predicate (e.g., "leave", "give").
    pub predicate: String,
    /// Participants mapped by theta role.
    pub participants: HashMap<ThetaRole, Term>,
    /// Polarity (true = affirmed, false = negated).
    pub polarity: bool,
}

impl Proposition {
    /// Create a new proposition.
    #[must_use]
    pub fn new(predicate: impl Into<String>, polarity: bool) -> Self {
        Self {
            predicate: predicate.into(),
            participants: HashMap::new(),
            polarity,
        }
    }

    /// Add a participant.
    #[must_use]
    pub fn with_participant(mut self, role: ThetaRole, term: Term) -> Self {
        self.participants.insert(role, term);
        self
    }

    /// Create a simple proposition with one participant.
    #[must_use]
    pub fn simple(
        predicate: impl Into<String>,
        role: ThetaRole,
        entity: impl Into<String>,
    ) -> Self {
        Self::new(predicate, true).with_participant(role, Term::Constant(entity.into()))
    }

    /// Negate this proposition.
    #[must_use]
    pub fn negated(mut self) -> Self {
        self.polarity = !self.polarity;
        self
    }
}

/// A term in a query (constant, variable, or referent).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Term {
    /// A constant value (e.g., "John", "book").
    Constant(String),
    /// A variable to be bound (e.g., "?x", "?who").
    Variable(String),
    /// A known discourse referent.
    ReferentId(ReferentId),
}

impl Term {
    /// Check if this term is a variable.
    #[must_use]
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }

    /// Check if this term is a constant.
    #[must_use]
    pub fn is_constant(&self) -> bool {
        matches!(self, Self::Constant(_))
    }

    /// Get the constant value if this is a constant.
    #[must_use]
    pub fn as_constant(&self) -> Option<&str> {
        match self {
            Self::Constant(s) => Some(s),
            _ => None,
        }
    }

    /// Get the variable name if this is a variable.
    #[must_use]
    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Self::Variable(s) => Some(s),
            _ => None,
        }
    }
}

/// A constraint on a query (for filtering results).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    /// Role must have a specific value.
    RoleEquals { role: ThetaRole, value: String },
    /// Role must match a referent.
    RoleMatchesReferent {
        role: ThetaRole,
        referent: ReferentId,
    },
    /// Predicate must be one of these.
    PredicateIn { predicates: Vec<String> },
}

/// Convert a QUD issue to a Query.
///
/// Maps wh-words to appropriate target roles:
/// - "who" → Agent (or Experiencer)
/// - "what" → Theme (or Patient)
/// - "where" → Location
/// - "when" → Time (not yet supported)
/// - "why" → Cause (not yet supported)
/// - "how" → Manner (not yet supported)
///
/// Returns `None` if the QUD cannot be converted to a structured query.
#[must_use]
pub fn qud_to_query(qud: &QudIssue) -> Option<Query> {
    // Check if there's a wh-word to determine question type
    match qud.wh_word.as_deref() {
        Some("who" | "whom") => {
            // "Who" questions target the Agent role
            let predicate = qud.predicate_focus.clone()?;
            Some(Query::WhQuestion {
                predicate,
                target_role: ThetaRole::Agent,
                constraints: Vec::new(),
            })
        }
        Some("what") => {
            // "What" questions can target Theme or ask about events
            if let Some(predicate) = &qud.predicate_focus {
                Some(Query::WhQuestion {
                    predicate: predicate.clone(),
                    target_role: ThetaRole::Theme,
                    constraints: Vec::new(),
                })
            } else {
                // "What happened?" type question
                Some(Query::WhatHappened { agent: None })
            }
        }
        Some("where") => {
            let predicate = qud.predicate_focus.clone()?;
            Some(Query::WhQuestion {
                predicate,
                target_role: ThetaRole::Location,
                constraints: Vec::new(),
            })
        }
        Some("which") => {
            // "Which" questions - use the first focus role if available
            let predicate = qud.predicate_focus.clone()?;
            let target_role = qud.focus_roles.first().copied().unwrap_or(ThetaRole::Theme);
            Some(Query::WhQuestion {
                predicate,
                target_role,
                constraints: Vec::new(),
            })
        }
        None => {
            // Yes/no question - try to build a proposition
            if let Some(predicate) = &qud.predicate_focus {
                let proposition = Proposition::new(predicate.clone(), true);
                Some(Query::YesNo { proposition })
            } else {
                None
            }
        }
        // Unsupported wh-words
        Some("when" | "why" | "how" | _) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yes_no_query() {
        let query = Query::yes_no("leave", "John", ThetaRole::Agent);
        match query {
            Query::YesNo { proposition } => {
                assert_eq!(proposition.predicate, "leave");
                assert!(proposition.polarity);
                assert!(proposition.participants.contains_key(&ThetaRole::Agent));
            }
            _ => panic!("Expected YesNo query"),
        }
    }

    #[test]
    fn test_wh_query() {
        let query = Query::wh("give", ThetaRole::Recipient);
        match query {
            Query::WhQuestion {
                predicate,
                target_role,
                constraints,
            } => {
                assert_eq!(predicate, "give");
                assert_eq!(target_role, ThetaRole::Recipient);
                assert!(constraints.is_empty());
            }
            _ => panic!("Expected WhQuestion"),
        }
    }

    #[test]
    fn test_proposition_builder() {
        let prop = Proposition::new("give", true)
            .with_participant(ThetaRole::Agent, Term::Constant("John".into()))
            .with_participant(ThetaRole::Recipient, Term::Constant("Mary".into()))
            .with_participant(ThetaRole::Theme, Term::Variable("?what".into()));

        assert_eq!(prop.predicate, "give");
        assert!(prop.polarity);
        assert_eq!(prop.participants.len(), 3);
    }

    #[test]
    fn test_proposition_negation() {
        let prop = Proposition::simple("leave", ThetaRole::Agent, "John");
        assert!(prop.polarity);

        let negated = prop.negated();
        assert!(!negated.polarity);
    }

    #[test]
    fn test_term_types() {
        let constant = Term::Constant("John".into());
        assert!(constant.is_constant());
        assert!(!constant.is_variable());
        assert_eq!(constant.as_constant(), Some("John"));

        let variable = Term::Variable("?x".into());
        assert!(variable.is_variable());
        assert!(!variable.is_constant());
        assert_eq!(variable.as_variable(), Some("?x"));

        let referent = Term::ReferentId(ReferentId::new(5));
        assert!(!referent.is_constant());
        assert!(!referent.is_variable());
    }

    // Tests for qud_to_query conversion

    #[test]
    fn test_qud_to_query_who() {
        let qud = QudIssue {
            wh_word: Some("who".to_string()),
            predicate_focus: Some("leave".to_string()),
            ..Default::default()
        };

        let query = qud_to_query(&qud);
        assert!(query.is_some());
        match query.unwrap() {
            Query::WhQuestion {
                predicate,
                target_role,
                ..
            } => {
                assert_eq!(predicate, "leave");
                assert_eq!(target_role, ThetaRole::Agent);
            }
            _ => panic!("Expected WhQuestion"),
        }
    }

    #[test]
    fn test_qud_to_query_what() {
        let qud = QudIssue {
            wh_word: Some("what".to_string()),
            predicate_focus: Some("give".to_string()),
            ..Default::default()
        };

        let query = qud_to_query(&qud);
        assert!(query.is_some());
        match query.unwrap() {
            Query::WhQuestion {
                predicate,
                target_role,
                ..
            } => {
                assert_eq!(predicate, "give");
                assert_eq!(target_role, ThetaRole::Theme);
            }
            _ => panic!("Expected WhQuestion"),
        }
    }

    #[test]
    fn test_qud_to_query_what_happened() {
        let qud = QudIssue {
            wh_word: Some("what".to_string()),
            predicate_focus: None,
            ..Default::default()
        };

        let query = qud_to_query(&qud);
        assert!(query.is_some());
        match query.unwrap() {
            Query::WhatHappened { agent } => {
                assert!(agent.is_none());
            }
            _ => panic!("Expected WhatHappened"),
        }
    }

    #[test]
    fn test_qud_to_query_where() {
        let qud = QudIssue {
            wh_word: Some("where".to_string()),
            predicate_focus: Some("put".to_string()),
            ..Default::default()
        };

        let query = qud_to_query(&qud);
        assert!(query.is_some());
        match query.unwrap() {
            Query::WhQuestion {
                predicate,
                target_role,
                ..
            } => {
                assert_eq!(predicate, "put");
                assert_eq!(target_role, ThetaRole::Location);
            }
            _ => panic!("Expected WhQuestion"),
        }
    }

    #[test]
    fn test_qud_to_query_yes_no() {
        let qud = QudIssue {
            wh_word: None,
            predicate_focus: Some("leave".to_string()),
            ..Default::default()
        };

        let query = qud_to_query(&qud);
        assert!(query.is_some());
        match query.unwrap() {
            Query::YesNo { proposition } => {
                assert_eq!(proposition.predicate, "leave");
                assert!(proposition.polarity);
            }
            _ => panic!("Expected YesNo"),
        }
    }

    #[test]
    fn test_qud_to_query_unsupported() {
        let qud = QudIssue {
            wh_word: Some("why".to_string()),
            predicate_focus: Some("leave".to_string()),
            ..Default::default()
        };

        let query = qud_to_query(&qud);
        assert!(query.is_none()); // "why" is not supported
    }
}
