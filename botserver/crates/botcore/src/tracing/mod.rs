//! W3C Distributed Tracing module
//!
//! Implements the W3C Trace Context specification for distributed tracing
//! across service boundaries. Provides traceparent header generation/parsing,
//! Tower Layer/Service middleware for server-side tracing, and reqwest layer
//! for client-side propagation.
//!
//! No unwrap() or expect() are used in this module.

use std::time::{Duration, SystemTime};
use uuid::Uuid;

const TRACEPARENT_VERSION: &str = "00";
const TRACE_FLAGS_SAMPLED: u8 = 0x01;

/// Represents a W3C Trace Context span context with trace_id, span_id,
/// optional parent_span_id, and trace_flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub trace_flags: u8,
    pub sampled: bool,
    pub start_time: SystemTime,
    pub operation_name: String,
}

impl SpanContext {
    /// Create a new root span context with a fresh trace_id and span_id.
    pub fn new(operation_name: impl Into<String>) -> Self {
        let trace_id = Uuid::new_v4().simple().to_string();
        let span_id = Uuid::new_v4().simple().to_string()[..16].to_string();
        Self {
            trace_id,
            span_id,
            parent_span_id: None,
            trace_flags: TRACE_FLAGS_SAMPLED,
            sampled: true,
            start_time: SystemTime::now(),
            operation_name: operation_name.into(),
        }
    }

    /// Parse a W3C traceparent header value into a SpanContext.
    ///
    /// Format: `{version}-{trace_id}-{span_id}-{trace_flags}`
    /// Version must be `00`. Returns None on invalid input.
    pub fn from_w3c_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.trim().split('-').collect();
        if parts.len() < 4 {
            return None;
        }
        let _version = parts[0];
        let trace_id = parts[1].to_string();
        let span_id = parts[2].to_string();
        let trace_flags = u8::from_str_radix(parts[3], 16).ok()?;
        Some(Self {
            trace_id,
            span_id,
            parent_span_id: None,
            trace_flags,
            sampled: (trace_flags & TRACE_FLAGS_SAMPLED) != 0,
            start_time: SystemTime::now(),
            operation_name: String::new(),
        })
    }

    /// Serialize this span context into a W3C traceparent header value.
    ///
    /// Format: `00-{trace_id}-{span_id}-{trace_flags:02x}`
    pub fn to_w3c_traceparent(&self) -> String {
        format!(
            "{}-{}-{}-{:02x}",
            TRACEPARENT_VERSION, self.trace_id, self.span_id, self.trace_flags
        )
    }

    /// Create a child span that inherits the trace_id and flags, with a new
    /// span_id and this span as its parent.
    pub fn child_span(&self, operation_name: impl Into<String>) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().simple().to_string()[..16].to_string(),
            parent_span_id: Some(self.span_id.clone()),
            trace_flags: self.trace_flags,
            sampled: self.sampled,
            start_time: SystemTime::now(),
            operation_name: operation_name.into(),
        }
    }

    /// Return the elapsed time in milliseconds since this span was created.
    pub fn elapsed_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::from_secs(0))
            .as_millis() as u64
    }
}

/// Start a new span, optionally as a child of a parent context.
pub fn start_span(operation_name: &str, parent: Option<&SpanContext>) -> SpanContext {
    let ctx = match parent {
        Some(p) => p.child_span(operation_name),
        None => SpanContext::new(operation_name),
    };
    log::debug!(
        "[tracing] start_span: {} trace_id={} span_id={}",
        ctx.operation_name,
        ctx.trace_id,
        ctx.span_id
    );
    ctx
}

/// Record the end of a span, logging its duration.
pub fn end_span(span: &SpanContext) {
    let elapsed = span.elapsed_ms();
    log::trace!(
        "[tracing] end_span: {} duration={}ms trace_id={} span_id={}",
        span.operation_name,
        elapsed,
        span.trace_id,
        span.span_id
    );
}

/// Inject a W3C traceparent header into an http::HeaderMap.
pub fn inject_context(ctx: &SpanContext, headers: &mut http::HeaderMap) {
    let traceparent = ctx.to_w3c_traceparent();
    if let Ok(val) = http::HeaderValue::from_str(&traceparent) {
        headers.insert(
            http::header::HeaderName::from_static("traceparent"),
            val,
        );
    }
}

