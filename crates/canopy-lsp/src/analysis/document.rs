//! Document parsing and sentence extraction
//!
//! Extracts sentences from different document types.

use crate::analysis::PositionMapper;
use crate::state::SentenceSpan;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Language type for document analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageType {
    /// Plain text files.
    PlainText,
    /// Markdown documents.
    Markdown,
    /// Source code (extract comments only).
    SourceCode,
}

impl LanguageType {
    /// Determine language type from LSP language ID.
    #[must_use]
    pub fn from_language_id(id: &str) -> Self {
        match id {
            "plaintext" | "text" => Self::PlainText,
            "markdown" | "md" => Self::Markdown,
            "rust" | "python" | "javascript" | "typescript" | "go" | "java" | "c" | "cpp"
            | "csharp" => Self::SourceCode,
            _ => Self::PlainText,
        }
    }
}

/// Extract sentences from document content based on language type.
#[must_use]
pub fn extract_sentences(content: &str, language_id: &str) -> Vec<SentenceSpan> {
    let lang_type = LanguageType::from_language_id(language_id);

    match lang_type {
        LanguageType::PlainText => extract_plaintext_sentences(content),
        LanguageType::Markdown => extract_markdown_sentences(content),
        LanguageType::SourceCode => extract_comment_sentences(content, language_id),
    }
}

/// Extract sentences from plain text.
///
/// Uses simple sentence boundary detection (., !, ?).
fn extract_plaintext_sentences(content: &str) -> Vec<SentenceSpan> {
    let mapper = PositionMapper::new(content);
    let mut sentences = Vec::new();
    let mut current_start = 0;
    let mut in_sentence = false;

    for (i, c) in content.char_indices() {
        if !in_sentence && !c.is_whitespace() {
            current_start = i;
            in_sentence = true;
        }

        // Sentence-ending punctuation followed by space or end of content
        if in_sentence && (c == '.' || c == '!' || c == '?') {
            let next_idx = i + c.len_utf8();
            let is_end = next_idx >= content.len()
                || content[next_idx..]
                    .chars()
                    .next()
                    .is_some_and(|nc| nc.is_whitespace());

            if is_end {
                let text = content[current_start..next_idx].trim().to_string();
                if !text.is_empty() {
                    let line_start = mapper.byte_to_line(current_start).unwrap_or(0);
                    let line_end = mapper.byte_to_line(next_idx.saturating_sub(1)).unwrap_or(0);

                    sentences.push(SentenceSpan {
                        text,
                        byte_start: current_start,
                        byte_end: next_idx,
                        line_start,
                        line_end,
                    });
                }
                in_sentence = false;
            }
        }
    }

    // Handle trailing text without sentence-ending punctuation
    if in_sentence && current_start < content.len() {
        let text = content[current_start..].trim().to_string();
        if !text.is_empty() {
            let line_start = mapper.byte_to_line(current_start).unwrap_or(0);
            let line_end = mapper
                .byte_to_line(content.len().saturating_sub(1))
                .unwrap_or(0);

            sentences.push(SentenceSpan {
                text,
                byte_start: current_start,
                byte_end: content.len(),
                line_start,
                line_end,
            });
        }
    }

    sentences
}

/// Extract sentences from Markdown using pulldown-cmark parser.
///
/// Only extracts text from paragraph elements, skipping:
/// - Code blocks
/// - Headings
/// - Lists (bullet points often aren't prose)
/// - Tables
/// - Front matter (via YAML extension)
fn extract_markdown_sentences(content: &str) -> Vec<SentenceSpan> {
    let mapper = PositionMapper::new(content);
    let mut sentences = Vec::new();

    // Enable common extensions
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;

    let parser = Parser::new_ext(content, options);

    let mut in_paragraph = false;
    let mut current_text = String::new();
    let mut para_start_offset: Option<usize> = None;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Paragraph) => {
                in_paragraph = true;
                current_text.clear();
                para_start_offset = Some(range.start);
            }
            Event::End(TagEnd::Paragraph) => {
                if in_paragraph && !current_text.trim().is_empty() {
                    let text = current_text.trim().to_string();
                    let byte_start = para_start_offset.unwrap_or(range.start);
                    let byte_end = range.end;

                    let line_start = mapper.byte_to_line(byte_start).unwrap_or(0);
                    let line_end = mapper.byte_to_line(byte_end.saturating_sub(1)).unwrap_or(0);

                    sentences.push(SentenceSpan {
                        text,
                        byte_start,
                        byte_end,
                        line_start,
                        line_end,
                    });
                }
                in_paragraph = false;
                current_text.clear();
                para_start_offset = None;
            }
            Event::Text(text) if in_paragraph => {
                current_text.push_str(&text);
            }
            Event::SoftBreak if in_paragraph => {
                current_text.push(' ');
            }
            Event::HardBreak if in_paragraph => {
                current_text.push(' ');
            }
            _ => {}
        }
    }

    sentences
}

