pub mod model_routing_config;
pub mod sse_config;
pub mod user_memory_config;

pub use model_routing_config::{ModelRoutingConfig, RoutingStrategy, TaskType};
pub use sse_config::SseConfig;
pub use user_memory_config::UserMemoryConfig;

use crate::shared::utils::DbPool;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use std::collections::HashMap;
use uuid::Uuid;

pub type Config = AppConfig;

#[derive(Clone, Debug, Default)]
pub struct AppConfig {
    pub drive: DriveConfig,
    pub server: ServerConfig,
    pub email: EmailConfig,
    pub site_path: String,
    pub data_dir: String,
}
#[derive(Clone, Debug, Default)]
pub struct DriveConfig {
    pub server: String,
    pub access_key: String,
    pub secret_key: String,
}
#[derive(Clone, Debug, Default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}
#[derive(Clone, Debug, Default)]
pub struct EmailConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub smtp_server: String,
    pub smtp_port: u16,
}

#[derive(Clone, Debug, Default)]
pub struct CustomDatabaseConfig {
    pub server: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl CustomDatabaseConfig {
    pub fn from_bot_config(
        pool: &DbPool,
        target_bot_id: &Uuid,
    ) -> Result<Option<Self>, diesel::result::Error> {
        use crate::shared::models::schema::bot_configuration::dsl::*;

        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let database: Option<String> = bot_configuration
            .filter(bot_id.eq(target_bot_id))
            .filter(config_key.eq("custom-database"))
            .select(config_value)
            .first::<String>(&mut conn)
            .ok()
            .filter(|s| !s.is_empty());

        let Some(database) = database else {
            return Ok(None);
        };

        let server: String = bot_configuration
            .filter(bot_id.eq(target_bot_id))
            .filter(config_key.eq("custom-server"))
            .select(config_value)
            .first::<String>(&mut conn)
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "localhost".to_string());

        let port: u16 = bot_configuration
            .filter(bot_id.eq(target_bot_id))
            .filter(config_key.eq("custom-port"))
            .select(config_value)
            .first::<String>(&mut conn)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3306);

        let username: String = bot_configuration
            .filter(bot_id.eq(target_bot_id))
            .filter(config_key.eq("custom-username"))
            .select(config_value)
            .first::<String>(&mut conn)
            .ok()
            .unwrap_or_default();

        let password: String = bot_configuration
            .filter(bot_id.eq(target_bot_id))
            .filter(config_key.eq("custom-password"))
            .select(config_value)
            .first::<String>(&mut conn)
            .ok()
            .unwrap_or_default();

        Ok(Some(Self {
            server,
            port,
            database,
            username,
            password,
        }))
    }

    pub fn connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.server, self.port, self.database
        )
    }

    pub fn is_valid(&self) -> bool {
        !self.database.is_empty() && !self.server.is_empty()
    }
}

