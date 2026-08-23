//! Shared plain HTTPS executor and validation helpers for the JSON REST
//! provider adapters (#950 slice 2).
//!
//! GitHub and Stripe authenticate with bearer credentials over plain HTTPS
//! instead of SigV4 signing, so both adapters share this small client. Every
//! response body is read incrementally under a hard byte cap before parsing,
//! provider error details stay in the server log only, and callers receive
//! static sentinel strings that map cleanly onto HTTP responses.

use std::sync::OnceLock;
use std::time::Duration;

use bytes::Bytes;
use serde_json::Value;

const REQUEST_TIMEOUT_SECS: u64 = 15;
/// Hard cap applied to every adapter response body (256 KiB).
pub(crate) const MAX_RESPONSE_BYTES: usize = 256 * 1024;

pub(crate) const ERR_NETWORK: &str = "provider_unreachable";
pub(crate) const ERR_RESPONSE_CAP: &str = "response_too_large";

fn shared_client() -> Result<&'static reqwest::Client, String> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .connect_timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .build()
                .map_err(|error| {
                    log::error!("shared REST http client init failed: {error}");
                })
                .ok()
        })
        .as_ref()
        .ok_or_else(|| ERR_NETWORK.to_string())
}

/// Fully specified outbound request for a JSON REST provider.
pub(crate) struct RestRequest {
    pub(crate) method: reqwest::Method,
    pub(crate) url: String,
    pub(crate) headers: Vec<(&'static str, String)>,
    pub(crate) body: Option<Vec<u8>>,
    /// Hard cap applied to the response body of this request.
    pub(crate) response_cap: usize,
}

/// Status line plus the size-capped response body of one provider call.
pub(crate) struct RestResponse {
    pub(crate) status: u16,
    pub(crate) body: Bytes,
}

impl RestResponse {
    /// Fails non-2xx responses; only the action label and status reach the
    /// caller while provider-side details stay logged.
    pub(crate) fn require_success(&self, what: &str) -> Result<(), String> {
        if (200..300).contains(&self.status) {
            return Ok(());
        }
        log::warn!("{what} failed with status {}", self.status);
        Err(format!(
            "provider_request_failed: {what} returned status {}",
            self.status
        ))
    }

    /// Parses the capped body as JSON. Malformed payloads are rejected with
    /// a static sentinel after being logged server-side.
    pub(crate) fn json(&self, what: &str) -> Result<Value, String> {
        serde_json::from_slice(&self.body).map_err(|error| {
            log::warn!("{what} returned malformed JSON: {error}");
            "invalid_response: provider returned malformed JSON".to_string()
        })
    }
}

/// Sends one plain HTTPS request and returns status plus capped body.
async fn send_once(request: &RestRequest) -> Result<(u16, reqwest::Response), String> {
    let mut outgoing = shared_client()?.request(request.method.clone(), &request.url);
    for (name, value) in &request.headers {
        outgoing = outgoing.header(*name, value);
    }
    if let Some(body) = &request.body {
        outgoing = outgoing.body(body.clone());
    }
    let response = outgoing.send().await.map_err(|error| {
        log::warn!("REST provider request failed: {error}");
        ERR_NETWORK.to_string()
    })?;
    Ok((response.status().as_u16(), response))
}

/// Sends one request, honoring a single `Retry-After` when the provider
/// answers 429 so short rate-limit windows do not surface as failures to
/// chat. Wait time is capped at twenty seconds.
pub(crate) async fn send(request: RestRequest) -> Result<RestResponse, String> {
    let (status, response) = send_once(&request).await?;
    if status != 429 {
        return Ok(RestResponse { status, body: read_capped(response, request.response_cap).await? });
    }
    let wait = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5)
        .min(20);
    log::info!("provider answered 429; retrying once after {wait}s");
    tokio::time::sleep(Duration::from_secs(wait)).await;
    let (status, response) = send_once(&request).await?;
    let body = read_capped(response, request.response_cap).await?;
    Ok(RestResponse { status, body })
}

async fn read_capped(mut response: reqwest::Response, cap: usize) -> Result<Bytes, String> {
    let mut buffer: Vec<u8> = Vec::with_capacity(8192);
    loop {
        let chunk = response.chunk().await.map_err(|error| {
            log::warn!("REST provider response read failed: {error}");
            ERR_NETWORK.to_string()
        })?;
        let Some(chunk) = chunk else {
            return Ok(Bytes::from(buffer));
        };
        if buffer.len().saturating_add(chunk.len()) > cap {
            log::warn!("REST provider response exceeded the {cap} byte cap");
            return Err(format!(
                "{ERR_RESPONSE_CAP}: response exceeds {cap} byte cap"
            ));
        }
        buffer.extend_from_slice(chunk.as_ref());
    }
}

// ---------------------------------------------------------------------------
// Shared parameter validation helpers. Every adapter validates parameters
// fully before any network activity happens.
// ---------------------------------------------------------------------------

pub(crate) fn invalid(detail: String) -> String {
    format!("invalid_request: {detail}")
}

pub(crate) fn required_text(params: &Value, key: &str, max_len: usize) -> Result<String, String> {
    let value = params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| invalid(format!("{key} is required")))?;
    if value.len() > max_len {
        return Err(invalid(format!(
            "{key} must be at most {max_len} characters"
        )));
    }
    Ok(value.to_string())
}

