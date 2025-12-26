use diesel::prelude::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::utils::DbPool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SseConfig {
    pub enabled: bool,

    pub heartbeat_seconds: u32,

    pub max_connections: u32,
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            heartbeat_seconds: 30,
            max_connections: 1000,
        }
    }
}

impl SseConfig {
    pub fn from_bot_config(pool: &DbPool, target_bot_id: &Uuid) -> Self {
        let mut config = Self::default();

        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to get database connection for SSE config: {}", e);
                return config;
            }
        };

        #[derive(QueryableByName)]
        struct ConfigRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_key: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_value: String,
        }

        let configs: Vec<ConfigRow> = diesel::sql_query(
            "SELECT config_key, config_value FROM bot_configuration \
             WHERE bot_id = $1 AND config_key LIKE 'sse-%'",
        )
        .bind::<diesel::sql_types::Uuid, _>(target_bot_id)
        .load(&mut conn)
        .unwrap_or_default();

        for row in configs {
            match row.config_key.as_str() {
                "sse-enabled" => {
                    config.enabled = row.config_value.to_lowercase() == "true";
                    debug!("SSE enabled: {}", config.enabled);
                }
                "sse-heartbeat" => {
                    config.heartbeat_seconds = row.config_value.parse().unwrap_or(30);
                    debug!("SSE heartbeat: {} seconds", config.heartbeat_seconds);
                }
                "sse-max-connections" => {
                    config.max_connections = row.config_value.parse().unwrap_or(1000);
                    debug!("SSE max connections: {}", config.max_connections);
                }
                _ => {}
            }
        }

        if config.heartbeat_seconds < 5 {
            warn!(
                "SSE heartbeat interval {} is too low, setting to minimum of 5 seconds",
                config.heartbeat_seconds
            );
            config.heartbeat_seconds = 5;
        }

        if config.max_connections < 1 {
            warn!("SSE max connections must be at least 1, setting to default 1000");
            config.max_connections = 1000;
        }

        config
    }

    pub fn can_accept_connection(&self, current_connections: u32) -> bool {
        self.enabled && current_connections < self.max_connections
    }

    pub fn heartbeat_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(u64::from(self.heartbeat_seconds))
    }
}
