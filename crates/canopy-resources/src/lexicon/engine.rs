//! Canopy Lexicon Engine
//!
//! This module provides the main lexicon engine that implements canopy-engine traits
//! for classification and analysis of closed-class words and functional lexical items.

use super::parser::LexiconXmlResource;
use super::types::{
    LexiconAnalysis, LexiconDatabase, Person, PronounCase, PronounFeatures, PronounGender,
    PronounNumber, WordClassType,
};
use crate::engine::{
    BaseEngine, CacheKeyFormat, CachedEngine, DataInfo, DataLoader, EngineConfigurable, EngineCore,
    EngineResult, EngineStats, PerformanceMetrics, SemanticEngine, SemanticResult,
    StatisticsProvider, XmlParser, XmlResource,
};
use crate::paths::data_path_string;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Lookup pronoun features based on word form (first person)
fn lookup_first_person(word: &str) -> Option<PronounFeatures> {
    Some(match word {
        "i" => PronounFeatures {
            person: Some(Person::First),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Nominative),
        },
        "me" => PronounFeatures {
            person: Some(Person::First),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Accusative),
        },
        "my" | "mine" => PronounFeatures {
            person: Some(Person::First),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Genitive),
        },
        "myself" => PronounFeatures {
            person: Some(Person::First),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Reflexive),
        },
        "we" => PronounFeatures {
            person: Some(Person::First),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Nominative),
        },
        "us" => PronounFeatures {
            person: Some(Person::First),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Accusative),
        },
        "our" | "ours" => PronounFeatures {
            person: Some(Person::First),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Genitive),
        },
        "ourselves" => PronounFeatures {
            person: Some(Person::First),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Reflexive),
        },
        _ => return None,
    })
}

/// Lookup pronoun features based on word form (second person)
fn lookup_second_person(word: &str) -> Option<PronounFeatures> {
    Some(match word {
        "you" => PronounFeatures {
            person: Some(Person::Second),
            number: None,
            gender: Some(PronounGender::Unknown),
            case: None,
        },
        "your" | "yours" => PronounFeatures {
            person: Some(Person::Second),
            number: None,
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Genitive),
        },
        "yourself" => PronounFeatures {
            person: Some(Person::Second),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Reflexive),
        },
        "yourselves" => PronounFeatures {
            person: Some(Person::Second),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Reflexive),
        },
        _ => return None,
    })
}

/// Lookup pronoun features based on word form (third person)
fn lookup_third_person(word: &str) -> Option<PronounFeatures> {
    Some(match word {
        "he" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Masculine),
            case: Some(PronounCase::Nominative),
        },
        "him" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Masculine),
            case: Some(PronounCase::Accusative),
        },
        "his" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Masculine),
            case: Some(PronounCase::Genitive),
        },
        "himself" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Masculine),
            case: Some(PronounCase::Reflexive),
        },
        "she" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Feminine),
            case: Some(PronounCase::Nominative),
        },
        "her" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Feminine),
            case: None,
        },
        "hers" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Feminine),
            case: Some(PronounCase::Genitive),
        },
        "herself" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Feminine),
            case: Some(PronounCase::Reflexive),
        },
        "it" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Neuter),
            case: None,
        },
        "its" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Neuter),
            case: Some(PronounCase::Genitive),
        },
        "itself" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Singular),
            gender: Some(PronounGender::Neuter),
            case: Some(PronounCase::Reflexive),
        },
        "they" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Nominative),
        },
        "them" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Accusative),
        },
        "their" | "theirs" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Genitive),
        },
        "themselves" => PronounFeatures {
            person: Some(Person::Third),
            number: Some(PronounNumber::Plural),
            gender: Some(PronounGender::Unknown),
            case: Some(PronounCase::Reflexive),
        },
        _ => return None,
    })
}

/// Lookup pronoun features for a word (English personal pronoun paradigm)
fn lookup_pronoun_features(word: &str) -> PronounFeatures {
    lookup_first_person(word)
        .or_else(|| lookup_second_person(word))
        .or_else(|| lookup_third_person(word))
        .unwrap_or(PronounFeatures {
            person: None,
            number: None,
            gender: None,
            case: None,
        })
}

/// Input type for Lexicon analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconInput {
    pub word: String,
}

impl Hash for LexiconInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word.hash(state);
    }
}

/// Configuration for Lexicon engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconConfig {
    /// Path to lexicon data directory
    pub data_path: String,
    /// Enable pattern matching
    pub enable_patterns: bool,
    /// Maximum number of classifications per word
    pub max_classifications: usize,
    /// Minimum confidence threshold for results
    pub min_confidence: f32,
    /// Enable fuzzy matching
    pub enable_fuzzy_matching: bool,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache capacity
    pub cache_capacity: usize,
}

