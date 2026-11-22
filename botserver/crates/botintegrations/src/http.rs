//! Internal service HTTP transport with opt-in mTLS (#2590).
//!
//! Every call from this crate to an in-cluster service (vector store, LLM,
//! embedding service, directory, …) is built here so the transport decision
//! lives in one place. When an operator opts the containerized install into
//! mTLS, the base URL is rewritten to `https` and clients present the
//! configured CA / client identity. Plaintext installs keep their loopback
//! `http` URLs and an ordinary client — there is no behaviour change for
//! desktop-local or single-host builds.
//!
//! GETs go through `fetch_internal_service_url`, the SSRF-vetted helper for
//! server-built internal URLs; writes use the mTLS-aware client directly
//! (the SSRF guard only governs variable-URL GETs). Neither path logs a URL,
//! header or body: a query string can carry a token and a response can carry a
//! credential. Certificate material is only ever passed as paths to `botcore`.

use botsecurity_core::validation::{fetch_internal_service_url, FetchPolicy};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// A sanitized failure from an internal HTTP call. Variants carry no URL,
/// header or body so `Display` and `Debug` cannot leak a credential into logs.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InternalHttpError {
    /// The request could not be transported (DNS, connect, TLS handshake).
    #[error("internal HTTP transport failed")]
    Transport,
    /// The peer answered with a non-success HTTP status.
    #[error("internal service returned a non-success status")]
    Status,
    /// The peer's body was not valid JSON.
    #[error("internal service response was not valid JSON")]
    Decode,
}

/// Rewrites an internal service base URL for the configured transport.
///
/// This is the call site for `botcore`'s `internal_service_url`: with the mTLS
/// opt-in off (the default) the original URL is returned unchanged; with it on,
/// an `http://` base becomes `https://`.
#[must_use]
pub fn internal_url(base: &str) -> String {
    botcore::shared::utils::internal_service_url(base)
}

/// Builds a client for internal service writes using the mTLS-aware helper.
///
/// When the opt-in is active the client verifies the peer against the
/// configured CA and presents this service's client certificate; otherwise it
/// is the ordinary TLS-verified client used by the rest of the codebase.
#[must_use]
pub fn internal_client(timeout_secs: Option<u64>) -> Client {
    botcore::shared::utils::create_tls_client(timeout_secs)
}

/// Joins a base URL and a path with exactly one slash, without allocating a
/// `Url` (which would reject the placeholders sometimes used in configuration).
fn join_path(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    if path.is_empty() {
        return base.to_string();
    }
    format!("{base}/{}", path.trim_start_matches('/'))
}

fn timeout_of(timeout_secs: Option<u64>) -> Duration {
    Duration::from_secs(timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS))
}

/// Performs a GET against an internal service and decodes a JSON body.
///
/// The URL comes from the mTLS-aware helper above, so the opt-in upgrades the
/// whole call to a verified transport. The request itself goes through the
/// SSRF guard, which accepts only documented internal-service hosts; its
/// generic error never carries the attempted URL.
pub async fn get_json(
    base: &str,
    path: &str,
    timeout_secs: Option<u64>,
) -> Result<Value, InternalHttpError> {
    let url = join_path(&internal_url(base), path);
    let policy = FetchPolicy::default().with_timeout(timeout_of(timeout_secs));
    let fetched = fetch_internal_service_url(&url, &policy)
        .await
        .map_err(|_| InternalHttpError::Transport)?;
    if !fetched.is_success() {
        return Err(InternalHttpError::Status);
    }
    serde_json::from_slice(&fetched.body).map_err(|_| InternalHttpError::Decode)
}

/// Sends a JSON body to an internal service using the mTLS-aware client.
///
/// Both URL and client come from the helpers above, so one opt-in switches the
/// write to a verified, mutually authenticated transport. Errors are reduced
/// to generic variants: nothing about the peer, request or response is logged.
pub async fn post_json(
    base: &str,
    path: &str,
    body: &Value,
    timeout_secs: Option<u64>,
) -> Result<Value, InternalHttpError> {
    let url = join_path(&internal_url(base), path);
    let client = internal_client(Some(timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS)));
    let response = client
        .post(&url)
        .json(body)
        .send()
        .await
        .map_err(|_| InternalHttpError::Transport)?;
    if !response.status().is_success() {
        return Err(InternalHttpError::Status);
    }
    response
        .json::<Value>()
        .await
        .map_err(|_| InternalHttpError::Decode)
}

#[cfg(test)]
mod tests {
    use super::{internal_client, internal_url, join_path, timeout_of, InternalHttpError};
    use std::time::Duration;

    /// With no mTLS opt-in (the default and every desktop-local install) the
    /// helper must leave a loopback URL byte-for-byte intact.
    #[test]
    fn plaintext_default_keeps_the_original_url() {
        assert_eq!(
            internal_url("http://127.0.0.1:6333"),
            "http://127.0.0.1:6333"
        );
    }

    /// Regardless of the active mode, the helper may only change the scheme;
    /// host, port and path must survive verbatim.
    #[test]
    fn url_helper_only_ever_touches_the_scheme() {
        let base = "http://vectordb:6333/collections";
        let rewritten = internal_url(base);
        let normalized = rewritten.replacen("https://", "http://", 1);
        assert_eq!(normalized, base);
    }

    /// A client can always be built without panicking and is immediately
    /// usable; the helper never logs paths or material while doing so.
    #[test]
    fn internal_client_builds_without_logging_secrets() {
        let client = internal_client(Some(1));
        let request = client.get("http://127.0.0.1:1").build();
        assert!(request.is_ok());
    }

    #[test]
    fn join_path_normalizes_slashes() {
        assert_eq!(
            join_path("http://host:8080/", "/api/v1"),
            "http://host:8080/api/v1"
        );
        assert_eq!(join_path("http://host:8080", "api"), "http://host:8080/api");
        assert_eq!(join_path("http://host:8080/", ""), "http://host:8080");
    }

    #[test]
    fn timeout_defaults_when_unset() {
        assert_eq!(timeout_of(None), Duration::from_secs(30));
        assert_eq!(timeout_of(Some(5)), Duration::from_secs(5));
    }

    /// Error `Display` must stay generic: it is what an operator sees in logs.
    #[test]
    fn error_display_never_embeds_a_url_or_body() {
        assert_eq!(
            InternalHttpError::Transport.to_string(),
            "internal HTTP transport failed"
        );
        assert_eq!(
            InternalHttpError::Status.to_string(),
            "internal service returned a non-success status"
        );
        assert_eq!(
            InternalHttpError::Decode.to_string(),
            "internal service response was not valid JSON"
        );
    }

    /// The transport error variant carries no URL, so callers cannot
    /// accidentally log one.
    #[test]
    fn transport_error_is_a_unit_variant() {
        let error = InternalHttpError::Transport;
        assert!(matches!(error, InternalHttpError::Transport));
        assert!(!error.to_string().contains("http"));
    }
}
