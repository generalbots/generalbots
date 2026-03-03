use git2::{Repository, Signature};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{DeploymentError, GeneratedApp, GeneratedFile};
use super::types::{AppType, DeploymentEnvironment};

pub struct ForgejoClient {
    base_url: String,
    token: String,
    client: Client,
}

impl ForgejoClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            base_url,
            token,
            client: Client::new(),
        }
    }

    /// Create a new repository in Forgejo
    pub async fn create_repository(
        &self,
        name: &str,
        description: &str,
        private: bool,
    ) -> Result<ForgejoRepo, ForgejoError> {
        let url = format!("{}/api/v1/user/repos", self.base_url);

        let payload = CreateRepoRequest {
            name: name.to_string(),
            description: description.to_string(),
            private,
            auto_init: true,
            gitignores: Some("Node,React,Vite".to_string()),
            license: Some("MIT".to_string()),
            readme: Some("Default".to_string()),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("token {}", self.token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ForgejoError::HttpError(e.to_string()))?;

        if response.status().is_success() {
            let repo: ForgejoRepo = response
                .json()
                .await
                .map_err(|e| ForgejoError::JsonError(e.to_string()))?;
            Ok(repo)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(ForgejoError::ApiError(format!("{}: {}", status, body)))
        }
    }

    /// Push generated app to Forgejo repository
    pub async fn push_app(
        &self,
        repo_url: &str,
        app: &GeneratedApp,
        branch: &str,
    ) -> Result<String, DeploymentError> {
        // 1. Create temporary directory for the app
        let temp_dir = app.temp_dir()?;
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| DeploymentError::GitError(format!("Failed to create temp dir: {}", e)))?;

        // 2. Write all files to temp directory
        for file in &app.files {
            let file_path = temp_dir.join(&file.path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DeploymentError::GitError(format!("Failed to create parent dir: {}", e)))?;
            }
            std::fs::write(&file_path, &file.content)
                .map_err(|e| DeploymentError::GitError(format!("Failed to write file: {}", e)))?;
        }

        // 3. Initialize local git repo
        let repo = Repository::init(&temp_dir)
            .map_err(|e| DeploymentError::GitError(format!("Failed to init repo: {}", e)))?;

        // 4. Add all files
        let mut index = repo.index()
            .map_err(|e| DeploymentError::GitError(format!("Failed to get index: {}", e)))?;

        // Add all files recursively
        self.add_all_files(&repo, &mut index, &temp_dir)
            .map_err(|e| DeploymentError::GitError(format!("Failed to add files: {}", e)))?;

        index.write()
            .map_err(|e| DeploymentError::GitError(format!("Failed to write index: {}", e)))?;

        // 5. Create commit
        let tree_id = index.write_tree()
            .map_err(|e| DeploymentError::GitError(format!("Failed to write tree: {}", e)))?;
        let tree = repo.find_tree(tree_id)
            .map_err(|e| DeploymentError::GitError(format!("Failed to find tree: {}", e)))?;

        let sig = Signature::now("GB Deployer", "deployer@generalbots.com")
            .map_err(|e| DeploymentError::GitError(format!("Failed to create signature: {}", e)))?;

        let oid = repo.commit(
            Some(&format!("refs/heads/{}", branch)),
            &sig,
            &sig,
            &format!("Initial commit: {}", app.description),
            &tree,
            &[],
        ).map_err(|e| DeploymentError::GitError(format!("Failed to commit: {}", e)))?;

        // 6. Add Forgejo remote with token authentication
        let auth_url = self.add_token_to_url(repo_url);
        let mut remote = repo.remote("origin", &auth_url)
            .map_err(|e| DeploymentError::GitError(format!("Failed to add remote: {}", e)))?;

        // 7. Push to Forgejo
        remote.push(&[format!("refs/heads/{}", branch)], None)
            .map_err(|e| DeploymentError::GitError(format!("Failed to push: {}", e)))?;

        Ok(oid.to_string())
    }

    /// Create CI/CD workflow for the app based on Phase 2.5 app types
    pub async fn create_cicd_workflow(
        &self,
        repo_url: &str,
        app_type: &AppType,
        environment: &DeploymentEnvironment,
    ) -> Result<(), DeploymentError> {
        let workflow = match app_type {
            AppType::GbNative { .. } => self.generate_gb_native_workflow(environment),
            AppType::Custom { framework, node_version, build_command, output_directory } => {
                self.generate_custom_workflow(framework, node_version.as_deref().unwrap_or("20"),
                    build_command.as_deref().unwrap_or("npm run build"),
                    output_directory.as_deref().unwrap_or("dist"), environment)
            }
        };

        // Create workflow file
        let workflow_file = GeneratedFile {
            path: ".forgejo/workflows/deploy.yml".to_string(),
            content: workflow.into_bytes(),
        };

        // Create a new commit with the workflow file
        let workflow_app = GeneratedApp {
            name: "workflow".to_string(),
            description: "CI/CD workflow".to_string(),
            files: vec![workflow_file],
        };

        self.push_app(repo_url, &workflow_app, "main").await?;

        Ok(())
    }

    fn add_all_files(
        &self,
        repo: &Repository,
        index: &mut git2::Index,
        dir: &Path,
    ) -> Result<(), git2::Error> {
        for entry in std::fs::read_dir(dir).map_err(|e| git2::Error::from_str(&e.to_string()))? {
            let entry = entry.map_err(|e| git2::Error::from_str(&e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                if path.file_name().map(|f| f == ".git").unwrap_or(false) {
                    continue;
                }
                self.add_all_files(repo, index, &path)?;
            } else {
                let relative_path = path.strip_prefix(repo.workdir().unwrap())
                    .map_err(|e| git2::Error::from_str(&e.to_string()))?;
                index.add_path(relative_path)?;
            }
        }
        Ok(())
    }

    fn add_token_to_url(&self, url: &str) -> String {
        // Convert https://forgejo.com/user/repo to https://token@forgejo.com/user/repo
        if url.starts_with("https://") {
            url.replace("https://", &format!("https://{}@", self.token))
        } else {
            url.to_string()
        }
    }






}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgejoRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub html_url: String,
}

