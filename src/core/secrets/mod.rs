use crate::core::shared::utils::get_stack_path;
use anyhow::{anyhow, Result};
use diesel::PgConnection;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Arc as StdArc;
use std::sync::OnceLock;
use std::sync::RwLock as StdRwLock;
use uuid::Uuid;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;

#[derive(Debug)]
pub struct SecretPaths;

impl SecretPaths {
    // System-wide paths (global fallback)
    pub const DIRECTORY: &'static str = "gbo/directory";
    pub const TABLES: &'static str = "gbo/tables";
    pub const DRIVE: &'static str = "gbo/drive";
    pub const CACHE: &'static str = "gbo/cache";
    pub const EMAIL: &'static str = "gbo/email";
    pub const LLM: &'static str = "gbo/llm";
    pub const ENCRYPTION: &'static str = "gbo/encryption";
    pub const JWT: &'static str = "gbo/jwt";
    pub const MEET: &'static str = "gbo/meet";
    pub const ALM: &'static str = "gbo/alm";
    pub const VECTORDB: &'static str = "gbo/vectordb";
    pub const OBSERVABILITY: &'static str = "gbo/system/observability";
    pub const SECURITY: &'static str = "gbo/system/security";
    pub const CLOUD: &'static str = "gbo/system/cloud";
    pub const APP: &'static str = "gbo/system/app";
    pub const MODELS: &'static str = "gbo/system/models";

    // Tenant infrastructure (per-cluster)
    pub fn tenant_infrastructure(tenant: &str) -> String {
        format!("gbo/tenants/{}/infrastructure", tenant)
    }
    pub fn tenant_config(tenant: &str) -> String {
        format!("gbo/tenants/{}/config", tenant)
    }

    // Organization (per-customer)
    pub fn org_bot(org_id: &str, bot_id: &str) -> String {
        format!("gbo/orgs/{}/bots/{}", org_id, bot_id)
    }
    pub fn org_user(org_id: &str, user_id: &str) -> String {
        format!("gbo/orgs/{}/users/{}", org_id, user_id)
    }
    pub fn org_config(org_id: &str) -> String {
        format!("gbo/orgs/{}/config", org_id)
    }
}

struct CachedSecret {
    data: HashMap<String, String>,
    expires_at: std::time::Instant,
}

#[derive(Clone)]
pub struct SecretsManager {
    client: Option<StdArc<VaultClient>>,
    cache: Arc<std::sync::RwLock<HashMap<String, CachedSecret>>>,
    cache_ttl: u64,
    enabled: bool,
}

