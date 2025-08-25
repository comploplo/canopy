//! Basic tests for traits.rs module structures

use canopy_core::ThetaRole as ThetaRoleType;
use canopy_pipeline::traits::*;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[cfg(test)]
mod traits_tests {
    use super::*;

    #[test]
    fn test_parser_info_creation() {
        let capabilities = ParserCapabilities {
            supports_tokenization: true,
            supports_pos_tagging: true,
            supports_lemmatization: false,
            supports_dependency_parsing: true,
            supports_morphological_features: false,
            max_sentence_length: Some(100),
        };

        let info = ParserInfo {
            name: "Test Parser".to_string(),
            version: "1.0.0".to_string(),
            model_type: "udpipe".to_string(),
            supported_languages: vec!["en".to_string(), "fr".to_string()],
            capabilities,
        };

        assert_eq!(info.name, "Test Parser");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.supported_languages.len(), 2);
        assert!(info.capabilities.supports_tokenization);
        assert!(!info.capabilities.supports_lemmatization);
        assert_eq!(info.capabilities.max_sentence_length, Some(100));
    }

    #[test]
    fn test_parser_capabilities_creation() {
        let capabilities = ParserCapabilities {
            supports_tokenization: true,
            supports_pos_tagging: true,
            supports_lemmatization: true,
            supports_dependency_parsing: true,
            supports_morphological_features: true,
            max_sentence_length: None,
        };

        assert!(capabilities.supports_tokenization);
        assert!(capabilities.supports_pos_tagging);
        assert!(capabilities.supports_lemmatization);
        assert!(capabilities.supports_dependency_parsing);
        assert!(capabilities.supports_morphological_features);
        assert_eq!(capabilities.max_sentence_length, None);
    }

    #[test]
    fn test_analyzer_info_creation() {
        let capabilities = AnalyzerCapabilities {
            supports_theta_roles: true,
            supports_event_structure: false,
            supports_movement_chains: true,
            supports_little_v: false,
            theta_role_inventory: vec![ThetaRoleType::Agent, ThetaRoleType::Patient],
        };

        let info = AnalyzerInfo {
            name: "VerbNet Analyzer".to_string(),
            version: "2.0".to_string(),
            approach: "verbnet".to_string(),
            capabilities,
        };

        assert_eq!(info.name, "VerbNet Analyzer");
        assert_eq!(info.version, "2.0");
        assert_eq!(info.approach, "verbnet");
        assert!(info.capabilities.supports_theta_roles);
        assert!(!info.capabilities.supports_event_structure);
        assert_eq!(info.capabilities.theta_role_inventory.len(), 2);
    }

    #[test]
    fn test_analyzer_capabilities_creation() {
        let capabilities = AnalyzerCapabilities {
            supports_theta_roles: true,
            supports_event_structure: true,
            supports_movement_chains: false,
            supports_little_v: true,
            theta_role_inventory: vec![
                ThetaRoleType::Agent,
                ThetaRoleType::Patient,
                ThetaRoleType::Theme,
            ],
        };

        assert!(capabilities.supports_theta_roles);
        assert!(capabilities.supports_event_structure);
        assert!(!capabilities.supports_movement_chains);
        assert!(capabilities.supports_little_v);
        assert_eq!(capabilities.theta_role_inventory.len(), 3);
    }

    #[test]
    fn test_analyzer_config_default() {
        let config = AnalyzerConfig::default();

        assert!(!config.enable_theta_assignment);
        assert!(!config.enable_event_creation);
        assert!(!config.enable_movement_detection);
        assert_eq!(config.performance_mode, PerformanceMode::Balanced);
        assert!(config.custom_settings.is_empty());
    }

    #[test]
    fn test_analyzer_config_creation() {
        let mut custom_settings = HashMap::new();
        custom_settings.insert("verbnet_path".to_string(), "/data/verbnet".to_string());

        let config = AnalyzerConfig {
            enable_theta_assignment: true,
            enable_event_creation: false,
            enable_movement_detection: true,
            performance_mode: PerformanceMode::Speed,
            custom_settings,
        };

        assert!(config.enable_theta_assignment);
        assert!(!config.enable_event_creation);
        assert!(config.enable_movement_detection);
        assert_eq!(config.performance_mode, PerformanceMode::Speed);
        assert_eq!(config.custom_settings.len(), 1);
    }

    #[test]
    fn test_performance_mode_variants() {
        let balanced = PerformanceMode::Balanced;
        let speed = PerformanceMode::Speed;
        let accuracy = PerformanceMode::Accuracy;

        assert_eq!(balanced, PerformanceMode::Balanced);
        assert_eq!(speed, PerformanceMode::Speed);
        assert_eq!(accuracy, PerformanceMode::Accuracy);
        assert_eq!(PerformanceMode::default(), PerformanceMode::Balanced);
    }

    #[test]
    fn test_feature_set_default() {
        let feature_set = FeatureSet::default();

        assert!(feature_set.morphological.is_empty());
        assert!(feature_set.semantic.is_empty());
        assert!(feature_set.verbnet.is_none());
        assert!(feature_set.custom.is_empty());
    }

    #[test]
    fn test_feature_set_creation() {
        let mut morphological = HashMap::new();
        morphological.insert("pos".to_string(), "VERB".to_string());

        let mut semantic = HashMap::new();
        semantic.insert("transitivity".to_string(), "transitive".to_string());

        let verbnet_features = VerbNetFeatures {
            verb_class: Some("run-51.3.2".to_string()),
            theta_roles: vec![ThetaRoleType::Agent, ThetaRoleType::Theme],
            selectional_restrictions: vec!["[+animate]".to_string()],
        };

        let feature_set = FeatureSet {
            morphological,
            semantic,
            verbnet: Some(verbnet_features),
            custom: HashMap::new(),
        };

        assert_eq!(feature_set.morphological.len(), 1);
        assert_eq!(feature_set.semantic.len(), 1);
        assert!(feature_set.verbnet.is_some());
        if let Some(vn) = &feature_set.verbnet {
            assert_eq!(vn.verb_class, Some("run-51.3.2".to_string()));
            assert_eq!(vn.theta_roles.len(), 2);
        }
    }

    #[test]
    fn test_verbnet_features_creation() {
        let features = VerbNetFeatures {
            verb_class: Some("eat-39.1".to_string()),
            theta_roles: vec![
                ThetaRoleType::Agent,
                ThetaRoleType::Patient,
                ThetaRoleType::Theme,
            ],
            selectional_restrictions: vec!["[+animate]".to_string(), "[+concrete]".to_string()],
        };

        assert_eq!(features.verb_class, Some("eat-39.1".to_string()));
        assert_eq!(features.theta_roles.len(), 3);
        assert_eq!(features.selectional_restrictions.len(), 2);
    }

    #[test]
    fn test_extractor_capabilities_creation() {
        let capabilities = ExtractorCapabilities {
            name: "VerbNet Extractor".to_string(),
            supported_features: vec!["theta_roles".to_string(), "verb_class".to_string()],
            requires_pos_tags: true,
            requires_lemmas: false,
            batch_optimized: true,
        };

        assert_eq!(capabilities.name, "VerbNet Extractor");
        assert_eq!(capabilities.supported_features.len(), 2);
        assert!(capabilities.requires_pos_tags);
        assert!(!capabilities.requires_lemmas);
        assert!(capabilities.batch_optimized);
    }

    #[test]
    fn test_model_metadata_creation() {
        let metadata = ModelMetadata {
            identifier: "udpipe-en-1.2".to_string(),
            name: "English UDPipe v1.2".to_string(),
            version: "1.2.0".to_string(),
            language: "en".to_string(),
            model_type: ModelType::UDPipe12,
            file_size: Some(50_000_000),
            download_url: Some("https://example.com/model.udpipe".to_string()),
            checksum: Some("abc123def456".to_string()),
        };

        assert_eq!(metadata.identifier, "udpipe-en-1.2");
        assert_eq!(metadata.language, "en");
        assert_eq!(metadata.model_type, ModelType::UDPipe12);
        assert_eq!(metadata.file_size, Some(50_000_000));
        assert!(metadata.download_url.is_some());
        assert!(metadata.checksum.is_some());
    }

    #[test]
    fn test_model_type_variants() {
        let udpipe12 = ModelType::UDPipe12;
        let udpipe215 = ModelType::UDPipe215;
        let custom = ModelType::Custom("stanza".to_string());

        assert_eq!(udpipe12, ModelType::UDPipe12);
        assert_eq!(udpipe215, ModelType::UDPipe215);
        if let ModelType::Custom(name) = custom {
            assert_eq!(name, "stanza");
        } else {
            panic!("Expected Custom variant");
        }
    }

    #[test]
    fn test_model_capabilities_creation() {
        let accuracy = AccuracyMetrics {
            pos_accuracy: 0.95,
            lemma_accuracy: 0.92,
            dependency_accuracy: 0.88,
        };

        let performance = PerformanceMetrics {
            tokens_per_second: 1000.0,
            memory_usage_mb: 512.0,
            model_size_mb: 50.0,
        };

        let capabilities = ModelCapabilities {
            accuracy_metrics: Some(accuracy),
            performance_metrics: Some(performance),
            supported_features: vec!["pos".to_string(), "lemma".to_string()],
        };

        assert!(capabilities.accuracy_metrics.is_some());
        assert!(capabilities.performance_metrics.is_some());
        assert_eq!(capabilities.supported_features.len(), 2);
    }

    #[test]
    fn test_accuracy_metrics_creation() {
        let metrics = AccuracyMetrics {
            pos_accuracy: 0.95,
            lemma_accuracy: 0.88,
            dependency_accuracy: 0.82,
        };

        assert_eq!(metrics.pos_accuracy, 0.95);
        assert_eq!(metrics.lemma_accuracy, 0.88);
        assert_eq!(metrics.dependency_accuracy, 0.82);
    }

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics {
            tokens_per_second: 2500.0,
            memory_usage_mb: 256.0,
            model_size_mb: 75.0,
        };

        assert_eq!(metrics.tokens_per_second, 2500.0);
        assert_eq!(metrics.memory_usage_mb, 256.0);
        assert_eq!(metrics.model_size_mb, 75.0);
    }

    #[test]
    fn test_cached_result_creation() {
        use canopy_semantic_layer::{AnalysisMetrics, LogicalForm, SemanticLayer1Output};
        use std::collections::HashMap;

        // Create a minimal SemanticLayer1Output for testing
        let logical_form = LogicalForm {
            predicates: vec![],
            variables: HashMap::new(),
            quantifiers: vec![],
        };

        let metrics = AnalysisMetrics {
            total_time_us: 50000,
            tokenization_time_us: 5000,
            framenet_time_us: 10000,
            verbnet_time_us: 15000,
            wordnet_time_us: 10000,
            token_count: 5,
            frame_count: 1,
            predicate_count: 2,
        };

        let analysis = SemanticLayer1Output {
            tokens: vec![],
            frames: vec![],
            predicates: vec![],
            logical_form,
            metrics,
        };

        let cached_result = CachedResult {
            text_hash: "hash123".to_string(),
            analysis,
            timestamp: SystemTime::now(),
            ttl: Duration::from_secs(3600),
        };

        assert_eq!(cached_result.text_hash, "hash123");
        assert_eq!(cached_result.ttl, Duration::from_secs(3600));
    }

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size_bytes, 0);
        assert_eq!(stats.entry_count, 0);
    }

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats {
            hits: 150,
            misses: 50,
            size_bytes: 1024000,
            entry_count: 200,
        };

        assert_eq!(stats.hits, 150);
        assert_eq!(stats.misses, 50);
        assert_eq!(stats.size_bytes, 1024000);
        assert_eq!(stats.entry_count, 200);
    }

    #[test]
    fn test_metrics_default() {
        let metrics = Metrics::default();

        assert!(metrics.timings.is_empty());
        assert!(metrics.counts.is_empty());
        assert!(metrics.errors.is_empty());
    }

    #[test]
    fn test_metrics_creation() {
        let mut timings = HashMap::new();
        timings.insert("parse".to_string(), vec![100, 150, 120]);

        let mut counts = HashMap::new();
        counts.insert("total_requests".to_string(), 1000);

        let mut errors = HashMap::new();
        errors.insert("parse_errors".to_string(), 5);

        let metrics = Metrics {
            timings,
            counts,
            errors,
        };

        assert_eq!(metrics.timings.len(), 1);
        assert_eq!(metrics.counts.len(), 1);
        assert_eq!(metrics.errors.len(), 1);
        assert_eq!(metrics.counts["total_requests"], 1000);
    }

    #[test]
    fn test_parser_config_creation() {
        let config = ParserConfig {
            model_path: Some("/models/english.udpipe".to_string()),
            model_type: ModelType::UDPipe215,
            performance_mode: PerformanceMode::Accuracy,
            enable_caching: true,
        };

        assert!(config.model_path.is_some());
        assert_eq!(config.model_type, ModelType::UDPipe215);
        assert_eq!(config.performance_mode, PerformanceMode::Accuracy);
        assert!(config.enable_caching);
    }

    #[test]
    fn test_extractor_config_creation() {
        let config = ExtractorConfig {
            extractor_type: "verbnet".to_string(),
            enable_verbnet: true,
            custom_rules: vec!["rule1".to_string(), "rule2".to_string()],
        };

        assert_eq!(config.extractor_type, "verbnet");
        assert!(config.enable_verbnet);
        assert_eq!(config.custom_rules.len(), 2);
    }

    #[test]
    fn test_cache_config_creation() {
        let config = CacheConfig {
            cache_type: "memory".to_string(),
            max_size_mb: 512,
            ttl_seconds: 3600,
        };

        assert_eq!(config.cache_type, "memory");
        assert_eq!(config.max_size_mb, 512);
        assert_eq!(config.ttl_seconds, 3600);
    }

    #[test]
    fn test_metrics_config_creation() {
        let config = MetricsConfig {
            enabled: true,
            backend: "prometheus".to_string(),
            collection_interval_ms: 5000,
        };

        assert!(config.enabled);
        assert_eq!(config.backend, "prometheus");
        assert_eq!(config.collection_interval_ms, 5000);
    }
}
