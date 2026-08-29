//! directory_setup - extracted from bootstrap.rs

use botcore::shared::utils::get_stack_path;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Resolve a stable SaaS JWT secret for cloud API tokens.
/// Priority: `directory_config.json` → env `SAAS_JWT_SECRET` → env `JWT_SECRET` →
/// development default. The resolved value is persisted into
/// `directory_config.json` so cloud tokens survive botserver restarts.
pub(crate) fn resolve_saas_jwt_secret() -> String {
    let stack_path = get_stack_path();
    let config_path = format!("{}/conf/system/directory_config.json", stack_path);

    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(secret) = json
                .get("saas_jwt_secret")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                return secret.to_string();
            }
        }
    }

    let secret = std::env::var("SAAS_JWT_SECRET")
        .or_else(|_| std::env::var("JWT_SECRET"))
        .unwrap_or_else(|_| {
            info!("SAAS_JWT_SECRET not set, using development default secret");
            "dev-secret-key-change-in-production-minimum-32-chars".to_string()
        });

    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
            if json.get("saas_jwt_secret").is_none() {
                json["saas_jwt_secret"] = serde_json::Value::String(secret.clone());
                match std::fs::File::create(&config_path) {
                    Ok(mut file) => {
                        use std::io::Write;
                        if let Ok(pretty) = serde_json::to_string_pretty(&json) {
                            let _ = writeln!(&mut file, "{pretty}");
                        }
                    }
                    Err(e) => warn!("Failed to persist saas_jwt_secret: {e}"),
                }
            }
        }
    }

    secret
}


/// Resolve the Zitadel config for the directory service.
/// Priority: `directory_config.json` → Vault `secret/gbo/directory` → defaults.
/// The Vault fallback lets a fresh stack pick up the OAuth client even when the
/// on-disk config has not been written yet, instead of starting with an empty
/// `api_url` (which produces "builder error" on every token request).
async fn resolve_directory_config() -> crate::directory::ZitadelConfig {
    let stack_path = get_stack_path();
    // Same path DirectorySetup saves to (BOTSERVER_STACK_PATH/conf/system/directory_config.json)
    let config_path = format!("{}/conf/system/directory_config.json", stack_path);

    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if let Ok(config) = serde_json::from_str::<crate::directory::ZitadelConfig>(&content) {
            if !config.api_url.is_empty() {
                info!(
                    "Loaded Zitadel config from {}: url={}",
                    config_path, config.api_url
                );
                return config;
            }
        }
        info!("directory_config.json missing or invalid, trying Vault");
    }

    match load_directory_config_from_vault().await {
        Ok(config) => {
            info!(
                "Loaded Zitadel config from Vault (secret/gbo/directory): url={}",
                config.api_url
            );
            config
        }
        Err(e) => {
            warn!(
                "directory_config.json not found and Vault lookup failed ({}): using default Zitadel config",
                e
            );
            default_zitadel_config()
        }
    }
}

/// Load the Zitadel config from Vault `secret/gbo/directory`.
/// Vault may store `url`/`api_url`; older entries use `host`/`port`, which is
/// normalized into an `api_url` here.
async fn load_directory_config_from_vault() -> anyhow::Result<crate::directory::ZitadelConfig> {
    let secrets_manager = botcoresecrets::SecretsManager::get()?;
    if !secrets_manager.is_enabled() {
        anyhow::bail!("Vault not enabled");
    }
    let secrets = secrets_manager
        .get_secret(botcoresecrets::SecretPaths::DIRECTORY)
        .await?;

    let url = secrets
        .get("url")
        .cloned()
        .filter(|u| !u.is_empty())
        .or_else(|| secrets.get("api_url").cloned().filter(|u| !u.is_empty()))
        .map(Ok)
        .unwrap_or_else(|| {
            let host = secrets
                .get("host")
                .cloned()
                .filter(|h| !h.is_empty())
                .unwrap_or_else(|| "localhost".to_string());
            let port = secrets
                .get("port")
                .cloned()
                .filter(|p| !p.is_empty())
                .unwrap_or_else(|| "8300".to_string());
            let base = if port == "443" {
                format!("https://{host}")
            } else {
                format!("http://{host}:{port}")
            };
            Ok::<String, anyhow::Error>(base)
        })?;

    Ok(crate::directory::ZitadelConfig {
        issuer_url: secrets
            .get("issuer_url")
            .cloned()
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| url.clone()),
        issuer: secrets
            .get("issuer")
            .cloned()
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| url.clone()),
        client_id: secrets.get("client_id").cloned().unwrap_or_default(),
        client_secret: secrets.get("client_secret").cloned().unwrap_or_default(),
        redirect_uri: secrets
            .get("redirect_uri")
            .cloned()
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| "/callback".to_string()),
        project_id: secrets
            .get("project_id")
            .cloned()
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| "default".to_string()),
        api_url: url,
        service_account_key: secrets.get("service_account_key").cloned(),
    })
}

pub(crate) async fn init_directory_service() -> Result<(Arc<Mutex<crate::directory::AuthService>>, crate::directory::ZitadelConfig), std::io::Error> {
    let zitadel_config = resolve_directory_config().await;

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

    // With no reachable URL there is nothing to bootstrap; retrying would just
    // spam "Failed to get access token: builder error".
    if zitadel_config.api_url.trim().is_empty() {
        warn!(
            "Cannot bootstrap directory admin: no Zitadel api_url configured. \
             Provide conf/system/directory_config.json or set secret/gbo/directory in Vault."
        );
        return;
    }

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

    let max_retries = 24;
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
