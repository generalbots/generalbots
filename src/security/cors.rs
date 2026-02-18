use axum::http::{header, HeaderValue, Method};
use std::collections::HashSet;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age_secs: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Request-ID".to_string(),
                "X-User-ID".to_string(),
                "Accept".to_string(),
                "Accept-Language".to_string(),
                "Origin".to_string(),
            ],
            exposed_headers: vec![
                "X-Request-ID".to_string(),
                "X-RateLimit-Limit".to_string(),
                "X-RateLimit-Remaining".to_string(),
                "X-RateLimit-Reset".to_string(),
                "Retry-After".to_string(),
            ],
            allow_credentials: true,
            max_age_secs: 3600,
        }
    }
}

impl CorsConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_config_value(allowed_origins: Option<&str>) -> Self {
        let mut config = Self::production();

        if let Some(origins) = allowed_origins {
            let origins: Vec<String> = origins
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            if !origins.is_empty() {
                info!("CORS configured with {} allowed origins", origins.len());
                config.allowed_origins = origins;
            }
        }

        config
    }

    pub fn production() -> Self {
        Self {
            allowed_origins: vec![],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Request-ID".to_string(),
            ],
            exposed_headers: vec![
                "X-Request-ID".to_string(),
                "X-RateLimit-Limit".to_string(),
                "X-RateLimit-Remaining".to_string(),
                "Retry-After".to_string(),
            ],
            allow_credentials: true,
            max_age_secs: 7200,
        }
    }

    pub fn development() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:9000".to_string(),
                "http://localhost:8300".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:9000".to_string(),
                "http://127.0.0.1:8300".to_string(),
                "https://localhost:3000".to_string(),
                "https://localhost:9000".to_string(),
                "https://localhost:8300".to_string(),
            ],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
                Method::HEAD,
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Request-ID".to_string(),
                "X-User-ID".to_string(),
                "Accept".to_string(),
                "Accept-Language".to_string(),
                "Origin".to_string(),
                "X-Debug".to_string(),
            ],
            exposed_headers: vec![
                "X-Request-ID".to_string(),
                "X-RateLimit-Limit".to_string(),
                "X-RateLimit-Remaining".to_string(),
                "X-RateLimit-Reset".to_string(),
                "Retry-After".to_string(),
                "X-Debug-Info".to_string(),
            ],
            allow_credentials: true,
            max_age_secs: 3600,
        }
    }

    pub fn api() -> Self {
        Self {
            allowed_origins: vec![],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Request-ID".to_string(),
                "X-API-Key".to_string(),
            ],
            exposed_headers: vec![
                "X-Request-ID".to_string(),
                "X-RateLimit-Limit".to_string(),
                "X-RateLimit-Remaining".to_string(),
                "Retry-After".to_string(),
            ],
            allow_credentials: false,
            max_age_secs: 86400,
        }
    }

    pub fn with_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    pub fn add_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    pub fn with_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods = methods;
        self
    }

    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.allowed_headers = headers;
        self
    }

    pub fn add_header(mut self, header: impl Into<String>) -> Self {
        self.allowed_headers.push(header.into());
        self
    }

    pub fn with_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    pub fn with_max_age(mut self, secs: u64) -> Self {
        self.max_age_secs = secs;
        self
    }

    pub fn build(self) -> CorsLayer {
        let mut cors = CorsLayer::new();

        if self.allowed_origins.is_empty() {
            let allowed_env_origins = get_allowed_origins_from_config();
            if allowed_env_origins.is_empty() {
                cors = cors.allow_origin(AllowOrigin::predicate(validate_origin));
            } else {
                let origins: Vec<HeaderValue> = allowed_env_origins
                    .iter()
                    .filter_map(|o| o.parse().ok())
                    .collect();
                if origins.is_empty() {
                    cors = cors.allow_origin(AllowOrigin::predicate(validate_origin));
                } else {
                    cors = cors.allow_origin(origins);
                }
            }
        } else {
            let origins: Vec<HeaderValue> = self
                .allowed_origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect();
            if origins.is_empty() {
                cors = cors.allow_origin(AllowOrigin::predicate(validate_origin));
            } else {
                cors = cors.allow_origin(origins);
            }
        }

        cors = cors.allow_methods(self.allowed_methods);

        let headers: Vec<header::HeaderName> = self
            .allowed_headers
            .iter()
            .filter_map(|h| h.parse().ok())
            .collect();
        cors = cors.allow_headers(headers);

        let exposed: Vec<header::HeaderName> = self
            .exposed_headers
            .iter()
            .filter_map(|h| h.parse().ok())
            .collect();
        cors = cors.expose_headers(exposed);

        if self.allow_credentials {
            cors = cors.allow_credentials(true);
        }

        cors = cors.max_age(std::time::Duration::from_secs(self.max_age_secs));

        cors
    }
}

