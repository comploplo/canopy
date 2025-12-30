//! Shared engine traits for reducing boilerplate
//!
//! This module provides traits that standardize common patterns across
//! all semantic engines (VerbNet, FrameNet, WordNet, Lexicon, PropBank, Treebank).

use crate::EngineConfig;

/// Trait for engine-specific configs that can be converted to EngineConfig
///
/// Implement this trait to standardize config conversion across engines.
/// All semantic engines share the same basic configuration needs (caching,
/// metrics, confidence thresholds), so this trait provides a uniform interface.
///
/// # Example
///
/// ```ignore
/// impl EngineConfigurable for VerbNetConfig {
///     fn enable_cache(&self) -> bool { self.enable_cache }
///     fn cache_capacity(&self) -> usize { self.cache_capacity }
///     fn confidence_threshold(&self) -> f32 { self.confidence_threshold }
/// }
///
/// // Then use:
/// let engine_config = verbnet_config.to_engine_config();
/// ```
pub trait EngineConfigurable {
    /// Whether caching is enabled
    fn enable_cache(&self) -> bool;

    /// Cache capacity (number of entries)
    fn cache_capacity(&self) -> usize;

    /// Minimum confidence threshold for results
    fn confidence_threshold(&self) -> f32;

    /// Whether to enable metrics collection (default: true)
    fn enable_metrics(&self) -> bool {
        true
    }

    /// Whether to enable parallel processing (default: false)
    fn enable_parallel(&self) -> bool {
        false
    }

    /// Maximum parallel threads (default: 4)
    fn max_threads(&self) -> usize {
        4
    }

    /// Convert to EngineConfig using trait methods
    fn to_engine_config(&self) -> EngineConfig {
        EngineConfig {
            enable_cache: self.enable_cache(),
            cache_capacity: self.cache_capacity(),
            enable_metrics: self.enable_metrics(),
            enable_parallel: self.enable_parallel(),
            max_threads: self.max_threads(),
            confidence_threshold: self.confidence_threshold(),
        }
    }
}

/// Macro to implement EngineConfigurable for configs with standard field names
///
/// This macro handles the common case where a config struct has fields named:
/// - `enable_cache: bool`
/// - `cache_capacity: usize`
/// - `confidence_threshold: f32` OR `min_confidence: f32`
///
/// # Example
///
/// ```ignore
/// // For configs with `confidence_threshold` field:
/// impl_engine_configurable!(VerbNetConfig);
///
/// // For configs with `min_confidence` field:
/// impl_engine_configurable!(WordNetConfig, min_confidence);
///
/// // For configs with custom field names:
/// impl_engine_configurable!(TreebankConfig {
///     enable_cache: enable_base_engine_cache,
///     cache_capacity: base_engine_cache_capacity,
///     confidence_threshold: 0.5  // literal default value
/// });
/// ```
#[macro_export]
macro_rules! impl_engine_configurable {
    // Standard case: confidence_threshold field
    ($config_type:ty) => {
        impl $crate::EngineConfigurable for $config_type {
            fn enable_cache(&self) -> bool {
                self.enable_cache
            }
            fn cache_capacity(&self) -> usize {
                self.cache_capacity
            }
            fn confidence_threshold(&self) -> f32 {
                self.confidence_threshold
            }
        }
    };

    // Alternative: min_confidence field instead of confidence_threshold
    ($config_type:ty, min_confidence) => {
        impl $crate::EngineConfigurable for $config_type {
            fn enable_cache(&self) -> bool {
                self.enable_cache
            }
            fn cache_capacity(&self) -> usize {
                self.cache_capacity
            }
            fn confidence_threshold(&self) -> f32 {
                self.min_confidence
            }
        }
    };

    // Full custom mapping
    ($config_type:ty {
        enable_cache: $cache_field:ident,
        cache_capacity: $capacity_field:ident,
        confidence_threshold: $conf_expr:expr
    }) => {
        impl $crate::EngineConfigurable for $config_type {
            fn enable_cache(&self) -> bool {
                self.$cache_field
            }
            fn cache_capacity(&self) -> usize {
                self.$capacity_field
            }
            fn confidence_threshold(&self) -> f32 {
                $conf_expr
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test config with standard field names
    #[derive(Clone)]
    struct TestConfig {
        enable_cache: bool,
        cache_capacity: usize,
        confidence_threshold: f32,
    }

    impl_engine_configurable!(TestConfig);

    #[test]
    fn test_to_engine_config() {
        let config = TestConfig {
            enable_cache: true,
            cache_capacity: 5000,
            confidence_threshold: 0.7,
        };

        let engine_config = config.to_engine_config();

        assert!(engine_config.enable_cache);
        assert_eq!(engine_config.cache_capacity, 5000);
        assert!((engine_config.confidence_threshold - 0.7).abs() < f32::EPSILON);
        assert!(engine_config.enable_metrics); // default
        assert!(!engine_config.enable_parallel); // default
        assert_eq!(engine_config.max_threads, 4); // default
    }

    // Test config with min_confidence field
    #[derive(Clone)]
    struct AltConfig {
        enable_cache: bool,
        cache_capacity: usize,
        min_confidence: f32,
    }

    impl_engine_configurable!(AltConfig, min_confidence);

    #[test]
    fn test_min_confidence_variant() {
        let config = AltConfig {
            enable_cache: false,
            cache_capacity: 1000,
            min_confidence: 0.3,
        };

        let engine_config = config.to_engine_config();

        assert!(!engine_config.enable_cache);
        assert_eq!(engine_config.cache_capacity, 1000);
        assert!((engine_config.confidence_threshold - 0.3).abs() < f32::EPSILON);
    }
}
