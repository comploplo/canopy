//! Discourse context management
//!
//! Tracks discourse state across sentences, managing:
//! - Active DRS
//! - Discourse referent registry
//! - Sentence history
//! - Temporal anchoring
//!
//! ## Anaphora Resolution
//!
//! Based on modern binding theory:
//! - Reuland (2011) "Anaphora and Language Design"
//! - Reinhart & Reuland (1993) "Reflexivity"
//! - Charnavel (2019) "Locality and Logophoricity"

use crate::drs::{Drs, DrsCondition, DrsId};
use crate::error::{DiscourseError, DiscourseResult};
use crate::gender::GenderLookup;
use crate::logophoricity::{LogophoricContext, LogophoricDetector};
use crate::referent::{
    classify_anaphor, is_pronoun, AnaphorClassification, AnaphorType, DiscourseReferent, Gender,
    ReferentId, ReferentRegistry, ReferentType,
};
use crate::reflexivity::PredicateAnalyzer;
use canopy_events::ComposedEvent;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Infer gender from a word (pronoun or name)
///
/// For pronouns, uses the anaphor classification.
/// For names, uses the name-gender dataset.
fn infer_gender(word: &str) -> Option<Gender> {
    // First check if it's a pronoun
    let classification = classify_anaphor(word);
    if classification.anaphor_type != AnaphorType::None {
        return classification.gender;
    }

    // Otherwise try name-based gender lookup
    GenderLookup::global().infer(word)
}

/// Configuration for discourse context
#[derive(Debug, Clone)]
pub struct DiscourseConfig {
    /// Maximum number of sentences to keep in context
    pub max_sentences: usize,

    /// Maximum number of referents to track
    pub max_referents: usize,

    /// Whether to perform automatic anaphora resolution
    pub auto_resolve_anaphora: bool,

    /// Minimum salience score for antecedent candidates
    pub min_antecedent_salience: f32,
}

impl Default for DiscourseConfig {
    fn default() -> Self {
        Self {
            max_sentences: 100,
            max_referents: 500,
            auto_resolve_anaphora: true,
            min_antecedent_salience: 0.1,
        }
    }
}

/// Tracks discourse state across multiple sentences
#[derive(Debug)]
pub struct DiscourseContext {
    /// Configuration
    config: DiscourseConfig,

    /// The main DRS being constructed
    main_drs: Drs,

    /// Referent registry
    referents: ReferentRegistry,

    /// Sentence history (sentence index -> events)
    sentence_history: IndexMap<usize, SentenceInfo>,

    /// Current sentence index
    current_sentence: usize,

    /// Next DRS ID
    next_drs_id: usize,

    /// Resolved anaphora mappings (pronoun referent -> antecedent referent)
    anaphora_resolutions: IndexMap<ReferentId, ReferentId>,

    /// Temporal anchor (current reference time for tense interpretation)
    temporal_anchor: Option<ReferentId>,

    /// Predicate analyzer for Condition B (Reinhart & Reuland 1993)
    predicate_analyzer: PredicateAnalyzer,

    /// Logophoric context detector (Charnavel 2019)
    logophoric_detector: LogophoricDetector,
}

/// Information about a processed sentence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceInfo {
    /// Sentence index
    pub index: usize,

    /// Original text
    pub text: String,

    /// Referents introduced in this sentence
    pub introduced_referents: Vec<ReferentId>,

    /// Events from this sentence
    pub events: Vec<ReferentId>,
}

impl DiscourseContext {
    /// Create a new discourse context
    #[must_use]
    pub fn new(config: DiscourseConfig) -> Self {
        Self {
            config,
            main_drs: Drs::new(DrsId(0)),
            referents: ReferentRegistry::new(),
            sentence_history: IndexMap::new(),
            current_sentence: 0,
            next_drs_id: 1,
            anaphora_resolutions: IndexMap::new(),
            temporal_anchor: None,
            predicate_analyzer: PredicateAnalyzer::new(),
            logophoric_detector: LogophoricDetector::new(),
        }
    }

