//! directory_setup - extracted from bootstrap.rs

use botcore::shared::utils::get_stack_path;
use log::{error, info, warn};
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

    // The embedded/SBC sample config ships a placeholder JWT secret
    // (#1347). Serving with it means every token is forgeable by anyone who
    // read the repository. Dev builds may continue (explicitly warned);
    // release builds get a random per-boot secret instead — nothing is
    // forgeable, and the fix is forced by tokens no longer surviving a
    // restart until a real secret is configured.
    const PLACEHOLDER_SECRETS: [&str; 2] = [
        "embedded-change-me-in-production",
        "dev-secret-key-change-in-production-minimum-32-chars",
    ];
    let secret = if PLACEHOLDER_SECRETS.contains(&secret.as_str()) {
        if cfg!(debug_assertions) {
            warn!(
                "JWT secret is a known placeholder; tokens minted with it are NOT secure - set SAAS_JWT_SECRET or JWT_SECRET"
            );
            secret
        } else {
            use rand::Rng;
            const CHARSET: &[u8; 64] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
            let random: String = (0..48)
                .map(|_| CHARSET[rand::rng().random_range(0..CHARSET.len())] as char)
                .collect();
            error!(
                "JWT secret is a known placeholder; a random per-boot secret was generated so tokens are not forgeable. \
                 Cloud logins will not survive restarts until SAAS_JWT_SECRET or JWT_SECRET is configured."
            );
            random
        }
    } else {
        secret
    };

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

/// The desktop first-run wizard writes this file before the suite opens. It is
/// the only credential for local/single-user installs, where the directory API
/// may not be reachable at all.
#[derive(Debug, Clone)]
pub(crate) struct BootstrapAdminIdentity {
    pub email: String,
    pub user_id: Option<uuid::Uuid>,
}

fn bootstrap_admin_credentials_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    for var in ["BOTSERVER_STACK_PATH", "GBO_STACK_PATH"] {
        if let Ok(stack) = std::env::var(var) {
            if !stack.trim().is_empty() {
                paths.push(
                    std::path::PathBuf::from(stack).join("conf/directory/admin-credentials.json"),
                );
            }
        }
    }
    let stack = get_stack_path();
    paths.push(std::path::PathBuf::from(&stack).join("conf/directory/admin-credentials.json"));
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("botserver-stack/conf/directory/admin-credentials.json"));
        paths.push(cwd.join("../botserver-stack/conf/directory/admin-credentials.json"));
    }
    paths
}

/// Reads the wizard administrator identity from disk. Returns `None` for
/// Zitadel/cloud deployments, which have no `admin-credentials.json`.
pub(crate) fn bootstrap_admin_identity() -> Option<BootstrapAdminIdentity> {
    let mut seen = std::collections::HashSet::new();
    for path in bootstrap_admin_credentials_paths() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        let email = json
            .get("email")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .trim();
        if email.is_empty() {
            continue;
        }
        let user_id = json
            .get("user_id")
            .and_then(|value| value.as_str())
            .and_then(|value| uuid::Uuid::parse_str(value.trim()).ok());
        return Some(BootstrapAdminIdentity {
            email: email.to_string(),
            user_id,
        });
    }
    None
}

/// Whether a session identity is the first-run administrator. Used when a
/// desktop/login path has to create a `users` row on demand.
pub(crate) fn is_bootstrap_admin_identity(user_id: uuid::Uuid, email: Option<&str>) -> bool {
    let Some(identity) = bootstrap_admin_identity() else {
        return false;
    };
    if identity.user_id == Some(user_id) {
        return true;
    }
    email
        .map(|value| identity.email.eq_ignore_ascii_case(value.trim()))
        .unwrap_or(false)
}

/// #1839 — persist the wizard administrator as an admin in the local `users`
/// table. Desktop logins do not go through directory provisioning, so without
/// this the account only appeared after an AI-account connect inserted a bare
/// row with `is_admin = false`.
///
/// No-op when `admin-credentials.json` is absent, so Zitadel/cloud installs
/// are untouched. Idempotent: it upgrades by id or email, otherwise inserts.
pub(crate) fn ensure_bootstrap_admin_user(pool: &botcore::shared::utils::DbPool) {
    use diesel::RunQueryDsl;

    let Some(identity) = bootstrap_admin_identity() else {
        return;
    };
    let Ok(mut conn) = pool.get() else {
        warn!("bootstrap admin: database unavailable; users row not reconciled");
        return;
    };

    let user_id = identity.user_id.unwrap_or_else(|| {
        uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("zitadel:{}", identity.email).as_bytes(),
        )
    });
    let username = identity
        .email
        .split('@')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("admin");

    let updated = diesel::sql_query(
        "UPDATE users SET is_admin = TRUE, is_active = TRUE, updated_at = NOW() \
         WHERE id = $1 OR lower(email) = lower($2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(&identity.email)
    .execute(&mut conn);

    match updated {
        Ok(count) if count > 0 => {
            info!(
                "bootstrap admin: users row for {} reconciled with is_admin=true",
                identity.email
            );
        }
        Ok(_) => {
            let inserted = diesel::sql_query(
                "INSERT INTO users \
                   (id, username, email, password_hash, is_active, is_admin, created_at, updated_at) \
                 VALUES ($1, $2, $3, '', TRUE, TRUE, NOW(), NOW()) \
                 ON CONFLICT DO NOTHING",
            )
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Text, _>(username)
            .bind::<diesel::sql_types::Text, _>(&identity.email)
            .execute(&mut conn);

            match inserted {
                Ok(1) => info!(
                    "bootstrap admin: created users row for {} with is_admin=true",
                    identity.email
                ),
                Ok(_) => warn!(
                    "bootstrap admin: {} already has a conflicting users row",
                    identity.email
                ),
                Err(e) => warn!(
                    "bootstrap admin: could not create users row for {}: {e}",
                    identity.email
                ),
            }
        }
        Err(e) => warn!(
            "bootstrap admin: could not reconcile users row for {}: {e}",
            identity.email
        ),
    }
}

