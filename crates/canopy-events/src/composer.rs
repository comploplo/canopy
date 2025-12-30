//! Event composition engine
//!
//! Orchestrates decomposition and binding to compose events from Layer 1 analysis.
//!
//! ## Pipeline Steps
//!
//! 1. Identify predicates (verbs)
//! 2. Decompose predicates to LittleV structures
//! 3. Bind syntactic dependents to theta roles
//! 4. Resolve modality (force + flavor)
//! 5. Apply negation scope
//! 6. Detect presuppositions
//! 7. Infer plurality/distributivity

use crate::binding::ParticipantBinder;
use crate::confidence::{update_confidence, ConfidenceCalculator};
use crate::config::EventComposerConfig;
use crate::decomposition::EventDecomposer;
use crate::error::EventResult;
use crate::modality::ModalityResolver;
use crate::negation::NegationHandler;
use crate::plurality::PluralityInferrer;
use crate::presupposition::PresuppositionDetector;
use crate::types::{
    ComposedEvents, PredicateInfo, SentenceAnalysis, UnbindingReason, UnboundEntity,
};
use canopy_core::UPos;
use std::time::Instant;

/// Main event composition engine
pub struct EventComposer {
    decomposer: EventDecomposer,
    binder: ParticipantBinder,
    modality_resolver: ModalityResolver,
    negation_handler: NegationHandler,
    presupposition_detector: PresuppositionDetector,
    plurality_inferrer: PluralityInferrer,
    confidence_calc: ConfidenceCalculator,
    config: EventComposerConfig,
}

impl EventComposer {
    /// Create a new composer with default configuration
    pub fn new() -> EventResult<Self> {
        Self::with_config(EventComposerConfig::default())
    }

    /// Create a composer with custom configuration
    pub fn with_config(config: EventComposerConfig) -> EventResult<Self> {
        Ok(Self {
            decomposer: EventDecomposer::new(&config)?,
            binder: ParticipantBinder::new(&config)?,
            modality_resolver: ModalityResolver::new(&config)?,
            negation_handler: NegationHandler::new(&config)?,
            presupposition_detector: PresuppositionDetector::new(&config)?,
            plurality_inferrer: PluralityInferrer::new(&config)?,
            confidence_calc: ConfidenceCalculator::new(),
            config,
        })
    }

    /// Compose events from a sentence analysis
    pub fn compose_sentence(&self, analysis: &SentenceAnalysis) -> EventResult<ComposedEvents> {
        let start = Instant::now();

        // Step 1: Identify predicates
        let predicates = self.identify_predicates(analysis)?;

        if predicates.is_empty() {
            return Ok(ComposedEvents {
                events: Vec::new(),
                unbound_entities: self.find_all_unbound(analysis),
                confidence: 0.0,
                processing_time_us: start.elapsed().as_micros() as u64,
                sources: Vec::new(),
            });
        }

        let mut events = Vec::new();
        let mut all_unbound = Vec::new();
        let mut all_sources = Vec::new();

        // Step 2: Process each predicate
        for (idx, predicate) in predicates.iter().enumerate() {
            // Decompose predicate into LittleV
            let decomposed = match self.decomposer.decompose(predicate) {
                Ok(d) => d,
                Err(e) => {
                    tracing::warn!("Decomposition failed for '{}': {}", predicate.lemma, e);
                    continue;
                }
            };

            // Collect sources
            all_sources.extend(decomposed.sources.clone());

            // Bind participants
            let (mut composed, unbound) =
                self.binder
                    .bind_participants(decomposed.clone(), analysis, predicate)?;

            // Set event ID
            composed.id = idx;

            // Step 4: Resolve modality (force + flavor)
            if let Some(modality) = self.modality_resolver.resolve(predicate, analysis) {
                composed.event.modality = Some(modality);
            }

            // Step 5: Apply negation scope
            let (polarity, _neg_result) = self
                .negation_handler
                .apply_scope(predicate, &analysis.metadata);
            composed.polarity = polarity;

            // Step 6: Detect presuppositions
            composed.presuppositions = self
                .presupposition_detector
                .detect(predicate, &composed.event.participants);

            // Step 7: Infer plurality/distributivity for participants
            self.infer_participant_plurality(&mut composed, analysis, predicate);

            // Calculate confidence
            let conf = self.confidence_calc.calculate_event_confidence(
                &analysis.tokens,
                &decomposed,
                &composed,
            );
            composed.decomposition_confidence = decomposed.confidence;

            // Filter by confidence threshold
            if conf >= self.config.confidence_threshold {
                events.push(composed);
            }

            all_unbound.extend(unbound);

            // Respect max events limit
            if events.len() >= self.config.max_events_per_sentence {
                break;
            }
        }

        // Deduplicate sources
        all_sources.sort();
        all_sources.dedup();

        let mut result = ComposedEvents {
            events,
            unbound_entities: all_unbound,
            confidence: 0.0,
            processing_time_us: start.elapsed().as_micros() as u64,
            sources: all_sources,
        };

        // Update overall confidence
        update_confidence(&mut result, &self.confidence_calc);

        Ok(result)
    }

    /// Compose events for multiple sentences
    pub fn compose_batch(&self, analyses: &[SentenceAnalysis]) -> EventResult<Vec<ComposedEvents>> {
        analyses.iter().map(|a| self.compose_sentence(a)).collect()
    }

