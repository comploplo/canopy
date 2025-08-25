//! Basic tests for api.rs module structures

use canopy_pipeline::api::*;

#[cfg(test)]
mod api_tests {
    use super::*;

    #[test]
    fn test_analyzer_creation_panics() {
        // Test that the unimplemented new method panics as expected
        let result = std::panic::catch_unwind(|| CanopyAnalyzer::new(None));
        assert!(result.is_err());
    }

    #[test]
    fn test_analyzer_creation_with_path_panics() {
        // Test that the unimplemented new method panics as expected
        let result = std::panic::catch_unwind(|| CanopyAnalyzer::new(Some("/path/to/model")));
        assert!(result.is_err());
    }
}
