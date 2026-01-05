//! Pattern matcher for semantic-aware dependency matching.
//!
//! Matches parsed syntax against patterns from the treebank and synthesizes
//! patterns from `VerbNet` classes for unknown verbs.

use super::pattern_types::{ArgumentPosition, DependencyPattern, SemanticSignature};
use super::verbnet_patterns::synthesize_pattern;
use canopy::core::{DepRel, ThetaRole};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;

/// Pattern matcher with caching and `VerbNet` synthesis.
pub struct PatternMatcher {
    /// Patterns indexed by verb lemma (from treebank)
    lemma_patterns: HashMap<String, Vec<DependencyPattern>>,
    /// LRU cache for recent lookups
    cache: LruCache<String, DependencyPattern>,
    /// Statistics
    stats: MatcherStats,
}

/// Statistics for pattern matching performance.
#[derive(Debug, Default, Clone)]
pub struct MatcherStats {
    /// Cache hits
    pub cache_hits: u32,
    /// Cache misses
    pub cache_misses: u32,
    /// Lemma index hits
    pub lemma_hits: u32,
    /// `VerbNet` synthesis count
    pub verbnet_synth: u32,
    /// Default pattern fallbacks
    pub default_fallbacks: u32,
}

impl MatcherStats {
    /// Calculate cache hit rate.
    #[must_use]
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            f64::from(self.cache_hits) / f64::from(total)
        }
    }
}

impl PatternMatcher {
    /// Create a new pattern matcher with default cache size.
    #[must_use]
    pub fn new() -> Self {
        Self::with_cache_size(1000)
    }

    /// Create a pattern matcher with specified cache size.
    ///
    /// If size is 0, defaults to 1.
    #[must_use]
    pub fn with_cache_size(size: usize) -> Self {
        // SAFETY: 1 is non-zero, so this will never panic
        const ONE: NonZeroUsize = match NonZeroUsize::new(1) {
            Some(n) => n,
            None => unreachable!(),
        };
        let cache_size = NonZeroUsize::new(size).unwrap_or(ONE);
        Self {
            lemma_patterns: HashMap::new(),
            cache: LruCache::new(cache_size),
            stats: MatcherStats::default(),
        }
    }

    /// Add patterns from treebank data.
    pub fn add_treebank_patterns(&mut self, patterns: Vec<DependencyPattern>) {
        for pattern in patterns {
            self.lemma_patterns
                .entry(pattern.verb_lemma.clone())
                .or_default()
                .push(pattern);
        }
    }

    /// Add a single pattern.
    pub fn add_pattern(&mut self, pattern: DependencyPattern) {
        self.lemma_patterns
            .entry(pattern.verb_lemma.clone())
            .or_default()
            .push(pattern);
    }

    /// Match a semantic signature to get a dependency pattern.
    pub fn match_pattern(&mut self, signature: &SemanticSignature) -> Option<DependencyPattern> {
        let cache_key = Self::make_cache_key(signature);

        // 1. Check cache
        if let Some(pattern) = self.cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return Some(pattern.clone());
        }
        self.stats.cache_misses += 1;

        // 2. Look up by lemma in treebank patterns
        if let Some(patterns) = self.lemma_patterns.get(&signature.lemma) {
            if !patterns.is_empty() {
                self.stats.lemma_hits += 1;
                let best = Self::select_best_pattern(patterns, signature);
                self.cache.put(cache_key, best.clone());
                return Some(best);
            }
        }

        // 3. Synthesize from VerbNet class
        if let Some(class) = &signature.verbnet_class {
            if let Some(pattern) = synthesize_pattern(&signature.lemma, class) {
                self.stats.verbnet_synth += 1;
                self.cache.put(cache_key, pattern.clone());
                return Some(pattern);
            }
        }