    /// Identify predicates (verbs) in the sentence
    fn identify_predicates(&self, analysis: &SentenceAnalysis) -> EventResult<Vec<PredicateInfo>> {
        let predicate_indices = analysis.find_predicates();

        let predicates: Vec<PredicateInfo> = predicate_indices
            .into_iter()
            .filter_map(|idx| {
                let token = analysis.get_token(idx)?;

                // Skip auxiliary verbs unless they're the only verb
                if matches!(token.pos, Some(UPos::Aux)) {
                    // Check if there's a main verb
                    let has_main_verb = analysis.tokens.iter().any(|t| {
                        matches!(t.pos, Some(UPos::Verb)) && t.original_word != token.original_word
                    });
                    if has_main_verb {
                        return None;
                    }
                }

                Some(PredicateInfo {
                    lemma: token.lemma.clone(),
                    token_idx: idx,
                    verbnet_analysis: token.verbnet.clone(),
                    framenet_analysis: token.framenet.clone(),
                    l1_confidence: token.confidence,
                })
            })
            .collect();

        Ok(predicates)
    }

    /// Infer plurality and distributivity for participants in an event
    fn infer_participant_plurality(
        &self,
        composed: &mut crate::types::ComposedEvent,
        analysis: &SentenceAnalysis,
        predicate: &PredicateInfo,
    ) {
        // Check for "each" adverb in the sentence
        let has_each_adverb = analysis.tokens.iter().any(|t| {
            t.lemma.to_lowercase() == "each" && matches!(t.pos, Some(UPos::Adv) | Some(UPos::Det))
        });

        // Iterate over participants and infer number/distributivity
        for entity in composed.event.participants.values_mut() {
            // Try to find the corresponding token by matching entity text
            if let Some(token) = analysis
                .tokens
                .iter()
                .find(|t| t.original_word == entity.text || t.lemma == entity.text)
            {
                // Infer semantic number
                if entity.number.is_none() {
                    entity.number = self.plurality_inferrer.infer_number(token);
                }

                // Infer distributivity for plural entities
                if entity.distributivity.is_none() {
                    if let Some(number) = entity.number {
                        entity.distributivity = self.plurality_inferrer.infer_distributivity(
                            number,
                            predicate,
                            has_each_adverb,
                        );
                    }
                }
            }
        }
    }

    /// Find all tokens that weren't assigned to any event
    fn find_all_unbound(&self, analysis: &SentenceAnalysis) -> Vec<UnboundEntity> {
        analysis
            .tokens
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                // Filter to content words that should have been assigned
                matches!(
                    t.pos,
                    Some(UPos::Noun) | Some(UPos::Propn) | Some(UPos::Pron)
                )
            })
            .map(|(idx, t)| UnboundEntity {
                token_idx: idx,
                text: t.original_word.clone(),
                suggested_role: None,
                reason: UnbindingReason::NoPredicateFound,
            })
            .collect()
    }

    /// Get the current configuration
    pub fn config(&self) -> &EventComposerConfig {
        &self.config
    }
}

impl Default for EventComposer {
    fn default() -> Self {
        Self::new().expect("Failed to create default EventComposer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DependencyArc;
    use canopy_treebank::types::DependencyRelation;

    fn make_token(
        word: &str,
        lemma: &str,
        pos: Option<UPos>,
    ) -> canopy_tokenizer::coordinator::Layer1SemanticResult {
        canopy_tokenizer::coordinator::Layer1SemanticResult {
            original_word: word.to_string(),
            lemma: lemma.to_string(),
            pos,
            lemmatization_confidence: None,
            verbnet: None,
            framenet: None,
            wordnet: None,
            lexicon: None,
            propbank: None,
            treebank: None,
            confidence: 0.8,
            sources: vec![],
            errors: vec![],
        }
    }

    #[test]
    fn test_composer_creation() {
        let composer = EventComposer::new();
        assert!(composer.is_ok());
    }

    #[test]
    fn test_compose_empty_sentence() {
        let composer = EventComposer::new().unwrap();
        let analysis = SentenceAnalysis::new("".to_string(), vec![]);
        let result = composer.compose_sentence(&analysis).unwrap();
        assert!(result.events.is_empty());
    }

    #[test]
    fn test_compose_simple_sentence() {
        let composer = EventComposer::new().unwrap();

        let tokens = vec![
            make_token("John", "john", Some(UPos::Propn)),
            make_token("runs", "run", Some(UPos::Verb)),
        ];

        let deps = vec![DependencyArc::new(1, 0, DependencyRelation::NominalSubject)];

        let analysis =
            SentenceAnalysis::new("John runs".to_string(), tokens).with_dependencies(deps);

        let result = composer.compose_sentence(&analysis).unwrap();

        // Should have one event (run)
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].event.predicate, "run");
    }

    #[test]
    fn test_identify_predicates() {
        let composer = EventComposer::new().unwrap();

        let tokens = vec![
            make_token("The", "the", Some(UPos::Det)),
            make_token("cat", "cat", Some(UPos::Noun)),
            make_token("sat", "sit", Some(UPos::Verb)),
        ];

        let analysis = SentenceAnalysis::new("The cat sat".to_string(), tokens);
        let predicates = composer.identify_predicates(&analysis).unwrap();

        assert_eq!(predicates.len(), 1);
        assert_eq!(predicates[0].lemma, "sit");
        assert_eq!(predicates[0].token_idx, 2);
    }
}
