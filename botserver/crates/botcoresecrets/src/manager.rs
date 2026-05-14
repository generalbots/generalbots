use crate::env_defaults::get_from_env;
use anyhow::{anyhow, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Arc as StdArc;
use std::sync::OnceLock;
use std::sync::RwLock as StdRwLock;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;

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
        let skip_verify =
            env::var("VAULT_SKIP_VERIFY").map(|v| v == "true" || v == "1").unwrap_or(false);
        let cache_ttl = env::var("VAULT_CACHE_TTL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        let stack_path = get_stack_path();
        let ca_cert = env::var("VAULT_CACERT")
            .unwrap_or_else(|_| format!("{}/conf/system/certificates/ca/ca.crt", stack_path));
        let client_cert = env::var("VAULT_CLIENT_CERT")
            .unwrap_or_else(|_| format!("{}/conf/system/certificates/botserver/client.crt", stack_path));
        let client_key = env::var("VAULT_CLIENT_KEY")
            .unwrap_or_else(|_| format!("{}/conf/system/certificates/botserver/client.key", stack_path));

        let enabled = !token.is_empty() && !addr.is_empty();

        if !enabled {
            info!("Vault not configured yet. Using environment variables directly.");
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
        INIT.get_or_init(|| match Self::from_env() {
            Ok(manager) => {
                info!(
                    "SecretsManager singleton initialized (Vault: {})",
                    env::var("VAULT_ADDR").unwrap_or_default()
                );
                Ok(manager)
            }
            Err(e) => {
                warn!("Failed to initialize SecretsManager: {}", e);
                Err(e)
            }
        })
        .as_ref()
        .map_err(|e| anyhow!("SecretsManager initialization failed: {}", e))
    }

    pub fn get_clone() -> Result<SecretsManager> {
        Self::get().cloned()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub async fn get_secret(&self, path: &str) -> Result<HashMap<String, String>> {
        if !self.enabled {
            return Self::get_from_env_static(path);
        }

        if let Some(cached) = self.get_cached(path).await {
            return Ok(cached);
        }

        let client = self.client.as_ref().ok_or_else(|| anyhow!("No Vault client"))?;

        let result: Result<HashMap<String, String>, _> = kv2::read(client.as_ref(), "secret", path).await;

        let data = match result {
            Ok(d) => d,
            Err(e) => {
                debug!("Vault read failed for '{}': {}, falling back to env", path, e);
                return Self::get_from_env_static(path);
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
        if let Ok(secrets) = get_from_env(path) {
            if let Some(value) = secrets.get(key) {
                return value.clone();
            }
        }
        default.to_string()
    }

    pub async fn put_secret(&self, path: &str, data: HashMap<String, String>) -> Result<()> {
        let client = self.client.as_ref().ok_or_else(|| anyhow!("Vault not enabled"))?;
        kv2::set(client.as_ref(), "secret", path, &data).await?;
        self.invalidate_cache(path).await;
        info!("Secret stored at '{}'", path);
        Ok(())
    }

    pub async fn delete_secret(&self, path: &str) -> Result<()> {
        let client = self.client.as_ref().ok_or_else(|| anyhow!("Vault not enabled"))?;
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

    fn get_from_env_static(path: &str) -> Result<HashMap<String, String>> {
        get_from_env(path)
    }
}

fn get_stack_path() -> String {
    env::var("GBO_STACK_PATH").unwrap_or_else(|_| "/opt/gbo".to_string())
}
