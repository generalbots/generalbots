//! Instagram adapter for the integration action plane (#950).
//!
//! Instagram publishing is not a single REST call: a media container is
//! created first and only then published, so the declarative generic engine
//! cannot express it. This adapter implements the documented two-step
//! container/publish flow plus the read actions, against the Instagram Graph
//! API reached through `graph.facebook.com`.
//!
//! The catalog entry for this provider is `Status::Partial` on purpose: the
//! Graph API exposes no media search and no media delete for Instagram, so
//! those catalog actions stay `implemented: false` instead of failing at
//! invocation time.
//!
//! Security contract (shared with the other adapters): credentials load from
//! Vault immediately before the call, never appear in URLs or logs, and every
//! outcome is redacted by the caller before reaching chat or API responses.

use std::future::Future;
use std::pin::Pin;

use serde_json::{json, Value};

use crate::providers::rest_client::{self, RestRequest, MAX_RESPONSE_BYTES};
use crate::providers::{ActionOutcome, LlmSafeAction, LlmSafeParam, ProviderAdapter};

const ORIGIN: &str = "https://graph.facebook.com/v21.0";
/// Graph page size ceiling for the `/media` edge.
const MAX_MEDIA_LIMIT: usize = 100;
const DEFAULT_MEDIA_LIMIT: usize = 25;
const MAX_CAPTION_LEN: usize = 2200;
const MAX_URL_LEN: usize = 2048;
/// Fields requested for listings; keeps responses inside the byte cap.
const MEDIA_FIELDS: &str = "id,caption,media_type,media_url,thumbnail_url,permalink,timestamp";

/// Credential envelope keys holding the Instagram business account id, in
/// resolution order. OAuth connections carry only the token, so the id is
/// usually discovered from the linked Facebook Page instead.
const ACCOUNT_ID_KEYS: &[&str] = &[
    "ig_user_id",
    "instagram_user_id",
    "instagram_account_id",
    "account_id",
];

/// Catalog action keys implemented by this adapter, mirroring the
/// `instagram.*` names expanded from `SOCIAL_ACTIONS` in
/// `botserver/src/apps/integration_catalog/actions/social.rs`. Keys stay
/// unprefixed because `integrations.invoke` receives `provider` and `action`
/// separately (`action = "posts.create"`), exactly like the AWS and GitHub
/// adapters.
pub const INSTAGRAM_IMPLEMENTED_ACTIONS: &[&str] = &[
    "posts.list",
    "posts.get",
    "posts.create",
];

fn param(name: &str, required: bool) -> LlmSafeParam {
    LlmSafeParam {
        name: name.to_string(),
        kind: "string".to_string(),
        required,
    }
}

/// Instagram Graph adapter. Long-lived Page or system-user tokens only; the
/// token is sent as an `Authorization: Bearer` header so it never appears in
/// a query string, access log or provider error echo.
pub struct InstagramAdapter;

fn bearer_headers(token: &str) -> Vec<(&'static str, String)> {
    vec![
        ("authorization", format!("Bearer {token}")),
        ("accept", "application/json".to_string()),
        ("user-agent", "generalbots-botintegrations".to_string()),
    ]
}

