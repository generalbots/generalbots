pub mod antivirus;
pub mod api_keys;
pub mod audit;
pub mod auth;
pub mod auth_provider;
pub mod ca;
pub mod cert_pinning;
pub mod command_guard;
pub mod cors;
pub mod csrf;
pub mod dlp;
pub mod encryption;
pub mod error_sanitizer;
pub mod headers;
pub mod integration;
pub mod jwt;
pub mod mfa;
pub mod mutual_tls;
pub mod panic_handler;
pub mod passkey;
pub mod password;
pub mod path_guard;
pub mod prompt_security;
pub mod protection;
pub mod rate_limiter;
pub mod rbac_middleware;
pub mod request_id;
pub mod secrets;
pub mod security_monitoring;
pub mod session;
pub mod sql_guard;
pub mod tls;
pub mod validation;
pub mod webhook;
pub mod zitadel_auth;

pub use protection::{configure_protection_routes, ProtectionManager, ProtectionTool, ToolStatus};

pub use auth_provider::{
    ApiKeyAuthProvider, ApiKeyInfo, AuthProvider, AuthProviderBuilder, AuthProviderRegistry,
    LocalJwtAuthProvider, ZitadelAuthProviderAdapter, create_default_registry,
};
pub use antivirus::{
    AntivirusConfig, AntivirusManager, ProtectionStatus, ScanResult, ScanStatus, ScanType, Threat,
    ThreatSeverity, ThreatStatus, Vulnerability,
};
pub use api_keys::{
    ApiKey as ManagedApiKey, ApiKeyConfig, ApiKeyManager, ApiKeyScope, ApiKeyStatus, ApiKeyUsage,
    CreateApiKeyRequest, CreateApiKeyResponse, RateLimitConfig as ApiKeyRateLimitConfig,
    extract_api_key_from_header,
};
pub use audit::{
    ActorType, AuditActor, AuditConfig, AuditEvent, AuditEventCategory, AuditEventType,
    AuditLogger, AuditOutcome, AuditQuery, AuditQueryResult, AuditResource, AuditSeverity,
    AuditStore, InMemoryAuditStore,
};
pub use auth::{
    admin_only_middleware, auth_middleware, auth_middleware_with_providers, bot_operator_middleware,
    bot_owner_middleware, bot_scope_middleware, extract_user_from_request, extract_user_with_providers,
    require_auth_middleware, require_bot_access, require_bot_permission, require_permission,
    require_permission_middleware, require_role, require_role_middleware, AuthConfig, AuthError,
    AuthenticatedUser, AuthMiddlewareState, BotAccess, Permission, Role,
};
pub use zitadel_auth::{ZitadelAuthConfig, ZitadelAuthProvider, ZitadelUser};
pub use jwt::{
    Claims, JwtAlgorithm, JwtConfig, JwtKey, JwtManager, JwkSet, TokenIntrospectionResponse,
    TokenPair, TokenType, extract_bearer_token,
};
pub use mfa::{
    MfaConfig, MfaManager, MfaMethod, MfaStatus, OtpChallenge, RecoveryCode, TotpAlgorithm,
    TotpEnrollment, UserMfaState, WebAuthnChallenge, WebAuthnCredential,
};
pub use password::{
    Argon2Config, PasswordConfig, PasswordHasher2, PasswordIssue, PasswordStrength,
    PasswordValidationResult, generate_recovery_codes, generate_secure_password, hash_password,
    validate_password, verify_password,
};
pub use rbac_middleware::{
    AccessDecision, AccessDecisionResult, RbacConfig, RbacError, RbacManager, RbacMiddlewareState,
    RequirePermission, RequireResourceAccess, RequireRole, ResourceAcl, ResourcePermission,
    RoutePermission, build_default_route_permissions, create_admin_layer, create_permission_layer,
    create_role_layer, rbac_middleware, require_admin_middleware, require_super_admin_middleware,
};
pub use session::{
    DeviceInfo, InMemorySessionStore, SameSite, Session, SessionConfig, SessionManager,
    SessionStatus, SessionStore, extract_session_id_from_cookie, generate_session_id,
};
pub use csrf::{
    CsrfConfig, CsrfLayer, CsrfManager, CsrfToken, CsrfValidationResult,
    SameSite as CsrfSameSite, csrf_middleware, extract_csrf_from_cookie, extract_csrf_from_form,
};
pub use encryption::{
    EncryptedData, EncryptionAlgorithm, EncryptionConfig, EncryptionKey, EncryptionManager,
    EnvelopeEncryptedData, KeyPurpose, decrypt_field, derive_key_from_password, encrypt_field,
    generate_salt, hash_for_search,
};
pub use prompt_security::{
    InjectionDetection, InjectionType, OutputIssue, OutputValidation, PromptSecurityConfig,
    PromptSecurityManager, ThreatLevel, escape_for_prompt, quick_injection_check,
};
pub use ca::{CaConfig, CaManager, CertificateRequest, CertificateResponse};
pub use cert_pinning::{
    compute_spki_fingerprint, format_fingerprint, parse_fingerprint, CertPinningConfig,
    CertPinningManager, PinType, PinValidationResult, PinnedCert, PinningStats,
};
pub use cors::{
    create_cors_layer, create_cors_layer_for_production, create_cors_layer_with_origins,
    get_cors_allowed_origins, set_cors_allowed_origins, CorsConfig, OriginValidator,
};
pub use command_guard::{
    has_nvidia_gpu_safe, safe_nvidia_smi, safe_pandoc_async, safe_pdftotext_async,
    sanitize_filename, validate_argument, validate_path, CommandGuardError, SafeCommand,
};
pub use error_sanitizer::{
    log_and_sanitize, safe_error, safe_error_with_request_id, sanitize_for_log,
    ErrorSanitizer, SafeErrorResponse,
};
pub use headers::{
    create_security_headers_layer, security_headers_middleware,
    security_headers_middleware_default, CspBuilder, SecurityHeadersConfig,
};
pub use integration::{
    create_https_client, get_tls_integration, init_tls_integration, to_secure_url, TlsIntegration,
};
pub use mutual_tls::{
    services::{
        configure_directory_mtls, configure_forgejo_mtls, configure_livekit_mtls,
        configure_postgres_mtls, configure_qdrant_mtls,
    },
    MtlsConfig, MtlsError, MtlsManager,
};
pub use panic_handler::{
    catch_panic, catch_panic_async, panic_handler_middleware,
    panic_handler_middleware_with_config, set_global_panic_hook, PanicError,
    PanicGuard, PanicHandlerConfig,
};
pub use path_guard::{
    canonicalize_safe, is_safe_path, join_safe, sanitize_path_component,
    PathGuard, PathGuardConfig, PathGuardError,
};
pub use rate_limiter::{
    create_default_rate_limit_layer, create_rate_limit_layer, rate_limit_middleware,
    simple_rate_limit_middleware, CombinedRateLimiter, HttpRateLimitConfig,
};
pub use request_id::{
    generate_prefixed_request_id, generate_request_id, get_current_sequence,
    get_request_id, get_request_id_string, request_id_middleware,
    request_id_middleware_with_config, RequestId, RequestIdConfig,
    CORRELATION_ID_HEADER, REQUEST_ID_HEADER,
};
pub use secrets::{
    is_sensitive_key, redact_sensitive_data, ApiKey, DatabaseCredentials, JwtSecret, SecretBytes,
    SecretString, SecretsStore,
};
pub use sql_guard::{
    build_safe_count_query, build_safe_delete_query, build_safe_select_by_id_query,
    build_safe_select_query, check_for_injection_patterns, escape_string_literal,
    is_table_allowed, sanitize_identifier, validate_identifier, validate_order_column,
    validate_order_direction, validate_table_name, SqlGuardError,
};
pub use tls::{create_https_server, ServiceTlsConfig, TlsConfig, TlsManager, TlsRegistry};
pub use validation::{
    sanitize_html, strip_html_tags, validate_alphanumeric, validate_email, validate_length,
    validate_no_html, validate_no_script_injection, validate_one_of, validate_password_strength,
    validate_phone, validate_range, validate_required, validate_slug, validate_url,
    validate_username, validate_uuid, ValidationError, ValidationResult, Validator,
};

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub tls_enabled: bool,

    pub mtls_enabled: bool,

    pub ca_config: CaConfig,

    pub tls_registry: TlsRegistry,

    pub auto_generate_certs: bool,

    pub renewal_threshold_days: i64,

    pub rate_limit_config: HttpRateLimitConfig,

    pub security_headers_config: SecurityHeadersConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut tls_registry = TlsRegistry::new();
        tls_registry.register_defaults();

        Self {
            tls_enabled: true,
            mtls_enabled: true,
            ca_config: CaConfig::default(),
            tls_registry,
            auto_generate_certs: true,
            renewal_threshold_days: 30,
            rate_limit_config: HttpRateLimitConfig::default(),
            security_headers_config: SecurityHeadersConfig::default(),
        }
    }
}