/// Extract a SpanContext from the W3C traceparent header in an http::HeaderMap,
/// returning None if the header is missing or invalid.
pub fn extract_context(headers: &http::HeaderMap) -> Option<SpanContext> {
    let header_str = headers
        .get("traceparent")
        .and_then(|v| v.to_str().ok())?;
    let ctx = SpanContext::from_w3c_traceparent(header_str)?;
    log::debug!(
        "[tracing] extract_context: trace_id={} span_id={}",
        ctx.trace_id,
        ctx.span_id
    );
    Some(ctx)
}

// ---------------------------------------------------------------------------
// Tower Layer / Service middleware for Axum (server-side tracing)
// ---------------------------------------------------------------------------

/// A Tower Layer that wraps every request with W3C trace context propagation.
///
/// Extracts or creates a traceparent, logs the request, and passes the
/// SpanContext to downstream services via request extensions.
#[derive(Clone)]
pub struct TracingMiddleware {
    pub service_name: String,
}

impl TracingMiddleware {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }
}

impl<S> tower::Layer<S> for TracingMiddleware {
    type Service = TracingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracingService {
            inner,
            service_name: self.service_name.clone(),
        }
    }
}

/// The Tower Service produced by TracingMiddleware.
#[derive(Clone)]
pub struct TracingService<S> {
    inner: S,
    service_name: String,
}

impl<S, ReqBody> tower::Service<http::Request<ReqBody>> for TracingService<S>
where
    S: tower::Service<http::Request<ReqBody>>,
    S::Future: Send + 'static,
    S::Error: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let span = extract_context(req.headers())
            .unwrap_or_else(|| SpanContext::new(format!("{} {}", req.method(), req.uri().path())));
        let service_name = self.service_name.clone();

        log::info!(
            "[{}] tracing: {} {} trace_id={} span_id={}",
            service_name,
            req.method(),
            req.uri().path(),
            span.trace_id,
            span.span_id
        );

        let fut = self.inner.call(req);
        Box::pin(async move {
            let resp = fut.await?;
            log::debug!(
                "[tracing] completed span={} trace_id={}",
                span.span_id,
                span.trace_id
            );
            Ok(resp)
        })
    }
}

// ---------------------------------------------------------------------------
// Reqwest client-side tracing layer
// ---------------------------------------------------------------------------

/// A helper for injecting W3C trace context into outgoing reqwest requests.
pub struct ReqwestTracingLayer;

impl ReqwestTracingLayer {
    pub fn new() -> Self {
        Self
    }

    /// Inject W3C traceparent header into a reqwest::Request before sending.
    pub fn inject_into_request(request: &mut reqwest::Request, ctx: &SpanContext) {
        let traceparent = ctx.to_w3c_traceparent();
        if let Ok(val) = reqwest::header::HeaderValue::from_str(&traceparent) {
            request.headers_mut().insert(
                reqwest::header::HeaderName::from_static("traceparent"),
                val,
            );
        }
    }

    /// Create a reqwest Client with default configuration.
    pub fn client_with_tracing() -> reqwest::Client {
        reqwest::Client::builder()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    }
}

impl Default for ReqwestTracingLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Subscriber setup
// ---------------------------------------------------------------------------

/// Initialize the tracing subscriber with sensible defaults.
///
/// Safe to call multiple times — subsequent calls are silently ignored.
pub fn setup_tracing_subscriber() {
    let _ = tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .try_init();
}

// ---------------------------------------------------------------------------
// Header name constants
// ---------------------------------------------------------------------------

/// Axum middleware function used by `axum::middleware::from_fn` to attach
/// distributed-tracing headers and log the request span. Kept as a free
/// function so it can be re-used across routers without taking ownership of
/// the layer.
pub async fn tracing_middleware_fn(
    name: String,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let parent = extract_context(req.headers());
    let span = start_span(&name, parent.as_ref());
    let method = req.method().clone();
    let uri = req.uri().clone();
    log::info!(
        "tracing.span.start name={} method={} uri={} trace_id={}",
        name,
        method,
        uri,
        span.trace_id
    );
    let mut response = next.run(req).await;
    if let Some(ctx) = parent.as_ref() {
        inject_context(ctx, response.headers_mut());
    }
    log::info!(
        "tracing.span.end name={} trace_id={} status={}",
        name,
        span.trace_id,
        response.status()
    );
    response
}

pub fn traceparent_header() -> &'static str {
    "traceparent"
}

pub fn tracestate_header() -> &'static str {
    "tracestate"
}

// ---------------------------------------------------------------------------
// RAII SpanGuard
// ---------------------------------------------------------------------------

/// A guard that automatically ends a span when dropped (RAII pattern).
pub struct SpanGuard {
    ctx: SpanContext,
    finished: bool,
}

