use axum::{
    body::Body,
    http::{header::HeaderName, HeaderValue, Request},
    middleware::Next,
    response::Response,
};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info_span, Instrument, Span};
use uuid::Uuid;

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const CORRELATION_ID_HEADER: &str = "x-correlation-id";

#[derive(Debug, Clone)]
pub struct RequestId {
    pub id: String,
    pub correlation_id: Option<String>,
    pub sequence: u64,
}

impl RequestId {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            correlation_id: None,
            sequence: REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst),
        }
    }

    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            correlation_id: None,
            sequence: REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst),
        }
    }

    pub fn with_correlation(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn short_id(&self) -> &str {
        if self.id.len() >= 8 {
            &self.id[..8]
        } else {
            &self.id
        }
    }

    pub fn as_header_value(&self) -> Option<HeaderValue> {
        HeaderValue::from_str(&self.id).ok()
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Clone)]
pub struct RequestIdConfig {
    pub header_name: String,
    pub correlation_header_name: String,
    pub generate_if_missing: bool,
    pub propagate_to_response: bool,
    pub add_to_tracing_span: bool,
    pub prefix: Option<String>,
}

impl Default for RequestIdConfig {
    fn default() -> Self {
        Self {
            header_name: REQUEST_ID_HEADER.to_string(),
            correlation_header_name: CORRELATION_ID_HEADER.to_string(),
            generate_if_missing: true,
            propagate_to_response: true,
            add_to_tracing_span: true,
            prefix: None,
        }
    }
}

impl RequestIdConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_header_name(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into();
        self
    }

    pub fn with_correlation_header(mut self, name: impl Into<String>) -> Self {
        self.correlation_header_name = name.into();
        self
    }

    pub fn generate_if_missing(mut self, generate: bool) -> Self {
        self.generate_if_missing = generate;
        self
    }

    pub fn propagate_to_response(mut self, propagate: bool) -> Self {
        self.propagate_to_response = propagate;
        self
    }

    pub fn add_to_span(mut self, add: bool) -> Self {
        self.add_to_tracing_span = add;
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }
}

pub async fn request_id_middleware(request: Request<Body>, next: Next) -> Response {
    request_id_middleware_with_config(request, next, &RequestIdConfig::default()).await
}

pub async fn request_id_middleware_with_config(
    mut request: Request<Body>,
    next: Next,
    config: &RequestIdConfig,
) -> Response {
    let header_name: HeaderName = config
        .header_name
        .parse()
        .unwrap_or_else(|_| HeaderName::from_static(REQUEST_ID_HEADER));

    let request_id = extract_or_generate_request_id(&request, &header_name, config);

    let correlation_id = request
        .headers()
        .get(&config.correlation_header_name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let request_id = if let Some(corr_id) = correlation_id {
        request_id.with_correlation(corr_id)
    } else {
        request_id
    };

    request.extensions_mut().insert(request_id.clone());

    let span = if config.add_to_tracing_span {
        info_span!(
            "request",
            request_id = %request_id.id,
            correlation_id = ?request_id.correlation_id,
            seq = request_id.sequence
        )
    } else {
        Span::none()
    };

    let response = next.run(request).instrument(span).await;

    if config.propagate_to_response {
        add_request_id_to_response(response, &request_id, &header_name)
    } else {
        response
    }
}

fn extract_or_generate_request_id(
    request: &Request<Body>,
    header_name: &HeaderName,
    config: &RequestIdConfig,
) -> RequestId {
    if let Some(existing_id) = request
        .headers()
        .get(header_name)
        .and_then(|v| v.to_str().ok())
    {
        if is_valid_request_id(existing_id) {
            return RequestId::with_id(existing_id);
        }
    }

    if config.generate_if_missing {
        let id = if let Some(ref prefix) = config.prefix {
            format!("{}-{}", prefix, Uuid::new_v4())
        } else {
            Uuid::new_v4().to_string()
        };
        RequestId::with_id(id)
    } else {
        RequestId::with_id("")
    }
}

fn is_valid_request_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 128 {
        return false;
    }

    id.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.'
    })
}

