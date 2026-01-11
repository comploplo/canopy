//! Code actions handler
//!
//! Provides quick fixes and refactoring suggestions for diagnostics.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::backend::CanopyBackend;

/// Diagnostic codes that we can provide actions for.
pub mod codes {
    // Original diagnostics
    pub const LOW_CONFIDENCE: &str = "low-confidence";
    pub const AMBIGUOUS_PREDICATE: &str = "ambiguous-predicate";
    pub const CONTRADICTION: &str = "contradiction";
    pub const PRESUPPOSITION_FAILURE: &str = "presupposition-failure";
    pub const UNBOUND_ARGUMENT: &str = "unbound-argument";

    // New diagnostics (LSP enhancements)
    pub const PRONOUN_AMBIGUOUS: &str = "pronoun-ambiguous";
    pub const PRONOUN_UNRESOLVED: &str = "pronoun-unresolved";
    pub const BINDING_VIOLATION: &str = "binding-violation";
    pub const SCOPE_AMBIGUOUS: &str = "scope-ambiguous";
    pub const CONFLICT_DETAIL: &str = "conflict-detail";
}

/// Build code actions for a specific diagnostic.
pub fn build_actions_for_diagnostic(
    uri: &Url,
    diagnostic: &Diagnostic,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();

    let code = match &diagnostic.code {
        Some(NumberOrString::String(s)) => s.as_str(),
        _ => return actions,
    };

    match code {
        codes::LOW_CONFIDENCE => {
            // Action: Acknowledge low confidence (dismiss diagnostic)
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Acknowledge low confidence binding".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Acknowledge".to_string(),
                    command: "canopy.acknowledgeBinding".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));

            // Action: Show alternative bindings
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Show alternative role bindings".to_string(),
                kind: Some(CodeActionKind::new("quickfix.showAlternatives")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Show Alternatives".to_string(),
                    command: "canopy.showAlternativeBindings".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }

        codes::AMBIGUOUS_PREDICATE => {
            // Action: Show predicate senses
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Show predicate senses".to_string(),
                kind: Some(CodeActionKind::new("quickfix.showSenses")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Show Senses".to_string(),
                    command: "canopy.showPredicateSenses".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));
        }

        codes::CONTRADICTION => {
            // Action: Explain contradiction
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Explain contradiction".to_string(),
                kind: Some(CodeActionKind::new("quickfix.explain")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Explain".to_string(),
                    command: "canopy.explainContradiction".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));
        }

        codes::PRESUPPOSITION_FAILURE => {
            // Action: Show presupposition details
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Show presupposition details".to_string(),
                kind: Some(CodeActionKind::new("quickfix.showDetails")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Show Details".to_string(),
                    command: "canopy.showPresupposition".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }

        codes::UNBOUND_ARGUMENT => {
            // Action: Suggest role assignment
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Suggest role for unbound argument".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Suggest Role".to_string(),
                    command: "canopy.suggestRole".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));
        }

        // =====================================================================
        // New diagnostic code actions
        // =====================================================================
        codes::PRONOUN_AMBIGUOUS => {
            // Action: Show binding candidates
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Show binding candidates".to_string(),
                kind: Some(CodeActionKind::new("quickfix.showCandidates")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Show Candidates".to_string(),
                    command: "canopy.showBindingCandidates".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));

            // Action: Select preferred antecedent
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Select preferred antecedent".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Select Antecedent".to_string(),
                    command: "canopy.selectAntecedent".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }

        codes::PRONOUN_UNRESOLVED => {
            // Action: Show why binding failed
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Explain binding failure".to_string(),
                kind: Some(CodeActionKind::new("quickfix.explain")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Explain".to_string(),
                    command: "canopy.explainBindingFailure".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));
        }

        codes::BINDING_VIOLATION => {
            // Action: Explain binding theory constraint
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Explain constraint violation".to_string(),
                kind: Some(CodeActionKind::new("quickfix.explain")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Explain".to_string(),
                    command: "canopy.explainBindingConstraint".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));
        }

        codes::SCOPE_AMBIGUOUS => {
            // Action: Show scope readings
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Show scope readings".to_string(),
                kind: Some(CodeActionKind::new("quickfix.showReadings")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Show Readings".to_string(),
                    command: "canopy.showScopeReadings".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));

            // Action: Disambiguate scope
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Select preferred scope".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Select Scope".to_string(),
                    command: "canopy.selectScope".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }

        codes::CONFLICT_DETAIL => {
            // Action: Explain conflict in detail
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Explain conflict details".to_string(),
                kind: Some(CodeActionKind::new("quickfix.explain")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Explain".to_string(),
                    command: "canopy.explainConflict".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));

            // Action: Show both conditions
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Show conflicting conditions".to_string(),
                kind: Some(CodeActionKind::new("quickfix.showDetails")),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: Some(Command {
                    title: "Show Conditions".to_string(),
                    command: "canopy.showConflictingConditions".to_string(),
                    arguments: Some(vec![
                        serde_json::to_value(uri.to_string()).unwrap(),
                        serde_json::to_value(diagnostic.range).unwrap(),
                    ]),
                }),
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }

        _ => {}
    }

    actions
}

