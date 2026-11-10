//! The one outbound fetch used for caller-supplied URLs (#2131).
//!
//! ```ignore
//! let page = botsecurity_core::validation::fetch_user_url(url).await?;
//! ```
//!
//! What the fetch loop guarantees, in order:
//!
//! 1. the URL parses and is `http`/`https`, with no userinfo and no
//!    control-plane port ([`validate_url_ssrf_strict`]);
//! 2. the host is resolved and **every** answer is global
//!    ([`resolve_checked`]) — a mixed answer set is refused;
//! 3. redirects are disabled on the client, so every hop is fetched by this
//!    loop and re-validated ([`validate_redirect_target`]) before it is sent;
//! 4. after connecting, the peer address is checked again
//!    (`Response::remote_addr`), which catches a resolver that changed its
//!    answer between step 2 and the connection (DNS rebinding);
//! 5. the body is read in chunks and truncated at [`FetchPolicy::max_bytes`],
//!    under one overall [`FetchPolicy::timeout`].
//!
//! The error returned to the caller is generic; the rejected host lives only
//! in [`SsrfError::detail`], which callers pass to `log::warn!`.

use std::time::Duration;
use url::Url;

use super::ssrf_allowlist::is_internal_service_host;
use super::ssrf_guard::{
    is_blocked_address, resolve_checked, validate_redirect_target, validate_url_ssrf_strict,
    SsrfError, SsrfErrorKind,
};

const DEFAULT_USER_AGENT: &str = concat!("GeneralBots/", env!("CARGO_PKG_VERSION"), " SSRFGuard");

/// Limits applied to one outbound fetch.
#[derive(Debug, Clone)]
pub struct FetchPolicy {
    /// Maximum number of body bytes kept. The remainder is discarded.
    pub max_bytes: usize,
    /// Wall-clock budget for the whole fetch, redirects included.
    pub timeout: Duration,
    /// Maximum redirect hops that will be followed.
    pub max_redirects: usize,
    /// `User-Agent` header sent with every hop.
    pub user_agent: String,
    /// Extra request headers (API tokens, `Accept`, …). Set only from
    /// server-side configuration, never from caller input.
    pub headers: Vec<(String, String)>,
}

impl Default for FetchPolicy {
    fn default() -> Self {
        Self {
            max_bytes: 2 * 1024 * 1024,
            timeout: Duration::from_secs(20),
            max_redirects: 5,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            headers: Vec::new(),
        }
    }
}

impl FetchPolicy {
    pub fn with_max_bytes(mut self, max_bytes: usize) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Adds one request header. Values come from server-side configuration.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

/// A fetched body plus the metadata the callers need.
#[derive(Debug, Clone)]
pub struct FetchedResponse {
    /// URL of the final hop (after redirects).
    pub final_url: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
    /// True when the body exceeded [`FetchPolicy::max_bytes`] and was cut.
    pub truncated: bool,
}

impl FetchedResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Lossy UTF-8 view of the body, for HTML/RSS/CSV callers.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    /// URL came from a caller (script, LLM, API) — full public-only policy.
    Caller,
    /// URL was built by the server from configuration for a documented
    /// internal service ([`super::ssrf_allowlist`]).
    InternalService,
}

