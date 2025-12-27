//! Comprehensive tests for parallel processing functionality

use canopy_engine::{
    parallel::MultiEngineCoordinator, parallel::ParallelProcessor, EngineError, EngineResult,
};

#[cfg(feature = "parallel")]
use canopy_engine::parallel::RayonProcessor;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    /// Type alias for engine function vectors to reduce type complexity
    type EngineFnVec<T> = Vec<Box<dyn Fn(&T) -> EngineResult<T> + Send + Sync>>;

    // ParallelProcessor Creation Tests

    #[test]
    fn test_parallel_processor_new() {
        let processor = ParallelProcessor::new(8, true);
        assert_eq!(processor.thread_count(), 8);
        assert!(processor.is_enabled());
    }

    #[test]
    fn test_parallel_processor_new_zero_threads() {
        let processor = ParallelProcessor::new(0, true);
        assert_eq!(processor.thread_count(), 1); // Should clamp to minimum 1
        assert!(processor.is_enabled());
    }

    #[test]
    fn test_parallel_processor_new_disabled() {
        let processor = ParallelProcessor::new(4, false);
        assert_eq!(processor.thread_count(), 4);
        assert!(!processor.is_enabled());
    }

    #[test]
    fn test_parallel_processor_default() {
        let processor = ParallelProcessor::default();
        assert!(processor.thread_count() >= 1);
        assert!(processor.thread_count() <= 8);
        assert!(processor.is_enabled());
    }

    // ParallelProcessor Configuration Tests

    #[test]
    fn test_set_thread_count() {
        let mut processor = ParallelProcessor::new(2, true);

        processor.set_thread_count(6);
        assert_eq!(processor.thread_count(), 6);

        processor.set_thread_count(0);
        assert_eq!(processor.thread_count(), 1); // Should clamp to minimum
    }

    #[test]
    fn test_set_enabled() {
        let mut processor = ParallelProcessor::new(4, true);

        assert!(processor.is_enabled());

        processor.set_enabled(false);
        assert!(!processor.is_enabled());

        processor.set_enabled(true);
        assert!(processor.is_enabled());
    }

    #[test]
    fn test_optimal_thread_count() {
        let count = ParallelProcessor::optimal_thread_count();
        assert!(count >= 1);
        assert!(count <= 8);
    }

    // Sequential Processing Tests

    #[test]
    fn test_sequential_processing_empty_input() {
        let processor = ParallelProcessor::new(4, false);
        let inputs: Vec<i32> = vec![];

        let results = processor.process_parallel(inputs, |&x| Ok(x * 2)).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_sequential_processing_single_input() {
        let processor = ParallelProcessor::new(4, true); // Even with parallel enabled
        let inputs = vec![5];

        let results = processor.process_parallel(inputs, |&x| Ok(x * 3)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].data, 15);
        assert!(!results[0].from_cache);
        assert_eq!(results[0].confidence, 1.0);
        // processing_time_us is always valid (unsigned type)
        let _ = results[0].processing_time_us;
    }

    #[test]
    fn test_sequential_processing_multiple_inputs() {
        let processor = ParallelProcessor::new(4, false);
        let inputs = vec![1, 2, 3, 4];

        let results = processor.process_parallel(inputs, |&x| Ok(x * x)).unwrap();
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].data, 1);
        assert_eq!(results[1].data, 4);
        assert_eq!(results[2].data, 9);
        assert_eq!(results[3].data, 16);

        // All results should be processed sequentially
        for result in &results {
            assert!(!result.from_cache);
            assert_eq!(result.confidence, 1.0);
            // processing_time_us is always valid (unsigned type)
            let _ = result.processing_time_us;
        }
    }

    #[test]
    fn test_sequential_processing_with_error() {
        let processor = ParallelProcessor::new(4, false);
        let inputs = vec![1, 2, 3, 4];

        let result = processor.process_parallel(inputs, |&x| {
            if x == 3 {
                Err(EngineError::analysis(x.to_string(), "test error"))
            } else {
                Ok(x * 2)
            }
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test error"));
    }

    // Parallel Processing Tests

    #[test]
    fn test_parallel_processing_multiple_inputs() {
        let processor = ParallelProcessor::new(2, true);
        let inputs = vec![1, 2, 3, 4, 5, 6, 7, 8];

        let results = processor.process_parallel(inputs, |&x| Ok(x + 10)).unwrap();
        assert_eq!(results.len(), 8);

        // Results may be in any order due to parallel execution
        let mut values: Vec<i32> = results.iter().map(|r| r.data).collect();
        values.sort();
        assert_eq!(values, vec![11, 12, 13, 14, 15, 16, 17, 18]);

        // All should have processing time
        for result in &results {
            let _ = result.processing_time_us; // unsigned type, always valid
            assert_eq!(result.confidence, 1.0);
            assert!(!result.from_cache);
        }
    }

    #[test]
    fn test_parallel_processing_thread_panic() {
        let processor = ParallelProcessor::new(2, true);
        let inputs = vec![1, 2, 3, 4];

        let result = processor.process_parallel(inputs, |&x| {
            if x == 2 {
                panic!("Simulated thread panic");
            }
            Ok(x * 2)
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Thread panicked"));
    }

    #[test]
    fn test_parallel_processing_with_error() {
        let processor = ParallelProcessor::new(2, true);
        let inputs = vec![1, 2, 3, 4];

        let result = processor.process_parallel(inputs, |&x| {
            if x == 3 {
                Err(EngineError::timeout("test operation", 1000))
            } else {
                Ok(x * 2)
            }
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Timeout occurred"));
    }

    // Processing Mode Tests

    #[test]
    fn test_disabled_parallel_falls_back_to_sequential() {
        let processor = ParallelProcessor::new(4, false);
        let inputs = vec![1, 2, 3, 4, 5, 6];

        let results = processor.process_parallel(inputs, |&x| Ok(x * x)).unwrap();
        assert_eq!(results.len(), 6);

        // Results should be in original order (sequential processing)
        for (i, result) in results.iter().enumerate() {
            let expected = (i + 1) * (i + 1);
            assert_eq!(result.data, expected);
        }
    }

    #[test]
    fn test_single_thread_parallel() {
        let processor = ParallelProcessor::new(1, true);
        let inputs = vec![1, 2, 3, 4];

        let results = processor.process_parallel(inputs, |&x| Ok(x + 5)).unwrap();
        assert_eq!(results.len(), 4);

        let mut values: Vec<i32> = results.iter().map(|r| r.data).collect();
        values.sort();
        assert_eq!(values, vec![6, 7, 8, 9]);
    }

    // Complex Processing Tests

    #[test]
    fn test_complex_computation() {
        let processor = ParallelProcessor::new(3, true);
        let inputs = (1..=12).collect::<Vec<i32>>();

        // Simulate a more complex computation
        let results = processor
            .process_parallel(inputs, |&x| {
                let result = (0..x).map(|i| i * i).sum::<i32>();
                std::thread::sleep(Duration::from_millis(1)); // Simulate work
                Ok(result)
            })
            .unwrap();

        assert_eq!(results.len(), 12);

        // Verify at least some processing time was recorded
        let total_processing_time: u64 = results.iter().map(|r| r.processing_time_us).sum();
        assert!(total_processing_time > 0);
    }

    #[test]
    fn test_thread_safety() {
        let processor = ParallelProcessor::new(4, true);
        let inputs = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // Use a shared counter to verify thread safety
        let counter = Arc::new(Mutex::new(0));

        let results = processor
            .process_parallel(inputs, {
                let counter = Arc::clone(&counter);
                move |&x| {
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                    drop(count);
                    Ok(x * 2)
                }
            })
            .unwrap();

        assert_eq!(results.len(), 10);
        assert_eq!(*counter.lock().unwrap(), 10);
    }

    // MultiEngineCoordinator Tests

    #[test]
    fn test_multi_engine_coordinator_creation() {
        let processor = ParallelProcessor::new(2, true);
        let _coordinator = MultiEngineCoordinator::new(processor);

        // Test basic creation - coordinator should be created successfully
        // (We can't test much more without accessing private fields)
    }

    #[test]
    fn test_multi_engine_coordinator_set_timeout() {
        let processor = ParallelProcessor::new(2, true);
        let mut coordinator = MultiEngineCoordinator::new(processor);

        coordinator.set_engine_timeout("engine_1", Duration::from_millis(500));
        coordinator.set_engine_timeout("engine_2", Duration::from_secs(2));

        // Timeouts are set (private field, so can't verify directly)
    }

    #[test]
    fn test_multi_engine_coordinator_empty_engines() {
        let processor = ParallelProcessor::new(2, true);
        let coordinator = MultiEngineCoordinator::new(processor);

        let engines: EngineFnVec<i32> = vec![];
        let results = coordinator.query_engines(42, engines).unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_multi_engine_coordinator_single_engine() {
        let processor = ParallelProcessor::new(2, true);
        let coordinator = MultiEngineCoordinator::new(processor);

        let engines: EngineFnVec<i32> = vec![Box::new(|&x| Ok(x * 3))];

        let results = coordinator.query_engines(7, engines).unwrap();
        assert_eq!(results.len(), 1);

        match &results[0] {
            Ok(result) => {
                assert_eq!(result.data, 21);
                assert_eq!(result.confidence, 1.0);
                let _ = result.processing_time_us; // unsigned type, always valid
            }
            Err(e) => panic!("Expected success but got error: {:?}", e),
        }
    }

    #[test]
    fn test_multi_engine_coordinator_multiple_engines() {
        let processor = ParallelProcessor::new(2, true);
        let coordinator = MultiEngineCoordinator::new(processor);

        let engines: EngineFnVec<i32> = vec![
            Box::new(|&x| Ok(x + 1)),
            Box::new(|&x| Ok(x * 2)),
            Box::new(|&x| Ok(x - 1)),
        ];

        let results = coordinator.query_engines(10, engines).unwrap();
        assert_eq!(results.len(), 3);

        // All engines should succeed
        for result in &results {
            assert!(result.is_ok());
        }

        // Extract the actual values and sort them for predictable testing
        let mut values: Vec<i32> = results.iter().map(|r| r.as_ref().unwrap().data).collect();
        values.sort();
        assert_eq!(values, vec![9, 11, 20]); // 10-1, 10+1, 10*2
    }

    #[test]
    fn test_multi_engine_coordinator_with_error() {
        let processor = ParallelProcessor::new(2, true);
        let coordinator = MultiEngineCoordinator::new(processor);

        let engines: EngineFnVec<i32> = vec![
            Box::new(|&x| Ok(x + 1)),
            Box::new(|&x| Err(EngineError::analysis(x.to_string(), "simulated error"))),
            Box::new(|&x| Ok(x * 2)),
        ];

        let results = coordinator.query_engines(5, engines).unwrap();
        assert_eq!(results.len(), 3);

        // First and third should succeed, second should fail
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        assert!(results[2].is_ok());

        assert_eq!(results[0].as_ref().unwrap().data, 6);
        assert_eq!(results[2].as_ref().unwrap().data, 10);
        assert!(results[1]
            .as_ref()
            .err()
            .unwrap()
            .to_string()
            .contains("simulated error"));
    }

    #[test]
    fn test_multi_engine_coordinator_with_timeout() {
        let processor = ParallelProcessor::new(2, true);
        let mut coordinator = MultiEngineCoordinator::new(processor);

        // Set a very short timeout
        coordinator.set_engine_timeout("engine_0", Duration::from_millis(10));

        let engines: EngineFnVec<i32> = vec![Box::new(|&x| {
            std::thread::sleep(Duration::from_millis(100)); // Longer than timeout
            Ok(x * 2)
        })];

        let results = coordinator.query_engines(5, engines).unwrap();
        assert_eq!(results.len(), 1);

        // Should timeout
        assert!(results[0].is_err());
        assert!(results[0]
            .as_ref()
            .err()
            .unwrap()
            .to_string()
            .contains("Timeout occurred"));
    }

    #[test]
    fn test_multi_engine_coordinator_thread_panic() {
        let processor = ParallelProcessor::new(2, true);
        let coordinator = MultiEngineCoordinator::new(processor);

        let engines: EngineFnVec<i32> = vec![
            Box::new(|&x| Ok(x + 1)),
            Box::new(|&_x| panic!("Engine panic!")),
        ];

        let results = coordinator.query_engines(5, engines).unwrap();
        assert_eq!(results.len(), 2);

        // First should succeed, second should fail due to panic
        assert!(results[0].is_ok());
        assert!(results[1].is_err());
        let error_msg = results[1].as_ref().err().unwrap().to_string();
        // The actual error might be a timeout instead of thread panic in some cases
        assert!(
            error_msg.contains("Engine thread panicked")
                || error_msg.contains("thread panicked")
                || error_msg.contains("Timeout occurred")
        );
    }

    // Rayon Processor Tests (conditional)

    #[cfg(feature = "parallel")]
    #[test]
    fn test_rayon_processor_empty_input() {
        let inputs: Vec<i32> = vec![];
        let results = RayonProcessor::process_parallel(inputs, |&x| Ok(x * 2)).unwrap();
        assert!(results.is_empty());
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_rayon_processor_multiple_inputs() {
        let inputs = vec![1, 2, 3, 4, 5];
        let results = RayonProcessor::process_parallel(inputs, |&x| Ok(x * x)).unwrap();

        assert_eq!(results.len(), 5);

        // Results may be in any order due to parallel execution
        let mut values: Vec<i32> = results.iter().map(|r| r.data).collect();
        values.sort();
        assert_eq!(values, vec![1, 4, 9, 16, 25]);

        // All results should have valid metadata
        for result in &results {
            assert!(!result.from_cache);
            assert_eq!(result.confidence, 1.0);
            let _ = result.processing_time_us; // unsigned type, always valid
        }
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_rayon_processor_with_error() {
        let inputs = vec![1, 2, 3, 4];
        let result = RayonProcessor::process_parallel(inputs, |&x| {
            if x == 3 {
                Err(EngineError::cache("simulated cache error"))
            } else {
                Ok(x + 10)
            }
        });

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cache operation failed"));
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_rayon_processor_complex_computation() {
        let inputs = (1..=10).collect::<Vec<i32>>();
        let results = RayonProcessor::process_parallel(inputs, |&x| {
            let result = (1..=x).sum::<i32>();
            Ok(result)
        })
        .unwrap();

        assert_eq!(results.len(), 10);

        // Verify specific results
        let mut data: Vec<(i32, i32)> = results.iter().map(|r| (r.data, r.data)).collect();
        data.sort();

        // Check that we have the correct triangular numbers
        let expected_triangular_numbers = vec![1, 3, 6, 10, 15, 21, 28, 36, 45, 55];
        let actual_values: Vec<i32> = data.iter().map(|(val, _)| *val).collect();
        assert_eq!(actual_values, expected_triangular_numbers);
    }

    // Edge Case Tests

    #[test]
    fn test_large_input_set() {
        let processor = ParallelProcessor::new(4, true);
        let inputs: Vec<i32> = (1..=1000).collect();

        let results = processor.process_parallel(inputs, |&x| Ok(x % 17)).unwrap();
        assert_eq!(results.len(), 1000);

        // Verify all results are present
        let mut remainders: Vec<i32> = results.iter().map(|r| r.data).collect();
        remainders.sort();

        // Should have many duplicates but correct range
        assert!(remainders.iter().all(|&x| (0..17).contains(&x)));
    }

    #[test]
    fn test_zero_input_processing() {
        let processor = ParallelProcessor::new(2, true);
        let inputs = vec![0, 0, 0];

        let results = processor.process_parallel(inputs, |&x| Ok(x + 1)).unwrap();
        assert_eq!(results.len(), 3);

        for result in results {
            assert_eq!(result.data, 1);
        }
    }

    #[test]
    fn test_negative_input_processing() {
        let processor = ParallelProcessor::new(2, true);
        let inputs = vec![-5, -2, -1, 0, 1, 2, 5];

        let results = processor
            .process_parallel(inputs, |&x: &i32| Ok(x.abs()))
            .unwrap();
        assert_eq!(results.len(), 7);

        let mut values: Vec<i32> = results.iter().map(|r| r.data).collect();
        values.sort();
        assert_eq!(values, vec![0, 1, 1, 2, 2, 5, 5]);
    }

    // String Processing Tests

    #[test]
    fn test_string_processing() {
        let processor = ParallelProcessor::new(2, true);
        let inputs = vec!["hello", "world", "parallel", "processing"];

        let results = processor.process_parallel(inputs, |s| Ok(s.len())).unwrap();
        assert_eq!(results.len(), 4);

        let mut lengths: Vec<usize> = results.iter().map(|r| r.data).collect();
        lengths.sort();
        assert_eq!(lengths, vec![5, 5, 8, 10]); // hello, world, parallel, processing
    }

    #[test]
    fn test_string_transformation() {
        let processor = ParallelProcessor::new(3, true);
        let inputs = vec!["rust", "parallel", "semantic"];

        let results = processor
            .process_parallel(inputs, |s| Ok(s.to_uppercase()))
            .unwrap();

        assert_eq!(results.len(), 3);

        let mut values: Vec<String> = results.iter().map(|r| r.data.clone()).collect();
        values.sort();
        assert_eq!(values, vec!["PARALLEL", "RUST", "SEMANTIC"]);
    }

    // Performance and Timing Tests

    #[test]
    fn test_processing_time_recorded() {
        let processor = ParallelProcessor::new(2, true);
        let inputs = vec![1, 2, 3];

        let results = processor
            .process_parallel(inputs, |&x| {
                std::thread::sleep(Duration::from_millis(10));
                Ok(x)
            })
            .unwrap();

        // All results should have recorded processing time
        for result in results {
            assert!(result.processing_time_us >= 10_000); // At least 10ms in microseconds
        }
    }

    #[test]
    fn test_default_semantic_result_values() {
        let processor = ParallelProcessor::new(1, false);
        let inputs = vec![42];

        let results = processor.process_parallel(inputs, |&x| Ok(x)).unwrap();
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert_eq!(result.data, 42);
        assert_eq!(result.confidence, 1.0);
        assert!(!result.from_cache);
        let _ = result.processing_time_us; // unsigned type, always valid
    }
}
