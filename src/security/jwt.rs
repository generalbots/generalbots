use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
    pub algorithm: JwtAlgorithm,
    pub leeway_seconds: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            issuer: "general-bots".into(),
            audience: "general-bots-api".into(),
            access_token_expiry_minutes: 15,
            refresh_token_expiry_days: 7,
            algorithm: JwtAlgorithm::HS256,
            leeway_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
    RS256,
    RS384,
    RS512,
    ES256,
    ES384,
}

impl JwtAlgorithm {
    pub fn to_jsonwebtoken(&self) -> Algorithm {
        match self {
            Self::HS256 => Algorithm::HS256,
            Self::HS384 => Algorithm::HS384,
            Self::HS512 => Algorithm::HS512,
            Self::RS256 => Algorithm::RS256,
            Self::RS384 => Algorithm::RS384,
            Self::RS512 => Algorithm::RS512,
            Self::ES256 => Algorithm::ES256,
            Self::ES384 => Algorithm::ES384,
        }
    }

    pub fn is_symmetric(&self) -> bool {
        matches!(self, Self::HS256 | Self::HS384 | Self::HS512)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    Access,
    Refresh,
    IdToken,
}

impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Access => "access",
            Self::Refresh => "refresh",
            Self::IdToken => "id_token",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
    pub jti: String,
    #[serde(rename = "type")]
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

impl Claims {
    pub fn new(
        user_id: Uuid,
        issuer: &str,
        audience: &str,
        token_type: TokenType,
        expiry: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id.to_string(),
            iss: issuer.to_string(),
            aud: audience.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: token_type.as_str().to_string(),
            email: None,
            username: None,
            roles: None,
            permissions: None,
            session_id: None,
            organization_id: None,
            device_id: None,
        }
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn with_username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(roles);
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_organization_id(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn user_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|e| anyhow!("Invalid user ID in claims: {e}"))
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    pub fn is_access_token(&self) -> bool {
        self.token_type == TokenType::Access.as_str()
    }

    pub fn is_refresh_token(&self) -> bool {
        self.token_type == TokenType::Refresh.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Clone)]
pub enum JwtKey {
    Symmetric(Vec<u8>),
    RsaPrivate(Vec<u8>),
    RsaPublic(Vec<u8>),
    EcPrivate(Vec<u8>),
    EcPublic(Vec<u8>),
}

impl JwtKey {
    pub fn from_secret(secret: &str) -> Self {
        Self::Symmetric(secret.as_bytes().to_vec())
    }

    pub fn from_rsa_pem(private_pem: &str, public_pem: &str) -> Result<(Self, Self)> {
        Ok((
            Self::RsaPrivate(private_pem.as_bytes().to_vec()),
            Self::RsaPublic(public_pem.as_bytes().to_vec()),
        ))
    }

    pub fn from_ec_pem(private_pem: &str, public_pem: &str) -> Result<(Self, Self)> {
        Ok((
            Self::EcPrivate(private_pem.as_bytes().to_vec()),
            Self::EcPublic(public_pem.as_bytes().to_vec()),
        ))
    }
}

pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    blacklist: Arc<RwLock<HashSet<String>>>,
}

