//! Configuration for event composition

use serde::{Deserialize, Serialize};

/// Configuration for the EventComposer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventComposerConfig {
    /// Minimum confidence threshold for event composition
    pub confidence_threshold: f32,

    /// Whether to require an Agent role for transitive events
    pub require_agent_for_transitives: bool,

    /// Whether to use FrameNet as fallback when VerbNet fails
    pub use_framenet_fallback: bool,

    /// Whether to use WordNet for animacy inference
    pub use_wordnet_animacy: bool,

    /// Maximum number of events to compose per sentence
    pub max_events_per_sentence: usize,

    /// Whether to include sub-events in decomposition
    pub include_sub_events: bool,
}

impl Default for EventComposerConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.3,
            require_agent_for_transitives: false,
            use_framenet_fallback: true,
            use_wordnet_animacy: true,
            max_events_per_sentence: 10,
            include_sub_events: true,
        }
    }
}

impl EventComposerConfig {
    /// Create a strict configuration requiring higher confidence
    pub fn strict() -> Self {
        Self {
            confidence_threshold: 0.6,
            require_agent_for_transitives: true,
            ..Default::default()
        }
    }

    /// Create a lenient configuration for exploratory analysis
    pub fn lenient() -> Self {
        Self {
            confidence_threshold: 0.1,
            require_agent_for_transitives: false,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_composer_config_default() {
        let config = EventComposerConfig::default();
        assert_eq!(config.confidence_threshold, 0.3);
        assert!(!config.require_agent_for_transitives);
        assert!(config.use_framenet_fallback);
        assert!(config.use_wordnet_animacy);
        assert_eq!(config.max_events_per_sentence, 10);
        assert!(config.include_sub_events);
    }

    #[test]
    fn test_event_composer_config_strict() {
        let config = EventComposerConfig::strict();
        assert_eq!(config.confidence_threshold, 0.6);
        assert!(config.require_agent_for_transitives);
        // Inherits other defaults
        assert!(config.use_framenet_fallback);
        assert!(config.use_wordnet_animacy);
        assert_eq!(config.max_events_per_sentence, 10);
        assert!(config.include_sub_events);
    }

    #[test]
    fn test_event_composer_config_lenient() {
        let config = EventComposerConfig::lenient();
        assert_eq!(config.confidence_threshold, 0.1);
        assert!(!config.require_agent_for_transitives);
        // Inherits other defaults
        assert!(config.use_framenet_fallback);
        assert!(config.use_wordnet_animacy);
        assert_eq!(config.max_events_per_sentence, 10);
        assert!(config.include_sub_events);
    }

    #[test]
    fn test_event_composer_config_clone_debug() {
        let config = EventComposerConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.confidence_threshold, 0.3);
        let debug = format!("{:?}", config);
        assert!(debug.contains("confidence_threshold"));
    }

    #[test]
    fn test_event_composer_config_serializable() {
        // Test that types derive Serialize/Deserialize (compile-time check)
        fn _assert_serializable<T: serde::Serialize + serde::de::DeserializeOwned>() {}
        _assert_serializable::<EventComposerConfig>();
    }
}
