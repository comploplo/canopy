//! Parallel processing infrastructure for semantic engines
//!
//! This module provides utilities for parallel query processing
//! across multiple semantic resources.

use crate::{EngineError, EngineResult, SemanticResult};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Type alias for complex engine function
type EngineFunction<I, O> = Box<dyn Fn(&I) -> EngineResult<O> + Send + Sync>;

/// Parallel processor for semantic engines
pub struct ParallelProcessor {
    /// Number of worker threads
    thread_count: usize,
    /// Whether parallel processing is enabled
    enabled: bool,
}

impl ParallelProcessor {
    /// Create a new parallel processor
    pub fn new(thread_count: usize, enabled: bool) -> Self {
        Self {
            thread_count: thread_count.max(1),
            enabled,
        }
    }

    /// Process multiple queries in parallel using a closure
    pub fn process_parallel<I, O, F>(
        &self,
        inputs: Vec<I>,
        processor: F,
    ) -> EngineResult<Vec<SemanticResult<O>>>
    where
        I: Clone + Send + 'static,
        O: Clone + Send + 'static,
        F: Fn(&I) -> EngineResult<O> + Send + Sync + 'static,
    {
        if !self.enabled || inputs.len() <= 1 {
            // Fall back to sequential processing
            return self.process_sequential(inputs, processor);
        }

        let _start_time = Instant::now();
        let processor = Arc::new(processor);
        let chunk_size = inputs.len().div_ceil(self.thread_count);

        let mut handles = Vec::new();

        // Split inputs into chunks for parallel processing
        for chunk in inputs.chunks(chunk_size) {
            let chunk = chunk.to_vec();
            let processor = Arc::clone(&processor);

            let handle = thread::spawn(move || {
                let mut results = Vec::new();

                for input in chunk {
                    let query_start = Instant::now();

                    match processor(&input) {
                        Ok(output) => {
                            let processing_time = query_start.elapsed().as_micros() as u64;
                            results.push(Ok(SemanticResult::new(
                                output,
                                1.0,
                                false,
                                processing_time,
                            )));
                        }
                        Err(error) => {
                            results.push(Err(error));
                        }
                    }
                }

                results
            });

            handles.push(handle);
        }

        // Collect results from all threads
        let mut all_results = Vec::new();

        for handle in handles {
            match handle.join() {
                Ok(thread_results) => {
                    all_results.extend(thread_results);
                }
                Err(_) => {
                    return Err(EngineError::parallel(
                        "Thread panicked during parallel processing",
                    ));
                }
            }
        }

        // Convert error results to final results
        let mut final_results = Vec::new();
        for result in all_results {
            match result {
                Ok(semantic_result) => final_results.push(semantic_result),
                Err(error) => return Err(error),
            }
        }

        Ok(final_results)
    }

    /// Process queries sequentially (fallback)
    fn process_sequential<I, O, F>(
        &self,
        inputs: Vec<I>,
        processor: F,
    ) -> EngineResult<Vec<SemanticResult<O>>>
    where
        I: Clone,
        O: Clone,
        F: Fn(&I) -> EngineResult<O>,
    {
        let mut results = Vec::new();

        for input in inputs {
            let start_time = Instant::now();

            match processor(&input) {
                Ok(output) => {
                    let processing_time = start_time.elapsed().as_micros() as u64;
                    results.push(SemanticResult::new(output, 1.0, false, processing_time));
                }
                Err(error) => return Err(error),
            }
        }

        Ok(results)
    }

    /// Set the number of threads to use
    pub fn set_thread_count(&mut self, count: usize) {
        self.thread_count = count.max(1);
    }

    /// Get the current thread count
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    /// Enable or disable parallel processing
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if parallel processing is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get optimal thread count for the current system
    pub fn optimal_thread_count() -> usize {
        // Simple fallback - use available parallelism or default to 4
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .clamp(1, 8) // Cap at 8 threads for I/O bound tasks
    }
}

impl Default for ParallelProcessor {
    fn default() -> Self {
        Self::new(Self::optimal_thread_count(), true)
    }
}

/// Utility for coordinating parallel queries across multiple engines
pub struct MultiEngineCoordinator {
    /// Parallel processor
    #[allow(dead_code)]
    processor: ParallelProcessor,
    /// Engine-specific timeout settings
    timeouts: std::collections::HashMap<String, std::time::Duration>,
}

impl MultiEngineCoordinator {
    /// Create a new coordinator
    pub fn new(processor: ParallelProcessor) -> Self {
        Self {
            processor,
            timeouts: std::collections::HashMap::new(),
        }
    }

