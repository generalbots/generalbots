pub mod forgejo;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
