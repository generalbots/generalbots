use git2::{Repository, Signature};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::types::{DeploymentEnvironment, DeploymentError, DeployTarget, GeneratedApp, GeneratedFile, ProjectType};

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

    pub async fn create_repository(
        &self,
        org: &str,
        name: &str,
        description: &str,
        private: bool,
    ) -> Result<ForgejoRepo, ForgejoError> {
        let url = format!("{}/api/v1/org/{org}/repos", self.base_url);

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

    pub async fn push_app(
        &self,
        repo_url: &str,
        app: &GeneratedApp,
        branch: &str,
    ) -> Result<String, DeploymentError> {
        let temp_dir = app.temp_dir()?;
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| DeploymentError::GitError(format!("Failed to create temp dir: {}", e)))?;

        for file in &app.files {
            let file_path = temp_dir.join(&file.path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DeploymentError::GitError(format!("Failed to create parent dir: {}", e)))?;
            }
            std::fs::write(&file_path, &file.content)
                .map_err(|e| DeploymentError::GitError(format!("Failed to write file: {}", e)))?;
        }

        let repo = Repository::init(&temp_dir)
            .map_err(|e| DeploymentError::GitError(format!("Failed to init repo: {}", e)))?;

        let mut index = repo.index()
            .map_err(|e| DeploymentError::GitError(format!("Failed to get index: {}", e)))?;

        self.add_all_files(&repo, &mut index, &temp_dir)
            .map_err(|e| DeploymentError::GitError(format!("Failed to add files: {}", e)))?;

        index.write()
            .map_err(|e| DeploymentError::GitError(format!("Failed to write index: {}", e)))?;

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

        let auth_url = self.add_token_to_url(repo_url);
        let mut remote = repo.remote("origin", &auth_url)
            .map_err(|e| DeploymentError::GitError(format!("Failed to add remote: {}", e)))?;

        remote.push(&[format!("refs/heads/{}", branch)], None)
            .map_err(|e| DeploymentError::GitError(format!("Failed to push: {}", e)))?;

        Ok(oid.to_string())
    }

    pub async fn create_cicd_workflow(
        &self,
        repo_url: &str,
        project_type: &ProjectType,
        deploy_target: &DeployTarget,
        environment: &DeploymentEnvironment,
    ) -> Result<(), DeploymentError> {
        let workflow = match deploy_target {
            DeployTarget::None => return Ok(()),
            DeployTarget::IncusContainer => self.generate_app_workflow(project_type, deploy_target, environment),
            DeployTarget::CaddyStatic => self.generate_site_workflow(environment),
        };

        let workflow_file = GeneratedFile {
            path: ".forgejo/workflows/deploy.yml".to_string(),
            content: workflow.into_bytes(),
        };

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
        if url.starts_with("https://") {
            url.replace("https://", &format!("https://{}@", self.token))
        } else {
            url.to_string()
        }
    }

    fn generate_app_workflow(&self, project_type: &ProjectType, _deploy_target: &DeployTarget, environment: &DeploymentEnvironment) -> String {
        let env_name = environment.to_string();
        let (framework, node_version, build_command, output_dir) = match project_type {
            ProjectType::App { framework, node_version, build_command, output_directory } => (
                framework.clone(),
                node_version.clone().unwrap_or_else(|| "20".to_string()),
                build_command.clone().unwrap_or_else(|| "npm run build".to_string()),
                output_directory.clone().unwrap_or_else(|| "dist".to_string()),
            ),
            _ => ("htmx".to_string(), "20".to_string(), "npm run build".to_string(), "dist".to_string()),
        };

        format!(r#"name: Deploy {framework} App

on:
  push:
    branches: [ main, {env_name} ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '{node_version}'

      - name: Install dependencies
        run: npm ci

      - name: Build {framework} app
        run: {build_command}
        env:
          NODE_ENV: production

      - name: Package artifacts
        run: tar -czf /tmp/artifact.tar.gz -C ./{output_dir} .

      - name: Deploy via Gateway
        run: |
          curl -X POST ${{{{ DEPLOY_GATEWAY_URL }}}}/deploy \
            -H "X-Deploy-Key: ${{{{ DEPLOY_KEY }}}}" \
            -H "Content-Type: application/json" \
            -d '{{
              "app_name": "${{{{ gitea.repository_name }}}}",
              "org": "${{{{ gitea.repository_owner }}}}",
              "project_type": "app",
              "artifact_url": "file:///tmp/artifact.tar.gz",
              "environment": "{env_name}"
            }}'
        env:
          DEPLOY_GATEWAY_URL: ${{{{ secrets.DEPLOY_GATEWAY_URL }}}}
          DEPLOY_KEY: ${{{{ secrets.DEPLOY_KEY }}}}
"#)
    }

    fn generate_site_workflow(&self, environment: &DeploymentEnvironment) -> String {
        let env_name = environment.to_string();

        format!(r#"name: Deploy Static Site

on:
  push:
    branches: [ main, {env_name} ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Build site
        run: npm run build
        env:
          NODE_ENV: production

      - name: Package artifacts
        run: tar -czf /tmp/artifact.tar.gz -C ./dist .

      - name: Deploy via Gateway
        run: |
          curl -X POST ${{{{ DEPLOY_GATEWAY_URL }}}}/deploy \
            -H "X-Deploy-Key: ${{{{ DEPLOY_KEY }}}}" \
            -H "Content-Type: application/json" \
            -d '{{
              "app_name": "${{{{ gitea.repository_name }}}}",
              "org": "${{{{ gitea.repository_owner }}}}",
              "project_type": "site",
              "artifact_url": "file:///tmp/artifact.tar.gz",
              "environment": "{env_name}"
            }}'
        env:
          DEPLOY_GATEWAY_URL: ${{{{ secrets.DEPLOY_GATEWAY_URL }}}}
          DEPLOY_KEY: ${{{{ secrets.DEPLOY_KEY }}}}
"#)
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

impl std::error::Error for ForgejoError {}
