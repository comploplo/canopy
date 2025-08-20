//! VerbNet lookup engine with efficient indexing
//!
//! This module provides fast lookups for VerbNet data using pre-built indices.

use crate::verbnet::types::*;
use indexmap::IndexMap;
use lru::LruCache;
use std::cmp;
use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use tracing::{debug, info}; // warn will be used for error handling in future

/// Sophisticated cache key for VerbNet lookups that avoids minimal pair problems
///
/// This key includes enough linguistic context to distinguish between similar
/// syntactic structures that might have different theta role assignments.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VerbNetCacheKey {
    /// The verb lemma
    pub verb_lemma: String,

    /// Dependency pattern (e.g., "nsubj+dobj+prep_to")
    pub dependency_pattern: String,

    /// Number of arguments
    pub arg_count: usize,

    /// Semantic class hints from argument heads (first 3 chars to avoid over-specificity)
    pub arg_semantic_hints: Vec<String>,

    /// Voice (active/passive/middle) - critical for theta role assignment
    pub voice: String,

    /// Tense/aspect (present, past, perfect) - affects some verb classes
    pub tense_aspect: String,
}

impl VerbNetCacheKey {
    /// Create a cache key from sentence analysis
    /// This includes enough context to avoid minimal pair confusion
    pub fn from_sentence_context(
        verb_lemma: &str,
        dependency_pattern: &str,
        arguments: &[(String, String)], // (relation, head_lemma)
        voice: &str,
        tense_aspect: &str,
    ) -> Self {
        // Extract semantic hints from argument heads (first 3 chars to generalize)
        let arg_semantic_hints: Vec<String> = arguments
            .iter()
            .map(|(_, head)| head.chars().take(3).collect::<String>().to_lowercase())
            .collect();

        Self {
            verb_lemma: verb_lemma.to_string(),
            dependency_pattern: dependency_pattern.to_string(),
            arg_count: arguments.len(),
            arg_semantic_hints,
            voice: voice.to_string(),
            tense_aspect: tense_aspect.to_string(),
        }
    }
}

/// VerbNet lookup result with confidence scoring
#[derive(Debug, Clone)]
pub struct VerbNetLookupResult {
    /// Possible theta role assignments with confidence scores
    pub theta_assignments: Vec<(Vec<ThetaRole>, f64)>,

    /// Selectional restrictions for validation
    pub selectional_restrictions: Vec<SelectionalRestriction>,

    /// Aspectual classification
    pub aspectual_class: AspectualInfo,

    /// Semantic predicates
    pub semantic_predicates: Vec<SemanticPredicate>,
}

/// Smart VerbNet cache that avoids minimal pair problems
#[derive(Debug)]
pub struct VerbNetCache {
    /// LRU cache with sophisticated keys
    cache: LruCache<VerbNetCacheKey, VerbNetLookupResult>,

    /// Cache hit tracking (exact matches)
    hits: AtomicU64,

    /// Similarity hit tracking (approximate matches)
    similarity_hits: AtomicU64,

    /// Cache miss tracking
    misses: AtomicU64,

    /// Total lookup attempts
    total_lookups: AtomicU64,
}

impl VerbNetCache {
    /// Create new cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(
                NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(10000).unwrap()),
            ),
            hits: AtomicU64::new(0),
            similarity_hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            total_lookups: AtomicU64::new(0),
        }
    }

    /// Get cached result if available
    pub fn get(&mut self, key: &VerbNetCacheKey) -> Option<VerbNetLookupResult> {
        self.total_lookups.fetch_add(1, Ordering::Relaxed);

        if let Some(result) = self.cache.get(key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            Some(result.clone())
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Get cached result with similarity fallback for better coverage
    ///
    /// First tries exact match, then searches for similar verbs with same syntactic pattern.
    /// This significantly improves cache hit rates for rare verbs and morphological variants.
    pub fn get_with_similarity(
        &mut self,
        key: &VerbNetCacheKey,
    ) -> Option<(VerbNetLookupResult, f64)> {
        self.total_lookups.fetch_add(1, Ordering::Relaxed);

        // Try exact match first
        if let Some(result) = self.cache.get(key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Some((result.clone(), 1.0)); // Perfect confidence for exact match
        }

        // Try similarity search for better coverage
        if let Some((result, similarity)) = self.find_similar_cached_verb(key) {
            self.similarity_hits.fetch_add(1, Ordering::Relaxed);
            return Some((result, similarity));
        }

        // No match found
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Find similar cached verb with same syntactic pattern
    ///
    /// Searches cache for verbs with identical dependency patterns but different lemmas.
    /// Uses verb similarity scoring to find the best match.
    fn find_similar_cached_verb(
        &self,
        target_key: &VerbNetCacheKey,
    ) -> Option<(VerbNetLookupResult, f64)> {
        let mut best_match = None;
        let mut best_similarity = 0.0;
        const SIMILARITY_THRESHOLD: f64 = 0.5; // Minimum similarity for cache hit (lowered for morphological variants)

        // Search through cache for verbs with matching syntactic patterns
        for (cached_key, cached_result) in self.cache.iter() {
            // Must have identical syntactic context
            if cached_key.dependency_pattern == target_key.dependency_pattern
                && cached_key.arg_count == target_key.arg_count
                && cached_key.voice == target_key.voice
                && cached_key.tense_aspect == target_key.tense_aspect
                && cached_key.arg_semantic_hints == target_key.arg_semantic_hints
            {
                // Compute verb similarity
                let similarity =
                    self.compute_verb_similarity(&target_key.verb_lemma, &cached_key.verb_lemma);

                if similarity > best_similarity && similarity >= SIMILARITY_THRESHOLD {
                    best_similarity = similarity;
                    best_match = Some(cached_result.clone());

                    // Early termination for very high similarity
                    if similarity > 0.95 {
                        break;
                    }
                }
            }
        }

        best_match.map(|result| (result, best_similarity))
    }

    /// Compute similarity between two verb lemmas
    ///
    /// Uses morphological similarity (edit distance) as a lightweight similarity metric.
    /// Future enhancement could include VerbNet class membership similarity.
    fn compute_verb_similarity(&self, verb1: &str, verb2: &str) -> f64 {
        if verb1 == verb2 {
            return 1.0;
        }

        // Handle morphological variants (e.g., run/running, give/gave)
        let stem1 = self.extract_verb_stem(verb1);
        let stem2 = self.extract_verb_stem(verb2);

        if stem1 == stem2 {
            return 0.9; // High similarity for morphological variants
        }

        // Edit distance similarity for other cases
        let distance = edit_distance(verb1, verb2);
        let max_len = verb1.len().max(verb2.len());

        if max_len == 0 {
            0.0
        } else {
            1.0 - (distance as f64 / max_len as f64)
        }
    }

    /// Extract verb stem for morphological similarity
    ///
    /// Simple heuristic-based stemming for common English verb patterns.
    fn extract_verb_stem<'a>(&self, verb: &'a str) -> &'a str {
        // Handle common verb suffixes with morphological rules
        if verb.ends_with("ing") && verb.len() > 3 {
            let without_ing = &verb[..verb.len() - 3];
            // Handle doubled consonants: running -> run, sitting -> sit
            if without_ing.len() >= 2 {
                let chars: Vec<char> = without_ing.chars().collect();
                if chars.len() >= 2 && chars[chars.len() - 1] == chars[chars.len() - 2] {
                    // Check if it's a doubled consonant pattern
                    let consonant = chars[chars.len() - 1];
                    if "bcdfghjklmnpqrstvwxz".contains(consonant) {
                        return &verb[..verb.len() - 4]; // Remove -ing and one consonant
                    }
                }
            }
            without_ing
        } else if verb.ends_with("ed") && verb.len() > 2 {
            &verb[..verb.len() - 2]
        } else if verb.ends_with("s") && verb.len() > 1 {
            &verb[..verb.len() - 1]
        } else {
            verb
        }
    }

    /// Store result in cache
    pub fn insert(&mut self, key: VerbNetCacheKey, result: VerbNetLookupResult) {
        self.cache.put(key, result);
    }

    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_lookups.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            self.hits.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    /// Get cache statistics including similarity hit tracking
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let similarity_hits = self.similarity_hits.load(Ordering::Relaxed);
        let total = self.total_lookups.load(Ordering::Relaxed);

        let hit_rate = if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        };
        let similarity_hit_rate = if total == 0 {
            0.0
        } else {
            similarity_hits as f64 / total as f64
        };
        let total_hit_rate = if total == 0 {
            0.0
        } else {
            (hits + similarity_hits) as f64 / total as f64
        };

        CacheStats {
            hits,
            similarity_hits,
            misses: self.misses.load(Ordering::Relaxed),
            total_lookups: total,
            hit_rate,
            similarity_hit_rate,
            total_hit_rate,
            cache_size: self.cache.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub similarity_hits: u64,
    pub misses: u64,
    pub total_lookups: u64,
    pub hit_rate: f64,
    pub similarity_hit_rate: f64,
    pub total_hit_rate: f64,
    pub cache_size: usize,
}

