//! Simple tests for engines.rs module
//!
//! Tests basic engine structures and functionality

use canopy_tokenizer::engines::{
    CoverageStats, FrameNetStatsSummary, MultiResourceConfig, MultiResourceResult, SemanticSource,
    UnifiedSemanticStats, VerbNetStatsSummary, WordNetStatsSummary,
};

#[cfg(test)]
mod engines_tests {
    use super::*;

    #[test]
    fn test_multi_resource_config_default() {
        let config = MultiResourceConfig::default();
        assert!(config.enable_framenet);
        assert!(config.enable_verbnet);
        assert!(config.enable_wordnet);
        assert_eq!(config.confidence_threshold, 0.5);
        assert_eq!(config.max_results_per_engine, 10);
    }

    #[test]
    fn test_multi_resource_config_custom() {
        let config = MultiResourceConfig {
            enable_framenet: false,
            enable_verbnet: true,
            enable_wordnet: false,
            confidence_threshold: 0.8,
            max_results_per_engine: 5,
        };

        assert!(!config.enable_framenet);
        assert!(config.enable_verbnet);
        assert!(!config.enable_wordnet);
        assert_eq!(config.confidence_threshold, 0.8);
        assert_eq!(config.max_results_per_engine, 5);
    }

    #[test]
    fn test_semantic_source_enum() {
        let sources = vec![
            SemanticSource::FrameNet,
            SemanticSource::VerbNet,
            SemanticSource::WordNet,
        ];

        assert_eq!(sources.len(), 3);

        // Test equality
        assert_eq!(SemanticSource::FrameNet, SemanticSource::FrameNet);
        assert_ne!(SemanticSource::FrameNet, SemanticSource::VerbNet);
    }

    #[test]
    fn test_verbnet_stats_summary_creation() {
        let stats = VerbNetStatsSummary {
            total_classes: 100,
            total_verbs: 500,
            total_theta_roles: 1000,
            cache_hit_rate: 0.8,
        };

        assert_eq!(stats.total_classes, 100);
        assert_eq!(stats.total_verbs, 500);
        assert_eq!(stats.total_theta_roles, 1000);
        assert_eq!(stats.cache_hit_rate, 0.8);
    }

    #[test]
    fn test_framenet_stats_summary_creation() {
        let stats = FrameNetStatsSummary {
            total_frames: 50,
            total_lexical_units: 200,
            unique_lemmas: 150,
            cache_hit_rate: 0.7,
        };

        assert_eq!(stats.total_frames, 50);
        assert_eq!(stats.total_lexical_units, 200);
        assert_eq!(stats.unique_lemmas, 150);
        assert_eq!(stats.cache_hit_rate, 0.7);
    }

    #[test]
    fn test_wordnet_stats_summary_creation() {
        let stats = WordNetStatsSummary {
            total_words: 1000,
            total_senses: 2000,
            total_hypernyms: 500,
            total_hyponyms: 800,
        };

        assert_eq!(stats.total_words, 1000);
        assert_eq!(stats.total_senses, 2000);
        assert_eq!(stats.total_hypernyms, 500);
        assert_eq!(stats.total_hyponyms, 800);
    }

    #[test]
    fn test_coverage_stats_creation() {
        let stats = CoverageStats {
            total_covered_lemmas: 100,
            verbnet_only: 20,
            framenet_only: 15,
            wordnet_only: 30,
            multi_resource_coverage: 35,
            coverage_percentage: 0.85,
        };

        assert_eq!(stats.total_covered_lemmas, 100);
        assert_eq!(stats.verbnet_only, 20);
        assert_eq!(stats.framenet_only, 15);
        assert_eq!(stats.wordnet_only, 30);
        assert_eq!(stats.multi_resource_coverage, 35);
        assert_eq!(stats.coverage_percentage, 0.85);

        // Test consistency
        let sum = stats.verbnet_only
            + stats.framenet_only
            + stats.wordnet_only
            + stats.multi_resource_coverage;
        assert_eq!(sum, 100); // Should add up to total
    }

    #[test]
    fn test_unified_semantic_stats_creation() {
        let verbnet_stats = VerbNetStatsSummary {
            total_classes: 10,
            total_verbs: 50,
            total_theta_roles: 100,
            cache_hit_rate: 0.6,
        };

        let framenet_stats = FrameNetStatsSummary {
            total_frames: 5,
            total_lexical_units: 25,
            unique_lemmas: 20,
            cache_hit_rate: 0.7,
        };

        let wordnet_stats = WordNetStatsSummary {
            total_words: 100,
            total_senses: 200,
            total_hypernyms: 50,
            total_hyponyms: 80,
        };

        let coverage = CoverageStats {
            total_covered_lemmas: 60,
            verbnet_only: 10,
            framenet_only: 15,
            wordnet_only: 20,
            multi_resource_coverage: 15,
            coverage_percentage: 0.75,
        };

        let unified_stats = UnifiedSemanticStats {
            verbnet: verbnet_stats,
            framenet: framenet_stats,
            wordnet: wordnet_stats,
            coverage,
        };

        assert_eq!(unified_stats.verbnet.total_classes, 10);
        assert_eq!(unified_stats.framenet.total_frames, 5);
        assert_eq!(unified_stats.wordnet.total_words, 100);
        assert_eq!(unified_stats.coverage.total_covered_lemmas, 60);
    }

