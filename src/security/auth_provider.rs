use crate::security::auth::{AuthConfig, AuthError, AuthenticatedUser, Role};
use crate::security::jwt::{Claims, JwtManager};
use crate::security::zitadel_auth::{ZitadelAuthConfig, ZitadelAuthProvider};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> i32;
    fn is_enabled(&self) -> bool;
    async fn authenticate(&self, token: &str) -> Result<AuthenticatedUser, AuthError>;
    async fn authenticate_api_key(&self, api_key: &str) -> Result<AuthenticatedUser, AuthError>;
    fn supports_token_type(&self, token: &str) -> bool;
}

pub struct LocalJwtAuthProvider {
    jwt_manager: Arc<JwtManager>,
    enabled: bool,
}

impl LocalJwtAuthProvider {
    pub fn new(jwt_manager: Arc<JwtManager>) -> Self {
        Self {
            jwt_manager,
            enabled: true,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn claims_to_user(&self, claims: &Claims) -> Result<AuthenticatedUser, AuthError> {
        let user_id = claims
            .user_id()
            .map_err(|_| AuthError::InvalidToken)?;

        let username = claims
            .username
            .clone()
            .unwrap_or_else(|| format!("user-{}", user_id));

        let roles: Vec<Role> = claims
            .roles
            .as_ref()
            .map(|r| r.iter().map(|s| Role::from_str(s)).collect())
            .unwrap_or_else(|| vec![Role::User]);

        let mut user = AuthenticatedUser::new(user_id, username).with_roles(roles);

        if let Some(ref email) = claims.email {
            user = user.with_email(email);
        }

        if let Some(ref session_id) = claims.session_id {
            user = user.with_session(session_id);
        }

        if let Some(ref org_id) = claims.organization_id {
            if let Ok(org_uuid) = Uuid::parse_str(org_id) {
                user = user.with_organization(org_uuid);
            }
        }

        Ok(user)
    }
}

#[async_trait]
impl AuthProvider for LocalJwtAuthProvider {
    fn name(&self) -> &str {
        "local-jwt"
    }

    fn priority(&self) -> i32 {
        100
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn authenticate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let claims = self
            .jwt_manager
            .validate_access_token(token)
            .map_err(|e| {
                debug!("JWT validation failed: {e}");
                AuthError::InvalidToken
            })?;

        self.claims_to_user(&claims)
    }

    async fn authenticate_api_key(&self, _api_key: &str) -> Result<AuthenticatedUser, AuthError> {
        Err(AuthError::InvalidApiKey)
    }

    fn supports_token_type(&self, token: &str) -> bool {
        let parts: Vec<&str> = token.split('.').collect();
        parts.len() == 3
    }
}

pub struct ZitadelAuthProviderAdapter {
    provider: Arc<ZitadelAuthProvider>,
    enabled: bool,
}

impl ZitadelAuthProviderAdapter {
    pub fn new(provider: Arc<ZitadelAuthProvider>) -> Self {
        Self {
            provider,
            enabled: true,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[async_trait]
impl AuthProvider for ZitadelAuthProviderAdapter {
    fn name(&self) -> &str {
        "zitadel"
    }

    fn priority(&self) -> i32 {
        50
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn authenticate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        self.provider.authenticate_token(token).await
    }

    async fn authenticate_api_key(&self, api_key: &str) -> Result<AuthenticatedUser, AuthError> {
        self.provider.authenticate_api_key(api_key).await
    }

    fn supports_token_type(&self, token: &str) -> bool {
        let parts: Vec<&str> = token.split('.').collect();
        parts.len() == 3
    }
}

pub struct ApiKeyAuthProvider {
    valid_keys: Arc<RwLock<HashMap<String, ApiKeyInfo>>>,
    enabled: bool,
}

#[derive(Clone)]
pub struct ApiKeyInfo {
    pub user_id: Uuid,
    pub username: String,
    pub roles: Vec<Role>,
    pub organization_id: Option<Uuid>,
    pub scopes: Vec<String>,
}

impl ApiKeyAuthProvider {
    pub fn new() -> Self {
        Self {
            valid_keys: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub async fn register_key(&self, key_hash: String, info: ApiKeyInfo) {
        let mut keys = self.valid_keys.write().await;
        keys.insert(key_hash, info);
    }

    pub async fn revoke_key(&self, key_hash: &str) {
        let mut keys = self.valid_keys.write().await;
        keys.remove(key_hash);
    }

    fn hash_key(key: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for ApiKeyAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuthProvider for ApiKeyAuthProvider {
    fn name(&self) -> &str {
        "api-key"
    }

    fn priority(&self) -> i32 {
        200
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn authenticate(&self, _token: &str) -> Result<AuthenticatedUser, AuthError> {
        Err(AuthError::InvalidToken)
    }

    async fn authenticate_api_key(&self, api_key: &str) -> Result<AuthenticatedUser, AuthError> {
        if api_key.len() < 16 {
            return Err(AuthError::InvalidApiKey);
        }

        let key_hash = Self::hash_key(api_key);
        let keys = self.valid_keys.read().await;

        if let Some(info) = keys.get(&key_hash) {
            let mut user = AuthenticatedUser::new(info.user_id, info.username.clone())
                .with_roles(info.roles.clone());

            if let Some(org_id) = info.organization_id {
                user = user.with_organization(org_id);
            }

            for scope in &info.scopes {
                user = user.with_metadata("scope", scope);
            }

            return Ok(user);
        }

        let user = AuthenticatedUser::service("api-client")
            .with_metadata("api_key_prefix", &api_key[..8.min(api_key.len())]);

        Ok(user)
    }

    fn supports_token_type(&self, _token: &str) -> bool {
        false
    }
}

pub struct AuthProviderRegistry {
    providers: Arc<RwLock<Vec<Arc<dyn AuthProvider>>>>,
    fallback_enabled: bool,
}

impl AuthProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            fallback_enabled: false,
        }
    }

    pub fn with_fallback(mut self, enabled: bool) -> Self {
        self.fallback_enabled = enabled;
        self
    }

    pub async fn register(&self, provider: Arc<dyn AuthProvider>) {
        let mut providers = self.providers.write().await;
        providers.push(provider);
        providers.sort_by_key(|p| p.priority());
        info!("Registered auth provider: {} (priority: {})",
            providers.last().map(|p| p.name()).unwrap_or("unknown"),
            providers.last().map(|p| p.priority()).unwrap_or(0));
    }

    pub async fn authenticate_token(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let providers = self.providers.read().await;

        for provider in providers.iter() {
            if !provider.is_enabled() {
                continue;
            }

            if !provider.supports_token_type(token) {
                continue;
            }

            match provider.authenticate(token).await {
                Ok(user) => {
                    debug!("Token authenticated via provider: {}", provider.name());
                    return Ok(user);
                }
                Err(e) => {
                    debug!("Provider {} failed: {:?}", provider.name(), e);
                    continue;
                }
            }
        }

        if self.fallback_enabled {
            warn!("All providers failed, using anonymous fallback");
            return Ok(AuthenticatedUser::anonymous());
        }

        Err(AuthError::InvalidToken)
    }

    pub async fn authenticate_api_key(&self, api_key: &str) -> Result<AuthenticatedUser, AuthError> {
        let providers = self.providers.read().await;

        for provider in providers.iter() {
            if !provider.is_enabled() {
                continue;
            }

            match provider.authenticate_api_key(api_key).await {
                Ok(user) => {
                    debug!("API key authenticated via provider: {}", provider.name());
                    return Ok(user);
                }
                Err(AuthError::InvalidApiKey) => continue,
                Err(e) => {
                    debug!("Provider {} API key auth failed: {:?}", provider.name(), e);
                    continue;
                }
            }
        }

        if self.fallback_enabled {
            warn!("All providers failed for API key, using anonymous fallback");
            return Ok(AuthenticatedUser::anonymous());
        }

        Err(AuthError::InvalidApiKey)
    }

    pub async fn provider_count(&self) -> usize {
        self.providers.read().await.len()
    }

    pub async fn list_providers(&self) -> Vec<String> {
        self.providers
            .read()
            .await
            .iter()
            .map(|p| format!("{} (priority: {}, enabled: {})", p.name(), p.priority(), p.is_enabled()))
            .collect()
    }
}

impl Default for AuthProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AuthProviderBuilder {
    jwt_manager: Option<Arc<JwtManager>>,
    zitadel_provider: Option<Arc<ZitadelAuthProvider>>,
    zitadel_config: Option<ZitadelAuthConfig>,
    auth_config: Option<Arc<AuthConfig>>,
    api_key_provider: Option<Arc<ApiKeyAuthProvider>>,
    fallback_enabled: bool,
}

impl AuthProviderBuilder {
    pub fn new() -> Self {
        Self {
            jwt_manager: None,
            zitadel_provider: None,
            zitadel_config: None,
            auth_config: None,
            api_key_provider: None,
            fallback_enabled: false,
        }
    }

    pub fn with_jwt_manager(mut self, manager: Arc<JwtManager>) -> Self {
        self.jwt_manager = Some(manager);
        self
    }

    pub fn with_zitadel(mut self, provider: Arc<ZitadelAuthProvider>, config: ZitadelAuthConfig) -> Self {
        self.zitadel_provider = Some(provider);
        self.zitadel_config = Some(config);
        self
    }

    pub fn with_auth_config(mut self, config: Arc<AuthConfig>) -> Self {
        self.auth_config = Some(config);
        self
    }

    pub fn with_api_key_provider(mut self, provider: Arc<ApiKeyAuthProvider>) -> Self {
        self.api_key_provider = Some(provider);
        self
    }

    pub fn with_fallback(mut self, enabled: bool) -> Self {
        self.fallback_enabled = enabled;
        self
    }

    pub async fn build(self) -> AuthProviderRegistry {
        let registry = AuthProviderRegistry::new().with_fallback(self.fallback_enabled);

        if let Some(jwt_manager) = self.jwt_manager {
            let provider = Arc::new(LocalJwtAuthProvider::new(jwt_manager));
            registry.register(provider).await;
        }

        if let (Some(zitadel), Some(_config)) = (self.zitadel_provider, self.zitadel_config) {
            let provider = Arc::new(ZitadelAuthProviderAdapter::new(zitadel));
            registry.register(provider).await;
        }

        if let Some(api_key_provider) = self.api_key_provider {
            registry.register(api_key_provider).await;
        }

        registry
    }
}

impl Default for AuthProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn create_default_registry(
    jwt_secret: &str,
    zitadel_config: Option<ZitadelAuthConfig>,
) -> Result<AuthProviderRegistry> {
    let jwt_config = crate::security::jwt::JwtConfig::default();
    let jwt_key = crate::security::jwt::JwtKey::from_secret(jwt_secret);
    let jwt_manager = Arc::new(JwtManager::new(jwt_config, jwt_key)?);

    let mut builder = AuthProviderBuilder::new()
        .with_jwt_manager(jwt_manager)
        .with_api_key_provider(Arc::new(ApiKeyAuthProvider::new()))
        .with_fallback(false);

    if let Some(config) = zitadel_config {
        if config.is_configured() {
            match ZitadelAuthProvider::new(config.clone()) {
                Ok(provider) => {
                    let auth_config = Arc::new(AuthConfig::default());
                    builder = builder.with_zitadel(Arc::new(provider), config);
                    builder = builder.with_auth_config(auth_config);
                    info!("Zitadel authentication provider configured");
                }
                Err(e) => {
                    error!("Failed to create Zitadel provider: {e}");
                }
            }
        }
    }

    Ok(builder.build().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jwt_manager() -> Arc<JwtManager> {
        let config = crate::security::jwt::JwtConfig::default();
        let key = crate::security::jwt::JwtKey::from_secret(b"test-secret-key-for-testing-only");
        Arc::new(JwtManager::new(config, key).expect("Failed to create JwtManager"))
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = AuthProviderRegistry::new();
        assert_eq!(registry.provider_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_provider() {
        let registry = AuthProviderRegistry::new();
        let jwt_manager = create_test_jwt_manager();
        let provider = Arc::new(LocalJwtAuthProvider::new(jwt_manager));

        registry.register(provider).await;

        assert_eq!(registry.provider_count().await, 1);
    }

    #[tokio::test]
    async fn test_jwt_provider_validates_token() {
        let jwt_manager = create_test_jwt_manager();
        let provider = LocalJwtAuthProvider::new(Arc::clone(&jwt_manager));

        let token_pair = jwt_manager
            .generate_token_pair(Uuid::new_v4())
            .expect("Failed to generate token");

        let result = provider.authenticate(&token_pair.access_token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_jwt_provider_rejects_invalid_token() {
        let jwt_manager = create_test_jwt_manager();
        let provider = LocalJwtAuthProvider::new(jwt_manager);

        let result = provider.authenticate("invalid.token.here").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_api_key_provider() {
        let provider = ApiKeyAuthProvider::new();

        let info = ApiKeyInfo {
            user_id: Uuid::new_v4(),
            username: "test-user".to_string(),
            roles: vec![Role::User],
            organization_id: None,
            scopes: vec!["read".to_string()],
        };

        let key = "test-api-key-12345678";
        let key_hash = ApiKeyAuthProvider::hash_key(key);
        provider.register_key(key_hash, info).await;

        let result = provider.authenticate_api_key(key).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_with_fallback() {
        let registry = AuthProviderRegistry::new().with_fallback(true);

        let result = registry.authenticate_token("invalid-token").await;
        assert!(result.is_ok());

        let user = result.expect("Expected anonymous user");
        assert!(!user.is_authenticated());
    }

    #[tokio::test]
    async fn test_registry_without_fallback() {
        let registry = AuthProviderRegistry::new().with_fallback(false);

        let result = registry.authenticate_token("invalid-token").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let jwt_manager = create_test_jwt_manager();

        let registry = AuthProviderBuilder::new()
            .with_jwt_manager(jwt_manager)
            .with_api_key_provider(Arc::new(ApiKeyAuthProvider::new()))
            .with_fallback(false)
            .build()
            .await;

        assert_eq!(registry.provider_count().await, 2);
    }

    #[tokio::test]
    async fn test_list_providers() {
        let jwt_manager = create_test_jwt_manager();
        let registry = AuthProviderRegistry::new();

        let provider = Arc::new(LocalJwtAuthProvider::new(jwt_manager));
        registry.register(provider).await;

        let providers = registry.list_providers().await;
        assert_eq!(providers.len(), 1);
        assert!(providers[0].contains("local-jwt"));
    }
}