impl Default for LexiconConfig {
    fn default() -> Self {
        Self {
            data_path: data_path_string("data/lexicon"),
            enable_patterns: true,
            max_classifications: 10,
            min_confidence: 0.1,
            enable_fuzzy_matching: false,
            enable_cache: true,
            cache_capacity: 10000,
        }
    }
}

// Implement EngineConfigurable trait via macro (uses min_confidence field)
crate::impl_engine_configurable!(LexiconConfig, min_confidence);

/// Canopy Lexicon Engine
#[derive(Debug)]
pub struct LexiconEngine {
    /// Base engine handling cache, stats, and metrics
    base_engine: BaseEngine<LexiconInput, LexiconAnalysis>,
    /// Lexicon database
    database: Arc<LexiconDatabase>,
    /// Lexicon-specific configuration
    lexicon_config: LexiconConfig,
    /// Is data loaded flag
    is_loaded: bool,
    /// Data-driven pronoun features, built from XML at load time
    pronoun_features: std::collections::HashMap<String, PronounFeatures>,
}

impl LexiconEngine {
    /// Create a new lexicon engine
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(LexiconConfig::default())
    }

    /// Create a new lexicon engine with custom configuration
    #[must_use]
    pub fn with_config(lexicon_config: LexiconConfig) -> Self {
        // Convert LexiconConfig to EngineConfig using trait
        let engine_config = lexicon_config.to_engine_config();

        Self {
            base_engine: BaseEngine::new(engine_config, "Lexicon".to_string()),
            database: Arc::new(LexiconDatabase::new()),
            lexicon_config,
            is_loaded: false,
            pronoun_features: std::collections::HashMap::new(),
        }
    }

    /// Analyze a word and return lexical classifications
    ///
    /// # Errors
    /// Returns an error if analysis fails.
    pub fn analyze_word(&self, word: &str) -> EngineResult<SemanticResult<LexiconAnalysis>> {
        let input = LexiconInput {
            word: word.to_string(),
        };
        self.base_engine.analyze(&input, self)
    }

    /// Load lexicon data from the configured path
    ///
    /// # Errors
    /// Returns an error if data file is not found or cannot be parsed.
    pub fn load_data(&mut self) -> EngineResult<()> {
        let data_file = Path::new(&self.lexicon_config.data_path).join("english-lexicon.xml");
        if !data_file.exists() {
            return Err(crate::engine::EngineError::data_load(format!(
                "Lexicon data file not found: {}",
                data_file.display()
            )));
        }

        let parser = XmlParser::new();
        let resource = parser.parse_file::<LexiconXmlResource>(&data_file)?;
        resource.validate()?;

        self.database = Arc::new(resource.database);
        self.is_loaded = true;
        self.pronoun_features = Self::build_pronoun_features(&self.database);

        let stats = self.database.stats();
        info!(
            "Lexicon database loaded with {} word classes, {} words, {} patterns ({} pronoun features)",
            stats.total_word_classes, stats.total_words, stats.total_patterns,
            self.pronoun_features.len()
        );

        Ok(())
    }

    /// Build pronoun features map from loaded XML data.
    fn build_pronoun_features(
        database: &LexiconDatabase,
    ) -> std::collections::HashMap<String, PronounFeatures> {
        let mut map = std::collections::HashMap::new();

        for wc in &database.word_classes {
            if wc.word_class_type != WordClassType::Pronouns {
                continue;
            }

            for word in &wc.words {
                let person = word.person.as_deref().and_then(|p| match p {
                    "1" => Some(Person::First),
                    "2" => Some(Person::Second),
                    "3" => Some(Person::Third),
                    _ => None,
                });
                let number = word.number.as_deref().and_then(|n| match n {
                    "singular" => Some(PronounNumber::Singular),
                    "plural" => Some(PronounNumber::Plural),
                    _ => None,
                });
                let case = word.case.as_deref().and_then(|c| match c {
                    "nominative" => Some(PronounCase::Nominative),
                    "accusative" => Some(PronounCase::Accusative),
                    "genitive" => Some(PronounCase::Genitive),
                    "reflexive" => Some(PronounCase::Reflexive),
                    _ => None,
                });
                let gender = word.gender.as_deref().map(|g| match g {
                    "masculine" => PronounGender::Masculine,
                    "feminine" => PronounGender::Feminine,
                    "neuter" => PronounGender::Neuter,
                    _ => PronounGender::Unknown,
                });

                let features = PronounFeatures {
                    person,
                    number,
                    gender,
                    case,
                };

                // Only insert if we have at least one feature
                if features.person.is_some()
                    || features.number.is_some()
                    || features.gender.is_some()
                    || features.case.is_some()
                {
                    map.insert(word.word.to_lowercase(), features);
                }
            }
        }

        map
    }

    /// Check if a word is a stop word
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_stop_word(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.data.get_stop_words().is_empty())
    }

    /// Check if a word is a negation indicator
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_negation(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.data.get_negations().is_empty())
    }

    /// Check if a word is a discourse marker
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_discourse_marker(&self, word: &str) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(!analysis.data.get_discourse_markers().is_empty())
    }

    /// Check if a word is a pronoun
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_pronoun(&self, word: &str) -> EngineResult<bool> {
        self.is_word_class(word, WordClassType::Pronouns)
    }

    /// Check if a word is an auxiliary verb (be, have, do and their forms)
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_auxiliary(&self, word: &str) -> EngineResult<bool> {
        self.is_word_class(word, WordClassType::Auxiliary)
    }

    /// Check if a word is a modal verb (can, could, will, would, shall, should, may, might, must)
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_modal(&self, word: &str) -> EngineResult<bool> {
        self.is_word_class(word, WordClassType::Modal)
    }

    /// Check if a word is a preposition
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_preposition(&self, word: &str) -> EngineResult<bool> {
        self.is_word_class(word, WordClassType::Prepositions)
    }

    /// Check if a word is a conjunction
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_conjunction(&self, word: &str) -> EngineResult<bool> {
        self.is_word_class(word, WordClassType::Conjunctions)
    }

    /// Check if a word is a wh-word (who, what, where, when, why, how, which, whose, whom)
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_wh_word(&self, word: &str) -> EngineResult<bool> {
        self.is_word_class(word, WordClassType::WhWords)
    }

    /// Check if a word is a quantifier
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_quantifier(&self, word: &str) -> EngineResult<bool> {
        self.is_word_class(word, WordClassType::Quantifiers)
    }

    /// Check if a word is a verb particle (up, down, out, off, etc.)
    ///
    /// Particles are words that combine with verbs to form phrasal verbs
    /// like "give up", "turn off", "look forward to".
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn is_particle(&self, word: &str) -> EngineResult<bool> {
        self.is_word_class(word, WordClassType::Particles)
    }

    /// Helper: check if a word belongs to a specific word class type
    fn is_word_class(&self, word: &str, class_type: WordClassType) -> EngineResult<bool> {
        let analysis = self.analyze_word(word)?;
        Ok(analysis
            .data
            .classifications
            .iter()
            .any(|c| c.word_class_type == class_type))
    }

    /// Get pronoun features (person, number, gender, case) if the word is a pronoun
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn get_pronoun_features(
        &self,
        word: &str,
    ) -> EngineResult<Option<super::types::PronounFeatures>> {
        if !self.is_pronoun(word)? {
            return Ok(None);
        }
        let lower = word.to_lowercase();
        // Use data-driven features from XML, fall back to hardcoded for unloaded state
        if let Some(features) = self.pronoun_features.get(&lower) {
            Ok(Some(features.clone()))
        } else {
            Ok(Some(lookup_pronoun_features(&lower)))
        }
    }

    /// Get the discourse relation for a word if it's a discourse marker
    /// Maps context values from lexicon XML to `DiscourseRelation` enum
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn get_discourse_relation(
        &self,
        word: &str,
    ) -> EngineResult<Option<canopy::runtime::DiscourseRelation>> {
        use canopy::runtime::DiscourseRelation;

        let analysis = self.analyze_word(word)?;

        // Find discourse marker classifications
        for classification in &analysis.data.classifications {
            if classification.word_class_type == WordClassType::DiscourseMarkers {
                if let Some(ref context) = classification.context {
                    // Map context to DiscourseRelation
                    let relation = match context.as_str() {
                        "temporal" => Some(DiscourseRelation::Temporal),
                        "causal" => Some(DiscourseRelation::Cause),
                        "contrastive" | "contrast" => Some(DiscourseRelation::Contrast),
                        "concessive" | "concession" => Some(DiscourseRelation::Concession),
                        "conditional" | "condition" => Some(DiscourseRelation::Condition),
                        "additive" | "addition" => Some(DiscourseRelation::Addition),
                        "elaborative" | "elaboration" | "conclusive" => {
                            Some(DiscourseRelation::Elaboration)
                        }
                        _ => None,
                    };
                    if relation.is_some() {
                        return Ok(relation);
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get all words of a specific class type
    ///
    /// # Errors
    /// Returns an error if the lexicon database is not loaded.
    pub fn get_words_by_type(&self, class_type: WordClassType) -> EngineResult<Vec<String>> {
        if !self.is_loaded {
            return Err(crate::engine::EngineError::data_load(
                "Lexicon database not loaded".to_string(),
            ));
        }

        let mut words = Vec::new();
        let classes = self.database.get_classes_by_type(&class_type);

        for word_class in classes {
            for word in &word_class.words {
                words.push(word.word.clone());
                words.extend(word.variants.clone());
            }
        }

        words.sort();
        words.dedup();
        Ok(words)
    }

    /// Analyze multiple words in a text
    ///
    /// # Errors
    /// Returns an error if analysis of any word fails.
    pub fn analyze_text(&self, text: &str) -> EngineResult<Vec<LexiconAnalysis>> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut results = Vec::new();

        for word in words {
            // Clean word of punctuation
            let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());
            if !clean_word.is_empty() {
                let analysis = self.analyze_word(clean_word)?;
                if analysis.data.has_results() {
                    results.push(analysis.data);
                }
            }
        }

        Ok(results)
    }

    /// Get semantic weight for a word (useful for stop word filtering)
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn get_semantic_weight(&self, word: &str) -> EngineResult<f64> {
        let analysis = self.analyze_word(word)?;

        if analysis.data.classifications.is_empty() {
            return Ok(1.0); // Default weight for unknown words
        }

        // Use the weight from the highest priority classification
        let weight = analysis
            .data
            .classifications
            .first()
            .map_or(1.0, super::types::WordClassification::semantic_weight);

        Ok(weight)
    }

    // Backward compatibility methods for BaseEngine integration
    #[must_use]
    pub fn config(&self) -> &LexiconConfig {
        &self.lexicon_config
    }

    #[must_use]
    pub fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }

    #[must_use]
    pub fn cache_stats(&self) -> crate::engine::CacheStats {
        self.base_engine.cache_stats()
    }

    /// Clear the analysis cache
    ///
    /// # Errors
    /// This function currently cannot fail but returns `Result` for API consistency.
    pub fn clear_cache(&mut self) -> EngineResult<()> {
        self.base_engine.clear_cache();
        Ok(())
    }
}

// EngineCore trait implementation for BaseEngine integration
impl EngineCore<LexiconInput, LexiconAnalysis> for LexiconEngine {
    fn perform_analysis(&self, input: &LexiconInput) -> EngineResult<LexiconAnalysis> {
        if !self.is_loaded {
            debug!("Lexicon database not loaded for analysis");
            return Ok(LexiconAnalysis::new(input.word.clone()));
        }

        let mut analysis = LexiconAnalysis::new(input.word.clone());

        // Get exact word classifications
        analysis.classifications = self.database.classify_word(&input.word);

        // Get pattern matches if enabled
        if self.lexicon_config.enable_patterns {
            analysis.pattern_matches = self.database.analyze_patterns(&input.word);
        }

        // Filter by confidence threshold
        analysis
            .classifications
            .retain(|c| c.confidence >= self.lexicon_config.min_confidence);
        analysis
            .pattern_matches
            .retain(|p| p.confidence >= self.lexicon_config.min_confidence);

        // Limit results
        analysis
            .classifications
            .truncate(self.lexicon_config.max_classifications);
        analysis
            .pattern_matches
            .truncate(self.lexicon_config.max_classifications);

        // Calculate overall confidence
        analysis.calculate_confidence();

        debug!(
            "Lexicon analysis for '{}': {} classifications, {} patterns, confidence: {:.2}",
            input.word,
            analysis.classifications.len(),
            analysis.pattern_matches.len(),
            analysis.confidence
        );

        Ok(analysis)
    }

    fn calculate_confidence(&self, _input: &LexiconInput, output: &LexiconAnalysis) -> f32 {
        output.confidence
    }

    fn generate_cache_key(&self, input: &LexiconInput) -> String {
        CacheKeyFormat::Typed("lexicon".to_string(), input.word.to_lowercase()).to_string()
    }

    fn engine_name(&self) -> &'static str {
        "Lexicon"
    }

    fn engine_version(&self) -> &'static str {
        "1.0"
    }

    fn is_initialized(&self) -> bool {
        self.is_loaded
    }
}

