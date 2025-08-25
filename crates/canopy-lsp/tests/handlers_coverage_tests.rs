//! Tests for LSP handlers to achieve coverage targets

use canopy_lsp::handlers::{
    Diagnostic, DiagnosticHandler, DiagnosticSeverity, HoverHandler, HoverResponse, Position,
    Range, create_diagnostic,
};

#[test]
fn test_diagnostic_creation_all_severities() {
    // Test creating diagnostics with all severity levels
    let error_diag =
        create_diagnostic("Error message".to_string(), DiagnosticSeverity::Error, 0, 0);

    assert_eq!(error_diag.message, "Error message");
    assert_eq!(error_diag.range.start.line, 0);
    assert_eq!(error_diag.range.end.character, 1);

    let warning_diag = create_diagnostic(
        "Warning message".to_string(),
        DiagnosticSeverity::Warning,
        1,
        5,
    );

    assert_eq!(warning_diag.message, "Warning message");
    assert_eq!(warning_diag.range.start.line, 1);
    assert_eq!(warning_diag.range.start.character, 5);

    let info_diag = create_diagnostic(
        "Info message".to_string(),
        DiagnosticSeverity::Information,
        2,
        0,
    );

    assert_eq!(info_diag.message, "Info message");
    assert_eq!(info_diag.range.start.line, 2);

    let hint_diag = create_diagnostic("Hint message".to_string(), DiagnosticSeverity::Hint, 3, 10);

    assert_eq!(hint_diag.message, "Hint message");
    assert_eq!(hint_diag.range.start.character, 10);
}

#[test]
fn test_position_operations() {
    let pos1 = Position {
        line: 5,
        character: 10,
    };
    let pos2 = Position {
        line: 5,
        character: 10,
    };
    let pos3 = Position {
        line: 6,
        character: 10,
    };

    // Test field access
    assert_eq!(pos1.line, pos2.line);
    assert_eq!(pos1.character, pos2.character);
    assert_ne!(pos1.line, pos3.line);

    // Test clone and debug
    let cloned = pos1.clone();
    assert_eq!(pos1.line, cloned.line);
    assert_eq!(pos1.character, cloned.character);

    // Debug should work
    let debug_str = format!("{:?}", pos1);
    assert!(debug_str.contains("line"));
    assert!(debug_str.contains("character"));
}

#[test]
fn test_range_operations() {
    let range = Range {
        start: Position {
            line: 1,
            character: 5,
        },
        end: Position {
            line: 1,
            character: 15,
        },
    };

    // Test clone and debug
    let cloned = range.clone();
    assert_eq!(range.start.line, cloned.start.line);
    assert_eq!(range.end.character, cloned.end.character);

    // Test debug output
    let debug_str = format!("{:?}", range);
    assert!(debug_str.contains("start"));
    assert!(debug_str.contains("end"));
}

#[test]
fn test_diagnostic_severity_variants() {
    // Test all diagnostic severity variants
    let severities = vec![
        DiagnosticSeverity::Error,
        DiagnosticSeverity::Warning,
        DiagnosticSeverity::Information,
        DiagnosticSeverity::Hint,
    ];

    for severity in severities {
        let diag = create_diagnostic("Test message".to_string(), severity, 0, 0);

        assert_eq!(diag.message, "Test message");

        // Test clone and debug
        let cloned = diag.clone();
        assert_eq!(diag.message, cloned.message);
        assert_eq!(diag.range.start.line, cloned.range.start.line);

        let debug_str = format!("{:?}", diag);
        assert!(debug_str.contains("message"));
        assert!(debug_str.contains("severity"));
    }
}

#[test]
fn test_hover_response_creation() {
    // Test creating hover responses
    let hover = HoverResponse {
        contents: "Type information".to_string(),
    };
    assert_eq!(hover.contents, "Type information");

    let hover_empty = HoverResponse {
        contents: "".to_string(),
    };
    assert_eq!(hover_empty.contents, "");

    // Test with longer content
    let long_content = "This is a longer hover message with detailed type information and examples";
    let hover_long = HoverResponse {
        contents: long_content.to_string(),
    };
    assert_eq!(hover_long.contents, long_content);
}

#[test]
fn test_hover_response_operations() {
    let hover = HoverResponse {
        contents: "Hover content".to_string(),
    };

    // Test clone
    let cloned = hover.clone();
    assert_eq!(hover.contents, cloned.contents);

    // Test debug
    let debug_str = format!("{:?}", hover);
    assert!(debug_str.contains("contents"));
    assert!(debug_str.contains("Hover content"));
}

