//! LSP Handler Tests
//!
//! Tests for LSP protocol handlers including hover, diagnostics, and helper functions.

use crate::handlers::{
    Diagnostic, DiagnosticHandler, DiagnosticSeverity, HoverHandler, HoverResponse, Position,
    Range, create_diagnostic,
};
// Core types for testing are included in the handlers module

#[cfg(test)]
mod handler_tests {
    use super::*;

    #[test]
    fn test_hover_handler_creation() {
        // Test that HoverHandler can be created
        let handler = HoverHandler;

        // Verify the handler exists (this tests the struct instantiation)
        // The actual hover functionality is TODO, but we test the structure
        assert_eq!(
            std::mem::size_of_val(&handler),
            0,
            "HoverHandler should be zero-sized"
        );
    }

    #[test]
    fn test_diagnostic_handler_creation() {
        // Test that DiagnosticHandler can be created
        let handler = DiagnosticHandler;

        // Verify the handler exists (this tests the struct instantiation)
        assert_eq!(
            std::mem::size_of_val(&handler),
            0,
            "DiagnosticHandler should be zero-sized"
        );
    }

    #[test]
    fn test_position_creation_and_fields() {
        // Test Position struct creation and field access
        let pos = Position {
            line: 10,
            character: 25,
        };

        assert_eq!(pos.line, 10, "Position line should be set correctly");
        assert_eq!(
            pos.character, 25,
            "Position character should be set correctly"
        );
    }

    #[test]
    fn test_position_clone_and_debug() {
        // Test Position Clone and Debug traits
        let pos = Position {
            line: 5,
            character: 15,
        };

        let cloned_pos = pos.clone();
        assert_eq!(
            pos.line, cloned_pos.line,
            "Cloned position should have same line"
        );
        assert_eq!(
            pos.character, cloned_pos.character,
            "Cloned position should have same character"
        );

        // Test debug formatting
        let debug_str = format!("{:?}", pos);
        assert!(
            debug_str.contains("Position"),
            "Debug output should contain Position"
        );
        assert!(
            debug_str.contains("5"),
            "Debug output should contain line number"
        );
        assert!(
            debug_str.contains("15"),
            "Debug output should contain character number"
        );
    }

    #[test]
    fn test_hover_response_creation() {
        // Test HoverResponse creation and field access
        let response = HoverResponse {
            contents: "Test hover content".to_string(),
        };

        assert_eq!(
            response.contents, "Test hover content",
            "Hover contents should be set correctly"
        );
    }

    #[test]
    fn test_hover_response_clone_and_debug() {
        // Test HoverResponse Clone and Debug traits
        let response = HoverResponse {
            contents: "Cloneable content".to_string(),
        };

        let cloned_response = response.clone();
        assert_eq!(
            response.contents, cloned_response.contents,
            "Cloned response should have same contents"
        );

        // Test debug formatting
        let debug_str = format!("{:?}", response);
        assert!(
            debug_str.contains("HoverResponse"),
            "Debug output should contain HoverResponse"
        );
        assert!(
            debug_str.contains("Cloneable content"),
            "Debug output should contain contents"
        );
    }

    #[test]
    fn test_range_creation() {
        // Test Range struct creation with Position fields
        let start_pos = Position {
            line: 1,
            character: 0,
        };
        let end_pos = Position {
            line: 1,
            character: 10,
        };

        let range = Range {
            start: start_pos,
            end: end_pos,
        };

        assert_eq!(range.start.line, 1, "Range start line should be correct");
        assert_eq!(
            range.start.character, 0,
            "Range start character should be correct"
        );
        assert_eq!(range.end.line, 1, "Range end line should be correct");
        assert_eq!(
            range.end.character, 10,
            "Range end character should be correct"
        );
    }

    #[test]
    fn test_diagnostic_severity_variants() {
        // Test all DiagnosticSeverity variants
        let error = DiagnosticSeverity::Error;
        let warning = DiagnosticSeverity::Warning;
        let info = DiagnosticSeverity::Information;
        let hint = DiagnosticSeverity::Hint;

        // Test debug formatting for each variant
        assert_eq!(format!("{:?}", error), "Error");
        assert_eq!(format!("{:?}", warning), "Warning");
        assert_eq!(format!("{:?}", info), "Information");
        assert_eq!(format!("{:?}", hint), "Hint");
    }

    #[test]
    fn test_diagnostic_creation_with_all_severities() {
        // Test Diagnostic creation with each severity level
        let severities = vec![
            DiagnosticSeverity::Error,
            DiagnosticSeverity::Warning,
            DiagnosticSeverity::Information,
            DiagnosticSeverity::Hint,
        ];

        for (i, severity) in severities.into_iter().enumerate() {
            let diagnostic = Diagnostic {
                message: format!("Test message {}", i),
                severity: severity.clone(),
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: 5,
                    },
                },
            };