impl SemanticEngine for LexiconEngine {
    type Input = String;
    type Output = LexiconAnalysis;
    type Config = LexiconConfig;

    fn analyze(&self, input: &Self::Input) -> EngineResult<SemanticResult<Self::Output>> {
        let lexicon_input = LexiconInput {
            word: input.clone(),
        };
        self.base_engine.analyze(&lexicon_input, self)
    }

    fn name(&self) -> &'static str {
        "Lexicon"
    }

    fn version(&self) -> &'static str {
        "1.0"
    }

    fn is_initialized(&self) -> bool {
        self.is_loaded
    }

    fn config(&self) -> &Self::Config {
        &self.lexicon_config
    }
}

impl CachedEngine for LexiconEngine {
    fn cache_stats(&self) -> crate::engine::CacheStats {
        self.base_engine.cache_stats()
    }

    fn clear_cache(&self) {
        self.base_engine.clear_cache();
    }

    fn set_cache_capacity(&mut self, capacity: usize) {
        self.lexicon_config.cache_capacity = capacity;
    }
}

impl StatisticsProvider for LexiconEngine {
    fn statistics(&self) -> EngineStats {
        self.base_engine.get_stats()
    }

    fn performance_metrics(&self) -> PerformanceMetrics {
        self.base_engine.get_performance_metrics()
    }
}