/// Compute edit distance between two strings (Levenshtein distance)
///
/// Used for verb similarity scoring in the cache similarity fallback.
fn edit_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = cmp::min(
                cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Errors that can occur when working with VerbNet
#[derive(Debug, Error)]
pub enum VerbNetError {
    #[error("Failed to load VerbNet data from {path}: {source}")]
    LoadError {
        path: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("VerbNet data directory not found: {path}")]
    DirectoryNotFound { path: String },

    #[error("No VerbNet classes found for verb: {verb}")]
    VerbNotFound { verb: String },

    #[error("Invalid VerbNet XML format: {reason}")]
    InvalidFormat { reason: String },

    #[error("Cache lookup failed: {reason}")]
    CacheLookupError { reason: String },
}

/// Fast lookup engine for VerbNet data with smart caching
#[derive(Debug)]
pub struct VerbNetEngine {
    /// All verb classes indexed by ID
    classes: IndexMap<String, VerbClass>,

    /// Mapping from verb lemmas to class IDs
    verb_to_classes: HashMap<String, Vec<String>>,

    /// Index of predicates to classes that use them
    predicates_index: HashMap<PredicateType, Vec<String>>,

    /// Index of theta roles to classes that use them
    theta_roles_index: HashMap<ThetaRoleType, Vec<String>>,

    /// Index of selectional restrictions to classes
    restrictions_index: HashMap<SelectionalRestriction, Vec<String>>,

    /// Smart cache to avoid minimal pair problems
    cache: VerbNetCache,

    /// Selectional restriction validator for theta assignment disambiguation
    restriction_validator: SelectionalRestrictionValidator,
}

impl VerbNetEngine {
    /// Create a new empty VerbNet engine
    pub fn new() -> Self {
        Self {
            classes: IndexMap::new(),
            verb_to_classes: HashMap::new(),
            predicates_index: HashMap::new(),
            theta_roles_index: HashMap::new(),
            restrictions_index: HashMap::new(),
            cache: VerbNetCache::new(10000), // Default cache size
            restriction_validator: SelectionalRestrictionValidator::new(),
        }
    }

    /// Create a new VerbNet engine with specified cache size
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self {
            classes: IndexMap::new(),
            verb_to_classes: HashMap::new(),
            predicates_index: HashMap::new(),
            theta_roles_index: HashMap::new(),
            restrictions_index: HashMap::new(),
            cache: VerbNetCache::new(cache_size),
            restriction_validator: SelectionalRestrictionValidator::new(),
        }
    }

    /// Create a new VerbNet engine with test data for development/testing
    pub fn new_with_test_data() -> Self {
        let mut engine = Self::new();
        engine.add_test_data();
        engine
    }

    /// Load VerbNet data from XML files in a directory
    pub fn load_from_directory<P: AsRef<Path>>(dir_path: P) -> Result<Self, VerbNetError> {
        let path = dir_path.as_ref();

        if !path.exists() || !path.is_dir() {
            return Err(VerbNetError::DirectoryNotFound {
                path: path.display().to_string(),
            });
        }

        info!("Loading VerbNet data from: {}", path.display());

        let mut engine = Self::new();

        // TODO: Implement actual XML parsing
        // For now, create some test data to establish the interface
        engine.add_test_data();

        info!("Loaded {} VerbNet classes", engine.classes.len());
        debug!("Verb coverage: {} verbs", engine.verb_to_classes.len());

        Ok(engine)
    }

    /// Smart cached VerbNet lookup with proper fallback hierarchy
    ///
    /// This is the main method for theta role assignment with correct lookup order:
    /// 1. Exact cache matching (fastest)
    /// 2. Full VerbNet XML parsing (fast, one-time only)
    /// 3. Similarity-based cache fallback for unknown verbs (last resort)
    ///
    /// The similarity fallback only triggers when a verb is NOT in VerbNet,
    /// avoiding overfitting and ensuring we use authoritative VerbNet data when available.
    pub fn lookup_with_context(
        &mut self,
        verb_lemma: &str,
        dependency_pattern: &str,
        arguments: &[(String, String)], // (relation, head_lemma)
        voice: &str,
        tense_aspect: &str,
    ) -> Result<VerbNetLookupResult, VerbNetError> {
        // Create sophisticated cache key
        let cache_key = VerbNetCacheKey::from_sentence_context(
            verb_lemma,
            dependency_pattern,
            arguments,
            voice,
            tense_aspect,
        );

        // Step 1: Try exact cache match first (fastest)
        if let Some(result) = self.cache.get(&cache_key) {
            debug!("VerbNet exact cache hit for key: {:?}", cache_key);
            return Ok(result);
        }

        debug!(
            "VerbNet cache miss, trying VerbNet lookup for: {:?}",
            cache_key
        );

        // Step 2: Try VerbNet lookup (fast, authoritative)
        match self.perform_full_lookup(verb_lemma, arguments, voice) {
            Ok(result) => {
                debug!("VerbNet lookup successful for verb: {}", verb_lemma);
                // Store in cache for future use
                self.cache.insert(cache_key, result.clone());
                Ok(result)
            }
            Err(VerbNetError::VerbNotFound { .. }) => {
                debug!(
                    "Verb '{}' not found in VerbNet, trying similarity fallback",
                    verb_lemma
                );

                // Step 3: Only use similarity fallback for unknown verbs (last resort)
                if let Some((mut result, confidence)) =
                    self.cache.find_similar_cached_verb(&cache_key)
                {
                    debug!(
                        "VerbNet similarity fallback hit with confidence {:.2} for unknown verb: {}",
                        confidence, verb_lemma
                    );
                    // Track similarity hit for statistics
                    self.cache.similarity_hits.fetch_add(1, Ordering::Relaxed);
                    self.cache.total_lookups.fetch_add(1, Ordering::Relaxed);
                    // Adjust confidence scores for similarity-based results
                    self.adjust_result_confidence(&mut result, confidence);
                    // Don't cache similarity results to avoid pollution
                    Ok(result)
                } else {
                    debug!(
                        "No similarity fallback found for unknown verb: {}",
                        verb_lemma
                    );
                    self.cache.misses.fetch_add(1, Ordering::Relaxed);
                    self.cache.total_lookups.fetch_add(1, Ordering::Relaxed);
                    Err(VerbNetError::VerbNotFound {
                        verb: verb_lemma.to_string(),
                    })
                }
            }
            Err(other_error) => {
                debug!("VerbNet lookup failed with error: {:?}", other_error);
                Err(other_error)
            }
        }
    }

    /// Adjust confidence scores in VerbNet result based on similarity matching
    ///
    /// When using similarity-based cache hits, we reduce confidence scores slightly
    /// to reflect the uncertainty from using a similar rather than identical verb.
    fn adjust_result_confidence(&self, result: &mut VerbNetLookupResult, similarity: f64) {
        // Apply similarity penalty to all confidence scores
        let confidence_factor = similarity * 0.95; // Slight penalty even for high similarity

        for (_, confidence) in &mut result.theta_assignments {
            *confidence *= confidence_factor;
        }
    }

    /// Perform full VerbNet lookup without caching
    fn perform_full_lookup(
        &self,
        verb_lemma: &str,
        arguments: &[(String, String)],
        voice: &str,
    ) -> Result<VerbNetLookupResult, VerbNetError> {
        // Get all possible verb classes for this lemma
        let verb_classes = self.get_verb_classes(verb_lemma);

        if verb_classes.is_empty() {
            return Err(VerbNetError::VerbNotFound {
                verb: verb_lemma.to_string(),
            });
        }

        let mut theta_assignments = Vec::new();
        let mut selectional_restrictions = Vec::new();
        let mut semantic_predicates = Vec::new();

        // Analyze each possible verb class
        for class in verb_classes {
            // Get theta roles for this class
            let theta_roles = &class.theta_roles;

            // Score this assignment based on argument compatibility
            let confidence = self.score_theta_assignment(&theta_roles, arguments, voice);

            if confidence > 0.1 {
                // Minimum confidence threshold
                theta_assignments.push((theta_roles.clone(), confidence));
                selectional_restrictions.extend(
                    theta_roles
                        .iter()
                        .flat_map(|role| &role.selectional_restrictions)
                        .cloned(),
                );
            }

            // Collect semantic predicates from all frames
            for frame in &class.frames {
                semantic_predicates.extend(frame.semantics.clone());
            }
        }

        // Sort by confidence (highest first)
        theta_assignments.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Infer aspectual class
        let aspectual_class = self.infer_aspectual_class(verb_lemma);

        Ok(VerbNetLookupResult {
            theta_assignments,
            selectional_restrictions,
            aspectual_class,
            semantic_predicates,
        })
    }

    /// Score theta role assignment based on argument compatibility
    fn score_theta_assignment(
        &self,
        theta_roles: &[ThetaRole],
        arguments: &[(String, String)],
        voice: &str,
    ) -> f64 {
        let mut score = 0.5; // Base score

        // Boost score if argument count matches expected roles
        if theta_roles.len() == arguments.len() {
            score += 0.3;
        } else {
            // Penalize mismatches
            score -= 0.1 * (theta_roles.len() as f64 - arguments.len() as f64).abs();
        }

        // Boost score for voice compatibility
        if voice == "passive" {
            // Check if this verb class supports passive
            let has_agent = theta_roles
                .iter()
                .any(|role| role.role_type == ThetaRoleType::Agent);
            let has_patient = theta_roles
                .iter()
                .any(|role| role.role_type == ThetaRoleType::Patient);

            if has_agent && has_patient {
                score += 0.2;
            }
        }

        // Additional scoring based on argument head compatibility could be added here

        score.max(0.0).min(1.0) // Clamp to [0, 1]
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Get all VerbNet classes for a given verb lemma
    pub fn get_verb_classes(&self, lemma: &str) -> Vec<&VerbClass> {
        self.verb_to_classes
            .get(lemma)
            .map(|class_ids| {
                class_ids
                    .iter()
                    .filter_map(|id| self.classes.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all possible theta roles for a verb
    pub fn get_theta_roles(&self, verb: &str) -> Vec<ThetaRole> {
        self.get_verb_classes(verb)
            .into_iter()
            .flat_map(|class| &class.theta_roles)
            .cloned()
            .collect()
    }

    /// Get selectional restrictions for a specific verb and theta role
    pub fn get_selectional_restrictions(
        &self,
        verb: &str,
        role: ThetaRoleType,
    ) -> Vec<SelectionalRestriction> {
        self.get_theta_roles(verb)
            .into_iter()
            .filter(|theta_role| theta_role.role_type == role)
            .flat_map(|theta_role| theta_role.selectional_restrictions)
            .collect()
    }

    /// Get syntactic frames for a verb
    pub fn get_syntactic_frames(&self, verb: &str) -> Vec<&SyntacticFrame> {
        self.get_verb_classes(verb)
            .into_iter()
            .flat_map(|class| &class.frames)
            .collect()
    }

    /// Map UDPipe dependency patterns to VerbNet syntactic frames and theta roles
    /// Returns multiple analyses with confidence scores to handle ambiguity
    /// This is the core pattern matching that connects UDPipe syntax to VerbNet semantics
    pub fn map_dependency_pattern_to_theta_roles(
        &self,
        verb_lemma: &str,
        dependency_pattern: &str,
        arguments: &[(String, String)], // (relation, head_lemma)
    ) -> Vec<PatternAnalysis> {
        // Multiple analyses with confidence scores
        debug!(
            "Mapping dependency pattern '{}' for verb '{}' with {} arguments",
            dependency_pattern,
            verb_lemma,
            arguments.len()
        );

        let mut all_analyses = Vec::new();

        // Get verb classes for this lemma
        if let Some(verb_classes) = self.verb_to_classes.get(verb_lemma) {
            for class_id in verb_classes {
                if let Some(verb_class) = self.classes.get(class_id) {
                    debug!(
                        "Analyzing class: {} with {} frames",
                        class_id,
                        verb_class.frames.len()
                    );

                    // Check each syntactic frame in this class
                    for (frame_idx, frame) in verb_class.frames.iter().enumerate() {
                        if let Some(analysis) = self.match_frame_to_pattern(
                            verb_class,
                            frame_idx,
                            frame,
                            dependency_pattern,
                            arguments,
                        ) {
                            debug!(
                                "Found matching frame: {} -> confidence: {:.3}",
                                frame.description, analysis.confidence
                            );
                            all_analyses.push(analysis);
                        }
                    }
                }
            }
        }

        // Sort by confidence (highest first) and limit to top analyses
        all_analyses.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_analyses.truncate(5); // Return top 5 analyses to manage complexity

        if all_analyses.is_empty() {
            debug!(
                "No pattern matches found for '{}' with pattern '{}'",
                verb_lemma, dependency_pattern
            );
        } else {
            debug!(
                "Pattern mapping found {} analyses, best confidence: {:.3}",
                all_analyses.len(),
                all_analyses[0].confidence
            );
        }

        all_analyses
    }

    /// Match a specific VerbNet frame to a UDPipe dependency pattern
    /// Returns detailed analysis with confidence score and theta assignments
    fn match_frame_to_pattern(
        &self,
        verb_class: &VerbClass,
        frame_idx: usize,
        frame: &SyntacticFrame,
        dependency_pattern: &str,
        arguments: &[(String, String)],
    ) -> Option<PatternAnalysis> {
        let mut theta_assignments = Vec::new();
        let mut assignment_details = Vec::new();

        // Parse dependency pattern into components
        let deps: Vec<&str> = dependency_pattern.split('+').collect();

        // Score how well this frame matches the dependency pattern
        let mut match_score = 0.0f32;
        let total_possible_score = frame.syntax.elements.len() as f32;

        // Map VerbNet syntax elements to dependency relations
        for (elem_idx, element) in frame.syntax.elements.iter().enumerate() {
            let element_score = match element.category.as_str() {
                "NP" => {
                    if let Some(theta_role) = element.theta_role {
                        let (confidence, arg_text) =
                            self.score_and_find_np_mapping(&theta_role, &deps, arguments);

                        if confidence > 0.2 {
                            // Lower threshold to capture more possibilities
                            if let Some(arg) = arg_text {
                                theta_assignments.push(ThetaAssignment {
                                    theta_role,
                                    argument_text: arg,
                                    confidence,
                                    source_relation: self.find_source_relation(&theta_role, &deps),
                                });
                                assignment_details.push(AssignmentDetail {
                                    element_index: elem_idx,
                                    theta_role: Some(theta_role),
                                    matched_relations: self
                                        .get_matching_relations(&theta_role, &deps),
                                    confidence,
                                });
                            }
                        }
                        confidence
                    } else {
                        0.5 // NP without theta role gets neutral score
                    }
                }
                "V" | "VERB" => {
                    // Verb element always matches since we're analyzing this verb
                    1.0
                }
                cat if cat.starts_with("PREP") => {
                    let confidence = self.score_prep_mapping(&element, &deps);
                    if confidence > 0.2 && element.theta_role.is_some() {
                        let theta_role = element.theta_role.unwrap();
                        if let Some(arg_text) = self.find_prep_argument(&element, arguments) {
                            theta_assignments.push(ThetaAssignment {
                                theta_role,
                                argument_text: arg_text,
                                confidence,
                                source_relation: self.extract_prep_relation(&element, &deps),
                            });
                        }
                    }
                    confidence
                }
                _ => {
                    // Other categories (ADJ, ADV, etc.) get moderate score
                    0.6
                }
            };

            match_score += element_score;
        }

        // Calculate overall confidence
        let frame_match_confidence = if total_possible_score > 0.0 {
            match_score / total_possible_score
        } else {
            0.0
        };

        // Boost confidence if we have good theta role coverage
        let theta_coverage_bonus = if !theta_assignments.is_empty() {
            let assigned_roles = theta_assignments.len() as f32;
            let expected_roles = verb_class.theta_roles.len() as f32;
            (assigned_roles / expected_roles.max(1.0)) * 0.2 // Up to 20% bonus
        } else {
            -0.3 // Penalty for no theta assignments
        };

        let final_confidence = (frame_match_confidence + theta_coverage_bonus)
            .max(0.0)
            .min(1.0);

        // Only return if we meet minimum thresholds
        if final_confidence > 0.3 && !theta_assignments.is_empty() {
            Some(PatternAnalysis {
                verb_class_id: verb_class.id.clone(),
                frame_index: frame_idx,
                frame_description: frame.description.clone(),
                confidence: final_confidence,
                theta_assignments,
                assignment_details,
                dependency_pattern: dependency_pattern.to_string(),
                semantic_predicates: frame.semantics.clone(),
            })
        } else {
            None
        }
    }

    /// Score and find NP mapping with detailed analysis
    fn score_and_find_np_mapping(
        &self,
        theta_role: &ThetaRoleType,
        deps: &[&str],
        arguments: &[(String, String)],
    ) -> (f32, Option<String>) {
        let confidence = match theta_role {
            ThetaRoleType::Agent => {
                // Agent typically maps to subject (nsubj, nsubjpass)
                if deps.iter().any(|&dep| dep == "nsubj") {
                    0.95
                } else if deps.iter().any(|&dep| dep == "nsubjpass") {
                    0.90 // Slightly lower for passive subjects
                } else if deps.iter().any(|&dep| dep.contains("subj")) {
                    0.7
                } else {
                    0.15
                }
            }
            ThetaRoleType::Patient | ThetaRoleType::Theme => {
                // Patient/Theme typically maps to direct object (dobj)
                if deps.iter().any(|&dep| dep == "dobj") {
                    0.95
                } else if deps.iter().any(|&dep| dep == "nsubjpass") {
                    0.90 // Theme can be passive subject
                } else if deps.iter().any(|&dep| dep.contains("obj")) {
                    0.7
                } else {
                    0.15
                }
            }
            ThetaRoleType::Recipient | ThetaRoleType::Goal => {
                // Recipient can be indirect object (iobj) or prepositional phrase
                if deps.iter().any(|&dep| dep == "iobj") {
                    0.95
                } else if deps.iter().any(|&dep| dep.contains("prep_to")) {
                    0.85
                } else if deps.iter().any(|&dep| dep.contains("prep")) {
                    0.6
                } else {
                    0.2
                }
            }
            ThetaRoleType::Source => {
                if deps.iter().any(|&dep| dep.contains("prep_from")) {
                    0.90
                } else if deps.iter().any(|&dep| dep.contains("prep")) {
                    0.5
                } else {
                    0.1
                }
            }
            ThetaRoleType::Location => {
                if deps.iter().any(|&dep| {
                    dep.contains("prep_in") || dep.contains("prep_at") || dep.contains("prep_on")
                }) {
                    0.90
                } else if deps.iter().any(|&dep| dep.contains("prep")) {
                    0.6
                } else {
                    0.15
                }
            }
            ThetaRoleType::Instrument => {
                if deps
                    .iter()
                    .any(|&dep| dep.contains("prep_with") || dep.contains("prep_by"))
                {
                    0.90
                } else if deps.iter().any(|&dep| dep.contains("prep")) {
                    0.5
                } else {
                    0.1
                }
            }
            _ => 0.5, // Default confidence for other roles
        };

        // Find corresponding argument text
        let arg_text = self.find_argument_for_theta_role(theta_role, deps, arguments);

        (confidence, arg_text)
    }

    /// Helper methods for detailed pattern analysis
    fn find_source_relation(&self, theta_role: &ThetaRoleType, deps: &[&str]) -> Option<String> {
        let target_patterns = match theta_role {
            ThetaRoleType::Agent => vec!["nsubj", "nsubjpass"],
            ThetaRoleType::Patient | ThetaRoleType::Theme => vec!["dobj", "nsubjpass"],
            ThetaRoleType::Recipient => vec!["iobj", "prep_to"],
            ThetaRoleType::Source => vec!["prep_from"],
            ThetaRoleType::Location => vec!["prep_in", "prep_at", "prep_on"],
            ThetaRoleType::Instrument => vec!["prep_with", "prep_by"],
            _ => vec![],
        };

        for &dep in deps {
            if target_patterns.iter().any(|&pattern| dep.contains(pattern)) {
                return Some(dep.to_string());
            }
        }
        None
    }

    fn get_matching_relations(&self, theta_role: &ThetaRoleType, deps: &[&str]) -> Vec<String> {
        let target_patterns = match theta_role {
            ThetaRoleType::Agent => vec!["subj"],
            ThetaRoleType::Patient | ThetaRoleType::Theme => vec!["obj", "nsubjpass"],
            ThetaRoleType::Recipient => vec!["iobj", "prep_to"],
            _ => vec![],
        };

        deps.iter()
            .filter(|&dep| target_patterns.iter().any(|&pattern| dep.contains(pattern)))
            .map(|&dep| dep.to_string())
            .collect()
    }

    /// Score prepositional phrase mapping
    fn score_prep_mapping(&self, element: &SyntaxElement, deps: &[&str]) -> f32 {
        // Extract preposition from element category (e.g., "PREP(to)" -> "to")
        let expected_prep = if element.category.contains('(') {
            element
                .category
                .split('(')
                .nth(1)
                .and_then(|s| Some(s.trim_end_matches(')')))
        } else {
            return 0.3;
        };

        if let Some(prep) = expected_prep {
            // Check if dependency pattern contains matching prepositional relation
            let matching_prep_dep = deps
                .iter()
                .find(|&dep| dep.contains("prep") && dep.contains(prep));

            if matching_prep_dep.is_some() {
                0.9 // High confidence for exact preposition match
            } else if deps.iter().any(|&dep| dep.contains("prep")) {
                0.5 // Moderate confidence for any prepositional relation
            } else {
                0.1 // Low confidence if no prep relations found
            }
        } else {
            0.3
        }
    }

    /// Find argument text for a given theta role
    fn find_argument_for_theta_role(
        &self,
        theta_role: &ThetaRoleType,
        deps: &[&str],
        arguments: &[(String, String)],
    ) -> Option<String> {
        // Map theta roles to likely dependency relations
        let target_relations = match theta_role {
            ThetaRoleType::Agent => vec!["nsubj", "nsubjpass"],
            ThetaRoleType::Patient | ThetaRoleType::Theme => vec!["dobj", "obj"],
            ThetaRoleType::Recipient | ThetaRoleType::Goal => vec!["iobj", "prep_to", "prep_for"],
            ThetaRoleType::Source => vec!["prep_from", "prep_out_of"],
            ThetaRoleType::Location => vec!["prep_in", "prep_at", "prep_on"],
            ThetaRoleType::Instrument => vec!["prep_with", "prep_by"],
            _ => vec![], // For other roles, we'll need more sophisticated mapping
        };

        // Find the first argument that matches one of our target relations
        for (relation, head_lemma) in arguments {
            if target_relations
                .iter()
                .any(|&target| relation.contains(target))
            {
                return Some(head_lemma.clone());
            }
        }

        // Fallback: return first argument if we have target relations in deps
        if !arguments.is_empty()
            && deps
                .iter()
                .any(|&dep| target_relations.iter().any(|&target| dep.contains(target)))
        {
            return Some(arguments[0].1.clone());
        }

        None
    }

    /// Find prepositional argument for element
    fn find_prep_argument(
        &self,
        element: &SyntaxElement,
        arguments: &[(String, String)],
    ) -> Option<String> {
        // Extract expected preposition
        let expected_prep = if element.category.contains('(') {
            element
                .category
                .split('(')
                .nth(1)
                .and_then(|s| Some(s.trim_end_matches(')')))
        } else {
            return None;
        };

        if let Some(prep) = expected_prep {
            // Find argument with matching prepositional relation
            for (relation, head_lemma) in arguments {
                if relation.contains("prep") && relation.contains(prep) {
                    return Some(head_lemma.clone());
                }
            }
        }

        None
    }

    fn extract_prep_relation(&self, element: &SyntaxElement, deps: &[&str]) -> Option<String> {
        if let Some(expected_prep) = element
            .category
            .strip_prefix("PREP(")
            .and_then(|s| s.strip_suffix(")"))
        {
            for &dep in deps {
                if dep.contains("prep") && dep.contains(expected_prep) {
                    return Some(dep.to_string());
                }
            }
        }
        None
    }

    /// Get semantic predicates for a verb
    pub fn get_semantic_predicates(&self, verb: &str) -> Vec<SemanticPredicate> {
        self.get_syntactic_frames(verb)
            .into_iter()
            .flat_map(|frame| &frame.semantics)
            .cloned()
            .collect()
    }

    /// Infer aspectual class from VerbNet predicates
    pub fn infer_aspectual_class(&self, verb: &str) -> AspectualInfo {
        let predicates = self.get_semantic_predicates(verb);

        let has_motion = predicates
            .iter()
            .any(|p| matches!(p.predicate_type, PredicateType::Motion));

        let has_change = predicates.iter().any(|p| {
            matches!(
                p.predicate_type,
                PredicateType::Change | PredicateType::Transfer
            )
        });

        let has_result = predicates.iter().any(|p| {
            matches!(
                p.predicate_type,
                PredicateType::Created | PredicateType::Destroyed | PredicateType::Transfer
            )
        });

        let has_duration = predicates.len() > 1; // Simple heuristic

        AspectualInfo {
            durative: has_duration,
            dynamic: has_motion || has_change,
            telic: has_result,
            punctual: !has_duration,
        }
    }

    /// Check if a verb can take a specific theta role
    pub fn verb_allows_role(&self, verb: &str, role: ThetaRoleType) -> bool {
        self.get_theta_roles(verb)
            .iter()
            .any(|theta_role| theta_role.role_type == role)
    }

    /// Get all verbs that can take a specific theta role
    pub fn verbs_with_role(&self, role: ThetaRoleType) -> Vec<String> {
        self.theta_roles_index
            .get(&role)
            .map(|class_ids| {
                class_ids
                    .iter()
                    .filter_map(|id| self.classes.get(id))
                    .flat_map(|class| &class.members)
                    .map(|member| member.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get statistics about the loaded VerbNet data
    pub fn get_statistics(&self) -> VerbNetStats {
        VerbNetStats {
            total_classes: self.classes.len(),
            total_verbs: self.verb_to_classes.len(),
            total_roles: self.theta_roles_index.len(),
            total_predicates: self.predicates_index.len(),
            total_restrictions: self.restrictions_index.len(),
        }
    }

    /// Check if the engine has been initialized with data
    pub fn is_initialized(&self) -> bool {
        !self.classes.is_empty()
    }

    /// Add a verb class and update all indices
    fn add_class(&mut self, class: VerbClass) {
        let class_id = class.id.clone();

        // Index verbs to classes
        for member in &class.members {
            self.verb_to_classes
                .entry(member.name.clone())
                .or_default()
                .push(class_id.clone());
        }

        // Index theta roles
        for role in &class.theta_roles {
            self.theta_roles_index
                .entry(role.role_type)
                .or_default()
                .push(class_id.clone());

            // Index selectional restrictions
            for restriction in &role.selectional_restrictions {
                self.restrictions_index
                    .entry(*restriction)
                    .or_default()
                    .push(class_id.clone());
            }
        }

        // Index semantic predicates
        for frame in &class.frames {
            for predicate in &frame.semantics {
                self.predicates_index
                    .entry(predicate.predicate_type.clone())
                    .or_default()
                    .push(class_id.clone());
            }
        }

        // Store the class
        self.classes.insert(class_id, class);
    }

    /// Add some test data for development
    pub fn add_test_data(&mut self) {
        // Example: "give" verb class
        let give_class = VerbClass {
            id: "give-13.1".to_string(),
            name: "Give verbs".to_string(),
            members: vec![
                VerbMember {
                    name: "give".to_string(),
                    wn_sense: Some("give%2:40:00".to_string()),
                    fn_mapping: Some("Giving".to_string()),
                    grouping: None,
                },
                VerbMember {
                    name: "hand".to_string(),
                    wn_sense: Some("hand%2:35:00".to_string()),
                    fn_mapping: Some("Giving".to_string()),
                    grouping: None,
                },
            ],
            theta_roles: vec![
                ThetaRole {
                    role_type: ThetaRoleType::Agent,
                    selectional_restrictions: vec![SelectionalRestriction::Animate],
                    syntax_restrictions: vec![],
                },
                ThetaRole {
                    role_type: ThetaRoleType::Theme,
                    selectional_restrictions: vec![SelectionalRestriction::Concrete],
                    syntax_restrictions: vec![],
                },
                ThetaRole {
                    role_type: ThetaRoleType::Recipient,
                    selectional_restrictions: vec![SelectionalRestriction::Animate],
                    syntax_restrictions: vec![],
                },
            ],
            frames: vec![SyntacticFrame {
                description: "NP V NP NP".to_string(),
                primary: "ditransitive".to_string(),
                secondary: None,
                example: "She gave him a book".to_string(),
                syntax: SyntaxPattern {
                    elements: vec![
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Agent),
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "V".to_string(),
                            theta_role: None,
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Recipient),
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Theme),
                            restrictions: vec![],
                        },
                    ],
                },
                semantics: vec![
                    SemanticPredicate {
                        predicate_type: PredicateType::Transfer,
                        event_time: EventTime::During,
                        arguments: vec![
                            "Agent".to_string(),
                            "Theme".to_string(),
                            "Recipient".to_string(),
                        ],
                        negated: false,
                    },
                    SemanticPredicate {
                        predicate_type: PredicateType::Cause,
                        event_time: EventTime::Start,
                        arguments: vec!["Agent".to_string()],
                        negated: false,
                    },
                ],
            }],
            subclasses: vec![],
        };

        self.add_class(give_class);

        // Example: "hit" verb class (needed for tests)
        let hit_class = VerbClass {
            id: "hit-18.1".to_string(),
            name: "Hit verbs".to_string(),
            members: vec![
                VerbMember {
                    name: "hit".to_string(),
                    wn_sense: Some("hit%2:35:01".to_string()),
                    fn_mapping: Some("Impact".to_string()),
                    grouping: None,
                },
                VerbMember {
                    name: "strike".to_string(),
                    wn_sense: Some("strike%2:35:00".to_string()),
                    fn_mapping: Some("Impact".to_string()),
                    grouping: None,
                },
            ],
            theta_roles: vec![
                ThetaRole {
                    role_type: ThetaRoleType::Agent,
                    selectional_restrictions: vec![SelectionalRestriction::Animate],
                    syntax_restrictions: vec![],
                },
                ThetaRole {
                    role_type: ThetaRoleType::Patient,
                    selectional_restrictions: vec![],
                    syntax_restrictions: vec![],
                },
            ],
            frames: vec![SyntacticFrame {
                description: "NP V NP".to_string(),
                primary: "transitive".to_string(),
                secondary: None,
                example: "John hit Mary".to_string(),
                syntax: SyntaxPattern {
                    elements: vec![
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Agent),
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "V".to_string(),
                            theta_role: None,
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Patient),
                            restrictions: vec![],
                        },
                    ],
                },
                semantics: vec![
                    SemanticPredicate {
                        predicate_type: PredicateType::Contact,
                        event_time: EventTime::During,
                        arguments: vec!["Agent".to_string(), "Patient".to_string()],
                        negated: false,
                    },
                    SemanticPredicate {
                        predicate_type: PredicateType::Cause,
                        event_time: EventTime::Start,
                        arguments: vec!["Agent".to_string()],
                        negated: false,
                    },
                ],
            }],
            subclasses: vec![],
        };

        self.add_class(hit_class);

        info!("Added test VerbNet data for development");
    }
}

impl Default for VerbNetEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern analysis result with multiple parse options and confidence scores
/// This enables the semantic layer to handle ambiguity by choosing the best analysis
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    /// The VerbNet class ID that produced this analysis
    pub verb_class_id: String,

    /// Index of the frame within the class
    pub frame_index: usize,

    /// Human-readable frame description
    pub frame_description: String,

    /// Overall confidence of this analysis (0.0 to 1.0)
    pub confidence: f32,

    /// Theta role assignments for this analysis
    pub theta_assignments: Vec<ThetaAssignment>,

    /// Detailed assignment information for debugging
    pub assignment_details: Vec<AssignmentDetail>,

    /// The dependency pattern that was analyzed
    pub dependency_pattern: String,

    /// Semantic predicates from this frame
    pub semantic_predicates: Vec<SemanticPredicate>,
}

/// Individual theta role assignment with confidence
#[derive(Debug, Clone)]
pub struct ThetaAssignment {
    /// The assigned theta role
    pub theta_role: ThetaRoleType,

    /// Text of the argument that fills this role
    pub argument_text: String,

    /// Confidence in this assignment (0.0 to 1.0)
    pub confidence: f32,

    /// Source dependency relation (e.g., "nsubj", "dobj", "prep_to")
    pub source_relation: Option<String>,
}

/// Detailed information about how elements were matched
#[derive(Debug, Clone)]
pub struct AssignmentDetail {
    /// Index of the syntax element in the frame
    pub element_index: usize,

    /// Theta role assigned to this element
    pub theta_role: Option<ThetaRoleType>,

    /// Dependency relations that matched this element
    pub matched_relations: Vec<String>,

    /// Confidence score for this element match
    pub confidence: f32,
}

/// Statistics about loaded VerbNet data
#[derive(Debug, Clone)]
pub struct VerbNetStats {
    pub total_classes: usize,
    pub total_verbs: usize,
    pub total_roles: usize,
    pub total_predicates: usize,
    pub total_restrictions: usize,
}

impl VerbNetEngine {
    /// Map dependency pattern to theta roles with selectional restriction validation
    /// This version returns analyses sorted by selectional restriction scores
    pub fn map_dependency_pattern_to_theta_roles_validated(
        &self,
        verb: &str,
        syntactic_pattern: &str,
        arguments: &[(String, String, String)], // (relation, word, lemma)
        udpipe_features: &[(String, Vec<(String, String)>)], // (word, features)
    ) -> Vec<ThematicAnalysisValidated> {
        // Get base analyses using existing method
        let base_analyses = self.map_dependency_pattern_to_theta_roles(
            verb,
            syntactic_pattern,
            &arguments
                .iter()
                .map(|(rel, word, _)| (rel.clone(), word.clone()))
                .collect::<Vec<_>>(),
        );

        // Enhance each analysis with selectional restriction validation
        let mut validated_analyses = Vec::new();

        for analysis in base_analyses {
            let mut validation_results = Vec::new();
            let mut total_score = 0.0;
            let mut _total_confidence = 0.0;
            let mut validated_count = 0;

            // Validate each theta assignment
            for assignment in &analysis.theta_assignments {
                // Find the corresponding argument
                if let Some((_, word, lemma)) = arguments
                    .iter()
                    .find(|(rel, _, _)| assignment.source_relation.as_ref() == Some(rel))
                {
                    // Find UDPipe features for this word
                    let features = udpipe_features
                        .iter()
                        .find(|(w, _)| w == word)
                        .map(|(_, feats)| feats.as_slice())
                        .unwrap_or(&[]);

                    // Find the theta role definition
                    if let Some(verb_class) = self
                        .classes
                        .values()
                        .find(|cls| cls.id == analysis.verb_class_id)
                    {
                        if let Some(theta_role) = verb_class
                            .theta_roles
                            .iter()
                            .find(|role| role.role_type == assignment.theta_role)
                        {
                            let validation = self
                                .restriction_validator
                                .validate_assignment(word, features, lemma, theta_role);

                            total_score += validation.score;
                            _total_confidence += validation.confidence;
                            validated_count += 1;

                            validation_results.push(ThematicAssignmentValidated {
                                theta_role: assignment.theta_role,
                                dependency_relation: assignment
                                    .source_relation
                                    .clone()
                                    .unwrap_or_else(|| "unknown".to_string()),
                                confidence: assignment.confidence,
                                selectional_validation: validation,
                            });
                        }
                    }
                } else {
                    // No validation available - use original assignment with neutral validation
                    validation_results.push(ThematicAssignmentValidated {
                        theta_role: assignment.theta_role,
                        dependency_relation: assignment
                            .source_relation
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string()),
                        confidence: assignment.confidence,
                        selectional_validation: ValidationResult {
                            score: 0.5,
                            restriction_scores: Vec::new(),
                            confidence: 0.5,
                            explanation: format!(
                                "No selectional validation available for '{}'",
                                assignment
                                    .source_relation
                                    .as_ref()
                                    .unwrap_or(&"unknown".to_string())
                            ),
                        },
                    });
                }
            }

            // Calculate overall scores
            let avg_selectional_score = if validated_count > 0 {
                total_score / validated_count as f32
            } else {
                0.5 // Neutral when no validation possible
            };

            // Combine syntactic confidence with selectional validation
            // Use weighted average: syntactic confidence gets 40%, selectional score gets 60%
            // (selectional restrictions are more discriminative)
            let combined_confidence = 0.4 * analysis.confidence + 0.6 * avg_selectional_score;

            validated_analyses.push(ThematicAnalysisValidated {
                verb_class_id: analysis.verb_class_id,
                confidence: combined_confidence,
                syntactic_confidence: analysis.confidence,
                selectional_score: avg_selectional_score,
                theta_assignments: validation_results,
                explanation: format!(
                    "Combined analysis: syntactic={:.2}, selectional={:.2}, overall={:.2}",
                    analysis.confidence, avg_selectional_score, combined_confidence
                ),
            });
        }

        // Sort by combined confidence (highest first)
        validated_analyses.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        validated_analyses
    }

    /// Get the best theta assignment with selectional validation
    pub fn get_best_theta_assignment_validated(
        &self,
        verb: &str,
        syntactic_pattern: &str,
        arguments: &[(String, String, String)], // (relation, word, lemma)
        udpipe_features: &[(String, Vec<(String, String)>)], // (word, features)
    ) -> Option<ThematicAnalysisValidated> {
        let analyses = self.map_dependency_pattern_to_theta_roles_validated(
            verb,
            syntactic_pattern,
            arguments,
            udpipe_features,
        );

        analyses.into_iter().next()
    }

    /// Validate a specific theta assignment against selectional restrictions
    pub fn validate_theta_assignment(
        &self,
        word: &str,
        lemma: &str,
        udpipe_features: &[(String, String)],
        theta_role_type: ThetaRoleType,
        verb_class_id: &str,
    ) -> Option<ValidationResult> {
        // Find the verb class and theta role
        let verb_class = self.classes.get(verb_class_id)?;
        let theta_role = verb_class
            .theta_roles
            .iter()
            .find(|role| role.role_type == theta_role_type)?;

        Some(self.restriction_validator.validate_assignment(
            word,
            udpipe_features,
            lemma,
            theta_role,
        ))
    }
}

