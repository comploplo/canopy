//! Binding theory tests
//!
//! Tests for the modern binding theory implementation based on:
//! - Reinhart & Reuland (1993) "Reflexivity"
//! - Reuland (2011) "Anaphora and Language Design"
//! - Charnavel (2019) "Locality and Logophoricity"

use canopy_discourse::{
    classify_anaphor, is_personal_pronoun, is_pronoun, is_self_anaphor, AnaphorType,
    ConditionBResult, DiscourseContext, Gender, GenderLookup, LogophoricContext,
    LogophoricDetector, NumberFeature, Person, PredicateAnalyzer, ReferentId, ReferentType,
};

// ============================================================================
// Anaphor Classification Tests
// ============================================================================

#[test]
fn test_self_anaphor_classification() {
    // SELF-anaphors (reflexives)
    let himself = classify_anaphor("himself");
    assert_eq!(himself.anaphor_type, AnaphorType::SelfAnaphor);
    assert_eq!(himself.person, Some(Person::Third));
    assert_eq!(himself.gender, Some(Gender::Masculine));
    assert_eq!(himself.number, Some(NumberFeature::Singular));

    let herself = classify_anaphor("herself");
    assert_eq!(herself.anaphor_type, AnaphorType::SelfAnaphor);
    assert_eq!(herself.gender, Some(Gender::Feminine));

    let themselves = classify_anaphor("themselves");
    assert_eq!(themselves.anaphor_type, AnaphorType::SelfAnaphor);
    assert_eq!(themselves.number, Some(NumberFeature::Plural));
    assert_eq!(themselves.gender, None); // Can be any gender

    // First person
    let myself = classify_anaphor("myself");
    assert_eq!(myself.person, Some(Person::First));
    assert_eq!(myself.anaphor_type, AnaphorType::SelfAnaphor);

    // Second person
    let yourself = classify_anaphor("yourself");
    assert_eq!(yourself.person, Some(Person::Second));
    assert_eq!(yourself.anaphor_type, AnaphorType::SelfAnaphor);
}

#[test]
fn test_personal_pronoun_classification() {
    let he = classify_anaphor("he");
    assert_eq!(he.anaphor_type, AnaphorType::Personal);
    assert_eq!(he.gender, Some(Gender::Masculine));
    assert_eq!(he.number, Some(NumberFeature::Singular));

    let she = classify_anaphor("she");
    assert_eq!(she.anaphor_type, AnaphorType::Personal);
    assert_eq!(she.gender, Some(Gender::Feminine));

    let it = classify_anaphor("it");
    assert_eq!(it.anaphor_type, AnaphorType::Personal);
    assert_eq!(it.gender, Some(Gender::Neuter));

    // Singular they (underspecified)
    let they = classify_anaphor("they");
    assert_eq!(they.anaphor_type, AnaphorType::Personal);
    assert_eq!(they.gender, None); // Can be any gender
    assert_eq!(they.number, None); // Can be singular or plural
}

#[test]
fn test_possessive_classification() {
    let his = classify_anaphor("his");
    assert_eq!(his.anaphor_type, AnaphorType::Possessive);
    assert_eq!(his.gender, Some(Gender::Masculine));

    let her_poss = classify_anaphor("hers");
    assert_eq!(her_poss.anaphor_type, AnaphorType::Possessive);
    assert_eq!(her_poss.gender, Some(Gender::Feminine));

    let their = classify_anaphor("their");
    assert_eq!(their.anaphor_type, AnaphorType::Possessive);
    assert_eq!(their.gender, None); // Underspecified
}

#[test]
fn test_non_pronoun_classification() {
    let john = classify_anaphor("John");
    assert_eq!(john.anaphor_type, AnaphorType::None);
    assert_eq!(john.gender, None);

    let table = classify_anaphor("table");
    assert_eq!(table.anaphor_type, AnaphorType::None);
}

#[test]
fn test_helper_functions() {
    assert!(is_self_anaphor("himself"));
    assert!(is_self_anaphor("herself"));
    assert!(is_self_anaphor("themselves"));
    assert!(!is_self_anaphor("him"));
    assert!(!is_self_anaphor("John"));

    assert!(is_personal_pronoun("he"));
    assert!(is_personal_pronoun("she"));
    assert!(is_personal_pronoun("they"));
    assert!(!is_personal_pronoun("himself"));
    assert!(!is_personal_pronoun("Mary"));

    assert!(is_pronoun("he"));
    assert!(is_pronoun("himself"));
    assert!(is_pronoun("their"));
    assert!(!is_pronoun("John"));
}

