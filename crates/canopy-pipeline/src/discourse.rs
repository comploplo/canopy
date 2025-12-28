//! Layer 3 Discourse Processing
//!
//! This module integrates canopy-discourse with the pipeline, providing
//! a unified interface for multi-sentence discourse analysis.
//!
//! ## Pipeline Flow
//!
//! ```text
//! Text → Layer 1 (Semantic Analysis)
//!      → Layer 2 (Event Composition)
//!      → Layer 3 (Discourse/DRT)
//!      → DRS (Discourse Representation Structure)
//! ```

use canopy_discourse::{DiscourseConfig, DiscourseContext, DiscourseResult, Drs, ReferentId};
use canopy_events::{ComposedEvent, ComposedEvents};

/// Processor for Layer 3 discourse analysis
///
/// Manages discourse context and builds Discourse Representation Structures (DRS)
/// from Layer 2 event compositions.
#[derive(Debug)]
pub struct DiscourseProcessor {
    context: DiscourseContext,
}

impl DiscourseProcessor {
    /// Create a new discourse processor with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: DiscourseContext::with_defaults(),
        }
    }

    /// Create a discourse processor with custom configuration
    #[must_use]
    pub fn with_config(config: DiscourseConfig) -> Self {
        Self {
            context: DiscourseContext::new(config),
        }
    }

    /// Process a single sentence's events and add them to discourse context
    ///
    /// Returns the event referent IDs created for this sentence.
    pub fn process_sentence(
        &mut self,
        text: &str,
        events: &ComposedEvents,
    ) -> DiscourseResult<Vec<ReferentId>> {
        self.context.begin_sentence(text.to_string());

        let mut event_ids = Vec::new();
        for event in &events.events {
            let event_id = self.context.process_event(event)?;
            event_ids.push(event_id);
        }

        self.context.end_sentence();
        Ok(event_ids)
    }

    /// Process multiple sentences and build a complete DRS
    ///
    /// Takes pairs of (sentence_text, composed_events) and processes them
    /// in order, building up the discourse context.
    pub fn process_document(
        &mut self,
        sentences: &[(String, ComposedEvents)],
    ) -> DiscourseResult<&Drs> {
        for (text, events) in sentences {
            self.process_sentence(text, events)?;
        }
        Ok(self.context.drs())
    }

    /// Process a single event directly (for fine-grained control)
    pub fn process_event(&mut self, event: &ComposedEvent) -> DiscourseResult<ReferentId> {
        self.context.process_event(event)
    }

    /// Resolve a pronoun to its antecedent in the current discourse
    ///
    /// Uses recency, animacy, and gender/number agreement to find
    /// the most likely antecedent.
    pub fn resolve_pronoun(&mut self, pronoun: &str) -> DiscourseResult<ReferentId> {
        self.context.resolve_pronoun(pronoun)
    }

    /// Get the current Discourse Representation Structure
    #[must_use]
    pub fn drs(&self) -> &Drs {
        self.context.drs()
    }

    /// Get the underlying discourse context for advanced operations
    #[must_use]
    pub fn context(&self) -> &DiscourseContext {
        &self.context
    }

    /// Get mutable access to the discourse context
    pub fn context_mut(&mut self) -> &mut DiscourseContext {
        &mut self.context
    }

    /// Clear all discourse state and start fresh
    pub fn reset(&mut self) {
        self.context.clear();
    }

    /// Get statistics about the current discourse state
    #[must_use]
    pub fn statistics(&self) -> DiscourseStatistics {
        let ctx_stats = self.context.statistics();
        DiscourseStatistics {
            sentence_count: ctx_stats.sentence_count,
            referent_count: ctx_stats.referent_count,
            condition_count: ctx_stats.condition_count,
            resolution_count: ctx_stats.resolution_count,
        }
    }
}

impl Default for DiscourseProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about discourse processing
#[derive(Debug, Clone)]
pub struct DiscourseStatistics {
    /// Number of sentences processed
    pub sentence_count: usize,
    /// Number of discourse referents (entities + events)
    pub referent_count: usize,
    /// Number of DRS conditions
    pub condition_count: usize,
    /// Number of anaphora resolutions performed
    pub resolution_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discourse_processor_creation() {
        let processor = DiscourseProcessor::new();
        assert_eq!(processor.statistics().sentence_count, 0);
        assert_eq!(processor.statistics().referent_count, 0);
    }

    #[test]
    fn test_discourse_processor_reset() {
        let mut processor = DiscourseProcessor::new();
        processor.context_mut().begin_sentence("Test.".to_string());
        processor.context_mut().end_sentence();
        assert_eq!(processor.statistics().sentence_count, 1);

        processor.reset();
        assert_eq!(processor.statistics().sentence_count, 0);
    }
}