    /// Create with default configuration
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(DiscourseConfig::default())
    }

    /// Process a composed event and integrate it into discourse
    pub fn process_event(&mut self, composed: &ComposedEvent) -> DiscourseResult<ReferentId> {
        // Check capacity
        if self.referents.len() >= self.config.max_referents {
            return Err(DiscourseError::ContextCapacityExceeded {
                max: self.config.max_referents,
                current: self.referents.len(),
            });
        }

        // Access the inner Event structure
        let event = &composed.event;

        // Create event referent
        let event_ref_id = self.referents.allocate_id();
        let mut event_ref =
            DiscourseReferent::event(event_ref_id, event.predicate.clone(), self.current_sentence);

        // Add event type property from little_v
        event_ref.add_property(
            "little_v",
            crate::referent::PropertyValue::String(format!("{:?}", event.little_v)),
        );

        self.referents.register(event_ref);

        // Add event predicate to DRS
        let mut participants_map = IndexMap::new();

        // Process participants from the Event's participant HashMap
        for (role, entity) in &event.participants {
            let part_id = self.referents.allocate_id();
            let mut part_ref =
                DiscourseReferent::entity(part_id, entity.text.clone(), self.current_sentence);

            // Set animacy if available
            if let Some(animacy) = entity.animacy {
                part_ref.set_animacy(animacy);
            }

            // Set gender from pronoun classification or name lookup
            if let Some(gender) = infer_gender(&entity.text) {
                part_ref.set_gender(gender);
            }

            // Mark if this is a pronoun (shouldn't be used as antecedent for other pronouns)
            if is_pronoun(&entity.text) {
                part_ref.add_property("is_pronoun", crate::referent::PropertyValue::Bool(true));
            }

            self.referents.register(part_ref.clone());

            // Clone for DRS
            self.main_drs.add_referent(part_ref.clone());

            // Add predicate condition for the entity
            self.main_drs.add_condition(DrsCondition::Predicate {
                name: entity.text.clone(),
                referent: part_id,
            });

            // Add theta role condition
            self.main_drs.add_condition(DrsCondition::ThetaRole {
                event_id: event_ref_id,
                role: *role,
                filler: part_id,
            });

            participants_map.insert(format!("{:?}", role), part_id);
        }

        // Add event to DRS
        self.main_drs.add_referent(DiscourseReferent::event(
            event_ref_id,
            event.predicate.clone(),
            self.current_sentence,
        ));

        self.main_drs.add_condition(DrsCondition::EventPredicate {
            event_id: event_ref_id,
            predicate: event.predicate.clone(),
            participants: participants_map,
        });

        // Add temporal relation to previous event (if any)
        // Default: narrative order implies temporal sequence
        if let Some(prev_event_id) = self.temporal_anchor {
            // Determine temporal relation based on aspectual class
            let relation = self.infer_temporal_relation(&event.aspect, prev_event_id, event_ref_id);
            self.main_drs.add_condition(DrsCondition::TemporalRelation {
                relation,
                event1: prev_event_id,
                event2: event_ref_id,
            });
        }

        // Update temporal anchor
        self.temporal_anchor = Some(event_ref_id);

        Ok(event_ref_id)
    }

    /// Infer temporal relation between two events based on aspectual class
    fn infer_temporal_relation(
        &self,
        aspect: &canopy_core::AspectualClass,
        _prev_event: ReferentId,
        _new_event: ReferentId,
    ) -> crate::drs::TemporalRelationType {
        use crate::drs::TemporalRelationType;
        use canopy_core::AspectualClass;

        // Aspectual class influences temporal interpretation:
        // - States often overlap with other events
        // - Achievements are punctual and typically follow prior events
        // - Activities and Accomplishments typically sequence
        match aspect {
            AspectualClass::State => TemporalRelationType::Overlaps,
            AspectualClass::Achievement => TemporalRelationType::Meets,
            AspectualClass::Activity | AspectualClass::Accomplishment => {
                TemporalRelationType::Before
            }
        }
    }

    /// Start processing a new sentence
    pub fn begin_sentence(&mut self, text: String) {
        let info = SentenceInfo {
            index: self.current_sentence,
            text,
            introduced_referents: Vec::new(),
            events: Vec::new(),
        };
        self.sentence_history.insert(self.current_sentence, info);
    }

    /// Finish processing current sentence and advance
    pub fn end_sentence(&mut self) {
        self.current_sentence += 1;

        // Prune old sentences if exceeding limit
        while self.sentence_history.len() > self.config.max_sentences {
            if let Some((oldest_idx, _)) = self.sentence_history.first() {
                let oldest_idx = *oldest_idx;
                self.sentence_history.shift_remove(&oldest_idx);
            }
        }
    }

    /// Resolve a pronoun to its antecedent
    ///
    /// In DRT, pronouns are resolved directly to existing discourse referents.
    /// The pronoun doesn't introduce a new referent - it simply refers back to
    /// an already-established one.
    ///
    /// Returns the ReferentId of the resolved antecedent.
    pub fn resolve_pronoun(&mut self, pronoun: &str) -> DiscourseResult<ReferentId> {
        let candidates = self
            .referents
            .find_antecedent_candidates(pronoun, self.current_sentence);

        // Filter by minimum salience
        let valid_candidates: Vec<_> = candidates
            .into_iter()
            .filter(|(_, score)| *score >= self.config.min_antecedent_salience)
            .collect();

        if valid_candidates.is_empty() {
            return Err(DiscourseError::AnaphoraResolutionFailed {
                pronoun: pronoun.to_string(),
                reason: "no suitable antecedent found".to_string(),
            });
        }

        // Take the most salient candidate
        let (antecedent_id, _salience) = valid_candidates[0];

        // Track the resolution for analysis purposes
        // Key: sentence index + pronoun form → antecedent ID
        self.anaphora_resolutions.insert(
            ReferentId(self.current_sentence * 1000 + self.anaphora_resolutions.len()),
            antecedent_id,
        );

        // Return the antecedent directly - no new referent created
        // The pronoun simply co-refers with the existing referent
        Ok(antecedent_id)
    }

    /// Resolve an anaphor with modern binding theory
    ///
    /// Based on Reinhart & Reuland (1993) and Charnavel (2019).
    ///
    /// This method handles:
    /// - SELF-anaphors (himself, herself, etc.) - require local co-argument or logophoric context
    /// - Personal pronouns (he, she, it) - cannot co-refer with co-arguments (Condition B)
    /// - Logophoric contexts - allow exempt readings for non-local binding
    ///
    /// # Arguments
    /// * `anaphor` - The anaphoric expression to resolve
    /// * `predicate` - The predicate the anaphor appears in (for co-argument detection)
    /// * `coargument_ids` - IDs of co-arguments in the same predicate
    /// * `logophoric_context` - Optional logophoric context for exempt readings
    pub fn resolve_anaphor(
        &mut self,
        anaphor: &str,
        predicate: &str,
        coargument_ids: &[ReferentId],
        logophoric_context: Option<&LogophoricContext>,
    ) -> DiscourseResult<ReferentId> {
        let classification = classify_anaphor(anaphor);

        match classification.anaphor_type {
            AnaphorType::SelfAnaphor => self.resolve_self_anaphor(
                anaphor,
                &classification,
                coargument_ids,
                logophoric_context,
            ),
            AnaphorType::Personal => {
                self.resolve_personal_pronoun(anaphor, predicate, &classification, coargument_ids)
            }
            AnaphorType::Possessive => {
                // Possessives have similar constraints to personal pronouns
                self.resolve_personal_pronoun(anaphor, predicate, &classification, coargument_ids)
            }
            AnaphorType::None => Err(DiscourseError::AnaphoraResolutionFailed {
                pronoun: anaphor.to_string(),
                reason: "not an anaphoric expression".to_string(),
            }),
        }
    }

    /// Resolve a SELF-anaphor (reflexive)
    ///
    /// Per Reinhart & Reuland (1993):
    /// - Plain reading: must be bound by a co-argument of the same predicate
    /// - Exempt reading: can be bound by logophoric center (Charnavel 2019)
    fn resolve_self_anaphor(
        &mut self,
        anaphor: &str,
        _classification: &AnaphorClassification,
        coargument_ids: &[ReferentId],
        logophoric_context: Option<&LogophoricContext>,
    ) -> DiscourseResult<ReferentId> {
        // Get all candidates with feature agreement
        let candidates = self
            .referents
            .find_antecedent_candidates(anaphor, self.current_sentence);

        // Filter by minimum salience
        let candidates: Vec<_> = candidates
            .into_iter()
            .filter(|(_, score)| *score >= self.config.min_antecedent_salience)
            .collect();

        // Try plain reading first: co-argument binding
        let local_candidates: Vec<_> = candidates
            .iter()
            .filter(|(id, _)| coargument_ids.contains(id))
            .cloned()
            .collect();

        if !local_candidates.is_empty() {
            let (antecedent_id, _) = local_candidates[0];
            self.track_resolution(antecedent_id);
            return Ok(antecedent_id);
        }

        // Try exempt reading: logophoric binding
        if let Some(ctx) = logophoric_context {
            if ctx.is_logophoric() {
                if let Some(perspective_center) = ctx.perspective_center() {
                    // Check if perspective center is a valid candidate
                    if candidates.iter().any(|(id, _)| *id == perspective_center) {
                        self.track_resolution(perspective_center);
                        return Ok(perspective_center);
                    }
                }
            }
        }

        // No valid antecedent found
        Err(DiscourseError::AnaphoraResolutionFailed {
            pronoun: anaphor.to_string(),
            reason: "SELF-anaphor requires local co-argument or logophoric center".to_string(),
        })
    }

    /// Resolve a personal pronoun
    ///
    /// Per Reinhart & Reuland (1993) Condition B:
    /// A reflexive semantic predicate must be reflexive-marked.
    /// Therefore, a personal pronoun CANNOT co-refer with a co-argument
    /// (that would make the predicate reflexive without reflexive-marking).
    fn resolve_personal_pronoun(
        &mut self,
        pronoun: &str,
        predicate: &str,
        _classification: &AnaphorClassification,
        coargument_ids: &[ReferentId],
    ) -> DiscourseResult<ReferentId> {
        // Get all candidates with feature agreement
        let candidates = self
            .referents
            .find_antecedent_candidates(pronoun, self.current_sentence);

        // Filter by minimum salience
        let mut valid_candidates: Vec<_> = candidates
            .into_iter()
            .filter(|(_, score)| *score >= self.config.min_antecedent_salience)
            .collect();

        // Apply Condition B: exclude co-arguments
        // (unless predicate is intrinsically reflexive)
        if !self
            .predicate_analyzer
            .is_intrinsically_reflexive(predicate)
        {
            valid_candidates.retain(|(id, _)| !coargument_ids.contains(id));
        }

        if valid_candidates.is_empty() {
            return Err(DiscourseError::AnaphoraResolutionFailed {
                pronoun: pronoun.to_string(),
                reason: "no suitable antecedent (Condition B blocks co-arguments)".to_string(),
            });
        }

        // Take the most salient candidate
        let (antecedent_id, _) = valid_candidates[0];
        self.track_resolution(antecedent_id);
        Ok(antecedent_id)
    }

    /// Track a resolution for analysis
    fn track_resolution(&mut self, antecedent_id: ReferentId) {
        self.anaphora_resolutions.insert(
            ReferentId(self.current_sentence * 1000 + self.anaphora_resolutions.len()),
            antecedent_id,
        );
    }

    /// Get the predicate analyzer
    #[must_use]
    pub fn predicate_analyzer(&self) -> &PredicateAnalyzer {
        &self.predicate_analyzer
    }

    /// Get the logophoric detector
    #[must_use]
    pub fn logophoric_detector(&self) -> &LogophoricDetector {
        &self.logophoric_detector
    }

    /// Get the current main DRS
    #[must_use]
    pub fn drs(&self) -> &Drs {
        &self.main_drs
    }

    /// Get referent registry
    #[must_use]
    pub fn referents(&self) -> &ReferentRegistry {
        &self.referents
    }

    /// Get current sentence index
    #[must_use]
    pub fn current_sentence_index(&self) -> usize {
        self.current_sentence
    }

    /// Get sentence history
    #[must_use]
    pub fn sentence_history(&self) -> &IndexMap<usize, SentenceInfo> {
        &self.sentence_history
    }

    /// Get anaphora resolutions
    #[must_use]
    pub fn anaphora_resolutions(&self) -> &IndexMap<ReferentId, ReferentId> {
        &self.anaphora_resolutions
    }

    /// Allocate a new DRS ID
    pub fn allocate_drs_id(&mut self) -> DrsId {
        let id = DrsId(self.next_drs_id);
        self.next_drs_id += 1;
        id
    }

    /// Create a new referent and add to context
    pub fn introduce_referent(
        &mut self,
        name: String,
        referent_type: ReferentType,
    ) -> DiscourseResult<ReferentId> {
        if self.referents.len() >= self.config.max_referents {
            return Err(DiscourseError::ContextCapacityExceeded {
                max: self.config.max_referents,
                current: self.referents.len(),
            });
        }

        let id = self.referents.allocate_id();
        let referent = DiscourseReferent {
            id,
            name: Some(name.clone()),
            referent_type,
            is_event: referent_type == ReferentType::Event,
            introduced_at: self.current_sentence,
            properties: IndexMap::new(),
        };

        self.referents.register(referent.clone());
        self.main_drs.add_referent(referent);

        // Track in current sentence
        if let Some(info) = self.sentence_history.get_mut(&self.current_sentence) {
            info.introduced_referents.push(id);
        }

        Ok(id)
    }

    /// Clear all discourse state
    pub fn clear(&mut self) {
        self.main_drs = Drs::new(DrsId(0));
        self.referents.clear();
        self.sentence_history.clear();
        self.current_sentence = 0;
        self.next_drs_id = 1;
        self.anaphora_resolutions.clear();
        self.temporal_anchor = None;
    }

    /// Get statistics about the context
    #[must_use]
    pub fn statistics(&self) -> ContextStatistics {
        ContextStatistics {
            sentence_count: self.current_sentence,
            referent_count: self.referents.len(),
            condition_count: self.main_drs.condition_count(),
            resolution_count: self.anaphora_resolutions.len(),
        }
    }
}

