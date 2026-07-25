use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::shared::utils::DbPool;
use botcoresecrets::manager::SecretsManager;
use botsecurity_crypto::encryption::{decrypt_field, derive_scope_key, load_master_encryption_key};
use botsecurity_crypto::secrets::is_sensitive_key;
use diesel::prelude::*;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub drive: DriveConfig,
    pub email: EmailConfig,
    pub site_path: String,
    pub data_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                base_url: String::new(),
            },
            database: DatabaseConfig {
                url: "postgresql://postgres:postgres@localhost/botserver".to_string(),
                max_connections: 10,
            },
            drive: DriveConfig::default(),
            email: EmailConfig::default(),
            site_path: "/opt/gbo/data".to_string(),
            data_dir: "/opt/gbo/data".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriveConfig {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub server: String,
}

impl DriveConfig {
    pub fn is_valid(&self) -> bool {
        !self.access_key.is_empty() && !self.secret_key.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub from_address: String,
}



impl AppConfig {
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "server.host" => Some(self.server.host.clone()),
            "server.port" => Some(self.server.port.to_string()),
            "server.base_url" => Some(self.server.base_url.clone()),
            "database.url" => Some(self.database.url.clone()),
            "drive.bucket" => Some(self.drive.bucket.clone()),
            "site_path" => Some(self.site_path.clone()),
            "data_dir" => Some(self.data_dir.clone()),
            _ => None,
        }
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::default())
    }

    pub fn from_database(
        _pool: &diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_env()
    }

pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
    let drive = match DriveConfig::from_vault() {
        Ok(d) => d,
        Err(vault_err) => {
            log::warn!("DriveConfig::from_vault() failed: {vault_err}, trying env");
            match DriveConfig::from_env() {
                Ok(d) => d,
                Err(env_err) => {
                    log::warn!("DriveConfig::from_env() also failed: {env_err} — starting without Drive");
                    DriveConfig::default()
                }
            }
        }
    };

    Ok(Self {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: std::env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),
            base_url: String::new(),
        },
        database: DatabaseConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/botserver".to_string()),
            max_connections: 10,
        },
        drive,
        email: EmailConfig::default(),
        site_path: std::env::var("SITE_PATH")
            .unwrap_or_else(|_| "/opt/gbo/data".to_string()),
        data_dir: std::env::var("DATA_DIR")
            .unwrap_or_else(|_| "/opt/gbo/data".to_string()),
    })
}
}

#[derive(Debug, Clone, QueryableByName)]
struct ConfigRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_value: String,
}

#[derive(Debug, Clone, QueryableByName)]
struct ConfigRowPair {
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_key: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_value: String,
}

#[derive(Debug, Clone, QueryableByName)]
struct BotIdentityRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: uuid::Uuid,
}

fn is_placeholder_value(val: &str) -> bool {
    let lower = val.trim().to_lowercase();
    lower.is_empty() || lower == "none" || lower == "null" || lower == "n/a"
}

/// Configuration manager for runtime config updates.
/// Sensitive keys (tokens, passwords, API keys) → Vault at `gbo/bot/{org_id}/{branch_id}/{bot_id}`.
/// Non-sensitive keys → `bot_configuration` database table.
pub struct ConfigManager {
    pool: Arc<DbPool>,
}

impl ConfigManager {
    pub fn new(pool: DbPool) -> Self {
        Self { pool: Arc::new(pool) }
    }

