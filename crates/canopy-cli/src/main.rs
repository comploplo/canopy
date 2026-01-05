/// Main entry point - testable version
fn main() {
    let result = main_impl();
    if let Err(code) = result {
        std::process::exit(code);
    }
}

/// Testable main implementation that returns exit code instead of calling exit
fn main_impl() -> Result<(), i32> {
    main_impl_with_cli(canopy_cli::run_cli)
}

/// Main implementation with injectable CLI function for testing
fn main_impl_with_cli<F>(cli_fn: F) -> Result<(), i32>
where
    F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
{
    match cli_fn() {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Error: {e}");
            Err(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_error_path_coverage() {
        // Test the error path in main_impl using dependency injection
        let mock_error_cli = || -> Result<(), Box<dyn std::error::Error>> {
            Err("Mock CLI error for testing".into())
        };

        // Test main_impl_with_cli with error condition
        let result = main_impl_with_cli(mock_error_cli);

        // Should return error exit code
        assert!(result.is_err(), "Should return error when CLI fails");
        assert_eq!(result.unwrap_err(), 1, "Should return exit code 1");
    }

    #[test]
    fn test_main_success_path_coverage() {
        // Test the success path in main_impl using dependency injection
        let mock_success_cli = || -> Result<(), Box<dyn std::error::Error>> { Ok(()) };

        // Test main_impl_with_cli with success condition
        let result = main_impl_with_cli(mock_success_cli);

        // Should return success
        assert!(result.is_ok(), "Should return success when CLI succeeds");
    }

    #[test]
    fn test_main_impl_with_cli_coverage() {
        // Additional coverage for main_impl_with_cli function
        // Test with various error types
        let error_with_message =
            || -> Result<(), Box<dyn std::error::Error>> { Err("Custom error".into()) };

        let result = main_impl_with_cli(error_with_message);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), 1);

        // Test success case again for branch coverage
        let success = || -> Result<(), Box<dyn std::error::Error>> { Ok(()) };
        assert!(main_impl_with_cli(success).is_ok());
    }
}
