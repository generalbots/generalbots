









use diesel::prelude::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::shared::utils::DbPool;





#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserMemoryConfig {

    pub enabled: bool,

    pub max_keys: u32,

    pub default_ttl: u64,
}

impl Default for UserMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_keys: 100,
            default_ttl: 86400,
        }
    }
}

impl UserMemoryConfig {






    pub fn from_bot_config(pool: &DbPool, target_bot_id: &Uuid) -> Self {
        let mut config = Self::default();

        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "Failed to get database connection for User Memory config: {}",
                    e
                );
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
             WHERE bot_id = $1 AND config_key LIKE 'user-memory-%'",
        )
        .bind::<diesel::sql_types::Uuid, _>(target_bot_id)
        .load(&mut conn)
        .unwrap_or_default();

        for row in configs {
            match row.config_key.as_str() {
                "user-memory-enabled" => {
                    config.enabled = row.config_value.to_lowercase() == "true";
                    debug!("User memory enabled: {}", config.enabled);
                }
                "user-memory-max-keys" => {
                    config.max_keys = row.config_value.parse().unwrap_or(100);
                    debug!("User memory max keys: {}", config.max_keys);
                }
                "user-memory-default-ttl" => {
                    config.default_ttl = row.config_value.parse().unwrap_or(86400);
                    debug!("User memory default TTL: {} seconds", config.default_ttl);
                }
                _ => {}
            }
        }


        if config.max_keys < 1 {
            warn!("User memory max keys must be at least 1, setting to default 100");
            config.max_keys = 100;
        }

        if config.max_keys > 10000 {
            warn!(
                "User memory max keys {} exceeds recommended limit, capping at 10000",
                config.max_keys
            );
            config.max_keys = 10000;
        }

        config
    }


    pub fn can_add_key(&self, current_key_count: u32) -> bool {
        self.enabled && current_key_count < self.max_keys
    }


    pub fn ttl_duration(&self) -> Option<std::time::Duration> {
        if self.default_ttl == 0 {
            None
        } else {
            Some(std::time::Duration::from_secs(self.default_ttl))
        }
    }


    pub fn has_expiration(&self) -> bool {
        self.default_ttl > 0
    }
}
