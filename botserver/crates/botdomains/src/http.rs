//! Shared HTTP plumbing for the registrar adapters.

use std::time::Duration;

use serde_json::Value;

use crate::types::{Credential, RegistrarError};

/// Builds a client with a timeout.
///
/// Every registrar call sits on the critical path of a purchase, so an unbounded
/// request would hold the provisioning task open indefinitely.
pub fn client(service: &str) -> Result<reqwest::Client, RegistrarError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(45))
        .connect_timeout(Duration::from_secs(15))
        .user_agent(concat!("general-bots-domains/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| RegistrarError::Transport(service.into(), e.to_string()))
}

/// Turns a non-success response into the narrowest error.
///
/// A vendor body can be an HTML error page; it is truncated so it cannot flood a
/// log line or a chat message.
pub fn classify(
    service: &str,
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
    body: &str,
) -> RegistrarError {
    let retry_after = headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());

    if matches!(status.as_u16(), 401 | 403) {
        return RegistrarError::Credentials(format!("{service} (HTTP {})", status.as_u16()));
    }
    if let Some(seconds) = retry_after.filter(|_| status.as_u16() == 429) {
        return RegistrarError::RateLimited(service.into(), seconds);
    }
    if status.as_u16() == 429 {
        return RegistrarError::RateLimited(service.into(), 60);
    }
    let body = truncate(body);
    if matches!(status.as_u16(), 501 | 405) {
        return RegistrarError::Unsupported(service.into(), format!("HTTP {}", status.as_u16()));
    }
    RegistrarError::Rejected(
        service.into(),
        format!("HTTP {}: {body}", status.as_u16()),
    )
}

/// Caps a vendor body so a stray HTML page cannot flood a log line.
pub fn truncate(body: &str) -> String {
    const LIMIT: usize = 240;
    let trimmed = body.trim();
    if trimmed.chars().count() <= LIMIT {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(LIMIT).collect();
    format!("{head}…")
}

/// Reads a response body, mapping a read failure to a transport error rather
/// than an empty success.
pub async fn body(service: &str, resp: reqwest::Response) -> Result<String, RegistrarError> {
    resp.text()
        .await
        .map_err(|e| RegistrarError::Transport(service.into(), format!("body unreadable: {e}")))
}

/// Parses a JSON body.
pub fn parse(service: &str, text: &str) -> Result<Value, RegistrarError> {
    serde_json::from_str(text).map_err(|e| {
        RegistrarError::Decode(
            service.into(),
            format!("{e} in {}", truncate(text)),
        )
    })
}

/// Reads a required credential pair.
pub fn credential<'a>(
    service: &str,
    credential: &'a Credential,
) -> Result<(&'a str, &'a str), RegistrarError> {
    credential.require(service)?;
    Ok((credential.api_key.trim(), credential.api_secret.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
    use reqwest::StatusCode;

    #[test]
    fn unauthorized_and_forbidden_both_read_as_bad_credentials() {
        assert!(matches!(
            classify("Porkbun", StatusCode::UNAUTHORIZED, &HeaderMap::new(), ""),
            RegistrarError::Credentials(_)
        ));
        assert!(matches!(
            classify("Porkbun", StatusCode::FORBIDDEN, &HeaderMap::new(), ""),
            RegistrarError::Credentials(_)
        ));
    }

    #[test]
    fn a_retry_after_header_becomes_the_backoff() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("17"));
        match classify("Cloudflare", StatusCode::TOO_MANY_REQUESTS, &headers, "") {
            RegistrarError::RateLimited(_, seconds) => assert_eq!(seconds, 17),
            other => panic!("expected RateLimited, got {other}"),
        }
    }

    #[test]
    fn a_429_without_a_header_still_backs_off() {
        assert!(matches!(
            classify("Cloudflare", StatusCode::TOO_MANY_REQUESTS, &HeaderMap::new(), ""),
            RegistrarError::RateLimited(_, 60)
        ));
    }

    #[test]
    fn an_unsupported_method_is_not_a_rejection() {
        assert!(matches!(
            classify("Namecheap", StatusCode::NOT_IMPLEMENTED, &HeaderMap::new(), ""),
            RegistrarError::Unsupported(_, _)
        ));
    }

    #[test]
    fn a_rejection_body_is_truncated() {
        let huge = "x".repeat(5_000);
        let err = classify("Dynadot", StatusCode::BAD_REQUEST, &HeaderMap::new(), &huge);
        assert!(err.to_string().chars().count() < 400);
    }

    #[test]
    fn a_client_is_built_with_a_timeout() {
        assert!(client("Porkbun").is_ok());
    }

    #[test]
    fn unreadable_json_is_a_decode_error_carrying_the_excerpt() {
        let err = parse("Porkbun", "<html>not json</html>").unwrap_err();
        assert!(matches!(err, RegistrarError::Decode(_, _)));
        assert!(err.to_string().contains("not json"));
    }
}