    /// Set timeout for a specific engine
    pub fn set_engine_timeout<S: Into<String>>(
        &mut self,
        engine_name: S,
        timeout: std::time::Duration,
    ) {
        self.timeouts.insert(engine_name.into(), timeout);
    }

    /// Process queries across multiple engines in parallel
    pub fn query_engines<I, O>(
        &self,
        input: I,
        engines: Vec<EngineFunction<I, O>>,
    ) -> EngineResult<Vec<EngineResult<SemanticResult<O>>>>
    where
        I: Clone + Send + Sync + 'static,
        O: Clone + Send + 'static,
    {
        if engines.is_empty() {
            return Ok(Vec::new());
        }

        let input = Arc::new(input);
        let mut handles = Vec::new();

        // Spawn a thread for each engine
        for (index, engine) in engines.into_iter().enumerate() {
            let input = Arc::clone(&input);
            let engine_name = format!("engine_{index}");
            let timeout = self
                .timeouts
                .get(&engine_name)
                .copied()
                .unwrap_or(std::time::Duration::from_secs(5));

            let handle = thread::spawn(move || {
                let start_time = Instant::now();

                // Use a channel for timeout handling
                let (tx, rx) = std::sync::mpsc::channel();

                let input_clone = Arc::clone(&input);
                thread::spawn(move || {
                    let result = engine(&input_clone);
                    let _ = tx.send(result);
                });

                // Wait for result with timeout
                match rx.recv_timeout(timeout) {
                    Ok(result) => {
                        let processing_time = start_time.elapsed().as_micros() as u64;

                        match result {
                            Ok(output) => {
                                Ok(SemanticResult::new(output, 1.0, false, processing_time))
                            }
                            Err(error) => Err(error),
                        }
                    }
                    Err(_) => Err(EngineError::timeout(
                        format!("Engine query for {engine_name}"),
                        timeout.as_millis() as u64,
                    )),
                }
            });

            handles.push(handle);
        }

        // Collect results from all engines
        let mut results = Vec::new();

        for handle in handles {
            match handle.join() {
                Ok(result) => results.push(result),
                Err(_) => results.push(Err(EngineError::parallel("Engine thread panicked"))),
            }
        }

        Ok(results)
    }
}

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Rayon-based parallel processor (optional)
#[cfg(feature = "parallel")]
pub struct RayonProcessor;

#[cfg(feature = "parallel")]
impl RayonProcessor {
    /// Process items in parallel using Rayon
    pub fn process_parallel<I, O, F>(
        inputs: Vec<I>,
        processor: F,
    ) -> EngineResult<Vec<SemanticResult<O>>>
    where
        I: Send + Sync,
        O: Send + Sync,
        F: Fn(&I) -> EngineResult<O> + Send + Sync,
    {
        let results: Result<Vec<_>, _> = inputs
            .par_iter()
            .map(|input| {
                let start_time = Instant::now();

                match processor(input) {
                    Ok(output) => {
                        let processing_time = start_time.elapsed().as_micros() as u64;
                        Ok(SemanticResult::new(output, 1.0, false, processing_time))
                    }
                    Err(error) => Err(error),
                }
            })
            .collect();

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_processor_creation() {
        let processor = ParallelProcessor::new(4, true);
        assert_eq!(processor.thread_count(), 4);
        assert!(processor.is_enabled());
    }

    #[test]
    fn test_sequential_processing() {
        let processor = ParallelProcessor::new(1, false);
        let inputs = vec![1, 2, 3, 4, 5];

        let results = processor.process_parallel(inputs, |&x| Ok(x * 2)).unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[0].data, 2);
        assert_eq!(results[4].data, 10);
    }

    #[test]
    fn test_parallel_processing() {
        let processor = ParallelProcessor::new(2, true);
        let inputs = vec![1, 2, 3, 4, 5];

        let results = processor.process_parallel(inputs, |&x| Ok(x * 2)).unwrap();

        assert_eq!(results.len(), 5);
        // Results should be correct but may be in different order due to parallelism
        let mut values: Vec<_> = results.iter().map(|r| r.data).collect();
        values.sort();
        assert_eq!(values, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_multi_engine_coordinator() {
        let processor = ParallelProcessor::new(2, true);
        let coordinator = MultiEngineCoordinator::new(processor);

        let engines: Vec<Box<dyn Fn(&i32) -> EngineResult<i32> + Send + Sync>> =
            vec![Box::new(|&x| Ok(x + 1)), Box::new(|&x| Ok(x + 2))];

        let results = coordinator.query_engines(5, engines).unwrap();
        assert_eq!(results.len(), 2);

        // Both engines should succeed
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}
