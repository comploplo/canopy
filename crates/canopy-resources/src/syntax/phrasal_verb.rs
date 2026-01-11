//! Phrasal verb detection.
//!
//! Detects verb+particle combinations using `compound:prt` dependency relations
//! and creates combined lemmas for semantic lookup.

use canopy::core::DepRel;
use canopy::runtime::{AnnotatedSyntax, TokenId};

/// Detects phrasal verbs by finding `compound:prt` relations.
///
/// Phrasal verbs are multi-word predicates where a verb combines with
/// a particle to form a new meaning (e.g., "give up", "turn off").
#[derive(Debug, Default)]
pub struct PhrasalVerbDetector;

impl PhrasalVerbDetector {
    /// Create a new phrasal verb detector.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Find all particles attached to a verb.
    ///
    /// Returns token IDs of particles connected via `compound:prt`.
    #[must_use]
    pub fn find_particles(&self, syntax: &AnnotatedSyntax, verb_id: TokenId) -> Vec<TokenId> {
        syntax
            .tokens
            .iter()
            .filter(|t| t.head == Some(verb_id) && t.deprel == DepRel::CompoundPrt)
            .map(|t| t.id)
            .collect()
    }

    /// Check if a verb has any attached particles.
    #[must_use]
    pub fn has_particle(&self, syntax: &AnnotatedSyntax, verb_id: TokenId) -> bool {
        syntax
            .tokens
            .iter()
            .any(|t| t.head == Some(verb_id) && t.deprel == DepRel::CompoundPrt)
    }

    /// Create a VerbNet-style combined lemma for lookup.
    ///
    /// Combines verb lemma with particle(s) using underscore:
    /// - "give" + "up" → "give\_up"
    /// - "look" + "forward" + "to" → "look\_forward\_to"
    #[must_use]
    pub fn phrasal_lemma(&self, syntax: &AnnotatedSyntax, verb_id: TokenId) -> String {
        let Some(verb) = syntax.get_token(verb_id) else {
            return String::new();
        };

        let mut particles = self.find_particles(syntax, verb_id);

        if particles.is_empty() {
            return verb.lemma.clone();
        }

        // Sort particles by position to maintain word order
        particles.sort_by_key(|t| t.index());

        let mut parts = vec![verb.lemma.clone()];
        for particle_id in particles {
            if let Some(particle) = syntax.get_token(particle_id) {
                parts.push(particle.lemma.clone());
            }
        }

        parts.join("_")
    }

    /// Get the combined span covering verb and all particles.
    ///
    /// Returns `(start, end)` byte offsets in the original text.
    #[must_use]
    pub fn phrasal_span(
        &self,
        syntax: &AnnotatedSyntax,
        verb_id: TokenId,
    ) -> Option<(usize, usize)> {
        let verb = syntax.get_token(verb_id)?;
        let particles = self.find_particles(syntax, verb_id);

        if particles.is_empty() {
            return Some(verb.span);
        }

        let mut all_tokens = vec![verb_id];
        all_tokens.extend(particles);

        let min_start = all_tokens
            .iter()
            .filter_map(|&id| syntax.get_token(id))
            .map(|t| t.span.0)
            .min()?;

        let max_end = all_tokens
            .iter()
            .filter_map(|&id| syntax.get_token(id))
            .map(|t| t.span.1)
            .max()?;

        Some((min_start, max_end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canopy::core::{MorphFeatures, UPos};
    use canopy::runtime::AnnotatedToken;

    fn make_token(
        id: usize,
        form: &str,
        lemma: &str,
        upos: UPos,
        head: Option<usize>,
        deprel: DepRel,
    ) -> AnnotatedToken {
        AnnotatedToken {
            id: TokenId::new(id),
            form: form.to_string(),
            lemma: lemma.to_string(),
            upos,
            xpos: None,
            feats: MorphFeatures::default(),
            head: head.map(TokenId::new),
            deprel,
            span: (0, form.len()),
        }
    }

    #[test]
    fn test_find_particles() {
        // "He gave up the fight"
        let syntax = AnnotatedSyntax::new(
            "He gave up the fight".to_string(),
            vec![
                make_token(0, "He", "he", UPos::Pron, Some(1), DepRel::Nsubj),
                make_token(1, "gave", "give", UPos::Verb, None, DepRel::Root),
                make_token(2, "up", "up", UPos::Part, Some(1), DepRel::CompoundPrt),
                make_token(3, "the", "the", UPos::Det, Some(4), DepRel::Det),
                make_token(4, "fight", "fight", UPos::Noun, Some(1), DepRel::Obj),
            ],
        );

        let detector = PhrasalVerbDetector::new();
        let particles = detector.find_particles(&syntax, TokenId::new(1));

        assert_eq!(particles.len(), 1);
        assert_eq!(particles[0], TokenId::new(2));
    }

    #[test]
    fn test_phrasal_lemma() {
        let syntax = AnnotatedSyntax::new(
            "He gave up".to_string(),
            vec![
                make_token(0, "He", "he", UPos::Pron, Some(1), DepRel::Nsubj),
                make_token(1, "gave", "give", UPos::Verb, None, DepRel::Root),
                make_token(2, "up", "up", UPos::Part, Some(1), DepRel::CompoundPrt),
            ],
        );

        let detector = PhrasalVerbDetector::new();
        let lemma = detector.phrasal_lemma(&syntax, TokenId::new(1));

        assert_eq!(lemma, "give_up");
    }

    #[test]
    fn test_no_particle() {
        let syntax = AnnotatedSyntax::new(
            "He runs".to_string(),
            vec![
                make_token(0, "He", "he", UPos::Pron, Some(1), DepRel::Nsubj),
                make_token(1, "runs", "run", UPos::Verb, None, DepRel::Root),
            ],
        );

        let detector = PhrasalVerbDetector::new();
        let lemma = detector.phrasal_lemma(&syntax, TokenId::new(1));

        assert_eq!(lemma, "run");
        assert!(!detector.has_particle(&syntax, TokenId::new(1)));
    }

    #[test]
    fn test_has_particle() {
        let syntax = AnnotatedSyntax::new(
            "Turn off the light".to_string(),
            vec![
                make_token(0, "Turn", "turn", UPos::Verb, None, DepRel::Root),
                make_token(1, "off", "off", UPos::Part, Some(0), DepRel::CompoundPrt),
                make_token(2, "the", "the", UPos::Det, Some(3), DepRel::Det),
                make_token(3, "light", "light", UPos::Noun, Some(0), DepRel::Obj),
            ],
        );

        let detector = PhrasalVerbDetector::new();
        assert!(detector.has_particle(&syntax, TokenId::new(0)));
    }
}
