use std::collections::HashMap;
use std::path::PathBuf;
use log::{error, info};

pub async fn setup_alm() -> anyhow::Result<()> {
    let stack_path_raw = get_stack_path();
    
    let stack_path = std::fs::canonicalize(&stack_path_raw)
        .unwrap_or_else(|_| PathBuf::from(&stack_path_raw));
    let stack_path_str = stack_path.to_string_lossy().to_string();
    
    let data_path = stack_path.join("data/alm");
    let config_path = stack_path.join("conf/alm-ci/config.yaml");

    // Check Vault if already set up
    if let Ok(secrets_manager) = botcoresecrets::SecretsManager::get() {
        if secrets_manager.is_enabled() {
            if let Ok(secrets) = secrets_manager.get_secret(botcoresecrets::SecretPaths::ALM).await {
                if let (Some(user), Some(token)) = (secrets.get("username"), secrets.get("runner_token")) {
                    if !user.is_empty() && !token.is_empty() {
                        info!("ALM is already configured in Vault for user {}", user);
                        return Ok(());
                    }
                }
            }
        }
    }

    info!("Initializing ALM (Forgejo) and CI Runner...");
    
    // Ensure ALM config directory exists and create minimal app.ini
    let alm_conf_dir = stack_path.join("conf/alm");
    std::fs::create_dir_all(&alm_conf_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create ALM config dir: {}", e))?;
    
    let app_ini_path = alm_conf_dir.join("app.ini");
    if !app_ini_path.exists() {
        let app_ini_content = format!(
            r#"APP_NAME = General Bots ALM
RUN_USER = alm
WORK_PATH = {}/data/alm

[repository]
ROOT = {}/data/alm/repositories

[database]
DB_TYPE = sqlite3
PATH = {}/data/alm/gitea.db

[server]
HTTP_PORT = 4747
DOMAIN = localhost
ROOT_URL = 

[security]
INSTALL_LOCK = true
"#,
            stack_path_str, stack_path_str, stack_path_str
        );
        std::fs::write(&app_ini_path, app_ini_content)
            .map_err(|e| anyhow::anyhow!("Failed to write app.ini: {}", e))?;
        info!("Created minimal ALM app.ini at {}", app_ini_path.display());
    }
    
    // Generate credentials and attempt to configure via HTTP API
    let username = "botserver";
    let password = generate_random_string(32);
    // Self-hosted Forgejo listens locally on 4747 (same as the "alm"
    // component in the installer); no remote host is hardcoded.
    let alm_url = "http://localhost:4747";

    // Complete Forgejo first-run setup and mint an API token for the botserver.
    let api_token = match try_alm_api_setup(alm_url, username, &password, data_path.to_str().unwrap_or(".")).await {
        Ok(token) => token,
        Err(e) => {
            info!("ALM automated setup unavailable via API: {e}");
            info!("ALM will need manual configuration. Create admin user and API token via web UI.");
            generate_random_string(40)
        }
    };

    info!("ALM API token ready");

    // Best-effort: fetch a runner registration token and register the CI runner.
    let runner_token = match fetch_runner_token(alm_url, &api_token).await {
        Ok(token) => token,
        Err(e) => {
            info!("Runner registration token unavailable: {e}");
            generate_random_string(40)
        }
    };

    // Register runner with forgejo-runner CLI
    let runner_bin = stack_path.join("bin/alm-ci/forgejo-runner");
    if runner_bin.exists() {
        match register_runner(&runner_bin, &runner_token, config_path.to_str().unwrap_or("config.yaml"), alm_url).await {
            Ok(_) => info!("ALM CI Runner successfully registered!"),
            Err(e) => info!("ALM runner not registered automatically: {e}"),
        }
    } else {
        info!("Forgejo runner binary not found at {}", runner_bin.display());
    }

    // Store in Vault
    if let Ok(secrets_manager) = botcoresecrets::SecretsManager::get() {
        if secrets_manager.is_enabled() {
            let mut secrets = HashMap::new();
            secrets.insert("url".to_string(), alm_url.to_string());
            secrets.insert("username".to_string(), username.to_string());
            secrets.insert("password".to_string(), password);
            secrets.insert("token".to_string(), api_token);
            secrets.insert("runner_token".to_string(), runner_token);

            match secrets_manager.put_secret(botcoresecrets::SecretPaths::ALM, secrets).await {
                Ok(_) => info!("ALM credentials, API token and runner token stored in Vault"),
                Err(e) => error!("Failed to store ALM credentials in Vault: {}", e),
            }
        }
    }

    Ok(())
}

/// Fetch a Forgejo runner registration token using the botserver API token.
async fn fetch_runner_token(base_url: &str, api_token: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| anyhow::anyhow!("ALM http client: {e}"))?;
    let resp = client
        .get(format!("{base_url}/api/v1/admin/runners/registration-token"))
        .bearer_auth(api_token)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("runner token request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("runner token endpoint returned {}", resp.status()));
    }
    let body: serde_json::Value = resp.json().await.unwrap_or_default();
    body.get("token")
        .and_then(|v| v.as_str())
        .map(|t| t.to_string())
        .ok_or_else(|| anyhow::anyhow!("runner token response missing 'token'"))
}

