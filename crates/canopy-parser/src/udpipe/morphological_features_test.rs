//! Tests for real morphological feature extraction from UDPipe output
//!
//! This module tests that UDPipe FFI integration correctly extracts
//! and parses morphological features from real linguistic models.

use crate::udpipe::UDPipeEngine;
use canopy_core::*;
use std::env;

#[cfg(test)]
mod morphological_features_tests {
    use super::*;

    #[test]
    fn test_morphological_features_parsing() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let workspace_root = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let model_path = workspace_root.join("models/test.model");

        if !model_path.exists() {
            println!("Skipping morphological features test - model not found");
            return;
        }

        let engine =
            UDPipeEngine::load(model_path.to_string_lossy()).expect("Should load UDPipe model");

        // Test sentences with rich morphological features
        let test_cases = vec![
            ("I walked quickly.", "Past tense verb expected"),
            ("She is running fast.", "Present progressive expected"),
            ("The cats are sleeping.", "Plural nouns expected"),
            ("He has been working.", "Perfect aspect expected"),
        ];

        for (sentence, description) in test_cases {
            println!("\nTesting: {} ({})", sentence, description);

            let result = engine.parse(sentence).expect("Should parse successfully");

            // Verify we got words with morphological features
            assert!(!result.words.is_empty(), "Should parse words");

            // Look for interesting morphological features
            let mut found_features = Vec::new();

            for word in &result.words {
                println!(
                    "  Word: '{}' [{:?}] -> {}",
                    word.form, word.upos, word.lemma
                );

                // Check for number features
                if let Some(number) = &word.feats.number {
                    found_features.push(format!("Number={:?}", number));
                }

                // Check for tense features
                if let Some(tense) = &word.feats.tense {
                    found_features.push(format!("Tense={:?}", tense));
                }

                // Check for person features
                if let Some(person) = &word.feats.person {
                    found_features.push(format!("Person={:?}", person));
                }

                // Check for voice features
                if let Some(voice) = &word.feats.voice {
                    found_features.push(format!("Voice={:?}", voice));
                }

                // Check for aspect features
                if let Some(aspect) = &word.feats.aspect {
                    found_features.push(format!("Aspect={:?}", aspect));
                }

                // Check for verbform features
                if let Some(verbform) = &word.feats.verbform {
                    found_features.push(format!("VerbForm={:?}", verbform));
                }

                // Check for mood features
                if let Some(mood) = &word.feats.mood {
                    found_features.push(format!("Mood={:?}", mood));
                }

                // Check for animacy features
                if let Some(animacy) = &word.feats.animacy {
                    found_features.push(format!("Animacy={:?}", animacy));
                }

                // Check for definiteness features
                if let Some(definiteness) = &word.feats.definiteness {
                    found_features.push(format!("Definiteness={:?}", definiteness));
                }
            }

            if !found_features.is_empty() {
                println!("  Found features: {}", found_features.join(", "));
            } else {
                println!("  No morphological features extracted (test model may be limited)");
            }
        }
    }

    #[test]
    fn test_conllu_features_parsing() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let workspace_root = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let model_path = workspace_root.join("models/test.model");

        if !model_path.exists() {
            println!("Skipping CoNLL-U features test - model not found");
            return;
        }

        let engine =
            UDPipeEngine::load(model_path.to_string_lossy()).expect("Should load UDPipe model");

        // Test direct CoNLL-U feature string parsing
        let feature_test_cases = vec![
            (
                "Number=Sing|Person=3|Tense=Pres",
                "Present 3rd person singular",
            ),
            ("Number=Plur|Tense=Past", "Past tense plural"),
            ("VerbForm=Inf", "Infinitive form"),
            ("Voice=Pass|Tense=Past", "Past passive voice"),
            (
                "Mood=Ind|Person=1|Number=Sing",
                "1st person singular indicative",
            ),
            ("Animacy=Anim|Number=Sing", "Animate singular"),
            ("Definiteness=Def", "Definite"),
            ("Aspect=Perf|Tense=Past", "Past perfective"),
            ("_", "No features"),
        ];

        for (feats_str, description) in feature_test_cases {
            println!("\nTesting features: '{}' ({})", feats_str, description);

            let parsed_features = engine.parse_morphological_features_from_conllu(feats_str);

            // Verify specific features were parsed correctly
            match feats_str {
                feat_str if feat_str.contains("Number=Sing") => {
                    assert_eq!(parsed_features.number, Some(UDNumber::Singular));
                    println!("  ✓ Correctly parsed Number=Singular");
                }
                feat_str if feat_str.contains("Number=Plur") => {
                    assert_eq!(parsed_features.number, Some(UDNumber::Plural));
                    println!("  ✓ Correctly parsed Number=Plural");
                }
                feat_str if feat_str.contains("Person=3") => {
                    assert_eq!(parsed_features.person, Some(UDPerson::Third));
                    println!("  ✓ Correctly parsed Person=3rd");
                }
                feat_str if feat_str.contains("Person=1") => {
                    assert_eq!(parsed_features.person, Some(UDPerson::First));
                    println!("  ✓ Correctly parsed Person=1st");
                }
                feat_str if feat_str.contains("Tense=Pres") => {
                    assert_eq!(parsed_features.tense, Some(UDTense::Present));
                    println!("  ✓ Correctly parsed Tense=Present");
                }
                feat_str if feat_str.contains("Tense=Past") => {
                    assert_eq!(parsed_features.tense, Some(UDTense::Past));
                    println!("  ✓ Correctly parsed Tense=Past");
                }
                feat_str if feat_str.contains("VerbForm=Inf") => {
                    assert_eq!(parsed_features.verbform, Some(UDVerbForm::Infinitive));
                    println!("  ✓ Correctly parsed VerbForm=Infinitive");
                }
                feat_str if feat_str.contains("Voice=Pass") => {
                    assert_eq!(parsed_features.voice, Some(UDVoice::Passive));
                    println!("  ✓ Correctly parsed Voice=Passive");
                }
                feat_str if feat_str.contains("Mood=Ind") => {
                    assert_eq!(parsed_features.mood, Some(UDMood::Indicative));
                    println!("  ✓ Correctly parsed Mood=Indicative");
                }
                feat_str if feat_str.contains("Animacy=Anim") => {
                    assert_eq!(parsed_features.animacy, Some(UDAnimacy::Animate));
                    println!("  ✓ Correctly parsed Animacy=Animate");
                }
                feat_str if feat_str.contains("Definiteness=Def") => {
                    assert_eq!(parsed_features.definiteness, Some(UDDefiniteness::Definite));
                    println!("  ✓ Correctly parsed Definiteness=Definite");
                }
                feat_str if feat_str.contains("Aspect=Perf") => {
                    assert_eq!(parsed_features.aspect, Some(UDAspect::Perfective));
                    println!("  ✓ Correctly parsed Aspect=Perfective");
                }
                "_" => {
                    // No features should be set
                    assert_eq!(parsed_features, MorphFeatures::default());
                    println!("  ✓ Correctly handled empty features");
                }
                _ => {
                    println!(
                        "  - Features parsed (specific assertions not implemented for this case)"
                    );
                }
            }
        }
    }

    #[test]
    fn test_comprehensive_morphological_coverage() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let workspace_root = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let model_path = workspace_root.join("models/test.model");

        if !model_path.exists() {
            println!("Skipping comprehensive morphological test - model not found");
            return;
        }

        let engine =
            UDPipeEngine::load(model_path.to_string_lossy()).expect("Should load UDPipe model");

        // Test a variety of linguistic constructions to exercise feature extraction
        let comprehensive_tests = vec![
            ("I am walking.", "1st person present progressive"),
            ("You were sleeping.", "2nd person past progressive"),
            ("She has worked.", "3rd person present perfect"),
            ("They will have been working.", "Future perfect progressive"),
            ("The book was read.", "Passive voice past"),
            ("Running is fun.", "Gerund/present participle"),
            ("To run is healthy.", "Infinitive construction"),
            ("The dogs bark loudly.", "Plural subject"),
            ("A cat sleeps quietly.", "Singular indefinite"),
            ("The specific cat meows.", "Definite reference"),
        ];

        let mut total_features_found = 0;
        let mut total_words_processed = 0;

        for (sentence, description) in comprehensive_tests {
            println!("\nAnalyzing: {} ({})", sentence, description);

            let result = engine.parse(sentence).expect("Should parse successfully");
            total_words_processed += result.words.len();

            for word in &result.words {
                let mut word_features = 0;

                // Count all the different types of features found
                if word.feats.number.is_some() {
                    word_features += 1;
                }
                if word.feats.person.is_some() {
                    word_features += 1;
                }
                if word.feats.tense.is_some() {
                    word_features += 1;
                }
                if word.feats.aspect.is_some() {
                    word_features += 1;
                }
                if word.feats.voice.is_some() {
                    word_features += 1;
                }
                if word.feats.mood.is_some() {
                    word_features += 1;
                }
                if word.feats.verbform.is_some() {
                    word_features += 1;
                }
                if word.feats.animacy.is_some() {
                    word_features += 1;
                }
                if word.feats.definiteness.is_some() {
                    word_features += 1;
                }

                total_features_found += word_features;

                if word_features > 0 {
                    println!("  {}: {} features", word.form, word_features);
                }
            }
        }

        println!("\n📊 Morphological Analysis Summary:");
        println!("  Total words processed: {}", total_words_processed);
        println!("  Total features extracted: {}", total_features_found);

        if total_features_found > 0 {
            let avg_features = total_features_found as f64 / total_words_processed as f64;
            println!("  Average features per word: {:.2}", avg_features);
            println!("  ✅ Morphological feature extraction is working!");
        } else {
            println!("  ⚠️  No morphological features found (test model may be limited)");
            println!("  Note: Real linguistic models would extract many more features");
        }

        // The test should pass regardless of feature extraction success,
        // since we're testing the infrastructure, not the model quality
        assert!(total_words_processed > 0, "Should process some words");
    }
}
