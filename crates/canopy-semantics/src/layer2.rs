//! Layer 2: Pure Semantic Analysis
//!
//! This module implements semantic analysis that processes pre-parsed Layer 1 output
//! to create event structures and assign theta roles. Layer 2 operates on already
//! parsed linguistic data and does not directly interface with UDPipe or raw text.
//!
//! ## Clean Architecture
//!
//! ```text
//! Layer 1 (Parser): Text → UDPipe → VerbNet → Enhanced Words
//!                                    ↓
//! Layer 2 (Semantics): Enhanced Words → Events + Theta Roles
//! ```
//!
//! Layer 2 receives structured linguistic data and focuses purely on semantic
//! interpretation without external dependencies.

use crate::ThetaRoleType;
use crate::events::event_semantics::{MovementType as EventMovementType, Predicate};
use crate::events::{Event, EventId, EventTime, MovementChain, Participant, PredicateType};
use crate::syntax::{EventDecomposer, MovementDetector};
use crate::verbnet::VerbNetEngine;
use canopy_core::{DepRel, MorphFeatures, UPos, Word};
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info};

/// Errors that can occur in Layer 2 semantic processing
#[derive(Debug, Error)]
pub enum Layer2Error {
    #[error("Theta assignment error: {0}")]
    ThetaAssignment(String),

    #[error("Event creation error: {0}")]
    EventCreation(String),

    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },
}

/// Performance mode for Layer 2 processing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceMode {
    /// Accuracy mode (UDPipe 2.15 without cache)
    Accuracy,
    /// Balanced mode (M4 - basic caching)
    Balanced,
    /// Speed mode (M5 - full optimization)
    Speed,
}

impl Default for PerformanceMode {
    fn default() -> Self {
        Self::Accuracy // Default to accuracy for M3
    }
}

/// Configuration for Layer 2 semantic processing
#[derive(Debug, Clone)]
pub struct Layer2Config {
    /// Enable theta role assignment (default: true)
    pub enable_theta_assignment: bool,

    /// Enable event structure creation (default: true)
    pub enable_event_creation: bool,

    /// Enable detailed logging
    pub enable_logging: bool,

    /// Debug mode
    pub debug: bool,

    /// Performance mode
    pub performance_mode: PerformanceMode,

    /// Performance threshold in microseconds
    pub performance_threshold_us: u64,

    /// Enable performance logging
    pub enable_performance_logging: bool,

    /// Enable VerbNet processing
    pub enable_verbnet: bool,

    /// Enable little v decomposition
    pub enable_little_v_decomposition: bool,
}

impl Default for Layer2Config {
    fn default() -> Self {
        Self {
            enable_theta_assignment: true,
            enable_event_creation: true,
            enable_logging: false,
            debug: false,
            performance_mode: PerformanceMode::default(),
            performance_threshold_us: 500,
            enable_performance_logging: true,
            enable_verbnet: true,
            enable_little_v_decomposition: true,
        }
    }
}

/// Performance metrics for Layer 2 operations
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Layer2Metrics {
    /// Total processing time in microseconds
    pub total_time_us: u64,

    /// Time spent on theta role assignment in microseconds
    pub theta_assignment_time_us: u64,

    /// Time spent on event creation in microseconds
    pub event_creation_time_us: u64,

    /// Time spent on little v decomposition in microseconds
    pub little_v_decomposition_time_us: Option<u64>,

    /// Number of words processed
    pub words_processed: usize,

    /// Number of events created
    pub events_created: usize,

    /// Number of events with little v decomposition
    pub events_with_little_v: usize,
}

impl Layer2Metrics {
    /// Calculate words processed per second
    pub fn words_per_second(&self) -> f64 {
        if self.total_time_us == 0 {
            0.0
        } else {
            self.words_processed as f64 / (self.total_time_us as f64 / 1_000_000.0)
        }
    }
}

/// Semantic analysis result from Layer 2
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticAnalysis {
    /// Enhanced words with semantic features
    pub words: Vec<canopy_core::EnhancedWord>,

    /// Events extracted from the text
    pub events: Vec<Event>,

    /// Theta role assignments by event
    pub theta_assignments: HashMap<EventId, HashMap<ThetaRoleType, Participant>>,

    /// Overall confidence score
    pub confidence: f64,

    /// Performance metrics
    pub metrics: Layer2Metrics,
}