#[test]
fn test_diagnostic_with_edge_case_positions() {
    // Test with zero positions
    let zero_diag = create_diagnostic("Zero position".to_string(), DiagnosticSeverity::Error, 0, 0);

    assert_eq!(zero_diag.range.start.line, 0);
    assert_eq!(zero_diag.range.start.character, 0);
    assert_eq!(zero_diag.range.end.line, 0);
    assert_eq!(zero_diag.range.end.character, 1);

    // Test with large positions
    let large_diag = create_diagnostic(
        "Large position".to_string(),
        DiagnosticSeverity::Warning,
        1000,
        500,
    );

    assert_eq!(large_diag.range.start.line, 1000);
    assert_eq!(large_diag.range.start.character, 500);
    assert_eq!(large_diag.range.end.line, 1000);
    assert_eq!(large_diag.range.end.character, 501);
}

#[test]
fn test_diagnostic_with_special_characters() {
    // Test diagnostic messages with special characters
    let special_diag = create_diagnostic(
        "Error: 'undefined' symbol at line 42, column 10".to_string(),
        DiagnosticSeverity::Error,
        41,
        9,
    );

    assert!(special_diag.message.contains("'undefined'"));
    assert!(special_diag.message.contains("42"));

    // Test with unicode characters
    let unicode_diag = create_diagnostic(
        "Erreur: caractère invalide 'é'".to_string(),
        DiagnosticSeverity::Error,
        5,
        10,
    );

    assert!(unicode_diag.message.contains("é"));
}

#[test]
fn test_empty_and_long_messages() {
    // Test with empty message
    let empty_diag = create_diagnostic("".to_string(), DiagnosticSeverity::Hint, 0, 0);

    assert_eq!(empty_diag.message, "");

    // Test with very long message
    let long_message = "This is a very long diagnostic message that contains a lot of detailed information about what went wrong and how to fix it. It should still be handled correctly by the diagnostic creation function.".to_string();
    let long_diag = create_diagnostic(long_message.clone(), DiagnosticSeverity::Information, 10, 0);

    assert_eq!(long_diag.message, long_message);
    assert!(long_diag.message.len() > 100);
}

#[test]
fn test_multiline_range_simulation() {
    // Test diagnostic spanning multiple lines (simulated with different line positions)
    let start_diag = create_diagnostic(
        "Multi-line error start".to_string(),
        DiagnosticSeverity::Error,
        5,
        20,
    );

    let end_diag = create_diagnostic(
        "Multi-line error end".to_string(),
        DiagnosticSeverity::Error,
        8,
        10,
    );

    assert_eq!(start_diag.range.start.line, 5);
    assert_eq!(end_diag.range.start.line, 8);
    assert_ne!(start_diag.range.start.line, end_diag.range.start.line);
}

#[test]
fn test_handler_creation() {
    // Test handler creation and basic operations
    let hover_handler = HoverHandler;
    let diagnostic_handler = DiagnosticHandler;

    // Test debug formatting
    let hover_debug = format!("{:?}", hover_handler);
    assert!(hover_debug.contains("HoverHandler"));

    let diag_debug = format!("{:?}", diagnostic_handler);
    assert!(diag_debug.contains("DiagnosticHandler"));
}

#[test]
fn test_character_overflow_handling() {
    // Test character overflow in diagnostic creation
    let max_char = u32::MAX;
    let diag = create_diagnostic(
        "Overflow test".to_string(),
        DiagnosticSeverity::Error,
        0,
        max_char,
    );

    // Should use saturating_add to handle overflow
    assert_eq!(diag.range.start.character, max_char);
    assert_eq!(diag.range.end.character, max_char); // saturating_add should prevent overflow
}

#[test]
fn test_diagnostic_severity_debug() {
    // Test debug output for all severity variants
    let error_debug = format!("{:?}", DiagnosticSeverity::Error);
    assert!(error_debug.contains("Error"));

    let warning_debug = format!("{:?}", DiagnosticSeverity::Warning);
    assert!(warning_debug.contains("Warning"));

    let info_debug = format!("{:?}", DiagnosticSeverity::Information);
    assert!(info_debug.contains("Information"));

    let hint_debug = format!("{:?}", DiagnosticSeverity::Hint);
    assert!(hint_debug.contains("Hint"));
}
