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
