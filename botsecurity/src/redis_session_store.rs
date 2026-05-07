use anyhow::{anyhow, Result};
use std::sync::Arc;

use super::session::{Session, SessionStore};

const SESSION_KEY_PREFIX: &str = "session:";

#[derive(Debug, Clone)]
pub struct RedisSessionStore {
    client: Arc<redis::Client>,
}

impl RedisSessionStore {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;

        let _ = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!("Redis connection error: {}", e))?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    fn session_key(&self, session_id: &str) -> String {
        format!("{}{}", SESSION_KEY_PREFIX, session_id)
    }
}

impl SessionStore for RedisSessionStore {
    fn create(&self, session: Session) -> impl std::future::Future<Output = Result<()>> + Send {
        let client = self.client.clone();
        let key = self.session_key(&session.id);
        let ttl = session.time_until_expiry();
        let ttl_secs = ttl.num_seconds().max(0) as usize;

        async move {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| anyhow!("Redis connection error: {}", e))?;

            let value = serde_json::to_string(&session)?;

            redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl_secs)
                .arg(&value)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| anyhow!("Failed to create session: {}", e))?;

            Ok(())
        }
    }

    fn get(&self, session_id: &str) -> impl std::future::Future<Output = Result<Option<Session>>> + Send {
        let client = self.client.clone();
        let key = self.session_key(session_id);

        async move {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| anyhow!("Redis connection error: {}", e))?;

            let value: Option<String> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .map_err(|e| anyhow!("Failed to get session: {}", e))?;

            match value {
                Some(v) => {
                    let session: Session = serde_json::from_str(&v)
                        .map_err(|e| anyhow!("Failed to deserialize session: {}", e))?;
                    Ok(Some(session))
                }
                None => Ok(None),
            }
        }
    }

    fn update(&self, session: &Session) -> impl std::future::Future<Output = Result<()>> + Send {
        let client = self.client.clone();
        let key = self.session_key(&session.id);
        let session = session.clone();
        let ttl = session.time_until_expiry();
        let ttl_secs = ttl.num_seconds().max(0) as usize;

        async move {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| anyhow!("Redis connection error: {}", e))?;

            let value = serde_json::to_string(&session)?;

            redis::cmd("SETEX")
                .arg(&key)
                .arg(ttl_secs)
                .arg(&value)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| anyhow!("Failed to update session: {}", e))?;

            Ok(())
        }
    }

    fn delete(&self, session_id: &str) -> impl std::future::Future<Output = Result<()>> + Send {
        let client = self.client.clone();
        let key = self.session_key(session_id);

        async move {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| anyhow!("Redis connection error: {}", e))?;

            redis::cmd("DEL")
                .arg(&key)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| anyhow!("Failed to delete session: {}", e))?;

            Ok(())
        }
    }

    fn get_user_sessions(&self, user_id: uuid::Uuid) -> impl std::future::Future<Output = Result<Vec<Session>>> + Send {
        let client = self.client.clone();
        let prefix = SESSION_KEY_PREFIX.to_string();

        async move {
            let mut conn = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| anyhow!("Redis connection error: {}", e))?;

            let pattern = format!("{}*", prefix);
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg(&pattern)
                .query_async(&mut conn)
                .await
                .map_err(|e| anyhow!("Failed to list sessions: {}", e))?;

            let mut sessions = Vec::new();

            for key in keys {
                let session_id = key.trim_start_matches(&prefix);
                let store = Self { client: client.clone() };
                if let Ok(Some(session)) = store.get(session_id).await {
                    if session.user_id == user_id && session.is_valid() {
                        sessions.push(session);
                    }
                }
            }

            Ok(sessions)
        }
    }

    fn delete_user_sessions(&self, user_id: uuid::Uuid) -> impl std::future::Future<Output = Result<usize>> + Send {
        let client = self.client.clone();

        async move {
            let sessions = Self { client: client.clone() }.get_user_sessions(user_id).await?;
            let count = sessions.len();

            for session in sessions {
                Self { client: client.clone() }.delete(&session.id).await?;
            }

            Ok(count)
        }
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        Ok(0)
    }
}