/// Layer 2 semantic analyzer
///
/// Layer 2 receives pre-analyzed EnhancedWords from Layer 1 and creates
/// semantic events and structures without re-computing theta roles.
#[derive(Debug)]
pub struct Layer2Analyzer {
    /// Configuration
    config: Layer2Config,
    /// Movement detector for passive/raising/wh-movement
    movement_detector: MovementDetector,
    /// Event decomposer for little v analysis
    event_decomposer: EventDecomposer,
    /// VerbNet engine for semantic analysis
    verbnet_engine: VerbNetEngine,
}

impl Layer2Analyzer {
    /// Create new Layer 2 analyzer with default configuration
    pub fn new() -> Self {
        Self::with_config(Layer2Config::default())
    }

    /// Create new Layer 2 analyzer with custom configuration
    pub fn with_config(config: Layer2Config) -> Self {
        Self {
            config,
            movement_detector: MovementDetector::new(),
            event_decomposer: EventDecomposer::new(),
            verbnet_engine: VerbNetEngine::new_with_test_data(),
        }
    }

    /// Create new Layer 2 analyzer with custom dependencies (dependency injection)
    pub fn with_dependencies(
        config: Layer2Config,
        movement_detector: MovementDetector,
        event_decomposer: EventDecomposer,
        verbnet_engine: VerbNetEngine,
    ) -> Self {
        Self {
            config,
            movement_detector,
            event_decomposer,
            verbnet_engine,
        }
    }

    /// Analyze enhanced words from Layer 1 to create semantic structures
    pub fn analyze(&mut self, enhanced_words: Vec<Word>) -> Result<SemanticAnalysis, Layer2Error> {
        let start_time = Instant::now();
        let mut metrics = Layer2Metrics::default();
        metrics.words_processed = enhanced_words.len();

        if self.config.debug {
            debug!("Layer2: Processing {} enhanced words", enhanced_words.len());
        }

        // Step 1: Extract theta role assignments using VerbNet
        let theta_start = Instant::now();
        let theta_assignments = if self.config.enable_verbnet {
            self.extract_verbnet_theta_assignments(&enhanced_words)
        } else {
            self.extract_basic_theta_assignments_from_words(&enhanced_words)
        };
        metrics.theta_assignment_time_us = theta_start.elapsed().as_micros() as u64;

        // Step 2: Event structure creation (if enabled)
        let event_start = Instant::now();
        let events = if self.config.enable_event_creation {
            self.create_event_structures_from_words(
                &enhanced_words,
                &theta_assignments,
                &mut metrics,
            )?
        } else {
            Vec::new()
        };
        metrics.event_creation_time_us = event_start.elapsed().as_micros() as u64;

        metrics.total_time_us = start_time.elapsed().as_micros() as u64;
        metrics.events_created = events.len();

        if self.config.enable_logging {
            self.log_performance_metrics(&metrics);
        }

        // Calculate overall confidence (simplified)
        let confidence = self.calculate_overall_confidence(&events);

        // Convert words to enhanced words format for output
        let enhanced_words: Vec<canopy_core::EnhancedWord> = enhanced_words
            .into_iter()
            .map(|word| canopy_core::EnhancedWord {
                base: word,
                semantic_features: canopy_core::SemanticFeatures::default(),
                confidence: canopy_core::FeatureConfidence::default(),
            })
            .collect();

        Ok(SemanticAnalysis {
            words: enhanced_words,
            events,
            theta_assignments,
            confidence,
            metrics,
        })
    }

    /// Extract VerbNet-based theta role assignments
    fn extract_verbnet_theta_assignments(
        &mut self,
        words: &[Word],
    ) -> HashMap<EventId, HashMap<ThetaRoleType, Participant>> {
        let mut assignments = HashMap::new();

        // Find verbs and use VerbNet for theta role assignment
        for word in words {
            if word.upos == UPos::Verb {
                let event_id = EventId(word.id);
                let mut event_assignments = HashMap::new();

                // Build syntactic context for VerbNet lookup
                let syntactic_context = self.build_syntactic_context(words, word);

                // Look up verb in VerbNet
                let (dependency_pattern, arguments) = syntactic_context;
                let verbnet_results = self.verbnet_engine.lookup_with_context(
                    &word.lemma,
                    &dependency_pattern,
                    &arguments,
                    "active",  // TODO: Detect voice from movement analysis
                    "present", // TODO: Extract tense/aspect from morphology
                );

                if let Ok(results) = verbnet_results {
                    if !results.theta_assignments.is_empty() {
                        // Use the best theta assignment (highest confidence)
                        let (best_theta_roles, _confidence) = &results.theta_assignments[0];

                        // Map VerbNet theta roles to syntactic arguments
                        for theta_role in best_theta_roles {
                            if let Some(participant) =
                                self.find_syntactic_argument(words, word, &theta_role.role_type)
                            {
                                event_assignments.insert(theta_role.role_type, participant);
                            }
                        }

                        if self.config.debug {
                            debug!(
                                "VerbNet lookup for '{}': found {} theta roles",
                                word.lemma,
                                event_assignments.len()
                            );
                        }
                    } else {
                        // Fall back to basic assignment if no VerbNet results
                        if self.config.debug {
                            debug!(
                                "No VerbNet results for '{}', using basic assignment",
                                word.lemma
                            );
                        }
                        event_assignments = self.extract_basic_assignments_for_verb(words, word);
                    }
                } else {
                    // Fall back to basic assignment on VerbNet error
                    if self.config.debug {
                        debug!(
                            "VerbNet error for '{}': {:?}, using basic assignment",
                            word.lemma, verbnet_results
                        );
                    }
                    event_assignments = self.extract_basic_assignments_for_verb(words, word);
                }

                if !event_assignments.is_empty() {
                    assignments.insert(event_id, event_assignments);
                }
            }
        }

        if self.config.debug {
            debug!(
                "Extracted {} VerbNet-based theta assignments",
                assignments.len()
            );
        }

        assignments
    }

