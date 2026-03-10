//! Type definitions for VibeCode deployment module - Phase 2.5

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

// Re-export ForgejoError for From implementation
use super::forgejo::ForgejoError;

/// App type determines the deployment strategy and CI/CD workflow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppType {
    /// GB Native - Optimized for General Bots platform
    /// Uses GB-specific features, shared resources, and optimized runtime
    GbNative {
        /// Use GB shared database connection pool
        shared_database: bool,
        /// Use GB authentication system
        shared_auth: bool,
        /// Use GB caching layer
        shared_cache: bool,
    },
    /// Custom - Any framework or technology
    /// Fully independent deployment with custom CI/CD
    Custom {
        /// Framework type: htmx, react, vue, nextjs, svelte, etc.
        framework: String,
        /// Node.js version for build
        node_version: Option<String>,
        /// Build command
        build_command: Option<String>,
        /// Output directory
        output_directory: Option<String>,
    },
}

impl Default for AppType {
    fn default() -> Self {
        AppType::GbNative {
            shared_database: true,
            shared_auth: true,
            shared_cache: true,
        }
    }
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppType::GbNative { .. } => write!(f, "gb-native"),
            AppType::Custom { framework, .. } => write!(f, "custom-{}", framework),
        }
    }
}

/// Deployment configuration for all apps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// Organization name (becomes part of repo: org/app_name)
    pub organization: String,
    /// Application name (becomes part of repo: org/app_name)
    pub app_name: String,
    /// App type determines deployment strategy
    pub app_type: AppType,
    /// Deployment environment
    pub environment: DeploymentEnvironment,
    /// Custom domain (optional)
    pub custom_domain: Option<String>,
    /// Enable CI/CD pipeline
    pub ci_cd_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum DeploymentEnvironment {
    #[default]
    Development,
    Staging,
    Production,
}


impl std::fmt::Display for DeploymentEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentEnvironment::Development => write!(f, "development"),
            DeploymentEnvironment::Staging => write!(f, "staging"),
            DeploymentEnvironment::Production => write!(f, "production"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub url: String,
    pub repository: String,
    pub app_type: String,
    pub environment: String,
    pub status: DeploymentStatus,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeploymentStatus {
    Pending,
    Building,
    Deployed,
    Failed,
}

#[derive(Debug)]
pub enum DeploymentError {
    ConfigurationError(String),
    ForgejoError(String),
    GitError(String),
    CiCdError(String),
}

impl std::fmt::Display for DeploymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentError::ConfigurationError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
            DeploymentError::ForgejoError(msg) => write!(f, "Forgejo error: {}", msg),
            DeploymentError::GitError(msg) => write!(f, "Git error: {}", msg),
            DeploymentError::CiCdError(msg) => write!(f, "CI/CD error: {}", msg),
        }
    }
}

impl std::error::Error for DeploymentError {}

impl From<ForgejoError> for DeploymentError {
    fn from(err: ForgejoError) -> Self {
        DeploymentError::ForgejoError(err.to_string())
    }
}

/// Helper type for wrapping string errors to implement Error trait
#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

#[derive(Debug, Clone)]
pub struct GeneratedApp {
    pub name: String,
    pub description: String,
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub path: String,
    pub content: Vec<u8>,
}

impl GeneratedApp {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            files: Vec::new(),
        }
    }

    pub fn temp_dir(&self) -> Result<std::path::PathBuf, DeploymentError> {
        let temp_dir = std::env::temp_dir()
            .join("gb-deployments")
            .join(&self.name);
        Ok(temp_dir)
    }

    pub fn add_file(&mut self, path: String, content: Vec<u8>) {
        self.files.push(GeneratedFile { path, content });
    }

    pub fn add_text_file(&mut self, path: String, content: String) {
        self.add_file(path, content.into_bytes());
    }
}

// =============================================================================
// API Types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct DeploymentRequest {
    pub organization: Option<String>,
    pub app_name: String,
    pub app_type: String,
    pub framework: Option<String>,
    pub environment: String,
    pub custom_domain: Option<String>,
    pub ci_cd_enabled: Option<bool>,
    pub shared_database: Option<bool>,
    pub shared_auth: Option<bool>,
    pub shared_cache: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub success: bool,
    pub url: Option<String>,
    pub repository: Option<String>,
    pub app_type: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AppTypesResponse {
    pub app_types: Vec<AppTypeInfo>,
}

#[derive(Debug, Serialize)]
pub struct AppTypeInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub features: Vec<String>,
}

#[derive(Debug)]
pub enum DeploymentApiError {
    ValidationError(String),
    DeploymentFailed(String),
    ConfigurationError(String),
}

impl IntoResponse for DeploymentApiError {
    fn into_response(self) -> Response {
        use crate::security::error_sanitizer::log_and_sanitize;

        let (status, message) = match self {
            DeploymentApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            DeploymentApiError::DeploymentFailed(msg) => {
                let error = StringError(msg);
                let sanitized = log_and_sanitize(&error, "deployment", None);
                (StatusCode::INTERNAL_SERVER_ERROR, sanitized.message)
            }
            DeploymentApiError::ConfigurationError(msg) => {
                let error = StringError(msg);
                let sanitized = log_and_sanitize(&error, "deployment_config", None);
                (StatusCode::INTERNAL_SERVER_ERROR, sanitized.message)
            }
        };

        let body = Json(serde_json::json!({
            "success": false,
            "error": message,
        }));

        (status, body).into_response()
    }
}
