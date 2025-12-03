//! SSE Configuration
//! Parameters: sse-enabled, sse-heartbeat, sse-max-connections
//!
//! Config.csv properties:
//! ```csv
//! sse-enabled,true
//! sse-heartbeat,30
//! sse-max-connections,1000
//! ```

use diesel::prelude::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::utils::DbPool;

/// Configuration for Server-Sent Events (SSE)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SseConfig {
    /// Whether SSE is enabled for real-time updates
    pub enabled: bool,
    /// Heartbeat interval in seconds to keep connections alive
    pub heartbeat_seconds: u32,
    /// Maximum number of concurrent SSE connections per bot
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
    /// Load SSE configuration from bot_configuration table
    ///
    /// Reads the following parameters:
    /// - `sse-enabled`: Whether SSE is enabled (default: true)
    /// - `sse-heartbeat`: Heartbeat interval in seconds (default: 30)
    /// - `sse-max-connections`: Maximum concurrent connections (default: 1000)
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

        // Validate configuration
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

    /// Check if more connections can be accepted
    pub fn can_accept_connection(&self, current_connections: u32) -> bool {
        self.enabled && current_connections < self.max_connections
    }

    /// Get the heartbeat duration for SSE keep-alive
    pub fn heartbeat_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.heartbeat_seconds as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SseConfig::default();
        assert!(config.enabled);
        assert_eq!(config.heartbeat_seconds, 30);
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_can_accept_connection() {
        let config = SseConfig::default();
        assert!(config.can_accept_connection(0));
        assert!(config.can_accept_connection(999));
        assert!(!config.can_accept_connection(1000));
        assert!(!config.can_accept_connection(1001));
    }

    #[test]
    fn test_can_accept_connection_disabled() {
        let config = SseConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!config.can_accept_connection(0));
    }

    #[test]
    fn test_heartbeat_duration() {
        let config = SseConfig {
            heartbeat_seconds: 45,
            ..Default::default()
        };
        assert_eq!(
            config.heartbeat_duration(),
            std::time::Duration::from_secs(45)
        );
    }
}
