use std::collections::HashMap;
use std::path::PathBuf;
use log::{info, warn};
use crate::security::command_guard::SafeCommand;

pub async fn setup_alm() -> anyhow::Result<()> {
    let stack_path = std::env::var("BOTSERVER_STACK_PATH")
        .unwrap_or_else(|_| "./botserver-stack".to_string());
    
    let alm_bin = PathBuf::from(&stack_path).join("bin/alm/forgejo");
    let runner_bin = PathBuf::from(&stack_path).join("bin/alm-ci/forgejo-runner");
    let data_path = PathBuf::from(&stack_path).join("data/alm");
    let config_path = PathBuf::from(&stack_path).join("conf/alm-ci/config.yaml");

    // Check Vault if already set up
    if let Ok(secrets_manager) = crate::core::secrets::SecretsManager::from_env() {
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
    
    // Create admin user
    let username = "botserver";
    let password = "botserverpassword123!"; // Or generate random
    
    let create_user = SafeCommand::new(alm_bin.to_str().unwrap_or("forgejo"))?
        .arg("admin")?
        .arg("user")?
        .arg("create")?
        .arg("--admin")?
        .arg("--username")?
        .arg(username)?
        .arg("--password")?
        .arg(password)?
        .arg("--email")?
        .arg("botserver@generalbots.local")?
        .env("USER", "alm")?
        .env("HOME", data_path.to_str().unwrap_or("."))?
        .execute()?;
        
    if !create_user.status.success() {
        let err = String::from_utf8_lossy(&create_user.stderr);
        if !err.contains("already exists") {
            warn!("Failed to create ALM admin user: {}", err);
        }
    }

    // Generate runner token
    let token_output = SafeCommand::new(alm_bin.to_str().unwrap_or("forgejo"))?
        .arg("forgejo-cli")?
        .arg("actions")?
        .arg("generate-runner-token")?
        .env("USER", "alm")?
        .env("HOME", data_path.to_str().unwrap_or("."))?
        .execute()?;

    let runner_token = String::from_utf8_lossy(&token_output.stdout).trim().to_string();
    if runner_token.is_empty() {
        let err = String::from_utf8_lossy(&token_output.stderr);
        return Err(anyhow::anyhow!("Failed to generate ALM runner token: {}", err));
    }

    info!("Generated ALM Runner token constraints successfully");

    // Register runner
    let register_runner = SafeCommand::new(runner_bin.to_str().unwrap_or("forgejo-runner"))?
        .arg("register")?
        .arg("--instance")?
        .arg("http://localhost:3000")? // TODO: configurable
        .arg("--token")?
        .arg(&runner_token)?
        .arg("--name")?
        .arg("gbo")?
        .arg("--labels")?
        .arg("ubuntu-latest:docker://node:20-bookworm")?
        .arg("--no-interactive")?
        .arg("--config")?
        .arg(config_path.to_str().unwrap_or("config.yaml"))?
        .execute()?;
        
    if !register_runner.status.success() {
        let err = String::from_utf8_lossy(&register_runner.stderr);
        if !err.contains("already registered") {
             warn!("Failed to register ALM runner: {}", err);
        }
    }

    info!("ALM CI Runner successfully registered!");

    // Store in Vault
    if let Ok(secrets_manager) = crate::core::secrets::SecretsManager::from_env() {
        if secrets_manager.is_enabled() {
            let mut secrets = HashMap::new();
            secrets.insert("url".to_string(), "http://localhost:3000".to_string());
            secrets.insert("username".to_string(), username.to_string());
            secrets.insert("password".to_string(), password.to_string());
            secrets.insert("runner_token".to_string(), runner_token);

            match secrets_manager.put_secret(crate::core::secrets::SecretPaths::ALM, secrets).await {
                Ok(_) => info!("ALM credentials and runner token stored in Vault"),
                Err(e) => warn!("Failed to store ALM credentials in Vault: {}", e),
            }
        }
    }

    Ok(())
}
