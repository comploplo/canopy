//! Discourse referent tracking
//!
//! Manages discourse referents - entities and events introduced in discourse
//! that can be referred back to by pronouns and other anaphoric expressions.

use canopy_core::{Animacy, Definiteness};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Unique identifier for a discourse referent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReferentId(pub usize);

/// A discourse referent (entity or event that can be referred to)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscourseReferent {
    /// Unique identifier
    pub id: ReferentId,

    /// Optional name/description (from the introducing NP)
    pub name: Option<String>,

    /// Type of referent
    pub referent_type: ReferentType,

    /// Whether this is an event referent (vs entity)
    pub is_event: bool,

    /// Sentence index where this referent was introduced
    pub introduced_at: usize,

    /// Properties accumulated through discourse
    pub properties: IndexMap<String, PropertyValue>,
}

/// Types of discourse referents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferentType {
    /// Individual entity: "a man", "John"
    Individual,
    /// Plural/group: "the men", "some books"
    Group,
    /// Mass/uncountable: "water", "information"
    Mass,
    /// Event: "the meeting", "his departure"
    Event,
    /// Proposition: "that he left", "the fact that..."
    Proposition,
    /// Time point or interval
    Time,
    /// Location/place
    Location,
}

/// Values for referent properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Bool(bool),
    Number(i64),
    Animacy(Animacy),
    Definiteness(Definiteness),
    Gender(Gender),
    NumberFeature(NumberFeature),
}

/// Grammatical gender for anaphora resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
    Unknown,
}

/// Grammatical number
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberFeature {
    Singular,
    Plural,
    Unknown,
}

/// Anaphor types per Reinhart & Reuland (1993)
///
/// This distinguishes between SELF-anaphors (reflexives) and personal pronouns,
/// which have fundamentally different binding behaviors:
/// - SELF-anaphors reflexive-mark predicates (Condition A)
/// - Personal pronouns cannot reflexive-mark (Condition B)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnaphorType {
    /// SELF-anaphors: himself, herself, itself, themselves, myself, yourself, etc.
    /// Feature-rich, morphologically complex (X-self pattern)
    /// These reflexive-mark the predicate they appear in
    SelfAnaphor,

    /// Personal pronouns: he, she, it, they, him, her, them, etc.
    /// Feature-rich but cannot reflexive-mark predicates
    /// Per Condition B, cannot co-refer with co-arguments
    Personal,

    /// Possessive pronouns: his, her, its, their, my, your, our
    /// Similar to personal pronouns for binding
    Possessive,

    /// Not anaphoric (full NP, proper name, or unknown)
    None,
}

/// Grammatical person feature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Person {
    /// First person: I, we, myself, ourselves
    First,
    /// Second person: you, yourself, yourselves
    Second,
    /// Third person: he, she, it, they, himself, herself, etc.
    Third,
}

/// Complete classification of an anaphoric expression
///
/// Based on Reinhart & Reuland (1993) "Reflexivity" and
/// Reuland (2011) "Anaphora and Language Design"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnaphorClassification {
    /// Type of anaphor (SELF-anaphor, personal pronoun, etc.)
    pub anaphor_type: AnaphorType,
    /// Grammatical person (first, second, third)
    pub person: Option<Person>,
    /// Grammatical gender
    pub gender: Option<Gender>,
    /// Grammatical number
    pub number: Option<NumberFeature>,
}

impl AnaphorClassification {
    /// Create a classification for a non-anaphoric expression
    #[must_use]
    pub fn none() -> Self {
        Self {
            anaphor_type: AnaphorType::None,
            person: None,
            gender: None,
            number: None,
        }
    }
}