pub(crate) fn optional_text(
    params: &Value,
    key: &str,
    max_len: usize,
) -> Result<Option<String>, String> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            if trimmed.len() > max_len {
                return Err(invalid(format!(
                    "{key} must be at most {max_len} characters"
                )));
            }
            Ok(Some(trimmed.to_string()))
        }
        Some(_) => Err(invalid(format!("{key} must be a string"))),
    }
}

/// Resolves an optional positive integer `limit` parameter bounded to
/// `max_allowed`, falling back to `default` when absent.
pub(crate) fn bounded_limit(
    params: &Value,
    key: &str,
    default: usize,
    max_allowed: usize,
) -> Result<usize, String> {
    let raw = match params.get(key) {
        None | Some(Value::Null) => return Ok(default),
        Some(value) => value,
    };
    let parsed = match raw.as_u64() {
        Some(number) => number,
        None => return Err(invalid(format!("{key} must be a positive integer"))),
    };
    if parsed == 0 || parsed > max_allowed as u64 {
        return Err(invalid(format!(
            "{key} must be between 1 and {max_allowed}"
        )));
    }
    Ok(parsed as usize)
}

/// Validates an `owner/repo` repository slug (`^[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)?$`)
/// and returns its two segments. Rejection happens before any network use.
pub(crate) fn validate_repository_slug(slug: &str) -> Result<(String, String), String> {
    let invalid_slug = || {
        invalid(
            "repository must match owner/repo using letters, digits, '.', '_' or '-'".to_string(),
        )
    };
    let mut segments = slug.split('/');
    let (Some(owner), Some(repository), None) = (segments.next(), segments.next(), segments.next())
    else {
        return Err(invalid_slug());
    };
    let valid_segment = |segment: &str| {
        !segment.is_empty()
            && segment.len() <= 100
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
    };
    if !valid_segment(owner) || !valid_segment(repository) {
        return Err(invalid_slug());
    }
    Ok((owner.to_string(), repository.to_string()))
}

/// Appends one URL-encoded query pair to a query string under construction.
pub(crate) fn push_query_pair(query: &mut String, name: &str, value: &str) {
    if !query.is_empty() {
        query.push('&');
    }
    query.push_str(name);
    query.push('=');
    query.push_str(&urlencoding::encode(value));
}

pub(crate) fn outcome(summary: String, data: Value) -> crate::providers::ActionOutcome {
    crate::providers::ActionOutcome {
        summary,
        data,
        truncated: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn repository_slugs_require_two_safe_segments() {
        assert_eq!(
            validate_repository_slug("owner/repo").ok(),
            Some(("owner".to_string(), "repo".to_string()))
        );
        assert!(validate_repository_slug("a.b-c_d/e.f-g_h").is_ok());
        assert!(validate_repository_slug("justone").is_err());
        assert!(validate_repository_slug("too/many/slashes").is_err());
        assert!(validate_repository_slug("/leading").is_err());
        assert!(validate_repository_slug("trailing/").is_err());
        assert!(validate_repository_slug("has space/x").is_err());
        assert!(validate_repository_slug("own er/repo").is_err());
    }

    #[test]
    fn required_and_optional_text_enforce_bounds() {
        let params = json!({ "name": " value ", "empty": "   ", "number": 7 });
        assert_eq!(
            required_text(&params, "name", 10).ok(),
            Some("value".to_string())
        );
        assert!(required_text(&params, "missing", 10).is_err());
        assert!(required_text(&params, "empty", 10).is_err());
        assert!(required_text(&params, "number", 10).is_err());

        assert_eq!(optional_text(&params, "missing", 10).ok(), Some(None));
        assert_eq!(optional_text(&params, "empty", 10).ok(), Some(None));
        assert!(optional_text(&params, "number", 10).is_err());

        let long = "x".repeat(11);
        let params = json!({ "name": long, "text": long });
        assert!(required_text(&params, "name", 10).is_err());
        assert!(optional_text(&params, "text", 10).is_err());
    }

    #[test]
    fn limits_are_positive_bounded_integers_with_defaults() {
        let params = json!({ "limit": 5 });
        assert_eq!(bounded_limit(&params, "limit", 50, 50).ok(), Some(5));
        assert_eq!(bounded_limit(&json!({}), "limit", 50, 50).ok(), Some(50));
        assert_eq!(
            bounded_limit(&json!({ "limit": null }), "limit", 50, 50).ok(),
            Some(50)
        );
        assert!(bounded_limit(&json!({ "limit": 0 }), "limit", 50, 50).is_err());
        assert!(bounded_limit(&json!({ "limit": 51 }), "limit", 50, 50).is_err());
        assert!(bounded_limit(&json!({ "limit": "ten" }), "limit", 50, 50).is_err());
        assert!(bounded_limit(&json!({ "limit": -1 }), "limit", 50, 50).is_err());
    }

    #[test]
    fn query_pairs_are_url_encoded() {
        let mut query = String::new();
        push_query_pair(&mut query, "q", "rust repo:a/b");
        push_query_pair(&mut query, "per_page", "20");
        assert_eq!(query, "q=rust%20repo%3Aa%2Fb&per_page=20");
    }
}