/// Complete Forgejo first-run setup (if needed) and mint an API token for the
/// botserver admin user. Self-hosted: everything targets `base_url`
/// (http://localhost:4747); no remote host is involved.
async fn try_alm_api_setup(
    base_url: &str,
    username: &str,
    password: &str,
    home: &str,
) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| anyhow::anyhow!("ALM http client: {e}"))?;

    // Installed Forgejo answers /api/v1/version; a fresh instance 404s there
    // and serves the install wizard at the root.
    let installed = client
        .get(format!("{base_url}/api/v1/version"))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !installed {
        info!("ALM at {base_url} is not installed — completing first-run setup");
        let run_user = std::env::var("USER").unwrap_or_else(|_| "alm".to_string());
        let form = [
            ("db_type", "sqlite3"),
            ("db_path", &format!("{home}/gitea.db")),
            ("app_name", "General Bots ALM"),
            ("repo_root_path", &format!("{home}/repositories")),
            ("lfs_root_path", &format!("{home}/data/lfs")),
            ("run_user", run_user.as_str()),
            ("domain", "localhost"),
            ("ssh_port", "22"),
            ("http_port", "4747"),
            ("app_url", &format!("{base_url}/")),
            ("log_root_path", &format!("{home}/log")),
            ("admin_name", username),
            ("admin_passwd", password),
            ("admin_confirm_passwd", password),
            ("admin_email", &format!("{username}@localhost")),
        ];
        let resp = client
            .post(base_url)
            .form(&form)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("ALM install request failed: {e}"))?;
        let install_status = resp.status();
        if !install_status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "ALM install returned {}: {}",
                install_status,
                body
            ));
        }
        info!("ALM first-run install completed");
    }

    // Mint an API token so the botserver can drive the ALM (create repos,
    // create PRs, register runners) without manual key handling.
    let token_resp = client
        .post(format!("{base_url}/api/v1/users/{username}/tokens"))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({ "name": "botserver", "scopes": ["all"] }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("ALM token request failed: {e}"))?;
    let token_status = token_resp.status();
    if token_status.is_success() {
        let body: serde_json::Value = token_resp.json().await.unwrap_or_default();
        if let Some(tok) = body.get("sha1").and_then(|v| v.as_str()) {
            info!("ALM API token minted for user {username}");
            return Ok(tok.to_string());
        }
    }
    Err(anyhow::anyhow!(
        "ALM API token generation failed (status {})",
        token_status
    ))
}

/// Register forgejo-runner with the instance
async fn register_runner(
    runner_bin: &std::path::Path,
    runner_token: &str,
    config_path: &str,
    instance_url: &str,
) -> anyhow::Result<()> {
    use botlib::security::command_guard::SafeCommand;

    let register_output = SafeCommand::new(runner_bin.to_str().unwrap_or("forgejo-runner"))?
        .arg("register")?
        .arg("--instance")?
        .arg(instance_url)?
        .arg("--token")?
        .arg(runner_token)?
        .arg("--name")?
        .arg("gbo")?
        .arg("--labels")?
        .trusted_arg("ubuntu-latest:docker://node:20-bookworm")?
        .arg("--no-interactive")?
        .arg("--config")?
        .arg(config_path)?
        .execute()?;
        
    if !register_output.status.success() {
        let err = String::from_utf8_lossy(&register_output.stderr);
        if !err.contains("already registered") && !err.is_empty() {
            return Err(anyhow::anyhow!("Runner registration failed: {}", err));
        }
    }

    Ok(())
}

use super::generate_random_string;
use crate::shared::utils::get_stack_path;
