//! Compute-provider adapters.
//!
//! Every adapter implements [`ComputeProvider`] against its vendor's REST API.
//! The set is deliberately closed and registered in [`registry`] so a caller
//! resolves a provider *name* rather than writing a `match` that silently
//! grows out of step with the feature flags.

#[cfg(feature = "providers-contabo")]
pub mod contabo;
#[cfg(feature = "providers-digitalocean")]
pub mod digitalocean;
#[cfg(feature = "providers-hetzner")]
pub mod hetzner;
#[cfg(feature = "providers-oracle")]
pub mod oracle;
#[cfg(feature = "providers-ovh")]
pub mod ovh;
pub mod registry;
#[cfg(feature = "providers-runpod")]
pub mod runpod;
#[cfg(feature = "providers-vast")]
pub mod vast;
#[cfg(feature = "providers-vultr")]
pub mod vultr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The machine a caller asked for.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MachineSpec {
    pub cpu_cores: u32,
    pub ram_gb: u32,
    pub disk_gb: u32,
    pub gpu_type: Option<String>,
    pub gpu_count: u32,
    pub bandwidth_tb: u32,
    /// Request a spot/preemptible instance where the provider offers one.
    /// Cheaper and interruptible; on-demand is the safe default.
    pub use_spot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionResult {
    pub provider: String,
    pub instance_id: String,
    pub status: String,
    pub ip_address: Option<String>,
    pub region: String,
    pub spec: MachineSpec,
    pub hourly_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub regions: Vec<String>,
    pub available_gpus: Vec<String>,
    pub supports_spot: bool,
}

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Insufficient capacity: {0}")]
    Capacity(String),
    #[error("Rate limited: retry after {0}s")]
    RateLimited(u64),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
}

#[async_trait::async_trait]
pub trait ComputeProvider: Send + Sync {
    fn name(&self) -> &str;
    fn info(&self) -> ProviderInfo;

    async fn provision(
        &self,
        spec: &MachineSpec,
        region: &str,
        api_key: &str,
    ) -> Result<ProvisionResult, ProviderError>;

    async fn terminate(&self, instance_id: &str, api_key: &str) -> Result<(), ProviderError>;
    async fn get_status(&self, instance_id: &str, api_key: &str) -> Result<String, ProviderError>;
    async fn list_instances(&self, api_key: &str) -> Result<Vec<ProvisionResult>, ProviderError>;
}

/// Maps an HTTP status and a response body to the narrowest [`ProviderError`.
///
/// A 401 is an authentication problem, not a generic API failure — a caller
/// retrying it forever will never succeed. A 429 carries the vendor's own
/// backoff when it offers one. Everything else is [`ProviderError::Api`].
///
/// Bodies are truncated: an HTML error page from a gateway must not end up in a
/// database column or a chat message.
pub(crate) fn classify_status(
    service: &str,
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
    body: &str,
) -> ProviderError {
    let retry_after = headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());

    if status.as_u16() == 401 || status.as_u16() == 403 {
        return ProviderError::Auth(format!("{service}: credentials rejected"));
    }
    if let Some(seconds) = retry_after.filter(|_| status.as_u16() == 429) {
        return ProviderError::RateLimited(seconds);
    }
    if status.as_u16() == 404 {
        return ProviderError::NotFound(format!("{service}: resource not found"));
    }
    ProviderError::Api(format!(
        "{service}: HTTP {} {}",
        status.as_u16(),
        truncate(body)
    ))
}

/// Caps a provider body so a stray HTML error page cannot flood a log line.
pub(crate) fn truncate(body: &str) -> String {
    const LIMIT: usize = 240;
    let trimmed = body.trim();
    if trimmed.chars().count() <= LIMIT {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(LIMIT).collect();
    format!("{head}…")
}

/// True when a status means the provider has no capacity right now, which is a
/// transient condition the caller should try elsewhere.
pub(crate) fn is_capacity_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 409 || status.as_u16() == 503
}

/// Like [`classify_status`], but reports a transient capacity shortage as
/// [`ProviderError::Capacity`].
///
/// The distinction matters at the call site: a capacity error is a signal to try
/// the next provider in the SKU's candidate list, while any other error means
/// the whole provisioning path is wrong.
pub(crate) fn classify_status_or_capacity(
    service: &str,
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
    body: &str,
) -> ProviderError {
    if is_capacity_status(status) {
        return ProviderError::Capacity(format!(
            "{service}: no capacity right now (HTTP {}): {}",
            status.as_u16(),
            truncate(body)
        ));
    }
    classify_status(service, status, headers, body)
}

/// Shared response-body read: a truncated body must not become an empty success.
pub(crate) async fn read_body(resp: reqwest::Response) -> Result<String, ProviderError> {
    resp.text().await.map_err(|e| ProviderError::Api(format!("response body unreadable: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
    use reqwest::StatusCode;

    #[test]
    fn unauthorized_is_an_auth_error_not_a_generic_api_error() {
        let err = classify_status("Vultr", StatusCode::UNAUTHORIZED, &HeaderMap::new(), "bad key");
        assert!(matches!(err, ProviderError::Auth(_)));
    }

    #[test]
    fn forbidden_is_treated_as_bad_credentials() {
        let err = classify_status("Hetzner", StatusCode::FORBIDDEN, &HeaderMap::new(), "");
        assert!(matches!(err, ProviderError::Auth(_)));
    }

    #[test]
    fn retry_after_header_becomes_a_rate_limit_error() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("42"));
        let err = classify_status("OVH", StatusCode::TOO_MANY_REQUESTS, &headers, "");
        match err {
            ProviderError::RateLimited(seconds) => assert_eq!(seconds, 42),
            other => panic!("expected RateLimited, got {other}"),
        }
    }

    #[test]
    fn a_retry_after_header_on_a_200_is_ignored() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("42"));
        let err = classify_status("Vultr", StatusCode::OK, &headers, "");
        assert!(matches!(err, ProviderError::Api(_)));
    }

    #[test]
    fn not_found_is_reported_as_such() {
        let err = classify_status("Vultr", StatusCode::NOT_FOUND, &HeaderMap::new(), "");
        assert!(matches!(err, ProviderError::NotFound(_)));
    }

    #[test]
    fn oversized_bodies_are_truncated() {
        let body = "x".repeat(1_000);
        let message = truncate(&body);
        assert!(message.chars().count() <= 241);
        let err = classify_status("Oracle", StatusCode::BAD_REQUEST, &HeaderMap::new(), &body);
        assert!(err.to_string().chars().count() < 400);
    }

    #[test]
    fn capacity_statuses_are_recognised() {
        assert!(is_capacity_status(StatusCode::CONFLICT));
        assert!(is_capacity_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(!is_capacity_status(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn a_capacity_response_is_distinguishable_from_a_hard_failure() {
        let shortage = classify_status_or_capacity(
            "Hetzner",
            StatusCode::CONFLICT,
            &HeaderMap::new(),
            "no capacity",
        );
        assert!(matches!(shortage, ProviderError::Capacity(_)));

        let bad_request = classify_status_or_capacity(
            "Hetzner",
            StatusCode::BAD_REQUEST,
            &HeaderMap::new(),
            "malformed",
        );
        assert!(matches!(bad_request, ProviderError::Api(_)));
    }

    #[test]
    fn a_default_spec_requests_on_demand() {
        assert!(!MachineSpec::default().use_spot);
    }
}
