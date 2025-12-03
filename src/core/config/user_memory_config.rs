//! User Memory Configuration
//! Parameters: user-memory-enabled, user-memory-max-keys, user-memory-default-ttl
//!
//! Config.csv properties:
//! ```csv
//! user-memory-enabled,true
//! user-memory-max-keys,100
//! user-memory-default-ttl,86400
//! ```

use diesel::prelude::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::utils::DbPool;

/// Configuration for User Memory storage
///
/// User memory allows bots to store and retrieve key-value pairs
/// associated with individual users for personalization and context retention.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserMemoryConfig {
    /// Whether user memory feature is enabled
    pub enabled: bool,
    /// Maximum number of keys that can be stored per user
    pub max_keys: u32,
    /// Default time-to-live for memory entries in seconds (0 = no expiration)
    pub default_ttl: u64,
}

impl Default for UserMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_keys: 100,
            default_ttl: 86400, // 24 hours
        }
    }
}

impl UserMemoryConfig {
    /// Load User Memory configuration from bot_configuration table
    ///
    /// Reads the following parameters:
    /// - `user-memory-enabled`: Whether user memory is enabled (default: true)
    /// - `user-memory-max-keys`: Maximum keys per user (default: 100)
    /// - `user-memory-default-ttl`: Default TTL in seconds (default: 86400)
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

        // Validate configuration
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

    /// Check if a new key can be added given the current count
    pub fn can_add_key(&self, current_key_count: u32) -> bool {
        self.enabled && current_key_count < self.max_keys
    }

    /// Get the TTL duration, returns None if TTL is 0 (no expiration)
    pub fn ttl_duration(&self) -> Option<std::time::Duration> {
        if self.default_ttl == 0 {
            None
        } else {
            Some(std::time::Duration::from_secs(self.default_ttl))
        }
    }

    /// Check if entries should expire
    pub fn has_expiration(&self) -> bool {
        self.default_ttl > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UserMemoryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_keys, 100);
        assert_eq!(config.default_ttl, 86400);
    }

    #[test]
    fn test_can_add_key() {
        let config = UserMemoryConfig::default();
        assert!(config.can_add_key(0));
        assert!(config.can_add_key(99));
        assert!(!config.can_add_key(100));
        assert!(!config.can_add_key(101));
    }

    #[test]
    fn test_can_add_key_disabled() {
        let config = UserMemoryConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!config.can_add_key(0));
    }

    #[test]
    fn test_ttl_duration() {
        let config = UserMemoryConfig {
            default_ttl: 3600,
            ..Default::default()
        };
        assert_eq!(
            config.ttl_duration(),
            Some(std::time::Duration::from_secs(3600))
        );
    }

    #[test]
    fn test_ttl_duration_no_expiration() {
        let config = UserMemoryConfig {
            default_ttl: 0,
            ..Default::default()
        };
        assert_eq!(config.ttl_duration(), None);
        assert!(!config.has_expiration());
    }

    #[test]
    fn test_has_expiration() {
        let config = UserMemoryConfig::default();
        assert!(config.has_expiration());

        let no_expiry = UserMemoryConfig {
            default_ttl: 0,
            ..Default::default()
        };
        assert!(!no_expiry.has_expiration());
    }
}
