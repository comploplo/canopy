//! Canopy CLI library
//!
//! This module exposes testable functions for the CLI to achieve test coverage.

use canopy_resources::CanopyPipeline;
use clap::{Parser, ValueEnum};

/// Canopy semantic analysis CLI.
#[derive(Parser, Debug)]
#[command(name = "canopy")]
#[command(
    version,
    about = "Semantic analysis powered by VerbNet, FrameNet, WordNet, and PropBank"
)]
pub struct Cli {
    /// Text to analyze (pass as argument or via stdin)
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,

    /// Enable derivation trace output
    #[arg(long, short = 't')]
    pub trace: bool,

    /// Analyze as multi-sentence document
    #[arg(long, short = 'd')]
    pub document: bool,

    /// Output format
    #[arg(long, short = 'f', value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Test error flag for coverage testing
    #[arg(long, hide = true)]
    pub test_error: bool,
}

/// Output format for analysis results.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// Plain text output (default)
    #[default]
    Text,
    /// JSON output
    Json,
}

/// Main CLI entry point (testable version)
///
/// # Errors
/// Returns an error if CLI execution fails.
pub fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    run_cli_with_args(&args)
}

/// CLI implementation with injectable arguments for testing
///
/// # Errors
/// Returns an error if CLI execution fails or `--test-error` flag is passed.
pub fn run_cli_with_args(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Let clap handle parsing naturally - it will read stdin if no text argument
    let cli = Cli::parse_from(args);

    // Check for test error flag
    if cli.test_error {
        return Err("Test error condition".into());
    }

    // Get text from argument or stdin (supports `echo "text" | canopy`)
    let text = if let Some(ref t) = cli.text {
        t.clone()
    } else {
        // Try to read from stdin
        use std::io::Read;
        let mut buffer = String::new();
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer.trim().to_string()
    };

    if text.is_empty() {
        return Err("No text provided. Pass text as argument or via stdin.".into());
    }

    // Initialize pipeline
    let pipeline = CanopyPipeline::new()?;

    // Run analysis based on options
    if cli.trace {
        run_traced_analysis(&pipeline, &text, cli.document, cli.format)?;
    } else {
        run_standard_analysis(&pipeline, &text, cli.document, cli.format)?;
    }

    Ok(())
}

fn run_traced_analysis(
    pipeline: &CanopyPipeline,
    text: &str,
    document_mode: bool,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    if document_mode {
        let (doc, trace) = pipeline.analyze_document_with_trace(text)?;
        match format {
            OutputFormat::Text => {
                println!("{}", trace.to_text());
                println!("\n--- Document Summary ---");
                println!("Sentences: {}", doc.sentence_count());
                if let Some(drs) = &doc.drs {
                    println!("Referents: {}", drs.universe.len());
                    println!("Conditions: {}", drs.conditions.len());
                }
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&trace)?;
                println!("{json}");
            }
        }
    } else {
        let (analysis, trace) = pipeline.analyze_with_trace(text)?;
        match format {
            OutputFormat::Text => {
                println!("{}", trace.to_text());
                println!("\n--- Analysis Summary ---");
                println!("Tokens: {}", analysis.syntax.tokens.len());
                println!("Predicates: {}", analysis.decompositions.len());
                println!("Events: {}", analysis.event_count());
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&trace)?;
                println!("{json}");
            }
        }
    }
    Ok(())
}

fn run_standard_analysis(
    pipeline: &CanopyPipeline,
    text: &str,
    document_mode: bool,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    if document_mode {
        let doc = pipeline.analyze_document(text)?;
        match format {
            OutputFormat::Text => output_document_text(&doc),
            OutputFormat::Json => output_document_json(&doc)?,
        }
    } else {
        let analysis = pipeline.analyze(text)?;
        match format {
            OutputFormat::Text => output_analysis_text(&analysis, text),
            OutputFormat::Json => output_analysis_json(&analysis, text)?,
        }
    }
    Ok(())
}

fn output_document_text(doc: &canopy_resources::DocumentAnalysis) {
    println!("Document Analysis:");
    println!("  Sentences: {}", doc.sentence_count());
    for (i, sent) in doc.sentences.iter().enumerate() {
        println!("\n  [{}] \"{}\"", i + 1, sent.text);
        println!("      Tokens: {}", sent.syntax.tokens.len());
        println!("      Events: {}", sent.event_count());
    }
    if let Some(drs) = &doc.drs {
        println!("\n  DRS:");
        println!("    Referents: {}", drs.universe.len());
        println!("    Conditions: {}", drs.conditions.len());
    }
}