    /// Extract basic theta role assignments from dependency structure (fallback)
    fn extract_basic_theta_assignments_from_words(
        &self,
        words: &[Word],
    ) -> HashMap<EventId, HashMap<ThetaRoleType, Participant>> {
        let mut assignments = HashMap::new();

        // Find verbs and create basic theta assignments from dependency structure
        for word in words {
            if word.upos == UPos::Verb {
                let event_id = EventId(word.id);
                let mut event_assignments = HashMap::new();

                // Look for subjects and objects in the dependency structure
                for candidate in words {
                    if candidate.head == Some(word.id) {
                        match candidate.deprel {
                            DepRel::Nsubj => {
                                let participant = Participant {
                                    word_id: candidate.id,
                                    expression: candidate.text.clone(),
                                    features: Default::default(),
                                    discourse_ref: None,
                                };
                                event_assignments.insert(ThetaRoleType::Agent, participant);
                            }
                            DepRel::Obj => {
                                let participant = Participant {
                                    word_id: candidate.id,
                                    expression: candidate.text.clone(),
                                    features: Default::default(),
                                    discourse_ref: None,
                                };
                                event_assignments.insert(ThetaRoleType::Patient, participant);
                            }
                            _ => {}
                        }
                    }
                }

                if !event_assignments.is_empty() {
                    assignments.insert(event_id, event_assignments);
                }
            }
        }

        if self.config.debug {
            debug!(
                "Extracted {} theta assignments from dependency structure",
                assignments.len()
            );
        }

        assignments
    }

    /// Build syntactic context for VerbNet lookup
    fn build_syntactic_context(
        &self,
        words: &[Word],
        verb: &Word,
    ) -> (String, Vec<(String, String)>) {
        let mut arguments = Vec::new();
        let mut dependency_pattern = String::new();

        // Find all arguments of this verb and build pattern
        for word in words {
            if word.head == Some(verb.id) {
                let relation_str = match word.deprel {
                    DepRel::Nsubj => "subj",
                    DepRel::Obj => "obj",
                    DepRel::Iobj => "iobj",
                    DepRel::Obl => "obl",
                    DepRel::Xcomp => "xcomp",
                    DepRel::Ccomp => "ccomp",
                    _ => continue,
                };

                arguments.push((relation_str.to_string(), word.lemma.clone()));
                if !dependency_pattern.is_empty() {
                    dependency_pattern.push(' ');
                }
                dependency_pattern.push_str(relation_str);
            }
        }

        (dependency_pattern, arguments)
    }

    /// Find syntactic argument for a theta role
    fn find_syntactic_argument(
        &self,
        words: &[Word],
        verb: &Word,
        theta_role: &ThetaRoleType,
    ) -> Option<Participant> {
        // Map theta roles to likely syntactic positions
        let target_deprels = match theta_role {
            ThetaRoleType::Agent => vec![DepRel::Nsubj],
            ThetaRoleType::Patient | ThetaRoleType::Theme => vec![DepRel::Obj, DepRel::NsubjPass],
            ThetaRoleType::Recipient => vec![DepRel::Iobj, DepRel::Nmod], // Add Nmod for passive constructions
            ThetaRoleType::Source => vec![DepRel::Obl],
            ThetaRoleType::Goal => vec![DepRel::Iobj, DepRel::Obl],
            ThetaRoleType::Location => vec![DepRel::Obl],
            ThetaRoleType::Instrument => vec![DepRel::Obl],
            ThetaRoleType::Experiencer => vec![DepRel::Nsubj, DepRel::Obj],
            _ => vec![DepRel::Obl],
        };

        // Find argument with matching dependency relation
        for target_deprel in target_deprels {
            for word in words {
                if word.head == Some(verb.id) && word.deprel == target_deprel {
                    return Some(Participant {
                        word_id: word.id,
                        expression: word.text.clone(),
                        features: Default::default(),
                        discourse_ref: None,
                    });
                }
            }
        }

        None
    }