        // 4. Fall back to basic SVO pattern
        self.stats.default_fallbacks += 1;
        let default = Self::default_pattern(&signature.lemma);
        self.cache.put(cache_key, default.clone());
        Some(default)
    }

    /// Get a pattern without modifying cache (for read-only access).
    #[must_use]
    pub fn get_pattern(&self, signature: &SemanticSignature) -> Option<DependencyPattern> {
        let cache_key = Self::make_cache_key(signature);

        // Check cache (peek doesn't update LRU order)
        if let Some(pattern) = self.cache.peek(&cache_key) {
            return Some(pattern.clone());
        }

        // Check lemma index
        if let Some(patterns) = self.lemma_patterns.get(&signature.lemma) {
            if !patterns.is_empty() {
                return Some(Self::select_best_pattern(patterns, signature));
            }
        }

        // Try VerbNet synthesis
        if let Some(class) = &signature.verbnet_class {
            if let Some(pattern) = synthesize_pattern(&signature.lemma, class) {
                return Some(pattern);
            }
        }

        // Default pattern
        Some(Self::default_pattern(&signature.lemma))
    }

    /// Get matcher statistics.
    #[must_use]
    pub fn stats(&self) -> &MatcherStats {
        &self.stats
    }

    /// Get the number of cached patterns.
    #[must_use]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Get the number of lemma patterns.
    #[must_use]
    pub fn lemma_pattern_count(&self) -> usize {
        self.lemma_patterns.values().map(Vec::len).sum()
    }

    /// Clear the cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Create cache key from signature.
    fn make_cache_key(signature: &SemanticSignature) -> String {
        match &signature.verbnet_class {
            Some(class) => format!("{}:{}", signature.lemma, class),
            None => signature.lemma.clone(),
        }
    }

    /// Select the best pattern from multiple candidates.
    fn select_best_pattern(
        patterns: &[DependencyPattern],
        signature: &SemanticSignature,
    ) -> DependencyPattern {
        // Prefer patterns with matching VerbNet class
        if let Some(class) = &signature.verbnet_class {
            if let Some(pattern) = patterns.iter().find(|p| {
                p.verbnet_class
                    .as_ref()
                    .is_some_and(|c| c == class || class.starts_with(c))
            }) {
                return pattern.clone();
            }
        }

        // Otherwise, select by frequency (most common pattern)
        patterns
            .iter()
            .max_by_key(|p| p.frequency)
            .cloned()
            .unwrap_or_else(|| Self::default_pattern(&signature.lemma))
    }

    /// Create a default SVO pattern for unknown verbs.
    fn default_pattern(lemma: &str) -> DependencyPattern {
        DependencyPattern::transitive(lemma).with_confidence(0.3)
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract dependency patterns from parsed syntax for indexing.
#[must_use]
pub fn extract_patterns_from_syntax(
    syntax: &canopy::runtime::AnnotatedSyntax,
) -> Vec<DependencyPattern> {
    use super::pattern_types::ArgumentPattern;

    let mut patterns = Vec::new();

    // Find all verbs
    for token in &syntax.tokens {
        if token.upos != canopy::core::UPos::Verb {
            continue;
        }

        let verb_id = token.id;
        let mut arguments = Vec::new();

        // Collect all dependents of this verb
        for dep_token in &syntax.tokens {
            if dep_token.head == Some(verb_id) && dep_token.id != verb_id {
                let position = if dep_token.id.0 < verb_id.0 {
                    ArgumentPosition::PreVerbal
                } else {
                    ArgumentPosition::PostVerbal
                };

                // Map dependency to theta role hint using UTAH
                let role_hint = dep_to_theta_role(&dep_token.deprel);

                arguments.push(ArgumentPattern {
                    dep_rel: dep_token.deprel.clone(),
                    role_hint,
                    position,
                    required: is_required_dep(&dep_token.deprel),
                });
            }
        }

        if !arguments.is_empty() {
            patterns.push(DependencyPattern::new(token.lemma.clone(), arguments));
        }
    }

    patterns
}

/// Map dependency relation to theta role hint (UTAH-inspired).
fn dep_to_theta_role(dep: &DepRel) -> Option<ThetaRole> {
    match *dep {
        DepRel::Nsubj | DepRel::NsubjPass => Some(ThetaRole::Agent),
        DepRel::Obj => Some(ThetaRole::Patient),
        DepRel::Iobj => Some(ThetaRole::Recipient),
        DepRel::Obl => Some(ThetaRole::Location), // Could also be Goal, Source, etc.
        DepRel::Ccomp | DepRel::Xcomp => Some(ThetaRole::Theme),
        _ => None,
    }
}

/// Check if a dependency is typically required.
fn is_required_dep(dep: &DepRel) -> bool {
    matches!(*dep, DepRel::Nsubj | DepRel::NsubjPass | DepRel::Obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matcher_new() {
        let matcher = PatternMatcher::new();
        assert_eq!(matcher.cache_size(), 0);
        assert_eq!(matcher.lemma_pattern_count(), 0);
    }

    #[test]
    fn test_add_pattern() {
        let mut matcher = PatternMatcher::new();
        let pattern = DependencyPattern::transitive("eat");
        matcher.add_pattern(pattern);

        assert_eq!(matcher.lemma_pattern_count(), 1);
    }

    #[test]
    fn test_match_pattern_default_fallback() {
        let mut matcher = PatternMatcher::new();
        let sig = SemanticSignature::from_lemma("unknown_verb");

        let pattern = matcher.match_pattern(&sig);
        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap().verb_lemma, "unknown_verb");
        assert_eq!(matcher.stats().default_fallbacks, 1);
    }

    #[test]
    fn test_match_pattern_lemma_hit() {
        let mut matcher = PatternMatcher::new();
        matcher.add_pattern(
            DependencyPattern::ditransitive("give")
                .with_verbnet_class("give-13.1")
                .with_frequency(100),
        );

        let sig = SemanticSignature::from_lemma("give");
        let pattern = matcher.match_pattern(&sig);

        assert!(pattern.is_some());
        let p = pattern.unwrap();
        assert_eq!(p.verb_lemma, "give");
        assert_eq!(matcher.stats().lemma_hits, 1);
    }

    #[test]
    fn test_match_pattern_cache_hit() {
        let mut matcher = PatternMatcher::new();
        let sig = SemanticSignature::from_lemma("test");

        // First call - cache miss
        let _ = matcher.match_pattern(&sig);
        assert_eq!(matcher.stats().cache_misses, 1);
        assert_eq!(matcher.stats().cache_hits, 0);

        // Second call - cache hit
        let _ = matcher.match_pattern(&sig);
        assert_eq!(matcher.stats().cache_hits, 1);
    }

    #[test]
    fn test_match_pattern_verbnet_synthesis() {
        let mut matcher = PatternMatcher::new();
        let sig = SemanticSignature::with_verbnet("donate", "give-13.1");

        let pattern = matcher.match_pattern(&sig);
        assert!(pattern.is_some());

        let p = pattern.unwrap();
        assert_eq!(p.verb_lemma, "donate");
        assert_eq!(p.verbnet_class, Some("give-13.1".to_string()));
        assert_eq!(matcher.stats().verbnet_synth, 1);
    }

    #[test]
    fn test_get_pattern_readonly() {
        let matcher = PatternMatcher::new();
        let sig = SemanticSignature::from_lemma("test");

        // get_pattern doesn't modify stats
        let pattern = matcher.get_pattern(&sig);
        assert!(pattern.is_some());
    }

    #[test]
    fn test_cache_key_with_verbnet() {
        let sig1 = SemanticSignature::from_lemma("give");
        let sig2 = SemanticSignature::with_verbnet("give", "give-13.1");

        let key1 = PatternMatcher::make_cache_key(&sig1);
        let key2 = PatternMatcher::make_cache_key(&sig2);

        assert_ne!(key1, key2);
        assert_eq!(key1, "give");
        assert_eq!(key2, "give:give-13.1");
    }

    #[test]
    fn test_stats_cache_hit_rate() {
        let mut stats = MatcherStats::default();
        assert!((stats.cache_hit_rate() - 0.0).abs() < f64::EPSILON);

        stats.cache_hits = 8;
        stats.cache_misses = 2;
        assert!((stats.cache_hit_rate() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_select_best_pattern_by_verbnet() {
        let mut matcher = PatternMatcher::new();

        // Add two patterns for "give"
        matcher.add_pattern(
            DependencyPattern::transitive("give")
                .with_verbnet_class("give-13.1")
                .with_frequency(50),
        );
        matcher.add_pattern(
            DependencyPattern::ditransitive("give")
                .with_verbnet_class("give-13.2")
                .with_frequency(100),
        );

        // Should prefer verbnet class match over frequency
        let sig = SemanticSignature::with_verbnet("give", "give-13.1");
        let pattern = matcher.match_pattern(&sig).unwrap();
        assert_eq!(pattern.verbnet_class, Some("give-13.1".to_string()));
    }

    #[test]
    fn test_dep_to_theta_role() {
        assert_eq!(dep_to_theta_role(&DepRel::Nsubj), Some(ThetaRole::Agent));
        assert_eq!(dep_to_theta_role(&DepRel::Obj), Some(ThetaRole::Patient));
        assert_eq!(dep_to_theta_role(&DepRel::Iobj), Some(ThetaRole::Recipient));
        assert_eq!(dep_to_theta_role(&DepRel::Punct), None);
    }

    #[test]
    fn test_is_required_dep() {
        assert!(is_required_dep(&DepRel::Nsubj));
        assert!(is_required_dep(&DepRel::Obj));
        assert!(!is_required_dep(&DepRel::Obl));
        assert!(!is_required_dep(&DepRel::Advmod));
    }
}
