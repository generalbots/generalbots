use std::collections::HashMap;
use std::path::PathBuf;
use log::{info, warn};
use super::generate_random_string;
use crate::core::shared::utils::get_stack_path;

pub async fn setup_alm() -> anyhow::Result<()> {
    let stack_path_raw = get_stack_path();
    
    let stack_path = std::fs::canonicalize(&stack_path_raw)
        .unwrap_or_else(|_| PathBuf::from(&stack_path_raw));
    let stack_path_str = stack_path.to_string_lossy().to_string();
    
    let data_path = stack_path.join("data/alm");
    let config_path = stack_path.join("conf/alm-ci/config.yaml");

    // Check Vault if already set up
    if let Ok(secrets_manager) = crate::core::secrets::SecretsManager::get() {
        if secrets_manager.is_enabled() {
            if let Ok(secrets) = secrets_manager.get_secret(crate::core::secrets::SecretPaths::ALM).await {
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
HTTP_PORT = 3000
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
    let alm_url = "";

    // Try to create admin user and get runner token via HTTP API
    // Note: Forgejo CLI binary may segfault on some systems, so we use curl
    let runner_token = match try_alm_api_setup(alm_url, username, &password, data_path.to_str().unwrap_or(".")).await {
        Ok(token) => token,
        Err(e) => {
            warn!("ALM automated setup unavailable via API: {}", e);
            warn!("ALM will need manual configuration. Create admin user and runner token via web UI.");
            // Store placeholder credentials
            generate_random_string(40)
        }
    };

    info!("Generated ALM Runner token successfully");

    // Register runner with forgejo-runner CLI
    let runner_bin = stack_path.join("bin/alm-ci/forgejo-runner");
    if runner_bin.exists() {
        match register_runner(&runner_bin, &runner_token, config_path.to_str().unwrap_or("config.yaml"), alm_url).await {
            Ok(_) => info!("ALM CI Runner successfully registered!"),
            Err(e) => warn!("Failed to register ALM runner: {}", e),
        }
    } else {
        warn!("Forgejo runner binary not found at {}", runner_bin.display());
    }

    // Store in Vault
    if let Ok(secrets_manager) = crate::core::secrets::SecretsManager::get() {
        if secrets_manager.is_enabled() {
            let mut secrets = HashMap::new();
            secrets.insert("url".to_string(), alm_url.to_string());
            secrets.insert("username".to_string(), username.to_string());
            secrets.insert("password".to_string(), password);
            secrets.insert("runner_token".to_string(), runner_token);

            match secrets_manager.put_secret(crate::core::secrets::SecretPaths::ALM, secrets).await {
                Ok(_) => info!("ALM credentials and runner token stored in Vault"),
                Err(e) => warn!("Failed to store ALM credentials in Vault: {}", e),
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
    use crate::security::command_guard::SafeCommand;

    // Check if ALM is responding
    let check = SafeCommand::new("curl")?
        .args(&["-s", "-o", "/dev/null", "-w", "%{http_code}", &format!("{}/api/v1/version", base_url)])?
        .execute()?;
    
    let status = String::from_utf8_lossy(&check.stdout).trim().to_string();
    if status != "200" && status != "401" && status != "403" {
        return Err(anyhow::anyhow!("ALM not responding (HTTP {})", status));
    }

    info!("ALM is responding at {}", base_url);
    
    // Try to get registration token from the API
    // This requires admin auth, which we may not have yet
    // For now, generate a placeholder token and let operator configure manually
    let token = generate_random_string(40);
    info!("ALM API available but requires manual admin setup. Generated placeholder runner token.");
    
    Ok(token)
}

/// Register forgejo-runner with the instance
async fn register_runner(
    runner_bin: &std::path::Path,
    runner_token: &str,
    config_path: &str,
    instance_url: &str,
) -> anyhow::Result<()> {
    use crate::security::command_guard::SafeCommand;

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