fn get_allowed_origins_from_config() -> Vec<String> {
    if let Some(origins) = CORS_ALLOWED_ORIGINS.read().ok().and_then(|g| g.clone()) {
        return origins;
    }
    Vec::new()
}

static CORS_ALLOWED_ORIGINS: std::sync::RwLock<Option<Vec<String>>> = std::sync::RwLock::new(None);

pub fn set_cors_allowed_origins(origins: Vec<String>) {
    if let Ok(mut guard) = CORS_ALLOWED_ORIGINS.write() {
        info!("Setting CORS allowed origins: {:?}", origins);
        *guard = Some(origins);
    }
}

pub fn get_cors_allowed_origins() -> Vec<String> {
    get_allowed_origins_from_config()
}

fn validate_origin(origin: &HeaderValue, _request: &axum::http::request::Parts) -> bool {
    let origin_str = match origin.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if origin_str.is_empty() {
        return false;
    }

    let config_origins = get_allowed_origins_from_config();
    if !config_origins.is_empty() {
        return config_origins.iter().any(|allowed| allowed == origin_str);
    }

    if is_valid_origin_format(origin_str) {
        return true;
    }

    false
}

fn is_valid_origin_format(origin: &str) -> bool {
    if !origin.starts_with("http://") && !origin.starts_with("https://") {
        return false;
    }

    if origin.contains("..") || origin.contains("//", ) && origin.matches("//").count() > 1 {
        return false;
    }

    let dangerous_patterns = [
        "<script",
        "javascript:",
        "data:",
        "vbscript:",
        "%3c",
        "%3e",
        "\\x",
        "\\u",
    ];

    let origin_lower = origin.to_lowercase();
    for pattern in &dangerous_patterns {
        if origin_lower.contains(pattern) {
            return false;
        }
    }

    true
}

pub fn create_cors_layer() -> CorsLayer {
    let config_origins = get_allowed_origins_from_config();

    if !config_origins.is_empty() {
        info!("Creating CORS layer with configured origins");
        CorsConfig::production().with_origins(config_origins).build()
    } else {
        info!("Creating CORS layer with development defaults (no origins configured)");
        CorsConfig::development().build()
    }
}

pub fn create_cors_layer_for_production(allowed_origins: Vec<String>) -> CorsLayer {
    if allowed_origins.is_empty() {
        CorsConfig::production().build()
    } else {
        CorsConfig::production().with_origins(allowed_origins).build()
    }
}

pub fn create_cors_layer_with_origins(origins: Vec<String>) -> CorsLayer {
    CorsConfig::production().with_origins(origins).build()
}

#[derive(Debug, Clone)]
pub struct OriginValidator {
    allowed_origins: HashSet<String>,
    allow_localhost: bool,
    allowed_patterns: Vec<String>,
}

impl Default for OriginValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl OriginValidator {
    pub fn new() -> Self {
        Self {
            allowed_origins: HashSet::new(),
            allow_localhost: false,
            allowed_patterns: Vec::new(),
        }
    }

    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.insert(origin.into());
        self
    }

    pub fn allow_localhost(mut self, allow: bool) -> Self {
        self.allow_localhost = allow;
        self
    }

    pub fn allow_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.allowed_patterns.push(pattern.into());
        self
    }

    pub fn from_config(origins: Vec<String>, patterns: Vec<String>, allow_localhost: bool) -> Self {
        let mut validator = Self::new();

        for origin in origins {
            if !origin.is_empty() {
                validator.allowed_origins.insert(origin);
            }
        }

        for pattern in patterns {
            if !pattern.is_empty() {
                validator.allowed_patterns.push(pattern);
            }
        }

        validator.allow_localhost = allow_localhost;
        validator
    }

    pub fn from_allowed_origins(origins: Vec<String>) -> Self {
        Self::from_config(origins, Vec::new(), false)
    }

    pub fn is_allowed(&self, origin: &str) -> bool {
        if self.allowed_origins.contains(origin) {
            return true;
        }

        if self.allow_localhost && is_localhost_origin(origin) {
            return true;
        }

        for pattern in &self.allowed_patterns {
            if matches_pattern(origin, pattern) {
                return true;
            }
        }

        false
    }
}

fn is_localhost_origin(origin: &str) -> bool {
    let localhost_patterns = [
        "http://localhost",
        "https://localhost",
        "http://127.0.0.1",
        "https://127.0.0.1",
        "http://[::1]",
        "https://[::1]",
    ];

    for pattern in &localhost_patterns {
        if origin.starts_with(pattern) {
            return true;
        }
    }

    false
}

