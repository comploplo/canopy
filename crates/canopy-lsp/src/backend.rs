//! LSP Backend Implementation
//!
//! Implements the `LanguageServer` trait from tower-lsp, connecting
//! LSP requests to Canopy's semantic analysis.

use std::sync::Arc;

use async_trait::async_trait;
use canopy::CanopyError;
use canopy_resources::{CanopyPipeline, SemanticAnalysis};
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::analysis::{extract_sentences, AnalysisCache};
use crate::handlers::{code_actions, diagnostics, hover, inlay_hints, semantic_tokens, symbols};
use crate::state::DocumentState;
use crate::tokens::semantic_token_legend;

/// Canopy LSP backend server.
pub struct CanopyBackend {
    /// LSP client for sending notifications.
    client: Client,
    /// Shared Canopy pipeline (initialized lazily, ~730ms).
    pipeline: Arc<RwLock<Option<CanopyPipeline>>>,
    /// Document state manager.
    documents: DocumentState,
    /// Per-sentence analysis cache (LRU, 1000 entries).
    analysis_cache: AnalysisCache,
}

impl CanopyBackend {
    /// Create a new backend instance.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            pipeline: Arc::new(RwLock::new(None)),
            documents: DocumentState::new(),
            analysis_cache: AnalysisCache::default(),
        }
    }

    /// Initialize the Canopy pipeline (expensive operation).
    async fn ensure_pipeline(&self) -> std::result::Result<(), String> {
        let mut pipeline = self.pipeline.write().await;
        if pipeline.is_none() {
            tracing::info!("Initializing Canopy pipeline...");
            match CanopyPipeline::new() {
                Ok(p) => {
                    tracing::info!("Canopy pipeline initialized successfully");
                    *pipeline = Some(p);
                }
                Err(e) => {
                    let msg = format!("Failed to initialize Canopy pipeline: {e}");
                    tracing::error!("{}", msg);
                    return Err(msg);
                }
            }
        }
        Ok(())
    }

    /// Get the pipeline, initializing if needed.
    pub async fn pipeline(
        &self,
    ) -> std::result::Result<tokio::sync::RwLockReadGuard<'_, Option<CanopyPipeline>>, String> {
        self.ensure_pipeline().await?;
        Ok(self.pipeline.read().await)
    }

    /// Get the document state.
    #[must_use]
    pub fn documents(&self) -> &DocumentState {
        &self.documents
    }

    /// Get the LSP client.
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the analysis cache.
    #[must_use]
    pub fn cache(&self) -> &AnalysisCache {
        &self.analysis_cache
    }

    /// Analyze a sentence using the cache.
    ///
    /// Uses LRU cache to avoid redundant analysis of identical sentences.
    pub async fn analyze_sentence(
        &self,
        sentence: &str,
    ) -> std::result::Result<SemanticAnalysis, CanopyError> {
        let pipeline_guard = self
            .pipeline()
            .await
            .map_err(|e| CanopyError::not_initialized(format!("Pipeline: {e}")))?;

        let pipeline = pipeline_guard
            .as_ref()
            .ok_or_else(|| CanopyError::not_initialized("Pipeline"))?;

        self.analysis_cache.get_or_analyze(sentence, pipeline)
    }

    /// Analyze a document and publish diagnostics.
    async fn analyze_and_publish_diagnostics(&self, uri: Url) {
        if let Some(mut doc) = self.documents.get_mut(&uri) {
            // Extract sentences if not already done
            if doc.sentences.is_empty() {
                let sentences = extract_sentences(&doc.content, &doc.language_id);
                doc.sentences = sentences;
            }
        }

        // Generate and publish diagnostics
        let diags = diagnostics::generate_diagnostics(self, &uri).await;
        self.client.publish_diagnostics(uri, diags, None).await;
    }
}

#[async_trait]
impl LanguageServer for CanopyBackend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        // Start pipeline initialization in background
        let pipeline = self.pipeline.clone();
        tokio::spawn(async move {
            let mut p = pipeline.write().await;
            if p.is_none() {
                tracing::info!("Background: Initializing Canopy pipeline...");
                if let Ok(new_pipeline) = CanopyPipeline::new() {
                    *p = Some(new_pipeline);
                    tracing::info!("Background: Canopy pipeline ready");
                }
            }
        });

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Full document sync for simplicity; incremental would be more efficient
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: semantic_token_legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: Some(false),
                            ..Default::default()
                        },
                    ),
                ),
                document_symbol_provider: Some(OneOf::Left(true)),
                // We use push diagnostics via publish_diagnostics, not pull
                inlay_hint_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![
                            CodeActionKind::QUICKFIX,
                            CodeActionKind::new("quickfix.showAlternatives"),
                            CodeActionKind::new("quickfix.showSenses"),
                            CodeActionKind::new("quickfix.explain"),
                            CodeActionKind::new("quickfix.showDetails"),
                            CodeActionKind::new("source.analyze"),
                        ]),
                        resolve_provider: Some(false),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "canopy-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("Canopy LSP server initialized");
        self.client
            .log_message(MessageType::INFO, "Canopy LSP server ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Canopy LSP server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;
        let language_id = params.text_document.language_id;

        tracing::debug!("Document opened: {}", uri);

        self.documents
            .open(uri.clone(), content, version, language_id);
        self.analyze_and_publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // With FULL sync, we get the complete new content
        if let Some(change) = params.content_changes.into_iter().next() {
            if let Some(mut doc) = self.documents.get_mut(&uri) {
                doc.update_content(change.text, version);
            }
        }

        tracing::debug!("Document changed: {}", uri);
        self.analyze_and_publish_diagnostics(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        tracing::debug!("Document closed: {}", uri);
        self.documents.close(&uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        hover::handle_hover(self, params).await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        semantic_tokens::handle_semantic_tokens_full(self, params).await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        symbols::handle_document_symbol(self, params).await
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        inlay_hints::handle_inlay_hints(self, params).await
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        code_actions::handle_code_actions(self, &params)
    }
}