#[derive(Debug)]
pub struct SecurityManager {
    config: SecurityConfig,
    ca_manager: CaManager,
    mtls_manager: Option<MtlsManager>,
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Result<Self> {
        let ca_manager = CaManager::new(config.ca_config.clone())?;

        let mtls_manager = if config.mtls_enabled {
            let ca_cert = std::fs::read_to_string(&config.ca_config.ca_cert_path).ok();
            let mtls_config = MtlsConfig::new(ca_cert, None, None);
            Some(MtlsManager::new(mtls_config))
        } else {
            None
        };

        Ok(Self {
            config,
            ca_manager,
            mtls_manager,
        })
    }

    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing security infrastructure");

        if self.config.auto_generate_certs && !self.ca_exists() {
            info!("No CA found, initializing new Certificate Authority");
            self.ca_manager.init_ca()?;

            info!("Generating certificates for all services");
            self.ca_manager.issue_service_certificates()?;
        }

        if self.config.mtls_enabled {
            self.initialize_mtls()?;
        }

        self.verify_all_certificates()?;

        if self.config.auto_generate_certs {
            self.start_renewal_monitor();
        }

        info!("Security infrastructure initialized successfully");
        Ok(())
    }

    fn initialize_mtls(&self) -> Result<()> {
        if let Some(ref manager) = self.mtls_manager {
            info!("Initializing mTLS for all services");

            let base_path = PathBuf::from("./botserver-stack/conf/system");

            let ca_path = base_path.join("ca/ca.crt");
            let cert_path = base_path.join("certs/api.crt");
            let key_path = base_path.join("certs/api.key");

            let _ = configure_qdrant_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_postgres_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_forgejo_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_livekit_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_directory_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));

            manager.validate()?;

            info!("mTLS initialized for all services");
        }
        Ok(())
    }

    fn ca_exists(&self) -> bool {
        self.config.ca_config.ca_cert_path.exists() && self.config.ca_config.ca_key_path.exists()
    }

    fn verify_all_certificates(&self) -> Result<()> {
        for service in self.config.tls_registry.services() {
            let cert_path = &service.tls_config.cert_path;
            let key_path = &service.tls_config.key_path;

            if !cert_path.exists() || !key_path.exists() {
                if self.config.auto_generate_certs {
                    warn!(
                        "Certificate missing for service {}, generating...",
                        service.service_name
                    );
                    self.ca_manager.issue_service_certificate(
                        &service.service_name,
                        vec!["localhost", &service.service_name, "127.0.0.1"],
                    )?;
                } else {
                    return Err(anyhow::anyhow!(
                        "Certificate missing for service {} and auto-generation is disabled",
                        service.service_name
                    ));
                }
            }
        }

        Ok(())
    }

    fn start_renewal_monitor(&self) {
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(24 * 60 * 60));

            loop {
                interval.tick().await;

                for service in config.tls_registry.services() {
                    if let Err(e) = check_certificate_renewal(&service.tls_config) {
                        warn!(
                            "Failed to check certificate renewal for {}: {}",
                            service.service_name, e
                        );
                    }
                }
            }
        });
    }

    pub fn get_tls_manager(&self, service_name: &str) -> Result<TlsManager> {
        self.config.tls_registry.get_manager(service_name)
    }

    pub fn ca_manager(&self) -> &CaManager {
        &self.ca_manager
    }

    pub fn is_tls_enabled(&self) -> bool {
        self.config.tls_enabled
    }

    pub fn is_mtls_enabled(&self) -> bool {
        self.config.mtls_enabled
    }

    pub fn mtls_manager(&self) -> Option<&MtlsManager> {
        self.mtls_manager.as_ref()
    }

    pub fn rate_limit_config(&self) -> &HttpRateLimitConfig {
        &self.config.rate_limit_config
    }

    pub fn security_headers_config(&self) -> &SecurityHeadersConfig {
        &self.config.security_headers_config
    }
}