fn add_request_id_to_response(
    mut response: Response,
    request_id: &RequestId,
    header_name: &HeaderName,
) -> Response {
    if let Some(value) = request_id.as_header_value() {
        response.headers_mut().insert(header_name.clone(), value);
    }

    if let Some(ref correlation_id) = request_id.correlation_id {
        if let Ok(value) = HeaderValue::from_str(correlation_id) {
            if let Ok(header) = CORRELATION_ID_HEADER.parse::<HeaderName>() {
                response.headers_mut().insert(header, value);
            }
        }
    }

    response
}

pub fn get_request_id<B>(request: &Request<B>) -> Option<&RequestId> {
    request.extensions().get::<RequestId>()
}

pub fn get_request_id_string<B>(request: &Request<B>) -> String {
    request
        .extensions()
        .get::<RequestId>()
        .map(|r| r.id.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_prefixed_request_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4())
}

pub fn get_current_sequence() -> u64 {
    REQUEST_COUNTER.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_new() {
        let id = RequestId::new();
        assert!(!id.id.is_empty());
        assert!(id.correlation_id.is_none());
    }

    #[test]
    fn test_request_id_with_id() {
        let id = RequestId::with_id("custom-id");
        assert_eq!(id.id, "custom-id");
    }

    #[test]
    fn test_request_id_with_correlation() {
        let id = RequestId::new().with_correlation("corr-123");
        assert_eq!(id.correlation_id, Some("corr-123".to_string()));
    }

    #[test]
    fn test_short_id() {
        let id = RequestId::with_id("12345678-1234-1234-1234-123456789012");
        assert_eq!(id.short_id(), "12345678");

        let short = RequestId::with_id("abc");
        assert_eq!(short.short_id(), "abc");
    }

    #[test]
    fn test_as_header_value() {
        let id = RequestId::with_id("valid-header-value");
        assert!(id.as_header_value().is_some());
    }

    #[test]
    fn test_display() {
        let id = RequestId::with_id("test-id");
        assert_eq!(format!("{}", id), "test-id");
    }

    #[test]
    fn test_config_default() {
        let config = RequestIdConfig::default();
        assert_eq!(config.header_name, REQUEST_ID_HEADER);
        assert!(config.generate_if_missing);
        assert!(config.propagate_to_response);
        assert!(config.add_to_tracing_span);
    }

    #[test]
    fn test_config_builder() {
        let config = RequestIdConfig::new()
            .with_header_name("X-Custom-ID")
            .with_correlation_header("X-Trace-ID")
            .generate_if_missing(false)
            .propagate_to_response(false)
            .add_to_span(false)
            .with_prefix("myapp");

        assert_eq!(config.header_name, "X-Custom-ID");
        assert_eq!(config.correlation_header_name, "X-Trace-ID");
        assert!(!config.generate_if_missing);
        assert!(!config.propagate_to_response);
        assert!(!config.add_to_tracing_span);
        assert_eq!(config.prefix, Some("myapp".to_string()));
    }

    #[test]
    fn test_is_valid_request_id() {
        assert!(is_valid_request_id("abc-123"));
        assert!(is_valid_request_id("test_id.v1"));
        assert!(is_valid_request_id("12345678-1234-1234-1234-123456789012"));

        assert!(!is_valid_request_id(""));
        assert!(!is_valid_request_id("id with space"));
        assert!(!is_valid_request_id("id<script>"));

        let too_long = "a".repeat(200);
        assert!(!is_valid_request_id(&too_long));
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert_ne!(id1, id2);
        assert!(Uuid::parse_str(&id1).is_ok());
    }

    #[test]
    fn test_generate_prefixed_request_id() {
        let id = generate_prefixed_request_id("myapp");
        assert!(id.starts_with("myapp-"));
    }

    #[test]
    fn test_sequence_increments() {
        let id1 = RequestId::new();
        let id2 = RequestId::new();
        assert!(id2.sequence > id1.sequence);
    }

    #[test]
    fn test_get_current_sequence() {
        let before = get_current_sequence();
        let _ = RequestId::new();
        let after = get_current_sequence();
        assert!(after > before);
    }

    #[test]
    fn test_request_id_default() {
        let id: RequestId = Default::default();
        assert!(!id.id.is_empty());
    }
}