// ============================================================================
// Condition B Tests (Reinhart & Reuland)
// ============================================================================

#[test]
fn test_condition_b_reflexive_marked() {
    let analyzer = PredicateAnalyzer::new();
    let john_id = ReferentId(1);

    // "John criticized himself" → valid (reflexive-marked by SELF-anaphor)
    let result = analyzer.check_condition_b("criticize", john_id, john_id, "himself");
    assert!(result.is_valid());
    assert!(matches!(result, ConditionBResult::Valid { .. }));
}

#[test]
fn test_condition_b_violation() {
    let analyzer = PredicateAnalyzer::new();
    let john_id = ReferentId(1);

    // "John criticized him" where him=John → VIOLATION
    // Predicate would be reflexive but not reflexive-marked
    let result = analyzer.check_condition_b("criticize", john_id, john_id, "him");
    assert!(result.is_violation());
}

#[test]
fn test_condition_b_intrinsically_reflexive() {
    let analyzer = PredicateAnalyzer::new();
    let john_id = ReferentId(1);

    // "John washed" (him=John implicit) → valid because "wash" is intrinsically reflexive
    let result = analyzer.check_condition_b("wash", john_id, john_id, "him");
    assert!(result.is_valid());

    // Same for shave, dress, etc.
    let result = analyzer.check_condition_b("shave", john_id, john_id, "");
    assert!(result.is_valid());
}

#[test]
fn test_condition_b_no_coreference() {
    let analyzer = PredicateAnalyzer::new();
    let john_id = ReferentId(1);
    let bill_id = ReferentId(2);

    // "John criticized him" where him=Bill → not applicable (no coreference)
    let result = analyzer.check_condition_b("criticize", john_id, bill_id, "him");
    assert!(matches!(result, ConditionBResult::NotApplicable));
}

#[test]
fn test_can_corefer_as_coarguments() {
    let analyzer = PredicateAnalyzer::new();

    // SELF-anaphors CAN co-refer with co-arguments (that's their job)
    assert!(analyzer.can_corefer_as_coarguments("criticize", "himself"));
    assert!(analyzer.can_corefer_as_coarguments("hit", "herself"));

    // Personal pronouns CANNOT co-refer with co-arguments (regular predicates)
    assert!(!analyzer.can_corefer_as_coarguments("criticize", "him"));
    assert!(!analyzer.can_corefer_as_coarguments("hit", "her"));

    // Personal pronouns CAN for intrinsically reflexive predicates
    assert!(analyzer.can_corefer_as_coarguments("wash", "him"));
    assert!(analyzer.can_corefer_as_coarguments("shave", "her"));
}

// ============================================================================
// Logophoric Context Tests (Charnavel)
// ============================================================================

#[test]
fn test_logophoric_attitude_holder() {
    let detector = LogophoricDetector::new();
    let john_id = ReferentId(1);

    // "John thinks [that Mary likes himself]"
    // John is the attitude holder whose perspective licenses "himself"
    let ctx = detector.detect("like", None, None, true, Some(john_id));

    assert!(ctx.is_logophoric());
    assert!(matches!(ctx, LogophoricContext::AttitudeHolder { holder, .. } if holder == john_id));
    assert_eq!(ctx.perspective_center(), Some(john_id));
}

#[test]
fn test_logophoric_empathy_locus() {
    let detector = LogophoricDetector::new();
    let john_id = ReferentId(1);

    // "Pictures of himself bother John"
    // John is the experiencer whose empathy licenses "himself"
    let ctx = detector.detect("bother", None, Some(john_id), false, None);

    assert!(ctx.is_logophoric());
    assert!(
        matches!(ctx, LogophoricContext::EmpathyLocus { experiencer, .. } if experiencer == john_id)
    );
}

#[test]
fn test_no_logophoric_context() {
    let detector = LogophoricDetector::new();

    // Regular predicate with no attitude or experiencer structure
    let ctx = detector.detect("run", Some(ReferentId(1)), None, false, None);
    assert!(!ctx.is_logophoric());
    assert!(matches!(ctx, LogophoricContext::None));
}

#[test]
fn test_logophoric_binding() {
    let detector = LogophoricDetector::new();
    let john_id = ReferentId(1);
    let mary_id = ReferentId(2);

    let ctx = LogophoricContext::AttitudeHolder {
        holder: john_id,
        verb: "think".to_string(),
    };

    // John (attitude holder) can bind logophorically
    assert!(detector.can_bind_logophorically(&ctx, john_id));
    // Mary (not attitude holder) cannot
    assert!(!detector.can_bind_logophorically(&ctx, mary_id));
}

