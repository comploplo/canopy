//! Discourse context management.
//!
//! Manages discourse state across sentences, including:
//! - DRS construction
//! - Referent tracking
//! - Pronoun resolution
//! - Temporal ordering

use super::binding::{AnaphorType, BindingResult, PronounResolver};
use super::coherence::{
    CoherenceClassification, CoherenceClassifier, CoherenceEdge, CoherenceGraph, SentenceData,
    SentenceReferents,
};
use super::drs::{Drs, DrsCondition, DrsId, TemporalRelationType};
use super::moves::{DiscourseMove, MoveClassification, MoveClassifier};
use super::presupposition::{PresuppositionManager, TrackedPresupposition};
use super::qud::{QudReport, QudStack, QudUpdate};
use super::referent::{Gender, NumberFeature, ReferentId, ReferentRegistry};
use super::relevance::{RelevanceReport, RelevanceScorer};
use super::validation::{ValidationEngine, ValidationReport, ValidationStatus};
use crate::core::ThetaRole;
use crate::kernel::events::{ComposedEvent, ComposedEvents, LittleVType};
use crate::kernel::incremental::SurprisalModel;
use crate::kernel::logic::{
    ClosedWorldReasoner, ConsistencyResult, EntailmentResult, Proposition, Query, QueryResult,
    Reasoner,
};
use crate::runtime::AnnotatedSyntax;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for discourse processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscourseConfig {
    /// Salience decay factor between sentences.
    pub salience_decay: f32,

    /// Minimum confidence for pronoun resolution.
    pub min_resolution_confidence: f32,

    /// Whether to track temporal relations.
    pub track_temporal: bool,

    /// Maximum discourse referents to track.
    pub max_referents: usize,
}

impl Default for DiscourseConfig {
    fn default() -> Self {
        Self {
            salience_decay: 0.8,
            min_resolution_confidence: 0.3,
            track_temporal: true,
            max_referents: 100,
        }
    }
}

/// Discourse context - manages state across sentences.
#[derive(Debug, Clone)]
pub struct DiscourseContext {
    /// Configuration.
    config: DiscourseConfig,

    /// The main DRS being built.
    drs: Drs,

    /// Registry of discourse referents.
    registry: ReferentRegistry,

    /// Pronoun resolver.
    resolver: PronounResolver,

    /// Current sentence index.
    current_sentence: usize,

    /// Last event referent (for temporal ordering).
    last_event: Option<ReferentId>,

    /// Next DRS ID.
    next_drs_id: usize,

    /// Stack of Questions Under Discussion.
    qud_stack: QudStack,

    /// Chronological record of QUD updates.
    qud_history: Vec<QudUpdate>,

    /// Relevance assessments per sentence.
    relevance_history: Vec<RelevanceReport>,

    /// Validation engine ensuring discourse consistency.
    validation_engine: ValidationEngine,

    /// Validation reports per event.
    validation_history: Vec<ValidationReport>,

    /// Discourse move classifier.
    move_classifier: MoveClassifier,

    /// Discourse move history (one per sentence).
    move_history: Vec<MoveClassification>,

    /// Previous sentence's discourse move.
    prev_move: Option<DiscourseMove>,

    /// Coherence relation classifier.
    coherence_classifier: CoherenceClassifier,

    /// Graph of coherence relations between sentences.
    coherence_graph: CoherenceGraph,

    /// Referent tracker per sentence (for coherence analysis).
    sentence_referents: SentenceReferents,

    /// Cached events from previous sentence.
    prev_events: Option<ComposedEvents>,

    /// Cached tokens from previous sentence (for negation detection).
    prev_tokens: Vec<String>,

    /// Whether previous sentence had negation.
    prev_has_negation: bool,

    /// Presupposition manager for tracking and resolving presuppositions.
    presupposition_manager: PresuppositionManager,

    /// Optional surprisal model for surprisal-based coherence adjustment.
    surprisal_model: Option<SurprisalModelRef>,
}

/// Wrapper for surprisal model reference that implements Debug and Clone.
#[derive(Clone)]
struct SurprisalModelRef(Arc<dyn SurprisalModel>);

impl std::fmt::Debug for SurprisalModelRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<SurprisalModel>")
    }
}

impl Default for DiscourseContext {
    fn default() -> Self {
        Self::new(DiscourseConfig::default())
    }
}