/// Statistics about discourse context
#[derive(Debug, Clone)]
pub struct ContextStatistics {
    pub sentence_count: usize,
    pub referent_count: usize,
    pub condition_count: usize,
    pub resolution_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = DiscourseContext::with_defaults();
        assert_eq!(ctx.current_sentence_index(), 0);
        assert!(ctx.referents().is_empty());
    }

    #[test]
    fn test_sentence_processing() {
        let mut ctx = DiscourseContext::with_defaults();

        ctx.begin_sentence("A man walks.".to_string());
        assert!(ctx.sentence_history().contains_key(&0));

        ctx.end_sentence();
        assert_eq!(ctx.current_sentence_index(), 1);
    }

    #[test]
    fn test_introduce_referent() {
        let mut ctx = DiscourseContext::with_defaults();
        ctx.begin_sentence("Test sentence.".to_string());

        let id = ctx
            .introduce_referent("man".to_string(), ReferentType::Individual)
            .unwrap();

        assert!(ctx.referents().get(id).is_some());
        assert_eq!(ctx.drs().referent_count(), 1);
    }

    #[test]
    fn test_capacity_limit() {
        let config = DiscourseConfig {
            max_referents: 2,
            ..Default::default()
        };
        let mut ctx = DiscourseContext::new(config);
        ctx.begin_sentence("Test.".to_string());

        ctx.introduce_referent("a".to_string(), ReferentType::Individual)
            .unwrap();
        ctx.introduce_referent("b".to_string(), ReferentType::Individual)
            .unwrap();

        let result = ctx.introduce_referent("c".to_string(), ReferentType::Individual);
        assert!(matches!(
            result,
            Err(DiscourseError::ContextCapacityExceeded { .. })
        ));
    }

    #[test]
    fn test_statistics() {
        let mut ctx = DiscourseContext::with_defaults();
        ctx.begin_sentence("Test.".to_string());
        ctx.introduce_referent("man".to_string(), ReferentType::Individual)
            .unwrap();
        ctx.end_sentence();

        let stats = ctx.statistics();
        assert_eq!(stats.sentence_count, 1);
        assert_eq!(stats.referent_count, 1);
    }
}
