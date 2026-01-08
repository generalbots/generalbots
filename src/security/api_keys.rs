use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

const API_KEY_PREFIX: &str = "gb_";
const API_KEY_LENGTH: usize = 32;
const API_KEY_HASH_ITERATIONS: u32 = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub default_rate_limit_per_minute: u32,
    pub default_rate_limit_per_hour: u32,
    pub default_rate_limit_per_day: u32,
    pub max_keys_per_user: usize,
    pub key_expiry_days: Option<u32>,
    pub allow_key_rotation: bool,
    pub rotation_grace_period_hours: u32,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            default_rate_limit_per_minute: 60,
            default_rate_limit_per_hour: 1000,
            default_rate_limit_per_day: 10000,
            max_keys_per_user: 10,
            key_expiry_days: Some(365),
            allow_key_rotation: true,
            rotation_grace_period_hours: 24,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApiKeyScope {
    Read,
    Write,
    Delete,
    Admin,
    Bot(Uuid),
    Resource(String),
    Custom(String),
}

impl ApiKeyScope {
    pub fn as_str(&self) -> String {
        match self {
            Self::Read => "read".into(),
            Self::Write => "write".into(),
            Self::Delete => "delete".into(),
            Self::Admin => "admin".into(),
            Self::Bot(id) => format!("bot:{id}"),
            Self::Resource(r) => format!("resource:{r}"),
            Self::Custom(c) => format!("custom:{c}"),
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "delete" => Some(Self::Delete),
            "admin" => Some(Self::Admin),
            s if s.starts_with("bot:") => {
                let id = s.strip_prefix("bot:")?;
                Uuid::parse_str(id).ok().map(Self::Bot)
            }
            s if s.starts_with("resource:") => {
                let r = s.strip_prefix("resource:")?;
                Some(Self::Resource(r.to_string()))
            }
            s if s.starts_with("custom:") => {
                let c = s.strip_prefix("custom:")?;
                Some(Self::Custom(c.to_string()))
            }
            _ => None,
        }
    }

