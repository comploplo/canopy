//! Basic tests for canopy-pipeline container.rs module
//!
//! Tests basic container creation and type structures

use canopy_pipeline::{ContainerBuilder, traits::*};

#[cfg(test)]
mod container_tests {
    use super::*;

    #[test]
    fn test_container_builder_creation() {
        let _builder = ContainerBuilder::new();
        // Reaching here without panic = test passes
    }

    #[test]
    fn test_container_builder_default() {
        let _builder = ContainerBuilder::default();
        // Reaching here without panic = test passes
    }

    #[test]
    fn test_performance_mode_variants() {
        let modes = [
            PerformanceMode::Balanced,
            PerformanceMode::Speed,
            PerformanceMode::Accuracy,
        ];

        assert_eq!(modes.len(), 3);
        assert_eq!(PerformanceMode::default(), PerformanceMode::Balanced);

        // Test equality
        assert_eq!(PerformanceMode::Speed, PerformanceMode::Speed);
        assert_ne!(PerformanceMode::Speed, PerformanceMode::Accuracy);
    }

    #[test]
    fn test_model_type_variants() {
        let types = [
            ModelType::UDPipe12,
            ModelType::UDPipe215,
            ModelType::Custom("test".to_string()),
        ];

        assert_eq!(types.len(), 3);
        assert_eq!(ModelType::UDPipe12, ModelType::UDPipe12);
        assert_ne!(ModelType::UDPipe12, ModelType::UDPipe215);
    }

    #[test]
    fn test_analyzer_config_creation() {
        use std::collections::HashMap;

        let config = AnalyzerConfig {
            enable_theta_assignment: true,
            enable_event_creation: false,
            enable_movement_detection: true,
            performance_mode: PerformanceMode::Speed,
            custom_settings: HashMap::new(),
        };

        assert!(config.enable_theta_assignment);
        assert!(!config.enable_event_creation);
        assert!(config.enable_movement_detection);
        assert_eq!(config.performance_mode, PerformanceMode::Speed);
        assert!(config.custom_settings.is_empty());
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
    fn test_parser_config_creation() {
        let config = ParserConfig {
            model_path: Some("/path/to/model".to_string()),
            model_type: ModelType::UDPipe215,
            performance_mode: PerformanceMode::Accuracy,
            enable_caching: true,
        };

        assert_eq!(config.model_path, Some("/path/to/model".to_string()));
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
            cache_type: "lru".to_string(),
            max_size_mb: 256,
            ttl_seconds: 3600,
        };

        assert_eq!(config.cache_type, "lru");
        assert_eq!(config.max_size_mb, 256);
        assert_eq!(config.ttl_seconds, 3600);
    }

    #[test]
    fn test_metrics_config_creation() {
        let config = MetricsConfig {
            enabled: true,
            backend: "prometheus".to_string(),
            collection_interval_ms: 1000,
        };

        assert!(config.enabled);
        assert_eq!(config.backend, "prometheus");
        assert_eq!(config.collection_interval_ms, 1000);
    }

    #[test]
    fn test_feature_set_creation() {
        let mut feature_set = FeatureSet::default();
        feature_set
            .morphological
            .insert("pos".to_string(), "NOUN".to_string());
        feature_set
            .semantic
            .insert("animacy".to_string(), "animate".to_string());

        assert_eq!(
            feature_set.morphological.get("pos"),
            Some(&"NOUN".to_string())
        );
        assert_eq!(
            feature_set.semantic.get("animacy"),
            Some(&"animate".to_string())
        );
        assert!(feature_set.verbnet.is_none());
        assert!(feature_set.custom.is_empty());
    }

    #[test]
    fn test_verbnet_features_creation() {
        use canopy_core::ThetaRole;

        let features = VerbNetFeatures {
            verb_class: Some("give-13.1".to_string()),
            theta_roles: vec![ThetaRole::Agent, ThetaRole::Theme, ThetaRole::Goal],
            selectional_restrictions: vec!["animate".to_string()],
        };

        assert_eq!(features.verb_class, Some("give-13.1".to_string()));
        assert_eq!(features.theta_roles.len(), 3);
        assert_eq!(features.selectional_restrictions.len(), 1);
    }

    #[test]
    fn test_extractor_capabilities_creation() {
        let capabilities = ExtractorCapabilities {
            name: "test_extractor".to_string(),
            supported_features: vec!["pos".to_string(), "lemma".to_string()],
            requires_pos_tags: true,
            requires_lemmas: false,
            batch_optimized: true,
        };

        assert_eq!(capabilities.name, "test_extractor");
        assert_eq!(capabilities.supported_features.len(), 2);
        assert!(capabilities.requires_pos_tags);
        assert!(!capabilities.requires_lemmas);
        assert!(capabilities.batch_optimized);
    }