impl DataLoader for LexiconEngine {
    fn load_from_directory<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<()> {
        let path = path.as_ref();
        info!("Loading Lexicon data from: {}", path.display());

        self.lexicon_config.data_path = path.to_string_lossy().to_string();
        self.load_data()
    }

    fn load_test_data(&mut self) -> EngineResult<()> {
        // Create minimal test data
        self.database = Arc::new(LexiconDatabase::new());
        self.is_loaded = true;
        Ok(())
    }

    fn reload(&mut self) -> EngineResult<()> {
        self.is_loaded = false;
        self.database = Arc::new(LexiconDatabase::new());
        self.load_data()
    }

    fn data_info(&self) -> DataInfo {
        if self.is_loaded {
            let stats = self.database.stats();
            DataInfo::new(
                format!(
                    "lexicon: {}/english-lexicon.xml",
                    self.lexicon_config.data_path
                ),
                stats.total_words,
            )
        } else {
            DataInfo::new("Not loaded".to_string(), 0)
        }
    }
}

/// Specialized analysis methods
impl LexiconEngine {
    /// Analyze negation scope in a sentence
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn analyze_negation_scope(&self, text: &str) -> EngineResult<Vec<(String, usize, usize)>> {
        let mut negations = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        for word in &words {
            let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());
            if self.is_negation(clean_word)? {
                // Calculate byte positions
                let start_byte = text.find(word).unwrap_or(0);
                let end_byte = start_byte + word.len();
                negations.push((clean_word.to_string(), start_byte, end_byte));
            }
        }