/// Reads the first non-empty string credential among `keys`.
fn cred_first(credentials: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        credentials
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn access_token(credentials: &Value) -> Result<String, String> {
    cred_first(credentials, &["token", "access_token"])
        .ok_or_else(|| rest_client::invalid("credential key token is missing".to_string()))
}

async fn graph_get(
    token: &str,
    url: String,
    what: &str,
) -> Result<(Value, u16), String> {
    let response = rest_client::send(RestRequest {
        method: reqwest::Method::GET,
        url,
        headers: bearer_headers(token),
        body: None,
        response_cap: MAX_RESPONSE_BYTES,
    })
    .await?;
    response.require_success(what)?;
    let status = response.status;
    Ok((response.json(what)?, status))
}

async fn graph_post(
    token: &str,
    url: String,
    what: &str,
) -> Result<Value, String> {
    let response = rest_client::send(RestRequest {
        method: reqwest::Method::POST,
        url,
        headers: bearer_headers(token),
        body: None,
        response_cap: MAX_RESPONSE_BYTES,
    })
    .await?;
    response.require_success(what)?;
    response.json(what)
}

/// Resolved publishing target: the Instagram business account id plus the
/// credential to call the Graph API with.
struct Target {
    account_id: String,
    token: String,
}

/// Credential envelope keys holding a Facebook Page token supplied directly
/// (system-user and manually connected envelopes).
const PAGE_TOKEN_KEYS: &[&str] = &["page_access_token", "facebook_page_token"];

/// Resolves what to act on. Instagram publishing requires a Facebook Page
/// token or a system-user token, while the OAuth flow stores a user token;
/// the Page token is therefore fetched alongside the account id from the
/// linked Page. Explicit credentials in the envelope always win.
async fn resolve_target(token: &str, credentials: &Value) -> Result<Target, String> {
    let explicit_id = cred_first(credentials, ACCOUNT_ID_KEYS);
    if let Some(page_token) = cred_first(credentials, PAGE_TOKEN_KEYS) {
        if let Some(account_id) = explicit_id {
            return Ok(Target {
                account_id,
                token: page_token,
            });
        }
    }

    // Discovery through the linked Page yields both values in one call.
    if let Ok((pages, _)) = graph_get(
        token,
        format!("{ORIGIN}/me/accounts?fields=instagram_business_account,access_token&limit=25"),
        "Instagram page discovery",
    )
    .await
    {
        if let Some(items) = pages.get("data").and_then(Value::as_array) {
            let linked = items.iter().find_map(|item| {
                let account_id = item
                    .pointer("/instagram_business_account/id")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())?;
                let page_token = item
                    .get("access_token")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())?;
                Some(Target {
                    account_id: account_id.to_string(),
                    token: page_token.to_string(),
                })
            });
            if let Some(target) = linked {
                return Ok(target);
            }
        }
    }

    if let Some(account_id) = explicit_id {
        return Ok(Target {
            account_id,
            token: token.to_string(),
        });
    }

    // A Page or system-user token resolves the account directly through /me.
    let (me, _) = graph_get(
        token,
        format!("{ORIGIN}/me?fields=instagram_business_account"),
        "Instagram account discovery",
    )
    .await?;
    let account_id = me
        .pointer("/instagram_business_account/id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            rest_client::invalid(
                "no Instagram business account is linked to this credential; link the Instagram account to a Facebook Page, or store ig_user_id together with page_access_token".to_string(),
            )
        })?;
    Ok(Target {
        account_id,
        token: token.to_string(),
    })
}

/// Validates one optional body field as a bounded string URL.
fn optional_url(data: &Value, key: &str) -> Result<Option<String>, String> {
    let value = match data.get(key) {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(text)) => text.trim().to_string(),
        Some(_) => return Err(rest_client::invalid(format!("{key} must be a string"))),
    };
    if value.is_empty() {
        return Ok(None);
    }
    if value.len() > MAX_URL_LEN {
        return Err(rest_client::invalid(format!(
            "{key} must be at most {MAX_URL_LEN} characters"
        )));
    }
    if !value.starts_with("https://") && !value.starts_with("http://") {
        return Err(rest_client::invalid(format!(
            "{key} must be an absolute http(s) URL that Meta can fetch"
        )));
    }
    Ok(Some(value))
}

