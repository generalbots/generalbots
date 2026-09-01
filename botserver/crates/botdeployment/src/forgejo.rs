use reqwest::Client;
use serde::{Deserialize, Serialize};

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
            // Only stock Forgejo gitignore templates are accepted; "React"
            // and "Vite" are not valid templates and make repo creation
            // fail with `GetRepoInitFile[React]: file does not exist`.
            gitignores: Some("Node".to_string()),
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
            // Idempotent re-deploys: the repo already exists from a previous
            // deploy. Fetch it instead of failing so the pipeline can push
            // the new commit (the re-deploy must succeed like git does).
            if status == reqwest::StatusCode::CONFLICT && body.contains("already e") {
                let existing = self
                    .client
                    .get(&format!("{}/api/v1/repos/{}/{}", self.base_url, org, name))
                    .header("Authorization", format!("token {}", self.token))
                    .send()
                    .await
                    .map_err(|e| ForgejoError::HttpError(e.to_string()))?;
                if existing.status().is_success() {
                    return existing
                        .json()
                        .await
                        .map_err(|e| ForgejoError::JsonError(e.to_string()));
                }
                return Err(ForgejoError::ApiError(format!(
                    "{}: {}",
                    existing.status(),
                    existing.text().await.unwrap_or_default()
                )));
            }
            // Vibe projects derive the ALM org from the branch id
            // (`org=branch`), so the org may not exist yet on a fresh
            // deployment. Auto-create it (admin token) and retry once.
            // Forgejo reports a missing org as 404 in some versions and as
            // 422 "org does not exist" in others — handle both.
            let org_missing = status == reqwest::StatusCode::NOT_FOUND
                || (status == reqwest::StatusCode::UNPROCESSABLE_ENTITY
                    && body.contains("does not exist"));
            if org_missing {
                if self.ensure_org(org).await.is_ok() {
                    let retry = self
                        .client
                        .post(&url)
                        .header("Authorization", format!("token {}", self.token))
                        .json(&payload)
                        .send()
                        .await
                        .map_err(|e| ForgejoError::HttpError(e.to_string()))?;
                    if retry.status().is_success() {
                        return retry
                            .json()
                            .await
                            .map_err(|e| ForgejoError::JsonError(e.to_string()));
                    }
                    let retry_status = retry.status();
                    let retry_body = retry.text().await.unwrap_or_default();
                    return Err(ForgejoError::ApiError(format!(
                        "{}: {}",
                        retry_status, retry_body
                    )));
                }
            }
            Err(ForgejoError::ApiError(format!("{}: {}", status, body)))
        }
    }

    /// Create the organization when missing so branch-derived orgs
    /// (org=branch) can own repos without a manual Forgejo setup step.
    async fn ensure_org(&self, org: &str) -> Result<(), ForgejoError> {
        let url = format!("{}/api/v1/orgs", self.base_url);
        let payload = serde_json::json!({
            "username": org,
            "full_name": format!("Workspace {org}"),
            "visibility": "private",
        });
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("token {}", self.token))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ForgejoError::HttpError(e.to_string()))?;
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(ForgejoError::ApiError(format!("{}: {}", status, body)))
        }
    }

    /// Delete a repository (used when a Vibe project is deleted so a
    /// recreated project with the same name starts from a clean repo
    /// instead of inheriting stale history that rejects the seed push).
    pub async fn delete_repository(&self, org: &str, name: &str) -> Result<(), ForgejoError> {
        let url = format!("{}/api/v1/repos/{}/{}", self.base_url, org, name);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("token {}", self.token))
            .send()
            .await
            .map_err(|e| ForgejoError::HttpError(e.to_string()))?;
        // 404 = already gone; treat as success (idempotent delete).
        if response.status().is_success() || response.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(())
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
        // The temp dir is a fixed path (`gb-deployments/{name}`) reused
        // across deployments; leaving a previous repo behind makes the next
        // root commit fail with "current tip is not the first parent".
        // Always start from a clean checkout.
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)
                .map_err(|e| DeploymentError::GitError(format!("Failed to clean temp dir: {}", e)))?;
        }
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

        let auth_url = self.add_token_to_url(repo_url);
        log::info!("git push url: {auth_url}");
        // Use git CLI entirely (init + add + commit + push) instead of
        // mixing git2 (which has ref-creation and HTTP auth issues) with
        // CLI. The proven flow: init → add → commit → push.
        let git = |args: &[&str]| -> Result<String, DeploymentError> {
            let output = std::process::Command::new("git")
                .current_dir(&temp_dir)
                .args(args)
                .output()
                .map_err(|e| DeploymentError::GitError(format!("git {}: {e}", args[0])))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DeploymentError::GitError(format!(
                    "git {} failed: {}",
                    args[0],
                    stderr.trim().lines().last().unwrap_or("unknown")
                )));
            }
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        };

        git(&["init", "-q"])?;
        git(&["add", "-A"])?;
        git(&["-c", "user.name=GB Deployer", "-c", "user.email=deployer@generalbots.com",
             "commit", "-m", &format!("Initial commit: {}", app.description)])?;
        let commit_hash = git(&["rev-parse", "HEAD"])?;
        git(&["push", "--force", &auth_url, &format!("HEAD:{branch}")])?;

        Ok(commit_hash)
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

        // The workflow file must land on `main` (Forgejo Actions runs
        // workflows from the default branch), but it must never replace the
        // project's history: git-mode workspaces keep `main` as the source
        // of truth, so a force-push of a fresh root commit would destroy it
        // and make the next deploy's push fail with "fetch first".
        // Clone → commit on top → non-force push instead.
        self.push_workflow_preserving(repo_url, &workflow_app).await?;

        Ok(())
    }

    /// Push a small app (CI/CD workflow) onto `main` without replacing
    /// existing history: clone the remote, write the files, commit on top of
    /// the current HEAD, and push non-force. Works for both an empty remote
    /// (creates the initial commit) and a repo with existing history.
    async fn push_workflow_preserving(
        &self,
        repo_url: &str,
        app: &GeneratedApp,
    ) -> Result<(), DeploymentError> {
        let temp_dir = app.temp_dir()?;
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)
                .map_err(|e| DeploymentError::GitError(format!("Failed to clean temp dir: {}", e)))?;
        }
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| DeploymentError::GitError(format!("Failed to create temp dir: {}", e)))?;

        let auth_url = self.add_token_to_url(repo_url);
        let git = |args: &[&str]| -> Result<String, DeploymentError> {
            let output = std::process::Command::new("git")
                .current_dir(&temp_dir)
                .args(args)
                .output()
                .map_err(|e| DeploymentError::GitError(format!("git {}: {e}", args[0])))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DeploymentError::GitError(format!(
                    "git {} failed: {}",
                    args[0],
                    stderr.trim().lines().last().unwrap_or("unknown")
                )));
            }
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        };

        // Clone the existing repo (empty remotes clone cleanly; the working
        // tree is then populated by the file writes below).
        let clone_output = std::process::Command::new("git")
            .current_dir(
                temp_dir
                    .parent()
                    .ok_or_else(|| DeploymentError::GitError("temp dir has no parent".to_string()))?,
            )
            .args(["clone", "--quiet", &auth_url, &temp_dir.to_string_lossy()])
            .output()
            .map_err(|e| DeploymentError::GitError(format!("git clone: {e}")))?;
        if !clone_output.status.success() {
            let stderr = String::from_utf8_lossy(&clone_output.stderr);
            return Err(DeploymentError::GitError(format!(
                "git clone failed: {}",
                stderr.trim().lines().last().unwrap_or("unknown")
            )));
        }

        // Ensure we are on `main` even when the clone produced no checkout
        // (empty remote) or checked out a different default branch.
        git(&["checkout", "-q", "-B", "main"])?;

        for file in &app.files {
            let file_path = temp_dir.join(&file.path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| DeploymentError::GitError(format!("Failed to create parent dir: {}", e)))?;
            }
            std::fs::write(&file_path, &file.content)
                .map_err(|e| DeploymentError::GitError(format!("Failed to write file: {}", e)))?;
        }

        git(&["add", "-A"])?;
        // The workflow may already exist and be identical (re-deploy after a
        // rebase brought it in): a "nothing to commit" is a no-op, not an
        // error — the push below then just confirms everything is in sync.
        let commit_out = std::process::Command::new("git")
            .current_dir(&temp_dir)
            .args(["-c", "user.name=GB Deployer", "-c", "user.email=deployer@generalbots.com",
                   "commit", "-m", &format!("Add CI/CD workflow: {}", app.description)])
            .output()
            .map_err(|e| DeploymentError::GitError(format!("git commit: {e}")))?;
        let commit_text = format!(
            "{} {}",
            String::from_utf8_lossy(&commit_out.stdout),
            String::from_utf8_lossy(&commit_out.stderr)
        );
        if !commit_out.status.success()
            && !(commit_text.contains("nothing to commit")
                || commit_text.contains("no changes added to commit"))
        {
            let stderr = String::from_utf8_lossy(&commit_out.stderr);
            return Err(DeploymentError::GitError(format!(
                "git commit failed: {}",
                stderr.trim().lines().last().unwrap_or("unknown")
            )));
        }
        git(&["push", &auth_url, "HEAD:main"])?;

        Ok(())
    }

    fn add_token_to_url(&self, url: &str) -> String {
        // Forgejo personal access tokens authenticate over HTTP(S) via
        // `token@host`; the self-hosted ALM serves plain `http://` clone
        // URLs, so both schemes must embed the token or the push fails
        // with "could not read Username for ..." (no auth at all).
        if let Some(rest) = url.strip_prefix("https://") {
            format!("https://{}@{}", self.token, rest)
        } else if let Some(rest) = url.strip_prefix("http://") {
            format!("http://{}@{}", self.token, rest)
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
