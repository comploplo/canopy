//! Model management and discovery

use crate::traits::ModelType;
use std::path::Path;

/// Model manager for discovering and loading language models
pub struct ModelManager;

impl ModelManager {
    /// List all available models
    pub fn list_available() -> Vec<ModelInfo> {
        let mut models = Vec::new();

        // Check standard model locations
        let model_paths = [
            "/Users/gabe/projects/canopy/models",
            "./models",
            "~/.canopy/models",
        ];

        for path in &model_paths {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Some(model_info) = Self::detect_model(&entry.path()) {
                        models.push(model_info);
                    }
                }
            }
        }

        models
    }

    /// Check if a specific model type is available
    pub fn is_available(model_type: ModelType) -> bool {
        Self::list_available()
            .iter()
            .any(|m| m.model_type == model_type)
    }

    /// Check if a model is available by name
    pub fn is_available_by_name(model_name: &str) -> bool {
        Self::list_available().iter().any(|m| m.name == model_name)
    }

    /// Detect model type from file
    fn detect_model(path: &Path) -> Option<ModelInfo> {
        let filename = path.file_name()?.to_str()?;

        if filename.contains("ud-1.2") && filename.ends_with(".udpipe") {
            Some(ModelInfo {
                name: "UDPipe 1.2 English".to_string(),
                path: path.to_path_buf(),
                model_type: ModelType::UDPipe12,
                language: "en".to_string(),
                version: "1.2".to_string(),
                size_mb: path.metadata().ok()?.len() / 1024 / 1024,
            })
        } else if filename.contains("ud-2.") && filename.ends_with(".udpipe") {
            Some(ModelInfo {
                name: "UDPipe 2.15 English".to_string(),
                path: path.to_path_buf(),
                model_type: ModelType::UDPipe215,
                language: "en".to_string(),
                version: "2.15".to_string(),
                size_mb: path.metadata().ok()?.len() / 1024 / 1024,
            })
        } else {
            None
        }
    }

    /// Get the best available model for a language
    pub fn get_best_model(language: &str) -> Option<ModelInfo> {
        let mut models: Vec<_> = Self::list_available()
            .into_iter()
            .filter(|m| m.language == language)
            .collect();

        // Prefer newer models
        models.sort_by(|a, b| match (&a.model_type, &b.model_type) {
            (ModelType::UDPipe215, ModelType::UDPipe12) => std::cmp::Ordering::Less,
            (ModelType::UDPipe12, ModelType::UDPipe215) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        });

        models.into_iter().next()
    }
}

/// Information about an available model
#[derive(Debug, Clone, PartialEq)]
pub struct ModelInfo {
    pub name: String,
    pub path: std::path::PathBuf,
    pub model_type: ModelType,
    pub language: String,
    pub version: String,
    pub size_mb: u64,
}

/// Supported model types
#[derive(Debug, Clone, PartialEq)]
pub enum SupportedModel {
    UDPipe12English,
    UDPipe215English,
    Custom { name: String, path: String },
}
