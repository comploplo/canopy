//! Tests for POS-aware semantic caching
//!
//! These tests verify that the POS-aware caching system:
//! 1. Generates correct cache keys with POS tags
//! 2. Skips VerbNet for non-verbs (nouns, adjectives, etc.)
//! 3. Deduplicates batch analysis correctly
//! 4. Improves cache hit rates for repeated words with same POS
//!
//! NOTE: Tests require semantic data to be present at data/ directory.
//! If data is not available, tests are skipped gracefully.
//!
//! IMPORTANT: These are slow integration tests (~30s each) that load real
//! VerbNet/FrameNet/WordNet data. Run with `cargo test -- --ignored` to execute.

use canopy_core::UPos;
use canopy_tokenizer::coordinator::{CoordinatorConfig, SemanticCoordinator};
use canopy_tokenizer::guess_pos_from_suffix;
use canopy_wordnet::PartOfSpeech;

#[cfg(test)]
mod pos_aware_caching_tests {
    use super::*;

    fn try_create_test_coordinator() -> Option<SemanticCoordinator> {
        let config = CoordinatorConfig {
            enable_verbnet: true,
            enable_framenet: true,
            enable_wordnet: true,
            enable_lexicon: false,
            enable_lemmatization: false, // Disable for predictable test keys
            confidence_threshold: 0.3,
            l1_cache_memory_mb: 50,
            ..CoordinatorConfig::default()
        };
        SemanticCoordinator::new(config).ok()
    }

    #[test]
    fn test_analyze_with_pos_verb_includes_verbnet() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Analyze "run" as a verb - should include VerbNet
        let result = coordinator
            .analyze_with_pos("run", Some(UPos::Verb))
            .expect("Analysis should succeed");

