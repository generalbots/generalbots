use anyhow::{anyhow, Result};
use std::sync::Arc;
use super::csrf::{CsrfToken, CsrfValidationResult, CsrfConfig};

const CSRF_KEY_PREFIX: &str = "csrf:";

#[derive(Debug, Clone)]
pub struct RedisCsrfStore {
    client: Arc<redis::Client>,
    config: CsrfConfig,
}

impl RedisCsrfStore {
    pub async fn new(redis_url: &str, config: CsrfConfig) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;

        let _ = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!("Redis connection error: {}", e))?;

        Ok(Self {
            client: Arc::new(client),
            config,
        })
    }

    fn token_key(&self, token: &str) -> String {
        format!("{}{}", CSRF_KEY_PREFIX, token)
    }
}

pub struct RedisCsrfManager {
    store: RedisCsrfStore,
    #[allow(dead_code)]
    secret: Vec<u8>,
}

impl RedisCsrfManager {
    pub async fn new(redis_url: &str, config: CsrfConfig, secret: &[u8]) -> Result<Self> {
        if secret.len() < 32 {
            return Err(anyhow!("CSRF secret must be at least 32 bytes"));
        }

        let store = RedisCsrfStore::new(redis_url, config).await?;

        Ok(Self {
            store,
            secret: secret.to_vec(),
        })
    }

    pub async fn generate_token(&self) -> Result<CsrfToken> {
        let token = CsrfToken::new(self.store.config.token_expiry_minutes);
        let key = self.store.token_key(&token.token);
        let value = serde_json::to_string(&token)?;
        let ttl_secs = self.store.config.token_expiry_minutes * 60;

        let client = self.store.client.clone();
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!("Redis connection error: {}", e))?;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_secs)
            .arg(&value)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| anyhow!("Failed to create CSRF token: {}", e))?;

        Ok(token)
    }

    pub async fn generate_token_with_session(&self, session_id: &str) -> Result<CsrfToken> {
        let token = CsrfToken::new(self.store.config.token_expiry_minutes)
            .with_session(session_id.to_string());
        let key = self.store.token_key(&token.token);
        let value = serde_json::to_string(&token)?;
        let ttl_secs = self.store.config.token_expiry_minutes * 60;

        let client = self.store.client.clone();
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!("Redis connection error: {}", e))?;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_secs)
            .arg(&value)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| anyhow!("Failed to create CSRF token: {}", e))?;

        Ok(token)
    }

    pub async fn validate_token(&self, token_value: &str) -> CsrfValidationResult {
        if token_value.is_empty() {
            return CsrfValidationResult::Missing;
        }

        let client = self.store.client.clone();
        let key = self.store.token_key(token_value);

        let mut conn = match client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(_) => return CsrfValidationResult::Invalid,
        };

        let value: Option<String> = match redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
        {
            Ok(v) => v,
            Err(_) => return CsrfValidationResult::Invalid,
        };

        match value {
            Some(v) => {
                let token: CsrfToken = match serde_json::from_str(&v) {
                    Ok(t) => t,
                    Err(_) => return CsrfValidationResult::Invalid,
                };

                if token.is_expired() {
                    CsrfValidationResult::Expired
                } else {
                    CsrfValidationResult::Valid
                }
            }
            None => CsrfValidationResult::Invalid,
        }
    }

    pub async fn validate_token_with_session(
        &self,
        token_value: &str,
        session_id: &str,
    ) -> CsrfValidationResult {
        if token_value.is_empty() {
            return CsrfValidationResult::Missing;
        }

        let client = self.store.client.clone();
        let key = self.store.token_key(token_value);

        let mut conn = match client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(_) => return CsrfValidationResult::Invalid,
        };

        let value: Option<String> = match redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
        {
            Ok(v) => v,
            Err(_) => return CsrfValidationResult::Invalid,
        };

        match value {
            Some(v) => {
                let token: CsrfToken = match serde_json::from_str(&v) {
                    Ok(t) => t,
                    Err(_) => return CsrfValidationResult::Invalid,
                };

                if token.is_expired() {
                    return CsrfValidationResult::Expired;
                }

                match &token.session_id {
                    Some(sid) if sid == session_id => CsrfValidationResult::Valid,
                    Some(_) => CsrfValidationResult::SessionMismatch,
                    None => CsrfValidationResult::Valid,
                }
            }
            None => CsrfValidationResult::Invalid,
        }
    }

    pub async fn revoke_token(&self, token_value: &str) -> Result<()> {
        let client = self.store.client.clone();
        let key = self.store.token_key(token_value);

        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| anyhow!("Redis connection error: {}", e))?;

        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| anyhow!("Failed to revoke CSRF token: {}", e))?;

        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<usize> {
        Ok(0)
    }
}