/// Classify an anaphoric expression
///
/// Returns the anaphor type and phi-features (person, gender, number).
///
/// Based on Reinhart & Reuland (1993):
/// - SELF-anaphors (himself, herself, etc.) are morphologically complex
///   and reflexive-mark predicates
/// - Personal pronouns (he, she, it) are feature-rich but cannot
///   reflexive-mark, so they obey Condition B
#[must_use]
pub fn classify_anaphor(word: &str) -> AnaphorClassification {
    match word.to_lowercase().as_str() {
        // SELF-anaphors (reflexives) - Third person
        "himself" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::Third),
            gender: Some(Gender::Masculine),
            number: Some(NumberFeature::Singular),
        },
        "herself" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::Third),
            gender: Some(Gender::Feminine),
            number: Some(NumberFeature::Singular),
        },
        "itself" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::Third),
            gender: Some(Gender::Neuter),
            number: Some(NumberFeature::Singular),
        },
        "themselves" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::Third),
            gender: None, // Can be any gender
            number: Some(NumberFeature::Plural),
        },
        "themself" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::Third),
            gender: None, // Singular they
            number: Some(NumberFeature::Singular),
        },

        // SELF-anaphors - First person
        "myself" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::First),
            gender: None,
            number: Some(NumberFeature::Singular),
        },
        "ourselves" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::First),
            gender: None,
            number: Some(NumberFeature::Plural),
        },

        // SELF-anaphors - Second person
        "yourself" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::Second),
            gender: None,
            number: Some(NumberFeature::Singular),
        },
        "yourselves" => AnaphorClassification {
            anaphor_type: AnaphorType::SelfAnaphor,
            person: Some(Person::Second),
            gender: None,
            number: Some(NumberFeature::Plural),
        },

        // Personal pronouns - Third person nominative/accusative
        "he" | "him" => AnaphorClassification {
            anaphor_type: AnaphorType::Personal,
            person: Some(Person::Third),
            gender: Some(Gender::Masculine),
            number: Some(NumberFeature::Singular),
        },
        "she" | "her" => AnaphorClassification {
            anaphor_type: AnaphorType::Personal,
            person: Some(Person::Third),
            gender: Some(Gender::Feminine),
            number: Some(NumberFeature::Singular),
        },
        "it" => AnaphorClassification {
            anaphor_type: AnaphorType::Personal,
            person: Some(Person::Third),
            gender: Some(Gender::Neuter),
            number: Some(NumberFeature::Singular),
        },
        "they" | "them" => AnaphorClassification {
            anaphor_type: AnaphorType::Personal,
            person: Some(Person::Third),
            gender: None, // Can be any gender (singular they)
            number: None, // Can be singular or plural
        },

        // Personal pronouns - First person
        "i" | "me" => AnaphorClassification {
            anaphor_type: AnaphorType::Personal,
            person: Some(Person::First),
            gender: None,
            number: Some(NumberFeature::Singular),
        },
        "we" | "us" => AnaphorClassification {
            anaphor_type: AnaphorType::Personal,
            person: Some(Person::First),
            gender: None,
            number: Some(NumberFeature::Plural),
        },

        // Personal pronouns - Second person
        "you" => AnaphorClassification {
            anaphor_type: AnaphorType::Personal,
            person: Some(Person::Second),
            gender: None,
            number: None, // Can be singular or plural
        },

        // Possessive pronouns
        "his" => AnaphorClassification {
            anaphor_type: AnaphorType::Possessive,
            person: Some(Person::Third),
            gender: Some(Gender::Masculine),
            number: Some(NumberFeature::Singular),
        },
        "hers" => AnaphorClassification {
            anaphor_type: AnaphorType::Possessive,
            person: Some(Person::Third),
            gender: Some(Gender::Feminine),
            number: Some(NumberFeature::Singular),
        },
        "its" => AnaphorClassification {
            anaphor_type: AnaphorType::Possessive,
            person: Some(Person::Third),
            gender: Some(Gender::Neuter),
            number: Some(NumberFeature::Singular),
        },
        "their" | "theirs" => AnaphorClassification {
            anaphor_type: AnaphorType::Possessive,
            person: Some(Person::Third),
            gender: None,
            number: None,
        },
        "my" | "mine" => AnaphorClassification {
            anaphor_type: AnaphorType::Possessive,
            person: Some(Person::First),
            gender: None,
            number: Some(NumberFeature::Singular),
        },
        "our" | "ours" => AnaphorClassification {
            anaphor_type: AnaphorType::Possessive,
            person: Some(Person::First),
            gender: None,
            number: Some(NumberFeature::Plural),
        },
        "your" | "yours" => AnaphorClassification {
            anaphor_type: AnaphorType::Possessive,
            person: Some(Person::Second),
            gender: None,
            number: None,
        },

        // Not a pronoun/anaphor
        _ => AnaphorClassification::none(),
    }
}