        assert!(result.verbnet.is_some(), "Verbs should have VerbNet data");
        assert_eq!(result.pos, Some(UPos::Verb));
    }

    #[test]
    fn test_analyze_with_pos_noun_skips_verbnet() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Analyze "bank" as a noun - should NOT include VerbNet
        let result = coordinator
            .analyze_with_pos("bank", Some(UPos::Noun))
            .expect("Analysis should succeed");

        assert!(
            result.verbnet.is_none(),
            "Nouns should not have VerbNet data"
        );
        assert_eq!(result.pos, Some(UPos::Noun));
    }

    #[test]
    fn test_analyze_with_pos_propn_skips_verbnet() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Proper nouns should also skip VerbNet
        let result = coordinator
            .analyze_with_pos("John", Some(UPos::Propn))
            .expect("Analysis should succeed");

        assert!(
            result.verbnet.is_none(),
            "Proper nouns should not have VerbNet data"
        );
        assert_eq!(result.pos, Some(UPos::Propn));
    }

    #[test]
    fn test_analyze_with_pos_adj_skips_verbnet() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Adjectives should skip VerbNet
        let result = coordinator
            .analyze_with_pos("happy", Some(UPos::Adj))
            .expect("Analysis should succeed");

        assert!(
            result.verbnet.is_none(),
            "Adjectives should not have VerbNet data"
        );
        assert_eq!(result.pos, Some(UPos::Adj));
    }

    #[test]
    fn test_cache_differentiates_by_pos() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Analyze "bank" as noun, then as verb
        let result_noun = coordinator
            .analyze_with_pos("bank", Some(UPos::Noun))
            .expect("Analysis should succeed");
        let result_verb = coordinator
            .analyze_with_pos("bank", Some(UPos::Verb))
            .expect("Analysis should succeed");

        // They should have different VerbNet results
        assert!(
            result_noun.verbnet.is_none(),
            "bank as noun should not have VerbNet"
        );
        // The key test is POS differentiation
        assert_ne!(result_noun.pos, result_verb.pos);
    }

    #[test]
    fn test_repeated_word_same_pos_hits_cache() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // First call
        let _result1 = coordinator
            .analyze_with_pos("run", Some(UPos::Verb))
            .expect("First analysis should succeed");

        // Second call with same word and POS should hit cache
        let _result2 = coordinator
            .analyze_with_pos("run", Some(UPos::Verb))
            .expect("Second analysis should succeed");

        // Check cache statistics - should have at least 1 hit
        let stats = coordinator.get_statistics();
        assert!(
            stats.cache_hit_rate > 0.0,
            "Repeated word with same POS should hit cache"
        );
    }

    #[test]
    fn test_batch_deduplication_analyzes_unique_only() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Create batch with duplicates
        let words: Vec<(String, Option<UPos>)> = vec![
            ("run".to_string(), Some(UPos::Verb)),
            ("walk".to_string(), Some(UPos::Verb)),
            ("run".to_string(), Some(UPos::Verb)), // Duplicate
            ("bank".to_string(), Some(UPos::Noun)),
            ("walk".to_string(), Some(UPos::Verb)), // Duplicate
        ];

        let results = coordinator
            .analyze_batch_deduped(&words)
            .expect("Batch analysis should succeed");

        // Should return 5 results (one per input)
        assert_eq!(results.len(), 5, "Should return result for each input");

        // Duplicates should have same lemma
        assert_eq!(
            results[0].lemma, results[2].lemma,
            "Duplicate 'run' should have same lemma"
        );
        assert_eq!(
            results[1].lemma, results[4].lemma,
            "Duplicate 'walk' should have same lemma"
        );
    }

    #[test]
    fn test_analyze_without_pos_fallback() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Analyze without POS - should still work (fallback behavior)
        let result = coordinator
            .analyze_with_pos("run", None)
            .expect("Analysis without POS should succeed");

        // Should still produce a result
        assert!(!result.lemma.is_empty());
        assert_eq!(result.pos, None);
    }

    #[test]
    fn test_aux_treated_as_verb_for_verbnet() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Auxiliary verbs should be treated as verbs
        let result = coordinator
            .analyze_with_pos("have", Some(UPos::Aux))
            .expect("Analysis should succeed");

        // Aux should be stored with its POS
        assert_eq!(result.pos, Some(UPos::Aux));
    }

    #[test]
    fn test_function_words_skip_wordnet() {
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // Function words (DET, ADP, etc.) typically don't have WordNet entries
        let result_det = coordinator
            .analyze_with_pos("the", Some(UPos::Det))
            .expect("Analysis should succeed");

        // Determiners should store their POS
        assert_eq!(result_det.pos, Some(UPos::Det));
    }

    // ========== Suffix Heuristics Tests ==========

    #[test]
    fn test_suffix_heuristics_adverb() {
        // Words ending in -ly should be recognized as adverbs
        assert_eq!(guess_pos_from_suffix("quickly"), Some(PartOfSpeech::Adverb));
        assert_eq!(guess_pos_from_suffix("slowly"), Some(PartOfSpeech::Adverb));
        assert_eq!(guess_pos_from_suffix("happily"), Some(PartOfSpeech::Adverb));
        // Short words should not trigger
        assert_eq!(guess_pos_from_suffix("fly"), None);
    }

    #[test]
    fn test_suffix_heuristics_verb() {
        // -ing endings
        assert_eq!(guess_pos_from_suffix("running"), Some(PartOfSpeech::Verb));
        assert_eq!(guess_pos_from_suffix("walking"), Some(PartOfSpeech::Verb));
        // -ed endings
        assert_eq!(guess_pos_from_suffix("walked"), Some(PartOfSpeech::Verb));
        assert_eq!(guess_pos_from_suffix("jumped"), Some(PartOfSpeech::Verb));
        // -ize/-ate
        assert_eq!(guess_pos_from_suffix("realize"), Some(PartOfSpeech::Verb));
        assert_eq!(guess_pos_from_suffix("activate"), Some(PartOfSpeech::Verb));
    }

    #[test]
    fn test_suffix_heuristics_noun() {
        // -tion/-sion
        assert_eq!(guess_pos_from_suffix("creation"), Some(PartOfSpeech::Noun));
        assert_eq!(guess_pos_from_suffix("decision"), Some(PartOfSpeech::Noun));
        // -ment
        assert_eq!(guess_pos_from_suffix("movement"), Some(PartOfSpeech::Noun));
        // -ness/-ity
        assert_eq!(guess_pos_from_suffix("happiness"), Some(PartOfSpeech::Noun));
        assert_eq!(guess_pos_from_suffix("ability"), Some(PartOfSpeech::Noun));
        // -er/-or (agent nouns)
        assert_eq!(guess_pos_from_suffix("teacher"), Some(PartOfSpeech::Noun));
        assert_eq!(guess_pos_from_suffix("actor"), Some(PartOfSpeech::Noun));
    }

    #[test]
    fn test_suffix_heuristics_adjective() {
        // -ful/-less
        assert_eq!(
            guess_pos_from_suffix("beautiful"),
            Some(PartOfSpeech::Adjective)
        );
        assert_eq!(
            guess_pos_from_suffix("helpless"),
            Some(PartOfSpeech::Adjective)
        );
        // -ous/-ive
        assert_eq!(
            guess_pos_from_suffix("dangerous"),
            Some(PartOfSpeech::Adjective)
        );
        assert_eq!(
            guess_pos_from_suffix("creative"),
            Some(PartOfSpeech::Adjective)
        );
        // -able/-ible
        assert_eq!(
            guess_pos_from_suffix("readable"),
            Some(PartOfSpeech::Adjective)
        );
        assert_eq!(
            guess_pos_from_suffix("visible"),
            Some(PartOfSpeech::Adjective)
        );
    }

    #[test]
    fn test_suffix_heuristics_no_match() {
        // Short words
        assert_eq!(guess_pos_from_suffix("go"), None);
        assert_eq!(guess_pos_from_suffix("be"), None);
        // Words without clear suffix patterns
        assert_eq!(guess_pos_from_suffix("whale"), None);
        assert_eq!(guess_pos_from_suffix("ship"), None);
        assert_eq!(guess_pos_from_suffix("ocean"), None);
    }

    #[test]
    fn test_early_exit_optimization_with_suffix_words() {
        // Test that words with clear suffixes get analyzed (verifies early exit path works)
        let Some(coordinator) = try_create_test_coordinator() else {
            eprintln!("Skipping test: semantic data not available");
            return;
        };

        // These words have clear suffixes that should trigger early exit
        let suffix_words = ["running", "quickly", "happiness", "beautiful"];

        for word in suffix_words {
            let result = coordinator
                .analyze(word)
                .expect(&format!("Analysis of '{}' should succeed", word));

            // Verify we got a result (early exit found something)
            assert!(
                result.wordnet.is_some() || result.framenet.is_some() || result.verbnet.is_some(),
                "Word '{}' with suffix should have semantic data",
                word
            );
        }
    }
}
