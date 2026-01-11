//! Semantic tree visualization.
//!
//! This module provides pretty-printing of semantic analysis results as trees.
//!
//! # Example Output
//!
//! ```text
//! Sentence: "John gave Mary a book."
//! ├── Syntax
//! │   ├── John [Noun] ─ nsubj
//! │   ├── gave [Verb] ─ root
//! │   ├── Mary [Propn] ─ iobj
//! │   ├── a [Det] ─ det
//! │   └── book [Noun] ─ obj
//! ├── Event: gave (CAUSE)
//! │   ├── Aspect: Accomplishment
//! │   ├── Voice: Active
//! │   └── Participants
//! │       ├── Agent: "John" (95%)
//! │       ├── Theme: "a book" (92%)
//! │       └── Recipient: "Mary" (90%)
//! └── Decomposition: give-13.1
//!     ├── Type: CAUSE
//!     ├── Confidence: 92%
//!     └── Roles: Agent, Theme, Recipient
//! ```

use crate::pipeline::{DocumentAnalysis, SemanticAnalysis};
use canopy::kernel::discourse::Drs;
use ptree::{print_tree, TreeItem};
use std::borrow::Cow;
use std::fmt::Write as FmtWrite;
use std::io::{self, Write};

// =============================================================================
// Tree Node Types
// =============================================================================

/// A node in the semantic tree.
#[derive(Debug, Clone)]
pub struct SemanticNode {
    /// Display text for this node.
    pub text: String,
    /// Child nodes.
    pub children: Vec<SemanticNode>,
    /// Optional style hint (for colored output).
    pub style: NodeStyle,
}

/// Style hints for tree nodes.
#[derive(Debug, Clone, Copy, Default)]
pub enum NodeStyle {
    /// Default style.
    #[default]
    Default,
    /// Header/title style.
    Header,
    /// Event/predicate style.
    Event,
    /// Participant/argument style.
    Participant,
    /// Syntax token style.
    Syntax,
    /// Metadata style.
    Meta,
    /// DRS/logical form style.
    Drs,
}

impl SemanticNode {
    /// Create a new node with text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            children: Vec::new(),
            style: NodeStyle::Default,
        }
    }

    /// Create a header node.
    #[must_use]
    pub fn header(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            children: Vec::new(),
            style: NodeStyle::Header,
        }
    }

    /// Add a child node.
    #[must_use]
    pub fn with_child(mut self, child: SemanticNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children.
    #[must_use]
    pub fn with_children(mut self, children: Vec<SemanticNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set the style.
    #[must_use]
    pub fn with_style(mut self, style: NodeStyle) -> Self {
        self.style = style;
        self
    }

    /// Add a child node (mutating).
    pub fn add_child(&mut self, child: SemanticNode) {
        self.children.push(child);
    }
}

impl TreeItem for SemanticNode {
    type Child = Self;

    fn write_self<W: Write>(&self, f: &mut W, _style: &ptree::Style) -> io::Result<()> {
        write!(f, "{}", self.text)
    }

    fn children(&self) -> Cow<'_, [Self::Child]> {
        Cow::Borrowed(&self.children)
    }
}

// =============================================================================
// Tree Building from Analysis
// =============================================================================