    #[test]
    fn test_model_metadata_creation() {
        let metadata = ModelMetadata {
            identifier: "udpipe-en".to_string(),
            name: "UDPipe English Model".to_string(),
            version: "2.15".to_string(),
            language: "en".to_string(),
            model_type: ModelType::UDPipe215,
            file_size: Some(16384),
            download_url: Some("https://example.com/model.udpipe".to_string()),
            checksum: Some("abc123".to_string()),
        };

        assert_eq!(metadata.identifier, "udpipe-en");
        assert_eq!(metadata.name, "UDPipe English Model");
        assert_eq!(metadata.version, "2.15");
        assert_eq!(metadata.language, "en");
        assert_eq!(metadata.model_type, ModelType::UDPipe215);
        assert_eq!(metadata.file_size, Some(16384));
        assert!(metadata.download_url.is_some());
        assert!(metadata.checksum.is_some());
    }

    #[test]
    fn test_model_capabilities_creation() {
        let accuracy = AccuracyMetrics {
            pos_accuracy: 0.95,
            lemma_accuracy: 0.93,
            dependency_accuracy: 0.89,
        };

        let performance = PerformanceMetrics {
            tokens_per_second: 1000.0,
            memory_usage_mb: 50.0,
            model_size_mb: 15.0,
        };

        let capabilities = ModelCapabilities {
            accuracy_metrics: Some(accuracy),
            performance_metrics: Some(performance),
            supported_features: vec![
                "tokenization".to_string(),
                "pos_tagging".to_string(),
                "lemmatization".to_string(),
            ],
        };

        assert!(capabilities.accuracy_metrics.is_some());
        assert!(capabilities.performance_metrics.is_some());
        assert_eq!(capabilities.supported_features.len(), 3);
    }

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats {
            hits: 100,
            misses: 20,
            size_bytes: 1024,
            entry_count: 50,
        };

        assert_eq!(stats.hits, 100);
        assert_eq!(stats.misses, 20);
        assert_eq!(stats.size_bytes, 1024);
        assert_eq!(stats.entry_count, 50);
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
    fn test_metrics_creation() {
        let mut metrics = Metrics::default();
        metrics
            .timings
            .insert("parse".to_string(), vec![10, 15, 12]);
        metrics.counts.insert("requests".to_string(), 100);
        metrics.errors.insert("timeout".to_string(), 2);

        assert_eq!(metrics.timings.get("parse"), Some(&vec![10, 15, 12]));
        assert_eq!(metrics.counts.get("requests"), Some(&100));
        assert_eq!(metrics.errors.get("timeout"), Some(&2));
    }

    #[test]
    fn test_metrics_default() {
        let metrics = Metrics::default();

        assert!(metrics.timings.is_empty());
        assert!(metrics.counts.is_empty());
        assert!(metrics.errors.is_empty());
    }

    #[test]
    fn test_parser_capabilities_creation() {
        let capabilities = ParserCapabilities {
            supports_tokenization: true,
            supports_pos_tagging: true,
            supports_lemmatization: true,
            supports_dependency_parsing: false,
            supports_morphological_features: false,
            max_sentence_length: Some(100),
        };

        assert!(capabilities.supports_tokenization);
        assert!(capabilities.supports_pos_tagging);
        assert!(capabilities.supports_lemmatization);
        assert!(!capabilities.supports_dependency_parsing);
        assert!(!capabilities.supports_morphological_features);
        assert_eq!(capabilities.max_sentence_length, Some(100));
    }

    #[test]
    fn test_analyzer_capabilities_creation() {
        use canopy_core::ThetaRole;

        let capabilities = AnalyzerCapabilities {
            supports_theta_roles: true,
            supports_event_structure: true,
            supports_movement_chains: false,
            supports_little_v: false,
            theta_role_inventory: vec![ThetaRole::Agent, ThetaRole::Theme],
        };

        assert!(capabilities.supports_theta_roles);
        assert!(capabilities.supports_event_structure);
        assert!(!capabilities.supports_movement_chains);
        assert!(!capabilities.supports_little_v);
        assert_eq!(capabilities.theta_role_inventory.len(), 2);
    }

    #[test]
    fn test_parser_info_creation() {
        let info = ParserInfo {
            name: "UDPipe Parser".to_string(),
            version: "2.15".to_string(),
            model_type: "UDPipe".to_string(),
            supported_languages: vec!["en".to_string(), "es".to_string()],
            capabilities: ParserCapabilities {
                supports_tokenization: true,
                supports_pos_tagging: true,
                supports_lemmatization: true,
                supports_dependency_parsing: true,
                supports_morphological_features: true,
                max_sentence_length: Some(1000),
            },
        };

        assert_eq!(info.name, "UDPipe Parser");
        assert_eq!(info.version, "2.15");
        assert_eq!(info.model_type, "UDPipe");
        assert_eq!(info.supported_languages.len(), 2);
        assert!(info.capabilities.supports_tokenization);
    }

    #[test]
    fn test_analyzer_info_creation() {
        use canopy_core::ThetaRole;

        let info = AnalyzerInfo {
            name: "VerbNet Analyzer".to_string(),
            version: "1.0.0".to_string(),
            approach: "verbnet".to_string(),
            capabilities: AnalyzerCapabilities {
                supports_theta_roles: true,
                supports_event_structure: true,
                supports_movement_chains: false,
                supports_little_v: false,
                theta_role_inventory: vec![ThetaRole::Agent, ThetaRole::Patient],
            },
        };

        assert_eq!(info.name, "VerbNet Analyzer");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.approach, "verbnet");
        assert!(info.capabilities.supports_theta_roles);
    }
}
