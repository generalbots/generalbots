//! directory_setup - extracted from bootstrap.rs

use botcore::shared::utils::get_stack_path;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;


pub(crate) fn init_directory_service() -> Result<(Arc<Mutex<crate::directory::AuthService>>, crate::directory::ZitadelConfig), std::io::Error> {
    let zitadel_config = {
        // Try to load from directory_config.json first
        // Use same path as DirectorySetup saves to (BOTSERVER_STACK_PATH/conf/system/directory_config.json)
        let stack_path = get_stack_path();
        let config_path = format!("{}/conf/system/directory_config.json", stack_path);
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let base_url = json
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let client_id = json.get("client_id").and_then(|v| v.as_str()).unwrap_or("");
                let client_secret = json
                    .get("client_secret")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                info!(
                    "Loaded Zitadel config from {}: url={}",
                    config_path, base_url
                );

                crate::directory::ZitadelConfig {
                    issuer_url: base_url.to_string(),
                    issuer: base_url.to_string(),
                    client_id: client_id.to_string(),
                    client_secret: client_secret.to_string(),
                    redirect_uri: format!("{}/callback", base_url),
                    project_id: "default".to_string(),
                    api_url: base_url.to_string(),
                    service_account_key: None,
                }
            } else {
                info!("Failed to parse directory_config.json, using defaults");
                default_zitadel_config()
            }
        } else {
            info!("directory_config.json not found, using default Zitadel config");
            default_zitadel_config()
        }
    };

    let auth_service = Arc::new(tokio::sync::Mutex::new(
        crate::directory::AuthService::new(zitadel_config.clone())
            .map_err(|e| std::io::Error::other(format!("Failed to create auth service: {}", e)))?,
    ));

    Ok((auth_service, zitadel_config))
}

fn default_zitadel_config() -> crate::directory::ZitadelConfig {
    crate::directory::ZitadelConfig {
        issuer_url: "".to_string(),
        issuer: "".to_string(),
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: "/callback".to_string(),
        project_id: "default".to_string(),
        api_url: "".to_string(),
        service_account_key: None,
    }
}

pub(crate) async fn bootstrap_directory_admin(zitadel_config: &crate::directory::ZitadelConfig) {
    use crate::directory::{bootstrap, ZitadelClient};

    let stack = get_stack_path();
    let pat_path = std::path::PathBuf::from(format!("{}/conf/directory/admin-pat.txt", stack));
    let bootstrap_client = if pat_path.exists() {
        match std::fs::read_to_string(pat_path) {
            Ok(pat_token) => {
                let pat_token = pat_token.trim().to_string();
                info!("Using admin PAT token for bootstrap authentication");
                ZitadelClient::with_pat_token(zitadel_config.clone(), pat_token)
                    .map_err(|e| {
                        std::io::Error::other(format!(
                            "Failed to create bootstrap client with PAT: {}",
                            e
                        ))
                    })
            }
            Err(e) => {
                warn!(
                    "Failed to read admin PAT token: {}, falling back to OAuth2",
                    e
                );
                ZitadelClient::new(zitadel_config.clone()).map_err(|e| {
                    std::io::Error::other(format!("Failed to create bootstrap client: {}", e))
                })
            }
        }
    } else {
        info!("Admin PAT not found, using OAuth2 client credentials for bootstrap");
        ZitadelClient::new(zitadel_config.clone()).map_err(|e| {
            std::io::Error::other(format!("Failed to create bootstrap client: {}", e))
        })
    };

    let bootstrap_client = match bootstrap_client {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create bootstrap client: {}", e);
            return;
        }
    };

    let max_retries = 12;
    let mut attempt = 0;
    loop {
        match bootstrap::check_and_bootstrap_admin(&bootstrap_client).await {
            Ok(Some(_)) => {
                info!("Bootstrap completed - admin credentials displayed in console");
                break;
            }
            Ok(None) => {
                info!("Admin user exists, bootstrap skipped");
                break;
            }
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    info!(
                        "Bootstrap check skipped after {} retries: {}",
                        max_retries, e
                    );
                    break;
                }
                info!(
                    "Bootstrap check failed (attempt {}/{}), waiting for Zitadel: {}",
                    attempt, max_retries, e
                );
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }

    let management_client = match ZitadelClient::new(zitadel_config.clone()) {
        Ok(client) => client,
        Err(e) => {
            warn!("Failed to create management client with OAuth2: {}", e);
            return;
        }
    };

    if let Err(e) = bootstrap::ensure_default_organization(&management_client).await {
        info!("Failed to ensure default organization (non-critical): {}", e);
    }

    if let Err(e) = bootstrap::ensure_admin_user(&management_client).await {
        info!("Failed to ensure admin user (non-critical): {}", e);
    }
}