/// Build a semantic tree from a sentence analysis.
#[must_use]
pub fn build_sentence_tree(analysis: &SemanticAnalysis) -> SemanticNode {
    let mut root = SemanticNode::header(format!("Sentence: \"{}\"", analysis.text));

    // Add syntax subtree
    root.add_child(build_syntax_tree(analysis));

    // Add events subtree if present
    if let Some(ref events) = analysis.events {
        for (i, event) in events.events.iter().enumerate() {
            let mut event_node = SemanticNode::new(format!(
                "Event {}: {} ({:?})",
                i + 1,
                event.predicate,
                event.little_v_type
            ))
            .with_style(NodeStyle::Event);

            // Aspect and voice
            event_node.add_child(
                SemanticNode::new(format!("Aspect: {:?}", event.aspect))
                    .with_style(NodeStyle::Meta),
            );
            event_node.add_child(
                SemanticNode::new(format!("Voice: {:?}", event.voice)).with_style(NodeStyle::Meta),
            );

            // Participants
            if !event.participants.is_empty() {
                let mut participants_node = SemanticNode::new("Participants");
                for (role, participant) in &event.participants {
                    participants_node.add_child(
                        SemanticNode::new(format!("{:?}: \"{}\"", role, participant.text))
                            .with_style(NodeStyle::Participant),
                    );
                }
                event_node.add_child(participants_node);
            }

            root.add_child(event_node);
        }
    }

    // Add decompositions
    if !analysis.decompositions.is_empty() {
        let mut decomp_node = SemanticNode::new("Predicate Decompositions");
        for decomp in &analysis.decompositions {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let confidence_pct = (decomp.confidence * 100.0).clamp(0.0, 100.0).round() as u32;
            let mut d_node = SemanticNode::new(format!("{} ({confidence_pct}%)", decomp.sense_id));

            d_node.add_child(
                SemanticNode::new(format!("Type: {:?}", decomp.little_v_type))
                    .with_style(NodeStyle::Meta),
            );

            if !decomp.expected_roles.is_empty() {
                let roles: Vec<_> = decomp
                    .expected_roles
                    .iter()
                    .map(|r| format!("{r:?}"))
                    .collect();
                d_node.add_child(
                    SemanticNode::new(format!("Roles: {}", roles.join(", ")))
                        .with_style(NodeStyle::Meta),
                );
            }

            decomp_node.add_child(d_node);
        }
        root.add_child(decomp_node);
    }

    // Add role bindings summary
    if !analysis.role_bindings.is_empty() {
        let mut bindings_node = SemanticNode::new("Role Bindings");
        for binding in &analysis.role_bindings {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let confidence_pct = (binding.confidence * 100.0).clamp(0.0, 100.0).round() as u32;
            let token_text = analysis
                .syntax
                .tokens
                .iter()
                .find(|t| t.id == binding.token_id)
                .map_or("?", |t| t.form.as_str());
            bindings_node.add_child(
                SemanticNode::new(format!(
                    "{:?} → \"{}\" ({confidence_pct}%)",
                    binding.role, token_text
                ))
                .with_style(NodeStyle::Participant),
            );
        }
        root.add_child(bindings_node);
    }

    // Add DRS if present
    if let Some(ref drs) = analysis.sentence_drs {
        root.add_child(build_drs_tree(drs));
    }

    root
}

/// Build the syntax subtree.
fn build_syntax_tree(analysis: &SemanticAnalysis) -> SemanticNode {
    let mut syntax_node = SemanticNode::new("Syntax");

    for token in &analysis.syntax.tokens {
        let token_node = SemanticNode::new(format!(
            "{} [{:?}] ─ {:?}",
            token.form, token.upos, token.deprel
        ))
        .with_style(NodeStyle::Syntax);
        syntax_node.add_child(token_node);
    }

    syntax_node
}

/// Build a DRS subtree.
fn build_drs_tree(drs: &Drs) -> SemanticNode {
    let mut drs_node = SemanticNode::new("DRS (Logical Form)").with_style(NodeStyle::Drs);

    // Universe (referents)
    if !drs.universe.is_empty() {
        let refs: Vec<_> = drs.universe.values().map(|r| r.id.to_string()).collect();
        drs_node.add_child(
            SemanticNode::new(format!("Universe: {}", refs.join(", "))).with_style(NodeStyle::Drs),
        );
    }

    // Conditions
    if !drs.conditions.is_empty() {
        let mut conds_node = SemanticNode::new("Conditions").with_style(NodeStyle::Drs);
        for cond in drs.conditions.iter().take(10) {
            conds_node.add_child(SemanticNode::new(cond.to_string()).with_style(NodeStyle::Drs));
        }
        if drs.conditions.len() > 10 {
            conds_node.add_child(SemanticNode::new(format!(
                "... ({} more)",
                drs.conditions.len() - 10
            )));
        }
        drs_node.add_child(conds_node);
    }

    drs_node
}