impl EmailConfig {
    pub fn from_bot_config(
        pool: &DbPool,
        target_bot_id: &Uuid,
    ) -> Result<Self, diesel::result::Error> {
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        fn get_config_value(
            conn: &mut diesel::r2d2::PooledConnection<
                diesel::r2d2::ConnectionManager<diesel::PgConnection>,
            >,
            target_bot_id: &Uuid,
            key: &str,
            default: &str,
        ) -> String {
            use crate::shared::models::schema::bot_configuration::dsl::*;
            bot_configuration
                .filter(bot_id.eq(target_bot_id))
                .filter(config_key.eq(key))
                .select(config_value)
                .first::<String>(conn)
                .unwrap_or_else(|_| default.to_string())
        }

        fn get_port_value(
            conn: &mut diesel::r2d2::PooledConnection<
                diesel::r2d2::ConnectionManager<diesel::PgConnection>,
            >,
            target_bot_id: &Uuid,
            key: &str,
            default: u16,
        ) -> u16 {
            use crate::shared::models::schema::bot_configuration::dsl::*;
            bot_configuration
                .filter(bot_id.eq(target_bot_id))
                .filter(config_key.eq(key))
                .select(config_value)
                .first::<String>(conn)
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        }

        let new_smtp_server = get_config_value(&mut conn, target_bot_id, "email-server", "");
        let smtp_server = if new_smtp_server.is_empty() {
            get_config_value(
                &mut conn,
                target_bot_id,
                "EMAIL_SMTP_SERVER",
                "smtp.gmail.com",
            )
        } else {
            new_smtp_server
        };

        let new_smtp_port = get_port_value(&mut conn, target_bot_id, "email-port", 0);
        let smtp_port = if new_smtp_port > 0 {
            new_smtp_port
        } else {
            get_port_value(&mut conn, target_bot_id, "EMAIL_SMTP_PORT", 587)
        };

        let new_from = get_config_value(&mut conn, target_bot_id, "email-from", "");
        let from = if new_from.is_empty() {
            get_config_value(&mut conn, target_bot_id, "EMAIL_FROM", "")
        } else {
            new_from
        };

        let new_user = get_config_value(&mut conn, target_bot_id, "email-user", "");
        let username = if new_user.is_empty() {
            get_config_value(&mut conn, target_bot_id, "EMAIL_USERNAME", "")
        } else {
            new_user
        };

        let new_pass = get_config_value(&mut conn, target_bot_id, "email-pass", "");
        let password = if new_pass.is_empty() {
            get_config_value(&mut conn, target_bot_id, "EMAIL_PASSWORD", "")
        } else {
            new_pass
        };

        let server = get_config_value(
            &mut conn,
            target_bot_id,
            "EMAIL_IMAP_SERVER",
            "imap.gmail.com",
        );
        let port = get_port_value(&mut conn, target_bot_id, "EMAIL_IMAP_PORT", 993);

        Ok(Self {
            server,
            port,
            username,
            password,
            from,
            smtp_server,
            smtp_port,
        })
    }
}
impl AppConfig {
    pub fn from_database(pool: &DbPool) -> Result<Self, diesel::result::Error> {
        use crate::shared::models::schema::bot_configuration::dsl::*;
        use diesel::prelude::*;

        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let config_map: HashMap<String, String> = bot_configuration
            .select((config_key, config_value))
            .load::<(String, String)>(&mut conn)
            .unwrap_or_default()
            .into_iter()
            .collect();

        let get_str = |key: &str, default: &str| -> String {
            config_map
                .get(key)
                .cloned()
                .unwrap_or_else(|| default.to_string())
        };

        let get_u16 = |key: &str, default: u16| -> u16 {
            config_map
                .get(key)
                .and_then(|v| v.parse().ok())
                .unwrap_or(default)
        };
        let drive = DriveConfig {
            server: crate::core::urls::InternalUrls::DRIVE.to_string(),
            access_key: String::new(),
            secret_key: String::new(),
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
        let port = std::env::var("BOTSERVER_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or_else(|| get_u16("server_port", 8080));

        Ok(Self {
            drive,
            email,
            server: ServerConfig {
                host: get_str("server_host", "0.0.0.0"),
                port,
                base_url: config_map.get("server_base_url").cloned().unwrap_or_else(|| "http://localhost:8080".to_string()),
            },
            site_path: {
                ConfigManager::new(pool.clone()).get_config(
                    &Uuid::nil(),
                    "SITES_ROOT",
                    Some("./botserver-stack/sites"),
                )?
            },
            data_dir: get_str("DATA_DIR", "./botserver-stack/data"),
        })
    }
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let minio = DriveConfig {
            server: crate::core::urls::InternalUrls::DRIVE.to_string(),
            access_key: String::new(),
            secret_key: String::new(),
        };
        let email = EmailConfig {
            server: "imap.gmail.com".to_string(),
            port: 993,
            username: String::new(),
            password: String::new(),
            from: String::new(),
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: 587,
        };
        let port = std::env::var("BOTSERVER_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8080);

        Ok(Self {
            drive: minio,
            email,
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port,
                base_url: "http://localhost:8080".to_string(),
            },

            site_path: "./botserver-stack/sites".to_string(),
            data_dir: "./botserver-stack/data".to_string(),
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

        // Helper function to check if a value should be treated as "not configured"
        fn is_placeholder_value(value: &str) -> bool {
            let trimmed = value.trim().to_lowercase();
            trimmed.is_empty() || trimmed == "none" || trimmed == "null" || trimmed == "n/a"
        }

        // Helper function to check if a value is a local file path (for local LLM server)
        // These should fall back to default bot's config when using remote API
        fn is_local_file_path(value: &str) -> bool {
            let value = value.trim();
            // Check for file path patterns
            value.starts_with("../") ||
            value.starts_with("./") ||
            value.starts_with('/') ||
            value.starts_with("~") ||
            value.contains(".gguf") ||
            value.contains(".bin") ||
            value.contains(".safetensors") ||
            value.starts_with("data/") ||
            value.starts_with("../../") ||
            value.starts_with("models/")
        }

        // Try to get value for the specific bot
        let result = bot_configuration
            .filter(bot_id.eq(code_bot_id))
            .filter(config_key.eq(key))
            .select(config_value)
            .first::<String>(&mut conn);

        let value = match result {
            Ok(v) => {
                // Check if it's a placeholder value or local file path - if so, fall back to default bot
                // Local file paths are valid for local LLM server but NOT for remote APIs
                if is_placeholder_value(&v) || is_local_file_path(&v) {
                    let (default_bot_id, _default_bot_name) = crate::bot::get_default_bot(&mut conn);
                    bot_configuration
                        .filter(bot_id.eq(default_bot_id))
                        .filter(config_key.eq(key))
                        .select(config_value)
                        .first::<String>(&mut conn)
                        .unwrap_or_else(|_| fallback_str.to_string())
                } else {
                    v
                }
            }
            Err(_) => {
                // Value not found, fall back to default bot
                let (default_bot_id, _default_bot_name) = crate::bot::get_default_bot(&mut conn);
                bot_configuration
                    .filter(bot_id.eq(default_bot_id))
                    .filter(config_key.eq(key))
                    .select(config_value)
                    .first::<String>(&mut conn)
                    .unwrap_or_else(|_| fallback_str.to_string())
            }
        };

        // Final check: if the result is still a placeholder value, use the fallback_str
        let final_value = if is_placeholder_value(&value) {
            fallback_str.to_string()
        } else {
            value
        };

        Ok(final_value)
    }

    pub fn get_bot_config_value(
        &self,
        target_bot_id: &uuid::Uuid,
        key: &str,
    ) -> Result<String, String> {
        use crate::shared::models::schema::bot_configuration::dsl::*;
        use diesel::prelude::*;

        let mut conn = self
            .get_conn()
            .map_err(|e| format!("Failed to acquire connection: {}", e))?;

        let value = bot_configuration
            .filter(bot_id.eq(target_bot_id))
            .filter(config_key.eq(key))
            .select(config_value)
            .first::<String>(&mut conn)
            .map_err(|e| format!("Failed to get bot config value: {}", e))?;

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

    /// Set a single configuration value for a bot (upsert)
    pub fn set_config(
        &self,
        target_bot_id: &uuid::Uuid,
        key: &str,
        value: &str,
    ) -> Result<(), diesel::result::Error> {
        let mut conn = self.get_conn()?;
        let new_id: uuid::Uuid = uuid::Uuid::new_v4();

        diesel::sql_query(
            "INSERT INTO bot_configuration (id, bot_id, config_key, config_value, config_type) \
             VALUES ($1, $2, $3, $4, 'string') \
             ON CONFLICT (bot_id, config_key) DO UPDATE SET config_value = EXCLUDED.config_value, updated_at = NOW()"
        )
        .bind::<diesel::sql_types::Uuid, _>(new_id)
        .bind::<diesel::sql_types::Uuid, _>(target_bot_id)
        .bind::<diesel::sql_types::Text, _>(key)
        .bind::<diesel::sql_types::Text, _>(value)
        .execute(&mut conn)?;

        Ok(())
    }
}