    #[test]
    fn test_multi_resource_result_creation() {
        let result = MultiResourceResult {
            verbnet_classes: Vec::new(),
            framenet_frames: Vec::new(),
            wordnet_senses: Vec::new(),
            framenet_units: Vec::new(),
            confidence: 0.8,
            sources: vec![SemanticSource::VerbNet, SemanticSource::FrameNet],
        };

        assert_eq!(result.verbnet_classes.len(), 0);
        assert_eq!(result.framenet_frames.len(), 0);
        assert_eq!(result.wordnet_senses.len(), 0);
        assert_eq!(result.framenet_units.len(), 0);
        assert_eq!(result.confidence, 0.8);
        assert_eq!(result.sources.len(), 2);
        assert!(result.sources.contains(&SemanticSource::VerbNet));
        assert!(result.sources.contains(&SemanticSource::FrameNet));
    }

    #[test]
    fn test_confidence_levels() {
        let low_confidence = MultiResourceResult {
            verbnet_classes: Vec::new(),
            framenet_frames: Vec::new(),
            wordnet_senses: Vec::new(),
            framenet_units: Vec::new(),
            confidence: 0.2,
            sources: vec![SemanticSource::WordNet],
        };

        let high_confidence = MultiResourceResult {
            verbnet_classes: Vec::new(),
            framenet_frames: Vec::new(),
            wordnet_senses: Vec::new(),
            framenet_units: Vec::new(),
            confidence: 0.95,
            sources: vec![
                SemanticSource::VerbNet,
                SemanticSource::FrameNet,
                SemanticSource::WordNet,
            ],
        };

        assert!(low_confidence.confidence < 0.5);
        assert!(high_confidence.confidence > 0.9);
        assert!(high_confidence.sources.len() > low_confidence.sources.len());
    }

    #[test]
    fn test_coverage_percentage_calculation() {
        // Test different coverage scenarios
        let full_coverage = CoverageStats {
            total_covered_lemmas: 100,
            verbnet_only: 25,
            framenet_only: 25,
            wordnet_only: 25,
            multi_resource_coverage: 25,
            coverage_percentage: 1.0,
        };

        let partial_coverage = CoverageStats {
            total_covered_lemmas: 50,
            verbnet_only: 20,
            framenet_only: 15,
            wordnet_only: 10,
            multi_resource_coverage: 5,
            coverage_percentage: 0.5,
        };

        assert_eq!(full_coverage.coverage_percentage, 1.0);
        assert_eq!(partial_coverage.coverage_percentage, 0.5);

        // Test that sums are consistent
        assert_eq!(
            full_coverage.verbnet_only
                + full_coverage.framenet_only
                + full_coverage.wordnet_only
                + full_coverage.multi_resource_coverage,
            100
        );
        assert_eq!(
            partial_coverage.verbnet_only
                + partial_coverage.framenet_only
                + partial_coverage.wordnet_only
                + partial_coverage.multi_resource_coverage,
            50
        );
    }

    #[test]
    fn test_source_combination_patterns() {
        // Test common source combinations
        let verbnet_only = vec![SemanticSource::VerbNet];
        let framenet_only = vec![SemanticSource::FrameNet];
        let wordnet_only = vec![SemanticSource::WordNet];
        let all_sources = vec![
            SemanticSource::VerbNet,
            SemanticSource::FrameNet,
            SemanticSource::WordNet,
        ];
        let partial_sources = vec![SemanticSource::VerbNet, SemanticSource::WordNet];

        assert_eq!(verbnet_only.len(), 1);
        assert_eq!(framenet_only.len(), 1);
        assert_eq!(wordnet_only.len(), 1);
        assert_eq!(all_sources.len(), 3);
        assert_eq!(partial_sources.len(), 2);

        // Test that we can identify specific sources
        assert!(verbnet_only.contains(&SemanticSource::VerbNet));
        assert!(!verbnet_only.contains(&SemanticSource::FrameNet));
        assert!(all_sources.contains(&SemanticSource::VerbNet));
        assert!(all_sources.contains(&SemanticSource::FrameNet));
        assert!(all_sources.contains(&SemanticSource::WordNet));
    }

    #[test]
    fn test_stats_aggregation() {
        // Test aggregating statistics from multiple engines
        let verbnet_stats = VerbNetStatsSummary {
            total_classes: 50,
            total_verbs: 200,
            total_theta_roles: 400,
            cache_hit_rate: 0.8,
        };

        let framenet_stats = FrameNetStatsSummary {
            total_frames: 30,
            total_lexical_units: 150,
            unique_lemmas: 100,
            cache_hit_rate: 0.75,
        };

        // Test that we can access all fields
        assert!(verbnet_stats.total_classes > 0);
        assert!(verbnet_stats.total_verbs > verbnet_stats.total_classes);
        assert!(verbnet_stats.total_theta_roles > verbnet_stats.total_verbs);
        assert!(verbnet_stats.cache_hit_rate > 0.0);
        assert!(verbnet_stats.cache_hit_rate <= 1.0);

        assert!(framenet_stats.total_frames > 0);
        assert!(framenet_stats.total_lexical_units > framenet_stats.total_frames);
        assert!(framenet_stats.unique_lemmas <= framenet_stats.total_lexical_units);
        assert!(framenet_stats.cache_hit_rate > 0.0);
        assert!(framenet_stats.cache_hit_rate <= 1.0);
    }
}