fn matches_pattern(origin: &str, pattern: &str) -> bool {
    if pattern.starts_with("*.") {
        let suffix = &pattern[1..];
        if let Some(host) = extract_host(origin) {
            return host.ends_with(suffix) || host == &suffix[1..];
        }
    }

    if let Some(prefix) = pattern.strip_suffix("*") {
        return origin.starts_with(prefix);
    }

    origin == pattern
}

fn extract_host(origin: &str) -> Option<&str> {
    let without_scheme = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"))?;

    Some(without_scheme.split(':').next().unwrap_or(without_scheme))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CorsConfig::default();
        assert!(config.allowed_origins.is_empty());
        assert!(config.allow_credentials);
        assert_eq!(config.max_age_secs, 3600);
    }

    #[test]
    fn test_production_config() {
        let config = CorsConfig::production();
        assert!(config.allowed_origins.is_empty());
        assert!(config.allow_credentials);
        assert_eq!(config.max_age_secs, 7200);
    }

    #[test]
    fn test_development_config() {
        let config = CorsConfig::development();
        assert!(!config.allowed_origins.is_empty());
        assert!(config.allowed_origins.contains(&"http://localhost:3000".to_string()));
    }

    #[test]
    fn test_api_config() {
        let config = CorsConfig::api();
        assert!(!config.allow_credentials);
        assert_eq!(config.max_age_secs, 86400);
    }

    #[test]
    fn test_builder_methods() {
        let config = CorsConfig::new()
            .with_origins(vec!["https://example.com".to_string()])
            .with_credentials(false)
            .with_max_age(1800);

        assert_eq!(config.allowed_origins.len(), 1);
        assert!(!config.allow_credentials);
        assert_eq!(config.max_age_secs, 1800);
    }

    #[test]
    fn test_add_origin() {
        let config = CorsConfig::new()
            .add_origin("https://example.com")
            .add_origin("https://api.example.com");

        assert_eq!(config.allowed_origins.len(), 2);
    }

    #[test]
    fn test_add_header() {
        let config = CorsConfig::new().add_header("X-Custom-Header");
        assert!(config.allowed_headers.contains(&"X-Custom-Header".to_string()));
    }

    #[test]
    fn test_valid_origin_format() {
        assert!(is_valid_origin_format("https://example.com"));
        assert!(is_valid_origin_format("http://localhost:3000"));
        assert!(is_valid_origin_format("https://api.example.com:8443"));

        assert!(!is_valid_origin_format("ftp://example.com"));
        assert!(!is_valid_origin_format("javascript:alert(1)"));
        assert!(!is_valid_origin_format("data:text/html,<script>"));
    }

    #[test]
    fn test_origin_validator() {
        let validator = OriginValidator::new()
            .allow_origin("https://example.com")
            .allow_localhost(true);

        assert!(validator.is_allowed("https://example.com"));
        assert!(validator.is_allowed("http://localhost:3000"));
        assert!(!validator.is_allowed("https://evil.com"));
    }

    #[test]
    fn test_pattern_matching() {
        let validator = OriginValidator::new().allow_pattern("*.example.com");

        assert!(validator.is_allowed("https://api.example.com"));
        assert!(validator.is_allowed("https://www.example.com"));
        assert!(!validator.is_allowed("https://example.org"));
    }

    #[test]
    fn test_localhost_detection() {
        assert!(is_localhost_origin("http://localhost"));
        assert!(is_localhost_origin("http://localhost:3000"));
        assert!(is_localhost_origin("https://localhost:8443"));
        assert!(is_localhost_origin("http://127.0.0.1"));
        assert!(is_localhost_origin("http://127.0.0.1:9000"));
        assert!(!is_localhost_origin("http://example.com"));
    }

    #[test]
    fn test_extract_host() {
        assert_eq!(extract_host("https://example.com"), Some("example.com"));
        assert_eq!(extract_host("https://example.com:8443"), Some("example.com"));
        assert_eq!(extract_host("http://localhost:3000"), Some("localhost"));
        assert_eq!(extract_host("invalid"), None);
    }

    #[test]
    fn test_build_cors_layer() {
        let config = CorsConfig::development();
        let _layer = config.build();
    }

    #[test]
    fn test_dangerous_patterns_blocked() {
        assert!(!is_valid_origin_format("https://example.com<script>"));
        assert!(!is_valid_origin_format("javascript:void(0)"));
        assert!(!is_valid_origin_format("https://example.com%3cscript%3e"));
    }
}