pub fn check_certificate_renewal(_tls_config: &TlsConfig) -> Result<()> {
    Ok(())
}

pub fn create_https_client_with_manager(tls_manager: &TlsManager) -> Result<reqwest::Client> {
    tls_manager.create_https_client()
}

pub fn convert_to_https(url: &str) -> String {
    if url.starts_with("http://") {
        url.replace("http://", "https://")
    } else if !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.to_string()
    }
}

pub fn get_secure_port(service: &str, default_port: u16) -> u16 {
    match service {
        "api" => 8443,
        "llm" => 8444,
        "embedding" => 8445,
        "qdrant" => 6334,
        "redis" => 6380,
        "postgres" => 5433,
        "minio" => 9001,
        "directory" => 8446,
        "email" => 465,
        "meet" => 7881,
        _ => default_port + 443,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TestUser {
        pub id: String,
        pub email: String,
        pub name: String,
        pub password: String,
        pub roles: Vec<String>,
        pub metadata: HashMap<String, String>,
    }

    impl Default for TestUser {
        fn default() -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
                password: "password123".to_string(),
                roles: vec!["user".to_string()],
                metadata: HashMap::new(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TokenResponse {
        access_token: String,
        token_type: String,
        expires_in: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        refresh_token: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id_token: Option<String>,
        scope: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct IntrospectionResponse {
        active: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        client_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        token_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exp: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        iat: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sub: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        aud: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        iss: Option<String>,
    }

    fn base64_url_encode(input: &str) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

        let bytes = input.as_bytes();
        let mut result = String::new();

        for chunk in bytes.chunks(3) {
            let mut n: u32 = 0;
            for (i, &byte) in chunk.iter().enumerate() {
                n |= u32::from(byte) << (16 - i * 8);
            }

            let chars_to_write = match chunk.len() {
                1 => 2,
                2 => 3,
                _ => 4,
            };

            for i in 0..chars_to_write {
                let idx = ((n >> (18 - i * 6)) & 0x3F) as usize;
                result.push(ALPHABET[idx] as char);
            }
        }

        result
    }

    #[test]
    fn test_test_user_default() {
        let user = TestUser::default();
        assert!(!user.id.is_empty());
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.roles, vec!["user"]);
    }

    #[test]
    fn test_base64_url_encode() {
        let encoded = base64_url_encode("hello");
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }

    #[test]
    fn test_token_response_serialization() {
        let response = TokenResponse {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some("refresh".to_string()),
            id_token: None,
            scope: "openid".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("access_token"));
        assert!(json.contains("Bearer"));
        assert!(json.contains("refresh_token"));
        assert!(!json.contains("id_token"));
    }

    #[test]
    fn test_introspection_response_active() {
        let response = IntrospectionResponse {
            active: true,
            scope: Some("openid".to_string()),
            client_id: Some("client".to_string()),
            username: Some("user@test.com".to_string()),
            token_type: Some("Bearer".to_string()),
            exp: Some(1_234_567_890),
            iat: Some(1_234_567_800),
            sub: Some("user-id".to_string()),
            aud: Some("audience".to_string()),
            iss: Some("issuer".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""active":true"#));
    }

    #[test]
    fn test_introspection_response_inactive() {
        let response = IntrospectionResponse {
            active: false,
            scope: None,
            client_id: None,
            username: None,
            token_type: None,
            exp: None,
            iat: None,
            sub: None,
            aud: None,
            iss: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""active":false"#));
        assert!(!json.contains("scope"));
    }

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.tls_enabled);
        assert!(config.mtls_enabled);
        assert!(config.auto_generate_certs);
        assert_eq!(config.renewal_threshold_days, 30);
    }

    #[test]
    fn test_convert_to_https() {
        assert_eq!(
            convert_to_https("http://example.com"),
            "https://example.com"
        );
        assert_eq!(
            convert_to_https("https://example.com"),
            "https://example.com"
        );
        assert_eq!(convert_to_https("example.com"), "https://example.com");
    }

    #[test]
    fn test_get_secure_port() {
        assert_eq!(get_secure_port("api", 8080), 8443);
        assert_eq!(get_secure_port("llm", 8080), 8444);
        assert_eq!(get_secure_port("qdrant", 6333), 6334);
        assert_eq!(get_secure_port("redis", 6379), 6380);
        assert_eq!(get_secure_port("postgres", 5432), 5433);
        assert_eq!(get_secure_port("minio", 9000), 9001);
        assert_eq!(get_secure_port("unknown", 8080), 8523);
    }

    #[test]
    fn test_test_user_with_roles() {
        let user = TestUser {
            roles: vec!["admin".to_string(), "user".to_string()],
            ..Default::default()
        };
        assert!(user.roles.contains(&"admin".to_string()));
        assert!(user.roles.contains(&"user".to_string()));
    }

    #[test]
    fn test_test_user_with_metadata() {
        let mut user = TestUser::default();
        user.metadata
            .insert("department".to_string(), "engineering".to_string());
        user.metadata
            .insert("location".to_string(), "NYC".to_string());

        assert_eq!(
            user.metadata.get("department"),
            Some(&"engineering".to_string())
        );
        assert_eq!(user.metadata.get("location"), Some(&"NYC".to_string()));
    }

    #[test]
    fn test_token_response_without_optional_fields() {
        let response = TokenResponse {
            access_token: "token123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 7200,
            refresh_token: None,
            id_token: None,
            scope: "openid profile".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("token123"));
        assert!(!json.contains("refresh_token"));
        assert!(!json.contains("id_token"));
    }

    #[test]
    fn test_introspection_response_with_all_fields() {
        let response = IntrospectionResponse {
            active: true,
            scope: Some("openid profile email".to_string()),
            client_id: Some("my-client-id".to_string()),
            username: Some("johndoe".to_string()),
            token_type: Some("Bearer".to_string()),
            exp: Some(1_700_000_000),
            iat: Some(1_699_996_400),
            sub: Some("user-123".to_string()),
            aud: Some("api://default".to_string()),
            iss: Some("https://issuer.example.com".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("openid profile email"));
        assert!(json.contains("my-client-id"));
        assert!(json.contains("johndoe"));
        assert!(json.contains("1700000000"));
    }
}