            assert_eq!(diagnostic.message, format!("Test message {}", i));
            assert!(format!("{:?}", diagnostic.severity) == format!("{:?}", severity));
            assert_eq!(diagnostic.range.start.line, i as u32);
        }
    }

    #[test]
    fn test_create_diagnostic_helper_function() {
        // Test the create_diagnostic helper function
        let diagnostic = create_diagnostic(
            "Test error message".to_string(),
            DiagnosticSeverity::Error,
            5,
            10,
        );

        assert_eq!(diagnostic.message, "Test error message");
        assert!(matches!(diagnostic.severity, DiagnosticSeverity::Error));
        assert_eq!(diagnostic.range.start.line, 5);
        assert_eq!(diagnostic.range.start.character, 10);
        assert_eq!(diagnostic.range.end.line, 5);
        assert_eq!(diagnostic.range.end.character, 11); // character + 1
    }

    #[test]
    fn test_create_diagnostic_with_different_positions() {
        // Test create_diagnostic with various position values
        let test_cases = vec![
            (0, 0, "Start of file"),
            (100, 50, "Middle of large file"),
            (u32::MAX - 1, u32::MAX - 1, "Near maximum values"),
        ];

        for (line, character, description) in test_cases {
            let diagnostic = create_diagnostic(
                description.to_string(),
                DiagnosticSeverity::Information,
                line,
                character,
            );

            assert_eq!(diagnostic.message, description);
            assert_eq!(diagnostic.range.start.line, line);
            assert_eq!(diagnostic.range.start.character, character);
            assert_eq!(diagnostic.range.end.line, line);
            assert_eq!(diagnostic.range.end.character, character.saturating_add(1));
        }
    }

    #[test]
    fn test_create_diagnostic_with_all_severity_levels() {
        // Test create_diagnostic with each severity level
        let severities = vec![
            (DiagnosticSeverity::Error, "Error message"),
            (DiagnosticSeverity::Warning, "Warning message"),
            (DiagnosticSeverity::Information, "Info message"),
            (DiagnosticSeverity::Hint, "Hint message"),
        ];

        for (severity, message) in severities {
            let diagnostic = create_diagnostic(message.to_string(), severity.clone(), 1, 1);

            assert_eq!(diagnostic.message, message);
            assert!(format!("{:?}", diagnostic.severity) == format!("{:?}", severity));
            assert_eq!(diagnostic.range.start.line, 1);
            assert_eq!(diagnostic.range.start.character, 1);
            assert_eq!(diagnostic.range.end.line, 1);
            assert_eq!(diagnostic.range.end.character, 2);
        }
    }

    #[test]
    fn test_diagnostic_clone_and_debug() {
        // Test Diagnostic Clone and Debug implementations
        let original = create_diagnostic(
            "Cloneable diagnostic".to_string(),
            DiagnosticSeverity::Warning,
            3,
            7,
        );

        let cloned = original.clone();

        // Verify all fields are cloned correctly
        assert_eq!(original.message, cloned.message);
        assert_eq!(
            format!("{:?}", original.severity),
            format!("{:?}", cloned.severity)
        );
        assert_eq!(original.range.start.line, cloned.range.start.line);
        assert_eq!(original.range.start.character, cloned.range.start.character);
        assert_eq!(original.range.end.line, cloned.range.end.line);
        assert_eq!(original.range.end.character, cloned.range.end.character);

        // Test debug formatting
        let debug_str = format!("{:?}", original);
        assert!(
            debug_str.contains("Diagnostic"),
            "Debug should contain Diagnostic"
        );
        assert!(
            debug_str.contains("Cloneable diagnostic"),
            "Debug should contain message"
        );
        assert!(
            debug_str.contains("Warning"),
            "Debug should contain severity"
        );
    }

    #[test]
    fn test_range_clone_and_debug() {
        // Test Range Clone and Debug implementations
        let range = Range {
            start: Position {
                line: 2,
                character: 4,
            },
            end: Position {
                line: 2,
                character: 8,
            },
        };

        let cloned_range = range.clone();

        // Verify cloning preserves all data
        assert_eq!(range.start.line, cloned_range.start.line);
        assert_eq!(range.start.character, cloned_range.start.character);
        assert_eq!(range.end.line, cloned_range.end.line);
        assert_eq!(range.end.character, cloned_range.end.character);

        // Test debug formatting
        let debug_str = format!("{:?}", range);
        assert!(debug_str.contains("Range"), "Debug should contain Range");
        assert!(
            debug_str.contains("start"),
            "Debug should contain start field"
        );
        assert!(debug_str.contains("end"), "Debug should contain end field");
    }

    #[test]
    fn test_edge_case_character_overflow() {
        // Test edge case where character + 1 might overflow
        let diagnostic = create_diagnostic(
            "Overflow test".to_string(),
            DiagnosticSeverity::Error,
            0,
            u32::MAX,
        );

        assert_eq!(diagnostic.range.start.character, u32::MAX);
        // Should handle overflow gracefully (saturating add)
        assert_eq!(diagnostic.range.end.character, u32::MAX);
    }

    #[test]
    fn test_empty_message_diagnostic() {
        // Test diagnostic with empty message
        let diagnostic = create_diagnostic(String::new(), DiagnosticSeverity::Information, 0, 0);

        assert_eq!(diagnostic.message, "");
        assert_eq!(diagnostic.range.start.line, 0);
        assert_eq!(diagnostic.range.start.character, 0);
        assert_eq!(diagnostic.range.end.line, 0);
        assert_eq!(diagnostic.range.end.character, 1);
    }

    #[test]
    fn test_long_message_diagnostic() {
        // Test diagnostic with very long message
        let long_message = "A".repeat(1000);
        let diagnostic = create_diagnostic(long_message.clone(), DiagnosticSeverity::Hint, 10, 20);

        assert_eq!(diagnostic.message, long_message);
        assert_eq!(diagnostic.message.len(), 1000);
        assert_eq!(diagnostic.range.start.line, 10);
        assert_eq!(diagnostic.range.start.character, 20);
    }
}