impl JwtManager {
    pub fn new(config: JwtConfig, key: JwtKey) -> Result<Self> {
        let (encoding_key, decoding_key) = match (&config.algorithm, key) {
            (JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512, JwtKey::Symmetric(secret)) => {
                (
                    EncodingKey::from_secret(&secret),
                    DecodingKey::from_secret(&secret),
                )
            }
            (JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512, JwtKey::RsaPrivate(pem)) => {
                let encoding = EncodingKey::from_rsa_pem(&pem)
                    .map_err(|e| anyhow!("Invalid RSA private key: {e}"))?;
                let decoding = DecodingKey::from_rsa_pem(&pem)
                    .map_err(|e| anyhow!("Invalid RSA key for decoding: {e}"))?;
                (encoding, decoding)
            }
            (JwtAlgorithm::ES256 | JwtAlgorithm::ES384, JwtKey::EcPrivate(pem)) => {
                let encoding = EncodingKey::from_ec_pem(&pem)
                    .map_err(|e| anyhow!("Invalid EC private key: {e}"))?;
                let decoding = DecodingKey::from_ec_pem(&pem)
                    .map_err(|e| anyhow!("Invalid EC key for decoding: {e}"))?;
                (encoding, decoding)
            }
            _ => return Err(anyhow!("Key type does not match algorithm")),
        };

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            blacklist: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    pub fn with_separate_keys(
        config: JwtConfig,
        signing_key: JwtKey,
        verification_key: JwtKey,
    ) -> Result<Self> {
        let encoding_key = match (&config.algorithm, signing_key) {
            (JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512, JwtKey::Symmetric(secret)) => {
                EncodingKey::from_secret(&secret)
            }
            (JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512, JwtKey::RsaPrivate(pem)) => {
                EncodingKey::from_rsa_pem(&pem)
                    .map_err(|e| anyhow!("Invalid RSA private key: {e}"))?
            }
            (JwtAlgorithm::ES256 | JwtAlgorithm::ES384, JwtKey::EcPrivate(pem)) => {
                EncodingKey::from_ec_pem(&pem)
                    .map_err(|e| anyhow!("Invalid EC private key: {e}"))?
            }
            _ => return Err(anyhow!("Signing key type does not match algorithm")),
        };

        let decoding_key = match (&config.algorithm, verification_key) {
            (JwtAlgorithm::HS256 | JwtAlgorithm::HS384 | JwtAlgorithm::HS512, JwtKey::Symmetric(secret)) => {
                DecodingKey::from_secret(&secret)
            }
            (JwtAlgorithm::RS256 | JwtAlgorithm::RS384 | JwtAlgorithm::RS512, JwtKey::RsaPublic(pem)) => {
                DecodingKey::from_rsa_pem(&pem)
                    .map_err(|e| anyhow!("Invalid RSA public key: {e}"))?
            }
            (JwtAlgorithm::ES256 | JwtAlgorithm::ES384, JwtKey::EcPublic(pem)) => {
                DecodingKey::from_ec_pem(&pem)
                    .map_err(|e| anyhow!("Invalid EC public key: {e}"))?
            }
            _ => return Err(anyhow!("Verification key type does not match algorithm")),
        };

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            blacklist: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    pub fn from_secret(secret: &str) -> Result<Self> {
        if secret.len() < 32 {
            return Err(anyhow!("JWT secret must be at least 32 characters"));
        }
        Self::new(JwtConfig::default(), JwtKey::from_secret(secret))
    }

    pub fn generate_access_token(&self, claims: Claims) -> Result<String> {
        let header = Header::new(self.config.algorithm.to_jsonwebtoken());
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to encode access token: {e}"))
    }

    pub fn generate_refresh_token(&self, claims: Claims) -> Result<String> {
        let header = Header::new(self.config.algorithm.to_jsonwebtoken());
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Failed to encode refresh token: {e}"))
    }

    pub fn generate_token_pair(&self, user_id: Uuid) -> Result<TokenPair> {
        let now = Utc::now();
        let access_expiry = now + Duration::minutes(self.config.access_token_expiry_minutes);
        let refresh_expiry = now + Duration::days(self.config.refresh_token_expiry_days);

        let access_claims = Claims::new(
            user_id,
            &self.config.issuer,
            &self.config.audience,
            TokenType::Access,
            access_expiry,
        );

        let refresh_claims = Claims::new(
            user_id,
            &self.config.issuer,
            &self.config.audience,
            TokenType::Refresh,
            refresh_expiry,
        );

        let access_token = self.generate_access_token(access_claims)?;
        let refresh_token = self.generate_refresh_token(refresh_claims)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".into(),
            expires_in: self.config.access_token_expiry_minutes * 60,
            refresh_expires_in: self.config.refresh_token_expiry_days * 24 * 60 * 60,
            id_token: None,
            scope: None,
        })
    }