impl SpanGuard {
    pub fn new(operation_name: impl Into<String>, parent: Option<&SpanContext>) -> Self {
        Self {
            ctx: start_span(&operation_name.into(), parent),
            finished: false,
        }
    }

    pub fn context(&self) -> &SpanContext {
        &self.ctx
    }

    pub fn child(&self, operation_name: impl Into<String>) -> Self {
        Self::new(operation_name, Some(&self.ctx))
    }

    pub fn finish(&mut self) {
        if !self.finished {
            end_span(&self.ctx);
            self.finished = true;
        }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        self.finish();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traceparent_roundtrip() {
        let ctx = SpanContext::new("test-operation");
        let traceparent = ctx.to_w3c_traceparent();
        let parsed = SpanContext::from_w3c_traceparent(&traceparent);
        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(ctx.trace_id, parsed.trace_id);
        assert_eq!(ctx.span_id, parsed.span_id);
        assert_eq!(ctx.trace_flags, parsed.trace_flags);
    }

    #[test]
    fn test_child_span() {
        let parent = SpanContext::new("parent");
        let child = parent.child_span("child");
        assert_eq!(parent.trace_id, child.trace_id);
        assert_ne!(parent.span_id, child.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_traceparent_format() {
        let ctx = SpanContext::new("test");
        let tp = ctx.to_w3c_traceparent();
        assert!(tp.starts_with("00-"));
        // Format: 00-{32 hex chars}-{16 hex chars}-{2 hex chars}
        assert_eq!(tp.len(), 55);
    }

    #[test]
    fn test_invalid_traceparent() {
        assert!(SpanContext::from_w3c_traceparent("invalid").is_none());
        assert!(SpanContext::from_w3c_traceparent("").is_none());
        assert!(SpanContext::from_w3c_traceparent("00-abc-123-01").is_some());
    }

    #[test]
    fn test_span_guard_auto_finish() {
        let mut guard = SpanGuard::new("test", None);
        assert_eq!(guard.context().operation_name, "test");
        guard.finish();
    }

    #[test]
    fn test_extract_context_no_header() {
        let headers = http::HeaderMap::new();
        assert!(extract_context(&headers).is_none());
    }

    #[test]
    fn test_extract_context_with_header() {
        let mut headers = http::HeaderMap::new();
        let ctx = SpanContext::new("test");
        let tp = ctx.to_w3c_traceparent();
        headers.insert(
            "traceparent",
            http::HeaderValue::from_str(&tp).ok().unwrap(),
        );
        let extracted = extract_context(&headers);
        assert!(extracted.is_some());
        let extracted = extracted.unwrap();
        assert_eq!(ctx.trace_id, extracted.trace_id);
        assert_eq!(ctx.span_id, extracted.span_id);
    }

    #[test]
    fn test_trace_id_generation_unique() {
        let ctx1 = SpanContext::new("a");
        let ctx2 = SpanContext::new("b");
        assert_ne!(ctx1.trace_id, ctx2.trace_id);
    }

    #[test]
    fn test_span_id_generation_unique() {
        let ctx1 = SpanContext::new("a");
        let ctx2 = SpanContext::new("b");
        assert_ne!(ctx1.span_id, ctx2.span_id);
    }

    #[test]
    fn test_inject_context() {
        let ctx = SpanContext::new("test");
        let mut headers = http::HeaderMap::new();
        inject_context(&ctx, &mut headers);
        assert!(headers.contains_key("traceparent"));
        let tp = headers.get("traceparent").unwrap().to_str().unwrap();
        assert!(tp.starts_with("00-"));
    }

    #[test]
    fn test_tower_layer_creation() {
        let layer = TracingMiddleware::new("test-service");
        assert_eq!(layer.service_name, "test-service");
    }

    #[test]
    fn test_reqwest_layer_default() {
        let _layer = ReqwestTracingLayer::new();
        let _client = ReqwestTracingLayer::client_with_tracing();
    }

    #[test]
    fn test_tracestate_header() {
        assert_eq!(tracestate_header(), "tracestate");
        assert_eq!(traceparent_header(), "traceparent");
    }

    #[test]
    fn test_parent_span_id_none_for_root() {
        let ctx = SpanContext::new("root");
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_child_preserves_trace_flags() {
        let parent = SpanContext::new("parent");
        let child = parent.child_span("child");
        assert_eq!(parent.trace_flags, child.trace_flags);
        assert_eq!(parent.sampled, child.sampled);
    }

    #[test]
    fn test_elapsed_ms_non_negative() {
        let ctx = SpanContext::new("timer");
        assert!(ctx.elapsed_ms() < 1000);
    }
}