#[cfg(feature = "directory")]
/// `DirectoryConfig` serializes both the canonical and legacy keys for some
/// fields (`service_account_key` + `service_token`). `ZitadelConfig` maps the
/// legacy key onto the canonical one with `#[serde(alias)]`, and serde rejects a
/// document that carries a field *and* its alias as a duplicate. Drop the
/// redundant key, preferring the non-empty value, so a well-formed on-disk
/// config is never discarded in favour of a possibly-stale Vault entry.
fn normalize_directory_config(value: &mut serde_json::Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    if obj.contains_key("service_account_key") && obj.contains_key("service_token") {
        let canonical_empty = obj
            .get("service_account_key")
            .map(|v| v.is_null() || v.as_str().map(|s| s.trim().is_empty()).unwrap_or(false))
            .unwrap_or(true);
        if canonical_empty {
            obj.remove("service_account_key");
        } else {
            obj.remove("service_token");
        }
    }
}

#[cfg(feature = "directory")]
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
        if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&content) {
            normalize_directory_config(&mut value);
            if let Ok(mut config) = serde_json::from_value::<crate::directory::ZitadelConfig>(value) {
                // Older configs stored the endpoint only as `base_url`.
                if config.api_url.trim().is_empty() && !config.base_url.trim().is_empty() {
                    config.api_url = config.base_url.clone();
                }
                if !config.api_url.is_empty() {
                    info!(
                        "Loaded Zitadel config from {}: url={}",
                        config_path, config.api_url
                    );
                    return config;
                }
            }
        }
        info!("directory_config.json missing or invalid, trying Vault");
    }

    match load_directory_config_from_vault().await {
        Ok(config) => {
            info!(
                "Loaded Zitadel config from Vault (secret/gbo/directory): url={}",
                config.api_url.split('?').next().unwrap_or("")
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

#[cfg(feature = "directory")]
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
        base_url: String::new(),
        service_account_key: secrets.get("service_account_key").cloned(),
        service_token: secrets.get("service_token").cloned(),
        allow_insecure_http: false,
    })
}

#[cfg(feature = "directory")]
pub(crate) async fn init_directory_service() -> Result<
    (
        Arc<Mutex<crate::directory::AuthService>>,
        crate::directory::ZitadelConfig,
    ),
    std::io::Error,
> {
    let zitadel_config = resolve_directory_config().await;

    let auth_service = Arc::new(tokio::sync::Mutex::new(
        crate::directory::AuthService::new(zitadel_config.clone())
            .map_err(|e| std::io::Error::other(format!("Failed to create auth service: {}", e)))?,
    ));

    Ok((auth_service, zitadel_config))
}

#[cfg(feature = "directory")]
pub(crate) fn default_zitadel_config() -> crate::directory::ZitadelConfig {
    crate::directory::ZitadelConfig {
        issuer_url: "".to_string(),
        issuer: "".to_string(),
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uri: "/callback".to_string(),
        project_id: "default".to_string(),
        api_url: "".to_string(),
        base_url: String::new(),
        service_account_key: None,
        service_token: None,
        allow_insecure_http: false,
    }
}

#[cfg(feature = "directory")]
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
                ZitadelClient::with_pat_token(zitadel_config.clone(), pat_token).map_err(|e| {
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
        ZitadelClient::new(zitadel_config.clone())
            .map_err(|e| std::io::Error::other(format!("Failed to create bootstrap client: {}", e)))
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

    // The PAT client is already authenticated as an instance manager. Rebuilding
    // an OAuth2 client here discarded that credential and every management call
    // failed with "No access token" (the client-credentials flow is not
    // configured for the BotServer OIDC app).
    if let Err(e) = bootstrap::ensure_default_organization(&bootstrap_client).await {
        info!(
            "Failed to ensure default organization (non-critical): {}",
            e
        );
    }

    if let Err(e) = bootstrap::ensure_admin_user(&bootstrap_client).await {
        info!("Failed to ensure admin user (non-critical): {}", e);
    }
}
