use crate::shared::utils::DbPool;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use std::collections::HashMap;
use uuid::Uuid;

// Type alias for backward compatibility
pub type Config = AppConfig;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub drive: DriveConfig,
    pub server: ServerConfig,
    pub email: EmailConfig,
    pub site_path: String,
}
#[derive(Clone, Debug)]
pub struct DriveConfig {
    pub server: String,
    pub access_key: String,
    pub secret_key: String,
}
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}
#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub smtp_server: String,
    pub smtp_port: u16,
}
impl AppConfig {
    pub fn from_database(pool: &DbPool) -> Result<Self, diesel::result::Error> {
        use crate::shared::models::schema::bot_configuration::dsl::*;
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;
        let config_map: HashMap<String, (Uuid, Uuid, String, String, String, bool)> =
            bot_configuration
                .select((
                    id,
                    bot_id,
                    config_key,
                    config_value,
                    config_type,
                    is_encrypted,
                ))
                .load::<(Uuid, Uuid, String, String, String, bool)>(&mut conn)
                .unwrap_or_default()
                .into_iter()
                .map(|(_, _, key, value, _, _)| {
                    (
                        key.clone(),
                        (Uuid::nil(), Uuid::nil(), key, value, String::new(), false),
                    )
                })
                .collect();
        let mut get_str = |key: &str, default: &str| -> String {
            bot_configuration
                .filter(config_key.eq(key))
                .select(config_value)
                .first::<String>(&mut conn)
                .unwrap_or_else(|_| default.to_string())
        };
        let _get_u32 = |key: &str, default: u32| -> u32 {
            config_map
                .get(key)
                .and_then(|v| v.3.parse().ok())
                .unwrap_or(default)
        };
        let get_u16 = |key: &str, default: u16| -> u16 {
            config_map
                .get(key)
                .and_then(|v| v.3.parse().ok())
                .unwrap_or(default)
        };
        let _get_bool = |key: &str, default: bool| -> bool {
            config_map
                .get(key)
                .map(|v| v.3.to_lowercase() == "true")
                .unwrap_or(default)
        };
        let drive = DriveConfig {
            server: std::env::var("DRIVE_SERVER").unwrap(),
            access_key: std::env::var("DRIVE_ACCESSKEY").unwrap(),
            secret_key: std::env::var("DRIVE_SECRET").unwrap(),
        };
        let email = EmailConfig {
            server: get_str("EMAIL_IMAP_SERVER", "imap.gmail.com"),
            port: get_u16("EMAIL_IMAP_PORT", 993),
            username: get_str("EMAIL_USERNAME", ""),
            password: get_str("EMAIL_PASSWORD", ""),
            from: get_str("EMAIL_FROM", ""),
            smtp_server: get_str("EMAIL_SMTP_SERVER", "smtp.gmail.com"),
            smtp_port: get_u16("EMAIL_SMTP_PORT", 587),
        };
        Ok(AppConfig {
            drive,
            email,
            server: ServerConfig {
                host: get_str("SERVER_HOST", "127.0.0.1"),
                port: get_u16("SERVER_PORT", 8080),
            },
            site_path: {
                ConfigManager::new(pool.clone())
                    .get_config(&Uuid::nil(), "SITES_ROOT", Some("./botserver-stack/sites"))?
                    .to_string()
            },
        })
    }
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let minio = DriveConfig {
            server: std::env::var("DRIVE_SERVER").unwrap(),
            access_key: std::env::var("DRIVE_ACCESSKEY").unwrap(),
            secret_key: std::env::var("DRIVE_SECRET").unwrap(),
        };
        let email = EmailConfig {
            server: std::env::var("EMAIL_IMAP_SERVER")
                .unwrap_or_else(|_| "imap.gmail.com".to_string()),
            port: std::env::var("EMAIL_IMAP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(993),
            username: std::env::var("EMAIL_USERNAME").unwrap_or_default(),
            password: std::env::var("EMAIL_PASSWORD").unwrap_or_default(),
            from: std::env::var("EMAIL_FROM").unwrap_or_default(),
            smtp_server: std::env::var("EMAIL_SMTP_SERVER")
                .unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: std::env::var("EMAIL_SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
        };
        Ok(AppConfig {
            drive: minio,
            email,
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: std::env::var("SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
            },
            site_path: {
                let pool = create_conn()?;
                ConfigManager::new(pool).get_config(
                    &Uuid::nil(),
                    "SITES_ROOT",
                    Some("./botserver-stack/sites"),
                )?
            },
        })
    }
}
#[derive(Debug)]
pub struct ConfigManager {
    conn: DbPool,
}
impl ConfigManager {
    pub fn new(conn: DbPool) -> Self {
        Self { conn }
    }
    fn get_conn(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, diesel::result::Error> {
        self.conn.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })
    }
    pub fn get_config(
        &self,
        code_bot_id: &uuid::Uuid,
        key: &str,
        fallback: Option<&str>,
    ) -> Result<String, diesel::result::Error> {
        use crate::shared::models::schema::bot_configuration::dsl::*;
        let mut conn = self.get_conn()?;
        let fallback_str = fallback.unwrap_or("");
        let result = bot_configuration
            .filter(bot_id.eq(code_bot_id))
            .filter(config_key.eq(key))
            .select(config_value)
            .first::<String>(&mut conn);
        let value = match result {
            Ok(v) => v,
            Err(_) => {
                let (default_bot_id, _default_bot_name) = crate::bot::get_default_bot(&mut conn);
                bot_configuration
                    .filter(bot_id.eq(default_bot_id))
                    .filter(config_key.eq(key))
                    .select(config_value)
                    .first::<String>(&mut conn)
                    .unwrap_or(fallback_str.to_string())
            }
        };
        Ok(value)
    }
    pub fn sync_gbot_config(&self, bot_id: &uuid::Uuid, content: &str) -> Result<usize, String> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let mut conn = self
            .get_conn()
            .map_err(|e| format!("Failed to acquire connection: {}", e))?;
        let mut updated = 0;
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();
                let new_id: uuid::Uuid = uuid::Uuid::new_v4();
                diesel::sql_query("INSERT INTO bot_configuration (id, bot_id, config_key, config_value, config_type) VALUES ($1, $2, $3, $4, 'string') ON CONFLICT (bot_id, config_key) DO UPDATE SET config_value = EXCLUDED.config_value, updated_at = NOW()")
 .bind::<diesel::sql_types::Uuid, _>(new_id)
 .bind::<diesel::sql_types::Uuid, _>(bot_id)
 .bind::<diesel::sql_types::Text, _>(key)
 .bind::<diesel::sql_types::Text, _>(value)
 .execute(&mut conn)
 .map_err(|e| format!("Failed to update config: {}", e))?;
                updated += 1;
            }
        }
        Ok(updated)
    }
}
fn create_conn() -> Result<DbPool, anyhow::Error> {
    crate::shared::utils::create_conn()
        .map_err(|e| anyhow::anyhow!("Failed to create database pool: {}", e))
}
