//! Document state management
//!
//! Tracks open documents, their versions, and cached analysis results.

use canopy_resources::DocumentAnalysis;
use dashmap::DashMap;
use std::time::Instant;
use tower_lsp::lsp_types::Url;

/// Represents a span of text identified as a sentence.
#[derive(Debug, Clone)]
pub struct SentenceSpan {
    /// The sentence text.
    pub text: String,
    /// Byte offset of sentence start in document.
    pub byte_start: usize,
    /// Byte offset of sentence end in document.
    pub byte_end: usize,
    /// Line number where sentence starts (0-indexed).
    pub line_start: u32,
    /// Line number where sentence ends (0-indexed).
    pub line_end: u32,
}

/// Cached document with analysis results.
#[derive(Debug)]
pub struct CachedDocument {
    /// LSP document version.
    pub version: i32,
    /// Raw text content.
    pub content: String,
    /// Parsed sentences with byte offsets.
    pub sentences: Vec<SentenceSpan>,
    /// Cached semantic analysis (lazily computed).
    pub analysis: Option<DocumentAnalysis>,
    /// Timestamp of last analysis.
    pub analyzed_at: Option<Instant>,
    /// Language ID from LSP.
    pub language_id: String,
}

impl CachedDocument {
    /// Create a new cached document.
    pub fn new(content: String, version: i32, language_id: String) -> Self {
        Self {
            version,
            content,
            sentences: Vec::new(),
            analysis: None,
            analyzed_at: None,
            language_id,
        }
    }

    /// Check if analysis is stale (older than threshold).
    #[must_use]
    pub fn is_analysis_stale(&self, max_age_ms: u64) -> bool {
        match self.analyzed_at {
            Some(at) => at.elapsed().as_millis() > u128::from(max_age_ms),
            None => true,
        }
    }

    /// Update the content and invalidate analysis.
    pub fn update_content(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
        self.sentences.clear();
        self.analysis = None;
        self.analyzed_at = None;
    }
}

/// Thread-safe document state manager.
#[derive(Debug, Default)]
pub struct DocumentState {
    /// Map: URI -> `CachedDocument`
    documents: DashMap<Url, CachedDocument>,
}

impl DocumentState {
    /// Create a new document state manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a new document.
    pub fn open(&self, uri: Url, content: String, version: i32, language_id: String) {
        let doc = CachedDocument::new(content, version, language_id);
        self.documents.insert(uri, doc);
    }

    /// Close a document.
    pub fn close(&self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// Get a reference to a document.
    pub fn get(&self, uri: &Url) -> Option<dashmap::mapref::one::Ref<'_, Url, CachedDocument>> {
        self.documents.get(uri)
    }

    /// Get a mutable reference to a document.
    pub fn get_mut(
        &self,
        uri: &Url,
    ) -> Option<dashmap::mapref::one::RefMut<'_, Url, CachedDocument>> {
        self.documents.get_mut(uri)
    }

    /// Check if a document is open.
    #[must_use]
    pub fn contains(&self, uri: &Url) -> bool {
        self.documents.contains_key(uri)
    }

    /// Get the number of open documents.
    #[must_use]
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if there are no open documents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_state_open_close() {
        let state = DocumentState::new();
        let uri = Url::parse("file:///test.txt").unwrap();

        state.open(
            uri.clone(),
            "Hello world.".to_string(),
            1,
            "plaintext".to_string(),
        );
        assert!(state.contains(&uri));
        assert_eq!(state.len(), 1);

        state.close(&uri);
        assert!(!state.contains(&uri));
        assert!(state.is_empty());
    }

    #[test]
    fn test_cached_document_update() {
        let mut doc = CachedDocument::new("Initial".to_string(), 1, "plaintext".to_string());
        assert!(doc.is_analysis_stale(1000));

        doc.analyzed_at = Some(Instant::now());
        assert!(!doc.is_analysis_stale(1000));

        doc.update_content("Updated".to_string(), 2);
        assert_eq!(doc.version, 2);
        assert!(doc.analysis.is_none());
        assert!(doc.is_analysis_stale(1000));
    }

    #[test]
    fn test_document_state_get() {
        let state = DocumentState::new();
        let uri = Url::parse("file:///test.txt").unwrap();

        assert!(state.get(&uri).is_none());

        state.open(
            uri.clone(),
            "Hello world.".to_string(),
            1,
            "plaintext".to_string(),
        );

        let doc = state.get(&uri).unwrap();
        assert_eq!(doc.content, "Hello world.");
        assert_eq!(doc.version, 1);
        assert_eq!(doc.language_id, "plaintext");
    }

    #[test]
    fn test_document_state_get_mut() {
        let state = DocumentState::new();
        let uri = Url::parse("file:///test.txt").unwrap();

        state.open(
            uri.clone(),
            "Hello world.".to_string(),
            1,
            "plaintext".to_string(),
        );

        {
            let mut doc = state.get_mut(&uri).unwrap();
            doc.update_content("Updated content.".to_string(), 2);
        }

        let doc = state.get(&uri).unwrap();
        assert_eq!(doc.content, "Updated content.");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_document_state_multiple_documents() {
        let state = DocumentState::new();
        let uri1 = Url::parse("file:///test1.txt").unwrap();
        let uri2 = Url::parse("file:///test2.txt").unwrap();

        state.open(uri1.clone(), "Content 1".to_string(), 1, "txt".to_string());
        state.open(uri2.clone(), "Content 2".to_string(), 1, "txt".to_string());

        assert_eq!(state.len(), 2);
        assert!(state.contains(&uri1));
        assert!(state.contains(&uri2));

        state.close(&uri1);
        assert_eq!(state.len(), 1);
        assert!(!state.contains(&uri1));
        assert!(state.contains(&uri2));
    }

    #[test]
    fn test_sentence_span() {
        let span = SentenceSpan {
            text: "Test sentence.".to_string(),
            byte_start: 0,
            byte_end: 14,
            line_start: 0,
            line_end: 0,
        };

        assert_eq!(span.text, "Test sentence.");
        assert_eq!(span.byte_start, 0);
        assert_eq!(span.byte_end, 14);
    }

    #[test]
    fn test_cached_document_sentences() {
        let mut doc = CachedDocument::new("Hello. World.".to_string(), 1, "txt".to_string());
        assert!(doc.sentences.is_empty());

        doc.sentences.push(SentenceSpan {
            text: "Hello.".to_string(),
            byte_start: 0,
            byte_end: 6,
            line_start: 0,
            line_end: 0,
        });

        assert_eq!(doc.sentences.len(), 1);

        // Update clears sentences
        doc.update_content("New content.".to_string(), 2);
        assert!(doc.sentences.is_empty());
    }
}
