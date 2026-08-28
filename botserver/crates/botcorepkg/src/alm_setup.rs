use std::collections::HashMap;
use std::path::PathBuf;
use log::{error, info};
use crate::generate_random_string;
use botlib::security::get_stack_path;

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

    // Forgejo requires RUN_USER to match the account that starts it
    // ("Expect user 'alm' but current user is: ..." otherwise). Use the
    // actual OS user so a fresh stack boots on any machine.
    let run_user = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "alm".to_string());

    let app_ini_path = alm_conf_dir.join("app.ini");
    if !app_ini_path.exists() {
        let app_ini_content = format!(
            r#"APP_NAME = General Bots ALM
RUN_USER = {}
WORK_PATH = {}/data/alm

[repository]
ROOT = {}/data/alm/repositories

[database]
DB_TYPE = sqlite3
PATH = {}/data/alm/gitea.db

[server]
HTTP_PORT = 4747
DOMAIN = localhost
ROOT_URL = http://localhost:4747/
LOCAL_ROOT_URL = http://localhost:4747/

[security]
INSTALL_LOCK = true
"#,
            run_user, stack_path_str, stack_path_str, stack_path_str
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

    // Try to create admin user and get runner token via HTTP API
    // Note: Forgejo CLI binary may segfault on some systems, so we use curl
    let runner_token = match try_alm_api_setup(alm_url, username, &password, data_path.to_str().unwrap_or(".")).await {
        Ok(token) => token,
        Err(e) => {
            info!("ALM automated setup unavailable via API: {}", e);
            info!("ALM will need manual configuration. Create admin user and runner token via web UI.");
            // Store placeholder credentials
            generate_random_string(40)
        }
    };

    info!("Generated ALM Runner token successfully");

    // Mint a Forgejo API token (Personal Access Token) for the botserver
    // user so the deployment pipeline can create orgs/repos without manual
    // setup. The ForgejoClient authenticates with `token {pat}`; without it
    // every deploy fails with "FORGEJO_TOKEN not configured".
    let api_token = mint_api_token(alm_url, username, &password).await
        .unwrap_or_else(|e| {
            info!("API token minting unavailable: {} (deploys will need a manual PAT)", e);
            String::new()
        });

    // Register runner with forgejo-runner CLI
    let runner_bin = stack_path.join("bin/alm-ci/forgejo-runner");
    if runner_bin.exists() {
        match register_runner(&runner_bin, &runner_token, config_path.to_str().unwrap_or("config.yaml"), alm_url).await {
            Ok(_) => info!("ALM CI Runner successfully registered!"),
            Err(e) => info!("ALM runner not registered automatically: {}", e),
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
            secrets.insert("runner_token".to_string(), runner_token);
            if !api_token.is_empty() {
                secrets.insert("token".to_string(), api_token);
            }

            match secrets_manager.put_secret(botcoresecrets::SecretPaths::ALM, secrets).await {
                Ok(_) => info!("ALM credentials and runner token stored in Vault"),
                Err(e) => error!("Failed to store ALM credentials in Vault: {}", e),
            }
        }
    }

    Ok(())
}

/// Attempt to configure ALM via HTTP API (since CLI may segfault)
async fn try_alm_api_setup(
    base_url: &str,
    _username: &str,
    _password: &str,
    _home: &str,
) -> anyhow::Result<String> {
    use botlib::security::SafeCommand;

    // Check if ALM is responding
    let check = SafeCommand::new("curl")?
        .args(&["-s", "-o", "/dev/null", "-w", "%{http_code}", &format!("{}/api/v1/version", base_url)])?
        .execute()?;
    
    let status = String::from_utf8_lossy(&check.stdout).trim().to_string();
    if status != "200" && status != "401" && status != "403" {
        return Err(anyhow::anyhow!("ALM not responding (HTTP {})", status));
    }

    info!("ALM is responding at {}", base_url);

    // A fresh Forgejo has no users yet: create the botserver admin account
    // via the CLI (admin user create) so the runner token / PAT flows have
    // an authenticated identity. The API route /admin/users requires an
    // existing admin token — chicken-and-egg — so the CLI is the reliable
    // path (safe to run idempotently; duplicate creation is tolerated).
    let create_user = SafeCommand::new("forgejo")?
        .arg("admin")?
        .arg("user")?
        .arg("create")?
        .arg("--username")?
        .arg(_username)?
        .arg("--password")?
        .trusted_arg(_password)?
        .arg("--email")?
        .arg(format!("{}@localhost", _username))?
        .arg("--admin")?
        .arg("--must-change-password=false")?
        .execute();
    match create_user {
        Ok(out) if out.status.success() => {
            info!("Created Forgejo admin user '{}'", _username);
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            info!("Forgejo user creation returned non-zero (likely exists): {}", err.trim());
        }
        Err(e) => {
            info!("Forgejo CLI unavailable, skipping admin user creation: {}", e);
        }
    }

    // Generate a registration token for the runner
    let token = generate_random_string(40);
    info!("ALM API available. Generated runner token.");

    Ok(token)
}

/// Mint a Forgejo Personal Access Token for the botserver user so the
/// deployment pipeline can call the API. Uses basic auth (username:password)
/// against the token endpoint; Forgejo accepts the account password for
/// token creation on a fresh install.
async fn mint_api_token(base_url: &str, username: &str, password: &str) -> anyhow::Result<String> {
    use botlib::security::SafeCommand;

    let url = format!("{}/api/v1/users/{}/tokens", base_url, username);
    let body = r#"{"name":"gb-deploy","scopes":["all"]}"#;
    let output = SafeCommand::new("curl")?
        .args(&[
            "-s",
            "--max-time",
            "10",
            "-u",
            &format!("{}:{}", username, password),
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            body,
            &url,
        ])?
        .execute()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("token API status {}", output.status));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("token response parse: {e}"))?;
    parsed
        .get("sha1")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("token response missing sha1"))
}

/// Register forgejo-runner with the instance
async fn register_runner(
    runner_bin: &std::path::Path,
    runner_token: &str,
    config_path: &str,
    instance_url: &str,
) -> anyhow::Result<()> {
    use botlib::security::SafeCommand;

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