impl std::fmt::Debug for SecretsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretsManager")
            .field("client", &self.client.is_some())
            .field("cache", &"<RwLock<HashMap>>")
            .field("cache_ttl", &self.cache_ttl)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl SecretsManager {
    pub fn from_env() -> Result<Self> {
        let addr = env::var("VAULT_ADDR").unwrap_or_default();
        let token = env::var("VAULT_TOKEN").unwrap_or_default();
        let skip_verify = env::var("VAULT_SKIP_VERIFY")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let cache_ttl = env::var("VAULT_CACHE_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        let stack_path = get_stack_path();
        let ca_cert = env::var("VAULT_CACERT")
            .unwrap_or_else(|_| format!("{}/conf/system/certificates/ca/ca.crt", stack_path));
        let client_cert = env::var("VAULT_CLIENT_CERT").unwrap_or_else(|_| {
            format!("{}/conf/system/certificates/botserver/client.crt", stack_path)
        });
        let client_key = env::var("VAULT_CLIENT_KEY").unwrap_or_else(|_| {
            format!("{}/conf/system/certificates/botserver/client.key", stack_path)
        });

        let enabled = !token.is_empty() && !addr.is_empty();

        if !enabled {
            warn!("Vault not configured. Using environment variables directly.");
            return Ok(Self {
                client: None,
                cache: Arc::new(StdRwLock::new(HashMap::new())),
                cache_ttl,
                enabled: false,
            });
        }

        let ca_path = PathBuf::from(&ca_cert);
        let cert_path = PathBuf::from(&client_cert);
        let key_path = PathBuf::from(&client_key);

        let mut settings_builder = VaultClientSettingsBuilder::default();
        settings_builder.address(&addr).token(&token);

        // Only warn about TLS verification for HTTPS connections
        let is_https = addr.starts_with("https://");
        if skip_verify {
            if is_https {
                warn!("TLS verification disabled - NOT RECOMMENDED FOR PRODUCTION");
            }
            settings_builder.verify(false);
        } else {
            settings_builder.verify(true);

            if ca_path.exists() {
                debug!("Using CA certificate for Vault: {}", ca_cert);
                settings_builder.ca_certs(vec![ca_cert]);
            }
        }

        if cert_path.exists() && key_path.exists() && !skip_verify {
            debug!("Using mTLS client certificate for Vault: {}", client_cert);
        }

        let settings = settings_builder.build()?;
        let client = VaultClient::new(settings)?;

        debug!("Vault client initialized with TLS: {}", addr);

        Ok(Self {
            client: Some(StdArc::new(client)),
            cache: Arc::new(StdRwLock::new(HashMap::new())),
            cache_ttl,
            enabled: true,
        })
    }

    pub fn get() -> Result<&'static SecretsManager> {
        static INIT: OnceLock<Result<SecretsManager>> = OnceLock::new();
        INIT.get_or_init(|| {
            match Self::from_env() {
                Ok(manager) => {
                    info!("SecretsManager singleton initialized (Vault: {})", env::var("VAULT_ADDR").unwrap_or_default());
                    Ok(manager)
                }
                Err(e) => {
                    warn!("Failed to initialize SecretsManager: {}", e);
                    Err(e)
                }
            }
        }).as_ref().map_err(|e| anyhow!("SecretsManager initialization failed: {}", e))
    }

    pub fn get_clone() -> Result<SecretsManager> {
        Self::get().map(|sm| sm.clone())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>> {
        if !self.enabled {
            return Self::get_from_env(path);
        }

        if let Some(cached) = self.get_cached(path).await {
            return Ok(cached);
        }

        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("No Vault client"))?;

        let result: Result<HashMap<String, String>, _> =
            kv2::read(client.as_ref(), "secret", path).await;

        let data = match result {
            Ok(d) => d,
            Err(e) => {
                debug!(
                    "Vault read failed for '{}': {}, falling back to env",
                    path, e
                );
                return Self::get_from_env(path);
            }
        };

        if self.cache_ttl > 0 {
            self.cache_secret(path, data.clone()).await;
        }

        Ok(data)
    }

    pub async fn get_value(&self, path: &str, key: &str) -> Result<String> {
        self.get_secret(path)
            .await?
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("Key '{}' not found in '{}'", key, path))
    }

    pub fn get_value_blocking(&self, path: &str, key: &str, default: &str) -> String {
        if let Ok(secrets) = Self::get_from_env(path) {
            if let Some(value) = secrets.get(key) {
                return value.clone();
            }
        }
        default.to_string()
    }

    pub fn get_drive_config(&self) -> Result<(String, String, String)> {
        // Try to read from Vault using std process (curl)
        if let Ok(vault_addr) = std::env::var("VAULT_ADDR") {
            if let Ok(vault_token) = std::env::var("VAULT_TOKEN") {
                let ca_cert = std::env::var("VAULT_CACERT").unwrap_or_default();

                log::info!("Attempting to read drive config from Vault: {}", vault_addr);

                let url = format!("{}/v1/secret/data/gbo/drive", vault_addr);

                // Use curl via Command for reliable TLS
                let result = std::process::Command::new("curl")
                    .args(&["-sf", "--cacert", &ca_cert, "-H", &format!("X-Vault-Token: {}", &vault_token), &url])
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                            if let Some(secret_data) = data.get("data").and_then(|d| d.get("data")) {
                                let host = secret_data.get("host").and_then(|v| v.as_str())
                                    .ok_or_else(|| anyhow!("drive host not configured in Vault"))?;
                                let accesskey = secret_data.get("accesskey").and_then(|v| v.as_str())
                                    .ok_or_else(|| anyhow!("drive accesskey not configured in Vault"))?;
                                let secret = secret_data.get("secret").and_then(|v| v.as_str())
                                    .ok_or_else(|| anyhow!("drive secret not configured in Vault"))?;
                                log::info!("get_drive_config: Successfully read from Vault - host={}", host);
                                return Ok((host.to_string(), accesskey.to_string(), secret.to_string()));
                            }
                        }
                    }
                    Ok(output) => {
                        log::error!("curl failed: {}", String::from_utf8_lossy(&output.stderr));
                    }
                    Err(e) => {
                        log::error!("Failed to run curl: {}", e);
                    }
                }
            }
        }

        Err(anyhow!("Drive configuration not available in Vault and VAULT_ADDR/VAULT_TOKEN not set"))
    }
    
    pub fn get_cache_config(&self) -> Result<(String, u16, Option<String>)> {
        if let Ok(vault_addr) = std::env::var("VAULT_ADDR") {
            if let Ok(vault_token) = std::env::var("VAULT_TOKEN") {
                let ca_cert = std::env::var("VAULT_CACERT").unwrap_or_default();

                log::info!("Attempting to read cache config from Vault: {}", vault_addr);
                let url = format!("{}/v1/secret/data/gbo/cache", vault_addr);

                let result = std::process::Command::new("curl")
                    .args(&["-sf", "--cacert", &ca_cert, "-H", &format!("X-Vault-Token: {}", &vault_token), &url])
                    .output();

                if let Ok(output) = result {
                    if output.status.success() {
                        if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                            if let Some(secret_data) = data.get("data").and_then(|d| d.get("data")) {
                                let host = secret_data.get("host").and_then(|v| v.as_str())
                                    .ok_or_else(|| anyhow!("cache host not configured in Vault"))?;
                                let port = secret_data.get("port").and_then(|v| v.as_str())
                                    .ok_or_else(|| anyhow!("cache port not configured in Vault"))?
                                    .parse().map_err(|e| anyhow!("Invalid cache port: {}", e))?;
                                let password = secret_data.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());
                                log::info!("get_cache_config: Successfully read from Vault - host={}", host);
                                return Ok((host.to_string(), port, password));
                            }
                        }
                    }
                }
            }
        }
        Err(anyhow!("Cache configuration not available in Vault and VAULT_ADDR/VAULT_TOKEN not set"))
    }
    
    pub fn get_qdrant_config(&self) -> Result<(String, Option<String>)> {
        if let Ok(vault_addr) = std::env::var("VAULT_ADDR") {
            if let Ok(vault_token) = std::env::var("VAULT_TOKEN") {
                let ca_cert = std::env::var("VAULT_CACERT").unwrap_or_default();

                log::info!("Attempting to read qdrant config from Vault: {}", vault_addr);
                let url = format!("{}/v1/secret/data/gbo/vectordb", vault_addr);

                let result = std::process::Command::new("curl")
                    .args(&["-sf", "--cacert", &ca_cert, "-H", &format!("X-Vault-Token: {}", &vault_token), &url])
                    .output();

                if let Ok(output) = result {
                    if output.status.success() {
                        if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                            if let Some(secret_data) = data.get("data").and_then(|d| d.get("data")) {
                                let url = secret_data.get("url").and_then(|v| v.as_str())
                                    .ok_or_else(|| anyhow!("vectordb url not configured in Vault"))?;
                                let api_key = secret_data.get("api_key").and_then(|v| v.as_str()).map(|s| s.to_string());
                                log::info!("get_qdrant_config: Successfully read from Vault - url={}", url);
                                return Ok((url.to_string(), api_key));
                            }
                        }
                    }
                }
            }
        }
        Err(anyhow!("VectorDB configuration not available in Vault and VAULT_ADDR/VAULT_TOKEN not set"))
    }

    pub fn get_database_config_sync(&self) -> Result<(String, u16, String, String, String)> {
        if let Ok(secrets) = Self::get_from_env(SecretPaths::TABLES) {
            let host = secrets.get("host").cloned()
                .ok_or_else(|| anyhow!("database host not configured"))?;
            let port = secrets.get("port").and_then(|p| p.parse().ok())
                .ok_or_else(|| anyhow!("database port not configured"))?;
            let database = secrets.get("database").cloned()
                .ok_or_else(|| anyhow!("database name not configured"))?;
            let username = secrets.get("username").cloned()
                .ok_or_else(|| anyhow!("database username not configured"))?;
            let password = secrets.get("password").cloned().unwrap_or_default();
            return Ok((host, port, database, username, password));
        }
        Err(anyhow!("Database configuration not available"))
    }

    pub async fn get_drive_credentials(&self) -> Result<(String, String)> {
        let s = self.get_secret(SecretPaths::DRIVE).await?;
        Ok((
            s.get("accesskey").cloned().unwrap_or_default(),
            s.get("secret").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_database_config(&self) -> Result<(String, u16, String, String, String)> {
        let s = self.get_secret(SecretPaths::TABLES).await?;
        let host = s.get("host").cloned()
            .ok_or_else(|| anyhow!("database host not configured in Vault"))?;
        let port = s.get("port").and_then(|p| p.parse().ok())
            .ok_or_else(|| anyhow!("database port not configured in Vault"))?;
        let database = s.get("database").cloned()
            .ok_or_else(|| anyhow!("database name not configured in Vault"))?;
        let username = s.get("username").cloned()
            .ok_or_else(|| anyhow!("database username not configured in Vault"))?;
        let password = s.get("password").cloned().unwrap_or_default();
        Ok((host, port, database, username, password))
    }

    pub async fn get_database_url(&self) -> Result<String> {
        let (host, port, db, user, pass) = self.get_database_config().await?;
        Ok(format!(
            "postgres://{}:{}@{}:{}/{}",
            user, pass, host, port, db
        ))
    }

    pub async fn get_database_credentials(&self) -> Result<(String, String)> {
        let s = self.get_secret(SecretPaths::TABLES).await?;
        Ok((
            s.get("username")
                .cloned()
                .unwrap_or_else(|| "gbuser".into()),
            s.get("password").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_cache_password(&self) -> Result<Option<String>> {
        Ok(self
            .get_secret(SecretPaths::CACHE)
            .await?
            .get("password")
            .cloned())
    }

    pub async fn get_directory_config(&self) -> Result<(String, String, String, String)> {
        let s = self.get_secret(SecretPaths::DIRECTORY).await?;
        let url = s.get("url").cloned()
            .ok_or_else(|| anyhow!("directory url not configured in Vault"))?;
        Ok((
            url,
            s.get("project_id").cloned().unwrap_or_default(),
            s.get("client_id").cloned().unwrap_or_default(),
            s.get("client_secret").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_directory_credentials(&self) -> Result<(String, String)> {
        let s = self.get_secret(SecretPaths::DIRECTORY).await?;
        Ok((
            s.get("client_id").cloned().unwrap_or_default(),
            s.get("client_secret").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_vectordb_config(&self) -> Result<(String, Option<String>)> {
        let s = self.get_secret(SecretPaths::VECTORDB).await?;
        let url = s.get("url").cloned()
            .ok_or_else(|| anyhow!("vectordb url not configured in Vault"))?;
        Ok((url, s.get("api_key").cloned()))
    }

    pub async fn get_observability_config(&self) -> Result<(String, String, String, String)> {
        let s = self.get_secret(SecretPaths::OBSERVABILITY).await?;
        let url = s.get("url").cloned()
            .ok_or_else(|| anyhow!("observability url not configured in Vault"))?;
        Ok((
            url,
            s.get("org").cloned().unwrap_or_else(|| "system".into()),
            s.get("bucket").cloned().unwrap_or_else(|| "metrics".into()),
            s.get("token").cloned().unwrap_or_default(),
        ))
    }

    pub async fn get_llm_api_key(&self, provider: &str) -> Result<Option<String>> {
        let s = self.get_secret(SecretPaths::LLM).await?;
        Ok(s.get(&format!("{}_key", provider.to_lowercase())).cloned())
    }

    pub async fn get_encryption_key(&self) -> Result<String> {
        self.get_value(SecretPaths::ENCRYPTION, "master_key").await
    }

    pub fn get_directory_config_sync(&self) -> (String, String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    self_owned.get_secret(SecretPaths::DIRECTORY).await.ok()
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_else(|| "".into()),
                secrets.get("project_id").cloned().unwrap_or_default(),
                secrets.get("client_id").cloned().unwrap_or_default(),
                secrets.get("client_secret").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), String::new(), String::new(), String::new())
    }

    pub fn get_email_config(&self) -> (String, u16, String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    self_owned.get_secret(SecretPaths::EMAIL).await.ok()
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("smtp_host").cloned().unwrap_or_default(),
                secrets.get("smtp_port").and_then(|p| p.parse().ok()).unwrap_or(587),
                secrets.get("smtp_user").cloned().unwrap_or_default(),
                secrets.get("smtp_password").cloned().unwrap_or_default(),
                secrets.get("smtp_from").cloned().unwrap_or_default(),
            );
        }
        (String::new(), 587, String::new(), String::new(), String::new())
    }

    pub fn get_llm_config(&self) -> (String, String, Option<String>, Option<String>, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    self_owned.get_secret(SecretPaths::LLM).await.ok()
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_else(|| "".into()),
                secrets.get("model").cloned().unwrap_or_else(|| "gpt-4".into()),
                secrets.get("openai_key").cloned(),
                secrets.get("anthropic_key").cloned(),
                secrets.get("ollama_url").cloned().unwrap_or_else(|| "".into()),
            );
        }
        ("".to_string(), "gpt-4".to_string(), None, None, "".to_string())
    }

    pub fn get_meet_config(&self) -> (String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    self_owned.get_secret(SecretPaths::MEET).await.ok()
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_else(|| "".into()),
                secrets.get("app_id").cloned().unwrap_or_default(),
                secrets.get("app_secret").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), String::new(), String::new())
    }

    pub fn get_vectordb_config_sync(&self) -> (String, Option<String>) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    self_owned.get_secret(SecretPaths::VECTORDB).await.ok()
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_else(|| "".into()),
                secrets.get("api_key").cloned(),
            );
        }
        ("".to_string(), None)
    }

    pub fn get_observability_config_sync(&self) -> (String, String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    self_owned.get_secret(SecretPaths::OBSERVABILITY).await.ok()
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_else(|| "".into()),
                secrets.get("org").cloned().unwrap_or_else(|| "system".into()),
                secrets.get("bucket").cloned().unwrap_or_else(|| "metrics".into()),
                secrets.get("token").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), "system".to_string(), "metrics".to_string(), String::new())
    }

    pub fn get_alm_config(&self) -> (String, String, String) {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    self_owned.get_secret(SecretPaths::ALM).await.ok()
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return (
                secrets.get("url").cloned().unwrap_or_else(|| "".into()),
                secrets.get("token").cloned().unwrap_or_default(),
                secrets.get("default_org").cloned().unwrap_or_default(),
            );
        }
        ("".to_string(), String::new(), String::new())
    }

    pub fn get_jwt_secret_sync(&self) -> String {
        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    self_owned.get_secret(SecretPaths::JWT).await.ok()
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });
        if let Ok(Some(secrets)) = rx.recv() {
            return secrets.get("secret").cloned().unwrap_or_default();
        }
        String::new()
    }

    pub async fn put_secret(&self, path: &str, data: HashMap<String, String>) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Vault not enabled"))?;
        kv2::set(client.as_ref(), "secret", path, &data).await?;
        self.invalidate_cache(path).await;
        info!("Secret stored at '{}'", path);
        Ok(())
    }

    pub async fn delete_secret(&self, path: &str) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Vault not enabled"))?;
        kv2::delete_latest(client.as_ref(), "secret", path).await?;
        self.invalidate_cache(path).await;
        info!("Secret deleted at '{}'", path);
        Ok(())
    }

    pub async fn health_check(&self) -> Result<bool> {
        if let Some(client) = &self.client {
            Ok(vaultrs::sys::health(client.as_ref()).await.is_ok())
        } else {
            Ok(false)
        }
    }

    pub async fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    async fn get_cached(&self, path: &str) -> Option<HashMap<String, String>> {
        let cache = self.cache.read().ok()?;
        cache
            .get(path)
            .and_then(|c| (c.expires_at > std::time::Instant::now()).then(|| c.data.clone()))
    }

    async fn cache_secret(&self, path: &str, data: HashMap<String, String>) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(
                path.to_string(),
                CachedSecret {
                    data,
                    expires_at: std::time::Instant::now()
                        + std::time::Duration::from_secs(self.cache_ttl),
                },
            );
        }
    }

    async fn invalidate_cache(&self, path: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(path);
        }
    }

    fn get_from_env(path: &str) -> Result<HashMap<String, String>> {
        let mut secrets = HashMap::new();

        // Only Vault-related env vars are allowed; all other secrets must come from Vault itself
        let normalized = if path.starts_with("gbo/system/") {
            path.strip_prefix("gbo/system/").unwrap_or(path)
        } else {
            path
        };

        match normalized {
            "tables" | "gbo/tables" | "system/tables" => {
                secrets.insert("host".into(), "localhost".into());
                secrets.insert("port".into(), "5432".into());
                secrets.insert("database".into(), "botserver".into());
                secrets.insert("username".into(), "gbuser".into());
                secrets.insert("password".into(), "changeme".into());
            }
        "directory" | "gbo/directory" | "system/directory" => {
            secrets.insert("url".into(), "".into());
            secrets.insert("host".into(), "localhost".into());
            secrets.insert("port".into(), "9000".into());
            secrets.insert("project_id".into(), String::new());
            secrets.insert("client_id".into(), String::new());
            secrets.insert("client_secret".into(), String::new());
        }
            "drive" | "gbo/drive" | "system/drive" => {
                secrets.insert("host".into(), "localhost".into());
                secrets.insert("port".into(), "9000".into());
                secrets.insert("accesskey".into(), "minioadmin".into());
                secrets.insert("secret".into(), "minioadmin".into());
            }
            "cache" | "gbo/cache" | "system/cache" => {
                secrets.insert("host".into(), "localhost".into());
                secrets.insert("port".into(), "6379".into());
                secrets.insert("password".into(), String::new());
            }
            "email" | "gbo/email" | "system/email" => {
                secrets.insert("smtp_host".into(), String::new());
                secrets.insert("smtp_port".into(), "587".into());
                secrets.insert("smtp_user".into(), String::new());
                secrets.insert("smtp_password".into(), String::new());
                secrets.insert("smtp_from".into(), String::new());
            }
            "llm" | "gbo/llm" | "system/llm" => {
                secrets.insert("url".into(), "".into());
                secrets.insert("model".into(), "gpt-4".into());
                secrets.insert("openai_key".into(), String::new());
                secrets.insert("anthropic_key".into(), String::new());
                secrets.insert("ollama_url".into(), "".into());
            }
            "encryption" | "gbo/encryption" | "system/encryption" => {
                secrets.insert("master_key".into(), String::new());
            }
            "meet" | "gbo/meet" | "system/meet" => {
                secrets.insert("url".into(), "".into());
                secrets.insert("app_id".into(), String::new());
                secrets.insert("app_secret".into(), String::new());
            }
            "vectordb" | "gbo/vectordb" | "system/vectordb" => {
                secrets.insert("url".to_string(), "".into());
                secrets.insert("host".to_string(), "localhost".into());
                secrets.insert("port".to_string(), "6333".into());
                secrets.insert("grpc_port".to_string(), "6334".into());
                secrets.insert("api_key".to_string(), String::new());
            }
            "observability" | "gbo/observability" | "system/observability" => {
                secrets.insert("url".into(), "".into());
                secrets.insert("org".into(), "system".into());
                secrets.insert("bucket".into(), "metrics".into());
                secrets.insert("token".into(), String::new());
            }
            "alm" | "gbo/alm" | "system/alm" => {
                secrets.insert("url".into(), "".into());
                secrets.insert("token".into(), String::new());
                secrets.insert("default_org".into(), String::new());
            }
            "security" | "gbo/security" | "system/security" => {
                secrets.insert("require_auth".into(), "true".into());
                secrets.insert("anonymous_paths".into(), String::new());
            }
            "cloud" | "gbo/cloud" | "system/cloud" => {
                secrets.insert("region".into(), "us-east-1".into());
                secrets.insert("access_key".into(), String::new());
                secrets.insert("secret_key".into(), String::new());
            }
            "app" | "gbo/app" | "system/app" => {
                secrets.insert("url".into(), "".into());
                secrets.insert("environment".into(), "development".into());
            }
            "jwt" | "gbo/jwt" | "system/jwt" => {
                secrets.insert("secret".into(), String::new());
            }
            "models" | "gbo/models" | "system/models" => {
                secrets.insert("url".into(), "".into());
            }
            _ => {
                log::debug!("No default values for secret path: {}", path);
            }
        }

        Ok(secrets)
    }

    // ============ TENANT INFRASTRUCTURE ============
    
    /// Get database config with tenant fallback to system
    pub async fn get_database_config_for_tenant(&self, tenant: &str) -> Result<(String, u16, String, String, String)> {
        // Try tenant first
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/tables", tenant_path)).await {
            return Ok((
                s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
                s.get("port").and_then(|p| p.parse().ok()).unwrap_or(5432),
                s.get("database").cloned().unwrap_or_else(|| "botserver".into()),
                s.get("username").cloned().unwrap_or_else(|| "gbuser".into()),
                s.get("password").cloned().unwrap_or_default(),
            ));
        }
        // Fallback to system
        self.get_database_config().await
    }

    /// Get drive config with tenant fallback to system
    pub async fn get_drive_config_for_tenant(&self, tenant: &str) -> Result<(String, String, String, String)> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/drive", tenant_path)).await {
            return Ok((
                s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
                s.get("port").cloned().unwrap_or_else(|| "9000".into()),
                s.get("accesskey").cloned().unwrap_or_default(),
                s.get("secret").cloned().unwrap_or_default(),
            ));
        }
        let s = self.get_secret(SecretPaths::DRIVE).await?;
        Ok((
            s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
            s.get("port").cloned().unwrap_or_else(|| "9000".into()),
            s.get("accesskey").cloned().unwrap_or_default(),
            s.get("secret").cloned().unwrap_or_default(),
        ))
    }

    /// Get cache config with tenant fallback to system
    pub async fn get_cache_config_for_tenant(&self, tenant: &str) -> Result<(String, u16, Option<String>)> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/cache", tenant_path)).await {
            return Ok((
                s.get("host").cloned().unwrap_or_else(|| "localhost".into()),
                s.get("port").and_then(|p| p.parse().ok()).unwrap_or(6379),
                s.get("password").cloned(),
            ));
        }
        let url = self.get_secret(SecretPaths::CACHE).await?
            .get("url").cloned();
        let host = url.as_ref().map(|u| u.split("://").nth(1).unwrap_or("localhost").split(':').next().unwrap_or("localhost")).unwrap_or("localhost").to_string();
        let port = url.as_ref().and_then(|u| u.split(':').nth(1)).and_then(|p| p.parse().ok()).unwrap_or(6379);
        Ok((host, port, None))
    }

    /// Get SMTP config with tenant fallback to system
    pub async fn get_smtp_config_for_tenant(&self, tenant: &str) -> Result<HashMap<String, String>> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/email", tenant_path)).await {
            return Ok(s);
        }
        self.get_secret(SecretPaths::EMAIL).await
    }

    /// Get LLM config with tenant fallback to system
    pub async fn get_llm_config_for_tenant(&self, tenant: &str) -> Result<HashMap<String, String>> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/llm", tenant_path)).await {
            return Ok(s);
        }
        self.get_secret(SecretPaths::LLM).await
    }

    /// Get directory (Zitadel) config with tenant fallback to system
    pub async fn get_directory_config_for_tenant(&self, tenant: &str) -> Result<HashMap<String, String>> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/directory", tenant_path)).await {
            return Ok(s);
        }
        self.get_secret(SecretPaths::DIRECTORY).await
    }

    /// Get models config with tenant fallback to system
    pub async fn get_models_config_for_tenant(&self, tenant: &str) -> Result<HashMap<String, String>> {
        let tenant_path = SecretPaths::tenant_infrastructure(tenant);
        if let Ok(s) = self.get_secret(&format!("{}/models", tenant_path)).await {
            return Ok(s);
        }
        self.get_secret(SecretPaths::MODELS).await
    }

    // ============ ORG BOT/USER SECRETS ============

    /// Get bot email credentials
    pub async fn get_bot_email_config(&self, org_id: &str, bot_id: &str) -> Result<HashMap<String, String>> {
        let path = SecretPaths::org_bot(org_id, bot_id);
        if let Ok(s) = self.get_secret(&format!("{}/email", path)).await {
            return Ok(s);
        }
        // Fallback to system email
        self.get_secret(SecretPaths::EMAIL).await
    }

    /// Get bot WhatsApp credentials
    pub async fn get_bot_whatsapp_config(&self, org_id: &str, bot_id: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_bot(org_id, bot_id);
        Ok(self.get_secret(&format!("{}/whatsapp", path)).await.ok())
    }

    /// Get bot LLM config (overrides tenant/system)
    pub async fn get_bot_llm_config(&self, org_id: &str, bot_id: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_bot(org_id, bot_id);
        Ok(self.get_secret(&format!("{}/llm", path)).await.ok())
    }

    /// Get bot API keys (openai, anthropic, custom)
    pub async fn get_bot_api_keys_config(&self, org_id: &str, bot_id: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_bot(org_id, bot_id);
        Ok(self.get_secret(&format!("{}/api-keys", path)).await.ok())
    }

    /// Get user email credentials
    pub async fn get_user_email_config(&self, org_id: &str, user_id: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_user(org_id, user_id);
        Ok(self.get_secret(&format!("{}/email", path)).await.ok())
    }

    /// Get user OAuth credentials
    pub async fn get_user_oauth_config(&self, org_id: &str, user_id: &str, provider: &str) -> Result<Option<HashMap<String, String>>> {
        let path = SecretPaths::org_user(org_id, user_id);
        Ok(self.get_secret(&format!("{}/oauth/{}", path, provider)).await.ok())
    }

    // ============ BOT EMAIL RESOLUTION (bot → default bot → system) ============

    /// Get email config for a specific bot with inheritance chain:
    /// 1. Bot-specific: `gbo/bots/{bot_id}/email`
    /// 2. Default bot: `gbo/bots/default/email`
    /// 3. System-wide: `gbo/email`
    pub fn get_email_config_for_bot_sync(&self, bot_id: &Uuid) -> (String, u16, String, String, String) {
        let bot_path = format!("gbo/bots/{}/email", bot_id);
        let default_path = "gbo/bots/default/email".to_string();

        let paths = vec![bot_path, default_path, SecretPaths::EMAIL.to_string()];

        let self_owned = self.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    for path in paths {
                        if let Ok(secrets) = self_owned.get_secret(&path).await {
                            if !secrets.is_empty() && secrets.contains_key("smtp_from") {
                                return Some((
                                    secrets.get("smtp_host").cloned().unwrap_or_default(),
                                    secrets.get("smtp_port").and_then(|p| p.parse().ok()).unwrap_or(587),
                                    secrets.get("smtp_user").cloned().unwrap_or_default(),
                                    secrets.get("smtp_password").cloned().unwrap_or_default(),
                                    secrets.get("smtp_from").cloned().unwrap_or_default(),
                                ));
                            }
                        }
                    }
                    None
                })
            } else {
                None
            };
            let _ = tx.send(result);
        });

        if let Ok(Some(config)) = rx.recv() {
            return config;
        }

        (String::new(), 587, String::new(), String::new(), String::new())
    }



    // ============ TENANT-AWARE METHODS (org_id -> tenant -> secrets) ============

    /// Get database config for an organization (resolves tenant from org, then gets infra)
    pub async fn get_database_config_for_org(&self, conn: &mut PgConnection, org_id: Uuid) -> Result<(String, u16, String, String, String)> {
        let tenant_id = self.get_tenant_id_for_org(conn, org_id)?;
        self.get_database_config_for_tenant(&tenant_id).await
    }

    /// Get drive config for an organization
    pub async fn get_drive_config_for_org(&self, conn: &mut PgConnection, org_id: Uuid) -> Result<(String, String, String, String)> {
        let tenant_id = self.get_tenant_id_for_org(conn, org_id)?;
        self.get_drive_config_for_tenant(&tenant_id).await
    }

    /// Get cache config for an organization
    pub async fn get_cache_config_for_org(&self, conn: &mut PgConnection, org_id: Uuid) -> Result<(String, u16, Option<String>)> {
        let tenant_id = self.get_tenant_id_for_org(conn, org_id)?;
        self.get_cache_config_for_tenant(&tenant_id).await
    }

    /// Get SMTP config for an organization
    pub async fn get_smtp_config_for_org(&self, conn: &mut PgConnection, org_id: Uuid) -> Result<HashMap<String, String>> {
        let tenant_id = self.get_tenant_id_for_org(conn, org_id)?;
        self.get_smtp_config_for_tenant(&tenant_id).await
    }

    /// Get LLM config for an organization
    pub async fn get_llm_config_for_org(&self, conn: &mut PgConnection, org_id: Uuid) -> Result<HashMap<String, String>> {
        let tenant_id = self.get_tenant_id_for_org(conn, org_id)?;
        self.get_llm_config_for_tenant(&tenant_id).await
    }

    /// Get tenant_id for an organization from database
    pub fn get_tenant_id_for_org(&self, conn: &mut PgConnection, org_id: Uuid) -> Result<String> {
        use diesel::prelude::*;
        use crate::core::shared::schema::organizations;

        let result: Option<Uuid> = organizations::table
            .filter(organizations::org_id.eq(org_id))
            .select(organizations::tenant_id)
            .first::<Uuid>(conn)
            .ok();

        Ok(result.map(|t| t.to_string()).unwrap_or_else(|| "default".to_string()))
    }
}

pub fn init_secrets_manager() -> Result<SecretsManager> {
    SecretsManager::from_env()
}

#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    pub vault_addr: String,
    pub vault_token: String,
}

impl BootstrapConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            vault_addr: env::var("VAULT_ADDR")?,
            vault_token: env::var("VAULT_TOKEN")?,
        })
    }

    pub fn is_configured() -> bool {
        env::var("VAULT_ADDR").is_ok() && env::var("VAULT_TOKEN").is_ok()
    }
}