fn output_document_json(
    doc: &canopy_resources::DocumentAnalysis,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = DocumentOutput {
        sentence_count: doc.sentence_count(),
        sentences: doc
            .sentences
            .iter()
            .map(|s| SentenceOutput {
                text: s.text.clone(),
                token_count: s.syntax.tokens.len(),
                event_count: s.event_count(),
            })
            .collect(),
        referent_count: doc.drs.as_ref().map(|d| d.universe.len()),
        condition_count: doc.drs.as_ref().map(|d| d.conditions.len()),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn output_analysis_text(analysis: &canopy_resources::SemanticAnalysis, text: &str) {
    println!("Semantic Analysis:");
    println!("  Input: \"{text}\"");
    println!("  Tokens: {}", analysis.syntax.tokens.len());

    if !analysis.decompositions.is_empty() {
        println!("  Predicates:");
        for decomp in &analysis.decompositions {
            let roles: Vec<_> = decomp
                .expected_roles
                .iter()
                .map(|r| format!("{r:?}"))
                .collect();
            println!(
                "    {} → {:?} [{}]",
                decomp.sense_id,
                decomp.little_v_type,
                roles.join(", ")
            );
        }
    }

    if !analysis.role_bindings.is_empty() {
        println!("  Role Bindings:");
        for binding in &analysis.role_bindings {
            if let Some(token) = analysis.syntax.get_token(binding.token_id) {
                println!(
                    "    \"{}\" → {:?} ({:.0}%)",
                    token.form,
                    binding.role,
                    binding.confidence * 100.0
                );
            }
        }
    }

    if let Some(events) = &analysis.events {
        println!("  Events: {} composed", events.events.len());
    }
}

fn output_analysis_json(
    analysis: &canopy_resources::SemanticAnalysis,
    text: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = AnalysisOutput {
        input: text.to_string(),
        token_count: analysis.syntax.tokens.len(),
        predicate_count: analysis.decompositions.len(),
        event_count: analysis.event_count(),
        predicates: analysis
            .decompositions
            .iter()
            .map(|d| PredicateOutput {
                sense_id: d.sense_id.to_string(),
                little_v: format!("{:?}", d.little_v_type),
                confidence: d.confidence,
            })
            .collect(),
        role_bindings: analysis
            .role_bindings
            .iter()
            .filter_map(|b| {
                analysis
                    .syntax
                    .get_token(b.token_id)
                    .map(|token| RoleBindingOutput {
                        filler: token.form.clone(),
                        role: format!("{:?}", b.role),
                        confidence: b.confidence,
                    })
            })
            .collect(),
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

// JSON output structures
#[derive(serde::Serialize)]
struct AnalysisOutput {
    input: String,
    token_count: usize,
    predicate_count: usize,
    event_count: usize,
    predicates: Vec<PredicateOutput>,
    role_bindings: Vec<RoleBindingOutput>,
}

#[derive(serde::Serialize)]
struct PredicateOutput {
    sense_id: String,
    little_v: String,
    confidence: f32,
}

#[derive(serde::Serialize)]
struct RoleBindingOutput {
    filler: String,
    role: String,
    confidence: f32,
}

#[derive(serde::Serialize)]
struct DocumentOutput {
    sentence_count: usize,
    sentences: Vec<SentenceOutput>,
    referent_count: Option<usize>,
    condition_count: Option<usize>,
}

#[derive(serde::Serialize)]
struct SentenceOutput {
    text: String,
    token_count: usize,
    event_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_help() {
        // --help should return an error (clap's help exits with error code)
        // but should not panic
        let result = Cli::try_parse_from(["canopy", "--help"]);
        // Help returns Err because it triggers early exit, not a parse failure
        assert!(result.is_err(), "Help flag should cause early exit");
        let err = result.unwrap_err();
        // Verify it's a help display, not a real error
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_cli_parse_with_text() {
        let cli = Cli::try_parse_from(["canopy", "John runs."]).unwrap();
        assert_eq!(cli.text.as_deref(), Some("John runs."));
        assert!(!cli.trace);
        assert!(!cli.document);
    }

    #[test]
    fn test_cli_parse_with_trace() {
        let cli = Cli::try_parse_from(["canopy", "--trace", "John runs."]).unwrap();
        assert!(cli.trace);
    }

    #[test]
    fn test_cli_parse_short_flags() {
        let cli = Cli::try_parse_from(["canopy", "-t", "-d", "Hello."]).unwrap();
        assert!(cli.trace);
        assert!(cli.document);
    }

    #[test]
    fn test_cli_parse_format() {
        let cli = Cli::try_parse_from(["canopy", "--format", "json", "Test."]).unwrap();
        assert!(matches!(cli.format, OutputFormat::Json));
    }

    #[test]
    fn test_run_cli_test_error() {
        let result = run_cli_with_args(&[
            "canopy".to_string(),
            "--test-error".to_string(),
            "text".to_string(),
        ]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Test error condition");
    }

    #[test]
    fn test_output_format_default() {
        let format = OutputFormat::default();
        assert!(matches!(format, OutputFormat::Text));
    }

    #[test]
    #[ignore = "requires data files - run with --ignored"]
    fn test_run_cli_with_text() {
        // Test run_cli_with_args with explicit text argument
        let args = vec!["canopy".to_string(), "John runs.".to_string()];
        let result = run_cli_with_args(&args);
        assert!(result.is_ok(), "CLI should run successfully: {result:?}");
    }

    #[test]
    #[ignore = "requires data files - run with --ignored"]
    fn test_run_cli_multiple_times_with_args() {
        for i in 0..3 {
            let args = vec!["canopy".to_string(), format!("Sentence {}.", i)];
            let result = run_cli_with_args(&args);
            assert!(result.is_ok());
        }
    }
}

// Add test module for main.rs coverage
#[cfg(test)]
mod main_tests;