/// Check if a word is a SELF-anaphor (reflexive)
#[must_use]
pub fn is_self_anaphor(word: &str) -> bool {
    matches!(
        classify_anaphor(word).anaphor_type,
        AnaphorType::SelfAnaphor
    )
}

/// Check if a word is a personal pronoun
#[must_use]
pub fn is_personal_pronoun(word: &str) -> bool {
    matches!(classify_anaphor(word).anaphor_type, AnaphorType::Personal)
}

/// Check if a word is any kind of pronoun
#[must_use]
pub fn is_pronoun(word: &str) -> bool {
    !matches!(classify_anaphor(word).anaphor_type, AnaphorType::None)
}

impl DiscourseReferent {
    /// Create a new discourse referent
    #[must_use]
    pub fn new(id: ReferentId, referent_type: ReferentType, introduced_at: usize) -> Self {
        Self {
            id,
            name: None,
            referent_type,
            is_event: matches!(referent_type, ReferentType::Event),
            introduced_at,
            properties: IndexMap::new(),
        }
    }

    /// Create a new entity referent with a name
    #[must_use]
    pub fn entity(id: ReferentId, name: String, introduced_at: usize) -> Self {
        Self {
            id,
            name: Some(name),
            referent_type: ReferentType::Individual,
            is_event: false,
            introduced_at,
            properties: IndexMap::new(),
        }
    }

    /// Create a new event referent
    #[must_use]
    pub fn event(id: ReferentId, predicate: String, introduced_at: usize) -> Self {
        Self {
            id,
            name: Some(predicate),
            referent_type: ReferentType::Event,
            is_event: true,
            introduced_at,
            properties: IndexMap::new(),
        }
    }

    /// Add a property to this referent
    pub fn add_property(&mut self, key: impl Into<String>, value: PropertyValue) {
        self.properties.insert(key.into(), value);
    }

    /// Set animacy
    pub fn set_animacy(&mut self, animacy: Animacy) {
        self.properties
            .insert("animacy".to_string(), PropertyValue::Animacy(animacy));
    }

    /// Get animacy if set
    #[must_use]
    pub fn animacy(&self) -> Option<Animacy> {
        self.properties.get("animacy").and_then(|v| match v {
            PropertyValue::Animacy(a) => Some(*a),
            _ => None,
        })
    }

    /// Set gender
    pub fn set_gender(&mut self, gender: Gender) {
        self.properties
            .insert("gender".to_string(), PropertyValue::Gender(gender));
    }

    /// Get gender if set
    #[must_use]
    pub fn gender(&self) -> Option<Gender> {
        self.properties.get("gender").and_then(|v| match v {
            PropertyValue::Gender(g) => Some(*g),
            _ => None,
        })
    }

    /// Set number
    pub fn set_number(&mut self, number: NumberFeature) {
        self.properties
            .insert("number".to_string(), PropertyValue::NumberFeature(number));
    }

    /// Get number if set
    #[must_use]
    pub fn number(&self) -> Option<NumberFeature> {
        self.properties.get("number").and_then(|v| match v {
            PropertyValue::NumberFeature(n) => Some(*n),
            _ => None,
        })
    }

