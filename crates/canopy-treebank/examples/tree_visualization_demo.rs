//! Tree Visualization Demo
//!
//! Shows dependency trees from the UD English-EWT treebank

use canopy_treebank::{ConlluParser, DependencyTree, ParsedSentence};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 UD Treebank Dependency Tree Visualization Demo");
    println!("================================================");

    // Use actual treebank data
    let dev_path = Path::new("data/ud_english-ewt/UD_English-EWT/en_ewt-ud-dev.conllu");
    if !dev_path.exists() {
        println!("❌ Treebank file not found: {:?}", dev_path);
        println!("   Please ensure the UD English-EWT data is available.");
        return Ok(());
    }

    let parser = ConlluParser::new(false);
    let sentences = parser.parse_file(dev_path)?;

    // Show first 8 examples from the treebank
    let examples_to_show = 8.min(sentences.len());

    for (i, sentence) in sentences.iter().take(examples_to_show).enumerate() {
        println!(
            "\n📝 Example {}: {}",
            i + 1,
            if sentence.text.is_empty() {
                "(no text)"
            } else {
                &sentence.text
            }
        );
        println!("{}", "─".repeat(60));

        println!("🌲 Dependency Tree:");
        render_full_tree(sentence);

        // Show both flat and hierarchical patterns
        if let Some(flat_key) = sentence.create_pattern_key() {
            println!("\n🔑 Pattern Key (flat): {}", flat_key);
        }

        if let Some(hierarchical_key) = sentence.create_hierarchical_pattern_key() {
            println!("🔗 Pattern Key (hierarchical): {}", hierarchical_key);
        }
    }

    println!("\n✨ Tree visualization complete!");
    println!("   These are real dependency trees from the UD English-EWT treebank.");
    println!("   Pattern keys are extracted from actual linguistic annotations.");

    Ok(())
}

/// Render hierarchical dependency tree from ParsedSentence
fn render_full_tree(sentence: &ParsedSentence) {
    if let Some(tree) = sentence.build_dependency_tree() {
        print_dependency_tree(&tree, 0, true);

        println!("\n📊 Tree Statistics:");
        println!("   • Depth: {}", tree.depth());
        println!("   • Total nodes: {}", tree.node_count());
    } else {
        println!("  (No dependency tree could be built)");
    }
}

/// Print dependency tree recursively with proper ASCII art
fn print_dependency_tree(tree: &DependencyTree, depth: usize, is_last: bool) {
    // Print indentation and connector
    for i in 0..depth {
        if i == depth - 1 {
            print!("{}", if is_last { "└─ " } else { "├─ " });
        } else {
            print!("│  ");
        }
    }

    // Print token information with relation if not root
    if depth == 0 {
        println!("{} ({:?})", tree.token.lemma, tree.token.upos);
    } else {
        println!(
            "{:?}: {} ({:?})",
            tree.token.deprel, tree.token.lemma, tree.token.upos
        );
    }

    // Print children recursively
    for (i, child) in tree.children.iter().enumerate() {
        let is_last_child = i == tree.children.len() - 1;
        print_dependency_tree(child, depth + 1, is_last_child);
    }
}

// Removed old flat pattern extraction functions - now using hierarchical methods from ConlluSentence
