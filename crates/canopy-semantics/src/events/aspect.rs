//! Aspectual classification following Vendler (1967)
//!
//! This module implements Vendler's four-way aspectual classification:
//! - States: "know", "love" (no inherent endpoint, homogeneous)
//! - Activities: "run", "swim" (no inherent endpoint, non-homogeneous)
//! - Accomplishments: "build a house" (inherent endpoint, durative)
//! - Achievements: "arrive", "die" (inherent endpoint, punctual)
//!
//! Extended with modern insights from Dowty (1979) and Filip (1999).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Vendler's aspectual classes with modern refinements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AspectualClass {
    /// States: no internal structure, homogeneous
    /// - Properties: [-dynamic], [-telic], [+durative]
    /// - Examples: "know", "love", "be tall"
    /// - Tests: *"What did you do was know French" (no progressive with 'do')
    State,

    /// Activities: internal structure, no inherent endpoint
    /// - Properties: [+dynamic], [-telic], [+durative]
    /// - Examples: "run", "swim", "walk"
    /// - Tests: "John ran for an hour" (for-adverbials)
    Activity,

    /// Accomplishments: process leading to inherent endpoint
    /// - Properties: [+dynamic], [+telic], [+durative]
    /// - Examples: "build a house", "write a letter"
    /// - Tests: "John built a house in an hour" (in-adverbials)
    Accomplishment,

    /// Achievements: punctual events with inherent endpoint
    /// - Properties: [+dynamic], [+telic], [-durative]
    /// - Examples: "arrive", "die", "recognize"
    /// - Tests: *"John arrived for an hour" (no for-adverbials)
    Achievement,

    /// Semelfactives: punctual events without inherent endpoint (Smith 1997)
    /// - Properties: [+dynamic], [-telic], [-durative]
    /// - Examples: "cough", "knock", "flash"
    /// - Tests: "John coughed for an hour" (iterative reading)
    Semelfactive,
}

impl AspectualClass {
    /// Determine aspectual class from verb properties
    pub fn classify_verb(verb_lemma: &str, has_direct_object: bool) -> Self {
        match verb_lemma {
            // Clear states
            "be" | "have" | "know" | "love" | "hate" | "want" | "need" | "own" | "believe"
            | "think" | "understand" | "remember" | "forget" | "doubt" | "resemble" | "contain"
            | "include" | "lack" | "owe" | "cost" | "weigh" | "measure" => AspectualClass::State,

            // Clear achievements (punctual, telic)
            "arrive" | "die" | "leave" | "start" | "stop" | "finish" | "begin" | "end"
            | "reach" | "find" | "lose" | "win" | "notice" | "realize" | "recognize"
            | "discover" | "spot" | "break" | "explode" | "collapse" => AspectualClass::Achievement,

            // Clear activities (durative, atelic)
            "run" | "walk" | "swim" | "dance" | "play" | "work" | "study" | "sleep" | "rest"
            | "wait" | "sit" | "stand" | "lie" | "laugh" | "cry" | "sing" | "talk" | "chat"
            | "argue" => AspectualClass::Activity,

            // Semelfactives (punctual, atelic)
            "cough" | "sneeze" | "hiccup" | "knock" | "tap" | "flash" | "blink" | "jump"
            | "hop" | "skip" | "nod" | "wink" => AspectualClass::Semelfactive,

            // Accomplishments often require objects
            "build" | "create" | "make" | "write" | "draw" | "paint" | "cook" | "destroy"
            | "repair" | "fix" | "clean" | "wash" => {
                if has_direct_object {
                    AspectualClass::Accomplishment
                } else {
                    AspectualClass::Activity
                }
            }

            // Consumption verbs: accomplishments with objects, activities without
            "eat" | "drink" | "read" | "watch" => {
                if has_direct_object {
                    AspectualClass::Accomplishment // "eat an apple"
                } else {
                    AspectualClass::Activity // "eat" (general activity)
                }
            }

            // Default classification based on common patterns
            _ => {
                if has_direct_object {
                    // Many transitive verbs are accomplishments
                    AspectualClass::Accomplishment
                } else {
                    // Intransitive verbs often activities
                    AspectualClass::Activity
                }
            }
        }
    }

    /// Check if aspectual class is telic (has inherent endpoint)
    pub fn is_telic(&self) -> bool {
        matches!(
            self,
            AspectualClass::Accomplishment | AspectualClass::Achievement
        )
    }