    /// Check if this referent is compatible with a pronoun
    #[must_use]
    pub fn matches_pronoun(&self, pronoun: &str) -> bool {
        let pronoun_lower = pronoun.to_lowercase();

        // Check gender compatibility
        let gender_match = match pronoun_lower.as_str() {
            "he" | "him" | "his" | "himself" => self
                .gender()
                .is_none_or(|g| g == Gender::Masculine || g == Gender::Unknown),
            "she" | "her" | "hers" | "herself" => self
                .gender()
                .is_none_or(|g| g == Gender::Feminine || g == Gender::Unknown),
            "it" | "its" | "itself" => self
                .gender()
                .is_none_or(|g| g == Gender::Neuter || g == Gender::Unknown),
            "they" | "them" | "their" | "theirs" | "themselves" => true, // Can be singular or plural
            _ => true,
        };

        // Check number compatibility
        let number_match = match pronoun_lower.as_str() {
            "he" | "him" | "his" | "himself" | "she" | "her" | "hers" | "herself" | "it"
            | "its" | "itself" => self
                .number()
                .is_none_or(|n| n == NumberFeature::Singular || n == NumberFeature::Unknown),
            "they" | "them" | "their" | "theirs" | "themselves" => true, // Singular they is valid
            "we" | "us" | "our" | "ours" | "ourselves" => {
                self.number().is_none_or(|n| n == NumberFeature::Plural)
            }
            _ => true,
        };

        // Check animacy for he/she vs it
        let animacy_match = match pronoun_lower.as_str() {
            "he" | "him" | "his" | "himself" | "she" | "her" | "hers" | "herself" => {
                self.animacy().is_none_or(|a| a == Animacy::Human)
            }
            "it" | "its" | "itself" => self
                .animacy()
                .is_none_or(|a| a != Animacy::Human || self.is_event),
            _ => true,
        };

        gender_match && number_match && animacy_match
    }

    /// Calculate salience score for anaphora resolution
    /// Higher scores indicate more salient (likely) antecedents
    #[must_use]
    pub fn salience_score(&self, current_sentence: usize) -> f32 {
        let mut score = 1.0;

        // Recency: referents from recent sentences are more salient
        let distance = current_sentence.saturating_sub(self.introduced_at);
        score *= 1.0 / (1.0 + distance as f32 * 0.3);

        // Animacy: humans are more salient
        if let Some(animacy) = self.animacy() {
            score *= match animacy {
                Animacy::Human => 1.5,
                Animacy::Animal => 1.2,
                Animacy::Plant => 0.8,
                Animacy::Inanimate => 0.6,
            };
        }

        // Named entities are more salient
        if self.name.is_some() {
            score *= 1.2;
        }

        score
    }
}

/// Manages a set of active discourse referents
#[derive(Debug, Clone)]
pub struct ReferentRegistry {
    /// All referents by ID
    referents: IndexMap<ReferentId, DiscourseReferent>,

    /// Next available ID
    next_id: usize,

    /// Currently salient referents (in focus)
    focus_stack: Vec<ReferentId>,
}

impl ReferentRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            referents: IndexMap::new(),
            next_id: 1,
            focus_stack: Vec::new(),
        }
    }

    /// Allocate a new referent ID
    pub fn allocate_id(&mut self) -> ReferentId {
        let id = ReferentId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Register a new referent
    pub fn register(&mut self, referent: DiscourseReferent) {
        let id = referent.id;
        self.referents.insert(id, referent);
        // Push to focus stack - new referents are salient
        self.focus_stack.push(id);
    }

    /// Get a referent by ID
    #[must_use]
    pub fn get(&self, id: ReferentId) -> Option<&DiscourseReferent> {
        self.referents.get(&id)
    }

    /// Get a mutable referent by ID
    pub fn get_mut(&mut self, id: ReferentId) -> Option<&mut DiscourseReferent> {
        self.referents.get_mut(&id)
    }

    /// Find candidate antecedents for a pronoun
    #[must_use]
    pub fn find_antecedent_candidates(
        &self,
        pronoun: &str,
        current_sentence: usize,
    ) -> Vec<(ReferentId, f32)> {
        let mut candidates: Vec<_> = self
            .referents
            .values()
            .filter(|r| {
                // Don't allow pronouns to be antecedents for other pronouns
                // (pronouns marked with is_pronoun property should be skipped)
                let is_pronoun_ref = r
                    .properties
                    .get("is_pronoun")
                    .map(|v| matches!(v, PropertyValue::Bool(true)))
                    .unwrap_or(false);

                // Don't allow events to be antecedents for personal pronouns
                let is_event = r.is_event;

                !is_pronoun_ref && !is_event && r.matches_pronoun(pronoun)
            })
            .map(|r| (r.id, r.salience_score(current_sentence)))
            .collect();

        // Sort by salience (descending)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        candidates
    }

    /// Get referent count
    #[must_use]
    pub fn len(&self) -> usize {
        self.referents.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.referents.is_empty()
    }

    /// Clear the registry
    pub fn clear(&mut self) {
        self.referents.clear();
        self.focus_stack.clear();
        self.next_id = 1;
    }

    /// Iterate over all referents
    pub fn iter(&self) -> impl Iterator<Item = &DiscourseReferent> {
        self.referents.values()
    }
}