    fn resolve_bot_identity(&self, bot_id: &uuid::Uuid) -> (uuid::Uuid, uuid::Uuid) {
        if bot_id == &uuid::Uuid::nil() {
            return (uuid::Uuid::nil(), uuid::Uuid::nil());
        }
        if let Ok(mut conn) = self.pool.get() {
            if let Ok(row) = diesel::sql_query(
                "SELECT org_id, branch_id FROM bots WHERE id = $1"
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .get_result::<BotIdentityRow>(&mut conn)
            {
                return (row.org_id, row.branch_id);
            }
        }
        (uuid::Uuid::nil(), uuid::Uuid::nil())
    }

    fn vault_path(org_id: &uuid::Uuid, branch_id: &uuid::Uuid, bot_id: &uuid::Uuid) -> String {
        format!("gbo/bot/{}/{}/{}", org_id, branch_id, bot_id)
    }

    // ── Vault helpers (for sensitive keys) ──────────────────────────

    fn read_vault_value(path: &str, key: &str) -> Option<String> {
        let sm = SecretsManager::get_clone().ok()?;
        if !sm.is_enabled() { return None; }
        let sm_clone = sm.clone();
        let path_owned = path.to_string();
        let key_owned = key.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { sm_clone.get_value(&path_owned, &key_owned).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        rx.recv().ok().flatten()
    }

    fn read_vault_all(path: &str) -> Option<std::collections::HashMap<String, String>> {
        let sm = SecretsManager::get_clone().ok()?;
        if !sm.is_enabled() { return None; }
        let sm_clone = sm.clone();
        let path_owned = path.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move { sm_clone.get_secret(&path_owned).await.ok() })
            } else { None };
            let _ = tx.send(result);
        });
        rx.recv().ok().flatten()
    }

    fn write_vault_value(path: &str, key: &str, value: &str) -> Result<(), String> {
        let sm = SecretsManager::get_clone().map_err(|e| format!("Vault not available: {}", e))?;
        if !sm.is_enabled() { return Err("Vault is not enabled".into()); }
        let sm_clone = sm.clone();
        let path_owned = path.to_string();
        let key_owned = key.to_string();
        let value_owned = value.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
            let result = if let Ok(rt) = rt {
                rt.block_on(async move {
                    let mut data = sm_clone.get_secret(&path_owned).await.unwrap_or_default();
                    data.insert(key_owned.clone(), value_owned.clone());
                    sm_clone.put_secret(&path_owned, data).await.ok()
                })
            } else { None };
            let _ = tx.send(result);
        });
        rx.recv().ok().flatten().ok_or_else(|| "Failed to write config to Vault".into())
    }

    // ── DB helpers (for non-sensitive keys) ─────────────────────────

    fn read_db_value(&self, bot_id: &uuid::Uuid, key: &str) -> Option<String> {
        if let Ok(mut conn) = self.pool.get() {
            let row: Option<ConfigRow> = diesel::sql_query(
                "SELECT config_value FROM bot_configuration \
                 WHERE bot_id = $1 AND config_key = $2 LIMIT 1"
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(key)
            .get_result(&mut conn).ok();
            if let Some(r) = row {
                if !is_placeholder_value(&r.config_value) {
                    return Some(r.config_value);
                }
            }
            // fallback: nil bot default
            let row: Option<ConfigRow> = diesel::sql_query(
                "SELECT config_value FROM bot_configuration \
                 WHERE bot_id = $1 AND config_key = $2 LIMIT 1"
            )
            .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::nil())
            .bind::<diesel::sql_types::Text, _>(key)
            .get_result(&mut conn).ok();
            if let Some(r) = row {
                if !is_placeholder_value(&r.config_value) {
                    return Some(r.config_value);
                }
            }
        }
        None
    }

    fn read_db_all(&self, bot_id: &uuid::Uuid) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        if let Ok(mut conn) = self.pool.get() {
            if let Ok(rows) = diesel::sql_query(
                "SELECT config_key, config_value FROM bot_configuration \
                 WHERE bot_id = $1"
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .load::<ConfigRowPair>(&mut conn)
            {
                for row in rows {
                    if !is_placeholder_value(&row.config_value) {
                        map.insert(row.config_key, row.config_value);
                    }
                }
            }
            // also get nil-bot defaults (not overwriting bot-specific)
            if let Ok(rows) = diesel::sql_query(
                "SELECT config_key, config_value FROM bot_configuration \
                 WHERE bot_id = $1"
            )
            .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::nil())
            .load::<ConfigRowPair>(&mut conn)
            {
                for row in rows {
                    if !is_placeholder_value(&row.config_value) && !map.contains_key(&row.config_key) {
                        map.insert(row.config_key, row.config_value);
                    }
                }
            }
        }
        map
    }

    fn write_db_value(&self, bot_id: &uuid::Uuid, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut conn) = self.pool.get() {
            diesel::sql_query(
                "INSERT INTO bot_configuration (id, bot_id, config_key, config_value, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, NOW(), NOW()) \
                 ON CONFLICT (bot_id, config_key) DO UPDATE SET config_value = $4, updated_at = NOW()"
            )
            .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Text, _>(key)
            .bind::<diesel::sql_types::Text, _>(value)
            .execute(&mut conn)?;
        }
        Ok(())
    }

    // ── public API ────────────────────────────────────────────────

    /// Read a config value.
    /// Sensitive keys → Vault, fallback DB, fallback env, fallback default.
    /// Non-sensitive keys → DB, fallback env, fallback default.
    pub fn get_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        default: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if is_sensitive_key(key) {
            let (org_id, branch_id) = self.resolve_bot_identity(bot_id);
            let nil = uuid::Uuid::nil();
            let paths = [
                Self::vault_path(&org_id, &branch_id, bot_id),
                Self::vault_path(&nil, &nil, bot_id),
            ];
            for path in &paths {
                if let Some(value) = Self::read_vault_value(path, key) {
                    if !value.starts_with("1:") {
                        return Ok(value);
                    }
                    let master_key = load_master_encryption_key();
                    let key_bytes = derive_scope_key(&master_key, "config", bot_id);
                    if let Ok(decrypted) = decrypt_field(&value, &key_bytes) {
                        return Ok(decrypted);
                    }
                    log::warn!("Failed to decrypt legacy value for '{}' in Vault for bot {}", key, bot_id);
                }
            }
            // fallback: try DB (legacy data)
            if let Some(value) = self.read_db_value(bot_id, key) {
                return Ok(value);
            }
        } else {
            if let Some(value) = self.read_db_value(bot_id, key) {
                return Ok(value);
            }
        }

        let env_key = key.to_uppercase().replace('-', "_");
        if let Ok(val) = std::env::var(&env_key) {
            if !val.is_empty() {
                return Ok(val);
            }
        }

        Ok(default.unwrap_or("").to_string())
    }

    /// Same as get_config but returns error instead of default on miss.
    pub fn get_bot_config_value(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if is_sensitive_key(key) {
            let (org_id, branch_id) = self.resolve_bot_identity(bot_id);
            let nil = uuid::Uuid::nil();
            let paths = [
                Self::vault_path(&org_id, &branch_id, bot_id),
                Self::vault_path(&nil, &nil, bot_id),
            ];
            for path in &paths {
                if let Some(value) = Self::read_vault_value(path, key) {
                    if !value.starts_with("1:") {
                        return Ok(value);
                    }
                    let master_key = load_master_encryption_key();
                    let key_bytes = derive_scope_key(&master_key, "config", bot_id);
                    if let Ok(decrypted) = decrypt_field(&value, &key_bytes) {
                        return Ok(decrypted);
                    }
                    log::warn!("Failed to decrypt legacy value for '{}' in Vault for bot {}", key, bot_id);
                }
            }
        }
        self.read_db_value(bot_id, key)
            .ok_or_else(|| "Config key not found".into())
    }

    /// Return all config (Vault sensitive + DB non-sensitive) for a bot.
    pub fn get_all_config(
        &self,
        bot_id: &uuid::Uuid,
    ) -> std::collections::HashMap<String, String> {
        let mut map = self.read_db_all(bot_id);
        let master_key = load_master_encryption_key();

        let nil = uuid::Uuid::nil();
        let (org_id, branch_id) = self.resolve_bot_identity(bot_id);
        let paths = [
            Self::vault_path(&org_id, &branch_id, bot_id),
            Self::vault_path(&nil, &nil, bot_id),
        ];
        for path in &paths {
            if let Some(vault_data) = Self::read_vault_all(path) {
                for (k, v) in vault_data {
                    if is_sensitive_key(&k) && !map.contains_key(&k) {
                        if v.starts_with("1:") {
                            let key_bytes = derive_scope_key(&master_key, "config", bot_id);
                            if let Ok(decrypted) = decrypt_field(&v, &key_bytes) {
                                map.insert(k, decrypted);
                            }
                        } else {
                            map.insert(k, v);
                        }
                    }
                }
            }
        }

        map
    }

    pub fn set_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.set_config_with_branch(bot_id, key, value, None)
    }

    /// Write a config value.
    /// Sensitive keys → Vault. Non-sensitive keys → bot_configuration table.
    pub fn set_config_with_branch(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        value: &str,
        _branch_id: Option<uuid::Uuid>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if is_sensitive_key(key) {
            let (org_id, branch_id) = self.resolve_bot_identity(bot_id);
            let path = Self::vault_path(&org_id, &branch_id, bot_id);
            Self::write_vault_value(&path, key, value)
                .map_err(|e| Box::new(std::io::Error::other(e)) as Box<dyn std::error::Error>)
        } else {
            self.write_db_value(bot_id, key, value)
        }
    }
}

