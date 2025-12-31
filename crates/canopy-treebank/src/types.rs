//! Type definitions for treebank analysis
//!
//! This module defines the core types used throughout the treebank engine,
//! including dependency patterns, relations, and analysis results.

use canopy_core::ThetaRole;
use serde::{Deserialize, Serialize};

/// Universal Dependency relations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DependencyRelation {
    /// Nominal subject
    #[serde(rename = "nsubj")]
    NominalSubject,
    /// Direct object
    #[serde(rename = "obj")]
    Object,
    /// Indirect object
    #[serde(rename = "iobj")]
    IndirectObject,
    /// Oblique nominal
    #[serde(rename = "obl")]
    Oblique,
    /// Adverbial modifier
    #[serde(rename = "advmod")]
    AdverbialModifier,
    /// Adjectival modifier
    #[serde(rename = "amod")]
    AdjectivalModifier,
    /// Compound
    #[serde(rename = "compound")]
    Compound,
    /// Coordination
    #[serde(rename = "conj")]
    Conjunction,
    /// Coordinating conjunction
    #[serde(rename = "cc")]
    CoordinatingConjunction,
    /// Determiner
    #[serde(rename = "det")]
    Determiner,
    /// Case marking
    #[serde(rename = "case")]
    Case,
    /// Auxiliary
    #[serde(rename = "aux")]
    Auxiliary,
    /// Copula
    #[serde(rename = "cop")]
    Copula,
    /// Mark
    #[serde(rename = "mark")]
    Mark,
    /// Clausal complement
    #[serde(rename = "ccomp")]
    ClausalComplement,
    /// Open clausal complement
    #[serde(rename = "xcomp")]
    XClausalComplement,
    /// Relative clause modifier
    #[serde(rename = "acl:relcl")]
    RelativeClause,
    /// Adverbial clause modifier
    #[serde(rename = "advcl")]
    AdverbialClause,
    /// Nominal modifier
    #[serde(rename = "nmod")]
    NominalModifier,
    /// Punctuation
    #[serde(rename = "punct")]
    Punctuation,
    /// Root
    #[serde(rename = "root")]
    Root,
    /// Flat (for names, etc.)
    #[serde(rename = "flat")]
    Flat,
    /// Numeric modifier
    #[serde(rename = "nummod")]
    NumericModifier,
    /// Parataxis
    #[serde(rename = "parataxis")]
    Parataxis,
    /// Expletive
    #[serde(rename = "expl")]
    Expletive,
    /// Adjectival clause
    #[serde(rename = "acl")]
    AdjectivalClause,
    /// Clausal subject
    #[serde(rename = "csubj")]
    ClausalSubject,
    /// Fixed multiword expression
    #[serde(rename = "fixed")]
    Fixed,
    /// Other relation (for extensibility)
    Other(String),
}

/// Dependency feature types from UD relation subtypes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DependencyFeatureType {
    /// Voice markers
    Voice(VoiceFeature),
    /// Semantic role markers
    SemanticRole(SemanticRoleFeature),
    /// Temporal markers
    Temporal(TemporalFeature),
    /// Syntactic markers
    Syntactic(SyntacticFeature),
    /// Other/unknown subtype
    Other(String),
}

/// Voice feature subtypes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VoiceFeature {
    /// Passive voice marker (:pass)
    Pass,
}

/// Semantic role feature subtypes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SemanticRoleFeature {
    /// Agent marker (:agent)
    Agent,
}

/// Temporal feature subtypes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TemporalFeature {
    /// Temporal modifier (:tmod)
    Tmod,
}

/// Syntactic feature subtypes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SyntacticFeature {
    /// Possessive (:poss)
    Poss,
    /// Pre-determiner (:predet)
    Predet,
    /// Particle (:prt)
    Prt,
    /// External subject (:xsubj)
    Xsubj,
    /// Outer clause (:outer)
    Outer,
    /// Relative clause (:relcl)
    Relcl,
    /// Descriptive (:desc)
    Desc,
    /// Unmarked (:unmarked)
    Unmarked,
}

