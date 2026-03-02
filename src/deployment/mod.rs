pub mod forgejo;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use crate::core::shared::state::AppState;

// Re-export types from forgejo module
pub use forgejo::{AppType, BuildConfig, ForgejoClient, ForgejoError, ForgejoRepo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentTarget {
    /// Serve from GB platform (/apps/{name})
    Internal {
        route: String,
        shared_resources: bool,
    },
    /// Deploy to external Forgejo repository
    External {
        repo_url: String,
        custom_domain: Option<String>,
        ci_cd_enabled: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub app_name: String,
    pub target: DeploymentTarget,
    pub environment: DeploymentEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentEnvironment {
    Development,
    Staging,
    Production,
}

pub struct DeploymentRouter {
    forgejo_url: String,
    forgejo_token: Option<String>,
    internal_base_path: PathBuf,
}

impl DeploymentRouter {
    pub fn new(
        forgejo_url: String,
        forgejo_token: Option<String>,
        internal_base_path: PathBuf,
    ) -> Self {
        Self {
            forgejo_url,
            forgejo_token,
            internal_base_path,
        }
    }

    /// Route deployment based on target type
    pub async fn deploy(
        &self,
        config: DeploymentConfig,
        generated_app: GeneratedApp,
    ) -> Result<DeploymentResult, DeploymentError> {
        match config.target {
            DeploymentTarget::Internal { route, .. } => {
                self.deploy_internal(route, generated_app).await
            }
            DeploymentTarget::External { ref repo_url, .. } => {
                self.deploy_external(repo_url, generated_app).await
            }
        }
    }

    /// Deploy internally to GB platform
    async fn deploy_internal(
        &self,
        route: String,
        app: GeneratedApp,
    ) -> Result<DeploymentResult, DeploymentError> {
        // 1. Store files in Drive
        // 2. Register route in app router
        // 3. Create API endpoints
        // 4. Return deployment URL

        let url = format!("/apps/{}/", route);

        Ok(DeploymentResult {
            url,
            deployment_type: "internal".to_string(),
            status: DeploymentStatus::Deployed,
            metadata: serde_json::json!({
                "route": route,
                "platform": "gb",
            }),
        })
    }

    /// Deploy externally to Forgejo
    async fn deploy_external(
        &self,
        repo_url: &str,
        app: GeneratedApp,
    ) -> Result<DeploymentResult, DeploymentError> {
        // 1. Initialize git repo
        // 2. Add Forgejo remote
        // 3. Push generated files
        // 4. Create CI/CD workflow
        // 5. Trigger build

        Ok(DeploymentResult {
            url: repo_url.to_string(),
            deployment_type: "external".to_string(),
            status: DeploymentStatus::Pending,
            metadata: serde_json::json!({
                "repo_url": repo_url,
                "forgejo": self.forgejo_url,
            }),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub url: String,
    pub deployment_type: String,
    pub status: DeploymentStatus,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Pending,
    Building,
    Deployed,
    Failed,
}

#[derive(Debug)]
pub enum DeploymentError {
    InternalDeploymentError(String),
    ForgejoError(String),
    GitError(String),
    CiCdError(String),
}

impl std::fmt::Display for DeploymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentError::InternalDeploymentError(msg) => {
                write!(f, "Internal deployment error: {}", msg)
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
    pub fn temp_dir(&self) -> Result<PathBuf, DeploymentError> {
        let temp_dir = std::env::temp_dir()
            .join("gb-deployments")
            .join(&self.name);
        Ok(temp_dir)
    }

    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            files: Vec::new(),
        }
    }

    pub fn add_file(&mut self, path: String, content: Vec<u8>) {
        self.files.push(GeneratedFile { path, content });
    }

    pub fn add_text_file(&mut self, path: String, content: String) {
        self.add_file(path, content.into_bytes());
    }
}

// =============================================================================
// API Types and Handlers
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct DeploymentRequest {
    pub app_name: String,
    pub target: String,
    pub environment: String,
    pub manifest: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub success: bool,
    pub url: Option<String>,
    pub deployment_type: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentTargetsResponse {
    pub targets: Vec<DeploymentTargetInfo>,
}

#[derive(Debug, Serialize)]
pub struct DeploymentTargetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub features: Vec<String>,
}

/// Configure deployment routes
pub fn configure_deployment_routes() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/api/deployment/targets", axum::routing::get(get_deployment_targets))
        .route("/api/deployment/deploy", axum::routing::post(deploy_app))
}

/// Get available deployment targets
pub async fn get_deployment_targets(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<DeploymentTargetsResponse>, DeploymentApiError> {
    let targets = vec![
        DeploymentTargetInfo {
            id: "internal".to_string(),
            name: "GB Platform".to_string(),
            description: "Deploy internally to General Bots platform".to_string(),
            features: vec![
                "Instant deployment".to_string(),
                "Shared resources".to_string(),
                "Auto-scaling".to_string(),
                "Built-in monitoring".to_string(),
                "Zero configuration".to_string(),
            ],
        },
        DeploymentTargetInfo {
            id: "external".to_string(),
            name: "Forgejo ALM".to_string(),
            description: "Deploy to external Git repository with CI/CD".to_string(),
            features: vec![
                "Git-based deployment".to_string(),
                "Custom domains".to_string(),
                "CI/CD pipelines".to_string(),
                "Version control".to_string(),
                "Team collaboration".to_string(),
            ],
        },
    ];

    Ok(Json(DeploymentTargetsResponse { targets }))
}

/// Deploy an application
pub async fn deploy_app(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DeploymentRequest>,
) -> Result<Json<DeploymentResponse>, DeploymentApiError> {
    log::info!(
        "Deployment request received: app={}, target={}, env={}",
        request.app_name,
        request.target,
        request.environment
    );

    // Parse deployment target
    let target = match request.target.as_str() {
        "internal" => {
            let route = request.manifest
                .get("route")
                .and_then(|v| v.as_str())
                .unwrap_or(&request.app_name)
                .to_string();

            let shared_resources = request.manifest
                .get("shared_resources")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            DeploymentTarget::Internal {
                route,
                shared_resources,
            }
        }
        "external" => {
            let repo_url = request.manifest
                .get("repo_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DeploymentApiError::ValidationError("repo_url is required for external deployment".to_string()))?
                .to_string();

            let custom_domain = request.manifest
                .get("custom_domain")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let ci_cd_enabled = request.manifest
                .get("ci_cd_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            DeploymentTarget::External {
                repo_url,
                custom_domain,
                ci_cd_enabled,
            }
        }
        _ => {
            return Err(DeploymentApiError::ValidationError(format!(
                "Unknown deployment target: {}",
                request.target
            )));
        }
    };

    // Parse environment
    let environment = match request.environment.as_str() {
        "development" => DeploymentEnvironment::Development,
        "staging" => DeploymentEnvironment::Staging,
        "production" => DeploymentEnvironment::Production,
        _ => DeploymentEnvironment::Development,
    };

    // Create deployment configuration
    let config = DeploymentConfig {
        app_name: request.app_name.clone(),
        target,
        environment,
    };

    // Get Forgejo configuration from environment
    let forgejo_url = std::env::var("FORGEJO_URL")
        .unwrap_or_else(|_| "https://alm.pragmatismo.com.br".to_string());

    let forgejo_token = std::env::var("FORGEJO_TOKEN").ok();

    // Create deployment router
    let internal_base_path = std::path::PathBuf::from("/opt/gbo/data/apps");
    let router = DeploymentRouter::new(forgejo_url, forgejo_token, internal_base_path);

    // Create a placeholder generated app
    // In real implementation, this would come from the orchestrator
    let generated_app = GeneratedApp::new(
        config.app_name.clone(),
        "Generated application".to_string(),
    );

    // Execute deployment
    let result = router.deploy(config, generated_app).await
        .map_err(|e| DeploymentApiError::DeploymentFailed(e.to_string()))?;

    log::info!(
        "Deployment successful: url={}, type={}, status={:?}",
        result.url,
        result.deployment_type,
        result.status
    );

    Ok(Json(DeploymentResponse {
        success: true,
        url: Some(result.url),
        deployment_type: Some(result.deployment_type),
        status: Some(format!("{:?}", result.status)),
        error: None,
    }))
}

#[derive(Debug)]
pub enum DeploymentApiError {
    ValidationError(String),
    DeploymentFailed(String),
    InternalError(String),
}

impl IntoResponse for DeploymentApiError {
    fn into_response(self) -> Response {
        use crate::security::error_sanitizer::log_and_sanitize;

        let (status, message) = match self {
            DeploymentApiError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            DeploymentApiError::DeploymentFailed(msg) => {
                let sanitized = log_and_sanitize(&msg, "deployment", None);
                (StatusCode::INTERNAL_SERVER_ERROR, sanitized)
            }
            DeploymentApiError::InternalError(msg) => {
                let sanitized = log_and_sanitize(&msg, "deployment", None);
                (StatusCode::INTERNAL_SERVER_ERROR, sanitized)
            }
        };

        let body = Json(serde_json::json!({
            "success": false,
            "error": message,
        }));

        (status, body).into_response()
    }
}