fn check_target(url: &Url, target: Target) -> Result<(), SsrfError> {
    match target {
        Target::Caller => validate_url_ssrf_strict(url.as_str()).map(|_| ()),
        Target::InternalService => {
            let host = url
                .host_str()
                .ok_or_else(|| SsrfError::new(SsrfErrorKind::MissingHost, "URL has no host"))?;
            if !is_internal_service_host(host) {
                return Err(SsrfError::new(
                    SsrfErrorKind::BlockedHost,
                    format!("`{host}` is not an allowlisted internal service"),
                ));
            }
            if !matches!(url.scheme(), "http" | "https") {
                return Err(SsrfError::new(
                    SsrfErrorKind::UnsupportedScheme,
                    format!("scheme `{}` is not allowed", url.scheme()),
                ));
            }
            if !url.username().is_empty() || url.password().is_some() {
                return Err(SsrfError::new(
                    SsrfErrorKind::CredentialsInUrl,
                    "URL carries userinfo",
                ));
            }
            Ok(())
        }
    }
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn build_client(policy: &FetchPolicy) -> Result<reqwest::Client, SsrfError> {
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in &policy.headers {
        let header_name = reqwest::header::HeaderName::from_bytes(name.as_bytes()).map_err(|e| {
            SsrfError::new(SsrfErrorKind::Transport, format!("invalid header name: {e}"))
        })?;
        let header_value = reqwest::header::HeaderValue::from_str(value).map_err(|e| {
            SsrfError::new(SsrfErrorKind::Transport, format!("invalid header value: {e}"))
        })?;
        headers.insert(header_name, header_value);
    }
    reqwest::Client::builder()
        // Every hop goes through this module: no automatic redirects.
        .redirect(reqwest::redirect::Policy::none())
        .timeout(policy.timeout)
        .user_agent(policy.user_agent.clone())
        .default_headers(headers)
        .build()
        .map_err(|e| SsrfError::new(SsrfErrorKind::Transport, format!("http client: {e}")))
}

async fn read_body(
    mut response: reqwest::Response,
    max_bytes: usize,
) -> Result<(Vec<u8>, bool), SsrfError> {
    let mut body: Vec<u8> = Vec::with_capacity(max_bytes.min(64 * 1024));
    let mut truncated = false;
    loop {
        let chunk = response
            .chunk()
            .await
            .map_err(|e| SsrfError::new(SsrfErrorKind::Transport, format!("body read: {e}")))?;
        let Some(chunk) = chunk else { break };
        if body.len() + chunk.len() > max_bytes {
            let remaining = max_bytes.saturating_sub(body.len());
            body.extend_from_slice(&chunk[..remaining]);
            truncated = true;
            break;
        }
        body.extend_from_slice(&chunk);
    }
    Ok((body, truncated))
}

async fn follow(
    start: Url,
    policy: &FetchPolicy,
    target: Target,
) -> Result<FetchedResponse, SsrfError> {
    check_target(&start, target)?;
    let client = build_client(policy)?;
    let mut current = start;

    for hop in 0..=policy.max_redirects {
        if target == Target::Caller {
            resolve_checked(&current).await?;
        }

        let response = client.get(current.clone()).send().await.map_err(|e| {
            SsrfError::new(SsrfErrorKind::Transport, format!("request failed: {e}"))
        })?;

        // Post-connect check: the peer must be an address the policy approved,
        // which is what closes the DNS-rebinding window.
        if target == Target::Caller {
            if let Some(peer) = response.remote_addr() {
                if is_blocked_address(peer.ip()) {
                    return Err(SsrfError::new(
                        SsrfErrorKind::DnsRebinding,
                        format!("connected peer {peer} is non-global"),
                    ));
                }
            }
        }

        let status = response.status().as_u16();
        if is_redirect(status) {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    SsrfError::new(
                        SsrfErrorKind::InvalidRedirect,
                        format!("HTTP {status} without a usable Location header"),
                    )
                })?;
            if hop == policy.max_redirects {
                return Err(SsrfError::new(
                    SsrfErrorKind::RedirectLimit,
                    format!("more than {} redirects", policy.max_redirects),
                ));
            }
            let next = match target {
                Target::Caller => validate_redirect_target(&current, location)?,
                Target::InternalService => current.join(location).map_err(|e| {
                    SsrfError::new(
                        SsrfErrorKind::InvalidRedirect,
                        format!("unusable Location header: {e}"),
                    )
                })?,
            };
            check_target(&next, target)?;
            current = next;
            continue;
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());
        let (body, truncated) = read_body(response, policy.max_bytes).await?;
        return Ok(FetchedResponse {
            final_url: current.to_string(),
            status,
            content_type,
            body,
            truncated,
        });
    }

    Err(SsrfError::new(
        SsrfErrorKind::RedirectLimit,
        "redirect limit reached",
    ))
}

/// Fetches a URL that came from a caller (bot script, LLM, API request).
///
/// This is the only function that may be used for such URLs. It returns a
/// generic error on rejection; log [`SsrfError::detail`] server-side.
pub async fn fetch_user_url(url: &str, policy: &FetchPolicy) -> Result<FetchedResponse, SsrfError> {
    let parsed = validate_url_ssrf_strict(url)?;
    let result = tokio::time::timeout(policy.timeout, follow(parsed, policy, Target::Caller))
        .await
        .map_err(|_| {
            SsrfError::new(
                SsrfErrorKind::Transport,
                format!("fetch exceeded {}s", policy.timeout.as_secs()),
            )
        })?;
    result
}

/// Convenience wrapper over [`fetch_user_url`] with [`FetchPolicy::default`].
pub async fn fetch_user_url_default(url: &str) -> Result<FetchedResponse, SsrfError> {
    fetch_user_url(url, &FetchPolicy::default()).await
}

/// Fetches a server-built URL for a documented internal service.
///
/// Only hosts in [`super::ssrf_allowlist::INTERNAL_SERVICES`] are accepted.
/// Never pass a value that came from a request, a script or an LLM.
pub async fn fetch_internal_service_url(
    url: &str,
    policy: &FetchPolicy,
) -> Result<FetchedResponse, SsrfError> {
    let parsed = Url::parse(url).map_err(|e| {
        SsrfError::new(SsrfErrorKind::MissingHost, format!("unparsable URL: {e}"))
    })?;
    check_target(&parsed, Target::InternalService)?;
    tokio::time::timeout(
        policy.timeout,
        follow(parsed, policy, Target::InternalService),
    )
    .await
    .map_err(|_| {
        SsrfError::new(
            SsrfErrorKind::Transport,
            format!("fetch exceeded {}s", policy.timeout.as_secs()),
        )
    })?
}