async fn create_post(
    token: &str,
    account_id: &str,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let data = params.get("data").cloned().unwrap_or(Value::Null);
    if !data.is_object() {
        return Err(rest_client::invalid(
            "data must be a JSON object with image_url or video_url".to_string(),
        ));
    }
    let image_url = optional_url(&data, "image_url")?;
    let video_url = optional_url(&data, "video_url")?;
    if image_url.is_none() && video_url.is_none() {
        return Err(rest_client::invalid(
            "either image_url or video_url is required".to_string(),
        ));
    }
    let caption = match data.get("caption") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => {
            let trimmed = text.trim().to_string();
            if trimmed.chars().count() > MAX_CAPTION_LEN {
                return Err(rest_client::invalid(format!(
                    "caption must be at most {MAX_CAPTION_LEN} characters"
                )));
            }
            (!trimmed.is_empty()).then_some(trimmed)
        }
        Some(_) => return Err(rest_client::invalid("caption must be a string".to_string())),
    };

    let mut container = format!("{ORIGIN}/{account_id}/media");
    let mut query = String::new();
    if let Some(url) = &image_url {
        rest_client::push_query_pair(&mut query, "image_url", url);
    }
    if let Some(url) = &video_url {
        rest_client::push_query_pair(&mut query, "video_url", url);
        // A video needs an explicit container type; Meta rejects the default.
        let media_type = data
            .get("media_type")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| matches!(*value, "REELS" | "STORIES"))
            .ok_or_else(|| {
                rest_client::invalid(
                    "media_type must be REELS or STORIES when video_url is used".to_string(),
                )
            })?;
        rest_client::push_query_pair(&mut query, "media_type", media_type);
    }
    if let Some(caption) = &caption {
        rest_client::push_query_pair(&mut query, "caption", caption);
    }
    container.push('?');
    container.push_str(&query);

    let created = graph_post(token, container, "Instagram media container").await?;
    let creation_id = created
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "invalid_response: Instagram returned no creation id".to_string())?;

    let mut publish = format!("{ORIGIN}/{account_id}/media_publish");
    let mut publish_query = String::new();
    rest_client::push_query_pair(&mut publish_query, "creation_id", creation_id);
    publish.push('?');
    publish.push_str(&publish_query);

    let published = graph_post(token, publish, "Instagram media publish").await?;
    let media_id = published
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| creation_id.to_string());

    Ok(ActionOutcome {
        summary: "Published an Instagram media post.".to_string(),
        data: json!({
            "id": media_id,
            "creation_id": creation_id,
            "account_id": account_id,
            "media_type": if video_url.is_some() { "VIDEO" } else { "IMAGE" },
        }),
        truncated: false,
    })
}

impl ProviderAdapter for InstagramAdapter {
    fn provider(&self) -> &'static str {
        "instagram"
    }

    fn implemented_actions(&self) -> &'static [&'static str] {
        INSTAGRAM_IMPLEMENTED_ACTIONS
    }

    fn safe_action_catalog(&self) -> Vec<LlmSafeAction> {
        vec![
            LlmSafeAction {
                name: "posts.list".to_string(),
                summary: "Listed Instagram media for the connected business account.".to_string(),
                params: vec![param("limit", false)],
                risk: "low".to_string(),
                requires_approval: false,
            },
            LlmSafeAction {
                name: "posts.get".to_string(),
                summary: "Read one Instagram media item and its engagement.".to_string(),
                params: vec![param("resource_id", true)],
                risk: "low".to_string(),
                requires_approval: false,
            },
            LlmSafeAction {
                name: "posts.create".to_string(),
                summary: "Published an image or video post to Instagram.".to_string(),
                params: vec![LlmSafeParam {
                    name: "data".to_string(),
                    kind: "json".to_string(),
                    required: true,
                }],
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
            let token = access_token(credentials)?;
            match action_key {
                "posts.list" => {
                    let target = resolve_target(&token, credentials).await?;
                    let limit = rest_client::bounded_limit(
                        params,
                        "limit",
                        DEFAULT_MEDIA_LIMIT,
                        MAX_MEDIA_LIMIT,
                    )?;
                    let url = format!(
                        "{ORIGIN}/{}/media?fields={MEDIA_FIELDS}&limit={limit}",
                        target.account_id
                    );
                    let (body, status) =
                        graph_get(&target.token, url, "Instagram media list").await?;
                    Ok(ActionOutcome {
                        summary: format!("Listed Instagram media (status {status})."),
                        data: body,
                        truncated: false,
                    })
                }
                "posts.get" => {
                    let media_id = rest_client::required_text(params, "resource_id", 200)?;
                    let target = resolve_target(&token, credentials).await?;
                    let url = format!(
                        "{ORIGIN}/{media_id}?fields={MEDIA_FIELDS},like_count,comments_count"
                    );
                    let (body, status) =
                        graph_get(&target.token, url, "Instagram media read").await?;
                    Ok(ActionOutcome {
                        summary: format!("Read Instagram media {media_id} (status {status})."),
                        data: body,
                        truncated: false,
                    })
                }
                "posts.create" => {
                    let target = resolve_target(&token, credentials).await?;
                    create_post(&target.token, &target.account_id, params).await
                }
                _ => Err(crate::providers::ERR_ACTION_NOT_AVAILABLE.to_string()),
            }
        })
    }
}
