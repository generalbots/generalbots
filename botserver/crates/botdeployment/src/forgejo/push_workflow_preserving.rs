use super::*;
use botlib::security::redact::redact_alm_secret;
use botlib::security::SafeCommand;

/// Strict single-line allowlist for values interpolated into generated
/// workflow YAML (#2649). Newlines, quotes and shell/YAML metacharacters must
/// never reach a file that runs with deploy secrets.
fn validate_workflow_field(name: &str, value: &str) -> Result<(), DeploymentError> {
    if value.is_empty() || value.len() > 128 {
        return Err(DeploymentError::ConfigurationError(format!(
            "workflow field {name} must be 1-128 characters"
        )));
    }
    if value
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || matches!(c, '_' | ' ' | '.' | '/' | ':' | '-')))
    {
        return Err(DeploymentError::ConfigurationError(format!(
            "workflow field {name} contains a forbidden character"
        )));
    }
    Ok(())
}

/// Defense-in-depth gate on the fully-rendered workflow (#2649). Every dynamic
/// value is allowlisted above, so the template's line count is fixed; an
/// injected newline would change it. NUL/CR are never legal in a workflow
/// that CI will execute with deploy secrets. This check runs before the YAML
/// string is handed to `create_cicd_workflow` (and hence before any write).
fn validate_rendered_workflow(yaml: &str) -> Result<(), DeploymentError> {
    if yaml.contains('\0') || yaml.contains('\r') {
        return Err(DeploymentError::ConfigurationError(
            "generated workflow contains a forbidden control character".to_string(),
        ));
    }
    const EXPECTED_LINES: usize = 43;
    let line_count = yaml.lines().count();
    if line_count != EXPECTED_LINES {
        return Err(DeploymentError::ConfigurationError(format!(
            "generated workflow line count changed ({line_count} != {EXPECTED_LINES})"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod workflow_injection_tests {
    use super::*;

    fn test_client() -> ForgejoClient {
        ForgejoClient {
            base_url: "http://127.0.0.1:0".to_string(),
            token: "test-token".to_string(),
            client: Client::new(),
        }
    }

    fn app(
        framework: &str,
        node_version: Option<&str>,
        build_command: Option<&str>,
        output_directory: Option<&str>,
    ) -> ProjectType {
        ProjectType::App {
            framework: framework.to_string(),
            node_version: node_version.map(str::to_string),
            build_command: build_command.map(str::to_string),
            output_directory: output_directory.map(str::to_string),
        }
    }

    fn generate(pt: &ProjectType) -> Result<String, DeploymentError> {
        test_client().generate_app_workflow(
            pt,
            &DeployTarget::IncusContainer,
            &DeploymentEnvironment::Production,
        )
    }

    #[test]
    fn rejects_multiline_and_shell_metacharacters() {
        assert!(validate_workflow_field("build_command", "npm run build").is_ok());
        assert!(validate_workflow_field("output_directory", "./dist/app").is_ok());
        assert!(
            validate_workflow_field("build_command", "npm run build\n  - run: curl evil").is_err()
        );
        assert!(validate_workflow_field("build_command", "npm; curl evil").is_err());
        assert!(validate_workflow_field("framework", "$(id)").is_err());
        assert!(validate_workflow_field("node_version", "'20'").is_err());
        assert!(validate_workflow_field("output_directory", "").is_err());
    }

    #[test]
    fn rejects_nul_quotes_backticks_and_control_characters() {
        assert!(validate_workflow_field("build_command", "npm\0run").is_err());
        assert!(validate_workflow_field("build_command", "npm\rrun").is_err());
        assert!(validate_workflow_field("build_command", "npm\trun").is_err());
        assert!(validate_workflow_field("build_command", "npm`id`").is_err());
        assert!(validate_workflow_field("build_command", "npm\"run").is_err());
        assert!(validate_workflow_field("build_command", "npm|curl").is_err());
        assert!(validate_workflow_field("build_command", "npm&curl").is_err());
    }

    #[test]
    fn generate_app_workflow_rejects_hostile_build_fields() {
        // A newline-injected build command must be refused before any YAML is
        // produced; create_cicd_workflow uses `?` on this result before it
        // builds the GeneratedFile, so no `.forgejo/workflows/deploy.yml` is
        // ever written for these inputs.
        assert!(generate(&app(
            "htmx",
            None,
            Some("npm run build\n  - run: curl evil"),
            None
        ))
        .is_err());
        assert!(generate(&app("htmx", None, None, Some("./dist\0;curl"))).is_err());
        assert!(generate(&app("$(id)", None, None, None)).is_err());
        assert!(generate(&app("htmx", Some("20'"), None, None)).is_err());
        assert!(generate(&app("htmx", None, Some("npm;curl evil"), None)).is_err());

        let benign = app("htmx", Some("20"), Some("npm run build"), Some("./dist"));
        let yaml = generate(&benign).expect("benign workflow must render");
        assert!(yaml.contains("name: Deploy htmx App"));
        assert!(validate_rendered_workflow(&yaml).is_ok());
    }

    #[test]
    fn rendered_workflow_guard_rejects_injected_step() {
        let benign = app("htmx", Some("20"), Some("npm run build"), Some("./dist"));
        let yaml = generate(&benign).expect("benign workflow must render");
        let injected = format!("{yaml}\n      - run: curl evil");
        assert!(validate_rendered_workflow(&injected).is_err());
        assert!(validate_rendered_workflow("name: x\0").is_err());
        assert!(validate_rendered_workflow("name: x\r\n").is_err());
    }
}

impl ForgejoClient {
    pub(crate) async fn push_workflow_preserving(
        &self,
        repo_url: &str,
        app: &GeneratedApp,
    ) -> Result<(), DeploymentError> {
        let temp_dir = app.temp_dir()?;
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir).map_err(|e| {
                DeploymentError::GitError(format!("Failed to clean temp dir: {}", e))
            })?;
        }
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| DeploymentError::GitError(format!("Failed to create temp dir: {}", e)))?;

        let auth_url = self.add_token_to_url(repo_url);
        // #2860: git errors may echo the token-bearing URL passed as an argv
        // element; strip the secret before the error can be logged upstream.
        let sanitize =
            |msg: String| DeploymentError::GitError(redact_alm_secret(&msg, &self.token));
        let git = |args: &[&str]| -> Result<String, DeploymentError> {
            let label = args.first().copied().unwrap_or("git");
            let mut command = SafeCommand::new("git")
                .map_err(|e| sanitize(format!("git {label}: {e}")))?
                .allow_path(temp_dir.clone());
            for arg in args {
                command = command
                    .trusted_arg(arg)
                    .map_err(|e| sanitize(format!("git {label}: {e}")))?;
            }
            let output = command
                .working_dir(&temp_dir)
                .map_err(|e| sanitize(format!("git {label}: {e}")))?
                .execute()
                .map_err(|e| sanitize(format!("git {label}: {e}")))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(sanitize(format!(
                    "git {} failed: {}",
                    label,
                    stderr.trim().lines().last().unwrap_or("unknown")
                )));
            }
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        };

        // Clone the existing repo (empty remotes clone cleanly; the working
        // tree is then populated by the file writes below).
        let parent_dir = temp_dir
            .parent()
            .ok_or_else(|| DeploymentError::GitError("temp dir has no parent".to_string()))?;
        let clone_target = temp_dir.to_string_lossy().to_string();
        let clone_output = SafeCommand::new("git")
            .map(|c| c.allow_path(parent_dir.to_path_buf()))
            .and_then(|c| c.args(&["clone", "--quiet"]))
            .and_then(|c| c.trusted_arg(&auth_url))
            .and_then(|c| c.trusted_arg(&clone_target))
            .and_then(|c| c.working_dir(parent_dir))
            .and_then(|c| c.execute())
            .map_err(|e| sanitize(format!("git clone: {e}")))?;
        if !clone_output.status.success() {
            let stderr = String::from_utf8_lossy(&clone_output.stderr);
            return Err(sanitize(format!(
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
                std::fs::create_dir_all(parent).map_err(|e| {
                    DeploymentError::GitError(format!("Failed to create parent dir: {}", e))
                })?;
            }
            std::fs::write(&file_path, &file.content)
                .map_err(|e| DeploymentError::GitError(format!("Failed to write file: {}", e)))?;
        }

        git(&["add", "-A"])?;
        // The workflow may already exist and be identical (re-deploy after a
        // rebase brought it in): a "nothing to commit" is a no-op, not an
        // error — the push below then just confirms everything is in sync.
        let commit_message = format!("Add CI/CD workflow: {}", app.description);
        let commit_out = SafeCommand::new("git")
            .and_then(|c| {
                c.args(&[
                    "-c",
                    "user.name=GB Deployer",
                    "-c",
                    "user.email=deployer@generalbots.com",
                    "commit",
                    "-m",
                ])
            })
            .and_then(|c| c.trusted_arg(&commit_message))
            .and_then(|c| c.working_dir(&temp_dir))
            .and_then(|c| c.execute())
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

    pub(super) fn add_token_to_url(&self, url: &str) -> String {
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

    pub(super) fn generate_app_workflow(
        &self,
        project_type: &ProjectType,
        _deploy_target: &DeployTarget,
        environment: &DeploymentEnvironment,
    ) -> Result<String, DeploymentError> {
        let env_name = environment.to_string();
        let (framework, node_version, build_command, output_dir) = match project_type {
            ProjectType::App {
                framework,
                node_version,
                build_command,
                output_directory,
            } => (
                framework.clone(),
                node_version.clone().unwrap_or_else(|| "20".to_string()),
                build_command
                    .clone()
                    .unwrap_or_else(|| "npm run build".to_string()),
                output_directory
                    .clone()
                    .unwrap_or_else(|| "dist".to_string()),
            ),
            _ => (
                "htmx".to_string(),
                "20".to_string(),
                "npm run build".to_string(),
                "dist".to_string(),
            ),
        };

        validate_workflow_field("framework", &framework)?;
        validate_workflow_field("node_version", &node_version)?;
        validate_workflow_field("build_command", &build_command)?;
        validate_workflow_field("output_directory", &output_dir)?;

        let workflow = format!(
            r#"name: Deploy {framework} App

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
"#
        );
        validate_rendered_workflow(&workflow)?;
        Ok(workflow)
    }

    pub(super) fn generate_site_workflow(&self, environment: &DeploymentEnvironment) -> String {
        let env_name = environment.to_string();

        format!(
            r#"name: Deploy Static Site

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
"#
        )
    }
}