        Ok(negations)
    }

    /// Extract discourse structure from text
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn extract_discourse_structure(&self, text: &str) -> EngineResult<Vec<(String, String)>> {
        let mut discourse_markers = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        for word in words {
            let clean_word = word.trim_matches(|c: char| c.is_ascii_punctuation());
            let analysis = self.analyze_word(clean_word)?;

            for marker in analysis.data.get_discourse_markers() {
                if let Some(context) = &marker.context {
                    discourse_markers.push((clean_word.to_string(), context.clone()));
                }
            }
        }

        Ok(discourse_markers)
    }

    /// Filter stop words from a list
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn filter_stop_words(&self, words: &[String]) -> EngineResult<Vec<String>> {
        let mut filtered = Vec::new();

        for word in words {
            if !self.is_stop_word(word)? {
                filtered.push(word.clone());
            }
        }

        Ok(filtered)
    }

    /// Get intensifier strength for a word
    ///
    /// # Errors
    /// Returns an error if word analysis fails.
    pub fn get_intensifier_strength(&self, word: &str) -> EngineResult<Option<String>> {
        let analysis = self.analyze_word(word)?;

        for classification in &analysis.data.classifications {
            if matches!(classification.word_class_type, WordClassType::Intensifiers) {
                return Ok(classification.context.clone());
            }
        }

        Ok(None)
    }
}

impl Default for LexiconEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_lexicon() -> (TempDir, LexiconConfig) {
        let temp_dir = TempDir::new().unwrap();
        let lexicon_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Test Lexicon</title>
    <description>Test lexicon for unit tests</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>

  <word-classes>
    <word-class id="test-stop-words" name="Test Stop Words" type="stop-words" priority="10">
      <description>Test stop words</description>
      <properties>
        <property name="semantic-weight" value="0.1" type="float"/>
      </properties>
      <words>
        <word pos="DT">the</word>
        <word pos="DT">a</word>
        <word pos="CC">and</word>
      </words>
    </word-class>

    <word-class id="test-negation" name="Test Negation" type="negation" priority="9">
      <description>Test negation words</description>
      <words>
        <word pos="RB">not</word>
        <word pos="DT">no</word>
      </words>
      <patterns>
        <pattern id="neg-prefix-un" type="prefix" confidence="0.8">
          <regex>^un[a-z]+</regex>
          <description>Un- prefix</description>
          <examples>
            <example>unhappy</example>
          </examples>
        </pattern>
      </patterns>
    </word-class>
  </word-classes>
</lexicon>"#;

        fs::write(temp_dir.path().join("english-lexicon.xml"), lexicon_xml).unwrap();

        let config = LexiconConfig {
            data_path: temp_dir.path().to_string_lossy().to_string(),
            ..LexiconConfig::default()
        };

