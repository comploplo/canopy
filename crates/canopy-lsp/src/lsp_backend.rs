//! Tower-LSP backend implementation for Canopy (STUB)
//!
//! TODO: Implement proper LSP server with tower-lsp
//! For now, this is a placeholder for future LSP integration.

/// Stub LSP backend - TODO: Implement with tower-lsp
pub struct CanopyLspStub;

impl CanopyLspStub {
    pub fn new() -> Self {
        Self
    }
    
    /// TODO: Implement actual LSP server
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        println!("LSP Server stub - not yet implemented");
        println!("TODO: Integrate tower-lsp for full LSP functionality");
        Ok(())
    }
}

impl Default for CanopyLspStub {
    fn default() -> Self {
        Self::new()
    }
}