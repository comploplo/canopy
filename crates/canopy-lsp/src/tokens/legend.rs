//! Semantic token legend definition
//!
//! Defines token types and modifiers for theta role highlighting.

use canopy::core::ThetaRole;
use tower_lsp::lsp_types::{SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend};

/// Token type index for theta roles and linguistic categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ThetaTokenType {
    // Core theta roles
    Agent = 0,
    Patient = 1,
    Theme = 2,
    Experiencer = 3,
    Recipient = 4,
    // Non-core theta roles
    Benefactive = 5,
    Instrument = 6,
    Location = 7,
    Source = 8,
    Goal = 9,
    Direction = 10,
    Temporal = 11,
    Manner = 12,
    Cause = 13,
    // Syntactic categories
    Predicate = 14,
    Auxiliary = 15,
    Determiner = 16,
    Conjunction = 17,
    // Discourse
    DiscourseConnective = 18,
    // Other
    Other = 19,
}

impl ThetaTokenType {
    /// Convert from Canopy `ThetaRole` to token type.
    #[must_use]
    pub fn from_theta_role(role: ThetaRole) -> Self {
        match role {
            ThetaRole::Agent => Self::Agent,
            ThetaRole::Patient => Self::Patient,
            ThetaRole::Theme => Self::Theme,
            ThetaRole::Experiencer => Self::Experiencer,
            ThetaRole::Recipient => Self::Recipient,
            ThetaRole::Benefactive => Self::Benefactive,
            ThetaRole::Instrument => Self::Instrument,
            ThetaRole::Location => Self::Location,
            ThetaRole::Source => Self::Source,
            ThetaRole::Goal => Self::Goal,
            ThetaRole::Direction => Self::Direction,
            ThetaRole::Temporal => Self::Temporal,
            ThetaRole::Manner => Self::Manner,
            ThetaRole::Cause => Self::Cause,
            _ => Self::Other,
        }
    }

    /// Get the string name for this token type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Patient => "patient",
            Self::Theme => "theme",
            Self::Experiencer => "experiencer",
            Self::Recipient => "recipient",
            Self::Benefactive => "benefactive",
            Self::Instrument => "instrument",
            Self::Location => "location",
            Self::Source => "source",
            Self::Goal => "goal",
            Self::Direction => "direction",
            Self::Temporal => "temporal",
            Self::Manner => "manner",
            Self::Cause => "cause",
            Self::Predicate => "predicate",
            Self::Auxiliary => "auxiliary",
            Self::Determiner => "determiner",
            Self::Conjunction => "conjunction",
            Self::DiscourseConnective => "discourseConnective",
            Self::Other => "other",
        }
    }
}

/// Token modifier flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenModifier {
    /// High confidence binding (>0.9).
    HighConfidence = 0,
    /// Low confidence binding (<0.7).
    LowConfidence = 1,
    /// Multiple possible readings.
    Ambiguous = 2,
    /// Negated predicate.
    Negated = 3,
}

impl TokenModifier {
    /// Get the bitmask for this modifier.
    #[must_use]
    pub fn bitmask(&self) -> u32 {
        1 << (*self as u32)
    }

    /// Get the string name for this modifier.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HighConfidence => "highConfidence",
            Self::LowConfidence => "lowConfidence",
            Self::Ambiguous => "ambiguous",
            Self::Negated => "negated",
        }
    }
}

/// All token types in order.
pub const TOKEN_TYPES: &[&str] = &[
    "agent",
    "patient",
    "theme",
    "experiencer",
    "recipient",
    "benefactive",
    "instrument",
    "location",
    "source",
    "goal",
    "direction",
    "temporal",
    "manner",
    "cause",
    "predicate",
    "auxiliary",
    "determiner",
    "conjunction",
    "discourseConnective",
    "other",
];

/// All token modifiers in order.
pub const TOKEN_MODIFIERS: &[&str] = &["highConfidence", "lowConfidence", "ambiguous", "negated"];

