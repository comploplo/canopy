//! Macros for reducing boilerplate in semantic engines

/// Macro to implement `EngineConfigurable` for configs with standard field names
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
/// ```
#[macro_export]
macro_rules! impl_engine_configurable {
    // Standard case: confidence_threshold field
    ($config_type:ty) => {
        impl $crate::engine::EngineConfigurable for $config_type {
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
        impl $crate::engine::EngineConfigurable for $config_type {
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
}

#[cfg(test)]
mod tests {
    use crate::engine::EngineConfigurable;

    #[derive(Clone)]
    struct TestConfig {
        enable_cache: bool,
        cache_capacity: usize,
        confidence_threshold: f32,
    }

    impl_engine_configurable!(TestConfig);

    #[test]
    fn test_impl_engine_configurable_macro() {
        let config = TestConfig {
            enable_cache: true,
            cache_capacity: 5000,
            confidence_threshold: 0.7,
        };

        assert!(config.enable_cache());
        assert_eq!(config.cache_capacity(), 5000);
        assert!((config.confidence_threshold() - 0.7).abs() < f32::EPSILON);
    }

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

        assert!(!config.enable_cache());
        assert_eq!(config.cache_capacity(), 1000);
        assert!((config.confidence_threshold() - 0.3).abs() < f32::EPSILON);
    }
}
