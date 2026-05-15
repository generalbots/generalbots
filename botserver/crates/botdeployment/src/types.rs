//! Type definitions for deployment module
//!
//! Three project types under the ALM (Forgejo) model:
//! - **bots**: GB Native scripts in .gbdialog/ — no ALM, runs in chat mode
//! - **apps**: Custom full-stack apps — ALM repo → alm-ci builds → own Incus container
//! - **sites**: Static HTML/CSS/JS — ALM repo → alm-ci builds → Caddy serves from proxy container

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use super::forgejo::ForgejoError;
use super::{log_and_sanitize, StringError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectType {
    Bot,
    App {
        framework: String,
        node_version: Option<String>,
        build_command: Option<String>,
        output_directory: Option<String>,
    },
    Site,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Bot
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Bot => write!(f, "bot"),
            ProjectType::App { framework, .. } => write!(f, "app-{}", framework),
            ProjectType::Site => write!(f, "site"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeployTarget {
    None,
    IncusContainer,
    CaddyStatic,
}

impl From<&ProjectType> for DeployTarget {
    fn from(pt: &ProjectType) -> Self {
        match pt {
            ProjectType::Bot => DeployTarget::None,
            ProjectType::App { .. } => DeployTarget::IncusContainer,
            ProjectType::Site => DeployTarget::CaddyStatic,
        }
    }
}

impl std::fmt::Display for DeployTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeployTarget::None => write!(f, "none"),
            DeployTarget::IncusContainer => write!(f, "incus-container"),
            DeployTarget::CaddyStatic => write!(f, "caddy-static"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployKey {
    pub key_id: String,
    pub key_hash: String,
    pub org: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployGatewayRequest {
    pub app_name: String,
    pub org: String,
    pub project_type: ProjectType,
    pub artifact_url: String,
    pub environment: DeploymentEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployGatewayResponse {
    pub success: bool,
    pub url: Option<String>,
    pub container: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub organization: String,
    pub app_name: String,
    pub project_type: ProjectType,
    pub deploy_target: DeployTarget,
    pub environment: DeploymentEnvironment,
    pub custom_domain: Option<String>,
    pub ci_cd_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    pub project_type: String,
    pub deploy_target: String,
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

#[derive(Debug, Deserialize)]
pub struct DeploymentRequest {
    pub organization: Option<String>,
    pub app_name: String,
    pub project_type: String,
    pub framework: Option<String>,
    pub environment: String,
    pub custom_domain: Option<String>,
    pub ci_cd_enabled: Option<bool>,
    pub node_version: Option<String>,
    pub build_command: Option<String>,
    pub output_directory: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub success: bool,
    pub url: Option<String>,
    pub repository: Option<String>,
    pub project_type: Option<String>,
    pub deploy_target: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectTypesResponse {
    pub project_types: Vec<ProjectTypeInfo>,
}

#[derive(Debug, Serialize)]
pub struct ProjectTypeInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub deploy_target: String,
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
        let (status, message) = match self {
            DeploymentApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            DeploymentApiError::DeploymentFailed(msg) => {
                let error = StringError(msg);
                let sanitized = log_and_sanitize(&error, "deployment");
                (StatusCode::INTERNAL_SERVER_ERROR, sanitized.message)
            }
            DeploymentApiError::ConfigurationError(msg) => {
                let error = StringError(msg);
                let sanitized = log_and_sanitize(&error, "deployment_config");
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