/// Build a tree from a document analysis.
#[must_use]
pub fn build_document_tree(analysis: &DocumentAnalysis) -> SemanticNode {
    let mut root =
        SemanticNode::header(format!("Document ({} sentences)", analysis.sentences.len()));

    // Add each sentence
    for (i, sentence) in analysis.sentences.iter().enumerate() {
        let mut sent_node = SemanticNode::new(format!(
            "Sentence {}: \"{}\"",
            i + 1,
            truncate(&sentence.text, 50)
        ));

        // Coherence relation
        if let Some(ref coherence) = sentence.coherence {
            sent_node.add_child(
                SemanticNode::new(format!("Coherence: {:?}", coherence.relation))
                    .with_style(NodeStyle::Meta),
            );
        }

        // Discourse move
        if let Some(ref move_class) = sentence.discourse_move {
            sent_node.add_child(
                SemanticNode::new(format!("Move: {:?}", move_class.move_type))
                    .with_style(NodeStyle::Meta),
            );
        }

        // Events summary
        if let Some(ref events) = sentence.events {
            for event in &events.events {
                let participants: Vec<_> = event
                    .participants
                    .iter()
                    .map(|(r, p)| format!("{r:?}={}", p.text))
                    .collect();
                let parts_str = if participants.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", participants.join(", "))
                };
                sent_node.add_child(
                    SemanticNode::new(format!(
                        "Event: {}({:?}){}",
                        event.predicate, event.little_v_type, parts_str
                    ))
                    .with_style(NodeStyle::Event),
                );
            }
        }

        root.add_child(sent_node);
    }

    // Add unified DRS
    if let Some(ref drs) = analysis.drs {
        root.add_child(build_drs_tree(drs));
    }

    root
}

// =============================================================================
// Printing Functions
// =============================================================================

/// Print a semantic tree to stdout.
///
/// # Errors
/// Returns an error if writing to stdout fails.
pub fn print_semantic_tree(tree: &SemanticNode) -> io::Result<()> {
    print_tree(tree)
}

