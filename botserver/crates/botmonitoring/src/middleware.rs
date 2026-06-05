//! HTTP middleware that propagates trace context across requests.
//!
//! Incoming headers (in priority order): `traceparent` (W3C), `X-Trace-Id`
//! (custom), `X-B3-TraceId` (Zipkin/B3). When none of them is present a
//! fresh trace id is generated. The resulting trace id is also surfaced to
//! the handler via the request extensions and echoed back on the response
//! as `X-Trace-Id` and `X-Run-Id` so downstream callers (curl, browser
//! devtools) can correlate logs.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use rand::RngCore;

const TRACEPARENT: &str = "traceparent";
const B3_TRACE: &str = "x-b3-traceid";
const X_TRACE_ID: &str = "X-Trace-Id";
const X_RUN_ID: &str = "X-Run-Id";

#[derive(Debug, Clone)]
pub struct TraceHeader(pub String);

pub fn generate_trace_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn extract_or_generate(request: &Request<Body>) -> String {
    if let Some(value) = request
        .headers()
        .get(TRACEPARENT)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(id) = parse_traceparent(value) {
            return id;
        }
    }
    if let Some(value) = request.headers().get(B3_TRACE).and_then(|v| v.to_str().ok()) {
        if let Some(id) = parse_b3(value) {
            return id;
        }
    }
    if let Some(value) = request.headers().get(X_TRACE_ID).and_then(|v| v.to_str().ok()) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    generate_trace_id()
}

pub fn parse_traceparent(value: &str) -> Option<String> {
    let mut parts = value.split('-');
    let version = parts.next()?;
    if version != "00" {
        return None;
    }
    let trace_id = parts.next()?;
    if trace_id.len() != 32 {
        return None;
    }
    Some(trace_id.to_string())
}

pub fn parse_b3(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return None;
    }
    Some(trimmed.to_string())
}

pub async fn trace_context_middleware(mut request: Request<Body>, next: Next) -> Response {
    let trace_id = extract_or_generate(&request);
    request.extensions_mut().insert(TraceHeader(trace_id.clone()));
    log::trace!(
        "incoming request trace_id={} method={} path={}",
        trace_id,
        request.method(),
        request.uri().path()
    );
    let mut response = next.run(request).await;
    let header_name = HeaderName::from_static(X_TRACE_ID.to_ascii_lowercase().as_str());
    let run_name = HeaderName::from_static(X_RUN_ID.to_ascii_lowercase().as_str());
    if let Ok(value) = HeaderValue::from_str(&trace_id) {
        response.headers_mut().insert(header_name, value.clone());
        response.headers_mut().insert(run_name, value);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_w3c_traceparent() {
        let id = parse_traceparent("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01").unwrap();
        assert_eq!(id, "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn parse_invalid_traceparent_returns_none() {
        assert!(parse_traceparent("ff-bad").is_none());
        assert!(parse_traceparent("00-abc-b7ad6b7169203331-01").is_none());
    }

    #[test]
    fn parse_b3_accepts_anything_nonempty() {
        assert_eq!(parse_b3("a").unwrap(), "a");
        assert!(parse_b3("").is_none());
    }

    #[test]
    fn generated_id_is_32_hex() {
        let id = generate_trace_id();
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