        (temp_dir, config)
    }

    #[test]
    fn test_lexicon_loading() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);

        engine.load_data().expect("Failed to load test lexicon");
        assert!(SemanticEngine::is_initialized(&engine));

        let stats = engine.database.stats();
        assert_eq!(stats.total_word_classes, 2);
        assert_eq!(stats.total_words, 5);
        assert_eq!(stats.total_patterns, 1);
    }

    #[test]
    fn test_word_classification() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        // Test stop word
        assert!(engine.is_stop_word("the").unwrap());
        assert!(engine.is_stop_word("and").unwrap());
        assert!(!engine.is_stop_word("happy").unwrap());

        // Test negation
        assert!(engine.is_negation("not").unwrap());
        assert!(engine.is_negation("no").unwrap());
        assert!(!engine.is_negation("yes").unwrap());
    }

    #[test]
    fn test_pattern_matching() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        let analysis = engine.analyze_word("unhappy").unwrap();
        assert!(!analysis.data.pattern_matches.is_empty());

        let pattern_match = &analysis.data.pattern_matches[0];
        assert_eq!(pattern_match.pattern_id, "neg-prefix-un");
        assert_eq!(pattern_match.matched_text, "unhappy");
    }

    #[test]
    fn test_semantic_engine_trait() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        let result = engine.analyze(&"the".to_string()).unwrap();
        assert!(result.data.has_results());
        assert!(result.confidence > 0.0);

        assert_eq!(engine.name(), "Lexicon");
        assert_eq!(engine.version(), "1.0");
    }

    #[test]
    fn test_lexicon_engine_new() {
        let engine = LexiconEngine::new();
        assert!(!SemanticEngine::is_initialized(&engine));
        assert_eq!(engine.name(), "Lexicon");
    }

    #[test]
    fn test_lexicon_engine_default() {
        let engine = LexiconEngine::default();
        assert!(!SemanticEngine::is_initialized(&engine));
    }

    #[test]
    fn test_lexicon_config_default() {
        let config = LexiconConfig::default();
        assert!(config.enable_patterns);
        assert!(config.enable_cache);
        assert_eq!(config.max_classifications, 10);
        assert_eq!(config.cache_capacity, 10000);
    }

    #[test]
    fn test_lexicon_engine_config() {
        let config = LexiconConfig {
            enable_patterns: false,
            max_classifications: 5,
            ..LexiconConfig::default()
        };
        let engine = LexiconEngine::with_config(config);
        let stored_config = engine.config();
        assert!(!stored_config.enable_patterns);
        assert_eq!(stored_config.max_classifications, 5);
    }

    #[test]
    fn test_lexicon_engine_not_loaded() {
        let engine = LexiconEngine::new();
        // Getting words by type should fail when not loaded
        let result = engine.get_words_by_type(WordClassType::StopWords);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_text() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        let results = engine.analyze_text("the quick and brown").unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_analyze_text_empty() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        let results = engine.analyze_text("").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_is_discourse_marker() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        // "the" is not a discourse marker
        assert!(!engine.is_discourse_marker("the").unwrap());
    }

    #[test]
    fn test_get_semantic_weight() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        // Stop words should have low semantic weight
        let weight = engine.get_semantic_weight("the").unwrap();
        assert!(weight < 1.0, "Expected weight < 1.0, got {weight}");

        // Unknown words should have default weight of 1.0
        let unknown_weight = engine.get_semantic_weight("xyzunknown").unwrap();
        assert!((unknown_weight - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_words_by_type() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        let words = engine.get_words_by_type(WordClassType::StopWords).unwrap();
        assert!(!words.is_empty());
        assert!(words.contains(&"the".to_string()));
    }

    #[test]
    fn test_cache_operations() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        // Perform some analyses to populate cache
        engine.analyze_word("the").unwrap();
        engine.analyze_word("and").unwrap();

        let cache_stats = engine.cache_stats();
        // Just verify we can get cache stats
        let _ = cache_stats;

        // Clear cache returns Result
        let () = engine.clear_cache();
    }

    #[test]
    fn test_performance_metrics() {
        let (_temp_dir, config) = create_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        engine.analyze_word("the").unwrap();

        let metrics = engine.performance_metrics();
        assert!(metrics.total_queries > 0);
    }

    #[test]
    fn test_lexicon_input_hash() {
        use std::collections::hash_map::DefaultHasher;

        let input1 = LexiconInput {
            word: "test".to_string(),
        };
        let input2 = LexiconInput {
            word: "test".to_string(),
        };
        let input3 = LexiconInput {
            word: "other".to_string(),
        };

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        input1.hash(&mut hasher1);
        input2.hash(&mut hasher2);
        input3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    #[test]
    fn test_load_data_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = LexiconConfig {
            data_path: temp_dir.path().to_string_lossy().to_string(),
            ..LexiconConfig::default()
        };

        let mut engine = LexiconEngine::with_config(config);
        let result = engine.load_data();
        assert!(result.is_err());
    }

    // Full test lexicon XML content for word class testing
    const FULL_TEST_LEXICON_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<lexicon version="1.0" language="en" xmlns="http://canopy.rs/lexicon">
  <metadata>
    <title>Full Test Lexicon</title>
    <description>Complete test lexicon for word class methods</description>
    <created>2024-01-01</created>
    <author>Test</author>
    <license>MIT</license>
  </metadata>

  <word-classes>
    <word-class id="test-pronouns" name="Pronouns" type="pronouns" priority="10">
      <description>Personal pronouns</description>
      <words>
        <word pos="PRP">he</word>
        <word pos="PRP">she</word>
        <word pos="PRP">it</word>
        <word pos="PRP">they</word>
        <word pos="PRP">we</word>
        <word pos="PRP">I</word>
        <word pos="PRP">you</word>
      </words>
    </word-class>

    <word-class id="test-auxiliary" name="Auxiliary Verbs" type="auxiliary" priority="9">
      <description>Auxiliary verbs</description>
      <words>
        <word pos="VBZ">is</word>
        <word pos="VBZ">are</word>
        <word pos="VBD">was</word>
        <word pos="VBD">were</word>
        <word pos="VBZ">has</word>
        <word pos="VBP">have</word>
        <word pos="VBD">had</word>
        <word pos="VBP">do</word>
        <word pos="VBZ">does</word>
        <word pos="VBD">did</word>
      </words>
    </word-class>

    <word-class id="test-modal" name="Modal Verbs" type="modal" priority="9">
      <description>Modal verbs</description>
      <words>
        <word pos="MD">can</word>
        <word pos="MD">could</word>
        <word pos="MD">will</word>
        <word pos="MD">would</word>
        <word pos="MD">shall</word>
        <word pos="MD">should</word>
        <word pos="MD">may</word>
        <word pos="MD">might</word>
        <word pos="MD">must</word>
      </words>
    </word-class>

    <word-class id="test-prepositions" name="Prepositions" type="prepositions" priority="8">
      <description>Prepositions</description>
      <words>
        <word pos="IN">in</word>
        <word pos="IN">on</word>
        <word pos="IN">at</word>
        <word pos="IN">to</word>
        <word pos="IN">from</word>
        <word pos="IN">with</word>
      </words>
    </word-class>

    <word-class id="test-conjunctions" name="Conjunctions" type="conjunctions" priority="8">
      <description>Conjunctions</description>
      <words>
        <word pos="CC">and</word>
        <word pos="CC">but</word>
        <word pos="CC">or</word>
        <word pos="IN">because</word>
        <word pos="IN">although</word>
      </words>
    </word-class>

    <word-class id="test-wh-words" name="Wh-Words" type="wh-words" priority="8">
      <description>Wh-words for questions</description>
      <words>
        <word pos="WP">who</word>
        <word pos="WP">what</word>
        <word pos="WRB">where</word>
        <word pos="WRB">when</word>
        <word pos="WRB">why</word>
        <word pos="WRB">how</word>
        <word pos="WDT">which</word>
      </words>
    </word-class>

    <word-class id="test-quantifiers" name="Quantifiers" type="quantifiers" priority="7">
      <description>Quantifiers</description>
      <words>
        <word pos="DT">all</word>
        <word pos="DT">some</word>
        <word pos="DT">any</word>
        <word pos="DT">every</word>
        <word pos="DT">each</word>
        <word pos="RB">many</word>
        <word pos="RB">few</word>
      </words>
    </word-class>
  </word-classes>