/// Write a semantic tree to a string.
#[must_use]
pub fn tree_to_string(tree: &SemanticNode) -> String {
    let mut buf = Vec::new();
    let _ = ptree::write_tree(tree, &mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

/// Print a sentence analysis as a tree.
///
/// # Errors
/// Returns an error if writing fails.
pub fn print_sentence_tree(analysis: &SemanticAnalysis) -> io::Result<()> {
    let tree = build_sentence_tree(analysis);
    print_semantic_tree(&tree)
}

/// Print a document analysis as a tree.
///
/// # Errors
/// Returns an error if writing fails.
pub fn print_document_tree(analysis: &DocumentAnalysis) -> io::Result<()> {
    let tree = build_document_tree(analysis);
    print_semantic_tree(&tree)
}

// =============================================================================
// Compact Tree Format
// =============================================================================

/// Build a compact event tree (just events and participants).
#[must_use]
pub fn build_compact_event_tree(analysis: &SemanticAnalysis) -> SemanticNode {
    let predicate = analysis
        .events
        .as_ref()
        .and_then(|e| e.events.first())
        .map_or("(no event)", |e| e.predicate.as_str());

    let mut root = SemanticNode::new(format!("{predicate}(...)"));

    if let Some(ref events) = analysis.events {
        for event in &events.events {
            let mut event_node =
                SemanticNode::new(format!("{} [{:?}]", event.predicate, event.little_v_type));

            for (role, participant) in &event.participants {
                event_node.add_child(SemanticNode::new(format!(
                    "{role:?}: \"{}\"",
                    participant.text
                )));
            }

            root = event_node;
        }
    }

    root
}

/// Build a dependency tree from syntax.
#[must_use]
pub fn build_dependency_tree(analysis: &SemanticAnalysis) -> SemanticNode {
    // Find the root token
    let root_token = analysis
        .syntax
        .tokens
        .iter()
        .find(|t| matches!(t.deprel, canopy::DepRel::Root))
        .or_else(|| analysis.syntax.tokens.first());

    match root_token {
        Some(root) => build_dep_subtree(analysis, root.id),
        None => SemanticNode::new("(empty)"),
    }
}

/// Recursively build dependency subtree.
fn build_dep_subtree(
    analysis: &SemanticAnalysis,
    token_id: canopy::runtime::TokenId,
) -> SemanticNode {
    let token = analysis
        .syntax
        .tokens
        .iter()
        .find(|t| t.id == token_id)
        .expect("token should exist");

    let mut node = SemanticNode::new(format!("{} [{:?}]", token.form, token.upos));

    // Find children (tokens whose head is this token)
    let children: Vec<_> = analysis
        .syntax
        .tokens
        .iter()
        .filter(|t| t.head == Some(token_id) && t.id != token_id)
        .collect();

    for child in children {
        let child_node = build_dep_subtree(analysis, child.id);
        let labeled_node = SemanticNode::new(format!("─{:?}─ {}", child.deprel, child_node.text))
            .with_children(child_node.children);
        node.add_child(labeled_node);
    }

    node
}

// =============================================================================
// Box Notation (DRS-style)
// =============================================================================

/// Format DRS in classic box notation.
#[must_use]
pub fn format_drs_box(drs: &Drs) -> String {
    let mut out = String::new();

    // Universe (referents) line
    let refs: Vec<_> = drs.universe.values().map(|r| r.id.to_string()).collect();
    let refs_str = refs.join(", ");

    // Conditions
    let conds: Vec<_> = drs.conditions.iter().map(ToString::to_string).collect();

    // Calculate width
    let max_width = refs_str
        .len()
        .max(conds.iter().map(String::len).max().unwrap_or(0))
        .max(10);
    let box_width = max_width + 4;

    // Top border
    let _ = writeln!(out, "┌{}┐", "─".repeat(box_width));

    // Universe
    let _ = writeln!(out, "│ {:<width$} │", refs_str, width = max_width + 2);

    // Separator
    let _ = writeln!(out, "├{}┤", "─".repeat(box_width));

    // Conditions
    for cond in &conds {
        let _ = writeln!(out, "│ {:<width$} │", cond, width = max_width + 2);
    }

    // Bottom border
    let _ = writeln!(out, "└{}┘", "─".repeat(box_width));

    out
}

// =============================================================================
// Utilities
// =============================================================================

/// Truncate a string with ellipsis.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CanopyPipeline;

    fn data_available() -> bool {
        crate::paths::data_path("data/verbnet").exists()
    }

    #[test]
    fn test_build_sentence_tree() {
        if !data_available() {
            eprintln!("Skipping: data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let analysis = pipeline.analyze("John gave Mary a book.").unwrap();

        let tree = build_sentence_tree(&analysis);

        // Should have root with children
        assert!(tree.text.contains("John gave Mary a book"));
        assert!(!tree.children.is_empty());

        // Should have syntax subtree
        assert!(tree.children.iter().any(|c| c.text == "Syntax"));
    }

    #[test]
    fn test_build_document_tree() {
        if !data_available() {
            eprintln!("Skipping: data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let analysis = pipeline.analyze_document("John ran. Mary walked.").unwrap();

        let tree = build_document_tree(&analysis);

        assert!(tree.text.contains("2 sentences"));
        assert_eq!(tree.children.len(), 2 + usize::from(analysis.drs.is_some()));
    }

    #[test]
    fn test_tree_to_string() {
        if !data_available() {
            eprintln!("Skipping: data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let analysis = pipeline.analyze("The cat runs.").unwrap();

        let tree = build_sentence_tree(&analysis);
        let output = tree_to_string(&tree);

        assert!(output.contains("The cat runs"));
        assert!(output.contains("Syntax"));
    }

    #[test]
    fn test_build_dependency_tree() {
        if !data_available() {
            eprintln!("Skipping: data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let analysis = pipeline.analyze("The big cat runs quickly.").unwrap();

        let tree = build_dependency_tree(&analysis);

        // Root should be the verb
        assert!(tree.text.contains("runs") || tree.text.contains("cat"));
    }

    #[test]
    fn test_compact_event_tree() {
        if !data_available() {
            eprintln!("Skipping: data not available");
            return;
        }

        let pipeline = CanopyPipeline::new().unwrap();
        let analysis = pipeline.analyze("John ran.").unwrap();

        let tree = build_compact_event_tree(&analysis);

        // Should show the event
        let output = tree_to_string(&tree);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_format_drs_box() {
        // Create a simple DRS for testing
        let drs = Drs::default();
        let output = format_drs_box(&drs);

        // Should have box characters
        assert!(output.contains('┌'));
        assert!(output.contains('└'));
        assert!(output.contains('│'));
    }

    #[test]
    fn test_semantic_node_builder() {
        let node = SemanticNode::new("Root")
            .with_style(NodeStyle::Header)
            .with_child(SemanticNode::new("Child 1"))
            .with_child(SemanticNode::new("Child 2"));

        assert_eq!(node.text, "Root");
        assert_eq!(node.children.len(), 2);
        assert!(matches!(node.style, NodeStyle::Header));
    }
}
