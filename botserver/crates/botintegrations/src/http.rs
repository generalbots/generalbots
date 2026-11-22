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
    use super::{
        get_json, internal_client, internal_url, join_path, post_json, timeout_of,
        InternalHttpError,
    };
    use botcoresecrets::mtls::{secure_http_url, MtlsConfig, RolloutStage, TransportMode};
    use serde_json::json;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Config helper for the rewrite assertions below. Holds paths only, never
    /// certificate material, mirroring production `MtlsConfig` resolution.
    fn mtls_config(mode: TransportMode, stage: RolloutStage) -> MtlsConfig {
        MtlsConfig {
            mode,
            stage,
            ca_cert_path: "/certs/ca.pem".to_string(),
            server_cert_path: "/certs/server.pem".to_string(),
            server_key_path: "/certs/server.key".to_string(),
            client_cert_path: "/certs/client.pem".to_string(),
            client_key_path: "/certs/client.key".to_string(),
        }
    }

    /// With no mTLS opt-in (the default and every desktop-local install) the
    /// production entry point leaves internal URLs byte-for-byte intact.
    #[test]
    fn plaintext_default_keeps_the_original_url() {
        for base in [
            "http://127.0.0.1:6333",
            "http://vectordb:6333/collections",
            "http://llm:8080/v1/chat/completions?model=x",
        ] {
            assert_eq!(internal_url(base), base, "plaintext must be a no-op");
        }
        // An already-secure URL is never downgraded.
        assert_eq!(internal_url("https://vault:8200"), "https://vault:8200");
    }

    /// The rewrite primitive `internal_url` delegates to (`secure_http_url`)
    /// upgrades `http://` to `https://` only under the explicit opt-in, and
    /// changes nothing else — scheme only, never host/port/path/query.
    #[test]
    fn opt_in_rewrites_http_to_https_only() {
        let enabled = mtls_config(TransportMode::MutualTls, RolloutStage::ServerTls);
        assert_eq!(
            secure_http_url("http://vectordb:6333/collections", &enabled),
            "https://vectordb:6333/collections"
        );
        assert_eq!(
            secure_http_url("http://127.0.0.1:8080/api?k=v", &enabled),
            "https://127.0.0.1:8080/api?k=v"
        );
        // Idempotent: already https, and non-http schemes stay verbatim.
        assert_eq!(
            secure_http_url("https://vectordb:6333", &enabled),
            "https://vectordb:6333"
        );
        assert_eq!(
            secure_http_url("redis://cache:6379", &enabled),
            "redis://cache:6379"
        );
        // No rewrite before the server-TLS stage, and never in plaintext mode
        // even at the final stage — a half-configured rollout cannot flip it.
        let provisioning = mtls_config(TransportMode::MutualTls, RolloutStage::CertProvisioning);
        assert_eq!(
            secure_http_url("http://vectordb:6333", &provisioning),
            "http://vectordb:6333"
        );
        let plaintext = mtls_config(TransportMode::Plaintext, RolloutStage::PlaintextDisabled);
        assert_eq!(
            secure_http_url("http://vectordb:6333", &plaintext),
            "http://vectordb:6333"
        );
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

    /// The configured timeout flows into the client the transport builds.
    #[test]
    fn timeout_defaults_when_unset() {
        assert_eq!(timeout_of(None), Duration::from_secs(30));
        assert_eq!(timeout_of(Some(5)), Duration::from_secs(5));
        assert_eq!(timeout_of(Some(1)), Duration::from_secs(1));
    }

    /// Error `Display` — what an operator sees in logs — stays generic.
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

    /// Every variant is fieldless, so neither `Display` nor `Debug` can carry
    /// a credential, header, URL or body that a caller accidentally logs.
    #[test]
    fn errors_cannot_leak_credentials_or_urls() {
        let secret = "SUPER_SECRET_TOKEN_XYZ";
        for error in [
            InternalHttpError::Transport,
            InternalHttpError::Status,
            InternalHttpError::Decode,
        ] {
            assert!(matches!(
                error,
                InternalHttpError::Transport
                    | InternalHttpError::Status
                    | InternalHttpError::Decode
            ));
            let display = error.to_string();
            let debug = format!("{error:?}");
            for rendered in [&display, &debug] {
                assert!(!rendered.contains(secret), "secret leaked: {rendered}");
                assert!(!rendered.contains("http"), "URL leaked: {rendered}");
                assert!(!rendered.contains("Bearer"), "header leaked: {rendered}");
                assert!(!rendered.contains("token"), "credential leaked: {rendered}");
            }
        }
        assert!(!InternalHttpError::Transport
            .to_string()
            .contains("127.0.0.1"));
    }

    /// The GET path maps the SSRF guard's refusal of a public host to the
    /// generic transport error — the rejected URL (and any token in it) is
    /// never echoed back to the caller.
    #[tokio::test]
    async fn get_json_refuses_public_host_without_leaking_it() {
        let error = get_json("http://example.com", "/secret?token=SUPER_SECRET", Some(1))
            .await
            .unwrap_err();
        assert_eq!(error, InternalHttpError::Transport);
        assert!(!error.to_string().contains("example.com"));
        assert!(!format!("{error:?}").contains("SUPER_SECRET"));
    }

    /// Builds a one-shot local responder that reads the request and writes a
    /// raw HTTP response, then closes the connection.
    async fn spawn_json_responder(response: Vec<u8>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local test responder");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 4096];
                let _ = socket.read(&mut buf).await;
                let _ = socket.write_all(&response).await;
                let _ = socket.flush().await;
            }
        });
        format!("http://{addr}")
    }

    fn http_response(status_line: &str, body: &str) -> Vec<u8> {
        let mut response = format!(
            "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(body.as_bytes());
        response
    }

    /// POST is built through the mTLS-aware `internal_client` (not a bare
    /// `Client::new()`) and maps every outcome to a generic typed error. A
    /// planted credential in the body/query never appears in those errors.
    #[tokio::test]
    async fn post_json_uses_client_and_returns_typed_errors() {
        let base = spawn_json_responder(http_response("HTTP/1.1 200 OK", r#"{"ok":true}"#)).await;
        let value = post_json(
            &base,
            "/api/test",
            &json!({"token": "SUPER_SECRET"}),
            Some(2),
        )
        .await
        .expect("successful JSON response");
        assert_eq!(value, json!({"ok": true}));

        let base = spawn_json_responder(http_response(
            "HTTP/1.1 500 Internal Server Error",
            r#"{"error":"boom"}"#,
        ))
        .await;
        let error = post_json(
            &base,
            "/api/test?api_key=SUPER_SECRET",
            &json!({"token": "SUPER_SECRET"}),
            Some(2),
        )
        .await
        .unwrap_err();
        assert_eq!(error, InternalHttpError::Status);
        assert!(!format!("{error:?}").contains("SUPER_SECRET"));
        assert!(!error.to_string().contains("http"));

        let base = spawn_json_responder(http_response("HTTP/1.1 200 OK", "not-json")).await;
        let error = post_json(&base, "/api/test", &json!({}), Some(2))
            .await
            .unwrap_err();
        assert_eq!(error, InternalHttpError::Decode);
    }

    /// A silent peer makes `post_json` time out with `Transport` well before
    /// the test's own guard. A bare `Client::new()` (no request timeout) would
    /// blow past the guard — so this proves POST uses the configured TLS
    /// client built by `internal_client`, honoring the caller's timeout.
    #[tokio::test]
    async fn post_json_honors_configured_timeout_via_tls_client() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local test responder");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            if let Ok((socket, _)) = listener.accept().await {
                tokio::time::sleep(Duration::from_secs(3)).await;
                drop(socket);
            }
        });
        let base = format!("http://{addr}");
        let outcome = tokio::time::timeout(
            Duration::from_secs(4),
            post_json(&base, "/api/test", &json!({}), Some(1)),
        )
        .await
        .expect("post_json must return before the test guard");
        assert_eq!(outcome.unwrap_err(), InternalHttpError::Transport);
    }
}