    /// Extract basic theta assignments for a single verb (fallback)
    fn extract_basic_assignments_for_verb(
        &self,
        words: &[Word],
        verb: &Word,
    ) -> HashMap<ThetaRoleType, Participant> {
        let mut assignments = HashMap::new();

        // Basic heuristic assignment based on dependency relations
        for word in words {
            if word.head == Some(verb.id) {
                let (theta_role, should_include) = match word.deprel {
                    DepRel::Nsubj => (ThetaRoleType::Agent, true),
                    DepRel::Obj => (ThetaRoleType::Patient, true),
                    DepRel::Iobj => (ThetaRoleType::Recipient, true),
                    DepRel::NsubjPass => (ThetaRoleType::Theme, true), // Passive subject
                    _ => (ThetaRoleType::Agent, false),                // Skip other relations
                };

                if should_include {
                    let participant = Participant {
                        word_id: word.id,
                        expression: word.text.clone(),
                        features: Default::default(),
                        discourse_ref: None,
                    };
                    assignments.insert(theta_role, participant);
                }
            }
        }

        assignments
    }

    /// Get VerbNet predicate information for event creation
    fn get_verbnet_predicate_info(
        &mut self,
        lemma: &str,
        words: &[Word],
        verb: &Word,
    ) -> (Option<String>, Vec<String>) {
        if !self.config.enable_verbnet {
            return (None, Vec::new());
        }

        // Build syntactic context
        let syntactic_context = self.build_syntactic_context(words, verb);

        // Look up verb in VerbNet
        let (dependency_pattern, arguments) = syntactic_context;
        match self.verbnet_engine.lookup_with_context(
            lemma,
            &dependency_pattern,
            &arguments,
            "active",  // TODO: Detect voice from movement analysis
            "present", // TODO: Extract tense/aspect from morphology
        ) {
            Ok(results) => {
                if !results.theta_assignments.is_empty() {
                    // Extract semantic features from predicates
                    let semantic_features: Vec<String> = results
                        .semantic_predicates
                        .iter()
                        .map(|pred| format!("{:?}", pred.predicate_type))
                        .collect();
                    (Some(format!("verbnet-{}", lemma)), semantic_features)
                } else {
                    (None, Vec::new())
                }
            }
            Err(_) => (None, Vec::new()),
        }
    }

    /// Convert string features to SemanticFeature enums
    fn convert_string_features_to_semantic_features(
        &self,
        features: &[String],
    ) -> Vec<crate::events::SemanticFeature> {
        use crate::events::SemanticFeature;

        features
            .iter()
            .filter_map(|feature| {
                match feature.as_str() {
                    "Motion" => Some(SemanticFeature::Motion),
                    "Transfer" => Some(SemanticFeature::Transfer),
                    "Contact" => Some(SemanticFeature::Contact),
                    "Change" => Some(SemanticFeature::ChangeOfState),
                    _ => None, // Other types not mapped yet
                }
            })
            .collect()
    }