/// Extracted linguistic features from dependency relation subtypes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DependencyFeatures {
    /// Parsed feature types from relation subtypes
    pub features: Vec<DependencyFeatureType>,
}

impl DependencyFeatures {
    /// Parse colon-separated subtypes from dependency relation string
    pub fn parse_subtypes(deprel_str: &str) -> (DependencyRelation, Self) {
        let parts: Vec<&str> = deprel_str.split(':').collect();
        let base_rel = parts[0];
        let subtypes = &parts[1..];

        let mut features = Vec::new();

        for subtype in subtypes {
            let feature = match *subtype {
                // Voice markers
                "pass" => DependencyFeatureType::Voice(VoiceFeature::Pass),

                // Semantic role markers
                "agent" => DependencyFeatureType::SemanticRole(SemanticRoleFeature::Agent),

                // Temporal markers
                "tmod" => DependencyFeatureType::Temporal(TemporalFeature::Tmod),

                // Syntactic markers
                "poss" => DependencyFeatureType::Syntactic(SyntacticFeature::Poss),
                "predet" => DependencyFeatureType::Syntactic(SyntacticFeature::Predet),
                "prt" => DependencyFeatureType::Syntactic(SyntacticFeature::Prt),
                "xsubj" => DependencyFeatureType::Syntactic(SyntacticFeature::Xsubj),
                "outer" => DependencyFeatureType::Syntactic(SyntacticFeature::Outer),
                "relcl" => DependencyFeatureType::Syntactic(SyntacticFeature::Relcl),
                "desc" => DependencyFeatureType::Syntactic(SyntacticFeature::Desc),
                "unmarked" => DependencyFeatureType::Syntactic(SyntacticFeature::Unmarked),

                // Unknown subtypes
                other => DependencyFeatureType::Other(other.to_string()),
            };
            features.push(feature);
        }

        let relation = DependencyRelation::from(base_rel);
        let dependency_features = Self { features };
        (relation, dependency_features)
    }

    /// Check if this has passive voice markers
    pub fn is_passive(&self) -> bool {
        self.features
            .iter()
            .any(|f| matches!(f, DependencyFeatureType::Voice(VoiceFeature::Pass)))
    }

    /// Check if this has agent markers
    pub fn is_agent(&self) -> bool {
        self.features.iter().any(|f| {
            matches!(
                f,
                DependencyFeatureType::SemanticRole(SemanticRoleFeature::Agent)
            )
        })
    }

