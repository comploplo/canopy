//! `VerbNet` class to dependency pattern mappings.
//!
//! Maps `VerbNet` verb classes to expected dependency patterns based on
//! their argument structures. Uses UTAH (Universal Theta Assignment
//! Hypothesis) principles to suggest theta roles for each position.

use super::pattern_types::{ArgumentPattern, ArgumentPosition, DependencyPattern};
use canopy::core::{DepRel, ThetaRole};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Static mapping from `VerbNet` class IDs to their argument patterns.
///
/// Covers the ~50 most common verb classes that account for the majority
/// of English verbs. Patterns are based on `VerbNet` 3.4 frame definitions.
pub static VERBNET_PATTERNS: LazyLock<HashMap<&'static str, Vec<ArgumentPattern>>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // ============================================================
        // TRANSFER/GIVING VERBS (13.x)
        // ============================================================

        // give-13.1: NP V NP NP / NP V NP PP.recipient
        // "John gave Mary a book" / "John gave a book to Mary"
        m.insert(
            "give-13.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Iobj,
                    ThetaRole::Recipient,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Recipient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // send-11.1: NP V NP PP.destination
        // "John sent the package to Mary"
        m.insert(
            "send-11.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Goal,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // get-13.5.1: NP V NP (PP.source)
        // "John got a book (from Mary)"
        m.insert(
            "get-13.5.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Source,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // MOTION VERBS (51.x)
        // ============================================================

        // run-51.3.2: NP V (PP.location/direction)
        // "John ran (to the store)"
        m.insert(
            "run-51.3.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Theme,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Goal,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // walk-51.3.2: NP V (PP.location)
        m.insert(
            "walk-51.3.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Location,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // escape-51.1: NP V (PP.source)
        // "The prisoner escaped (from jail)"
        m.insert(
            "escape-51.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Theme,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Source,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // arrive-51.1: NP V (PP.location)
        m.insert(
            "arrive-51.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Theme,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Goal,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // PERCEPTION/COGNITION (30.x, 31.x)
        // ============================================================

        // see-30.1: NP V NP/S
        // "John saw Mary" / "John saw that it was raining"
        m.insert(
            "see-30.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Experiencer,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Stimulus,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Ccomp,
                    ThetaRole::Stimulus,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // peer-30.3: NP V (PP.location)
        // "John peered (through the window)"
        m.insert(
            "peer-30.3",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Location,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // admire-31.2: NP V NP
        // "John admires Mary"
        m.insert(
            "admire-31.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Experiencer,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Stimulus,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // amuse-31.1: NP V NP
        // Psych verbs with Stimulus subject: "The movie amused John"
        m.insert(
            "amuse-31.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Stimulus,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Experiencer,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // marvel-31.3: NP V PP.stimulus
        // "John marveled at the view"
        m.insert(
            "marvel-31.3",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Experiencer,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obl,
                    ThetaRole::Stimulus,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // COMMUNICATION (37.x)
        // ============================================================

        // say-37.7: NP V S/NP (PP.recipient)
        // "John said that..." / "John told Mary that..."
        m.insert(
            "say-37.7",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Ccomp,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Iobj,
                    ThetaRole::Recipient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // tell-37.2: NP V NP S/NP
        // "John told Mary the news"
        m.insert(
            "tell-37.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Iobj,
                    ThetaRole::Recipient,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Ccomp,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ask-37.1.2: NP V NP PP/S
        // "John asked Mary about the weather"
        m.insert(
            "ask-37.1.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Recipient,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // CREATION/DESTRUCTION (26.x, 44.x, 45.x)
        // ============================================================

        // build-26.1: NP V NP (PP.material)
        // "John built a house (from wood)"
        m.insert(
            "build-26.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Source,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // create-26.4: NP V NP
        // "John created a masterpiece"
        m.insert(
            "create-26.4",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // destroy-44: NP V NP
        // "The fire destroyed the building"
        m.insert(
            "destroy-44",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Patient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // break-45.1: NP V NP / NP V
        // "John broke the window" / "The window broke"
        m.insert(
            "break-45.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent, // or Theme in inchoative
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Patient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // CHANGE OF STATE (45.x)
        // ============================================================

        // other_cos-45.4: NP V NP / NP V
        // "John opened the door" / "The door opened"
        m.insert(
            "other_cos-45.4",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Patient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // PUTTING/PLACEMENT (9.x)
        // ============================================================

        // put-9.1: NP V NP PP.location
        // "John put the book on the table"
        m.insert(
            "put-9.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obl,
                    ThetaRole::Location,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // spray-9.7: NP V NP PP.location / NP V PP.location NP
        // "John sprayed paint on the wall" / "John sprayed the wall with paint"
        m.insert(
            "spray-9.7",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Location,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // fill-9.8: NP V NP PP.theme
        // "John filled the glass with water"
        m.insert(
            "fill-9.8",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Location,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // REMOVAL (10.x)
        // ============================================================

        // remove-10.1: NP V NP PP.source
        // "John removed the stain from the shirt"
        m.insert(
            "remove-10.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Source,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // clear-10.3: NP V NP (PP.theme)
        // "John cleared the table (of dishes)"
        m.insert(
            "clear-10.3",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Source,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // CONSUMPTION (39.x)
        // ============================================================

        // eat-39.1: NP V (NP)
        // "John ate (the pizza)"
        m.insert(
            "eat-39.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Patient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // devour-39.4: NP V NP
        // "John devoured the pizza" (obligatory object)
        m.insert(
            "devour-39.4",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Patient,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // CONTACT/IMPACT (18.x)
        // ============================================================

        // hit-18.1: NP V NP (PP.instrument)
        // "John hit the ball (with a bat)"
        m.insert(
            "hit-18.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Patient,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Instrument,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // touch-20: NP V NP
        // "John touched the wall"
        m.insert(
            "touch-20",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // SEARCHING (35.x)
        // ============================================================

        // search-35.2: NP V (NP/PP)
        // "John searched (the room) (for the key)"
        m.insert(
            "search-35.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Location,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // hunt-35.1: NP V (NP)
        // "John hunted (deer)"
        m.insert(
            "hunt-35.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // SOCIAL INTERACTION (36.x)
        // ============================================================

        // meet-36.3: NP V (NP)
        // "John met Mary" / "They met"
        m.insert(
            "meet-36.3",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // marry-36.2: NP V NP
        // "John married Mary"
        m.insert(
            "marry-36.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // POSSESSION (13.5.x)
        // ============================================================

        // obtain-13.5.2: NP V NP (PP.source)
        // "John obtained the document (from the office)"
        m.insert(
            "obtain-13.5.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Source,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // STATIVE VERBS (47.x)
        // ============================================================

        // exist-47.1: NP V (PP.location)
        // "The problem exists (in this area)"
        m.insert(
            "exist-47.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Theme,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obl,
                    ThetaRole::Location,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // ASPECTUAL VERBS (55.x)
        // ============================================================

        // begin-55.1: NP V (NP/VP)
        // "John began (the work)" / "John began working"
        m.insert(
            "begin-55.1",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Xcomp,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // complete-55.2: NP V NP
        // "John completed the task"
        m.insert(
            "complete-55.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Agent,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        // ============================================================
        // MEASURE (54.x)
        // ============================================================

        // cost-54.2: NP V NP (NP)
        // "The book cost (me) ten dollars"
        m.insert(
            "cost-54.2",
            vec![
                ArgumentPattern::required(
                    DepRel::Nsubj,
                    ThetaRole::Theme,
                    ArgumentPosition::PreVerbal,
                ),
                ArgumentPattern::required(
                    DepRel::Obj,
                    ThetaRole::Theme,
                    ArgumentPosition::PostVerbal,
                ),
                ArgumentPattern::optional(
                    DepRel::Iobj,
                    ThetaRole::Experiencer,
                    ArgumentPosition::PostVerbal,
                ),
            ],
        );

        m
    });

/// Get the pattern for a `VerbNet` class.
#[must_use]
pub fn get_verbnet_pattern(class_id: &str) -> Option<&'static Vec<ArgumentPattern>> {
    // Try exact match first
    if let Some(pattern) = VERBNET_PATTERNS.get(class_id) {
        return Some(pattern);
    }

    // Try base class (without subclass suffix)
    // e.g., "give-13.1-1" -> "give-13.1"
    let base = class_id.rsplit_once('-').map(|(base, _)| base)?;
    VERBNET_PATTERNS.get(base)
}

/// Create a `DependencyPattern` from `VerbNet` class patterns.
#[must_use]
pub fn synthesize_pattern(lemma: &str, class_id: &str) -> Option<DependencyPattern> {
    let args = get_verbnet_pattern(class_id)?;
    Some(
        DependencyPattern::new(lemma.to_string(), args.clone())
            .with_verbnet_class(class_id)
            .with_confidence(0.8),
    )
}

/// Get the number of defined `VerbNet` patterns.
#[must_use]
pub fn pattern_count() -> usize {
    VERBNET_PATTERNS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_give_pattern() {
        let pattern = get_verbnet_pattern("give-13.1");
        assert!(pattern.is_some());

        let args = pattern.unwrap();
        assert!(args.iter().any(|a| a.dep_rel == DepRel::Nsubj));
        assert!(args.iter().any(|a| a.dep_rel == DepRel::Obj));
    }

    #[test]
    fn test_run_pattern() {
        let pattern = get_verbnet_pattern("run-51.3.2");
        assert!(pattern.is_some());

        let args = pattern.unwrap();
        let nsubj = args.iter().find(|a| a.dep_rel == DepRel::Nsubj);
        assert!(nsubj.is_some());
        assert_eq!(nsubj.unwrap().role_hint, Some(ThetaRole::Theme));
    }

    #[test]
    fn test_psych_verb_pattern() {
        // amuse-31.1 has Stimulus as subject (object experiencer)
        let pattern = get_verbnet_pattern("amuse-31.1");
        assert!(pattern.is_some());

        let args = pattern.unwrap();
        let nsubj = args.iter().find(|a| a.dep_rel == DepRel::Nsubj);
        assert_eq!(nsubj.unwrap().role_hint, Some(ThetaRole::Stimulus));
    }

    #[test]
    fn test_synthesize_pattern() {
        let pattern = synthesize_pattern("donate", "give-13.1");
        assert!(pattern.is_some());

        let p = pattern.unwrap();
        assert_eq!(p.verb_lemma, "donate");
        assert_eq!(p.verbnet_class, Some("give-13.1".to_string()));
        assert!((p.confidence - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_base_class_fallback() {
        // Try a subclass that should fall back to base
        let pattern = get_verbnet_pattern("give-13.1-1");
        assert!(pattern.is_some()); // Should find "give-13.1"
    }

    #[test]
    fn test_pattern_count() {
        let count = pattern_count();
        assert!(count >= 30, "Should have at least 30 patterns, got {count}");
    }

    #[test]
    fn test_unknown_class() {
        let pattern = get_verbnet_pattern("nonexistent-999.999");
        assert!(pattern.is_none());
    }
}