    pub fn generate_token_pair_with_claims(
        &self,
        user_id: Uuid,
        email: Option<String>,
        username: Option<String>,
        roles: Option<Vec<String>>,
        session_id: Option<String>,
    ) -> Result<TokenPair> {
        let now = Utc::now();
        let access_expiry = now + Duration::minutes(self.config.access_token_expiry_minutes);
        let refresh_expiry = now + Duration::days(self.config.refresh_token_expiry_days);

        let mut access_claims = Claims::new(
            user_id,
            &self.config.issuer,
            &self.config.audience,
            TokenType::Access,
            access_expiry,
        );

        if let Some(e) = email.clone() {
            access_claims = access_claims.with_email(e);
        }
        if let Some(u) = username.clone() {
            access_claims = access_claims.with_username(u);
        }
        if let Some(r) = roles.clone() {
            access_claims = access_claims.with_roles(r);
        }
        if let Some(s) = session_id.clone() {
            access_claims = access_claims.with_session_id(s);
        }

        let mut refresh_claims = Claims::new(
            user_id,
            &self.config.issuer,
            &self.config.audience,
            TokenType::Refresh,
            refresh_expiry,
        );

        if let Some(s) = session_id {
            refresh_claims = refresh_claims.with_session_id(s);
        }

        let access_token = self.generate_access_token(access_claims)?;
        let refresh_token = self.generate_refresh_token(refresh_claims)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".into(),
            expires_in: self.config.access_token_expiry_minutes * 60,
            refresh_expires_in: self.config.refresh_token_expiry_days * 24 * 60 * 60,
            id_token: None,
            scope: None,
        })
    }

    pub fn validate_token(&self, token: &str) -> Result<TokenData<Claims>> {
        let mut validation = Validation::new(self.config.algorithm.to_jsonwebtoken());
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.leeway = self.config.leeway_seconds;

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| anyhow!("Token validation failed: {e}"))
    }

    pub async fn validate_token_with_blacklist(&self, token: &str) -> Result<TokenData<Claims>> {
        let token_data = self.validate_token(token)?;

        let blacklist = self.blacklist.read().await;
        if blacklist.contains(&token_data.claims.jti) {
            return Err(anyhow!("Token has been revoked"));
        }

        Ok(token_data)
    }

    pub fn validate_access_token(&self, token: &str) -> Result<Claims> {
        let token_data = self.validate_token(token)?;

        if !token_data.claims.is_access_token() {
            return Err(anyhow!("Token is not an access token"));
        }

        Ok(token_data.claims)
    }

    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims> {
        let token_data = self.validate_token(token)?;

        if !token_data.claims.is_refresh_token() {
            return Err(anyhow!("Token is not a refresh token"));
        }

        Ok(token_data.claims)
    }

    pub async fn refresh_tokens(&self, refresh_token: &str) -> Result<TokenPair> {
        let claims = self.validate_refresh_token(refresh_token)?;

        {
            let blacklist = self.blacklist.read().await;
            if blacklist.contains(&claims.jti) {
                return Err(anyhow!("Refresh token has been revoked"));
            }
        }

        let user_id = claims.user_id()?;

        self.revoke_token(&claims.jti).await?;

        let new_pair = self.generate_token_pair_with_claims(
            user_id,
            claims.email,
            claims.username,
            claims.roles,
            claims.session_id,
        )?;

        debug!("Refreshed tokens for user {user_id}");
        Ok(new_pair)
    }

    pub async fn revoke_token(&self, jti: &str) -> Result<()> {
        let mut blacklist = self.blacklist.write().await;
        blacklist.insert(jti.to_string());
        debug!("Revoked token {jti}");
        Ok(())
    }

    pub async fn revoke_by_token(&self, token: &str) -> Result<()> {
        let token_data = self.validate_token(token)?;
        self.revoke_token(&token_data.claims.jti).await
    }

    pub async fn is_revoked(&self, jti: &str) -> bool {
        let blacklist = self.blacklist.read().await;
        blacklist.contains(jti)
    }

    pub async fn cleanup_blacklist(&self, _expired_before: DateTime<Utc>) -> usize {
        let mut blacklist = self.blacklist.write().await;
        let initial_count = blacklist.len();
        blacklist.clear();
        let removed = initial_count;
        if removed > 0 {
            info!("Cleaned up {removed} entries from token blacklist");
        }
        removed
    }

    pub fn decode_without_validation(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(self.config.algorithm.to_jsonwebtoken());
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.validate_aud = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| anyhow!("Failed to decode token: {e}"))?;

        Ok(token_data.claims)
    }

    pub fn config(&self) -> &JwtConfig {
        &self.config
    }
}

pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ").or_else(|| auth_header.strip_prefix("bearer "))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jwk {
    pub kty: String,
    pub use_: Option<String>,
    pub kid: Option<String>,
    pub alg: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub crv: Option<String>,
}

impl JwkSet {
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    pub fn add_key(&mut self, key: Jwk) {
        self.keys.push(key);
    }
}

impl Default for JwkSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenIntrospectionResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

impl TokenIntrospectionResponse {
    pub fn inactive() -> Self {
        Self {
            active: false,
            scope: None,
            client_id: None,
            username: None,
            token_type: None,
            exp: None,
            iat: None,
            nbf: None,
            sub: None,
            aud: None,
            iss: None,
            jti: None,
        }
    }

    pub fn from_claims(claims: &Claims, active: bool) -> Self {
        Self {
            active,
            scope: None,
            client_id: None,
            username: claims.username.clone(),
            token_type: Some(claims.token_type.clone()),
            exp: Some(claims.exp),
            iat: Some(claims.iat),
            nbf: Some(claims.nbf),
            sub: Some(claims.sub.clone()),
            aud: Some(claims.aud.clone()),
            iss: Some(claims.iss.clone()),
            jti: Some(claims.jti.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> JwtManager {
        JwtManager::from_secret("this-is-a-very-long-secret-key-for-testing-purposes-only")
            .expect("Failed to create manager")
    }

    #[test]
    fn test_generate_token_pair() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
    }

    #[test]
    fn test_validate_access_token() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");
        let claims = manager
            .validate_access_token(&pair.access_token)
            .expect("Validation failed");

        assert_eq!(claims.user_id().expect("Invalid user ID"), user_id);
        assert!(claims.is_access_token());
    }

    #[test]
    fn test_validate_refresh_token() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");
        let claims = manager
            .validate_refresh_token(&pair.refresh_token)
            .expect("Validation failed");

        assert_eq!(claims.user_id().expect("Invalid user ID"), user_id);
        assert!(claims.is_refresh_token());
    }