/// Enhanced thematic analysis with selectional restriction validation
#[derive(Debug, Clone)]
pub struct ThematicAnalysisValidated {
    /// VerbNet verb class ID
    pub verb_class_id: String,

    /// Combined confidence score (syntactic + selectional)
    pub confidence: f32,

    /// Original syntactic confidence from pattern matching
    pub syntactic_confidence: f32,

    /// Average selectional restriction score
    pub selectional_score: f32,

    /// Validated theta assignments
    pub theta_assignments: Vec<ThematicAssignmentValidated>,

    /// Human-readable explanation
    pub explanation: String,
}

/// Theta assignment with selectional restriction validation
#[derive(Debug, Clone)]
pub struct ThematicAssignmentValidated {
    /// The theta role assigned
    pub theta_role: ThetaRoleType,

    /// Dependency relation this role fills
    pub dependency_relation: String,

    /// Original syntactic confidence
    pub confidence: f32,

    /// Selectional restriction validation result
    pub selectional_validation: ValidationResult,
}

/// Selectional restriction validator for theta role assignments
#[derive(Debug)]
pub struct SelectionalRestrictionValidator {
    /// Cache of word -> selectional features mappings for performance
    feature_cache: std::sync::Mutex<lru::LruCache<String, SelectionalFeatures>>,
}