/// Extract sentences from source code comments.
fn extract_comment_sentences(content: &str, _language_id: &str) -> Vec<SentenceSpan> {
    let mapper = PositionMapper::new(content);
    let mut sentences = Vec::new();

    // Simple pattern: extract /// and // comments
    let lines: Vec<&str> = content.lines().collect();
    let mut line_byte_offset = 0;

    for line in &lines {
        let trimmed = line.trim();

        // Doc comments (///)
        if let Some(comment_content) = trimmed.strip_prefix("///") {
            let comment_text = comment_content.trim();
            if !comment_text.is_empty() {
                let comment_start = line_byte_offset + line.find("///").unwrap_or(0) + 3;
                let _comment_end = line_byte_offset + line.len();

                // Find sentences in the comment
                let inner_sentences =
                    extract_sentences_from_line(comment_text, comment_start, &mapper);
                sentences.extend(inner_sentences);
            }
        }
        // Regular comments (//)
        else if let Some(comment_content) = trimmed.strip_prefix("//") {
            let comment_text = comment_content.trim();
            if !comment_text.is_empty()
                && !comment_text.starts_with('!')
                && !comment_text.starts_with('#')
            {
                let comment_start = line_byte_offset + line.find("//").unwrap_or(0) + 2;

                let inner_sentences =
                    extract_sentences_from_line(comment_text, comment_start, &mapper);
                sentences.extend(inner_sentences);
            }
        }

        line_byte_offset += line.len() + 1;
    }

    sentences
}

/// Extract sentences from a single line of text.
fn extract_sentences_from_line(
    line: &str,
    base_offset: usize,
    mapper: &PositionMapper,
) -> Vec<SentenceSpan> {
    // For simplicity, treat each non-empty line as a potential sentence
    // A more sophisticated approach would use proper sentence boundary detection
    let text = line.trim().to_string();
    if text.is_empty() {
        return Vec::new();
    }

    let line_start = mapper.byte_to_line(base_offset).unwrap_or(0);

    vec![SentenceSpan {
        text,
        byte_start: base_offset,
        byte_end: base_offset + line.len(),
        line_start,
        line_end: line_start,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_plaintext_simple() {
        let content = "Hello world. This is a test.";
        let sentences = extract_plaintext_sentences(content);

        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "Hello world.");
        assert_eq!(sentences[1].text, "This is a test.");
    }

    #[test]
    fn test_extract_plaintext_multiline() {
        let content = "First sentence.\nSecond sentence.";
        let sentences = extract_plaintext_sentences(content);

        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].line_start, 0);
        assert_eq!(sentences[1].line_start, 1);
    }

    #[test]
    fn test_extract_markdown_skip_code() {
        let content = "Normal text.\n```\ncode block\n```\nMore text.";
        let sentences = extract_markdown_sentences(content);

        assert_eq!(sentences.len(), 2);
        assert!(sentences.iter().all(|s| !s.text.contains("code block")));
    }

    #[test]
    fn test_extract_markdown_skip_frontmatter() {
        let content = "---\ntitle: Test\n---\nActual content.";
        let sentences = extract_markdown_sentences(content);

        assert_eq!(sentences.len(), 1);
        assert_eq!(sentences[0].text, "Actual content.");
    }

    #[test]
    fn test_extract_comments() {
        let content = "/// This is a doc comment.\nfn main() {\n    // Regular comment.\n}";
        let sentences = extract_comment_sentences(content, "rust");

        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].text.contains("doc comment"));
        assert!(sentences[1].text.contains("Regular comment"));
    }

    #[test]
    fn test_language_type_detection() {
        assert_eq!(
            LanguageType::from_language_id("plaintext"),
            LanguageType::PlainText
        );
        assert_eq!(
            LanguageType::from_language_id("markdown"),
            LanguageType::Markdown
        );
        assert_eq!(
            LanguageType::from_language_id("rust"),
            LanguageType::SourceCode
        );
        assert_eq!(
            LanguageType::from_language_id("unknown"),
            LanguageType::PlainText
        );
    }

    #[test]
    fn test_extract_markdown_skip_headings() {
        // Markdown headings are properly parsed and skipped
        let content = "# Heading\n\nThe cat runs.\n\n## Another heading\n\nThe dog walks.";
        let sentences = extract_markdown_sentences(content);

        // Should only get paragraphs, not headings
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].text.contains("cat"));
        assert!(sentences[1].text.contains("dog"));
    }

    #[test]
    fn test_extract_markdown_paragraphs_only() {
        // Only paragraph content is extracted
        let content = "# Title\n\nParagraph one.\n\n- List item\n- Another item\n\nParagraph two.";
        let sentences = extract_markdown_sentences(content);

        // Should only get paragraphs, not headings or list items
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "Paragraph one.");
        assert_eq!(sentences[1].text, "Paragraph two.");
    }
}