    /// Create event structures from words and theta assignments
    fn create_event_structures_from_words(
        &mut self,
        words: &[Word],
        theta_assignments: &HashMap<EventId, HashMap<ThetaRoleType, Participant>>,
        metrics: &mut Layer2Metrics,
    ) -> Result<Vec<Event>, Layer2Error> {
        let mut events = Vec::new();

        // 1. Detect movement in the sentence
        let movement_start = Instant::now();
        let movement_analysis = self.movement_detector.detect_movement(words);
        let _movement_time = movement_start.elapsed();

        if self.config.debug {
            debug!(
                "Movement analysis: detected {} movement types with confidence {:.3}",
                movement_analysis.movement_types.len(),
                movement_analysis.confidence
            );
            for movement_type in &movement_analysis.movement_types {
                debug!("  - {}", movement_type);
            }
        }

        // Create events for verbs
        for word in words {
            if word.upos == UPos::Verb {
                let event_id = EventId(word.id);

                // Create predicate with VerbNet information if available
                let (verbnet_class, semantic_features_strings) = if self.config.enable_verbnet {
                    self.get_verbnet_predicate_info(&word.lemma, words, word)
                } else {
                    (None, Vec::new())
                };

                // Convert string features to SemanticFeature enums
                let semantic_features =
                    self.convert_string_features_to_semantic_features(&semantic_features_strings);

                let predicate = Predicate {
                    lemma: word.lemma.clone(),
                    semantic_type: self.classify_predicate_type(&word.lemma),
                    verbnet_class,
                    features: semantic_features,
                };

                let mut event = Event::new(event_id, predicate);

                // 2. Add participants from theta assignments (potentially adjusted by movement)
                if let Some(assignments) = theta_assignments.get(&event_id) {
                    let adjusted_assignments = self.adjust_theta_assignments_for_movement(
                        assignments,
                        &movement_analysis,
                        words,
                        word,
                    );

                    for (role, participant) in &adjusted_assignments {
                        event.add_participant(*role, participant.clone());
                    }
                }

                // 3. Add movement chains to the event
                if !movement_analysis.movement_types.is_empty() {
                    let chains =
                        self.create_movement_chains_for_event(&movement_analysis, words, word);
                    for chain in chains {
                        event.add_movement_chain(chain);
                    }
                }

                // Add basic temporal information
                event.time = EventTime::Now; // Simplified

                // 4. Apply little v decomposition if enabled
                if self.config.enable_little_v_decomposition {
                    let decomposition_start = Instant::now();
                    if let Some(little_v_shell) = self.event_decomposer.decompose_event(&event) {
                        event.set_little_v(little_v_shell);
                        metrics.events_with_little_v += 1;

                        if self.config.debug {
                            debug!(
                                "Applied little v decomposition to event {}: {:?}",
                                event_id.0,
                                event.get_little_v().map(|lv| &lv.v_head)
                            );
                        }
                    }

                    if metrics.little_v_decomposition_time_us.is_none() {
                        metrics.little_v_decomposition_time_us = Some(0);
                    }
                    metrics.little_v_decomposition_time_us = Some(
                        metrics.little_v_decomposition_time_us.unwrap()
                            + decomposition_start.elapsed().as_micros() as u64,
                    );
                }

                if self.config.debug {
                    debug!(
                        "Created event {} for verb '{}' with {} movement chains",
                        event_id.0,
                        word.text,
                        event.get_movement_chains().len()
                    );
                }

                events.push(event);
            }
        }

        // Movement time is included in event creation time

        Ok(events)
    }

    /// Adjust theta assignments based on movement analysis
    fn adjust_theta_assignments_for_movement(
        &self,
        original_assignments: &HashMap<ThetaRoleType, Participant>,
        movement_analysis: &crate::syntax::MovementAnalysis,
        _words: &[Word],
        _verb_word: &Word,
    ) -> HashMap<ThetaRoleType, Participant> {
        let adjusted = original_assignments.clone();

        // Handle passive movement: Agent becomes optional/implicit, Patient becomes Subject
        if movement_analysis
            .movement_types
            .contains(&crate::syntax::MovementType::PassiveMovement)
        {
            if self.config.debug {
                debug!("Adjusting theta assignments for passive movement");
            }

            // In passive: "The ball was hit [by John]"
            // - Patient (ball) moves to subject position
            // - Agent (John) becomes optional oblique
            // No adjustment needed here as UDPipe already provides correct dependencies
            // The movement chain will track the syntactic displacement
        }

        // Handle raising movement: Subject of matrix clause is theta-marked in embedded clause
        if movement_analysis
            .movement_types
            .contains(&crate::syntax::MovementType::RaisingMovement)
        {
            if self.config.debug {
                debug!("Adjusting theta assignments for raising movement");
            }

            // In raising: "John seems to be happy"
            // - John is syntactic subject of "seems" but theta-marked by "be"
            // This requires more sophisticated analysis of embedded clauses
            // For now, keep original assignments
        }

        adjusted
    }