    /// Check if aspectual class is dynamic (involves change)
    pub fn is_dynamic(&self) -> bool {
        !matches!(self, AspectualClass::State)
    }

    /// Check if aspectual class is durative (takes time)
    pub fn is_durative(&self) -> bool {
        matches!(
            self,
            AspectualClass::State | AspectualClass::Activity | AspectualClass::Accomplishment
        )
    }

    /// Check if aspectual class is punctual (instantaneous)
    pub fn is_punctual(&self) -> bool {
        matches!(
            self,
            AspectualClass::Achievement | AspectualClass::Semelfactive
        )
    }

    /// Get the aspectual features as a feature bundle
    pub fn features(&self) -> AspectualFeatures {
        match self {
            AspectualClass::State => AspectualFeatures {
                dynamic: false,
                telic: false,
                durative: true,
            },
            AspectualClass::Activity => AspectualFeatures {
                dynamic: true,
                telic: false,
                durative: true,
            },
            AspectualClass::Accomplishment => AspectualFeatures {
                dynamic: true,
                telic: true,
                durative: true,
            },
            AspectualClass::Achievement => AspectualFeatures {
                dynamic: true,
                telic: true,
                durative: false,
            },
            AspectualClass::Semelfactive => AspectualFeatures {
                dynamic: true,
                telic: false,
                durative: false,
            },
        }
    }
}

/// Aspectual feature bundle for compositional analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectualFeatures {
    /// Does the event involve change? [±dynamic]
    pub dynamic: bool,

    /// Does the event have an inherent endpoint? [±telic]
    pub telic: bool,

    /// Does the event take time? [±durative]
    pub durative: bool,
}

impl AspectualFeatures {
    /// Combine two aspectual feature sets (for complex predicates)
    pub fn compose(&self, other: &AspectualFeatures) -> AspectualFeatures {
        AspectualFeatures {
            // Composition is dynamic if either component is
            dynamic: self.dynamic || other.dynamic,

            // Composition is telic if either component is (but see exceptions)
            telic: self.telic || other.telic,

            // Composition is durative if either component is
            durative: self.durative || other.durative,
        }
    }
}

/// Progressive compatibility (can the verb take progressive aspect?)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressiveCompatibility {
    /// Always compatible: "John is running"
    Compatible,

    /// Never compatible: "*John is knowing French"
    Incompatible,

    /// Compatible with coercion: "John is understanding more each day"
    Coercible,
}

impl AspectualClass {
    /// Check progressive compatibility
    pub fn progressive_compatibility(&self) -> ProgressiveCompatibility {
        match self {
            AspectualClass::State => ProgressiveCompatibility::Incompatible,
            AspectualClass::Activity => ProgressiveCompatibility::Compatible,
            AspectualClass::Accomplishment => ProgressiveCompatibility::Compatible,
            AspectualClass::Achievement => ProgressiveCompatibility::Coercible,
            AspectualClass::Semelfactive => ProgressiveCompatibility::Compatible,
        }
    }
}

/// Temporal modifier compatibility (Dowty 1979)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalModifierType {
    /// "for X time" - measures duration
    ForAdverbial,

    /// "in X time" - measures time to completion
    InAdverbial,

    /// "at X time" - specifies time point
    AtAdverbial,
}

impl AspectualClass {
    /// Check compatibility with temporal modifiers
    pub fn temporal_modifier_compatibility(&self, modifier: TemporalModifierType) -> bool {
        match (self, modifier) {
            // States: compatible with "for" but not "in"
            (AspectualClass::State, TemporalModifierType::ForAdverbial) => true,
            (AspectualClass::State, TemporalModifierType::InAdverbial) => false,
            (AspectualClass::State, TemporalModifierType::AtAdverbial) => false,

            // Activities: compatible with "for" but not "in"
            (AspectualClass::Activity, TemporalModifierType::ForAdverbial) => true,
            (AspectualClass::Activity, TemporalModifierType::InAdverbial) => false,
            (AspectualClass::Activity, TemporalModifierType::AtAdverbial) => false,

            // Accomplishments: compatible with both "for" and "in"
            (AspectualClass::Accomplishment, TemporalModifierType::ForAdverbial) => true,
            (AspectualClass::Accomplishment, TemporalModifierType::InAdverbial) => true,
            (AspectualClass::Accomplishment, TemporalModifierType::AtAdverbial) => false,

            // Achievements: compatible with "at" but not "for"
            (AspectualClass::Achievement, TemporalModifierType::ForAdverbial) => false,
            (AspectualClass::Achievement, TemporalModifierType::InAdverbial) => true,
            (AspectualClass::Achievement, TemporalModifierType::AtAdverbial) => true,

            // Semelfactives: special behavior (iterative with "for")
            (AspectualClass::Semelfactive, TemporalModifierType::ForAdverbial) => true,
            (AspectualClass::Semelfactive, TemporalModifierType::InAdverbial) => false,
            (AspectualClass::Semelfactive, TemporalModifierType::AtAdverbial) => true,
        }
    }
}