    pub fn includes(&self, other: &Self) -> bool {
        if self == &Self::Admin {
            return true;
        }
        if self == other {
            return true;
        }
        if self == &Self::Write && other == &Self::Read {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiKeyStatus {
    Active,
    Expired,
    Revoked,
    Rotating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub key_hash: String,
    pub key_prefix: String,
    pub scopes: HashSet<ApiKeyScope>,
    pub status: ApiKeyStatus,
    pub rate_limits: RateLimitConfig,
    pub allowed_ips: Option<Vec<String>>,
    pub allowed_origins: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub rotated_from: Option<Uuid>,
    pub rotation_deadline: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

impl ApiKey {
    pub fn is_valid(&self) -> bool {
        if self.status != ApiKeyStatus::Active && self.status != ApiKeyStatus::Rotating {
            return false;
        }

        if let Some(expires) = self.expires_at {
            if Utc::now() > expires {
                return false;
            }
        }

        if self.status == ApiKeyStatus::Rotating {
            if let Some(deadline) = self.rotation_deadline {
                if Utc::now() > deadline {
                    return false;
                }
            }
        }

        true
    }

    pub fn has_scope(&self, scope: &ApiKeyScope) -> bool {
        if self.scopes.contains(&ApiKeyScope::Admin) {
            return true;
        }

        for s in &self.scopes {
            if s.includes(scope) {
                return true;
            }
        }

        false
    }

    pub fn has_any_scope(&self, scopes: &[ApiKeyScope]) -> bool {
        scopes.iter().any(|s| self.has_scope(s))
    }

    pub fn has_all_scopes(&self, scopes: &[ApiKeyScope]) -> bool {
        scopes.iter().all(|s| self.has_scope(s))
    }

    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        match &self.allowed_ips {
            Some(ips) if !ips.is_empty() => ips.iter().any(|allowed| {
                if allowed.contains('/') {
                    matches_cidr(ip, allowed)
                } else {
                    allowed == ip
                }
            }),
            _ => true,
        }
    }

    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        match &self.allowed_origins {
            Some(origins) if !origins.is_empty() => origins.iter().any(|allowed| {
                if allowed == "*" {
                    true
                } else if allowed.starts_with("*.") {
                    let suffix = allowed.strip_prefix("*").unwrap_or(allowed);
                    origin.ends_with(suffix)
                } else {
                    allowed == origin
                }
            }),
            _ => true,
        }
    }

    pub fn time_until_expiry(&self) -> Option<Duration> {
        self.expires_at.map(|e| e - Utc::now())
    }

    pub fn is_expiring_soon(&self, days: i64) -> bool {
        if let Some(remaining) = self.time_until_expiry() {
            remaining < Duration::days(days)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<ApiKeyScope>,
    pub expires_in_days: Option<u32>,
    pub rate_limits: Option<RateLimitConfig>,
    pub allowed_ips: Option<Vec<String>>,
    pub allowed_origins: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub key: ApiKey,
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyUsage {
    pub key_id: Uuid,
    pub requests_this_minute: u32,
    pub requests_this_hour: u32,
    pub requests_this_day: u32,
    pub minute_reset_at: DateTime<Utc>,
    pub hour_reset_at: DateTime<Utc>,
    pub day_reset_at: DateTime<Utc>,
    pub total_requests: u64,
    pub last_request_at: Option<DateTime<Utc>>,
}

impl ApiKeyUsage {
    pub fn new(key_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            key_id,
            requests_this_minute: 0,
            requests_this_hour: 0,
            requests_this_day: 0,
            minute_reset_at: now + Duration::minutes(1),
            hour_reset_at: now + Duration::hours(1),
            day_reset_at: now + Duration::days(1),
            total_requests: 0,
            last_request_at: None,
        }
    }

    pub fn record_request(&mut self) {
        let now = Utc::now();

        if now >= self.minute_reset_at {
            self.requests_this_minute = 0;
            self.minute_reset_at = now + Duration::minutes(1);
        }
        if now >= self.hour_reset_at {
            self.requests_this_hour = 0;
            self.hour_reset_at = now + Duration::hours(1);
        }
        if now >= self.day_reset_at {
            self.requests_this_day = 0;
            self.day_reset_at = now + Duration::days(1);
        }

        self.requests_this_minute += 1;
        self.requests_this_hour += 1;
        self.requests_this_day += 1;
        self.total_requests += 1;
        self.last_request_at = Some(now);
    }

    pub fn is_rate_limited(&self, limits: &RateLimitConfig) -> bool {
        let now = Utc::now();

        if now < self.minute_reset_at && self.requests_this_minute >= limits.requests_per_minute {
            return true;
        }
        if now < self.hour_reset_at && self.requests_this_hour >= limits.requests_per_hour {
            return true;
        }
        if now < self.day_reset_at && self.requests_this_day >= limits.requests_per_day {
            return true;
        }

        false
    }

    pub fn time_until_reset(&self, limits: &RateLimitConfig) -> Option<Duration> {
        let now = Utc::now();

        if now < self.minute_reset_at && self.requests_this_minute >= limits.requests_per_minute {
            return Some(self.minute_reset_at - now);
        }
        if now < self.hour_reset_at && self.requests_this_hour >= limits.requests_per_hour {
            return Some(self.hour_reset_at - now);
        }
        if now < self.day_reset_at && self.requests_this_day >= limits.requests_per_day {
            return Some(self.day_reset_at - now);
        }

        None
    }
}

pub struct ApiKeyManager {
    config: ApiKeyConfig,
    keys: Arc<RwLock<HashMap<Uuid, ApiKey>>>,
    key_hashes: Arc<RwLock<HashMap<String, Uuid>>>,
    usage: Arc<RwLock<HashMap<Uuid, ApiKeyUsage>>>,
}

impl ApiKeyManager {
    pub fn new(config: ApiKeyConfig) -> Self {
        Self {
            config,
            keys: Arc::new(RwLock::new(HashMap::new())),
            key_hashes: Arc::new(RwLock::new(HashMap::new())),
            usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(ApiKeyConfig::default())
    }

    pub async fn create_key(
        &self,
        user_id: Uuid,
        request: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse> {
        let user_keys = self.get_user_keys(user_id).await;
        if user_keys.len() >= self.config.max_keys_per_user {
            return Err(anyhow!(
                "Maximum number of API keys ({}) reached",
                self.config.max_keys_per_user
            ));
        }

        let (secret, key_hash) = generate_api_key();
        let key_prefix = extract_prefix(&secret);

        let expires_at = request
            .expires_in_days
            .or(self.config.key_expiry_days)
            .map(|days| Utc::now() + Duration::days(days as i64));

        let rate_limits = request.rate_limits.unwrap_or_else(|| RateLimitConfig {
            requests_per_minute: self.config.default_rate_limit_per_minute,
            requests_per_hour: self.config.default_rate_limit_per_hour,
            requests_per_day: self.config.default_rate_limit_per_day,
        });

        let key = ApiKey {
            id: Uuid::new_v4(),
            user_id,
            name: request.name,
            description: request.description,
            key_hash: key_hash.clone(),
            key_prefix,
            scopes: request.scopes.into_iter().collect(),
            status: ApiKeyStatus::Active,
            rate_limits,
            allowed_ips: request.allowed_ips,
            allowed_origins: request.allowed_origins,
            created_at: Utc::now(),
            expires_at,
            last_used_at: None,
            rotated_from: None,
            rotation_deadline: None,
            metadata: request.metadata.unwrap_or_default(),
        };

        let key_id = key.id;

        {
            let mut keys = self.keys.write().await;
            keys.insert(key_id, key.clone());
        }

        {
            let mut hashes = self.key_hashes.write().await;
            hashes.insert(key_hash, key_id);
        }

        info!("Created API key {} for user {}", key_id, user_id);

        Ok(CreateApiKeyResponse { key, secret })
    }

    pub async fn validate_key(&self, secret: &str) -> Result<Option<ApiKey>> {
        let key_hash = hash_api_key(secret);

        let key_id = {
            let hashes = self.key_hashes.read().await;
            hashes.get(&key_hash).copied()
        };

        let key_id = match key_id {
            Some(id) => id,
            None => return Ok(None),
        };

        let key = {
            let keys = self.keys.read().await;
            keys.get(&key_id).cloned()
        };

        let key = match key {
            Some(k) => k,
            None => return Ok(None),
        };

        if !key.is_valid() {
            return Ok(None);
        }

        {
            let mut keys = self.keys.write().await;
            if let Some(k) = keys.get_mut(&key_id) {
                k.last_used_at = Some(Utc::now());
            }
        }

        Ok(Some(key))
    }

    pub async fn validate_and_check_rate_limit(&self, secret: &str) -> Result<(ApiKey, bool)> {
        let key = self
            .validate_key(secret)
            .await?
            .ok_or_else(|| anyhow!("Invalid API key"))?;

        let is_rate_limited = {
            let mut usage = self.usage.write().await;
            let key_usage = usage
                .entry(key.id)
                .or_insert_with(|| ApiKeyUsage::new(key.id));

            if key_usage.is_rate_limited(&key.rate_limits) {
                true
            } else {
                key_usage.record_request();
                false
            }
        };

        Ok((key, is_rate_limited))
    }

    pub async fn get_key(&self, key_id: Uuid) -> Option<ApiKey> {
        let keys = self.keys.read().await;
        keys.get(&key_id).cloned()
    }

    pub async fn get_user_keys(&self, user_id: Uuid) -> Vec<ApiKey> {
        let keys = self.keys.read().await;
        keys.values()
            .filter(|k| k.user_id == user_id)
            .cloned()
            .collect()
    }

    pub async fn revoke_key(&self, key_id: Uuid) -> Result<bool> {
        let mut keys = self.keys.write().await;

        if let Some(key) = keys.get_mut(&key_id) {
            key.status = ApiKeyStatus::Revoked;
            info!("Revoked API key {}", key_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn revoke_all_user_keys(&self, user_id: Uuid) -> Result<usize> {
        let mut keys = self.keys.write().await;
        let mut revoked = 0;

        for key in keys.values_mut() {
            if key.user_id == user_id && key.status == ApiKeyStatus::Active {
                key.status = ApiKeyStatus::Revoked;
                revoked += 1;
            }
        }

        info!("Revoked {} API keys for user {}", revoked, user_id);
        Ok(revoked)
    }

    pub async fn rotate_key(&self, key_id: Uuid) -> Result<CreateApiKeyResponse> {
        if !self.config.allow_key_rotation {
            return Err(anyhow!("API key rotation is not enabled"));
        }

        let old_key = {
            let keys = self.keys.read().await;
            keys.get(&key_id).cloned()
        };

        let old_key = old_key.ok_or_else(|| anyhow!("API key not found"))?;

        if old_key.status != ApiKeyStatus::Active {
            return Err(anyhow!("Can only rotate active keys"));
        }

        let (secret, key_hash) = generate_api_key();
        let key_prefix = extract_prefix(&secret);
        let grace_period = Duration::hours(self.config.rotation_grace_period_hours as i64);

        let new_key = ApiKey {
            id: Uuid::new_v4(),
            user_id: old_key.user_id,
            name: old_key.name.clone(),
            description: old_key.description.clone(),
            key_hash: key_hash.clone(),
            key_prefix,
            scopes: old_key.scopes.clone(),
            status: ApiKeyStatus::Active,
            rate_limits: old_key.rate_limits.clone(),
            allowed_ips: old_key.allowed_ips.clone(),
            allowed_origins: old_key.allowed_origins.clone(),
            created_at: Utc::now(),
            expires_at: old_key.expires_at,
            last_used_at: None,
            rotated_from: Some(key_id),
            rotation_deadline: None,
            metadata: old_key.metadata.clone(),
        };

        let new_key_id = new_key.id;

        {
            let mut keys = self.keys.write().await;

            if let Some(old) = keys.get_mut(&key_id) {
                old.status = ApiKeyStatus::Rotating;
                old.rotation_deadline = Some(Utc::now() + grace_period);
            }

            keys.insert(new_key_id, new_key.clone());
        }

        {
            let mut hashes = self.key_hashes.write().await;
            hashes.insert(key_hash, new_key_id);
        }

        info!(
            "Rotated API key {} to {} (grace period: {} hours)",
            key_id, new_key_id, self.config.rotation_grace_period_hours
        );

        Ok(CreateApiKeyResponse {
            key: new_key,
            secret,
        })
    }

    pub async fn update_key_scopes(&self, key_id: Uuid, scopes: Vec<ApiKeyScope>) -> Result<bool> {
        let mut keys = self.keys.write().await;

        if let Some(key) = keys.get_mut(&key_id) {
            key.scopes = scopes.into_iter().collect();
            info!("Updated scopes for API key {}", key_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn update_key_rate_limits(
        &self,
        key_id: Uuid,
        rate_limits: RateLimitConfig,
    ) -> Result<bool> {
        let mut keys = self.keys.write().await;

        if let Some(key) = keys.get_mut(&key_id) {
            key.rate_limits = rate_limits;
            info!("Updated rate limits for API key {}", key_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_key_usage(&self, key_id: Uuid) -> Option<ApiKeyUsage> {
        let usage = self.usage.read().await;
        usage.get(&key_id).cloned()
    }

    pub async fn cleanup_expired_keys(&self) -> Result<usize> {
        let mut keys = self.keys.write().await;
        let mut hashes = self.key_hashes.write().await;

        let expired_ids: Vec<Uuid> = keys
            .iter()
            .filter(|(_, k)| {
                if let Some(expires) = k.expires_at {
                    Utc::now() > expires
                } else {
                    false
                }
            })
            .map(|(id, _)| *id)
            .collect();

        for id in &expired_ids {
            if let Some(key) = keys.remove(id) {
                hashes.remove(&key.key_hash);
            }
        }

        if !expired_ids.is_empty() {
            info!("Cleaned up {} expired API keys", expired_ids.len());
        }

        Ok(expired_ids.len())
    }

    pub async fn cleanup_rotation_grace_periods(&self) -> Result<usize> {
        let mut keys = self.keys.write().await;
        let mut count = 0;

        for key in keys.values_mut() {
            if key.status == ApiKeyStatus::Rotating {
                if let Some(deadline) = key.rotation_deadline {
                    if Utc::now() > deadline {
                        key.status = ApiKeyStatus::Revoked;
                        count += 1;
                    }
                }
            }
        }

        if count > 0 {
            info!(
                "Expired {} API keys past rotation grace period",
                count
            );
        }

        Ok(count)
    }

    pub async fn get_expiring_keys(&self, days: i64) -> Vec<ApiKey> {
        let keys = self.keys.read().await;
        keys.values()
            .filter(|k| k.status == ApiKeyStatus::Active && k.is_expiring_soon(days))
            .cloned()
            .collect()
    }

    pub fn config(&self) -> &ApiKeyConfig {
        &self.config
    }
}

fn generate_api_key() -> (String, String) {
    let mut rng = rand::rng();

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let random_part: String = (0..API_KEY_LENGTH)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect();

    let secret = format!("{API_KEY_PREFIX}{random_part}");
    let hash = hash_api_key(&secret);

    (secret, hash)
}

fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());

    for _ in 0..API_KEY_HASH_ITERATIONS {
        let result = hasher.finalize_reset();
        hasher.update(result);
    }

    let result = hasher.finalize();
    hex::encode(result)
}

fn extract_prefix(key: &str) -> String {
    let stripped = key.strip_prefix(API_KEY_PREFIX).unwrap_or(key);
    format!("{API_KEY_PREFIX}{}...", &stripped[..8.min(stripped.len())])
}

fn matches_cidr(ip: &str, cidr: &str) -> bool {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return false;
    }

    let network = parts[0];
    let prefix_len: u8 = match parts[1].parse() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let ip_octets: Vec<u8> = match ip
        .split('.')
        .map(|s| s.parse())
        .collect::<Result<Vec<u8>, _>>()
    {
        Ok(o) if o.len() == 4 => o,
        _ => return false,
    };

    let network_octets: Vec<u8> = match network
        .split('.')
        .map(|s| s.parse())
        .collect::<Result<Vec<u8>, _>>()
    {
        Ok(o) if o.len() == 4 => o,
        _ => return false,
    };

    let ip_u32 = u32::from_be_bytes([ip_octets[0], ip_octets[1], ip_octets[2], ip_octets[3]]);
    let network_u32 = u32::from_be_bytes([
        network_octets[0],
        network_octets[1],
        network_octets[2],
        network_octets[3],
    ]);

    let mask = if prefix_len >= 32 {
        u32::MAX
    } else {
        u32::MAX << (32 - prefix_len)
    };

    (ip_u32 & mask) == (network_u32 & mask)
}

pub fn extract_api_key_from_header(header_value: &str) -> Option<&str> {
    if let Some(key) = header_value.strip_prefix("Bearer ") {
        return Some(key);
    }
    if let Some(key) = header_value.strip_prefix("bearer ") {
        return Some(key);
    }
    if let Some(key) = header_value.strip_prefix("ApiKey ") {
        return Some(key);
    }
    if let Some(key) = header_value.strip_prefix("api-key ") {
        return Some(key);
    }
    if header_value.starts_with(API_KEY_PREFIX) {
        return Some(header_value);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_validate_key() {
        let manager = ApiKeyManager::with_defaults();
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Test Key".into(),
            description: Some("A test API key".into()),
            scopes: vec![ApiKeyScope::Read, ApiKeyScope::Write],
            expires_in_days: Some(30),
            rate_limits: None,
            allowed_ips: None,
            allowed_origins: None,
            metadata: None,
        };

        let response = manager.create_key(user_id, request).await.expect("Create failed");

        assert!(response.secret.starts_with(API_KEY_PREFIX));
        assert_eq!(response.key.user_id, user_id);
        assert!(response.key.is_valid());

        let validated = manager
            .validate_key(&response.secret)
            .await
            .expect("Validate failed");
        assert!(validated.is_some());
        assert_eq!(validated.as_ref().map(|k| k.id), Some(response.key.id));
    }

    #[tokio::test]
    async fn test_key_scopes() {
        let manager = ApiKeyManager::with_defaults();
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Limited Key".into(),
            description: None,
            scopes: vec![ApiKeyScope::Read],
            expires_in_days: None,
            rate_limits: None,
            allowed_ips: None,
            allowed_origins: None,
            metadata: None,
        };

        let response = manager.create_key(user_id, request).await.expect("Create failed");

        assert!(response.key.has_scope(&ApiKeyScope::Read));
        assert!(!response.key.has_scope(&ApiKeyScope::Write));
        assert!(!response.key.has_scope(&ApiKeyScope::Admin));
    }

    #[tokio::test]
    async fn test_admin_scope_includes_all() {
        let manager = ApiKeyManager::with_defaults();
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Admin Key".into(),
            description: None,
            scopes: vec![ApiKeyScope::Admin],
            expires_in_days: None,
            rate_limits: None,
            allowed_ips: None,
            allowed_origins: None,
            metadata: None,
        };

        let response = manager.create_key(user_id, request).await.expect("Create failed");

        assert!(response.key.has_scope(&ApiKeyScope::Read));
        assert!(response.key.has_scope(&ApiKeyScope::Write));
        assert!(response.key.has_scope(&ApiKeyScope::Delete));
        assert!(response.key.has_scope(&ApiKeyScope::Admin));
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let manager = ApiKeyManager::with_defaults();
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Revokable Key".into(),
            description: None,
            scopes: vec![ApiKeyScope::Read],
            expires_in_days: None,
            rate_limits: None,
            allowed_ips: None,
            allowed_origins: None,
            metadata: None,
        };

        let response = manager.create_key(user_id, request).await.expect("Create failed");
        let key_id = response.key.id;

        manager.revoke_key(key_id).await.expect("Revoke failed");

        let validated = manager
            .validate_key(&response.secret)
            .await
            .expect("Validate failed");
        assert!(validated.is_none());
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let manager = ApiKeyManager::with_defaults();
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Rate Limited Key".into(),
            description: None,
            scopes: vec![ApiKeyScope::Read],
            expires_in_days: None,
            rate_limits: Some(RateLimitConfig {
                requests_per_minute: 2,
                requests_per_hour: 100,
                requests_per_day: 1000,
            }),
            allowed_ips: None,
            allowed_origins: None,
            metadata: None,
        };

        let response = manager.create_key(user_id, request).await.expect("Create failed");

        let (_, limited1) = manager
            .validate_and_check_rate_limit(&response.secret)
            .await
            .expect("Validate failed");
        assert!(!limited1);

        let (_, limited2) = manager
            .validate_and_check_rate_limit(&response.secret)
            .await
            .expect("Validate failed");
        assert!(!limited2);

        let (_, limited3) = manager
            .validate_and_check_rate_limit(&response.secret)
            .await
            .expect("Validate failed");
        assert!(limited3);
    }

    #[test]
    fn test_ip_cidr_matching() {
        assert!(matches_cidr("192.168.1.100", "192.168.1.0/24"));
        assert!(matches_cidr("192.168.1.1", "192.168.1.0/24"));
        assert!(!matches_cidr("192.168.2.1", "192.168.1.0/24"));
        assert!(matches_cidr("10.0.0.1", "10.0.0.0/8"));
    }

    #[test]
    fn test_extract_api_key_from_header() {
        assert_eq!(
            extract_api_key_from_header("Bearer gb_abc123"),
            Some("gb_abc123")
        );
        assert_eq!(
            extract_api_key_from_header("ApiKey gb_xyz789"),
            Some("gb_xyz789")
        );
        assert_eq!(
            extract_api_key_from_header("gb_direct_key"),
            Some("gb_direct_key")
        );
        assert_eq!(extract_api_key_from_header("Basic abc123"), None);
    }

    #[test]
    fn test_scope_inclusion() {
        assert!(ApiKeyScope::Admin.includes(&ApiKeyScope::Read));
        assert!(ApiKeyScope::Admin.includes(&ApiKeyScope::Write));
        assert!(ApiKeyScope::Write.includes(&ApiKeyScope::Read));
        assert!(!ApiKeyScope::Read.includes(&ApiKeyScope::Write));
    }

    #[test]
    fn test_origin_matching() {
        let key = ApiKey {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: "Test".into(),
            description: None,
            key_hash: "hash".into(),
            key_prefix: "gb_abc...".into(),
            scopes: HashSet::new(),
            status: ApiKeyStatus::Active,
            rate_limits: RateLimitConfig::default(),
            allowed_ips: None,
            allowed_origins: Some(vec!["*.example.com".into(), "https://specific.org".into()]),
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            rotated_from: None,
            rotation_deadline: None,
            metadata: HashMap::new(),
        };

        assert!(key.is_origin_allowed("api.example.com"));
        assert!(key.is_origin_allowed("sub.example.com"));
        assert!(key.is_origin_allowed("https://specific.org"));
        assert!(!key.is_origin_allowed("https://other.org"));
    }
}