    /// Create movement chains for the given event
    fn create_movement_chains_for_event(
        &self,
        movement_analysis: &crate::syntax::MovementAnalysis,
        words: &[Word],
        _verb_word: &Word,
    ) -> Vec<MovementChain> {
        let mut chains = Vec::new();

        for movement_type in &movement_analysis.movement_types {
            match movement_type {
                crate::syntax::MovementType::PassiveMovement => {
                    if let Some(chain) =
                        self.create_passive_movement_chain(words, movement_analysis)
                    {
                        chains.push(chain);
                    }
                }
                crate::syntax::MovementType::WhMovement => {
                    if let Some(chain) = self.create_wh_movement_chain(words, movement_analysis) {
                        chains.push(chain);
                    }
                }
                crate::syntax::MovementType::RaisingMovement => {
                    if let Some(chain) =
                        self.create_raising_movement_chain(words, movement_analysis)
                    {
                        chains.push(chain);
                    }
                }
                _ => {
                    // Handle other movement types as needed
                    debug!(
                        "Movement type {:?} not yet implemented in chain creation",
                        movement_type
                    );
                }
            }
        }

        chains
    }

    /// Create a movement chain for passive constructions
    fn create_passive_movement_chain(
        &self,
        words: &[Word],
        _movement_analysis: &crate::syntax::MovementAnalysis,
    ) -> Option<MovementChain> {
        // Find the passive subject (moved element)
        let moved_element = words.iter().find(|w| w.deprel == DepRel::NsubjPass)?;

        if self.config.debug {
            debug!(
                "Creating passive movement chain for moved element: {}",
                moved_element.lemma
            );
        }

        // Create movement chain (simplified - in full implementation would track precise positions)
        Some(MovementChain {
            movement_type: EventMovementType::AMovement,
            moved_element: crate::events::ChainLink {
                word_id: moved_element.id,
                position: "Spec,TP".to_string(), // Subject position
                phonetically_realized: true,
                theta_role: Some(ThetaRoleType::Theme), // Typically Theme/Patient
                case: Some(crate::events::CaseType::Nominative),
            },
            intermediate_positions: Vec::new(),
            base_position: crate::events::ChainLink {
                word_id: 0,                          // Trace position (simplified)
                position: "VP-internal".to_string(), // Object position
                phonetically_realized: false,
                theta_role: Some(ThetaRoleType::Theme),
                case: Some(crate::events::CaseType::Accusative),
            },
            landing_site: crate::events::LandingSite {
                position: "Spec,TP".to_string(),
                driving_features: vec![crate::events::MovementFeature::EPP], // Extended Projection Principle
                is_intermediate: false,
            },
        })
    }

    /// Create a movement chain for wh-movement
    fn create_wh_movement_chain(
        &self,
        words: &[Word],
        movement_analysis: &crate::syntax::MovementAnalysis,
    ) -> Option<MovementChain> {
        // Find the wh-word
        let wh_word = movement_analysis.signals.wh_word.as_ref()?;
        let moved_element = words.iter().find(|w| w.lemma == *wh_word)?;

        if self.config.debug {
            debug!("Creating wh-movement chain for wh-word: {}", wh_word);
        }

        Some(MovementChain {
            movement_type: EventMovementType::ABarMovement,
            moved_element: crate::events::ChainLink {
                word_id: moved_element.id,
                position: "Spec,CP".to_string(), // Spec of CP
                phonetically_realized: true,
                theta_role: self.infer_wh_theta_role(wh_word),
                case: None, // Wh-words don't get structural case
            },
            intermediate_positions: Vec::new(),
            base_position: crate::events::ChainLink {
                word_id: 0,                          // Gap position
                position: "VP-internal".to_string(), // Simplified
                phonetically_realized: false,
                theta_role: self.infer_wh_theta_role(wh_word),
                case: Some(crate::events::CaseType::Accusative), // Gap would have gotten case
            },
            landing_site: crate::events::LandingSite {
                position: "Spec,CP".to_string(),
                driving_features: vec![crate::events::MovementFeature::Wh],
                is_intermediate: false,
            },
        })
    }

    /// Create a movement chain for raising constructions
    fn create_raising_movement_chain(
        &self,
        words: &[Word],
        _movement_analysis: &crate::syntax::MovementAnalysis,
    ) -> Option<MovementChain> {
        // Find the raising verb and its subject
        let raising_verbs = ["seem", "appears", "happen", "tend", "prove", "turn"];
        let raising_verb = words
            .iter()
            .find(|w| raising_verbs.contains(&w.lemma.as_str()))?;

        let raised_subject = words
            .iter()
            .find(|w| w.head == Some(raising_verb.id) && w.deprel == DepRel::Nsubj)?;

        if self.config.debug {
            debug!(
                "Creating raising movement chain for raised subject: {}",
                raised_subject.lemma
            );
        }

        Some(MovementChain {
            movement_type: EventMovementType::AMovement,
            moved_element: crate::events::ChainLink {
                word_id: raised_subject.id,
                position: "Spec,TP".to_string(), // Matrix subject position
                phonetically_realized: true,
                theta_role: None, // Subject of raising verb has no theta role from matrix verb
                case: Some(crate::events::CaseType::Nominative),
            },
            intermediate_positions: Vec::new(),
            base_position: crate::events::ChainLink {
                word_id: 0,                               // Position in embedded clause
                position: "Spec,TP-embedded".to_string(), // Embedded subject position
                phonetically_realized: false,
                theta_role: Some(ThetaRoleType::Theme), // Gets theta role from embedded predicate
                case: None,                             // No case in embedded clause
            },
            landing_site: crate::events::LandingSite {
                position: "Spec,TP".to_string(),
                driving_features: vec![crate::events::MovementFeature::EPP],
                is_intermediate: false,
            },
        })
    }