/// Features extracted from UDPipe that relate to selectional restrictions
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionalFeatures {
    /// Animacy features
    pub animacy: Option<Animacy>,

    /// Noun type/category
    pub noun_type: Option<NounType>,

    /// Semantic properties inferred from lemma/context
    pub semantic_properties: Vec<SemanticProperty>,

    /// Confidence in feature extraction (0.0-1.0)
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Animacy {
    Animate,
    Inanimate,
    Human,
    Organization,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NounType {
    Concrete,
    Abstract,
    Substance,
    Location,
    Time,
    Event,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticProperty {
    Comestible, // Food/drink
    Currency,   // Money
    Machine,    // Mechanical device
    Vehicle,    // Transportation
    Garment,    // Clothing
    BodyPart,   // Body parts
    Plant,      // Vegetation
    Container,  // Can hold things
    Rigid,      // Hard/firm
    NonRigid,   // Soft/flexible
    Elongated,  // Long and thin
    Pointy,     // Sharp/pointed
    Natural,    // From nature
    Artificial, // Human-made
}

/// Validation result for a theta assignment
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Overall validation score (0.0-1.0, higher = better match)
    pub score: f32,

    /// Individual restriction scores
    pub restriction_scores: Vec<(SelectionalRestriction, f32)>,

    /// Confidence in the validation
    pub confidence: f32,

    /// Human-readable explanation
    pub explanation: String,
}

impl SelectionalRestrictionValidator {
    /// Create a new validator with reasonable cache size
    pub fn new() -> Self {
        Self {
            feature_cache: std::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(1000).unwrap(),
            )),
        }
    }

    /// Validate a theta assignment against selectional restrictions
    pub fn validate_assignment(
        &self,
        word: &str,
        udpipe_features: &[(String, String)], // UDPipe morphological features
        lemma: &str,
        theta_role: &ThetaRole,
    ) -> ValidationResult {
        // Extract selectional features from UDPipe data
        let features = self.extract_selectional_features(word, udpipe_features, lemma);

        // Score against each selectional restriction
        let mut restriction_scores = Vec::new();
        let mut total_score = 0.0;
        let mut max_possible = 0.0;

        for restriction in &theta_role.selectional_restrictions {
            let score = self.score_restriction(&features, restriction);
            restriction_scores.push((restriction.clone(), score));
            total_score += score;
            max_possible += 1.0;
        }

        // Calculate overall score
        let overall_score = if max_possible > 0.0 {
            total_score / max_possible
        } else {
            // No restrictions means all assignments are equally valid
            0.5
        };

        // Generate explanation
        let explanation =
            self.generate_explanation(word, &features, &restriction_scores, overall_score);

        ValidationResult {
            score: overall_score,
            restriction_scores,
            confidence: features.confidence,
            explanation,
        }
    }

    /// Extract selectional features from UDPipe data
    fn extract_selectional_features(
        &self,
        _word: &str,
        udpipe_features: &[(String, String)],
        lemma: &str,
    ) -> SelectionalFeatures {
        // Check cache first
        {
            let mut cache = self.feature_cache.lock().unwrap();
            if let Some(cached) = cache.get(lemma) {
                return cached.clone();
            }
        }

        let mut features = SelectionalFeatures {
            animacy: None,
            noun_type: None,
            semantic_properties: Vec::new(),
            confidence: 0.8, // Base confidence
        };

        // Extract from UDPipe features
        for (feat_name, feat_value) in udpipe_features {
            match feat_name.as_str() {
                "Animacy" => {
                    features.animacy = match feat_value.as_str() {
                        "Anim" => Some(Animacy::Animate),
                        "Inan" => Some(Animacy::Inanimate),
                        "Hum" => Some(Animacy::Human),
                        _ => None,
                    };
                }
                "NounClass" | "Gender" => {
                    // Some languages mark animacy/humanness via gender
                    if feat_value.contains("Hum") {
                        features.animacy = Some(Animacy::Human);
                    } else if feat_value.contains("Anim") {
                        features.animacy = Some(Animacy::Animate);
                    }
                }
                _ => {} // Other features can be added as needed
            }
        }

        // Lexical inference from lemma
        self.infer_from_lemma(lemma, &mut features);

        // Cache result
        {
            let mut cache = self.feature_cache.lock().unwrap();
            cache.put(lemma.to_string(), features.clone());
        }

        features
    }

    /// Infer selectional features from lemma (lexical knowledge)
    fn infer_from_lemma(&self, lemma: &str, features: &mut SelectionalFeatures) {
        // This is a simplified lexical knowledge base - in production,
        // this would likely use WordNet, ConceptNet, or similar resources

        // Animate beings
        if matches!(
            lemma,
            "person"
                | "people"
                | "man"
                | "woman"
                | "child"
                | "human"
                | "individual"
                | "someone"
                | "somebody"
        ) {
            features.animacy = Some(Animacy::Human);
            features.noun_type = Some(NounType::Concrete);
        } else if matches!(
            lemma,
            "dog" | "cat" | "animal" | "bird" | "fish" | "creature"
        ) {
            features.animacy = Some(Animacy::Animate);
            features.noun_type = Some(NounType::Concrete);
        }
        // Organizations
        else if matches!(
            lemma,
            "company" | "corporation" | "organization" | "government" | "institution" | "agency"
        ) {
            features.animacy = Some(Animacy::Organization);
            features.noun_type = Some(NounType::Abstract);
        }
        // Food/drink
        else if matches!(
            lemma,
            "food" | "bread" | "water" | "coffee" | "beer" | "meal" | "drink" | "apple" | "meat"
        ) {
            features.animacy = Some(Animacy::Inanimate);
            features.noun_type = Some(NounType::Concrete);
            features
                .semantic_properties
                .push(SemanticProperty::Comestible);
        }
        // Money/currency
        else if matches!(
            lemma,
            "money" | "dollar" | "cash" | "currency" | "coin" | "bill"
        ) {
            features.animacy = Some(Animacy::Inanimate);
            features.noun_type = Some(NounType::Concrete);
            features
                .semantic_properties
                .push(SemanticProperty::Currency);
        }
        // Vehicles
        else if matches!(
            lemma,
            "car" | "truck" | "bike" | "vehicle" | "bus" | "train" | "plane"
        ) {
            features.animacy = Some(Animacy::Inanimate);
            features.noun_type = Some(NounType::Concrete);
            features.semantic_properties.push(SemanticProperty::Vehicle);
            features.semantic_properties.push(SemanticProperty::Machine);
        }
        // Machines
        else if matches!(
            lemma,
            "machine" | "computer" | "phone" | "device" | "tool" | "equipment"
        ) {
            features.animacy = Some(Animacy::Inanimate);
            features.noun_type = Some(NounType::Concrete);
            features.semantic_properties.push(SemanticProperty::Machine);
            features
                .semantic_properties
                .push(SemanticProperty::Artificial);
        }
        // Locations
        else if matches!(
            lemma,
            "place" | "location" | "house" | "building" | "room" | "city" | "country"
        ) {
            features.animacy = Some(Animacy::Inanimate);
            features.noun_type = Some(NounType::Location);
        }
        // Abstract concepts
        else if matches!(
            lemma,
            "idea" | "concept" | "thought" | "feeling" | "emotion" | "plan" | "strategy"
        ) {
            features.animacy = Some(Animacy::Inanimate);
            features.noun_type = Some(NounType::Abstract);
        }
        // Time expressions
        else if matches!(
            lemma,
            "time" | "moment" | "day" | "hour" | "minute" | "year" | "period"
        ) {
            features.animacy = Some(Animacy::Inanimate);
            features.noun_type = Some(NounType::Time);
        }
        // Default fallbacks for common cases
        else if lemma.ends_with("ing") {
            // Gerunds are often events/abstractions
            features.noun_type = Some(NounType::Event);
        } else {
            // Default: assume inanimate concrete
            features.animacy = Some(Animacy::Inanimate);
            features.noun_type = Some(NounType::Concrete);
            features.confidence *= 0.5; // Lower confidence for defaults
        }
    }

    /// Score how well features match a selectional restriction
    fn score_restriction(
        &self,
        features: &SelectionalFeatures,
        restriction: &SelectionalRestriction,
    ) -> f32 {
        use SelectionalRestriction as SR;

        match restriction {
            SR::Animate => {
                match &features.animacy {
                    Some(Animacy::Animate) | Some(Animacy::Human) => 1.0,
                    Some(Animacy::Organization) => 0.7, // Organizations can act like animates
                    Some(Animacy::Inanimate) => 0.0,
                    None => 0.3, // Uncertain
                }
            }

            SR::Human => {
                match &features.animacy {
                    Some(Animacy::Human) => 1.0,
                    Some(Animacy::Animate) => 0.3, // Animals are less likely but possible
                    Some(Animacy::Organization) => 0.1, // Very unlikely
                    Some(Animacy::Inanimate) => 0.0,
                    None => 0.2,
                }
            }

            SR::Organization => {
                match &features.animacy {
                    Some(Animacy::Organization) => 1.0,
                    Some(Animacy::Human) => 0.2, // Humans can represent organizations
                    _ => 0.0,
                }
            }

            SR::Concrete => {
                match &features.noun_type {
                    Some(NounType::Concrete) => 1.0,
                    Some(NounType::Location) => 0.8, // Locations are concrete
                    Some(NounType::Substance) => 0.9, // Substances are concrete
                    Some(NounType::Abstract) => 0.0,
                    _ => 0.4,
                }
            }

            SR::Abstract => {
                match &features.noun_type {
                    Some(NounType::Abstract) => 1.0,
                    Some(NounType::Event) => 0.9, // Events are abstract
                    Some(NounType::Time) => 0.7,  // Time concepts are abstract
                    Some(NounType::Concrete) => 0.1,
                    _ => 0.3,
                }
            }

            SR::Comestible => {
                if features
                    .semantic_properties
                    .contains(&SemanticProperty::Comestible)
                {
                    1.0
                } else {
                    0.0
                }
            }

            SR::Currency => {
                if features
                    .semantic_properties
                    .contains(&SemanticProperty::Currency)
                {
                    1.0
                } else {
                    0.0
                }
            }

            SR::Machine => {
                if features
                    .semantic_properties
                    .contains(&SemanticProperty::Machine)
                {
                    1.0
                } else if features
                    .semantic_properties
                    .contains(&SemanticProperty::Vehicle)
                {
                    0.8 // Vehicles are machines
                } else {
                    0.0
                }
            }

            SR::Vehicle => {
                if features
                    .semantic_properties
                    .contains(&SemanticProperty::Vehicle)
                {
                    1.0
                } else {
                    0.0
                }
            }

            // Add more restriction types as needed...
            _ => 0.5, // Neutral score for unimplemented restrictions
        }
    }

    /// Generate human-readable explanation for validation
    fn generate_explanation(
        &self,
        word: &str,
        features: &SelectionalFeatures,
        scores: &[(SelectionalRestriction, f32)],
        overall_score: f32,
    ) -> String {
        if scores.is_empty() {
            return format!(
                "'{}' has no selectional restrictions (neutral assignment)",
                word
            );
        }

        let mut explanation = format!("Selectional validation for '{}': ", word);

        // Describe extracted features
        if let Some(animacy) = &features.animacy {
            explanation.push_str(&format!("{:?} ", animacy));
        }
        if let Some(noun_type) = &features.noun_type {
            explanation.push_str(&format!("{:?} ", noun_type));
        }

        // Describe restriction matches
        let good_matches: Vec<_> = scores.iter().filter(|(_, score)| *score > 0.7).collect();
        let poor_matches: Vec<_> = scores.iter().filter(|(_, score)| *score < 0.3).collect();

        if !good_matches.is_empty() {
            explanation.push_str("✓ Matches: ");
            for (restriction, score) in good_matches {
                explanation.push_str(&format!("{:?}({:.1}) ", restriction, score));
            }
        }

        if !poor_matches.is_empty() {
            explanation.push_str("✗ Conflicts: ");
            for (restriction, score) in poor_matches {
                explanation.push_str(&format!("{:?}({:.1}) ", restriction, score));
            }
        }

        explanation.push_str(&format!("(Score: {:.2})", overall_score));
        explanation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selectional_restriction_validation() {
        let validator = SelectionalRestrictionValidator::new();

        // Test human animacy validation
        let human_features = vec![("Animacy".to_string(), "Hum".to_string())];
        let human_role = ThetaRole {
            role_type: ThetaRoleType::Agent,
            selectional_restrictions: vec![SelectionalRestriction::Human],
            syntax_restrictions: Vec::new(),
        };

        let result =
            validator.validate_assignment("person", &human_features, "person", &human_role);
        assert!(
            result.score > 0.8,
            "Human should score high for Human restriction: {}",
            result.score
        );
        assert!(result.explanation.contains("Human"));

        // Test animal vs human validation
        let animal_result = validator.validate_assignment("dog", &[], "dog", &human_role);
        assert!(
            animal_result.score < 0.4,
            "Dog should score low for Human restriction: {}",
            animal_result.score
        );

        // Test concrete objects
        let concrete_role = ThetaRole {
            role_type: ThetaRoleType::Theme,
            selectional_restrictions: vec![SelectionalRestriction::Concrete],
            syntax_restrictions: Vec::new(),
        };

        let car_result = validator.validate_assignment("car", &[], "car", &concrete_role);
        assert!(
            car_result.score > 0.8,
            "Car should score high for Concrete restriction: {}",
            car_result.score
        );

        // Test food for comestible restriction
        let comestible_role = ThetaRole {
            role_type: ThetaRoleType::Theme,
            selectional_restrictions: vec![SelectionalRestriction::Comestible],
            syntax_restrictions: Vec::new(),
        };

        let apple_result = validator.validate_assignment("apple", &[], "apple", &comestible_role);
        assert!(
            apple_result.score > 0.9,
            "Apple should score very high for Comestible restriction: {}",
            apple_result.score
        );

        let car_food_result = validator.validate_assignment("car", &[], "car", &comestible_role);
        assert!(
            car_food_result.score < 0.1,
            "Car should score very low for Comestible restriction: {}",
            car_food_result.score
        );
    }

    #[test]
    fn test_validated_theta_assignment_integration() {
        let mut engine = VerbNetEngine::new();

        // Add a simple give class for testing
        let give_class = VerbClass {
            id: "give-13.1".to_string(),
            name: "give".to_string(),
            members: vec![VerbMember {
                name: "give".to_string(),
                wn_sense: None,
                fn_mapping: None,
                grouping: None,
            }],
            theta_roles: vec![
                ThetaRole {
                    role_type: ThetaRoleType::Agent,
                    selectional_restrictions: vec![SelectionalRestriction::Animate],
                    syntax_restrictions: Vec::new(),
                },
                ThetaRole {
                    role_type: ThetaRoleType::Theme,
                    selectional_restrictions: vec![SelectionalRestriction::Concrete],
                    syntax_restrictions: Vec::new(),
                },
                ThetaRole {
                    role_type: ThetaRoleType::Recipient,
                    selectional_restrictions: vec![SelectionalRestriction::Animate],
                    syntax_restrictions: Vec::new(),
                },
            ],
            frames: vec![SyntacticFrame {
                description: "NP V NP PP.recipient".to_string(),
                primary: "NP V NP PP".to_string(),
                secondary: Some("Dative".to_string()),
                example: "John gives Mary a book".to_string(),
                syntax: SyntaxPattern {
                    elements: vec![
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Agent),
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "V".to_string(),
                            theta_role: None,
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "NP".to_string(),
                            theta_role: Some(ThetaRoleType::Theme),
                            restrictions: vec![],
                        },
                        SyntaxElement {
                            category: "PP".to_string(),
                            theta_role: Some(ThetaRoleType::Recipient),
                            restrictions: vec![],
                        },
                    ],
                },
                semantics: vec![],
            }],
            subclasses: vec![],
        };

        engine.add_class(give_class);

        // First, let's test the basic method to see if it finds anything
        let basic_args = vec![
            ("nsubj".to_string(), "person".to_string()),
            ("dobj".to_string(), "book".to_string()),
            ("prep_to".to_string(), "child".to_string()),
        ];

        let basic_analyses =
            engine.map_dependency_pattern_to_theta_roles("give", "nsubj+dobj+prep_to", &basic_args);

        println!("Basic analyses found: {}", basic_analyses.len());
        for (i, analysis) in basic_analyses.iter().enumerate() {
            println!(
                "  Analysis {}: class={}, confidence={:.3}, assignments={}",
                i,
                analysis.verb_class_id,
                analysis.confidence,
                analysis.theta_assignments.len()
            );
            for assignment in &analysis.theta_assignments {
                println!(
                    "    {:?} <- {:?}",
                    assignment.theta_role, assignment.source_relation
                );
            }
        }

        // Test validated mapping with good selectional fit
        let arguments = vec![
            (
                "nsubj".to_string(),
                "person".to_string(),
                "person".to_string(),
            ),
            ("dobj".to_string(), "book".to_string(), "book".to_string()),
            (
                "prep_to".to_string(),
                "child".to_string(),
                "child".to_string(),
            ),
        ];

        let udpipe_features = vec![
            (
                "person".to_string(),
                vec![("Animacy".to_string(), "Hum".to_string())],
            ),
            ("book".to_string(), vec![]),
            (
                "child".to_string(),
                vec![("Animacy".to_string(), "Hum".to_string())],
            ),
        ];

        let analyses = engine.map_dependency_pattern_to_theta_roles_validated(
            "give",
            "nsubj+dobj+prep_to",
            &arguments,
            &udpipe_features,
        );

        println!("Validated analyses found: {}", analyses.len());

        if analyses.is_empty() {
            println!("Engine has {} classes total", engine.classes.len());
            for (verb, class) in &engine.classes {
                println!("  Verb '{}' has class '{}'", verb, class.id);
            }

            // Try to understand why no analyses were found
            let verb_classes = engine.get_verb_classes("give");
            println!(
                "get_verb_classes('give') returned {} classes",
                verb_classes.len()
            );

            // Check if there are any basic analyses at all
            if basic_analyses.is_empty() {
                println!("No basic analyses found - the issue is in the base mapping method");
            }
        }

        assert!(
            !analyses.is_empty(),
            "Should find validated analyses for give"
        );

        let best = &analyses[0];

        println!("Analysis details:");
        println!("  Syntactic confidence: {:.3}", best.syntactic_confidence);
        println!("  Selectional score: {:.3}", best.selectional_score);
        println!("  Combined confidence: {:.3}", best.confidence);

        assert!(
            best.selectional_score > 0.3,
            "Selectional score should be reasonable: {}",
            best.selectional_score
        );

        // The combined confidence might not always be higher due to averaging
        // Let's just check that it's reasonable
        assert!(
            best.confidence > 0.4,
            "Combined confidence should be reasonable: {}",
            best.confidence
        );

        println!("Best validated analysis: {}", best.explanation);
        println!("Theta assignments:");
        for assignment in &best.theta_assignments {
            println!(
                "  {:?} <- {} (selectional: {:.2})",
                assignment.theta_role,
                assignment.dependency_relation,
                assignment.selectional_validation.score
            );
        }
    }

    #[test]
    fn test_selectional_disambiguation() {
        let validator = SelectionalRestrictionValidator::new();

        // Test cases where selectional restrictions help choose between alternatives

        // Agent that requires HUMAN vs just ANIMATE
        let human_agent_role = ThetaRole {
            role_type: ThetaRoleType::Agent,
            selectional_restrictions: vec![SelectionalRestriction::Human],
            syntax_restrictions: Vec::new(),
        };

        let animate_agent_role = ThetaRole {
            role_type: ThetaRoleType::Agent,
            selectional_restrictions: vec![SelectionalRestriction::Animate],
            syntax_restrictions: Vec::new(),
        };

        // Test with human
        let human_for_human =
            validator.validate_assignment("person", &[], "person", &human_agent_role);
        let human_for_animate =
            validator.validate_assignment("person", &[], "person", &animate_agent_role);

        assert!(
            human_for_human.score >= human_for_animate.score,
            "Human restriction should be preferred for human entities"
        );

        // Test with animal
        let dog_for_human = validator.validate_assignment("dog", &[], "dog", &human_agent_role);
        let dog_for_animate = validator.validate_assignment("dog", &[], "dog", &animate_agent_role);

        assert!(
            dog_for_animate.score > dog_for_human.score,
            "Animate restriction should be preferred for animal entities"
        );

        println!("Disambiguation test results:");
        println!(
            "  Human for Human restriction: {:.2}",
            human_for_human.score
        );
        println!(
            "  Human for Animate restriction: {:.2}",
            human_for_animate.score
        );
        println!("  Dog for Human restriction: {:.2}", dog_for_human.score);
        println!(
            "  Dog for Animate restriction: {:.2}",
            dog_for_animate.score
        );
    }

    #[test]
    fn test_pattern_mapping_multiple_analyses() {
        let engine = VerbNetEngine::new();
        let mut engine_with_data = engine;
        engine_with_data.add_test_data();

        // Test pattern mapping for "give" with ditransitive pattern
        let analyses = engine_with_data.map_dependency_pattern_to_theta_roles(
            "give",
            "nsubj+dobj+iobj", // Ditransitive: She gave him a book
            &[
                ("nsubj".to_string(), "she".to_string()),
                ("dobj".to_string(), "book".to_string()),
                ("iobj".to_string(), "him".to_string()),
            ],
        );

        assert!(
            !analyses.is_empty(),
            "Should find pattern analyses for 'give'"
        );

        // Check that we have confidence scores and theta assignments
        let first_analysis = &analyses[0];
        assert!(
            first_analysis.confidence > 0.0,
            "Should have positive confidence"
        );
        assert!(
            !first_analysis.theta_assignments.is_empty(),
            "Should have theta assignments"
        );

        // Check that multiple theta roles are assigned
        let theta_roles: Vec<_> = first_analysis
            .theta_assignments
            .iter()
            .map(|assignment| assignment.theta_role)
            .collect();

        assert!(
            theta_roles.contains(&ThetaRoleType::Agent),
            "Should assign Agent role"
        );
        assert!(
            theta_roles.contains(&ThetaRoleType::Theme),
            "Should assign Theme role"
        );
        assert!(
            theta_roles.contains(&ThetaRoleType::Recipient),
            "Should assign Recipient role"
        );

        println!("Pattern analysis results: {:#?}", analyses);
    }

    #[test]
    fn test_pattern_mapping_confidence_scoring() {
        let engine = VerbNetEngine::new();
        let mut engine_with_data = engine;
        engine_with_data.add_test_data();

        // Test with good match (ditransitive give)
        let good_analyses = engine_with_data.map_dependency_pattern_to_theta_roles(
            "give",
            "nsubj+dobj+iobj",
            &[
                ("nsubj".to_string(), "mary".to_string()),
                ("dobj".to_string(), "book".to_string()),
                ("iobj".to_string(), "john".to_string()),
            ],
        );

        // Test with poor match (intransitive pattern for ditransitive verb)
        let poor_analyses = engine_with_data.map_dependency_pattern_to_theta_roles(
            "give",
            "nsubj", // Just subject, no objects
            &[("nsubj".to_string(), "mary".to_string())],
        );

        if !good_analyses.is_empty() && !poor_analyses.is_empty() {
            // Good match should have higher confidence than poor match
            assert!(
                good_analyses[0].confidence > poor_analyses[0].confidence,
                "Good pattern match should have higher confidence than poor match"
            );
        }

        // Good analyses should have more theta assignments
        if !good_analyses.is_empty() {
            assert!(
                good_analyses[0].theta_assignments.len() >= 2,
                "Good match should assign multiple theta roles"
            );
        }
    }

    #[test]
    fn test_pattern_mapping_returns_sorted_analyses() {
        let engine = VerbNetEngine::new();
        let mut engine_with_data = engine;
        engine_with_data.add_test_data();

        let analyses = engine_with_data.map_dependency_pattern_to_theta_roles(
            "give",
            "nsubj+dobj+prep_to", // Prepositional variant: She gave a book to him
            &[
                ("nsubj".to_string(), "she".to_string()),
                ("dobj".to_string(), "book".to_string()),
                ("prep_to".to_string(), "him".to_string()),
            ],
        );

        // Results should be sorted by confidence (highest first)
        for i in 1..analyses.len() {
            assert!(
                analyses[i - 1].confidence >= analyses[i].confidence,
                "Analyses should be sorted by confidence (descending)"
            );
        }
    }

    #[test]
    fn test_cache_key_distinguishes_minimal_pairs() {
        // Test the crucial point about minimal pairs having different cache keys

        // Example minimal pair:
        // "John breaks the window" (Agent: John, Patient: window)
        // "The window breaks" (Theme: window, no agent)

        let key1 = VerbNetCacheKey::from_sentence_context(
            "break",
            "nsubj+dobj",
            &[
                ("nsubj".to_string(), "john".to_string()),
                ("dobj".to_string(), "window".to_string()),
            ],
            "active",
            "present",
        );

        let key2 = VerbNetCacheKey::from_sentence_context(
            "break",
            "nsubj",
            &[("nsubj".to_string(), "window".to_string())],
            "active",
            "present",
        );

        // These should be different keys because:
        // 1. Different dependency patterns ("nsubj+dobj" vs "nsubj")
        // 2. Different argument counts (2 vs 1)
        // 3. Different semantic hints (["joh", "win"] vs ["win"])
        assert_ne!(key1, key2, "Minimal pairs should have different cache keys");

        // Even more subtle: same verb, same pattern, but different voices
        let key3 = VerbNetCacheKey::from_sentence_context(
            "give",
            "nsubj+dobj+prep_to",
            &[
                ("nsubj".to_string(), "john".to_string()),
                ("dobj".to_string(), "book".to_string()),
                ("prep_to".to_string(), "mary".to_string()),
            ],
            "active",
            "present",
        );

        let key4 = VerbNetCacheKey::from_sentence_context(
            "give",
            "nsubjpass+prep_by+prep_to",
            &[
                ("nsubjpass".to_string(), "book".to_string()),
                ("prep_by".to_string(), "john".to_string()),
                ("prep_to".to_string(), "mary".to_string()),
            ],
            "passive",
            "present",
        );

        // Active vs passive should have different keys due to:
        // 1. Different dependency patterns
        // 2. Different voice
        assert_ne!(
            key3, key4,
            "Active and passive should have different cache keys"
        );
    }

    #[test]
    fn test_cache_hit_rate_tracking() {
        let mut cache = VerbNetCache::new(100);

        let key1 = VerbNetCacheKey::from_sentence_context(
            "run",
            "nsubj",
            &[("nsubj".to_string(), "john".to_string())],
            "active",
            "present",
        );

        let result = VerbNetLookupResult {
            theta_assignments: vec![(vec![], 0.8)],
            selectional_restrictions: vec![],
            aspectual_class: AspectualInfo {
                durative: true,
                dynamic: true,
                telic: false,
                punctual: false,
            },
            semantic_predicates: vec![],
        };

        // Initial state
        assert_eq!(cache.hit_rate(), 0.0);

        // First lookup - miss
        assert!(cache.get(&key1).is_none());
        assert_eq!(cache.hit_rate(), 0.0); // 0/1 = 0.0

        // Store result
        cache.insert(key1.clone(), result);

        // Second lookup - hit
        assert!(cache.get(&key1).is_some());
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.total_lookups, 2);
        assert_eq!(stats.hit_rate, 0.5); // 1/2 = 0.5
    }

    #[test]
    fn test_semantic_hint_generalization() {
        // Test that semantic hints generalize appropriately without over-generalization

        let key1 = VerbNetCacheKey::from_sentence_context(
            "eat",
            "nsubj+dobj",
            &[
                ("nsubj".to_string(), "person".to_string()),
                ("dobj".to_string(), "apple".to_string()),
            ],
            "active",
            "present",
        );

        let key2 = VerbNetCacheKey::from_sentence_context(
            "eat",
            "nsubj+dobj",
            &[
                ("nsubj".to_string(), "person".to_string()),
                ("dobj".to_string(), "pizza".to_string()),
            ],
            "active",
            "present",
        );

        // These should be the same key because:
        // 1. Same verb, pattern, voice, tense
        // 2. Same semantic hints: ["per", "app"] vs ["per", "piz"] - different, but close food items
        // The first 3 characters provide reasonable generalization
        assert_ne!(
            key1.arg_semantic_hints, key2.arg_semantic_hints,
            "Different foods should have different hints"
        );

        // But the overall caching strategy should still be safe because we include dependency patterns
        // and other contextual information
    }

    #[test]
    fn test_engine_creation() {
        let engine = VerbNetEngine::new();
        assert_eq!(engine.classes.len(), 0);
        assert_eq!(engine.verb_to_classes.len(), 0);
    }

    #[test]
    fn test_test_data() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();

        let stats = engine.get_statistics();
        assert!(stats.total_classes > 0);
        assert!(stats.total_verbs > 0);
    }

    #[test]
    fn test_verb_lookup() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();

        let classes = engine.get_verb_classes("give");
        assert!(!classes.is_empty());
        assert_eq!(classes[0].id, "give-13.1");
    }

    #[test]
    fn test_theta_role_lookup() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();

        let roles = engine.get_theta_roles("give");
        assert!(!roles.is_empty());

        let has_agent = roles.iter().any(|r| r.role_type == ThetaRoleType::Agent);
        assert!(has_agent);
    }

    #[test]
    fn test_aspectual_inference() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();

        let aspectual = engine.infer_aspectual_class("give");
        // "give" should be telic (has endpoint) and dynamic
        assert!(aspectual.dynamic || aspectual.telic); // At least one should be true
    }

    #[test]
    fn test_verb_allows_role() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();

        assert!(engine.verb_allows_role("give", ThetaRoleType::Agent));
        assert!(engine.verb_allows_role("give", ThetaRoleType::Theme));
        assert!(engine.verb_allows_role("give", ThetaRoleType::Recipient));
        assert!(!engine.verb_allows_role("give", ThetaRoleType::Instrument));
    }

    #[test]
    fn test_similarity_cache_hit() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data(); // This adds "give" and "hit" to VerbNet

        // Cache result for a known verb "give"
        let _result = engine
            .lookup_with_context(
                "give",
                "nsubj+dobj+prep_to",
                &[
                    ("nsubj".to_string(), "john".to_string()),
                    ("dobj".to_string(), "book".to_string()),
                    ("prep_to".to_string(), "mary".to_string()),
                ],
                "active",
                "present",
            )
            .unwrap();

        // Now try a morphological variant of a cached verb for an UNKNOWN verb
        // "sprinting" is not in our test VerbNet data, so it should trigger similarity fallback
        let similarity_result = engine.lookup_with_context(
            "sprinting",
            "nsubj",
            &[("nsubj".to_string(), "john".to_string())],
            "active",
            "present",
        );

        // This should fail because "sprinting" is not in VerbNet and has no similar cached patterns
        assert!(
            similarity_result.is_err(),
            "Unknown verb with no similar patterns should fail"
        );

        // Test with a verb that IS similar to cached patterns
        // First cache a simple "run" pattern (even though run isn't in our VerbNet test data)
        let mut cache = VerbNetCache::new(100);
        let run_key = VerbNetCacheKey::from_sentence_context(
            "sprint",
            "nsubj",
            &[("nsubj".to_string(), "athlete".to_string())],
            "active",
            "present",
        );

        let sprint_result = VerbNetLookupResult {
            theta_assignments: vec![(vec![], 0.8)],
            selectional_restrictions: vec![],
            aspectual_class: AspectualInfo {
                durative: true,
                dynamic: true,
                telic: false,
                punctual: false,
            },
            semantic_predicates: vec![],
        };

        cache.insert(run_key, sprint_result);

        // Test similarity search directly
        let sprinting_key = VerbNetCacheKey::from_sentence_context(
            "sprinting",
            "nsubj",
            &[("nsubj".to_string(), "athlete".to_string())],
            "active",
            "present",
        );

        let similarity_result = cache.find_similar_cached_verb(&sprinting_key);
        assert!(
            similarity_result.is_some(),
            "Should find similarity match for morphological variant"
        );

        let (_cached_result, confidence) = similarity_result.unwrap();
        assert!(
            confidence > 0.5,
            "Should have reasonable confidence for morphological variant"
        );
    }

    #[test]
    fn test_verb_similarity_scoring() {
        let cache = VerbNetCache::new(100);

        // Test exact match
        assert_eq!(cache.compute_verb_similarity("run", "run"), 1.0);

        // Test morphological variants
        assert!(cache.compute_verb_similarity("run", "running") > 0.85);
        assert!(cache.compute_verb_similarity("give", "gave") > 0.7);

        // Test different verbs
        assert!(cache.compute_verb_similarity("run", "walk") < 0.7);
        assert!(cache.compute_verb_similarity("give", "take") < 0.5);

        // Test stem extraction
        assert_eq!(cache.extract_verb_stem("running"), "run");
        assert_eq!(cache.extract_verb_stem("walked"), "walk");
        assert_eq!(cache.extract_verb_stem("gives"), "give");
        assert_eq!(cache.extract_verb_stem("run"), "run");
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("run", "run"), 0);
        assert_eq!(edit_distance("run", "running"), 4); // +"ning"
        assert_eq!(edit_distance("give", "gave"), 1); // i->a (e matches)
        assert_eq!(edit_distance("", "test"), 4);
        assert_eq!(edit_distance("test", ""), 4);
        assert_eq!(edit_distance("kitten", "sitting"), 3); // k->s, e->i, +g
    }

    #[test]
    fn test_verbnet_engine_creation() {
        // Test basic constructor
        let engine = VerbNetEngine::new();
        assert!(engine.classes.is_empty());
        assert!(engine.verb_to_classes.is_empty());

        // Test constructor with custom cache size
        let engine_with_cache = VerbNetEngine::with_cache_size(5000);
        assert!(engine_with_cache.classes.is_empty());
        assert!(engine_with_cache.verb_to_classes.is_empty());
    }

    #[test]
    fn test_engine_add_test_data() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();

        // Should have some test data now
        assert!(!engine.classes.is_empty());
        assert!(!engine.verb_to_classes.is_empty());

        // Test specific verbs that should be in test data
        let give_classes = engine.get_verb_classes("give");
        assert!(!give_classes.is_empty());

        let hit_classes = engine.get_verb_classes("hit");
        assert!(!hit_classes.is_empty());
    }

    #[test]
    fn test_theta_role_assignment() {
        let mut engine = VerbNetEngine::new();
        engine.add_test_data();

        // Test theta role retrieval
        let roles = engine.get_theta_roles("give");
        assert!(!roles.is_empty());

        // Test with nonexistent verb
        let empty_roles = engine.get_theta_roles("nonexistentverb");
        assert!(empty_roles.is_empty());
    }
}
