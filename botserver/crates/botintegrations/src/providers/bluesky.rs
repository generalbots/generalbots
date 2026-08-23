use std::future::Future;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::pin::Pin;

use serde_json::{json, Value};

use crate::providers::rest_client::{self, RestRequest, MAX_RESPONSE_BYTES};
use crate::providers::{ActionOutcome, LlmSafeAction, LlmSafeParam, ProviderAdapter};

const ORIGIN: &str = "https://bsky.social";

fn param(name: &str, required: bool) -> LlmSafeParam {
    LlmSafeParam {
        name: name.to_string(),
        kind: "string".to_string(),
        required,
    }
}

pub const BLUESKY_IMPLEMENTED_ACTIONS: &[&str] = &[
    "bluesky.posts.list",
    "bluesky.posts.search",
    "bluesky.posts.create",
];

/// Bluesky AT Protocol adapter. Authenticates with an app password
/// (identifier + password) through createSession and caches the resulting
/// JWT in-process per handle.
pub struct BlueskyAdapter;

fn session_cache() -> &'static Mutex<HashMap<String, (String, String)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (String, String)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn access_jwt(
    http: &reqwest::Client,
    credentials: &Value,
) -> Result<String, String> {
    let identifier = credentials
        .get("identifier")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| rest_client::invalid("credential key identifier is missing".to_string()))?;
    let password = credentials
        .get("app_password")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| rest_client::invalid("credential key app_password is missing".to_string()))?;

    if let Ok(cache) = session_cache().lock() {
        if let Some((jwt, _handle)) = cache.get(identifier) {
            return Ok(jwt.clone());
        }
    }

    let payload = json!({ "identifier": identifier, "password": password.trim() });
    let response = http
        .post(format!("{ORIGIN}/xrpc/com.atproto.server.createSession"))
        .json(&payload)
        .send()
        .await
        .map_err(|error| {
            log::warn!("bluesky createSession failed: {error}");
            rest_client::ERR_NETWORK.to_string()
        })?;
    if !response.status().is_success() {
        log::warn!("bluesky createSession returned {}", response.status());
        return Err("provider_request_failed: bluesky session rejected".to_string());
    }
    let body: Value = response.json().await.map_err(|_| {
        "invalid_response: bluesky session malformed".to_string()
    })?;
    let jwt = body
        .get("accessJwt")
        .and_then(Value::as_str)
        .ok_or_else(|| "invalid_response: missing accessJwt".to_string())?
        .to_string();
    if let Ok(mut cache) = session_cache().lock() {
        cache.insert(identifier.to_string(), (jwt.clone(), identifier.to_string()));
    }
    Ok(jwt)
}

impl ProviderAdapter for BlueskyAdapter {
    fn provider(&self) -> &'static str {
        "bluesky"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        BLUESKY_IMPLEMENTED_ACTIONS
    }

    fn safe_action_catalog(&self) -> Vec<LlmSafeAction> {
        vec![
            LlmSafeAction {
                name: "bluesky.posts.list".to_string(),
                summary: "Listed the home timeline.".to_string(),
                params: vec![param("limit", false)],
                risk: "low".to_string(),
                requires_approval: false,
            },
            LlmSafeAction {
                name: "bluesky.posts.search".to_string(),
                summary: "Searched Bluesky posts.".to_string(),
                params: vec![param("query", true)],
                risk: "low".to_string(),
                requires_approval: false,
            },
            LlmSafeAction {
                name: "bluesky.posts.create".to_string(),
                summary: "Published a skeet.".to_string(),
                params: vec![param("text", true)],
                risk: "high".to_string(),
                requires_approval: true,
            },
        ]
    }

    fn invoke<'a>(
        &'a self,
        action_key: &'a str,
        credentials: &'a Value,
        params: &'a Value,
    ) -> Pin<Box<dyn Future<Output = Result<ActionOutcome, String>> + Send + 'a>> {
        Box::pin(async move {
            let http = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .map_err(|_| rest_client::ERR_NETWORK.to_string())?;
            let jwt = access_jwt(&http, credentials).await?;

            let (method, url, body): (reqwest::Method, String, Option<Value>) = match action_key {
                "bluesky.posts.list" => (
                    reqwest::Method::GET,
                    format!("{ORIGIN}/xrpc/app.bsky.feed.getTimeline?limit={}",
                        params.get("limit").and_then(Value::as_u64).unwrap_or(25)),
                    None,
                ),
                "bluesky.posts.search" => (
                    reqwest::Method::GET,
                    format!("{ORIGIN}/xrpc/app.bsky.feed.searchPosts?q={}",
                        urlencoding::encode(params.get("query").and_then(Value::as_str).unwrap_or_default())),
                    None,
                ),
                "bluesky.posts.create" => (
                    reqwest::Method::POST,
                    format!("{ORIGIN}/xrpc/com.atproto.server.createRecord"),
                    Some(json!({
                        "collection": "app.bsky.feed.post",
                        "repo": credentials.get("identifier"),
                        "record": {
                            "$type": "app.bsky.feed.post",
                            "text": params.get("text"),
                            "createdAt": chrono::Utc::now().to_rfc3339()
                        }
                    })),
                ),
                _ => return Err(crate::providers::ERR_ACTION_NOT_AVAILABLE.to_string()),
            };

            let response = rest_client::send(RestRequest {
                method,
                url,
                headers: vec![
                    ("authorization", format!("Bearer {jwt}")),
                    ("content-type", "application/json".to_string()),
                    ("user-agent", "generalbots-botintegrations".to_string()),
                ],
                body: body.map(|value| value.to_string().into_bytes()),
                response_cap: MAX_RESPONSE_BYTES,
            })
            .await?;
            response.require_success("Bluesky")?;
            let parsed = response.json("Bluesky")?;
            Ok(ActionOutcome {
                summary: format!("{} completed (status {})", action_key, response.status),
                data: parsed,
                truncated: false,
            })
        })
    }
}