    /// Infer theta role for wh-words
    fn infer_wh_theta_role(&self, wh_word: &str) -> Option<ThetaRoleType> {
        match wh_word.to_lowercase().as_str() {
            "what" => Some(ThetaRoleType::Theme),
            "who" | "whom" => Some(ThetaRoleType::Agent), // Could be Theme in passive
            "where" => Some(ThetaRoleType::Location),
            "when" => Some(ThetaRoleType::Time),
            "why" => Some(ThetaRoleType::Cause),
            "how" => Some(ThetaRoleType::Instrument), // Simplified
            _ => None,
        }
    }

    /// Classify predicate type based on verb lemma (simplified)
    fn classify_predicate_type(&self, lemma: &str) -> PredicateType {
        // Simple classification based on common verbs
        match lemma {
            "run" | "walk" | "move" | "go" | "come" => PredicateType::Action,
            "know" | "love" | "exist" | "be" => PredicateType::State,
            "arrive" | "die" | "notice" | "recognize" => PredicateType::Achievement,
            "build" | "write" | "destroy" | "create" => PredicateType::Accomplishment,
            _ => PredicateType::Action, // Default
        }
    }

    /// Log performance metrics
    fn log_performance_metrics(&self, metrics: &Layer2Metrics) {
        let total_us = metrics.total_time_us;
        let theta_us = metrics.theta_assignment_time_us;
        let event_us = metrics.event_creation_time_us;

        info!("Layer2 Semantic Analysis Metrics:");
        info!("  Total time: {}μs", total_us);
        info!(
            "  Theta assignment: {}μs ({:.1}%)",
            theta_us,
            theta_us as f64 / total_us as f64 * 100.0
        );
        info!(
            "  Event creation: {}μs ({:.1}%)",
            event_us,
            event_us as f64 / total_us as f64 * 100.0
        );

        if let Some(little_v_us) = metrics.little_v_decomposition_time_us {
            info!(
                "  Little v decomposition: {}μs ({:.1}%)",
                little_v_us,
                little_v_us as f64 / total_us as f64 * 100.0
            );
        }

        info!("  Words processed: {}", metrics.words_processed);
        info!("  Events created: {}", metrics.events_created);
        if metrics.events_with_little_v > 0 {
            info!(
                "  Events with little v: {} ({:.1}%)",
                metrics.events_with_little_v,
                metrics.events_with_little_v as f64 / metrics.events_created as f64 * 100.0
            );
        }
        info!(
            "  Processing rate: {:.1} words/sec",
            metrics.words_per_second()
        );
    }

    /// Calculate overall confidence score
    fn calculate_overall_confidence(&self, events: &[Event]) -> f64 {
        // Simplified confidence calculation
        if events.is_empty() {
            0.5 // Neutral confidence for no events
        } else {
            0.8 // Higher confidence when events are created
        }
    }
}