/// Create the semantic token legend for LSP.
#[must_use]
pub fn semantic_token_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES
            .iter()
            .map(|&s| SemanticTokenType::new(s))
            .collect(),
        token_modifiers: TOKEN_MODIFIERS
            .iter()
            .map(|&s| SemanticTokenModifier::new(s))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theta_role_to_token_type_all_roles() {
        // Core theta roles
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Agent),
            ThetaTokenType::Agent
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Patient),
            ThetaTokenType::Patient
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Theme),
            ThetaTokenType::Theme
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Experiencer),
            ThetaTokenType::Experiencer
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Recipient),
            ThetaTokenType::Recipient
        );
        // Non-core theta roles
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Benefactive),
            ThetaTokenType::Benefactive
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Instrument),
            ThetaTokenType::Instrument
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Location),
            ThetaTokenType::Location
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Source),
            ThetaTokenType::Source
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Goal),
            ThetaTokenType::Goal
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Direction),
            ThetaTokenType::Direction
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Temporal),
            ThetaTokenType::Temporal
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Manner),
            ThetaTokenType::Manner
        );
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Cause),
            ThetaTokenType::Cause
        );
        // Other roles map to Other
        assert_eq!(
            ThetaTokenType::from_theta_role(ThetaRole::Stimulus),
            ThetaTokenType::Other
        );
    }

    #[test]
    fn test_token_type_index_all() {
        assert_eq!(ThetaTokenType::Agent as u32, 0);
        assert_eq!(ThetaTokenType::Patient as u32, 1);
        assert_eq!(ThetaTokenType::Theme as u32, 2);
        assert_eq!(ThetaTokenType::Experiencer as u32, 3);
        assert_eq!(ThetaTokenType::Recipient as u32, 4);
        assert_eq!(ThetaTokenType::Benefactive as u32, 5);
        assert_eq!(ThetaTokenType::Instrument as u32, 6);
        assert_eq!(ThetaTokenType::Location as u32, 7);
        assert_eq!(ThetaTokenType::Source as u32, 8);
        assert_eq!(ThetaTokenType::Goal as u32, 9);
        assert_eq!(ThetaTokenType::Direction as u32, 10);
        assert_eq!(ThetaTokenType::Temporal as u32, 11);
        assert_eq!(ThetaTokenType::Manner as u32, 12);
        assert_eq!(ThetaTokenType::Cause as u32, 13);
        assert_eq!(ThetaTokenType::Predicate as u32, 14);
        assert_eq!(ThetaTokenType::Auxiliary as u32, 15);
        assert_eq!(ThetaTokenType::Determiner as u32, 16);
        assert_eq!(ThetaTokenType::Conjunction as u32, 17);
        assert_eq!(ThetaTokenType::DiscourseConnective as u32, 18);
        assert_eq!(ThetaTokenType::Other as u32, 19);
    }

    #[test]
    fn test_token_type_as_str() {
        assert_eq!(ThetaTokenType::Agent.as_str(), "agent");
        assert_eq!(ThetaTokenType::Patient.as_str(), "patient");
        assert_eq!(ThetaTokenType::Theme.as_str(), "theme");
        assert_eq!(ThetaTokenType::Experiencer.as_str(), "experiencer");
        assert_eq!(ThetaTokenType::Recipient.as_str(), "recipient");
        assert_eq!(ThetaTokenType::Benefactive.as_str(), "benefactive");
        assert_eq!(ThetaTokenType::Instrument.as_str(), "instrument");
        assert_eq!(ThetaTokenType::Location.as_str(), "location");
        assert_eq!(ThetaTokenType::Source.as_str(), "source");
        assert_eq!(ThetaTokenType::Goal.as_str(), "goal");
        assert_eq!(ThetaTokenType::Direction.as_str(), "direction");
        assert_eq!(ThetaTokenType::Temporal.as_str(), "temporal");
        assert_eq!(ThetaTokenType::Manner.as_str(), "manner");
        assert_eq!(ThetaTokenType::Cause.as_str(), "cause");
        assert_eq!(ThetaTokenType::Predicate.as_str(), "predicate");
        assert_eq!(ThetaTokenType::Auxiliary.as_str(), "auxiliary");
        assert_eq!(ThetaTokenType::Determiner.as_str(), "determiner");
        assert_eq!(ThetaTokenType::Conjunction.as_str(), "conjunction");
        assert_eq!(
            ThetaTokenType::DiscourseConnective.as_str(),
            "discourseConnective"
        );
        assert_eq!(ThetaTokenType::Other.as_str(), "other");
    }

    #[test]
    fn test_modifier_bitmask_all() {
        assert_eq!(TokenModifier::HighConfidence.bitmask(), 1);
        assert_eq!(TokenModifier::LowConfidence.bitmask(), 2);
        assert_eq!(TokenModifier::Ambiguous.bitmask(), 4);
        assert_eq!(TokenModifier::Negated.bitmask(), 8);
    }

    #[test]
    fn test_modifier_as_str() {
        assert_eq!(TokenModifier::HighConfidence.as_str(), "highConfidence");
        assert_eq!(TokenModifier::LowConfidence.as_str(), "lowConfidence");
        assert_eq!(TokenModifier::Ambiguous.as_str(), "ambiguous");
        assert_eq!(TokenModifier::Negated.as_str(), "negated");
    }

    #[test]
    fn test_legend_size() {
        let legend = semantic_token_legend();
        assert_eq!(legend.token_types.len(), TOKEN_TYPES.len());
        assert_eq!(legend.token_modifiers.len(), TOKEN_MODIFIERS.len());
    }

    #[test]
    fn test_token_types_match_enum() {
        // Verify TOKEN_TYPES array matches the enum as_str() values
        assert_eq!(
            TOKEN_TYPES[ThetaTokenType::Agent as usize],
            ThetaTokenType::Agent.as_str()
        );
        assert_eq!(
            TOKEN_TYPES[ThetaTokenType::Patient as usize],
            ThetaTokenType::Patient.as_str()
        );
        assert_eq!(
            TOKEN_TYPES[ThetaTokenType::Other as usize],
            ThetaTokenType::Other.as_str()
        );
    }

    #[test]
    fn test_token_modifiers_match_enum() {
        // Verify TOKEN_MODIFIERS array matches the enum as_str() values
        assert_eq!(TOKEN_MODIFIERS[0], TokenModifier::HighConfidence.as_str());
        assert_eq!(TOKEN_MODIFIERS[1], TokenModifier::LowConfidence.as_str());
        assert_eq!(TOKEN_MODIFIERS[2], TokenModifier::Ambiguous.as_str());
        assert_eq!(TOKEN_MODIFIERS[3], TokenModifier::Negated.as_str());
    }
}
