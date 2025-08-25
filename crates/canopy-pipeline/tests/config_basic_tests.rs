//! Basic tests for config.rs module structures

use canopy_pipeline::config::*;
use std::collections::HashMap;

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();

        assert_eq!(config.model.language, "en");
        assert_eq!(config.performance.mode, "balanced");
        assert!(config.cache.enabled);
        assert!(config.memory.enable_gc);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();

        assert!(config.model_path.is_none());
        assert_eq!(config.model_type, "udpipe-1.2");
        assert_eq!(config.language, "en");
        assert!(!config.auto_download);
        assert!(config.validate_on_load);
    }

    #[test]
    fn test_model_config_custom() {
        let config = ModelConfig {
            model_path: Some("/path/to/model.udpipe".to_string()),
            model_type: "udpipe-2.15".to_string(),
            language: "es".to_string(),
            auto_download: true,
            validate_on_load: false,
        };

        assert_eq!(config.model_path, Some("/path/to/model.udpipe".to_string()));
        assert_eq!(config.model_type, "udpipe-2.15");
        assert_eq!(config.language, "es");
        assert!(config.auto_download);
        assert!(!config.validate_on_load);
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();

        assert_eq!(config.mode, "balanced");
        assert_eq!(config.max_text_length, 10_000);
        assert_eq!(config.timeout_seconds, 30);
        assert!(!config.enable_parallel);
        assert_eq!(config.batch_size, 10);
        assert!(config.thread_pool_size.is_none());
    }

    #[test]
    fn test_performance_config_custom() {
        let config = PerformanceConfig {
            mode: "speed".to_string(),
            max_text_length: 50_000,
            timeout_seconds: 60,
            enable_parallel: true,
            batch_size: 20,
            thread_pool_size: Some(4),
        };

        assert_eq!(config.mode, "speed");
        assert_eq!(config.max_text_length, 50_000);
        assert_eq!(config.timeout_seconds, 60);
        assert!(config.enable_parallel);
        assert_eq!(config.batch_size, 20);
        assert_eq!(config.thread_pool_size, Some(4));
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();

        assert!(config.enabled);
        assert_eq!(config.cache_type, "memory");
        assert_eq!(config.max_size_mb, 100);
        assert_eq!(config.ttl_seconds, 3600);
        assert_eq!(config.cleanup_interval_seconds, 300);
    }

    #[test]
    fn test_cache_config_custom() {
        let config = CacheConfig {
            enabled: false,
            cache_type: "redis".to_string(),
            max_size_mb: 512,
            ttl_seconds: 7200,
            cleanup_interval_seconds: 600,
        };

        assert!(!config.enabled);
        assert_eq!(config.cache_type, "redis");
        assert_eq!(config.max_size_mb, 512);
        assert_eq!(config.ttl_seconds, 7200);
        assert_eq!(config.cleanup_interval_seconds, 600);
    }

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();

        assert!(config.max_memory_mb.is_none());
        assert!(config.enable_gc);
        assert_eq!(config.gc_threshold_mb, 500);
        assert!(config.object_pooling);
    }

    #[test]
    fn test_memory_config_custom() {
        let config = MemoryConfig {
            max_memory_mb: Some(1024),
            enable_gc: false,
            gc_threshold_mb: 256,
            object_pooling: false,
        };

        assert_eq!(config.max_memory_mb, Some(1024));
        assert!(!config.enable_gc);
        assert_eq!(config.gc_threshold_mb, 256);
        assert!(!config.object_pooling);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();

        assert_eq!(config.level, "info");
        assert!(config.enable_tracing);
        assert!(config.enable_metrics);
        assert!(!config.log_to_file);
        assert!(config.log_file_path.is_none());
    }

    #[test]
    fn test_logging_config_custom() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            enable_tracing: false,
            enable_metrics: false,
            log_to_file: true,
            log_file_path: Some("/var/log/canopy.log".to_string()),
        };

        assert_eq!(config.level, "debug");
        assert!(!config.enable_tracing);
        assert!(!config.enable_metrics);
        assert!(config.log_to_file);
        assert_eq!(
            config.log_file_path,
            Some("/var/log/canopy.log".to_string())
        );
    }

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();

        assert!(config.enable_theta_roles);
        assert!(config.enable_events);
        assert!(config.enable_movement);
        assert!(config.enable_little_v);
        assert!(config.custom_features.is_empty());
    }

    #[test]
    fn test_analysis_config_custom() {
        let mut custom_features = HashMap::new();
        custom_features.insert("custom_feature_1".to_string(), true);
        custom_features.insert("custom_feature_2".to_string(), false);

        let config = AnalysisConfig {
            enable_theta_roles: false,
            enable_events: false,
            enable_movement: false,
            enable_little_v: false,
            custom_features,
        };

        assert!(!config.enable_theta_roles);
        assert!(!config.enable_events);
        assert!(!config.enable_movement);
        assert!(!config.enable_little_v);
        assert_eq!(config.custom_features.len(), 2);
        assert_eq!(config.custom_features.get("custom_feature_1"), Some(&true));
        assert_eq!(config.custom_features.get("custom_feature_2"), Some(&false));
    }

    #[test]
    fn test_pipeline_config_custom() {
        let config = PipelineConfig {
            model: ModelConfig {
                model_path: Some("custom_model.udpipe".to_string()),
                model_type: "custom".to_string(),
                language: "fr".to_string(),
                auto_download: true,
                validate_on_load: true,
            },
            performance: PerformanceConfig {
                mode: "accuracy".to_string(),
                max_text_length: 100_000,
                timeout_seconds: 120,
                enable_parallel: true,
                batch_size: 50,
                thread_pool_size: Some(8),
            },
            cache: CacheConfig {
                enabled: true,
                cache_type: "disk".to_string(),
                max_size_mb: 1024,
                ttl_seconds: 14400,
                cleanup_interval_seconds: 1800,
            },
            memory: MemoryConfig {
                max_memory_mb: Some(2048),
                enable_gc: true,
                gc_threshold_mb: 1024,
                object_pooling: true,
            },
            logging: LoggingConfig {
                level: "trace".to_string(),
                enable_tracing: true,
                enable_metrics: true,
                log_to_file: true,
                log_file_path: Some("canopy.log".to_string()),
            },
        };

        assert_eq!(config.model.language, "fr");
        assert_eq!(config.performance.mode, "accuracy");
        assert_eq!(config.cache.cache_type, "disk");
        assert_eq!(config.memory.max_memory_mb, Some(2048));
        assert_eq!(config.logging.level, "trace");
    }
}