</lexicon>"#;

    // Helper to create a more complete test lexicon with all word class types
    fn create_full_test_lexicon() -> (TempDir, LexiconConfig) {
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("english-lexicon.xml"),
            FULL_TEST_LEXICON_XML,
        )
        .unwrap();

        let config = LexiconConfig {
            data_path: temp_dir.path().to_string_lossy().to_string(),
            ..LexiconConfig::default()
        };

        (temp_dir, config)
    }

    #[test]
    fn test_is_pronoun() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        assert!(engine.is_pronoun("he").unwrap());
        assert!(engine.is_pronoun("she").unwrap());
        assert!(engine.is_pronoun("they").unwrap());
        assert!(!engine.is_pronoun("run").unwrap());
        assert!(!engine.is_pronoun("happy").unwrap());
    }

    #[test]
    fn test_is_auxiliary() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        assert!(engine.is_auxiliary("is").unwrap());
        assert!(engine.is_auxiliary("are").unwrap());
        assert!(engine.is_auxiliary("has").unwrap());
        assert!(engine.is_auxiliary("have").unwrap());
        assert!(engine.is_auxiliary("do").unwrap());
        assert!(!engine.is_auxiliary("run").unwrap());
    }

    #[test]
    fn test_is_modal() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        assert!(engine.is_modal("can").unwrap());
        assert!(engine.is_modal("could").unwrap());
        assert!(engine.is_modal("will").unwrap());
        assert!(engine.is_modal("must").unwrap());
        assert!(!engine.is_modal("is").unwrap());
        assert!(!engine.is_modal("run").unwrap());
    }

    #[test]
    fn test_is_preposition() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        assert!(engine.is_preposition("in").unwrap());
        assert!(engine.is_preposition("on").unwrap());
        assert!(engine.is_preposition("at").unwrap());
        assert!(engine.is_preposition("to").unwrap());
        assert!(!engine.is_preposition("run").unwrap());
    }

    #[test]
    fn test_is_conjunction() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        assert!(engine.is_conjunction("and").unwrap());
        assert!(engine.is_conjunction("but").unwrap());
        assert!(engine.is_conjunction("or").unwrap());
        assert!(engine.is_conjunction("because").unwrap());
        assert!(!engine.is_conjunction("run").unwrap());
    }

    #[test]
    fn test_is_wh_word() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        assert!(engine.is_wh_word("who").unwrap());
        assert!(engine.is_wh_word("what").unwrap());
        assert!(engine.is_wh_word("where").unwrap());
        assert!(engine.is_wh_word("when").unwrap());
        assert!(engine.is_wh_word("why").unwrap());
        assert!(engine.is_wh_word("how").unwrap());
        assert!(!engine.is_wh_word("run").unwrap());
    }

    #[test]
    fn test_is_quantifier() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        assert!(engine.is_quantifier("all").unwrap());
        assert!(engine.is_quantifier("some").unwrap());
        assert!(engine.is_quantifier("every").unwrap());
        assert!(engine.is_quantifier("many").unwrap());
        assert!(!engine.is_quantifier("run").unwrap());
    }

    #[test]
    fn test_is_particle() {
        let mut engine = LexiconEngine::new();
        if engine.load_data().is_err() {
            eprintln!("Skipping: Lexicon data not available");
            return;
        }

        // Common verb particles
        assert!(engine.is_particle("up").unwrap());
        assert!(engine.is_particle("down").unwrap());
        assert!(engine.is_particle("out").unwrap());
        assert!(engine.is_particle("off").unwrap());
        assert!(engine.is_particle("away").unwrap());
        // Not particles
        assert!(!engine.is_particle("run").unwrap());
        assert!(!engine.is_particle("the").unwrap());
    }

    #[test]
    fn test_get_pronoun_features() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        // Test "he" - third person singular masculine nominative
        let features = engine.get_pronoun_features("he").unwrap().unwrap();
        assert_eq!(features.person, Some(super::super::types::Person::Third));
        assert_eq!(
            features.number,
            Some(super::super::types::PronounNumber::Singular)
        );
        assert_eq!(
            features.gender,
            Some(super::super::types::PronounGender::Masculine)
        );
        assert_eq!(
            features.case,
            Some(super::super::types::PronounCase::Nominative)
        );

        // Test "she" - third person singular feminine nominative
        let features = engine.get_pronoun_features("she").unwrap().unwrap();
        assert_eq!(features.person, Some(super::super::types::Person::Third));
        assert_eq!(
            features.gender,
            Some(super::super::types::PronounGender::Feminine)
        );

        // Test "they" - third person plural
        let features = engine.get_pronoun_features("they").unwrap().unwrap();
        assert_eq!(features.person, Some(super::super::types::Person::Third));
        assert_eq!(
            features.number,
            Some(super::super::types::PronounNumber::Plural)
        );

        // Test non-pronoun returns None
        let features = engine.get_pronoun_features("run").unwrap();
        assert!(features.is_none());
    }

    #[test]
    fn test_get_pronoun_features_first_person() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        // Test "I" - first person singular nominative
        let features = engine.get_pronoun_features("I").unwrap().unwrap();
        assert_eq!(features.person, Some(super::super::types::Person::First));
        assert_eq!(
            features.number,
            Some(super::super::types::PronounNumber::Singular)
        );
        assert_eq!(
            features.case,
            Some(super::super::types::PronounCase::Nominative)
        );

        // Test "we" - first person plural nominative
        let features = engine.get_pronoun_features("we").unwrap().unwrap();
        assert_eq!(features.person, Some(super::super::types::Person::First));
        assert_eq!(
            features.number,
            Some(super::super::types::PronounNumber::Plural)
        );
    }

    #[test]
    fn test_get_pronoun_features_second_person() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        // Test "you" - second person (ambiguous number in English)
        let features = engine.get_pronoun_features("you").unwrap().unwrap();
        assert_eq!(features.person, Some(super::super::types::Person::Second));
        assert_eq!(features.number, None); // Ambiguous in English
    }

    #[test]
    fn test_is_word_class_helper() {
        let (_temp_dir, config) = create_full_test_lexicon();
        let mut engine = LexiconEngine::with_config(config);
        engine.load_data().unwrap();

        // Test the helper method works correctly
        assert!(engine.is_word_class("he", WordClassType::Pronouns).unwrap());
        assert!(!engine.is_word_class("he", WordClassType::Modal).unwrap());
        assert!(engine.is_word_class("can", WordClassType::Modal).unwrap());
        assert!(!engine
            .is_word_class("can", WordClassType::Pronouns)
            .unwrap());
    }
}
