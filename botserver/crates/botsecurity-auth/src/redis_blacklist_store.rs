//! Redis/Valkey-backed token blacklist (feature `cache`).
//!
//! Revocations are persisted in the shared cache so they survive process
//! restarts. Each entry is stored under `jwt:blacklist:{jti}` with a TTL
//! equal to the remaining token lifetime, bounding growth even when the
//! explicit cleanup pass never runs.

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::info;

use crate::blacklist::{BlacklistEntry, BlacklistStore};

const BLACKLIST_KEY_PREFIX: &str = "jwt:blacklist:";

#[derive(Clone)]
pub struct RedisBlacklistStore {
    client: Arc<redis::Client>,
    /// Lazily-created blocking connection used by the synchronous revocation
    /// check (#2830). `fn`-pointer JWT verifiers cannot await the async
    /// store, so they do a direct `EXISTS` on the shared Redis instance —
    /// that is what makes a logout on instance A visible on instance B.
    sync_conn: Arc<std::sync::Mutex<Option<redis::Connection>>>,
}

impl std::fmt::Debug for RedisBlacklistStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RedisBlacklistStore")
            .field("client", &self.client)
            .finish_non_exhaustive()
    }
}

impl RedisBlacklistStore {
    /// Creates a Redis-backed blacklist from a shared client.
    pub fn new(client: Arc<redis::Client>) -> Self {
        Self {
            client,
            sync_conn: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn key(jti: &str) -> String {
        format!("{}{}", BLACKLIST_KEY_PREFIX, jti)
    }
}

#[async_trait::async_trait]
impl BlacklistStore for RedisBlacklistStore {
    async fn insert(&self, entry: BlacklistEntry) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow::anyhow!("Redis connection error: {e}"))?;

        let value = serde_json::to_string(&entry)?;
        // TTL bounds growth: an entry can never outlive the original token
        // expiration, so expire the key then. Revoked tokens must stay
        // rejected for as long as they could still be presented.
        let ttl_secs = (entry.expires_at - Utc::now()).num_seconds().max(1);

        redis::cmd("SETEX")
            .arg(Self::key(&entry.jti))
            .arg(ttl_secs)
            .arg(&value)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to store revoked token: {e}"))
    }

    async fn contains(&self, jti: &str) -> bool {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::warn!("Redis connection error while checking token blacklist: {e}");
                return false;
            }
        };
        let key = Self::key(jti);
        match redis::cmd("EXISTS").arg(&key).query_async::<i64>(&mut conn).await {
            Ok(count) => count > 0,
            Err(e) => {
                tracing::warn!("Redis EXISTS failed for token blacklist key: {e}");
                false
            }
        }
    }

    fn contains_sync(&self, jti: &str) -> Option<bool> {
        // Synchronous EXISTS against the *shared* store: a jti blacklisted by
        // a logout on another instance is rejected here too (#2830). The
        // connection is cached so a plain-fn verifier pays no async/await cost
        // and no per-check connection setup beyond the first call.
        let mut guard = match self.sync_conn.lock() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::warn!("Redis sync blacklist lock poisoned: {e}");
                return None;
            }
        };
        if guard.is_none() {
            match self.client.get_connection() {
                Ok(connection) => *guard = Some(connection),
                Err(e) => {
                    tracing::warn!("Redis connection error during sync blacklist check: {e}");
                    return None;
                }
            }
        }
        let key = Self::key(jti);
        let result = redis::cmd("EXISTS")
            .arg(&key)
            .query::<i64>(guard.as_mut()?);
        match result {
            Ok(count) => Some(count > 0),
            Err(e) => {
                tracing::warn!("Redis sync EXISTS failed for token blacklist key: {e}");
                *guard = None; // force a reconnect on the next check
                None
            }
        }
    }

    async fn cleanup(&self, expired_before: DateTime<Utc>) -> usize {
        // Redis TTL already prunes keys whose original expiry passed. This
        // pass additionally removes entries whose revocation AND original
        // expiration both precede the given instant (e.g. after a restart).
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::warn!("Redis connection error during token blacklist cleanup: {e}");
                return 0;
            }
        };

        let pattern = format!("{}*", BLACKLIST_KEY_PREFIX);
        let mut removed = 0usize;
        let mut cursor = 0i64;

        loop {
            let (next_cursor, keys): (i64, Vec<String>) =
                match redis::cmd("SCAN").arg(cursor).arg("MATCH").arg(&pattern)
                    .query_async(&mut conn)
                    .await
                {
                    Ok(result) => result,
                    Err(e) => {
                        tracing::warn!("Redis SCAN failed during token blacklist cleanup: {e}");
                        break;
                    }
                };

            for key in keys {
                let value: Option<String> = match redis::cmd("GET")
                    .arg(&key)
                    .query_async(&mut conn)
                    .await
                {
                    Ok(value) => value,
                    Err(e) => {
                        tracing::warn!("Redis GET failed during token blacklist cleanup: {e}");
                        continue;
                    }
                };

                let expired = match value {
                    Some(raw) => match serde_json::from_str::<BlacklistEntry>(&raw) {
                        Ok(entry) => entry.is_expired_before(expired_before),
                        Err(_) => false,
                    },
                    None => false,
                };

                if expired {
                    if let Err(e) = redis::cmd("DEL").arg(&key).query_async::<()>(&mut conn).await {
                        tracing::warn!("Redis DEL failed during token blacklist cleanup: {e}");
                    }
                    removed += 1;
                }
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        if removed > 0 {
            info!("Token blacklist cleanup removed {removed} expired entries from Redis");
        }
        removed
    }

    async fn len(&self) -> usize {
        let mut conn = match self.client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::warn!("Redis connection error while sizing token blacklist: {e}");
                return 0;
            }
        };
        let pattern = format!("{}*", BLACKLIST_KEY_PREFIX);
        let mut count = 0usize;
        let mut cursor = 0i64;

        loop {
            let (next_cursor, keys): (i64, Vec<String>) =
                match redis::cmd("SCAN").arg(cursor).arg("MATCH").arg(&pattern)
                    .query_async(&mut conn)
                    .await
                {
                    Ok(result) => result,
                    Err(_) => break,
                };
            count += keys.len();
            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_format() {
        assert_eq!(RedisBlacklistStore::key("abc"), "jwt:blacklist:abc");
    }
}
