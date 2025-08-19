//! Configuration types for the pipeline

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub model: ModelConfig,
    pub performance: PerformanceConfig,
    pub cache: CacheConfig,
    pub memory: MemoryConfig,
    pub logging: LoggingConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            performance: PerformanceConfig::default(),
            cache: CacheConfig::default(),
            memory: MemoryConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: Option<String>,
    pub model_type: String,
    pub language: String,
    pub auto_download: bool,
    pub validate_on_load: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            model_type: "udpipe-1.2".to_string(),
            language: "en".to_string(),
            auto_download: false,
            validate_on_load: true,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub mode: String,
    pub max_text_length: usize,
    pub timeout_seconds: u64,
    pub enable_parallel: bool,
    pub batch_size: usize,
    pub thread_pool_size: Option<usize>,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            mode: "balanced".to_string(),
            max_text_length: 10_000,
            timeout_seconds: 30,
            enable_parallel: false,
            batch_size: 10,
            thread_pool_size: None,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub cache_type: String,
    pub max_size_mb: u64,
    pub ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_type: "memory".to_string(),
            max_size_mb: 100,
            ttl_seconds: 3600,
            cleanup_interval_seconds: 300,
        }
    }
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub max_memory_mb: Option<u64>,
    pub enable_gc: bool,
    pub gc_threshold_mb: u64,
    pub object_pooling: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: None,
            enable_gc: true,
            gc_threshold_mb: 500,
            object_pooling: true,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub enable_tracing: bool,
    pub enable_metrics: bool,
    pub log_to_file: bool,
    pub log_file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            enable_tracing: true,
            enable_metrics: true,
            log_to_file: false,
            log_file_path: None,
        }
    }
}

/// Analysis-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub enable_theta_roles: bool,
    pub enable_events: bool,
    pub enable_movement: bool,
    pub enable_little_v: bool,
    pub custom_features: HashMap<String, bool>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            enable_theta_roles: true,
            enable_events: true,
            enable_movement: true,
            enable_little_v: true,
            custom_features: HashMap::new(),
        }
    }
}
