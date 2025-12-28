use crate::security::auth::{AuthConfig, AuthError, AuthenticatedUser, BotAccess, Permission, Role};
use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    http::{header, Request},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitadelAuthConfig {
    pub issuer_url: String,
    pub api_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub project_id: String,
    pub cache_ttl_secs: u64,
    pub introspect_tokens: bool,
}

impl Default for ZitadelAuthConfig {
    fn default() -> Self {
        Self {
            issuer_url: std::env::var("ZITADEL_ISSUER_URL")
                .unwrap_or_else(|_| "https://localhost:8080".to_string()),
            api_url: std::env::var("ZITADEL_API_URL")
                .unwrap_or_else(|_| "https://localhost:8080".to_string()),
            client_id: std::env::var("ZITADEL_CLIENT_ID").unwrap_or_default(),
            client_secret: std::env::var("ZITADEL_CLIENT_SECRET").unwrap_or_default(),
            project_id: std::env::var("ZITADEL_PROJECT_ID").unwrap_or_default(),
            cache_ttl_secs: 300,
            introspect_tokens: true,
        }
    }
}

impl ZitadelAuthConfig {
    pub fn new(issuer_url: &str, api_url: &str, client_id: &str, client_secret: &str) -> Self {
        Self {
            issuer_url: issuer_url.to_string(),
            api_url: api_url.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            project_id: String::new(),
            cache_ttl_secs: 300,
            introspect_tokens: true,
        }
    }