impl Default for ReferentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referent_creation() {
        let referent = DiscourseReferent::entity(ReferentId(1), "John".to_string(), 0);
        assert_eq!(referent.id, ReferentId(1));
        assert_eq!(referent.name, Some("John".to_string()));
        assert!(!referent.is_event);
    }

    #[test]
    fn test_event_referent() {
        let referent = DiscourseReferent::event(ReferentId(2), "run".to_string(), 1);
        assert!(referent.is_event);
        assert_eq!(referent.referent_type, ReferentType::Event);
    }

    #[test]
    fn test_pronoun_matching() {
        let mut referent = DiscourseReferent::entity(ReferentId(1), "John".to_string(), 0);
        referent.set_gender(Gender::Masculine);
        referent.set_number(NumberFeature::Singular);
        referent.set_animacy(Animacy::Human);

        assert!(referent.matches_pronoun("he"));
        assert!(referent.matches_pronoun("him"));
        assert!(!referent.matches_pronoun("she"));
        assert!(!referent.matches_pronoun("it"));
    }

    #[test]
    fn test_salience_score() {
        let mut referent = DiscourseReferent::entity(ReferentId(1), "John".to_string(), 0);
        referent.set_animacy(Animacy::Human);

        let score_same_sentence = referent.salience_score(0);
        let score_next_sentence = referent.salience_score(1);
        let score_later = referent.salience_score(5);

        assert!(score_same_sentence > score_next_sentence);
        assert!(score_next_sentence > score_later);
    }

    #[test]
    fn test_registry() {
        let mut registry = ReferentRegistry::new();

        let id1 = registry.allocate_id();
        let referent1 = DiscourseReferent::entity(id1, "John".to_string(), 0);
        registry.register(referent1);

        let id2 = registry.allocate_id();
        let referent2 = DiscourseReferent::entity(id2, "Mary".to_string(), 0);
        registry.register(referent2);

        assert_eq!(registry.len(), 2);
        assert!(registry.get(id1).is_some());
        assert!(registry.get(id2).is_some());
    }

    #[test]
    fn test_find_antecedent_candidates() {
        let mut registry = ReferentRegistry::new();

        let id1 = registry.allocate_id();
        let mut referent1 = DiscourseReferent::entity(id1, "John".to_string(), 0);
        referent1.set_gender(Gender::Masculine);
        referent1.set_animacy(Animacy::Human);
        registry.register(referent1);

        let id2 = registry.allocate_id();
        let mut referent2 = DiscourseReferent::entity(id2, "Mary".to_string(), 0);
        referent2.set_gender(Gender::Feminine);
        referent2.set_animacy(Animacy::Human);
        registry.register(referent2);

        let candidates = registry.find_antecedent_candidates("he", 1);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].0, id1);

        let candidates = registry.find_antecedent_candidates("she", 1);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].0, id2);
    }
}