pub use AppConfig as Config;

impl DriveConfig {
    /// Carrega credenciais exclusivamente do Vault.
    /// Se o Vault não estiver acessível ou o token for inválido, retorna `Err`.
    pub fn from_vault() -> Result<Self, String> {
        let vault_addr = std::env::var("VAULT_ADDR").map_err(|_| "VAULT_ADDR not set".to_string())?;
        let vault_token = std::env::var("VAULT_TOKEN").map_err(|_| "VAULT_TOKEN not set".to_string())?;
        let ca_cert = std::env::var("VAULT_CACERT").unwrap_or_default();
        let url = format!("{}/v1/secret/data/gbo/drive", vault_addr);

        let mut curl_args = vec!["-sf"];
        if ca_cert.is_empty() || !std::path::Path::new(&ca_cert).exists() {
            curl_args.push("-k");
        } else {
            curl_args.push("--cacert");
            curl_args.push(&ca_cert);
        }
        let auth_header = format!("X-Vault-Token: {}", vault_token);
        curl_args.push("-H");
        curl_args.push(&auth_header);
        curl_args.push(&url);
        let output = std::process::Command::new("curl")
            .args(&curl_args)
            .output()
            .map_err(|e| format!("Failed to execute curl to Vault: {}", e))?;

        let data: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Failed to parse Vault response: {}", e))?;

        let secret_data = data.get("data")
            .and_then(|d| d.get("data"))
            .ok_or_else(|| "Vault response missing 'data.data' path".to_string())?;

        let host = secret_data.get("host").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'host' not set".to_string())?;
        let port = secret_data.get("port").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'port' not set".to_string())?;
        let accesskey = secret_data.get("accesskey").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'accesskey' not set".to_string())?;
        let secret = secret_data.get("secret").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'secret' not set".to_string())?;
        let bucket = secret_data.get("bucket").and_then(|v| v.as_str()).ok_or_else(|| "Vault secret/gbo/drive: 'bucket' not set".to_string())?;
        let server = format!("{}:{}", host, port);

        Ok(Self {
            endpoint: format!("http://{}:{}", host, port),
            bucket: bucket.to_string(),
            region: "auto".to_string(),
            access_key: accesskey.to_string(),
            secret_key: secret.to_string(),
            server,
        })
    }

    /// Carrega credenciais das variáveis de ambiente.
    /// NÃO faz fallback para credenciais hardcoded — se as env vars não existirem,
    /// retorna `Err`.
    pub fn from_env() -> Result<Self, String> {
        let access_key = std::env::var("MINIO_ACCESS_KEY")
            .map_err(|_| "MINIO_ACCESS_KEY not set".to_string())?;
        let secret_key = std::env::var("MINIO_SECRET_KEY")
            .map_err(|_| "MINIO_SECRET_KEY not set".to_string())?;
        let endpoint = std::env::var("MINIO_ENDPOINT")
            .map_err(|_| "MINIO_ENDPOINT not set".to_string())?;
        let bucket = std::env::var("MINIO_BUCKET")
            .map_err(|_| "MINIO_BUCKET not set".to_string())?;
        let server = std::env::var("MINIO_SERVER")
            .map_err(|_| "MINIO_SERVER not set".to_string())?;

        Ok(Self {
            endpoint,
            bucket,
            region: "auto".to_string(),
            access_key,
            secret_key,
            server,
        })
    }
}


