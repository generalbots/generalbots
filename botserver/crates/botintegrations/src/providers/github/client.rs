//! GitHub REST request policy for the provider adapter (#950 slice 2).
//!
//! Credentials arrive strictly from the Vault envelope loaded by
//! [`crate::providers::invoke_registered`]; they are never logged and never
//! included in error strings. Every call targets `https://api.github.com`
//! with the pinned REST API version header and a descriptive user agent.

use serde_json::Value;

use crate::providers::rest_client::{self, RestRequest, MAX_RESPONSE_BYTES};

const API_ORIGIN: &str = "https://api.github.com";
/// Pinned GitHub REST API version advertised through `X-GitHub-Api-Version`.
const API_VERSION: &str = "2022-11-28";
const ACCEPT_HEADER: &str = "application/vnd.github+json";
const USER_AGENT: &str = "generalbots-botintegrations";

/// Maximum accepted personal access token length; real tokens are far
/// shorter, so anything larger is treated as a misconfigured envelope.
const MAX_TOKEN_LEN: usize = 512;

/// Parsed GitHub credential envelope from Vault.
#[derive(Debug, Clone)]
pub(crate) struct GithubCreds {
    pub(crate) token: String,
}

impl GithubCreds {
    /// Validates the required `token` key before any network activity.
    /// Tokens may be classic (`ghp_...`), fine-grained (`github_pat_...`),
    /// OAuth or installation tokens, so only charset and length are checked:
    /// printable ASCII word characters without whitespace.
    pub(crate) fn parse(credentials: &Value) -> Result<Self, String> {
        let object = match credentials {
            Value::Object(map) => map,
            _ => {
                return Err(rest_client::invalid(
                    "stored credential envelope must be an object".to_string(),
                ))
            }
        };
        let raw = object
            .get("token")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| {
                rest_client::invalid("credential key token is missing or empty".to_string())
            })?;
        if raw.len() > MAX_TOKEN_LEN {
            return Err(rest_client::invalid(format!(
                "credential key token must be at most {MAX_TOKEN_LEN} characters"
            )));
        }
        if !raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(rest_client::invalid(
                "credential key token contains invalid characters".to_string(),
            ));
        }
        Ok(Self {
            token: raw.to_string(),
        })
    }
}

fn authed_headers(creds: &GithubCreds) -> Vec<(&'static str, String)> {
    vec![
        ("authorization", format!("Bearer {}", creds.token)),
        ("accept", ACCEPT_HEADER.to_string()),
        ("x-github-api-version", API_VERSION.to_string()),
        ("user-agent", USER_AGENT.to_string()),
    ]
}

/// Sends an authenticated GET against the GitHub REST API.
pub(crate) async fn get(
    creds: &GithubCreds,
    path_and_query: &str,
) -> Result<rest_client::RestResponse, String> {
    rest_client::send(RestRequest {
        method: reqwest::Method::GET,
        url: format!("{API_ORIGIN}{path_and_query}"),
        headers: authed_headers(creds),
        body: None,
        response_cap: MAX_RESPONSE_BYTES,
    })
    .await
}

/// Sends an authenticated JSON-body request (POST or PATCH) and returns the
/// response. The payload is capped to keep outbound bodies bounded.
pub(crate) async fn send_json(
    creds: &GithubCreds,
    method: reqwest::Method,
    path_and_query: &str,
    payload: Value,
) -> Result<rest_client::RestResponse, String> {
    let mut headers = authed_headers(creds);
    headers.push(("content-type", "application/json".to_string()));
    rest_client::send(RestRequest {
        method,
        url: format!("{API_ORIGIN}{path_and_query}"),
        headers,
        body: Some(payload.to_string().into_bytes()),
        response_cap: MAX_RESPONSE_BYTES,
    })
    .await
}