impl fmt::Display for AspectualClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AspectualClass::State => write!(f, "State"),
            AspectualClass::Activity => write!(f, "Activity"),
            AspectualClass::Accomplishment => write!(f, "Accomplishment"),
            AspectualClass::Achievement => write!(f, "Achievement"),
            AspectualClass::Semelfactive => write!(f, "Semelfactive"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aspectual_classification() {
        // States
        assert_eq!(
            AspectualClass::classify_verb("know", false),
            AspectualClass::State
        );
        assert_eq!(
            AspectualClass::classify_verb("love", false),
            AspectualClass::State
        );

        // Activities
        assert_eq!(
            AspectualClass::classify_verb("run", false),
            AspectualClass::Activity
        );
        assert_eq!(
            AspectualClass::classify_verb("swim", false),
            AspectualClass::Activity
        );

        // Achievements
        assert_eq!(
            AspectualClass::classify_verb("arrive", false),
            AspectualClass::Achievement
        );
        assert_eq!(
            AspectualClass::classify_verb("die", false),
            AspectualClass::Achievement
        );

        // Accomplishments (with objects)
        assert_eq!(
            AspectualClass::classify_verb("build", true),
            AspectualClass::Accomplishment
        );
        assert_eq!(
            AspectualClass::classify_verb("build", false),
            AspectualClass::Activity
        );

        // Semelfactives
        assert_eq!(
            AspectualClass::classify_verb("cough", false),
            AspectualClass::Semelfactive
        );
        assert_eq!(
            AspectualClass::classify_verb("knock", false),
            AspectualClass::Semelfactive
        );
    }

    #[test]
    fn test_aspectual_features() {
        let state = AspectualClass::State;
        assert!(!state.is_dynamic());
        assert!(!state.is_telic());
        assert!(state.is_durative());

        let achievement = AspectualClass::Achievement;
        assert!(achievement.is_dynamic());
        assert!(achievement.is_telic());
        assert!(!achievement.is_durative());
    }

    #[test]
    fn test_progressive_compatibility() {
        assert_eq!(
            AspectualClass::State.progressive_compatibility(),
            ProgressiveCompatibility::Incompatible
        );
        assert_eq!(
            AspectualClass::Activity.progressive_compatibility(),
            ProgressiveCompatibility::Compatible
        );
    }

    #[test]
    fn test_temporal_modifier_compatibility() {
        // States compatible with "for", not "in"
        assert!(
            AspectualClass::State
                .temporal_modifier_compatibility(TemporalModifierType::ForAdverbial)
        );
        assert!(
            !AspectualClass::State
                .temporal_modifier_compatibility(TemporalModifierType::InAdverbial)
        );

        // Accomplishments compatible with both
        assert!(
            AspectualClass::Accomplishment
                .temporal_modifier_compatibility(TemporalModifierType::ForAdverbial)
        );
        assert!(
            AspectualClass::Accomplishment
                .temporal_modifier_compatibility(TemporalModifierType::InAdverbial)
        );

        // Achievements not compatible with "for"
        assert!(
            !AspectualClass::Achievement
                .temporal_modifier_compatibility(TemporalModifierType::ForAdverbial)
        );
        assert!(
            AspectualClass::Achievement
                .temporal_modifier_compatibility(TemporalModifierType::AtAdverbial)
        );
    }

    #[test]
    fn test_feature_composition() {
        let state_features = AspectualClass::State.features();
        let activity_features = AspectualClass::Activity.features();

        let composed = state_features.compose(&activity_features);

        // Composition should be dynamic (from activity)
        assert!(composed.dynamic);
        // Should be durative (both are)
        assert!(composed.durative);
        // Should not be telic (neither is)
        assert!(!composed.telic);
    }
}
