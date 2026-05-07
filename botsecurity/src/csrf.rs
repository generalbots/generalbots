use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64, Engine};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

const TOKEN_LENGTH: usize = 32;
const DEFAULT_TOKEN_EXPIRY_MINUTES: i64 = 60;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfConfig {
    pub enabled: bool,
    pub token_expiry_minutes: i64,
    pub cookie_name: String,
    pub header_name: String,
    pub form_field_name: String,
    pub cookie_secure: bool,
    pub cookie_same_site: SameSite,
    pub exempt_paths: Vec<String>,
    pub exempt_methods: Vec<String>,
    pub double_submit_cookie: bool,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            token_expiry_minutes: DEFAULT_TOKEN_EXPIRY_MINUTES,
            cookie_name: "csrf_token".into(),
            header_name: "X-CSRF-Token".into(),
            form_field_name: "_csrf".into(),
            cookie_secure: true,
            cookie_same_site: SameSite::Strict,
            exempt_paths: vec!["/api/health".into(), "/api/version".into()],
            exempt_methods: vec!["GET".into(), "HEAD".into(), "OPTIONS".into()],
            double_submit_cookie: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl SameSite {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "Strict",
            Self::Lax => "Lax",
            Self::None => "None",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub token: String,
    pub session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl CsrfToken {
    pub fn new(expiry_minutes: i64) -> Self {
        let token = generate_token();
        let now = Utc::now();

        Self {
            token,
            session_id: None,
            created_at: now,
            expires_at: now + Duration::minutes(expiry_minutes),
        }
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.token.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CsrfValidationResult {
    Valid,
    Missing,
    Invalid,
    Expired,
    SessionMismatch,
}

impl CsrfValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    pub fn error_message(&self) -> &'static str {
        match self {
            Self::Valid => "Valid",
            Self::Missing => "CSRF token missing",
            Self::Invalid => "CSRF token invalid",
            Self::Expired => "CSRF token expired",
            Self::SessionMismatch => "CSRF token session mismatch",
        }
    }
}

pub struct CsrfManager {
    config: CsrfConfig,
    secret: Vec<u8>,
    tokens: Arc<RwLock<HashMap<String, CsrfToken>>>,
}

impl CsrfManager {
    pub fn new(config: CsrfConfig, secret: &[u8]) -> Result<Self> {
        if secret.len() < 32 {
            return Err(anyhow!("CSRF secret must be at least 32 bytes"));
        }

        Ok(Self {
            config,
            secret: secret.to_vec(),
            tokens: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn from_secret(secret: &str) -> Result<Self> {
        Self::new(CsrfConfig::default(), secret.as_bytes())
    }

    pub async fn generate_token(&self) -> CsrfToken {
        let token = CsrfToken::new(self.config.token_expiry_minutes);

        let mut tokens = self.tokens.write().await;
        tokens.insert(token.token.clone(), token.clone());

        token
    }

    pub async fn generate_token_with_session(&self, session_id: &str) -> CsrfToken {
        let token = CsrfToken::new(self.config.token_expiry_minutes)
            .with_session(session_id.to_string());

        let mut tokens = self.tokens.write().await;
        tokens.insert(token.token.clone(), token.clone());

        token
    }

    pub async fn validate_token(&self, token_value: &str) -> CsrfValidationResult {
        if token_value.is_empty() {
            return CsrfValidationResult::Missing;
        }

        let tokens = self.tokens.read().await;

        match tokens.get(token_value) {
            Some(token) => {
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

        let tokens = self.tokens.read().await;

        match tokens.get(token_value) {
            Some(token) => {
                if token.is_expired() {
                    return CsrfValidationResult::Expired;
                }

                if let Some(ref stored_session) = token.session_id {
                    if stored_session != session_id {
                        return CsrfValidationResult::SessionMismatch;
                    }
                }

                CsrfValidationResult::Valid
            }
            None => CsrfValidationResult::Invalid,
        }
    }

    pub fn validate_double_submit(
        &self,
        cookie_token: &str,
        header_token: &str,
    ) -> CsrfValidationResult {
        if cookie_token.is_empty() || header_token.is_empty() {
            return CsrfValidationResult::Missing;
        }

        if !constant_time_compare(cookie_token, header_token) {
            return CsrfValidationResult::Invalid;
        }

        if !self.verify_signed_token(cookie_token) {
            return CsrfValidationResult::Invalid;
        }

        CsrfValidationResult::Valid
    }

    pub fn generate_signed_token(&self) -> String {
        let token = generate_token();
        let timestamp = Utc::now().timestamp().to_string();
        let data = format!("{token}.{timestamp}");

        let signature = self.sign_data(&data);
        format!("{data}.{signature}")
    }

    pub fn verify_signed_token(&self, signed_token: &str) -> bool {
        let parts: Vec<&str> = signed_token.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        let data = format!("{}.{}", parts[0], parts[1]);
        let provided_signature = parts[2];

        let expected_signature = self.sign_data(&data);
        if !constant_time_compare(&expected_signature, provided_signature) {
            return false;
        }

        if let Ok(timestamp) = parts[1].parse::<i64>() {
            let created = DateTime::from_timestamp(timestamp, 0);
            if let Some(created_time) = created {
                let expiry = created_time + Duration::minutes(self.config.token_expiry_minutes);
                return Utc::now() <= expiry;
            }
        }

        false
    }

    fn sign_data(&self, data: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .expect("HMAC can take key of any size");
        mac.update(data.as_bytes());
        let result = mac.finalize();
        BASE64.encode(result.into_bytes())
    }

    pub async fn revoke_token(&self, token_value: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.remove(token_value);
    }

    pub async fn revoke_session_tokens(&self, session_id: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.retain(|_, t| {
            t.session_id.as_ref().map(|s| s != session_id).unwrap_or(true)
        });
    }

    pub async fn cleanup_expired(&self) -> usize {
        let mut tokens = self.tokens.write().await;
        let initial_count = tokens.len();
        tokens.retain(|_, t| !t.is_expired());
        initial_count - tokens.len()
    }

    pub fn build_cookie(&self, token: &str) -> String {
        let max_age = self.config.token_expiry_minutes * 60;
        let secure = if self.config.cookie_secure {
            "; Secure"
        } else {
            ""
        };
        let same_site = format!("; SameSite={}", self.config.cookie_same_site.as_str());

        format!(
            "{}={}; Path=/; Max-Age={max_age}; HttpOnly{secure}{same_site}",
            self.config.cookie_name, token
        )
    }

    pub fn is_exempt_path(&self, path: &str) -> bool {
        self.config.exempt_paths.iter().any(|p| {
            if p.ends_with('*') {
                let prefix = p.trim_end_matches('*');
                path.starts_with(prefix)
            } else {
                p == path
            }
        })
    }

    pub fn is_exempt_method(&self, method: &str) -> bool {
        self.config
            .exempt_methods
            .iter()
            .any(|m| m.eq_ignore_ascii_case(method))
    }

    pub fn config(&self) -> &CsrfConfig {
        &self.config
    }
}

fn generate_token() -> String {
    let mut rng = rand::rng();
    let bytes: Vec<u8> = (0..TOKEN_LENGTH).map(|_| rng.random()).collect();
    BASE64.encode(&bytes)
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

pub fn extract_csrf_from_cookie(cookie_header: &str, cookie_name: &str) -> Option<String> {
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some((name, value)) = cookie.split_once('=') {
            if name.trim() == cookie_name {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

pub fn extract_csrf_from_form(body: &str, field_name: &str) -> Option<String> {
    for pair in body.split('&') {
        if let Some((name, value)) = pair.split_once('=') {
            if name == field_name {
                return Some(urlencoding::decode(value).ok()?.to_string());
            }
        }
    }
    None
}

pub async fn csrf_middleware(
    csrf_manager: Arc<CsrfManager>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let config = csrf_manager.config();

    if !config.enabled {
        return next.run(request).await;
    }

    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    if csrf_manager.is_exempt_method(&method) {
        return next.run(request).await;
    }

    if csrf_manager.is_exempt_path(&path) {
        return next.run(request).await;
    }

    let header_token = request
        .headers()
        .get(&config.header_name)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let cookie_token = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|c| extract_csrf_from_cookie(c, &config.cookie_name));

    let validation_result = if config.double_submit_cookie {
        match (cookie_token.as_ref(), header_token.as_ref()) {
            (Some(cookie), Some(hdr)) => csrf_manager.validate_double_submit(cookie, hdr),
            _ => CsrfValidationResult::Missing,
        }
    } else {
        match header_token.as_ref() {
            Some(token) => csrf_manager.validate_token(token).await,
            None => CsrfValidationResult::Missing,
        }
    };

    if !validation_result.is_valid() {
        warn!(
            "CSRF validation failed for {} {}: {}",
            method,
            path,
            validation_result.error_message()
        );

        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "csrf_validation_failed",
                "message": validation_result.error_message()
            })),
        )
            .into_response();
    }

    next.run(request).await
}

#[derive(Clone)]
pub struct CsrfLayer {
    manager: Arc<CsrfManager>,
}

impl CsrfLayer {
    pub fn new(manager: Arc<CsrfManager>) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> Arc<CsrfManager> {
        self.manager.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> CsrfManager {
        CsrfManager::from_secret("this-is-a-very-long-secret-key-for-testing-csrf")
            .expect("Failed to create manager")
    }

    #[tokio::test]
    async fn test_generate_and_validate_token() {
        let manager = create_test_manager();

        let token = manager.generate_token().await;
        assert!(!token.token.is_empty());
        assert!(token.is_valid());

        let result = manager.validate_token(&token.token).await;
        assert!(result.is_valid());
    }

    #[tokio::test]
    async fn test_token_with_session() {
        let manager = create_test_manager();
        let session_id = "test-session-123";

        let token = manager.generate_token_with_session(session_id).await;
        assert_eq!(token.session_id, Some(session_id.to_string()));

        let result = manager.validate_token_with_session(&token.token, session_id).await;
        assert!(result.is_valid());

        let result = manager.validate_token_with_session(&token.token, "wrong-session").await;
        assert_eq!(result, CsrfValidationResult::SessionMismatch);
    }

    #[test]
    fn test_signed_token() {
        let manager = create_test_manager();

        let signed = manager.generate_signed_token();
        assert!(manager.verify_signed_token(&signed));
        assert!(!manager.verify_signed_token("invalid.token.here"));
    }

    #[test]
    fn test_double_submit_validation() {
        let manager = create_test_manager();

        let token = manager.generate_signed_token();
        let result = manager.validate_double_submit(&token, &token);
        assert!(result.is_valid());

        let result = manager.validate_double_submit(&token, "different-token");
        assert_eq!(result, CsrfValidationResult::Invalid);
    }

    #[tokio::test]
    async fn test_revoke_token() {
        let manager = create_test_manager();

        let token = manager.generate_token().await;
        assert!(manager.validate_token(&token.token).await.is_valid());

        manager.revoke_token(&token.token).await;
        assert_eq!(
            manager.validate_token(&token.token).await,
            CsrfValidationResult::Invalid
        );
    }

    #[test]
    fn test_exempt_paths() {
        let manager = create_test_manager();

        assert!(manager.is_exempt_path("/api/health"));
        assert!(manager.is_exempt_path("/api/version"));
        assert!(!manager.is_exempt_path("/api/users"));
    }

    #[test]
    fn test_exempt_methods() {
        let manager = create_test_manager();

        assert!(manager.is_exempt_method("GET"));
        assert!(manager.is_exempt_method("HEAD"));
        assert!(manager.is_exempt_method("OPTIONS"));
        assert!(!manager.is_exempt_method("POST"));
        assert!(!manager.is_exempt_method("PUT"));
        assert!(!manager.is_exempt_method("DELETE"));
    }

    #[test]
    fn test_extract_csrf_from_cookie() {
        let cookie = "session=abc123; csrf_token=xyz789; other=value";
        let token = extract_csrf_from_cookie(cookie, "csrf_token");
        assert_eq!(token, Some("xyz789".to_string()));

        let token = extract_csrf_from_cookie(cookie, "nonexistent");
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_csrf_from_form() {
        let body = "username=test&_csrf=abc123&password=secret";
        let token = extract_csrf_from_form(body, "_csrf");
        assert_eq!(token, Some("abc123".to_string()));

        let token = extract_csrf_from_form(body, "nonexistent");
        assert_eq!(token, None);
    }

    #[test]
    fn test_build_cookie() {
        let manager = create_test_manager();
        let cookie = manager.build_cookie("test-token");

        assert!(cookie.contains("csrf_token=test-token"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Strict"));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(!constant_time_compare("abc123", "abc124"));
        assert!(!constant_time_compare("abc", "abcd"));
        assert!(!constant_time_compare("", "a"));
    }

    #[test]
    fn test_same_site_as_str() {
        assert_eq!(SameSite::Strict.as_str(), "Strict");
        assert_eq!(SameSite::Lax.as_str(), "Lax");
        assert_eq!(SameSite::None.as_str(), "None");
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let mut config = CsrfConfig::default();
        config.token_expiry_minutes = 0;
        let manager = CsrfManager::new(config, b"this-is-a-very-long-secret-key-for-testing")
            .expect("Failed to create manager");

        manager.generate_token().await;
        manager.generate_token().await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let cleaned = manager.cleanup_expired().await;
        assert_eq!(cleaned, 2);
    }

    #[test]
    fn test_csrf_validation_result_messages() {
        assert_eq!(CsrfValidationResult::Valid.error_message(), "Valid");
        assert_eq!(CsrfValidationResult::Missing.error_message(), "CSRF token missing");
        assert_eq!(CsrfValidationResult::Invalid.error_message(), "CSRF token invalid");
        assert_eq!(CsrfValidationResult::Expired.error_message(), "CSRF token expired");
        assert_eq!(
            CsrfValidationResult::SessionMismatch.error_message(),
            "CSRF token session mismatch"
        );
    }
}