    #[test]
    fn test_token_with_claims() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager
            .generate_token_pair_with_claims(
                user_id,
                Some("test@example.com".into()),
                Some("testuser".into()),
                Some(vec!["admin".into(), "user".into()]),
                Some("session-123".into()),
            )
            .expect("Failed to generate");

        let claims = manager
            .validate_access_token(&pair.access_token)
            .expect("Validation failed");

        assert_eq!(claims.email, Some("test@example.com".into()));
        assert_eq!(claims.username, Some("testuser".into()));
        assert_eq!(claims.roles, Some(vec!["admin".into(), "user".into()]));
        assert_eq!(claims.session_id, Some("session-123".into()));
    }

    #[test]
    fn test_invalid_token() {
        let manager = create_test_manager();
        let result = manager.validate_token("invalid.token.here");

        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_token_type() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");

        let result = manager.validate_refresh_token(&pair.access_token);
        assert!(result.is_err());

        let result = manager.validate_access_token(&pair.refresh_token);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager.generate_token_pair(user_id).expect("Failed to generate");

        let token_data = manager
            .validate_token(&pair.access_token)
            .expect("Validation failed");

        manager
            .revoke_token(&token_data.claims.jti)
            .await
            .expect("Revoke failed");

        let result = manager
            .validate_token_with_blacklist(&pair.access_token)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_tokens() {
        let manager = create_test_manager();
        let user_id = Uuid::new_v4();

        let pair = manager
            .generate_token_pair_with_claims(
                user_id,
                Some("test@example.com".into()),
                Some("testuser".into()),
                None,
                None,
            )
            .expect("Failed to generate");

        let new_pair = manager
            .refresh_tokens(&pair.refresh_token)
            .await
            .expect("Refresh failed");

        assert_ne!(new_pair.access_token, pair.access_token);
        assert_ne!(new_pair.refresh_token, pair.refresh_token);

        let claims = manager
            .validate_access_token(&new_pair.access_token)
            .expect("Validation failed");
        assert_eq!(claims.email, Some("test@example.com".into()));
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer abc123"),
            Some("abc123")
        );
        assert_eq!(
            extract_bearer_token("bearer abc123"),
            Some("abc123")
        );
        assert_eq!(extract_bearer_token("Basic abc123"), None);
    }

    #[test]
    fn test_claims_builder() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            "issuer",
            "audience",
            TokenType::Access,
            Utc::now() + Duration::hours(1),
        )
        .with_email("test@example.com".into())
        .with_username("testuser".into())
        .with_roles(vec!["admin".into()])
        .with_organization_id("org-123".into());

        assert_eq!(claims.email, Some("test@example.com".into()));
        assert_eq!(claims.username, Some("testuser".into()));
        assert_eq!(claims.roles, Some(vec!["admin".into()]));
        assert_eq!(claims.organization_id, Some("org-123".into()));
    }

    #[test]
    fn test_token_type() {
        assert_eq!(TokenType::Access.as_str(), "access");
        assert_eq!(TokenType::Refresh.as_str(), "refresh");
        assert_eq!(TokenType::IdToken.as_str(), "id_token");
    }

    #[test]
    fn test_jwt_algorithm() {
        assert!(JwtAlgorithm::HS256.is_symmetric());
        assert!(JwtAlgorithm::HS384.is_symmetric());
        assert!(JwtAlgorithm::HS512.is_symmetric());
        assert!(!JwtAlgorithm::RS256.is_symmetric());
        assert!(!JwtAlgorithm::ES256.is_symmetric());
    }

    #[test]
    fn test_token_introspection_response() {
        let inactive = TokenIntrospectionResponse::inactive();
        assert!(!inactive.active);

        let user_id = Uuid::new_v4();
        let claims = Claims::new(
            user_id,
            "issuer",
            "audience",
            TokenType::Access,
            Utc::now() + Duration::hours(1),
        );

        let active = TokenIntrospectionResponse::from_claims(&claims, true);
        assert!(active.active);
        assert_eq!(active.sub, Some(user_id.to_string()));
    }

    #[test]
    fn test_jwk_set() {
        let mut jwk_set = JwkSet::new();
        assert!(jwk_set.keys.is_empty());

        jwk_set.add_key(Jwk {
            kty: "RSA".into(),
            use_: Some("sig".into()),
            kid: Some("key-1".into()),
            alg: Some("RS256".into()),
            n: None,
            e: None,
            x: None,
            y: None,
            crv: None,
        });

        assert_eq!(jwk_set.keys.len(), 1);
    }
}