// ============================================================================
// Gender Agreement Tests
// ============================================================================

#[test]
fn test_gender_from_name_lookup() {
    let lookup = GenderLookup::global();

    // Common names should have gender
    // Note: May fail if dataset not loaded (e.g., in CI)
    if !lookup.is_empty() {
        assert_eq!(lookup.infer("john"), Some(Gender::Masculine));
        assert_eq!(lookup.infer("mary"), Some(Gender::Feminine));
        assert_eq!(lookup.infer("michael"), Some(Gender::Masculine));
        assert_eq!(lookup.infer("sarah"), Some(Gender::Feminine));
    }
}

#[test]
fn test_gender_case_insensitive() {
    let lookup = GenderLookup::global();

    if !lookup.is_empty() {
        // Should work regardless of case
        assert_eq!(lookup.infer("JOHN"), Some(Gender::Masculine));
        assert_eq!(lookup.infer("Mary"), Some(Gender::Feminine));
        assert_eq!(lookup.infer("jOhN"), Some(Gender::Masculine));
    }
}

// ============================================================================
// Integration Tests: Full Anaphora Resolution
// ============================================================================

#[test]
fn test_resolve_anaphor_self_with_coargument() {
    let mut ctx = DiscourseContext::with_defaults();
    ctx.begin_sentence("John criticized himself.".to_string());

    // Introduce John
    let john_id = ctx
        .introduce_referent("John".to_string(), ReferentType::Individual)
        .unwrap();

    // Note: In real usage, gender would be set from the name dataset
    // when the referent is created

    // "himself" should resolve to John (co-argument)
    let result = ctx.resolve_anaphor("himself", "criticize", &[john_id], None);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), john_id);
}

#[test]
fn test_resolve_anaphor_personal_blocked_by_condition_b() {
    let mut ctx = DiscourseContext::with_defaults();
    ctx.begin_sentence("John criticized him.".to_string());

    // Introduce John
    let john_id = ctx
        .introduce_referent("John".to_string(), ReferentType::Individual)
        .unwrap();

    // "him" should NOT resolve to John (Condition B blocks co-argument binding)
    let result = ctx.resolve_anaphor("him", "criticize", &[john_id], None);
    // Should fail because the only candidate (John) is blocked
    assert!(result.is_err());
}

#[test]
fn test_resolve_anaphor_personal_across_clause() {
    let mut ctx = DiscourseContext::with_defaults();
    ctx.begin_sentence("John said Mary criticized him.".to_string());

    // Introduce John (matrix subject)
    let john_id = ctx
        .introduce_referent("John".to_string(), ReferentType::Individual)
        .unwrap();

    // Note: Gender would be set from name dataset in real usage

    // Introduce Mary (embedded subject)
    let mary_id = ctx
        .introduce_referent("Mary".to_string(), ReferentType::Individual)
        .unwrap();

    // "him" in embedded clause - Mary is the co-argument, John is not
    // Should resolve to John (not blocked by Condition B since John is not co-argument)
    let result = ctx.resolve_anaphor("him", "criticize", &[mary_id], None);
    // This should succeed and resolve to John
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), john_id);
}

#[test]
fn test_resolve_anaphor_with_logophoric_context() {
    let mut ctx = DiscourseContext::with_defaults();
    ctx.begin_sentence("John thinks Mary likes himself.".to_string());

    // Introduce John (attitude holder)
    let john_id = ctx
        .introduce_referent("John".to_string(), ReferentType::Individual)
        .unwrap();

    // Introduce Mary (embedded subject) - unused in this test since we force logophoric path
    let _mary_id = ctx
        .introduce_referent("Mary".to_string(), ReferentType::Individual)
        .unwrap();

    // Create logophoric context (John is attitude holder)
    let logo_ctx = LogophoricContext::AttitudeHolder {
        holder: john_id,
        verb: "think".to_string(),
    };

    // "himself" in embedded clause with John as attitude holder
    // When there are no local co-argument matches, the logophoric reading kicks in
    // and resolves to John (the attitude holder/perspective center)
    let result = ctx.resolve_anaphor("himself", "like", &[], Some(&logo_ctx));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), john_id);
}

#[test]
fn test_resolve_anaphor_self_without_antecedent_fails() {
    let mut ctx = DiscourseContext::with_defaults();
    ctx.begin_sentence("Himself runs.".to_string());

    // Don't introduce any referents - "himself" has no possible antecedent

    // "himself" without any co-argument or logophoric center should fail
    let result = ctx.resolve_anaphor("himself", "run", &[], None);
    // Should fail - no valid antecedent
    assert!(result.is_err());
}