#[derive(Debug, Serialize)]
struct CreateRepoRequest {
    name: String,
    description: String,
    private: bool,
    auto_init: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    gitignores: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    readme: Option<String>,
}

// AppType and related types are now defined in types.rs

#[derive(Debug)]
pub enum ForgejoError {
    HttpError(String),
    JsonError(String),
    ApiError(String),
    GitError(String),
}

impl std::fmt::Display for ForgejoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForgejoError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            ForgejoError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            ForgejoError::ApiError(msg) => write!(f, "API error: {}", msg),
            ForgejoError::GitError(msg) => write!(f, "Git error: {}", msg),
        }
    }
}

// =============================================================================
// CI/CD Workflow Generation for Phase 2.5
// =============================================================================

impl ForgejoClient {
    /// Generate CI/CD workflow for GB Native apps
    fn generate_gb_native_workflow(&self, environment: &DeploymentEnvironment) -> String {
        let env_name = environment.to_string();
        format!(r#"name: Deploy GB Native App

on:
  push:
    branches: [ main, {env_name} ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Build app
        run: npm run build
        env:
          NODE_ENV: production
          GB_ENV: {env_name}

      - name: Deploy to GB Platform
        run: |
          echo "Deploying to GB Platform ({env_name})"
          # GB Platform deployment logic here
        env:
          GB_DEPLOYMENT_TOKEN: ${{{{ secrets.GB_DEPLOYMENT_TOKEN }}}}
"#)
    }

    /// Generate CI/CD workflow for Custom apps
    fn generate_custom_workflow(&self, framework: &str, node_version: &str,
        build_command: &str, output_dir: &str, environment: &DeploymentEnvironment) -> String {
        let env_name = environment.to_string();
        format!(r#"name: Deploy Custom {framework} App

on:
  push:
    branches: [ main, {env_name} ]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '{node_version}'

      - name: Install dependencies
        run: npm ci

      - name: Build {framework} app
        run: {build_command}
        env:
          NODE_ENV: production

      - name: Upload build artifacts
        uses: actions/upload-artifact@v3
        with:
          name: build-output
          path: {output_dir}

      - name: Deploy to custom hosting
        run: |
          echo "Deploying {framework} app to {env_name}"
          # Custom deployment logic here
"#)
    }
}

impl std::error::Error for ForgejoError {}