/// Handle code action request.
pub fn handle_code_actions(
    backend: &CanopyBackend,
    params: &CodeActionParams,
) -> Result<Option<CodeActionResponse>> {
    let uri = &params.text_document.uri;

    // Check if document exists
    if backend.documents().get(uri).is_none() {
        return Ok(None);
    }

    let mut actions = Vec::new();

    // Process diagnostics in the context
    for diagnostic in &params.context.diagnostics {
        // Only process canopy diagnostics
        if diagnostic.source.as_deref() != Some("canopy") {
            continue;
        }

        let diagnostic_actions = build_actions_for_diagnostic(uri, diagnostic);
        actions.extend(diagnostic_actions);
    }

    // Add general code actions (not tied to specific diagnostics)
    // These appear in the lightbulb menu

    // Action: Analyze selection
    if params.range.start != params.range.end {
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Analyze semantic structure".to_string(),
            kind: Some(CodeActionKind::new("source.analyze")),
            diagnostics: None,
            edit: None,
            command: Some(Command {
                title: "Analyze".to_string(),
                command: "canopy.analyzeSelection".to_string(),
                arguments: Some(vec![
                    serde_json::to_value(uri.to_string()).unwrap(),
                    serde_json::to_value(params.range).unwrap(),
                ]),
            }),
            is_preferred: Some(false),
            disabled: None,
            data: None,
        }));
    }

    if actions.is_empty() {
        return Ok(None);
    }

    Ok(Some(actions))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diagnostic(code: &str, message: &str) -> Diagnostic {
        Diagnostic {
            range: Range::new(Position::new(0, 0), Position::new(0, 10)),
            severity: Some(DiagnosticSeverity::INFORMATION),
            source: Some("canopy".to_string()),
            message: message.to_string(),
            code: Some(NumberOrString::String(code.to_string())),
            ..Default::default()
        }
    }

    #[test]
    fn test_low_confidence_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic(codes::LOW_CONFIDENCE, "Low confidence for role Agent");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 2);

        // Check first action
        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("Acknowledge"));
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
        } else {
            panic!("Expected CodeAction");
        }

        // Check second action
        if let CodeActionOrCommand::CodeAction(action) = &actions[1] {
            assert!(action.title.contains("alternative"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_ambiguous_predicate_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic(codes::AMBIGUOUS_PREDICATE, "Multiple senses possible");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 1);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("senses"));
            assert_eq!(action.is_preferred, Some(true));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_contradiction_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic(codes::CONTRADICTION, "Contradiction detected");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 1);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("Explain"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_unbound_argument_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic(codes::UNBOUND_ARGUMENT, "Unbound argument");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 1);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("Suggest"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_unknown_diagnostic_no_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic("unknown-code", "Unknown issue");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert!(actions.is_empty());
    }

    #[test]
    fn test_non_canopy_diagnostic_code() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let mut diagnostic = make_diagnostic(codes::LOW_CONFIDENCE, "Test");
        diagnostic.code = Some(NumberOrString::Number(123)); // Number code

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert!(actions.is_empty());
    }

    // Tests for new diagnostic code actions

    #[test]
    fn test_pronoun_ambiguous_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic(codes::PRONOUN_AMBIGUOUS, "he has multiple antecedents");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 2);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("candidates"));
            assert_eq!(action.is_preferred, Some(true));
        } else {
            panic!("Expected CodeAction");
        }

        if let CodeActionOrCommand::CodeAction(action) = &actions[1] {
            assert!(action.title.contains("antecedent"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_pronoun_unresolved_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic =
            make_diagnostic(codes::PRONOUN_UNRESOLVED, "it has no accessible antecedent");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 1);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("Explain"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_binding_violation_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic(codes::BINDING_VIOLATION, "Condition A violation");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 1);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("constraint"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_scope_ambiguous_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic(codes::SCOPE_AMBIGUOUS, "every > some vs some > every");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 2);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("readings"));
            assert_eq!(action.is_preferred, Some(true));
        } else {
            panic!("Expected CodeAction");
        }

        if let CodeActionOrCommand::CodeAction(action) = &actions[1] {
            assert!(action.title.contains("scope"));
        } else {
            panic!("Expected CodeAction");
        }
    }

    #[test]
    fn test_conflict_detail_actions() {
        let uri = Url::parse("file:///test.txt").unwrap();
        let diagnostic = make_diagnostic(codes::CONFLICT_DETAIL, "Polarity conflict: P vs NOT P");

        let actions = build_actions_for_diagnostic(&uri, &diagnostic);

        assert_eq!(actions.len(), 2);

        if let CodeActionOrCommand::CodeAction(action) = &actions[0] {
            assert!(action.title.contains("Explain"));
            assert_eq!(action.is_preferred, Some(true));
        } else {
            panic!("Expected CodeAction");
        }

        if let CodeActionOrCommand::CodeAction(action) = &actions[1] {
            assert!(action.title.contains("conditions"));
        } else {
            panic!("Expected CodeAction");
        }
    }
}