impl DiscourseContext {
    /// Create a new discourse context.
    #[must_use]
    pub fn new(config: DiscourseConfig) -> Self {
        let mut resolver = PronounResolver::new();
        resolver.min_confidence = config.min_resolution_confidence;

        Self {
            config,
            drs: Drs::new(DrsId::new(0)),
            registry: ReferentRegistry::new(),
            resolver,
            current_sentence: 0,
            last_event: None,
            next_drs_id: 1,
            qud_stack: QudStack::default(),
            qud_history: Vec::new(),
            relevance_history: Vec::new(),
            validation_engine: ValidationEngine::default(),
            validation_history: Vec::new(),
            move_classifier: MoveClassifier::new(),
            move_history: Vec::new(),
            prev_move: None,
            coherence_classifier: CoherenceClassifier::new(),
            coherence_graph: CoherenceGraph::new(),
            sentence_referents: SentenceReferents::new(),
            prev_events: None,
            prev_tokens: Vec::new(),
            prev_has_negation: false,
            presupposition_manager: PresuppositionManager::new(),
            surprisal_model: None,
        }
    }

    /// Set a surprisal model for surprisal-based coherence adjustment.
    ///
    /// When a surprisal model is provided, coherence classification will use
    /// surprisal values to refine confidence scores.
    #[must_use]
    pub fn with_surprisal_model<L: SurprisalModel + 'static>(mut self, lm: L) -> Self {
        self.surprisal_model = Some(SurprisalModelRef(Arc::new(lm)));
        self
    }

    /// Set a shared surprisal model reference.
    #[must_use]
    pub fn with_surprisal_model_arc(mut self, lm: Arc<dyn SurprisalModel>) -> Self {
        self.surprisal_model = Some(SurprisalModelRef(lm));
        self
    }

    /// Begin processing a new sentence.
    pub fn begin_sentence(&mut self) {
        // Decay salience of existing referents
        self.registry.decay_salience(self.config.salience_decay);
    }

    /// Prepare sentence state, classify discourse move, detect presuppositions, and evaluate QUD cues.
    pub fn prepare_sentence(&mut self, syntax: &AnnotatedSyntax, events: Option<&ComposedEvents>) {
        self.begin_sentence();

        // Classify discourse move before QUD processing
        // (QUD may be updated by the move classification indirectly)
        let _ = self.classify_move(syntax, events);

        // Detect presupposition triggers in the sentence
        self.detect_presuppositions(syntax);

        let updates = self
            .qud_stack
            .observe_sentence(self.current_sentence, syntax, events);
        self.record_qud_updates(updates);
    }

    /// End processing of current sentence.
    pub fn end_sentence(&mut self) {
        // Update previous move for next sentence's classification
        if let Some(last_move) = self.move_history.last() {
            self.prev_move = Some(last_move.move_type);
        }

        self.current_sentence += 1;
        self.registry.next_sentence();
    }

    /// Classify the discourse move of the current sentence.
    pub fn classify_move(
        &mut self,
        syntax: &AnnotatedSyntax,
        events: Option<&ComposedEvents>,
    ) -> MoveClassification {
        // Get current relevance level if available
        let relevance = self.relevance_history.last().map(|r| r.level);

        let classification = self.move_classifier.classify(
            syntax,
            events,
            &self.qud_stack,
            relevance,
            self.prev_move,
        );

        self.move_history.push(classification.clone());
        classification
    }

    /// Get the discourse move history.
    #[must_use]
    pub fn move_history(&self) -> &[MoveClassification] {
        &self.move_history
    }

    /// Get the last classified discourse move.
    #[must_use]
    pub fn last_move(&self) -> Option<&MoveClassification> {
        self.move_history.last()
    }

    /// Classify the coherence relation between current and previous sentence.
    ///
    /// Should be called after events have been processed for the current sentence.
    pub fn classify_coherence(
        &mut self,
        syntax: &AnnotatedSyntax,
        events: Option<&ComposedEvents>,
    ) -> Option<CoherenceClassification> {
        // Can't classify coherence for first sentence
        if self.current_sentence == 0 {
            return None;
        }

        // Get tokens for marker detection
        let curr_tokens: Vec<String> = syntax.tokens.iter().map(|t| t.form.clone()).collect();
        let curr_has_negation = self.coherence_classifier.has_negation(&curr_tokens);

        // Get referents for current and previous sentence
        let curr_referents =
            SentenceReferents::extract_from_registry(&self.registry, self.current_sentence);
        let prev_referents = self
            .sentence_referents
            .get(self.current_sentence.saturating_sub(1))
            .to_vec();

        // Build sentence data structures
        let prev_data = SentenceData::new(
            self.prev_events.as_ref(),
            &prev_referents,
            self.prev_has_negation,
        );
        let curr_data = SentenceData::new(events, &curr_referents, curr_has_negation);

        // Classify
        let mut classification =
            self.coherence_classifier
                .classify(&prev_data, &curr_data, &curr_tokens);

        // Adjust confidence using surprisal if surprisal model is available
        if let Some(ref lm) = self.surprisal_model {
            classification = self.coherence_classifier.adjust_with_surprisal(
                classification,
                &self.prev_tokens,
                &curr_tokens,
                lm.0.as_ref(),
            );
        }

        // Add edge to graph
        let edge = CoherenceEdge {
            from_sentence: self.current_sentence.saturating_sub(1),
            to_sentence: self.current_sentence,
            classification: classification.clone(),
        };
        self.coherence_graph.add_edge(edge);

        Some(classification)
    }

    /// Finalize sentence processing and cache state for coherence analysis.
    ///
    /// Call after all events have been processed.
    pub fn finalize_sentence(&mut self, syntax: &AnnotatedSyntax, events: Option<&ComposedEvents>) {
        // Record referents introduced in this sentence
        let referents =
            SentenceReferents::extract_from_registry(&self.registry, self.current_sentence);
        self.sentence_referents
            .record(self.current_sentence, referents);

        // Cache state for next sentence's coherence analysis
        self.prev_events = events.cloned();
        self.prev_tokens = syntax.tokens.iter().map(|t| t.form.clone()).collect();
        self.prev_has_negation = self.coherence_classifier.has_negation(&self.prev_tokens);
    }

    /// Get the coherence graph.
    #[must_use]
    pub fn coherence_graph(&self) -> &CoherenceGraph {
        &self.coherence_graph
    }

    /// Get the coherence relation to the previous sentence (if any).
    #[must_use]
    pub fn last_coherence(&self) -> Option<&CoherenceEdge> {
        if self.current_sentence == 0 {
            return None;
        }
        self.coherence_graph.relation_between(
            self.current_sentence.saturating_sub(1),
            self.current_sentence,
        )
    }

    /// Detect and track presuppositions from the current sentence.
    pub fn detect_presuppositions(&mut self, syntax: &AnnotatedSyntax) {
        let tokens: Vec<String> = syntax.tokens.iter().map(|t| t.form.clone()).collect();
        self.presupposition_manager
            .detect_from_tokens(&tokens, self.current_sentence);
    }

    /// Resolve all pending presuppositions against the current DRS.
    pub fn resolve_presuppositions(&mut self) {
        self.presupposition_manager.resolve_all(&self.drs);
    }

    /// Get all tracked presuppositions.
    #[must_use]
    pub fn presuppositions(&self) -> &[TrackedPresupposition] {
        self.presupposition_manager.all()
    }

    /// Get presuppositions from the current sentence.
    #[must_use]
    pub fn current_presuppositions(&self) -> Vec<&TrackedPresupposition> {
        self.presupposition_manager
            .from_sentence(self.current_sentence)
    }

    /// Get the presupposition manager for advanced access.
    #[must_use]
    pub fn presupposition_manager(&self) -> &PresuppositionManager {
        &self.presupposition_manager
    }

    /// Get the current DRS.
    #[must_use]
    pub fn drs(&self) -> &Drs {
        &self.drs
    }

    /// Get mutable reference to DRS.
    pub fn drs_mut(&mut self) -> &mut Drs {
        &mut self.drs
    }

    /// Get the referent registry.
    #[must_use]
    pub fn registry(&self) -> &ReferentRegistry {
        &self.registry
    }

    /// Introduce a new entity referent.
    pub fn introduce_entity(&mut self, name: impl Into<String>) -> ReferentId {
        let id = self.registry.introduce_entity(name.into());

        // Add to DRS universe
        if let Some(referent) = self.registry.get(id) {
            self.drs.add_referent(referent.clone());
        }

        id
    }

    /// Introduce an entity with specific features.
    pub fn introduce_entity_with_features(
        &mut self,
        name: impl Into<String>,
        gender: Gender,
        number: NumberFeature,
    ) -> ReferentId {
        let id = self.registry.introduce_entity(name);

        if let Some(r) = self.registry.get_mut(id) {
            r.gender = gender;
            r.number = number;
        }

        if let Some(referent) = self.registry.get(id) {
            self.drs.add_referent(referent.clone());
        }

        id
    }

    /// Introduce an event referent.
    pub fn introduce_event(&mut self, predicate: impl Into<String>) -> ReferentId {
        let id = self.registry.introduce_event(predicate);

        if let Some(referent) = self.registry.get(id) {
            self.drs.add_referent(referent.clone());
        }

        // Track temporal ordering
        if self.config.track_temporal {
            if let Some(prev_event) = self.last_event {
                // By default, events in sequence are ordered temporally
                self.drs.add_condition(DrsCondition::TemporalRelation {
                    relation: TemporalRelationType::Before,
                    event1: prev_event,
                    event2: id,
                });
            }
            self.last_event = Some(id);
        }

        id
    }

    /// Add a predicate condition for a referent.
    pub fn add_predicate(&mut self, predicate: impl Into<String>, referent: ReferentId) {
        self.drs.add_predicate(predicate, referent);
    }

    /// Add a theta role binding.
    pub fn add_theta_role(&mut self, event: ReferentId, role: ThetaRole, filler: ReferentId) {
        self.drs.add_theta_role(event, role, filler);

        // Boost salience of filler
        self.registry.boost_salience(filler, 0.2);
    }

    /// Resolve a pronoun.
    pub fn resolve_pronoun(
        &mut self,
        anaphor_type: AnaphorType,
        gender: Option<Gender>,
        number: Option<NumberFeature>,
    ) -> BindingResult {
        let result = self.resolver.resolve(
            &self.registry,
            anaphor_type,
            gender,
            number,
            self.current_sentence,
        );

        // Boost salience of resolved antecedent
        if let Some(antecedent) = result.antecedent {
            self.registry.boost_salience(antecedent, 0.3);
        }

        result
    }

    /// Process composed events from Layer 2.
    pub fn process_events(&mut self, events: &ComposedEvents) {
        self.score_relevance(events);

        for event in &events.events {
            let validation = self.validation_engine.assess(self.current_sentence, event);
            self.validation_history.push(validation.clone());

            if validation.status != ValidationStatus::Accepted {
                continue;
            }

            self.process_single_event(event);
        }

        let updates = self
            .qud_stack
            .resolve_with_events(events, self.current_sentence);
        self.record_qud_updates(updates);
    }

    /// Process a single composed event.
    fn process_single_event(&mut self, event: &ComposedEvent) {
        // Create event referent
        let event_id = self.introduce_event(&event.predicate);

        // Add event predicate
        self.drs.add_predicate(&event.predicate, event_id);

        // Process participants
        for (role, participant) in &event.participants {
            // Look up or create referent for participant
            // For now, create new referent (real implementation would resolve)
            let participant_id = self.introduce_entity(&participant.text);

            // Add theta role
            self.add_theta_role(event_id, *role, participant_id);
        }

        // Add aspectual class as property
        let aspect_pred = match event.little_v_type {
            LittleVType::Cause => "causative",
            LittleVType::Become => "inchoative",
            LittleVType::Be => "stative",
            LittleVType::Do => "activity",
            LittleVType::Experience => "psychological",
            LittleVType::Go => "motion",
            LittleVType::Have => "possessive",
            LittleVType::Say => "communication",
            LittleVType::Exist => "existential",
        };
        self.drs.add_predicate(aspect_pred, event_id);

        // Handle polarity
        if !event.polarity {
            // Negated event - would need proper DRS negation
            self.drs.add_predicate("negated", event_id);
        }
    }

    /// Get current sentence index.
    #[must_use]
    pub fn current_sentence(&self) -> usize {
        self.current_sentence
    }

    /// Get referent count.
    #[must_use]
    pub fn referent_count(&self) -> usize {
        self.registry.len()
    }

    /// Allocate a new DRS ID.
    pub fn next_drs_id(&mut self) -> DrsId {
        let id = DrsId::new(self.next_drs_id);
        self.next_drs_id += 1;
        id
    }

    /// Accessor for the QUD stack (primarily for testing/telemetry).
    #[must_use]
    pub fn qud_stack(&self) -> &QudStack {
        &self.qud_stack
    }

    /// Snapshot suitable for trace/CLI output.
    #[must_use]
    pub fn qud_report(&self) -> QudReport {
        self.qud_stack.report(&self.qud_history)
    }

    /// Recorded relevance reports for each processed sentence.
    #[must_use]
    pub fn relevance_history(&self) -> &[RelevanceReport] {
        &self.relevance_history
    }

    /// Get validation reports accumulated so far.
    #[must_use]
    pub fn validation_history(&self) -> &[ValidationReport] {
        &self.validation_history
    }

    fn record_qud_updates(&mut self, updates: Vec<QudUpdate>) {
        if !updates.is_empty() {
            self.qud_history.extend(updates);
        }
    }

    fn score_relevance(&mut self, events: &ComposedEvents) {
        let question = self
            .qud_stack
            .peek()
            .filter(|issue| issue.introduced_at < self.current_sentence)
            .cloned();
        let report = RelevanceScorer::score(self.current_sentence, question.as_ref(), events);
        self.relevance_history.push(report);
    }

    // =========================================================================
    // Logic Layer: Query Answering and Reasoning
    // =========================================================================

    /// Check if the current discourse is internally consistent.
    ///
    /// Returns a `ConsistencyResult` indicating whether any contradictions
    /// were detected in the DRS.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = context.is_consistent();
    /// if !result.consistent {
    ///     for conflict in &result.conflicts {
    ///         println!("Conflict: {}", conflict.description);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn is_consistent(&self) -> ConsistencyResult {
        let reasoner = ClosedWorldReasoner::new();
        reasoner.check_consistent(&self.drs)
    }

    /// Check if a proposition is entailed by the discourse.
    ///
    /// Under the closed-world assumption, if something is not stated or
    /// derivable, it is considered false.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let prop = Proposition::simple("leave", ThetaRole::Agent, "John");
    /// let result = context.entails(&prop);
    /// if result.is_yes() {
    ///     println!("John left is entailed");
    /// }
    /// ```
    #[must_use]
    pub fn entails(&self, proposition: &Proposition) -> EntailmentResult {
        let reasoner = ClosedWorldReasoner::new();
        reasoner.entails(&self.drs, proposition)
    }

    /// Answer a query against the discourse.
    ///
    /// Supports yes/no questions, wh-questions, and existence checks.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Who left?
    /// let query = Query::wh("leave", ThetaRole::Agent);
    /// let result = context.query(&query);
    /// for value in result.all_values_for("?agent") {
    ///     println!("Answer: {}", value);
    /// }
    /// ```
    #[must_use]
    pub fn query(&self, query: &Query) -> QueryResult {
        let reasoner = ClosedWorldReasoner::new();
        reasoner.answer(&self.drs, query)
    }

    /// Check if adding a condition would create a contradiction.
    ///
    /// Useful for checking whether a new assertion is consistent with
    /// existing discourse before committing it.
    #[must_use]
    pub fn would_contradict(&self, condition: &DrsCondition) -> bool {
        let reasoner = ClosedWorldReasoner::new();
        reasoner.would_contradict(&self.drs, std::slice::from_ref(condition))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AspectualClass, DepRel, MorphFeatures, ThetaRole, UPos, Voice};
    use crate::kernel::discourse::{QudUpdateAction, RelevanceLevel, ValidationStatus};
    use crate::kernel::events::{ComposedEvent, ComposedEvents, LittleVType, Participant};
    use crate::runtime::{AnnotatedSyntax, AnnotatedToken, TokenId};
    use std::collections::HashMap;

    #[test]
    fn test_context_creation() {
        let ctx = DiscourseContext::default();
        assert_eq!(ctx.current_sentence(), 0);
        assert_eq!(ctx.referent_count(), 0);
    }

    #[test]
    fn test_introduce_entity() {
        let mut ctx = DiscourseContext::default();
        let id = ctx.introduce_entity("John");

        assert_eq!(ctx.referent_count(), 1);
        assert!(ctx.drs().get_referent(id).is_some());
    }

    #[test]
    fn test_introduce_event() {
        let mut ctx = DiscourseContext::default();
        let id = ctx.introduce_event("walk");

        assert_eq!(ctx.referent_count(), 1);
        let referent = ctx.registry().get(id).unwrap();
        assert!(referent.is_event);
    }

    #[test]
    fn test_sentence_progression() {
        let mut ctx = DiscourseContext::default();

        ctx.begin_sentence();
        ctx.introduce_entity("John");
        ctx.end_sentence();

        assert_eq!(ctx.current_sentence(), 1);

        ctx.begin_sentence();
        ctx.introduce_entity("Mary");
        ctx.end_sentence();

        assert_eq!(ctx.current_sentence(), 2);
        assert_eq!(ctx.referent_count(), 2);
    }

    #[test]
    fn test_pronoun_resolution() {
        let mut ctx = DiscourseContext::default();

        // Introduce John
        ctx.begin_sentence();
        let john_id =
            ctx.introduce_entity_with_features("John", Gender::Masculine, NumberFeature::Singular);
        ctx.end_sentence();

        // Resolve "he"
        ctx.begin_sentence();
        let result = ctx.resolve_pronoun(
            AnaphorType::Personal,
            Some(Gender::Masculine),
            Some(NumberFeature::Singular),
        );

        assert!(result.is_resolved());
        assert_eq!(result.antecedent, Some(john_id));
    }

    #[test]
    fn test_temporal_ordering() {
        let config = DiscourseConfig {
            track_temporal: true,
            ..Default::default()
        };
        let mut ctx = DiscourseContext::new(config);

        ctx.begin_sentence();
        let e1 = ctx.introduce_event("walk");
        let e2 = ctx.introduce_event("fall");
        ctx.end_sentence();

        // Check temporal relation was added
        let has_temporal = ctx.drs().conditions.iter().any(|c| {
            matches!(c, DrsCondition::TemporalRelation {
                relation: TemporalRelationType::Before,
                event1,
                event2,
            } if *event1 == e1 && *event2 == e2)
        });

        assert!(has_temporal);
    }

    #[test]
    fn test_theta_role_addition() {
        let mut ctx = DiscourseContext::default();

        ctx.begin_sentence();
        let event_id = ctx.introduce_event("walk");
        let john_id = ctx.introduce_entity("John");
        ctx.add_theta_role(event_id, ThetaRole::Agent, john_id);
        ctx.end_sentence();

        // Check theta role was added
        let has_role = ctx.drs().conditions.iter().any(|c| {
            matches!(c, DrsCondition::ThetaRole {
                event_id: e,
                role: ThetaRole::Agent,
                filler: f,
            } if *e == event_id && *f == john_id)
        });

        assert!(has_role);
    }

    #[test]
    fn test_salience_decay() {
        let mut ctx = DiscourseContext::default();

        // Introduce entity in first sentence
        ctx.begin_sentence();
        let id = ctx.introduce_entity("John");
        if let Some(r) = ctx.registry.get_mut(id) {
            r.salience = 1.0;
        }
        ctx.end_sentence();

        // Begin new sentence (triggers decay)
        ctx.begin_sentence();

        let salience = ctx.registry().get(id).unwrap().salience;
        assert!(salience < 1.0);
        assert!((salience - 0.8).abs() < 0.01); // Default decay is 0.8
    }

    #[test]
    fn test_drs_box_notation() {
        let mut ctx = DiscourseContext::default();

        ctx.begin_sentence();
        let john_id = ctx.introduce_entity("John");
        ctx.add_predicate("man", john_id);
        let event_id = ctx.introduce_event("walk");
        ctx.add_theta_role(event_id, ThetaRole::Agent, john_id);
        ctx.end_sentence();

        let notation = ctx.drs().to_box_notation();
        assert!(notation.contains("man"));
        assert!(notation.contains("Agent"));
    }

    #[test]
    fn test_qud_push_for_explicit_question() {
        let mut ctx = DiscourseContext::default();
        let syntax = make_question_syntax();

        ctx.prepare_sentence(&syntax, None);
        assert_eq!(ctx.qud_stack().len(), 1);
        let report = ctx.qud_report();
        assert_eq!(report.stack_depth, 1);
        assert!(report.active_question.is_some());
    }

    #[test]
    fn test_qud_resolves_after_answer() {
        let mut ctx = DiscourseContext::default();
        let question = make_question_syntax();
        let question_events = make_question_events();
        ctx.prepare_sentence(&question, Some(&question_events));
        ctx.process_events(&question_events);
        ctx.end_sentence();

        let answer_syntax = make_statement_syntax();
        let events = make_answer_events();
        ctx.prepare_sentence(&answer_syntax, Some(&events));
        ctx.process_events(&events);
        ctx.end_sentence();

        assert_eq!(ctx.qud_stack().len(), 0);
        let report = ctx.qud_report();
        assert!(report
            .history
            .iter()
            .any(|entry| matches!(entry.action, QudUpdateAction::Resolved)));
    }

    #[test]
    fn test_relevance_history_levels() {
        let mut ctx = DiscourseContext::default();

        let question = make_question_syntax();
        let question_events = make_question_events();
        ctx.prepare_sentence(&question, Some(&question_events));
        ctx.process_events(&question_events);
        ctx.end_sentence();

        let answer_syntax = make_statement_syntax();
        let events = make_answer_events();
        ctx.prepare_sentence(&answer_syntax, Some(&events));
        ctx.process_events(&events);
        ctx.end_sentence();

        let reports = ctx.relevance_history();
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].level, RelevanceLevel::NoQuestion);
        // With question type detection, "Who" questions expect Agent or Experiencer.
        // The answer only provides Agent, so it's Partial (one of the expected roles).
        assert_eq!(reports[1].level, RelevanceLevel::Partial);

        // Introduce a new unresolved question.
        let follow_up = make_question_syntax();
        let follow_events = make_question_events();
        ctx.prepare_sentence(&follow_up, Some(&follow_events));
        ctx.process_events(&follow_events);
        ctx.end_sentence();

        let off_topic = make_offtopic_events();
        let off_syntax = make_offtopic_syntax();
        ctx.prepare_sentence(&off_syntax, Some(&off_topic));
        ctx.process_events(&off_topic);
        ctx.end_sentence();

        let reports = ctx.relevance_history();
        assert_eq!(reports.len(), 4);
        // The "off-topic" event still has an Agent role, which matches one of the
        // expected roles for "Who" questions. The permissive scoring gives Partial.
        assert_eq!(reports.last().unwrap().level, RelevanceLevel::Partial);
    }

    #[test]
    fn test_validation_contradiction() {
        let mut ctx = DiscourseContext::default();

        let statement = make_statement_syntax();
        let events = make_answer_events();
        ctx.prepare_sentence(&statement, Some(&events));
        ctx.process_events(&events);
        ctx.end_sentence();

        let neg_syntax = make_negative_statement_syntax();
        let neg_events = make_negative_answer_events();
        ctx.prepare_sentence(&neg_syntax, Some(&neg_events));
        ctx.process_events(&neg_events);
        ctx.end_sentence();

        let validations = ctx.validation_history();
        assert_eq!(validations.len(), 2);
        assert_eq!(
            validations.last().unwrap().status,
            ValidationStatus::Contradiction
        );
    }

    fn make_question_syntax() -> AnnotatedSyntax {
        let mut who = AnnotatedToken::new(
            TokenId::new(0),
            "Who".to_string(),
            "who".to_string(),
            UPos::Pron,
            DepRel::Nsubj,
            (0, 3),
        );
        who.head = Some(TokenId::new(1));

        let mut verb = AnnotatedToken::new(
            TokenId::new(1),
            "left".to_string(),
            "leave".to_string(),
            UPos::Verb,
            DepRel::Root,
            (4, 9),
        );
        verb.feats = MorphFeatures::default();

        let mut punct = AnnotatedToken::new(
            TokenId::new(2),
            "?".to_string(),
            "?".to_string(),
            UPos::Punct,
            DepRel::Punct,
            (9, 10),
        );
        punct.head = Some(TokenId::new(1));

        AnnotatedSyntax::new("Who left?".to_string(), vec![who, verb, punct])
    }

    fn make_statement_syntax() -> AnnotatedSyntax {
        let mut subj = AnnotatedToken::new(
            TokenId::new(0),
            "John".to_string(),
            "john".to_string(),
            UPos::Propn,
            DepRel::Nsubj,
            (0, 4),
        );
        subj.head = Some(TokenId::new(1));

        let mut verb = AnnotatedToken::new(
            TokenId::new(1),
            "left".to_string(),
            "leave".to_string(),
            UPos::Verb,
            DepRel::Root,
            (5, 10),
        );
        verb.feats = MorphFeatures::default();

        AnnotatedSyntax::new("John left.".to_string(), vec![subj, verb])
    }

    fn make_negative_statement_syntax() -> AnnotatedSyntax {
        let mut subj = AnnotatedToken::new(
            TokenId::new(0),
            "John".to_string(),
            "john".to_string(),
            UPos::Propn,
            DepRel::Nsubj,
            (0, 4),
        );
        subj.head = Some(TokenId::new(2));

        let mut aux = AnnotatedToken::new(
            TokenId::new(1),
            "did".to_string(),
            "do".to_string(),
            UPos::Aux,
            DepRel::Aux,
            (5, 8),
        );
        aux.head = Some(TokenId::new(2));

        let mut neg = AnnotatedToken::new(
            TokenId::new(2),
            "not".to_string(),
            "not".to_string(),
            UPos::Part,
            DepRel::Advmod,
            (9, 12),
        );
        neg.head = Some(TokenId::new(3));

        let mut verb = AnnotatedToken::new(
            TokenId::new(3),
            "leave".to_string(),
            "leave".to_string(),
            UPos::Verb,
            DepRel::Root,
            (13, 18),
        );
        verb.feats = MorphFeatures::default();

        AnnotatedSyntax::new(
            "John did not leave.".to_string(),
            vec![subj, aux, neg, verb],
        )
    }

    fn make_offtopic_syntax() -> AnnotatedSyntax {
        let mut subj = AnnotatedToken::new(
            TokenId::new(0),
            "Music".to_string(),
            "music".to_string(),
            UPos::Noun,
            DepRel::Nsubj,
            (0, 5),
        );
        subj.head = Some(TokenId::new(1));

        let mut verb = AnnotatedToken::new(
            TokenId::new(1),
            "played".to_string(),
            "play".to_string(),
            UPos::Verb,
            DepRel::Root,
            (6, 12),
        );
        verb.feats = MorphFeatures::default();

        AnnotatedSyntax::new("Music played.".to_string(), vec![subj, verb])
    }

    fn make_question_events() -> ComposedEvents {
        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, Participant::new(TokenId::new(0), "Who"));

        let event = ComposedEvent {
            id: 0,
            predicate: "leave".to_string(),
            little_v_type: LittleVType::Go,
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            token_span: (TokenId::new(0), TokenId::new(1)),
            source_sense: None,
            decomposition_confidence: 1.0,
            binding_confidence: 1.0,
            presuppositions: Vec::new(),
            polarity: true,
        };

        ComposedEvents {
            events: vec![event],
            unbound_participants: Vec::new(),
            confidence: 1.0,
            sources: Vec::new(),
        }
    }

    fn make_answer_events() -> ComposedEvents {
        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, Participant::new(TokenId::new(0), "John"));

        let event = ComposedEvent {
            id: 0,
            predicate: "leave".to_string(),
            little_v_type: LittleVType::Go,
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            token_span: (TokenId::new(0), TokenId::new(1)),
            source_sense: None,
            decomposition_confidence: 1.0,
            binding_confidence: 1.0,
            presuppositions: Vec::new(),
            polarity: true,
        };

        ComposedEvents {
            events: vec![event],
            unbound_participants: Vec::new(),
            confidence: 1.0,
            sources: Vec::new(),
        }
    }

    fn make_negative_answer_events() -> ComposedEvents {
        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, Participant::new(TokenId::new(0), "John"));

        let event = ComposedEvent {
            id: 1,
            predicate: "leave".to_string(),
            little_v_type: LittleVType::Go,
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            token_span: (TokenId::new(0), TokenId::new(3)),
            source_sense: None,
            decomposition_confidence: 1.0,
            binding_confidence: 1.0,
            presuppositions: Vec::new(),
            polarity: false,
        };

        ComposedEvents {
            events: vec![event],
            unbound_participants: Vec::new(),
            confidence: 1.0,
            sources: Vec::new(),
        }
    }

    fn make_offtopic_events() -> ComposedEvents {
        let mut participants = HashMap::new();
        participants.insert(ThetaRole::Agent, Participant::new(TokenId::new(0), "Music"));

        let event = ComposedEvent {
            id: 0,
            predicate: "play".to_string(),
            little_v_type: LittleVType::Do,
            participants,
            aspect: AspectualClass::Activity,
            voice: Voice::Active,
            token_span: (TokenId::new(0), TokenId::new(1)),
            source_sense: None,
            decomposition_confidence: 1.0,
            binding_confidence: 1.0,
            presuppositions: Vec::new(),
            polarity: true,
        };

        ComposedEvents {
            events: vec![event],
            unbound_participants: Vec::new(),
            confidence: 1.0,
            sources: Vec::new(),
        }
    }
}