impl Default for Layer2Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create word from basic information
pub fn create_word_from_parse(id: usize, text: &str, lemma: &str, upos: UPos) -> Word {
    Word {
        id,
        text: text.to_string(),
        lemma: lemma.to_string(),
        upos,
        xpos: None,
        feats: MorphFeatures::default(),
        head: Some(0),
        deprel: DepRel::Root,
        deps: None,
        misc: None,
        start: 0,
        end: text.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer2_basic_analysis() {
        let mut analyzer = Layer2Analyzer::new();

        let words = vec![
            create_word_from_parse(1, "John", "John", UPos::Propn),
            create_word_from_parse(2, "runs", "run", UPos::Verb),
            create_word_from_parse(3, "quickly", "quickly", UPos::Adv),
        ];

        let result = analyzer.analyze(words);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.words.len(), 3);
        assert_eq!(analysis.events.len(), 1); // One verb = one event
        assert!(analysis.confidence > 0.0);
    }

    #[test]
    fn test_layer2_performance_monitoring() {
        let config = Layer2Config {
            enable_logging: true,
            debug: true,
            ..Default::default()
        };

        let mut analyzer = Layer2Analyzer::with_config(config);

        let words = vec![
            create_word_from_parse(1, "She", "she", UPos::Pron),
            create_word_from_parse(2, "walks", "walk", UPos::Verb),
        ];

        let result = analyzer.analyze(words);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.metrics.total_time_us > 0);
        assert_eq!(analysis.metrics.words_processed, 2);
        assert_eq!(analysis.metrics.events_created, 1);
    }

    #[test]
    fn test_predicate_classification() {
        let analyzer = Layer2Analyzer::new();

        assert_eq!(
            analyzer.classify_predicate_type("run"),
            PredicateType::Action
        );
        assert_eq!(
            analyzer.classify_predicate_type("know"),
            PredicateType::State
        );
        assert_eq!(
            analyzer.classify_predicate_type("arrive"),
            PredicateType::Achievement
        );
        assert_eq!(
            analyzer.classify_predicate_type("build"),
            PredicateType::Accomplishment
        );
        assert_eq!(
            analyzer.classify_predicate_type("unknown"),
            PredicateType::Action
        );
    }

    #[test]
    fn test_word_creation() {
        let word = create_word_from_parse(1, "test", "test", UPos::Verb);

        assert_eq!(word.text, "test");
        assert_eq!(word.lemma, "test");
        assert_eq!(word.upos, UPos::Verb);
        assert_eq!(word.id, 1);
    }

    #[test]
    fn test_little_v_decomposition_integration() {
        let config = Layer2Config {
            enable_little_v_decomposition: true,
            debug: true,
            enable_logging: true,
            ..Default::default()
        };

        let mut analyzer = Layer2Analyzer::with_config(config);

        // Test "John broke the vase" - should get CAUSE decomposition
        let words = vec![
            create_word_from_parse(1, "John", "John", UPos::Propn),
            create_word_from_parse(2, "broke", "break", UPos::Verb),
            create_word_from_parse(3, "the", "the", UPos::Det),
            create_word_from_parse(4, "vase", "vase", UPos::Noun),
        ];

        // Set up dependency relations
        let mut words = words;
        words[0].head = Some(2); // John -> broke
        words[0].deprel = DepRel::Nsubj;
        words[2].head = Some(4); // the -> vase
        words[2].deprel = DepRel::Det;
        words[3].head = Some(2); // vase -> broke
        words[3].deprel = DepRel::Obj;

        let result = analyzer.analyze(words);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.events.len(), 1);

        let event = &analysis.events[0];
        assert!(event.get_little_v().is_some());

        // Should detect causative decomposition for transitive "break"
        let little_v = event.get_little_v().unwrap();
        assert_eq!(little_v.v_head, crate::syntax::LittleVType::Cause);
        assert!(little_v.external_argument.is_some());
        assert_eq!(
            little_v.external_argument.as_ref().unwrap().expression,
            "John"
        );

        // Check metrics
        assert_eq!(analysis.metrics.events_with_little_v, 1);
        assert!(analysis.metrics.little_v_decomposition_time_us.is_some());
    }

    #[test]
    fn test_dependency_injection_pattern() {
        use crate::syntax::{EventDecomposer, MovementDetector};

        let config = Layer2Config::default();
        let movement_detector = MovementDetector::new();
        let event_decomposer = EventDecomposer::new();

        // Test dependency injection constructor
        let verbnet_engine = crate::verbnet::VerbNetEngine::new_with_test_data();
        let analyzer = Layer2Analyzer::with_dependencies(
            config,
            movement_detector,
            event_decomposer,
            verbnet_engine,
        );

        // Verify analyzer was created successfully (constructor worked without panic)
        assert!(analyzer.config.enable_verbnet);

        // Test that we can disable little v decomposition
        let config_no_decomp = Layer2Config {
            enable_little_v_decomposition: false,
            ..Default::default()
        };

        let mut analyzer_no_decomp = Layer2Analyzer::with_config(config_no_decomp);

        let words = vec![
            create_word_from_parse(1, "John", "John", UPos::Propn),
            create_word_from_parse(2, "runs", "run", UPos::Verb),
        ];

        let result = analyzer_no_decomp.analyze(words);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.events.len(), 1);
        assert!(analysis.events[0].get_little_v().is_none());
        assert_eq!(analysis.metrics.events_with_little_v, 0);
        assert!(analysis.metrics.little_v_decomposition_time_us.is_none());
    }
}