    /// Get all voice features
    pub fn voice_features(&self) -> Vec<&VoiceFeature> {
        self.features
            .iter()
            .filter_map(|f| {
                if let DependencyFeatureType::Voice(voice) = f {
                    Some(voice)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all semantic role features
    pub fn semantic_role_features(&self) -> Vec<&SemanticRoleFeature> {
        self.features
            .iter()
            .filter_map(|f| {
                if let DependencyFeatureType::SemanticRole(role) = f {
                    Some(role)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl From<&str> for DependencyRelation {
    fn from(s: &str) -> Self {
        // Extract base relation (before first colon)
        let base_rel = s.split(':').next().unwrap_or(s);

        match base_rel {
            "nsubj" => Self::NominalSubject,
            "obj" => Self::Object,
            "iobj" => Self::IndirectObject,
            "obl" => Self::Oblique,
            "advmod" => Self::AdverbialModifier,
            "amod" => Self::AdjectivalModifier,
            "compound" => Self::Compound,
            "conj" => Self::Conjunction,
            "cc" => Self::CoordinatingConjunction,
            "det" => Self::Determiner,
            "case" => Self::Case,
            "aux" => Self::Auxiliary,
            "cop" => Self::Copula,
            "mark" => Self::Mark,
            "ccomp" => Self::ClausalComplement,
            "xcomp" => Self::XClausalComplement,
            "acl" => {
                // Special case for acl:relcl
                if s == "acl:relcl" {
                    Self::RelativeClause
                } else {
                    Self::AdjectivalClause
                }
            }
            "advcl" => Self::AdverbialClause,
            "nmod" => Self::NominalModifier,
            "punct" => Self::Punctuation,
            "root" => Self::Root,
            "flat" => Self::Flat,
            "nummod" => Self::NumericModifier,
            "parataxis" => Self::Parataxis,
            "expl" => Self::Expletive,
            "csubj" => Self::ClausalSubject,
            "fixed" => Self::Fixed,
            other => Self::Other(other.to_string()),
        }
    }
}

impl DependencyRelation {
    /// Convert to theta role mapping
    pub fn to_theta_role(&self) -> Option<ThetaRole> {
        match self {
            Self::NominalSubject => Some(ThetaRole::Agent),
            Self::Object => Some(ThetaRole::Patient),
            Self::IndirectObject => Some(ThetaRole::Recipient),
            Self::Oblique => Some(ThetaRole::Location),
            _ => None,
        }
    }
}

/// A dependency pattern extracted from treebank data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyPattern {
    /// Root verb lemma
    pub verb_lemma: String,
    /// List of dependency relations with their argument roles
    pub dependencies: Vec<(DependencyRelation, String)>,
    /// Pattern confidence score (0.0-1.0)
    pub confidence: f32,
    /// Frequency count in treebank
    pub frequency: u32,
    /// Pattern source information
    pub source: PatternSource,
}

impl DependencyPattern {
    /// Create a new dependency pattern
    pub fn new(
        verb_lemma: String,
        dependencies: Vec<(DependencyRelation, String)>,
        confidence: f32,
        frequency: u32,
        source: PatternSource,
    ) -> Self {
        Self {
            verb_lemma,
            dependencies,
            confidence,
            frequency,
            source,
        }
    }

    /// Check if pattern has a specific dependency relation
    pub fn has_relation(&self, relation: &DependencyRelation) -> bool {
        self.dependencies.iter().any(|(rel, _)| rel == relation)
    }

    /// Get argument for a specific dependency relation
    pub fn get_argument(&self, relation: &DependencyRelation) -> Option<&str> {
        self.dependencies
            .iter()
            .find(|(rel, _)| rel == relation)
            .map(|(_, arg)| arg.as_str())
    }

    /// Get all theta roles from dependencies
    pub fn get_theta_roles(&self) -> Vec<ThetaRole> {
        self.dependencies
            .iter()
            .filter_map(|(rel, _)| rel.to_theta_role())
            .collect()
    }
}

/// Source of a dependency pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternSource {
    /// Extracted directly from treebank index
    Indexed,
    /// Synthesized from VerbNet class
    VerbNet(String),
    /// Synthesized from FrameNet frame
    FrameNet(String),
    /// Default fallback pattern
    Default,
}

/// Complete treebank analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreebankAnalysis {
    /// Original word analyzed
    pub word: String,
    /// Primary dependency pattern (highest confidence)
    pub pattern: Option<DependencyPattern>,
    /// Alternative patterns with their confidence scores
    pub alternative_patterns: Vec<DependencyPattern>,
    /// Overall analysis confidence
    pub confidence: f32,
    /// Processing time in microseconds
    pub processing_time_us: u64,
    /// Whether result came from cache
    pub from_cache: bool,
}

impl TreebankAnalysis {
    /// Create a new treebank analysis
    pub fn new(
        word: String,
        pattern: Option<DependencyPattern>,
        confidence: f32,
        processing_time_us: u64,
        from_cache: bool,
    ) -> Self {
        Self {
            word,
            pattern,
            alternative_patterns: Vec::new(),
            confidence,
            processing_time_us,
            from_cache,
        }
    }

    /// Create analysis with multiple patterns
    pub fn with_alternatives(
        word: String,
        patterns: Vec<DependencyPattern>,
        processing_time_us: u64,
        from_cache: bool,
    ) -> Self {
        let (primary, alternatives) = if patterns.is_empty() {
            (None, Vec::new())
        } else {
            let mut sorted = patterns;
            sorted.sort_by(|a, b| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let primary = Some(sorted[0].clone());
            let alternatives = sorted[1..].to_vec();
            (primary, alternatives)
        };

        let overall_confidence = primary.as_ref().map(|p| p.confidence).unwrap_or(0.0);

        Self {
            word,
            pattern: primary,
            alternative_patterns: alternatives,
            confidence: overall_confidence,
            processing_time_us,
            from_cache,
        }
    }

    /// Create analysis with no pattern found
    pub fn no_pattern(word: String, processing_time_us: u64) -> Self {
        Self::new(word, None, 0.0, processing_time_us, false)
    }

    /// Get total number of patterns (primary + alternatives)
    pub fn total_patterns(&self) -> usize {
        (if self.pattern.is_some() { 1 } else { 0 }) + self.alternative_patterns.len()
    }

    /// Get all patterns sorted by confidence
    pub fn all_patterns(&self) -> Vec<&DependencyPattern> {
        let mut patterns = Vec::new();
        if let Some(ref p) = self.pattern {
            patterns.push(p);
        }
        patterns.extend(self.alternative_patterns.iter());
        patterns.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        patterns
    }

    /// Check if analysis has a dependency pattern
    pub fn has_pattern(&self) -> bool {
        self.pattern.is_some()
    }

    /// Get theta roles if pattern exists
    pub fn get_theta_roles(&self) -> Vec<ThetaRole> {
        self.pattern
            .as_ref()
            .map(|p| p.get_theta_roles())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === DependencyRelation From<&str> Tests ===

    #[test]
    fn test_dependency_relation_from_str() {
        assert_eq!(
            DependencyRelation::from("nsubj"),
            DependencyRelation::NominalSubject
        );
        assert_eq!(DependencyRelation::from("obj"), DependencyRelation::Object);
        assert_eq!(
            DependencyRelation::from("iobj"),
            DependencyRelation::IndirectObject
        );
        assert_eq!(DependencyRelation::from("obl"), DependencyRelation::Oblique);
        assert_eq!(
            DependencyRelation::from("advmod"),
            DependencyRelation::AdverbialModifier
        );
        assert_eq!(
            DependencyRelation::from("amod"),
            DependencyRelation::AdjectivalModifier
        );
        assert_eq!(
            DependencyRelation::from("compound"),
            DependencyRelation::Compound
        );
        assert_eq!(
            DependencyRelation::from("conj"),
            DependencyRelation::Conjunction
        );
        assert_eq!(
            DependencyRelation::from("cc"),
            DependencyRelation::CoordinatingConjunction
        );
        assert_eq!(
            DependencyRelation::from("det"),
            DependencyRelation::Determiner
        );
        assert_eq!(DependencyRelation::from("case"), DependencyRelation::Case);
        assert_eq!(
            DependencyRelation::from("aux"),
            DependencyRelation::Auxiliary
        );
        assert_eq!(DependencyRelation::from("cop"), DependencyRelation::Copula);
        assert_eq!(DependencyRelation::from("mark"), DependencyRelation::Mark);
        assert_eq!(
            DependencyRelation::from("ccomp"),
            DependencyRelation::ClausalComplement
        );
        assert_eq!(
            DependencyRelation::from("xcomp"),
            DependencyRelation::XClausalComplement
        );
        assert_eq!(
            DependencyRelation::from("advcl"),
            DependencyRelation::AdverbialClause
        );
        assert_eq!(
            DependencyRelation::from("nmod"),
            DependencyRelation::NominalModifier
        );
        assert_eq!(
            DependencyRelation::from("punct"),
            DependencyRelation::Punctuation
        );
        assert_eq!(DependencyRelation::from("root"), DependencyRelation::Root);
        assert_eq!(DependencyRelation::from("flat"), DependencyRelation::Flat);
        assert_eq!(
            DependencyRelation::from("nummod"),
            DependencyRelation::NumericModifier
        );
        assert_eq!(
            DependencyRelation::from("parataxis"),
            DependencyRelation::Parataxis
        );
        assert_eq!(
            DependencyRelation::from("expl"),
            DependencyRelation::Expletive
        );
        assert_eq!(
            DependencyRelation::from("csubj"),
            DependencyRelation::ClausalSubject
        );
        assert_eq!(DependencyRelation::from("fixed"), DependencyRelation::Fixed);
    }

    #[test]
    fn test_dependency_relation_from_str_acl() {
        assert_eq!(
            DependencyRelation::from("acl:relcl"),
            DependencyRelation::RelativeClause
        );
        assert_eq!(
            DependencyRelation::from("acl"),
            DependencyRelation::AdjectivalClause
        );
    }

    #[test]
    fn test_dependency_relation_from_str_other() {
        let rel = DependencyRelation::from("unknown_rel");
        assert!(matches!(rel, DependencyRelation::Other(_)));
        if let DependencyRelation::Other(s) = rel {
            assert_eq!(s, "unknown_rel");
        }
    }

    #[test]
    fn test_dependency_relation_from_str_with_subtype() {
        // Should extract base relation
        assert_eq!(
            DependencyRelation::from("nsubj:pass"),
            DependencyRelation::NominalSubject
        );
        assert_eq!(
            DependencyRelation::from("obl:agent"),
            DependencyRelation::Oblique
        );
    }

    // === DependencyRelation to_theta_role Tests ===

    #[test]
    fn test_dependency_relation_to_theta_role() {
        assert_eq!(
            DependencyRelation::NominalSubject.to_theta_role(),
            Some(ThetaRole::Agent)
        );
        assert_eq!(
            DependencyRelation::Object.to_theta_role(),
            Some(ThetaRole::Patient)
        );
        assert_eq!(
            DependencyRelation::IndirectObject.to_theta_role(),
            Some(ThetaRole::Recipient)
        );
        assert_eq!(
            DependencyRelation::Oblique.to_theta_role(),
            Some(ThetaRole::Location)
        );
        assert_eq!(DependencyRelation::Auxiliary.to_theta_role(), None);
        assert_eq!(DependencyRelation::Determiner.to_theta_role(), None);
    }

    // === DependencyFeatures Tests ===

    #[test]
    fn test_dependency_features_parse_subtypes_simple() {
        let (rel, features) = DependencyFeatures::parse_subtypes("nsubj");
        assert_eq!(rel, DependencyRelation::NominalSubject);
        assert!(features.features.is_empty());
    }

    #[test]
    fn test_dependency_features_parse_subtypes_passive() {
        let (rel, features) = DependencyFeatures::parse_subtypes("nsubj:pass");
        assert_eq!(rel, DependencyRelation::NominalSubject);
        assert_eq!(features.features.len(), 1);
        assert!(features.is_passive());
        assert!(!features.is_agent());
    }

    #[test]
    fn test_dependency_features_parse_subtypes_agent() {
        let (rel, features) = DependencyFeatures::parse_subtypes("obl:agent");
        assert_eq!(rel, DependencyRelation::Oblique);
        assert!(features.is_agent());
        assert!(!features.is_passive());
    }

    #[test]
    fn test_dependency_features_parse_subtypes_tmod() {
        let (_, features) = DependencyFeatures::parse_subtypes("obl:tmod");
        assert!(matches!(
            features.features[0],
            DependencyFeatureType::Temporal(TemporalFeature::Tmod)
        ));
    }

    #[test]
    fn test_dependency_features_parse_subtypes_syntactic() {
        let (_, features) = DependencyFeatures::parse_subtypes("nmod:poss");
        assert!(matches!(
            features.features[0],
            DependencyFeatureType::Syntactic(SyntacticFeature::Poss)
        ));

        let (_, features) = DependencyFeatures::parse_subtypes("det:predet");
        assert!(matches!(
            features.features[0],
            DependencyFeatureType::Syntactic(SyntacticFeature::Predet)
        ));

        let (_, features) = DependencyFeatures::parse_subtypes("compound:prt");
        assert!(matches!(
            features.features[0],
            DependencyFeatureType::Syntactic(SyntacticFeature::Prt)
        ));
    }

    #[test]
    fn test_dependency_features_parse_subtypes_unknown() {
        let (_, features) = DependencyFeatures::parse_subtypes("nmod:xyz");
        assert!(matches!(
            features.features[0],
            DependencyFeatureType::Other(_)
        ));
    }

    #[test]
    fn test_dependency_features_voice_features() {
        let (_, features) = DependencyFeatures::parse_subtypes("nsubj:pass");
        let voice = features.voice_features();
        assert_eq!(voice.len(), 1);
        assert!(matches!(voice[0], VoiceFeature::Pass));
    }

    #[test]
    fn test_dependency_features_semantic_role_features() {
        let (_, features) = DependencyFeatures::parse_subtypes("obl:agent");
        let roles = features.semantic_role_features();
        assert_eq!(roles.len(), 1);
        assert!(matches!(roles[0], SemanticRoleFeature::Agent));
    }

    // === DependencyPattern Tests ===

    #[test]
    fn test_dependency_pattern_new() {
        let pattern = DependencyPattern::new(
            "give".to_string(),
            vec![
                (DependencyRelation::NominalSubject, "agent".to_string()),
                (DependencyRelation::Object, "theme".to_string()),
            ],
            0.9,
            100,
            PatternSource::Indexed,
        );
        assert_eq!(pattern.verb_lemma, "give");
        assert_eq!(pattern.dependencies.len(), 2);
        assert_eq!(pattern.confidence, 0.9);
        assert_eq!(pattern.frequency, 100);
        assert!(matches!(pattern.source, PatternSource::Indexed));
    }

    #[test]
    fn test_dependency_pattern_has_relation() {
        let pattern = DependencyPattern::new(
            "run".to_string(),
            vec![(DependencyRelation::NominalSubject, "runner".to_string())],
            0.8,
            50,
            PatternSource::Default,
        );
        assert!(pattern.has_relation(&DependencyRelation::NominalSubject));
        assert!(!pattern.has_relation(&DependencyRelation::Object));
    }

    #[test]
    fn test_dependency_pattern_get_argument() {
        let pattern = DependencyPattern::new(
            "give".to_string(),
            vec![
                (DependencyRelation::NominalSubject, "giver".to_string()),
                (DependencyRelation::Object, "gift".to_string()),
            ],
            0.9,
            100,
            PatternSource::Indexed,
        );
        assert_eq!(
            pattern.get_argument(&DependencyRelation::NominalSubject),
            Some("giver")
        );
        assert_eq!(
            pattern.get_argument(&DependencyRelation::Object),
            Some("gift")
        );
        assert_eq!(
            pattern.get_argument(&DependencyRelation::IndirectObject),
            None
        );
    }

    #[test]
    fn test_dependency_pattern_get_theta_roles() {
        let pattern = DependencyPattern::new(
            "give".to_string(),
            vec![
                (DependencyRelation::NominalSubject, "agent".to_string()),
                (DependencyRelation::Object, "theme".to_string()),
                (DependencyRelation::IndirectObject, "recipient".to_string()),
            ],
            0.9,
            100,
            PatternSource::Indexed,
        );
        let roles = pattern.get_theta_roles();
        assert_eq!(roles.len(), 3);
        assert!(roles.contains(&ThetaRole::Agent));
        assert!(roles.contains(&ThetaRole::Patient));
        assert!(roles.contains(&ThetaRole::Recipient));
    }

    // === PatternSource Tests ===

    #[test]
    fn test_pattern_source_variants() {
        let indexed = PatternSource::Indexed;
        assert!(matches!(indexed, PatternSource::Indexed));

        let verbnet = PatternSource::VerbNet("give-13.1".to_string());
        if let PatternSource::VerbNet(class) = verbnet {
            assert_eq!(class, "give-13.1");
        }

        let framenet = PatternSource::FrameNet("Giving".to_string());
        if let PatternSource::FrameNet(frame) = framenet {
            assert_eq!(frame, "Giving");
        }

        let default = PatternSource::Default;
        assert!(matches!(default, PatternSource::Default));
    }

    // === TreebankAnalysis Tests ===

    #[test]
    fn test_treebank_analysis_new() {
        let pattern = DependencyPattern::new(
            "run".to_string(),
            vec![(DependencyRelation::NominalSubject, "runner".to_string())],
            0.8,
            50,
            PatternSource::Indexed,
        );
        let analysis = TreebankAnalysis::new("running".to_string(), Some(pattern), 0.8, 100, true);
        assert_eq!(analysis.word, "running");
        assert!(analysis.pattern.is_some());
        assert!(analysis.alternative_patterns.is_empty());
        assert_eq!(analysis.confidence, 0.8);
        assert_eq!(analysis.processing_time_us, 100);
        assert!(analysis.from_cache);
    }

    #[test]
    fn test_treebank_analysis_no_pattern() {
        let analysis = TreebankAnalysis::no_pattern("xyz".to_string(), 50);
        assert_eq!(analysis.word, "xyz");
        assert!(analysis.pattern.is_none());
        assert_eq!(analysis.confidence, 0.0);
        assert_eq!(analysis.processing_time_us, 50);
        assert!(!analysis.from_cache);
    }

    #[test]
    fn test_treebank_analysis_with_alternatives() {
        let patterns = vec![
            DependencyPattern::new(
                "run".to_string(),
                vec![(DependencyRelation::NominalSubject, "runner".to_string())],
                0.6,
                30,
                PatternSource::Default,
            ),
            DependencyPattern::new(
                "run".to_string(),
                vec![
                    (DependencyRelation::NominalSubject, "runner".to_string()),
                    (DependencyRelation::Oblique, "location".to_string()),
                ],
                0.9,
                80,
                PatternSource::Indexed,
            ),
        ];

        let analysis = TreebankAnalysis::with_alternatives("run".to_string(), patterns, 100, false);

        // Primary should be the highest confidence (0.9)
        assert!(analysis.pattern.is_some());
        assert_eq!(analysis.pattern.as_ref().unwrap().confidence, 0.9);
        assert_eq!(analysis.alternative_patterns.len(), 1);
        assert_eq!(analysis.alternative_patterns[0].confidence, 0.6);
        assert_eq!(analysis.confidence, 0.9);
    }

    #[test]
    fn test_treebank_analysis_with_alternatives_empty() {
        let analysis = TreebankAnalysis::with_alternatives("empty".to_string(), vec![], 50, false);
        assert!(analysis.pattern.is_none());
        assert!(analysis.alternative_patterns.is_empty());
        assert_eq!(analysis.confidence, 0.0);
    }

    #[test]
    fn test_treebank_analysis_total_patterns() {
        let analysis = TreebankAnalysis::no_pattern("test".to_string(), 0);
        assert_eq!(analysis.total_patterns(), 0);

        let pattern =
            DependencyPattern::new("run".to_string(), vec![], 0.8, 10, PatternSource::Default);
        let analysis = TreebankAnalysis::new("run".to_string(), Some(pattern), 0.8, 0, false);
        assert_eq!(analysis.total_patterns(), 1);

        let patterns = vec![
            DependencyPattern::new("a".to_string(), vec![], 0.9, 10, PatternSource::Indexed),
            DependencyPattern::new("b".to_string(), vec![], 0.8, 10, PatternSource::Default),
            DependencyPattern::new("c".to_string(), vec![], 0.7, 10, PatternSource::Default),
        ];
        let analysis = TreebankAnalysis::with_alternatives("test".to_string(), patterns, 0, false);
        assert_eq!(analysis.total_patterns(), 3);
    }

    #[test]
    fn test_treebank_analysis_all_patterns() {
        let patterns = vec![
            DependencyPattern::new("a".to_string(), vec![], 0.7, 10, PatternSource::Default),
            DependencyPattern::new("b".to_string(), vec![], 0.9, 10, PatternSource::Indexed),
            DependencyPattern::new("c".to_string(), vec![], 0.8, 10, PatternSource::Default),
        ];
        let analysis = TreebankAnalysis::with_alternatives("test".to_string(), patterns, 0, false);
        let all = analysis.all_patterns();
        assert_eq!(all.len(), 3);
        // Should be sorted by confidence descending
        assert_eq!(all[0].confidence, 0.9);
        assert_eq!(all[1].confidence, 0.8);
        assert_eq!(all[2].confidence, 0.7);
    }

    #[test]
    fn test_treebank_analysis_has_pattern() {
        let analysis = TreebankAnalysis::no_pattern("test".to_string(), 0);
        assert!(!analysis.has_pattern());

        let pattern =
            DependencyPattern::new("run".to_string(), vec![], 0.8, 10, PatternSource::Default);
        let analysis = TreebankAnalysis::new("run".to_string(), Some(pattern), 0.8, 0, false);
        assert!(analysis.has_pattern());
    }

    #[test]
    fn test_treebank_analysis_get_theta_roles() {
        let analysis = TreebankAnalysis::no_pattern("test".to_string(), 0);
        assert!(analysis.get_theta_roles().is_empty());

        let pattern = DependencyPattern::new(
            "give".to_string(),
            vec![
                (DependencyRelation::NominalSubject, "agent".to_string()),
                (DependencyRelation::Object, "theme".to_string()),
            ],
            0.9,
            100,
            PatternSource::Indexed,
        );
        let analysis = TreebankAnalysis::new("give".to_string(), Some(pattern), 0.9, 0, false);
        let roles = analysis.get_theta_roles();
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&ThetaRole::Agent));
        assert!(roles.contains(&ThetaRole::Patient));
    }

    // === Serialization Tests ===

    #[test]
    fn test_dependency_relation_serialization() {
        let rel = DependencyRelation::NominalSubject;
        let json = serde_json::to_string(&rel).unwrap();
        let deserialized: DependencyRelation = serde_json::from_str(&json).unwrap();
        assert_eq!(rel, deserialized);
    }

    #[test]
    fn test_dependency_pattern_serialization() {
        let pattern = DependencyPattern::new(
            "run".to_string(),
            vec![(DependencyRelation::NominalSubject, "runner".to_string())],
            0.8,
            50,
            PatternSource::Indexed,
        );
        let json = serde_json::to_string(&pattern).unwrap();
        let deserialized: DependencyPattern = serde_json::from_str(&json).unwrap();
        assert_eq!(pattern.verb_lemma, deserialized.verb_lemma);
        assert_eq!(pattern.confidence, deserialized.confidence);
    }

    #[test]
    fn test_treebank_analysis_serialization() {
        let analysis = TreebankAnalysis::no_pattern("test".to_string(), 100);
        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: TreebankAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(analysis.word, deserialized.word);
        assert_eq!(analysis.confidence, deserialized.confidence);
    }

    // === Default Tests ===

    #[test]
    fn test_dependency_features_default() {
        let features = DependencyFeatures::default();
        assert!(features.features.is_empty());
        assert!(!features.is_passive());
        assert!(!features.is_agent());
    }
}