    pub fn with_project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = project_id.into();
        self
    }

    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl_secs = ttl_secs;
        self
    }

    pub fn without_introspection(mut self) -> Self {
        self.introspect_tokens = false;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZitadelUser {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub organization_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ZitadelUser {
    pub fn to_authenticated_user(&self) -> Result<AuthenticatedUser, AuthError> {
        let user_id = Uuid::parse_str(&self.id).map_err(|_| {
            AuthError::InternalError(format!("Invalid user ID format: {}", self.id))
        })?;

        let username = if !self.username.is_empty() {
            self.username.clone()
        } else {
            self.email.clone().unwrap_or_else(|| self.id.clone())
        };

        let roles: Vec<Role> = self
            .roles
            .iter()
            .map(|r| map_zitadel_role_to_role(r))
            .collect();

        let roles = if roles.is_empty() {
            vec![Role::User]
        } else {
            roles
        };

        let mut user = AuthenticatedUser::new(user_id, username)
            .with_roles(roles);

        if let Some(ref email) = self.email {
            user = user.with_email(email);
        }

        if let Some(ref org_id) = self.organization_id {
            if let Ok(org_uuid) = Uuid::parse_str(org_id) {
                user = user.with_organization(org_uuid);
            }
        }

        for (key, value) in &self.metadata {
            user = user.with_metadata(key, value);
        }

        Ok(user)
    }
}

fn map_zitadel_role_to_role(zitadel_role: &str) -> Role {
    let role_lower = zitadel_role.to_lowercase();

    if role_lower.contains("super") || role_lower.contains("root") {
        Role::SuperAdmin
    } else if role_lower.contains("admin") {
        Role::Admin
    } else if role_lower.contains("moderator") || role_lower.contains("mod") {
        Role::Moderator
    } else if role_lower.contains("bot_owner") || role_lower.contains("owner") {
        Role::BotOwner
    } else if role_lower.contains("bot_operator") || role_lower.contains("operator") {
        Role::BotOperator
    } else if role_lower.contains("bot_viewer") || role_lower.contains("viewer") {
        Role::BotViewer
    } else if role_lower.contains("service") {
        Role::Service
    } else if role_lower.contains("bot") && !role_lower.contains("_") {
        Role::Bot
    } else if role_lower.contains("user") || !role_lower.is_empty() {
        Role::User
    } else {
        Role::Anonymous
    }
}

#[derive(Debug, Clone)]
struct CachedUser {
    user: AuthenticatedUser,
    expires_at: i64,
}

pub struct ZitadelAuthProvider {
    config: ZitadelAuthConfig,
    http_client: reqwest::Client,
    user_cache: Arc<RwLock<HashMap<String, CachedUser>>>,
    service_token: Arc<RwLock<Option<ServiceToken>>>,
}

#[derive(Debug, Clone)]
struct ServiceToken {
    access_token: String,
    expires_at: i64,
}

impl ZitadelAuthProvider {
    pub fn new(config: ZitadelAuthConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(
                std::env::var("ZITADEL_SKIP_TLS_VERIFY")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
            )
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            http_client,
            user_cache: Arc::new(RwLock::new(HashMap::new())),
            service_token: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn authenticate_request(
        &self,
        request: &Request<Body>,
        auth_config: &AuthConfig,
    ) -> Result<AuthenticatedUser, AuthError> {
        if let Some(token) = self.extract_bearer_token(request, auth_config) {
            return self.authenticate_token(&token).await;
        }

        if let Some(api_key) = self.extract_api_key(request, auth_config) {
            return self.authenticate_api_key(&api_key).await;
        }

        Err(AuthError::MissingToken)
    }

    fn extract_bearer_token(&self, request: &Request<Body>, config: &AuthConfig) -> Option<String> {
        request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|auth| auth.strip_prefix(&config.bearer_prefix))
            .map(|s| s.to_string())
    }

    fn extract_api_key(&self, request: &Request<Body>, config: &AuthConfig) -> Option<String> {
        request
            .headers()
            .get(&config.api_key_header)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }

    pub async fn authenticate_token(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        if let Some(cached) = self.get_cached_user(token).await {
            return Ok(cached);
        }

        let user = if self.config.introspect_tokens {
            self.introspect_and_get_user(token).await?
        } else {
            self.decode_jwt_user(token)?
        };

        self.cache_user(token, &user).await;

        Ok(user)
    }

    pub async fn authenticate_api_key(&self, api_key: &str) -> Result<AuthenticatedUser, AuthError> {
        if api_key.len() < 16 {
            return Err(AuthError::InvalidApiKey);
        }

        if let Some(cached) = self.get_cached_user(api_key).await {
            return Ok(cached);
        }

        let user = self.validate_api_key_with_zitadel(api_key).await?;

        self.cache_user(api_key, &user).await;

        Ok(user)
    }

    async fn introspect_and_get_user(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let introspect_url = format!("{}/oauth/v2/introspect", self.config.api_url);

        let params = [
            ("token", token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .http_client
            .post(&introspect_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                error!("Token introspection request failed: {}", e);
                AuthError::InternalError("Authentication service unavailable".to_string())
            })?;

        if !response.status().is_success() {
            warn!("Token introspection failed with status: {}", response.status());
            return Err(AuthError::InvalidToken);
        }

        let introspection: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse introspection response: {}", e);
            AuthError::InternalError("Invalid authentication response".to_string())
        })?;

        let active = introspection
            .get("active")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !active {
            return Err(AuthError::ExpiredToken);
        }

        let user_id = introspection
            .get("sub")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidToken)?;

        let username = introspection
            .get("username")
            .or_else(|| introspection.get("preferred_username"))
            .and_then(|v| v.as_str())
            .unwrap_or(user_id);

        let email = introspection
            .get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let roles: Vec<String> = introspection
            .get("roles")
            .or_else(|| {
                introspection
                    .get(&format!("urn:zitadel:iam:org:project:{}:roles", self.config.project_id))
            })
            .and_then(|v| v.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default();

        let zitadel_user = ZitadelUser {
            id: user_id.to_string(),
            username: username.to_string(),
            email,
            email_verified: introspection
                .get("email_verified")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            first_name: introspection
                .get("given_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            last_name: introspection
                .get("family_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            display_name: introspection
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            roles,
            organization_id: introspection
                .get("urn:zitadel:iam:user:resourceowner:id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            metadata: HashMap::new(),
        };

        zitadel_user.to_authenticated_user()
    }

    fn decode_jwt_user(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::InvalidToken);
        }

        let payload = parts[1];
        let decoded = base64_url_decode(payload).map_err(|_| AuthError::InvalidToken)?;

        let claims: serde_json::Value =
            serde_json::from_slice(&decoded).map_err(|_| AuthError::InvalidToken)?;

        let user_id = claims
            .get("sub")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::InvalidToken)?;

        let username = claims
            .get("preferred_username")
            .or_else(|| claims.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or(user_id);

        let exp = claims
            .get("exp")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if exp > 0 && exp < chrono::Utc::now().timestamp() {
            return Err(AuthError::ExpiredToken);
        }

        let roles: Vec<String> = claims
            .get("roles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let zitadel_user = ZitadelUser {
            id: user_id.to_string(),
            username: username.to_string(),
            email: claims
                .get("email")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            email_verified: claims
                .get("email_verified")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            first_name: claims
                .get("given_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            last_name: claims
                .get("family_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            display_name: claims
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            roles,
            organization_id: None,
            metadata: HashMap::new(),
        };

        zitadel_user.to_authenticated_user()
    }

    async fn validate_api_key_with_zitadel(
        &self,
        api_key: &str,
    ) -> Result<AuthenticatedUser, AuthError> {
        let service_token = self.get_service_token().await?;

        let url = format!("{}/v2/users/_search", self.config.api_url);

        let body = serde_json::json!({
            "queries": [{
                "typeQuery": {
                    "type": "TYPE_MACHINE"
                }
            }],
            "limit": 1
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&service_token)
            .json(&body)
            .header("x-zitadel-api-key", api_key)
            .send()
            .await
            .map_err(|e| {
                error!("API key validation request failed: {}", e);
                AuthError::InternalError("Authentication service unavailable".to_string())
            })?;

        if !response.status().is_success() {
            return Err(AuthError::InvalidApiKey);
        }

        Ok(AuthenticatedUser::service("api-key-user")
            .with_metadata("api_key_prefix", &api_key[..8.min(api_key.len())]))
    }

    async fn get_service_token(&self) -> Result<String, AuthError> {
        {
            let token = self.service_token.read().await;
            if let Some(ref t) = *token {
                if t.expires_at > chrono::Utc::now().timestamp() {
                    return Ok(t.access_token.clone());
                }
            }
        }

        let token_url = format!("{}/oauth/v2/token", self.config.api_url);

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("scope", "openid profile email"),
        ];

        let response = self
            .http_client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                error!("Service token request failed: {}", e);
                AuthError::InternalError("Authentication service unavailable".to_string())
            })?;

        if !response.status().is_success() {
            return Err(AuthError::InternalError(
                "Failed to obtain service token".to_string(),
            ));
        }

        let token_data: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse token response: {}", e);
            AuthError::InternalError("Invalid token response".to_string())
        })?;

        let access_token = token_data
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::InternalError("No access token in response".to_string()))?
            .to_string();

        let expires_in = token_data
            .get("expires_in")
            .and_then(|v| v.as_i64())
            .unwrap_or(3600);

        let expires_at = chrono::Utc::now().timestamp() + expires_in - 60;

        {
            let mut token = self.service_token.write().await;
            *token = Some(ServiceToken {
                access_token: access_token.clone(),
                expires_at,
            });
        }

        Ok(access_token)
    }

    async fn get_cached_user(&self, key: &str) -> Option<AuthenticatedUser> {
        let cache = self.user_cache.read().await;
        cache.get(key).and_then(|cached| {
            if cached.expires_at > chrono::Utc::now().timestamp() {
                Some(cached.user.clone())
            } else {
                None
            }
        })
    }

    async fn cache_user(&self, key: &str, user: &AuthenticatedUser) {
        let expires_at = chrono::Utc::now().timestamp() + self.config.cache_ttl_secs as i64;
        let cached = CachedUser {
            user: user.clone(),
            expires_at,
        };

        let mut cache = self.user_cache.write().await;
        cache.insert(key.to_string(), cached);
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.user_cache.write().await;
        cache.clear();
    }

    pub async fn invalidate_user(&self, token: &str) {
        let mut cache = self.user_cache.write().await;
        cache.remove(token);
    }

    pub async fn get_user_bot_access(
        &self,
        user_id: &str,
    ) -> Result<Vec<BotAccess>, AuthError> {
        let service_token = self.get_service_token().await?;

        let url = format!(
            "{}/v2/users/{}/grants",
            self.config.api_url, user_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&service_token)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get user grants: {}", e);
                AuthError::InternalError("Failed to fetch user permissions".to_string())
            })?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let grants: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse grants response: {}", e);
            AuthError::InternalError("Invalid grants response".to_string())
        })?;

        let mut bot_access = Vec::new();

        if let Some(results) = grants.get("result").and_then(|r| r.as_array()) {
            for grant in results {
                if let Some(roles) = grant.get("roles").and_then(|r| r.as_array()) {
                    for role_value in roles {
                        if let Some(role_str) = role_value.as_str() {
                            if role_str.starts_with("bot:") {
                                let parts: Vec<&str> = role_str.splitn(3, ':').collect();
                                if parts.len() >= 2 {
                                    if let Ok(bot_id) = Uuid::parse_str(parts[1]) {
                                        let role = if parts.len() >= 3 {
                                            map_zitadel_role_to_role(parts[2])
                                        } else {
                                            Role::BotViewer
                                        };

                                        bot_access.push(BotAccess::new(bot_id, role));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(bot_access)
    }

    pub async fn check_bot_permission(
        &self,
        user_id: &str,
        bot_id: &Uuid,
        permission: &Permission,
    ) -> Result<bool, AuthError> {
        let bot_access = self.get_user_bot_access(user_id).await?;

        for access in bot_access {
            if &access.bot_id == bot_id && access.role.has_permission(permission) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>, String> {
    let input = input.replace('-', "+").replace('_', "/");

    let padding = match input.len() % 4 {
        0 => "",
        2 => "==",
        3 => "=",
        _ => return Err("Invalid base64 length".to_string()),
    };

    let padded = format!("{}{}", input, padding);

    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(&padded)
        .map_err(|e| format!("Base64 decode error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zitadel_auth_config_default() {
        let config = ZitadelAuthConfig::default();
        assert_eq!(config.cache_ttl_secs, 300);
        assert!(config.introspect_tokens);
    }

    #[test]
    fn test_zitadel_auth_config_builder() {
        let config = ZitadelAuthConfig::new(
            "https://auth.example.com",
            "https://api.example.com",
            "client123",
            "secret456",
        )
        .with_project_id("project789")
        .with_cache_ttl(600)
        .without_introspection();

        assert_eq!(config.issuer_url, "https://auth.example.com");
        assert_eq!(config.api_url, "https://api.example.com");
        assert_eq!(config.client_id, "client123");
        assert_eq!(config.project_id, "project789");
        assert_eq!(config.cache_ttl_secs, 600);
        assert!(!config.introspect_tokens);
    }

    #[test]
    fn test_map_zitadel_role_to_role() {
        assert_eq!(map_zitadel_role_to_role("superadmin"), Role::SuperAdmin);
        assert_eq!(map_zitadel_role_to_role("admin"), Role::Admin);
        assert_eq!(map_zitadel_role_to_role("ADMIN"), Role::Admin);
        assert_eq!(map_zitadel_role_to_role("moderator"), Role::Moderator);
        assert_eq!(map_zitadel_role_to_role("bot_owner"), Role::BotOwner);
        assert_eq!(map_zitadel_role_to_role("bot_operator"), Role::BotOperator);
        assert_eq!(map_zitadel_role_to_role("bot_viewer"), Role::BotViewer);
        assert_eq!(map_zitadel_role_to_role("user"), Role::User);
        assert_eq!(map_zitadel_role_to_role("custom_role"), Role::User);
        assert_eq!(map_zitadel_role_to_role(""), Role::Anonymous);
    }

    #[test]
    fn test_zitadel_user_to_authenticated_user() {
        let zitadel_user = ZitadelUser {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            username: "testuser".to_string(),
            email: Some("test@example.com".to_string()),
            email_verified: true,
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            display_name: Some("Test User".to_string()),
            roles: vec!["admin".to_string(), "bot_owner".to_string()],
            organization_id: Some("660e8400-e29b-41d4-a716-446655440001".to_string()),
            metadata: HashMap::new(),
        };

        let auth_user = zitadel_user.to_authenticated_user().unwrap();

        assert_eq!(auth_user.username, "testuser");
        assert_eq!(auth_user.email, Some("test@example.com".to_string()));
        assert!(auth_user.has_role(&Role::Admin));
        assert!(auth_user.has_role(&Role::BotOwner));
        assert!(auth_user.is_admin());
    }

    #[test]
    fn test_zitadel_user_invalid_uuid() {
        let zitadel_user = ZitadelUser {
            id: "invalid-uuid".to_string(),
            username: "testuser".to_string(),
            email: None,
            email_verified: false,
            first_name: None,
            last_name: None,
            display_name: None,
            roles: vec![],
            organization_id: None,
            metadata: HashMap::new(),
        };

        assert!(zitadel_user.to_authenticated_user().is_err());
    }

    #[test]
    fn test_base64_url_decode() {
        let encoded = "SGVsbG8gV29ybGQ";
        let decoded = base64_url_decode(encoded).unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello World");
    }

    #[test]
    fn test_base64_url_decode_with_special_chars() {
        let encoded = "PDw_Pz4-";
        let result = base64_url_decode(encoded);
        assert!(result.is_ok());
    }
}
