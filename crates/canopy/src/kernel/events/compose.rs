//! Event composition: combines decomposition and participant binding.
//!
//! The `EventComposer` takes pre-decomposed predicate structures from providers
//! and assembles them into composed events with bound participants.
//!
//! # Kernel Purity
//!
//! This module contains NO word-level knowledge. All predicate decomposition
//! happens in the resources layer via `SenseProvider::decompose_predicate()`.
//!
//! The kernel only knows:
//! - UTAH-based dependency-to-role mappings (nsubj→Agent, obj→Patient)
//! - How to assemble events from pre-decomposed structures
//! - How to bind participants from `RoleProvider` bindings

use super::types::{
    ComposedEvent, ComposedEvents, PackedEvents, Participant, SenseAlternative, SenseChoicePoint,
    SentenceAnalysis, SharedEventStructure, UnbindingReason, UnboundParticipant,
};
use crate::core::{CanopyError, DepRel, ThetaRole, Voice};
use crate::kernel::underspec::ChoiceId;
use crate::runtime::{PredicateDecomposition, RoleBinding, TokenId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the event composer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventComposerConfig {
    /// Minimum confidence for decomposition.
    pub min_decomposition_confidence: f32,

    /// Minimum confidence for role binding.
    pub min_binding_confidence: f32,

    /// Whether to detect presuppositions.
    pub detect_presuppositions: bool,

    /// Whether to handle negation.
    pub handle_negation: bool,
}

impl Default for EventComposerConfig {
    fn default() -> Self {
        Self {
            min_decomposition_confidence: 0.3,
            min_binding_confidence: 0.3,
            detect_presuppositions: true,
            handle_negation: true,
        }
    }
}

/// Composes events from pre-decomposed predicate structures.
///
/// The `EventComposer` is pure in that it contains NO word-level knowledge.
/// All decomposition happens via providers; the composer just assembles.
pub struct EventComposer {
    config: EventComposerConfig,
}

impl Default for EventComposer {
    fn default() -> Self {
        Self::new(EventComposerConfig::default())
    }
}

impl EventComposer {
    /// Create a new event composer.
    #[must_use]
    pub fn new(config: EventComposerConfig) -> Self {
        Self { config }
    }

    /// Compose events from pre-decomposed predicate structures.
    ///
    /// # Arguments
    /// * `analysis` - The sentence analysis containing syntax and dependencies
    /// * `decompositions` - Pre-decomposed predicates from `SenseProvider` (indexed by predicate token ID)
    /// * `role_bindings` - Role bindings from `RoleProvider` (indexed by predicate token ID)
    ///
    /// # Returns
    /// Composed events with bound participants.
    ///
    /// # Errors
    /// Returns an error if event composition fails due to invalid predicate structure.
    pub fn compose(
        &self,
        analysis: &SentenceAnalysis,
        decompositions: &HashMap<TokenId, Vec<PredicateDecomposition>>,
        role_bindings: &HashMap<TokenId, Vec<RoleBinding>>,
    ) -> Result<ComposedEvents, CanopyError> {
        let predicates = analysis.find_predicates();

        if predicates.is_empty() {
            return Ok(ComposedEvents::empty());
        }

        let mut events = Vec::new();
        let mut unbound = Vec::new();
        let mut sources = Vec::new();

        for (idx, pred_id) in predicates.into_iter().enumerate() {
            // Get decomposition for this predicate
            let pred_decomps = decompositions.get(&pred_id).cloned().unwrap_or_default();
            let pred_bindings = role_bindings.get(&pred_id).cloned().unwrap_or_default();

            match self.compose_single_event(analysis, pred_id, idx, &pred_decomps, &pred_bindings) {
                Ok((event, mut event_unbound, mut event_sources)) => {
                    events.push(event);
                    unbound.append(&mut event_unbound);
                    sources.append(&mut event_sources);
                }
                Err(e) => {
                    // Log but continue with other predicates
                    eprintln!("Warning: Failed to compose event for predicate {pred_id}: {e}");
                }
            }
        }

        // Deduplicate sources
        sources.sort();
        sources.dedup();

        // Calculate overall confidence
        #[allow(clippy::cast_precision_loss)] // Event count is small enough
        let confidence = if events.is_empty() {
            0.0
        } else {
            events
                .iter()
                .map(super::types::ComposedEvent::overall_confidence)
                .sum::<f32>()
                / events.len() as f32
        };

        Ok(ComposedEvents {
            events,
            unbound_participants: unbound,
            confidence,
            sources,
        })
    }

    /// Compose events preserving all readings (sense ambiguity).
    ///
    /// Unlike `compose()` which selects the best reading, this method
    /// returns a packed representation with all alternatives preserved.
    ///
    /// # Arguments
    /// * `analysis` - The sentence analysis containing syntax and dependencies
    /// * `decompositions` - All pre-decomposed predicates from `SenseProvider` (indexed by predicate token ID)
    /// * `role_bindings` - Role bindings from `RoleProvider` (indexed by predicate token ID)
    ///
    /// # Returns
    /// Packed events with all sense alternatives preserved.
    ///
    /// # Errors
    /// Returns an error if composition fails for all predicates.
    pub fn compose_packed(
        &self,
        analysis: &SentenceAnalysis,
        decompositions: &HashMap<TokenId, Vec<PredicateDecomposition>>,
        role_bindings: &HashMap<TokenId, Vec<RoleBinding>>,
    ) -> Result<PackedEvents, CanopyError> {
        let predicates = analysis.find_predicates();

        let shared = SharedEventStructure::from_analysis(analysis);
        let mut packed = PackedEvents::new(shared);

        if predicates.is_empty() {
            return Ok(packed);
        }

        for (choice_idx, pred_id) in predicates.into_iter().enumerate() {
            let pred_decomps = decompositions.get(&pred_id).cloned().unwrap_or_default();
            let pred_bindings = role_bindings.get(&pred_id).cloned().unwrap_or_default();

            if let Some(choice) = self.build_sense_choice(
                analysis,
                pred_id,
                choice_idx,
                &pred_decomps,
                &pred_bindings,
            ) {
                // Collect sources
                for alt in &choice.alternatives {
                    packed
                        .sources
                        .push(format!("{:?}", alt.decomposition.source));
                }
                packed.add_sense_choice(choice);
            }
        }

        // Deduplicate sources
        packed.sources.sort();
        packed.sources.dedup();

        Ok(packed)
    }

    /// Build a sense choice point for a predicate.
    fn build_sense_choice(
        &self,
        analysis: &SentenceAnalysis,
        pred_id: TokenId,
        choice_idx: usize,
        decompositions: &[PredicateDecomposition],
        bindings: &[RoleBinding],
    ) -> Option<SenseChoicePoint> {
        let token = analysis.syntax.get_token(pred_id)?;
        let lemma = &token.lemma;

        // Filter decompositions by confidence threshold
        let valid_decomps: Vec<_> = decompositions
            .iter()
            .filter(|d| d.confidence >= self.config.min_decomposition_confidence)
            .collect();

        if valid_decomps.is_empty() {
            return None;
        }

        let choice_id = ChoiceId::new(choice_idx.try_into().unwrap_or(0));
        let mut choice = SenseChoicePoint::new(choice_id, pred_id, lemma.clone());

        // Compute voice once (shared across alternatives)
        let voice = self.detect_voice(analysis, pred_id);

        // Compute span once (shared)
        let span_start = pred_id;
        let span_end = analysis
            .get_dependents(pred_id)
            .iter()
            .map(|arc| arc.dependent_id)
            .max()
            .unwrap_or(pred_id);
        let span = (span_start, span_end);

        for decomp in valid_decomps {
            // Bind participants for this decomposition
            let Ok((participants, _unbound)) =
                self.bind_participants(analysis, pred_id, bindings, &decomp.expected_roles)
            else {
                continue;
            };

            // Calculate binding confidence
            #[allow(clippy::cast_precision_loss)]
            let binding_confidence = if participants.is_empty() {
                0.0
            } else {
                participants.values().map(|p| p.confidence).sum::<f32>() / participants.len() as f32
            };

            let alt = SenseAlternative::new(decomp.clone())
                .with_participants(participants)
                .with_voice(voice)
                .with_span(span)
                .with_binding_confidence(binding_confidence);

            choice.add_alternative(alt);
        }

        if choice.alternatives.is_empty() {
            None
        } else {
            Some(choice)
        }
    }

    /// Compose a single event for a predicate.
    fn compose_single_event(
        &self,
        analysis: &SentenceAnalysis,
        pred_id: TokenId,
        event_idx: usize,
        decompositions: &[PredicateDecomposition],
        bindings: &[RoleBinding],
    ) -> Result<(ComposedEvent, Vec<UnboundParticipant>, Vec<String>), CanopyError> {
        let token = analysis
            .syntax
            .get_token(pred_id)
            .ok_or_else(|| CanopyError::analysis(format!("{pred_id}"), "Token not found"))?;

        let lemma = &token.lemma;

        // Get best decomposition (highest confidence)
        let best_decomp = decompositions.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // If no decomposition provided, we can't compose an event
        let decomp = best_decomp.ok_or_else(|| {
            CanopyError::analysis(lemma.clone(), "No decomposition provided by SenseProvider")
        })?;

        if decomp.confidence < self.config.min_decomposition_confidence {
            return Err(CanopyError::analysis(
                lemma.clone(),
                format!(
                    "Decomposition confidence {} below threshold {}",
                    decomp.confidence, self.config.min_decomposition_confidence
                ),
            ));
        }

        // Bind participants - prefer provider bindings, fallback to UTAH heuristics
        let (participants, unbound) =
            self.bind_participants(analysis, pred_id, bindings, &decomp.expected_roles)?;

        // Calculate binding confidence
        #[allow(clippy::cast_precision_loss)] // Participant count is small
        let binding_confidence = if participants.is_empty() {
            0.0
        } else {
            participants.values().map(|p| p.confidence).sum::<f32>() / participants.len() as f32
        };

        // Detect voice
        let voice = self.detect_voice(analysis, pred_id);

        // Find token span
        let span_start = pred_id;
        let span_end = analysis
            .get_dependents(pred_id)
            .iter()
            .map(|arc| arc.dependent_id)
            .max()
            .unwrap_or(pred_id);

        // Collect sources
        let sources = vec![format!("{:?}", decomp.source)];

        let event = ComposedEvent {
            id: event_idx,
            predicate: lemma.clone(),
            little_v_type: decomp.little_v_type,
            participants,
            aspect: decomp.little_v_type.aspectual_class(),
            voice,
            token_span: (span_start, span_end),
            source_sense: Some(decomp.sense_id.clone()),
            decomposition_confidence: decomp.confidence,
            binding_confidence,
            presuppositions: vec![], // TODO: implement presupposition detection
            polarity: !analysis.metadata.is_negated,
        };

        Ok((event, unbound, sources))
    }

    /// Bind participants to theta roles.
    ///
    /// Priority:
    /// 1. Use `RoleProvider` bindings if available
    /// 2. Fall back to UTAH-based dependency heuristics
    fn bind_participants(
        &self,
        analysis: &SentenceAnalysis,
        pred_id: TokenId,
        bindings: &[RoleBinding],
        expected_roles: &[ThetaRole],
    ) -> Result<(HashMap<ThetaRole, Participant>, Vec<UnboundParticipant>), CanopyError> {
        let mut participants: HashMap<ThetaRole, Participant> = HashMap::new();
        let mut unbound = Vec::new();

        // First, try provider-based bindings
        for binding in bindings {
            if binding.confidence >= self.config.min_binding_confidence {
                if let Some(token) = analysis.syntax.get_token(binding.token_id) {
                    participants.insert(
                        binding.role,
                        Participant {
                            token_id: binding.token_id,
                            text: token.form.clone(),
                            number: None,
                            distributivity: None,
                            confidence: binding.confidence,
                        },
                    );
                }
            }
        }

        // If no bindings from provider, fall back to UTAH heuristics
        if participants.is_empty() {
            let (dep_participants, dep_unbound) =
                self.bind_by_utah(analysis, pred_id, expected_roles)?;
            participants = dep_participants;
            unbound = dep_unbound;
        }

        Ok((participants, unbound))
    }

    /// Bind participants using UTAH (Uniformity of Theta Assignment Hypothesis).
    ///
    /// This is a linguistic universal: certain syntactic positions consistently
    /// map to certain thematic roles across languages.
    #[allow(clippy::unnecessary_wraps)] // Consistent with other binding methods
    fn bind_by_utah(
        &self,
        analysis: &SentenceAnalysis,
        pred_id: TokenId,
        expected_roles: &[ThetaRole],
    ) -> Result<(HashMap<ThetaRole, Participant>, Vec<UnboundParticipant>), CanopyError> {
        let mut participants: HashMap<ThetaRole, Participant> = HashMap::new();
        let mut unbound = Vec::new();

        let dependents = analysis.get_dependents(pred_id);

        for arc in dependents {
            if let Some(token) = analysis.syntax.get_token(arc.dependent_id) {
                let role = self.dep_to_role(&arc.relation, analysis.metadata.is_passive);

                if let Some(role) = role {
                    if expected_roles.contains(&role) && !participants.contains_key(&role) {
                        participants.insert(
                            role,
                            Participant {
                                token_id: arc.dependent_id,
                                text: token.form.clone(),
                                number: None,
                                distributivity: None,
                                confidence: arc.confidence * 0.7, // Lower confidence for heuristic
                            },
                        );
                    } else {
                        unbound.push(UnboundParticipant {
                            token_id: arc.dependent_id,
                            text: token.form.clone(),
                            suggested_role: Some(role),
                            reason: if participants.contains_key(&role) {
                                UnbindingReason::ExtraCoreArgument
                            } else {
                                UnbindingReason::AmbiguousRole
                            },
                        });
                    }
                }
            }
        }

        Ok((participants, unbound))
    }

    /// Map dependency relation to theta role using UTAH.
    ///
    /// These are linguistic universals based on Baker's Uniformity of Theta
    /// Assignment Hypothesis - they hold across languages and don't require
    /// word-level knowledge.
    #[allow(clippy::unused_self)] // May use config in future
    fn dep_to_role(&self, dep: &DepRel, is_passive: bool) -> Option<ThetaRole> {
        match (dep, is_passive) {
            (DepRel::NsubjPass | DepRel::Nmod, _) | (DepRel::Nsubj, true) => Some(ThetaRole::Theme),
            (DepRel::Nsubj, false) => Some(ThetaRole::Agent),
            (DepRel::Obj, _) => Some(ThetaRole::Patient),
            (DepRel::Iobj, _) => Some(ThetaRole::Recipient),
            (DepRel::Obl, _) => Some(ThetaRole::Location), // Simplified
            (DepRel::Advmod, _) => Some(ThetaRole::Manner),
            _ => None,
        }
    }

    /// Detect voice from dependencies.
    #[allow(clippy::unused_self)] // May use config in future
    fn detect_voice(&self, analysis: &SentenceAnalysis, pred_id: TokenId) -> Voice {
        if analysis.metadata.is_passive {
            return Voice::Passive;
        }

        // Check for passive auxiliary
        for arc in &analysis.dependencies {
            if arc.head_id == pred_id && matches!(arc.relation, DepRel::AuxPass) {
                return Voice::Passive;
            }
        }

        Voice::Active
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{DependencyArc, LittleVType};
    use super::*;
    use crate::core::{DepRel as DepRelCore, UPos};
    use crate::runtime::{AnnotatedSyntax, AnnotatedToken, DecompositionSource, SenseId};

    // Helper to create mock syntax
    fn make_syntax(tokens: Vec<(&str, &str, UPos)>) -> AnnotatedSyntax {
        let annotated_tokens: Vec<AnnotatedToken> = tokens
            .into_iter()
            .enumerate()
            .map(|(id, (text, lemma, pos))| {
                AnnotatedToken::new(
                    TokenId::new(id),
                    text.to_string(),
                    lemma.to_string(),
                    pos,
                    DepRelCore::Root,
                    (0, text.len()),
                )
            })
            .collect();

        AnnotatedSyntax::new("test".to_string(), annotated_tokens)
    }

    // Helper to create a mock decomposition
    fn mock_decomposition(
        sense: &str,
        little_v: LittleVType,
        roles: Vec<ThetaRole>,
        confidence: f32,
    ) -> PredicateDecomposition {
        PredicateDecomposition::new(SenseId::new(sense), little_v, roles)
            .with_confidence(confidence)
            .with_source(DecompositionSource::VerbNet)
    }

    #[test]
    fn test_composer_default() {
        let composer = EventComposer::default();
        assert!((composer.config.min_decomposition_confidence - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compose_empty_sentence() {
        let composer = EventComposer::default();
        let syntax = make_syntax(vec![("The", "the", UPos::Det)]);
        let analysis = SentenceAnalysis::new("The", syntax);

        let result = composer
            .compose(&analysis, &HashMap::new(), &HashMap::new())
            .unwrap();
        assert!(!result.has_events());
    }

    #[test]
    fn test_compose_with_mock_decomposition() {
        let composer = EventComposer::default();

        // Create syntax: "John eats"
        let tokens = vec![
            AnnotatedToken::new(
                TokenId::new(0),
                "John".to_string(),
                "john".to_string(),
                UPos::Propn,
                DepRelCore::Nsubj,
                (0, 4),
            )
            .with_head(TokenId::new(1)),
            AnnotatedToken::new(
                TokenId::new(1),
                "eats".to_string(),
                "eat".to_string(),
                UPos::Verb,
                DepRelCore::Root,
                (5, 9),
            ),
        ];
        let syntax = AnnotatedSyntax::new("John eats".to_string(), tokens);

        let deps = vec![DependencyArc::new(
            TokenId::new(1),
            TokenId::new(0),
            DepRel::Nsubj,
        )];

        let analysis = SentenceAnalysis::new("John eats", syntax).with_dependencies(deps);

        // Create mock decomposition - provider returns this
        let mut decompositions = HashMap::new();
        decompositions.insert(
            TokenId::new(1),
            vec![mock_decomposition(
                "consume-39.1",
                LittleVType::Do,
                vec![ThetaRole::Agent],
                0.9,
            )],
        );

        let result = composer
            .compose(&analysis, &decompositions, &HashMap::new())
            .unwrap();

        assert!(result.has_events());
        let event = result.primary_event().unwrap();
        assert_eq!(event.little_v_type, LittleVType::Do);
        assert!(event.has_role(ThetaRole::Agent)); // Bound via UTAH fallback
    }

    #[test]
    fn test_compose_with_provider_bindings() {
        let composer = EventComposer::default();

        let tokens = vec![
            AnnotatedToken::new(
                TokenId::new(0),
                "John".to_string(),
                "john".to_string(),
                UPos::Propn,
                DepRelCore::Nsubj,
                (0, 4),
            ),
            AnnotatedToken::new(
                TokenId::new(1),
                "gives".to_string(),
                "give".to_string(),
                UPos::Verb,
                DepRelCore::Root,
                (5, 10),
            ),
            AnnotatedToken::new(
                TokenId::new(2),
                "Mary".to_string(),
                "mary".to_string(),
                UPos::Propn,
                DepRelCore::Iobj,
                (11, 15),
            ),
            AnnotatedToken::new(
                TokenId::new(3),
                "a".to_string(),
                "a".to_string(),
                UPos::Det,
                DepRelCore::Det,
                (16, 17),
            ),
            AnnotatedToken::new(
                TokenId::new(4),
                "book".to_string(),
                "book".to_string(),
                UPos::Noun,
                DepRelCore::Obj,
                (18, 22),
            ),
        ];
        let syntax = AnnotatedSyntax::new("John gives Mary a book".to_string(), tokens);
        let analysis = SentenceAnalysis::new("John gives Mary a book", syntax);

        // Mock decomposition for "give"
        let mut decompositions = HashMap::new();
        decompositions.insert(
            TokenId::new(1),
            vec![mock_decomposition(
                "give-13.1",
                LittleVType::Cause,
                vec![ThetaRole::Agent, ThetaRole::Theme, ThetaRole::Recipient],
                0.95,
            )],
        );

        // Mock role bindings from provider
        let mut bindings = HashMap::new();
        bindings.insert(
            TokenId::new(1),
            vec![
                RoleBinding::new(TokenId::new(0), ThetaRole::Agent, 0.95),
                RoleBinding::new(TokenId::new(2), ThetaRole::Recipient, 0.9),
                RoleBinding::new(TokenId::new(4), ThetaRole::Theme, 0.9),
            ],
        );

        let result = composer
            .compose(&analysis, &decompositions, &bindings)
            .unwrap();

        assert!(result.has_events());
        let event = result.primary_event().unwrap();
        assert_eq!(event.little_v_type, LittleVType::Cause);
        assert!(event.has_role(ThetaRole::Agent));
        assert!(event.has_role(ThetaRole::Recipient));
        assert!(event.has_role(ThetaRole::Theme));
    }

    #[test]
    fn test_compose_causative_with_sub_event() {
        let composer = EventComposer::default();

        let tokens = vec![
            AnnotatedToken::new(
                TokenId::new(0),
                "Mary".to_string(),
                "mary".to_string(),
                UPos::Propn,
                DepRelCore::Nsubj,
                (0, 4),
            ),
            AnnotatedToken::new(
                TokenId::new(1),
                "broke".to_string(),
                "break".to_string(),
                UPos::Verb,
                DepRelCore::Root,
                (5, 10),
            ),
            AnnotatedToken::new(
                TokenId::new(2),
                "the".to_string(),
                "the".to_string(),
                UPos::Det,
                DepRelCore::Det,
                (11, 14),
            ),
            AnnotatedToken::new(
                TokenId::new(3),
                "vase".to_string(),
                "vase".to_string(),
                UPos::Noun,
                DepRelCore::Obj,
                (15, 19),
            ),
        ];
        let syntax = AnnotatedSyntax::new("Mary broke the vase".to_string(), tokens);

        let deps = vec![
            DependencyArc::new(TokenId::new(1), TokenId::new(0), DepRel::Nsubj),
            DependencyArc::new(TokenId::new(1), TokenId::new(3), DepRel::Obj),
        ];
        let analysis = SentenceAnalysis::new("Mary broke the vase", syntax).with_dependencies(deps);

        // Causative decomposition with sub-event
        let become_sub = PredicateDecomposition::new(
            SenseId::new("break-45.1-become"),
            LittleVType::Become,
            vec![ThetaRole::Patient],
        )
        .with_confidence(0.9);

        let decomp = PredicateDecomposition::new(
            SenseId::new("break-45.1"),
            LittleVType::Cause,
            vec![ThetaRole::Agent, ThetaRole::Patient],
        )
        .with_confidence(0.9)
        .with_sub_event(become_sub);

        let mut decompositions = HashMap::new();
        decompositions.insert(TokenId::new(1), vec![decomp]);

        let result = composer
            .compose(&analysis, &decompositions, &HashMap::new())
            .unwrap();

        assert!(result.has_events());
        let event = result.primary_event().unwrap();
        assert_eq!(event.little_v_type, LittleVType::Cause);
        // Agent and Patient bound via UTAH
        assert!(event.has_role(ThetaRole::Agent));
        assert!(event.has_role(ThetaRole::Patient));
    }

    #[test]
    fn test_compose_no_decomposition_fails() {
        let composer = EventComposer::default();

        let tokens = vec![AnnotatedToken::new(
            TokenId::new(0),
            "runs".to_string(),
            "run".to_string(),
            UPos::Verb,
            DepRelCore::Root,
            (0, 4),
        )];
        let syntax = AnnotatedSyntax::new("runs".to_string(), tokens);
        let analysis = SentenceAnalysis::new("runs", syntax);

        // No decomposition provided
        let result = composer.compose(&analysis, &HashMap::new(), &HashMap::new());

        // Should succeed but with no events (predicate found but no decomposition)
        assert!(result.is_ok());
        let composition_result = result.unwrap();
        assert!(!composition_result.has_events());
    }

    #[test]
    fn test_dep_to_role_active() {
        let composer = EventComposer::default();
        assert_eq!(
            composer.dep_to_role(&DepRel::Nsubj, false),
            Some(ThetaRole::Agent)
        );
        assert_eq!(
            composer.dep_to_role(&DepRel::Obj, false),
            Some(ThetaRole::Patient)
        );
        assert_eq!(
            composer.dep_to_role(&DepRel::Iobj, false),
            Some(ThetaRole::Recipient)
        );
    }

    #[test]
    fn test_dep_to_role_passive() {
        let composer = EventComposer::default();
        assert_eq!(
            composer.dep_to_role(&DepRel::Nsubj, true),
            Some(ThetaRole::Theme)
        );
        assert_eq!(
            composer.dep_to_role(&DepRel::NsubjPass, false),
            Some(ThetaRole::Theme)
        );
    }

    #[test]
    fn test_confidence_threshold() {
        let config = EventComposerConfig {
            min_decomposition_confidence: 0.8,
            ..Default::default()
        };
        let composer = EventComposer::new(config);

        let tokens = vec![AnnotatedToken::new(
            TokenId::new(0),
            "runs".to_string(),
            "run".to_string(),
            UPos::Verb,
            DepRelCore::Root,
            (0, 4),
        )];
        let syntax = AnnotatedSyntax::new("runs".to_string(), tokens);
        let analysis = SentenceAnalysis::new("runs", syntax);

        // Low confidence decomposition
        let mut decompositions = HashMap::new();
        decompositions.insert(
            TokenId::new(0),
            vec![mock_decomposition(
                "run-51.3",
                LittleVType::Go,
                vec![ThetaRole::Agent],
                0.5, // Below threshold
            )],
        );

        let result = composer
            .compose(&analysis, &decompositions, &HashMap::new())
            .unwrap();

        // Event should be rejected due to low confidence
        assert!(!result.has_events());
    }
}
