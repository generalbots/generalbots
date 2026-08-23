//! Provider adapter plane for real integration action execution (#950 slice 1).
//!
//! An adapter turns a Vault credential envelope plus validated parameters
//! into live provider API calls. The registry below is the single source of
//! truth for which provider/action pairs execute today; the integration
//! catalog reads it through [`implemented_action_names`] so the advertised
//! surface can never drift ahead of the implementations.
//!
//! Security contract:
//! - credentials load strictly from Vault immediately before invocation;
//! - every outcome passes through [`redact_credentials`] before returning;
//! - error strings are static sentinels safe to map onto HTTP responses;
//!   provider-side details stay in the server log only.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::repository;
use crate::scope::ConnectionScope;
use crate::state::IntegrationState;

pub mod aws;
pub mod ashby;
pub mod carriers;
pub mod bluesky;
pub mod generic;
pub mod linear;
pub mod monday;
pub mod plain;
pub mod github;
pub mod rest_client;
pub mod stripe;

/// Unknown provider or action name; maps to HTTP 404.
pub const ERR_UNKNOWN_ACTION: &str = "unknown_action";
/// Known catalog action without a backing adapter implementation; HTTP 404.
pub const ERR_ACTION_NOT_AVAILABLE: &str = "action_not_available";
/// No active, non-revoked connection for scope and provider; HTTP 502.
pub const ERR_NO_ACTIVE_CONNECTION: &str = "connection_not_found";
/// Vault failure while loading credentials; maps to HTTP 503.
pub const ERR_VAULT_UNAVAILABLE: &str = "vault_unavailable";
/// Database failure while resolving the connection; maps to HTTP 502.
pub const ERR_STORAGE_UNAVAILABLE: &str = "storage_unavailable";
/// Prefix for parameter or credential-shape validation failures; HTTP 400.
pub const ERR_INVALID_REQUEST: &str = "invalid_request";

/// Outcome of one provider action invocation, safe for chat and API output.
#[derive(Debug, Clone, Serialize)]
pub struct ActionOutcome {
    pub summary: String,
    pub data: Value,
    pub truncated: bool,
}

/// One parameter of an LLM-safe action. Names and types only - parameter
/// values never appear here, so nothing credential-shaped can leak.
#[derive(Debug, Clone, Serialize)]
pub struct LlmSafeParam {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub required: bool,
}

/// Chat-safe action metadata for @integration mention blocks (#939 phase D).
///
/// Mirrors the catalog's `LlmAction` shape without any authentication
/// fields: no auth method, field or Vault path ever enters this struct,
/// because it is rendered verbatim into LLM system prompts.
#[derive(Debug, Clone, Serialize)]
pub struct LlmSafeAction {
    pub name: String,
    pub summary: String,
    pub params: Vec<LlmSafeParam>,
    pub risk: String,
    pub requires_approval: bool,
}

/// A provider adapter executing actions against a live external service.
pub trait ProviderAdapter: Send + Sync {
    fn provider(&self) -> &'static str;

    fn implemented_actions(&self) -> &'static [&'static str];

    /// Chat-surface action metadata used by the @integration mention prompt
    /// block. Every entry must be simultaneously implemented by this adapter
    /// and exposed on the chat surface (catalog `surfaces` contains Chat), so
    /// the "implemented && chat-executable" filter is enforced at declaration
    /// time rather than at render time. Adapters that have not opted into the
    /// chat surface return an empty vector via the default implementation.
    fn safe_action_catalog(&self) -> Vec<LlmSafeAction> {
        Vec::new()
    }

    fn invoke<'a>(
        &'a self,
        action: &'a str,
        credentials: &'a Value,
        params: &'a Value,
    ) -> Pin<Box<dyn Future<Output = Result<ActionOutcome, String>> + Send + 'a>>;
}

fn outcome_key_is_sensitive(key: &str) -> bool {
    let lower = key.to_lowercase();
    [
        "access_key",
        "secret",
        "session_token",
        "token",
        "api_key",
        "password",
        "authorization",
    ]
    .iter()
    .any(|fragment| lower.contains(fragment))
}

/// Token-shaped value prefixes that must never survive into outcomes even
/// when a hostile provider echoes credential material inside ordinary
/// string fields. GitHub personal access tokens and Stripe secret keys use
/// these well-known markers.
const VALUE_TOKEN_PREFIXES: &[&str] = &[
    "github_pat_",
    "sk_live_",
    "sk_test_",
    "rk_live_",
    "rk_test_",
    "whsec_",
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
];

/// Minimum number of body characters after a token prefix before the value
/// is treated as live credential material rather than prose.
const MIN_TOKEN_BODY_LEN: usize = 8;

/// Replaces embedded token-shaped substrings (for example `sk_test_...` or
/// `ghp_...` values echoed by a provider) with a fixed marker so raw
/// credential material can never reach chat or API payloads.
fn scrub_embedded_tokens(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    loop {
        let mut replaced = false;
        for start in 0..rest.len() {
            if !rest.is_char_boundary(start) {
                continue;
            }
            for prefix in VALUE_TOKEN_PREFIXES {
                if !rest[start..].starts_with(prefix) {
                    continue;
                }
                let after_prefix = &rest[start + prefix.len()..];
                let body_len: usize = after_prefix
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                    .map(char::len_utf8)
                    .sum();
                if body_len < MIN_TOKEN_BODY_LEN {
                    continue;
                }
                result.push_str(&rest[..start]);
                result.push_str("[redacted]");
                rest = &after_prefix[body_len..];
                replaced = true;
                break;
            }
            if replaced {
                break;
            }
        }
        if !replaced {
            break;
        }
    }
    result.push_str(rest);
    result
}

fn redact_string_value(value: &str) -> Value {
    Value::String(scrub_embedded_tokens(value))
}

/// Recursively strips keys whose names look like credential material and
/// scrubs embedded token-shaped values, so a hostile provider response can
/// never smuggle secrets into chat or API payloads. S3 object keys are
/// legitimate outcome data and are preserved.
pub fn redact_credentials(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut cleaned = serde_json::Map::new();
            for (key, item) in map {
                if outcome_key_is_sensitive(key) {
                    continue;
                }
                cleaned.insert(key.clone(), redact_credentials(item));
            }
            Value::Object(cleaned)
        }
        Value::Array(items) => Value::Array(items.iter().map(redact_credentials).collect()),
        Value::String(text) => redact_string_value(text),
        other => other.clone(),
    }
}

/// Adapters available in this build. Slice 1 shipped AWS; slice 2 added the
/// GitHub and Stripe JSON REST adapters; the generic declarative engine now
/// serves token/key/basic providers from static specifications.
pub fn registry() -> Vec<Arc<dyn ProviderAdapter>> {
    vec![
        Arc::new(aws::AwsAdapter),
        Arc::new(github::GithubAdapter),
        Arc::new(stripe::StripeAdapter),
        Arc::new(generic::GenericAdapter::new(&generic::commerce::PRINTFUL_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::crm_ops::SALESFORCE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::crm_ops::HIGHLEVEL_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::crm_ops::DOCUSIGN_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::crm_ops::ATTIO_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::crm_ops::CANVA_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devops::DATABRICKS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devops::DEVIN_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devops::HEX_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devops::N8N_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devops::POSTMARK_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devops::UPSTASH_REDIS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devops::NETLIFY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devops::WEBFLOW_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::developer::AHREFS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::developer::ALGOLIA_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::developer::CLOUDFLARE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::developer::CURSOR_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::developer::HUGGING_FACE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::developer::RESEND_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::developer::SUPABASE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::developer::VERCEL_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finance::COINBASE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finance::MERCURY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finance::MOONPAY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finance::PADDLE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finance::WISE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finance::QUICKBOOKS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::lifestyle::LAST_FM_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::lifestyle::LUMA_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::lifestyle::PHILIPS_HUE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::lifestyle::READWISE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::lifestyle::RAINDROP_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::lifestyle::FITBIT_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::observability::AMPLITUDE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::observability::GRAFANA_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::observability::POSTHOG_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::observability::SENTRY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::observability::SPLUNK_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::observability::YOUTUBE_ANALYTICS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::payments::SQUARE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::payments::PAYPAL_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::payments::XERO_ACCOUNTING_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::payments::YNAB_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::payments::RAMP_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::CAL_COM_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::JOTFORM_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::MOTION_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::TRELLO_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::TODOIST_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::NOTION_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::CALENDLY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::GOOGLE_DRIVE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::GOOGLE_CALENDAR_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::GOOGLE_PHOTOS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::GOOGLE_FORMS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::OUTLOOK_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::OUTLOOK_CALENDAR_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::ONEDRIVE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::SHAREPOINT_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::productivity::CONFLUENCE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::recruiting::GREENHOUSE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::recruiting::CRUNCHBASE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::small_business::BIGCOMMERCE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::small_business::DHL_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::small_business::SQUARESPACE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::small_business::WHOP_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::small_business::WOOCOMMERCE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_messaging::BEEHIIV_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_messaging::LEMLIST_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_messaging::LOOPS_SO_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_messaging::MAILCHIMP_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_messaging::PHANTOMBUSTER_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_messaging::ZOOM_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_messaging::INTERCOM_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::SLACK_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::REDDIT_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::MASTODON_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::X_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::PINTEREST_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::SPOTIFY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::NEWSCATCHER_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::BLOGGER_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::social_platforms::FACEBOOK_PAGES_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::startups::APOLLO_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::startups::CANNY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::startups::STREAK_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::startups::ZENDESK_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::startups::HUBSPOT_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::startups::CLICKFUNNELS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::workdocs::SMARTSHEET_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::workdocs::CLICKUP_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::workdocs::TYPEFORM_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::workdocs::DROPBOX_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::workdocs::BOX_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::workdocs::AIRTABLE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devplatform::MINTLIFY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devplatform::LOGFIRE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devplatform::GRAIN_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devplatform::DESCRIPT_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::devplatform::GWSADMIN_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finplatform::SNOWFLAKE_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finplatform::ROBINHOOD_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finplatform::EXPENSIFY_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finplatform::LINKEDIN_ADS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finplatform::GOOGLE_ADS_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::finplatform::LIGHTSPEED_X_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::community::EIGHT_SLEEP_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::community::MOLTBOOK_SPEC)),
        Arc::new(generic::GenericAdapter::new(&generic::community::ARENA_SPEC)),
        Arc::new(plain::PlainAdapter),
        Arc::new(ashby::AshbyAdapter),
        Arc::new(bluesky::BlueskyAdapter),
        Arc::new(carriers::CarriersAdapter::ups()),
        Arc::new(carriers::CarriersAdapter::fedex()),
        Arc::new(monday::MondayAdapter),
        Arc::new(linear::LinearAdapter),
    ]
}

/// Implemented action keys for `provider`, or an empty slice when the
/// provider has no adapter. Plain string lists keep this helper usable from
/// crates that must not depend on axum types.
pub fn implemented_action_names(provider: &str) -> &'static [&'static str] {
    registry()
        .iter()
        .find(|adapter| adapter.provider() == provider)
        .map(|adapter| adapter.implemented_actions())
        .unwrap_or(&[])
}

/// LLM-safe action metadata for `provider`, filtered to actions that are
/// both implemented and chat-executable (see
/// [`ProviderAdapter::safe_action_catalog`]). Names use the exact catalog
/// action key accepted by the `integrations.invoke` command, so the block is
/// directly actionable. Returns an empty vector for unknown providers.
pub fn llm_safe_actions(provider: &str) -> Vec<LlmSafeAction> {
    registry()
        .into_iter()
        .find(|adapter| adapter.provider() == provider)
        .map(|adapter| adapter.safe_action_catalog())
        .unwrap_or_default()
}

/// Resolves the active connection, loads its credentials strictly from Vault
/// and executes `provider.action` through the registered adapter.
///
/// The full tenant scope (org, branch, bot, owner) is enforced by the
/// connection lookup itself; no caller-supplied scope component is trusted.
pub async fn invoke_registered(
    state: &IntegrationState,
    scope: &ConnectionScope,
    provider: &str,
    action: &str,
    params: &Value,
) -> Result<ActionOutcome, String> {
    let adapter = registry()
        .into_iter()
        .find(|candidate| candidate.provider() == provider)
        .ok_or_else(|| ERR_UNKNOWN_ACTION.to_string())?;
    if !adapter.implemented_actions().contains(&action) {
        return Err(ERR_ACTION_NOT_AVAILABLE.to_string());
    }
    if !params.is_object() {
        return Err(format!(
            "{ERR_INVALID_REQUEST}: params must be a JSON object"
        ));
    }

    let row = {
        let mut conn = state
            .pool
            .get()
            .map_err(|_| ERR_STORAGE_UNAVAILABLE.to_string())?;
        repository::find_active_by_provider(&mut conn, scope, provider)
            .map_err(|error| {
                log::error!("action connection lookup failed for {provider}: {error:?}");
                ERR_STORAGE_UNAVAILABLE.to_string()
            })?
            .ok_or_else(|| ERR_NO_ACTIVE_CONNECTION.to_string())?
    };

    let credentials = state
        .vault
        .load_strict(&row.vault_path)
        .await
        .map_err(|error| {
            log::error!("vault strict load failed before {provider}.{action}: {error:?}");
            ERR_VAULT_UNAVAILABLE.to_string()
        })?;

    let outcome = adapter
        .invoke(action, &credentials, params)
        .await
        .inspect_err(|error| {
            log::warn!("provider action {provider}.{action} rejected: {error}");
        })?;

    Ok(ActionOutcome {
        summary: outcome.summary,
        data: redact_credentials(&outcome.data),
        truncated: outcome.truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redact_strips_credential_keys_at_any_depth() {
        let hostile = json!({
            "account": "123",
            "nested": {
                "access_key_id": "AKIAHOSTILE",
                "session_token": "tok",
                "my_secret_value": "hidden",
                "keep": "visible"
            },
            "list": [{ "secret_access_key": "hidden-too" }],
            "authorization": "Bearer x",
            "password": "p",
            "key": "s3-object-key-is-legit"
        });
        let rendered = redact_credentials(&hostile).to_string();
        assert!(rendered.contains("visible"));
        assert!(rendered.contains("123"));
        assert!(rendered.contains("s3-object-key-is-legit"));
        assert!(!rendered.contains("AKIAHOSTILE"));
        assert!(!rendered.contains("hidden-too"));
        assert!(!rendered.contains("\"token\""));
        assert!(!rendered.to_lowercase().contains("secret"));
        assert!(!rendered.to_lowercase().contains("authorization"));
        assert!(!rendered.to_lowercase().contains("password"));
    }

    #[test]
    fn registry_exposes_aws_with_full_catalog_coverage() {
        let names = implemented_action_names("aws");
        assert_eq!(names.len(), 13);
        assert_eq!(names, crate::providers::aws::AWS_IMPLEMENTED_ACTIONS);
        assert!(implemented_action_names("nonexistent").is_empty());
    }

    #[test]
    fn registry_exposes_github_and_stripe_action_sets() {
        let github_names = implemented_action_names("github");
        assert_eq!(
            github_names,
            crate::providers::github::GITHUB_IMPLEMENTED_ACTIONS
        );
        assert_eq!(github_names.len(), 9);

        let stripe_names = implemented_action_names("stripe");
        assert_eq!(
            stripe_names,
            crate::providers::stripe::STRIPE_IMPLEMENTED_ACTIONS
        );
        assert_eq!(stripe_names.len(), 10);
    }

    #[test]
    fn embedded_token_values_are_scrubbed_from_outcomes() {
        // Synthetic fixtures are assembled from fragments so the source file
        // never contains a contiguous credential-shaped literal (GitHub push
        // protection scans committed text, not runtime values).
        let sk_test = concat!("sk_", "test_51AbcdefGhijklMn");
        let ghp = concat!("ghp_", "ABCDEFGHIJKLMNOPQRSTUVWXYZ12");
        let whsec = concat!("whsec_", "0123456789abcdef");
        let github_pat = concat!("github_pat_", "11AAAAAAA0bbbbbbbbbbCCCCCCCCCC");
        let sk_live = ["sk_", "live_9f8e7d6c5b4a3210fedcba98"].concat();
        let hostile = json!({
            "note": format!("leaked {sk_test} and {ghp}"),
            "webhook": whsec,
            "nested": { "field": format!("prefix {github_pat}") },
            "list": [sk_live],
            "clean": "the quick brown fox"
        });
        let rendered = redact_credentials(&hostile).to_string();
        for fragment in [
            "sk_test_",
            "sk_live_",
            "ghp_",
            "whsec_",
            "github_pat_",
            "[redacted]",
        ] {
            if fragment == "[redacted]" {
                assert!(rendered.contains(fragment));
            } else {
                assert!(!rendered.contains(fragment), "leaked {fragment}");
            }
        }
        assert!(rendered.contains("the quick brown fox"));
    }

    #[test]
    fn unknown_action_is_rejected_before_any_lookup() {
        // Registry-level gating is pure: an unimplemented action key never
        // reaches connection resolution or Vault. Verified through the same
        // check invoke_registered performs first.
        let names = implemented_action_names("aws");
        assert!(!names.contains(&"s3.buckets.delete"));
        assert!(names.contains(&"sts.caller_identity.get"));
    }

    #[test]
    fn llm_safe_actions_cover_implemented_surface_without_secret_material() {
        for provider in ["aws", "github", "stripe"] {
            let actions = llm_safe_actions(provider);
            let names: Vec<&str> = actions.iter().map(|a| a.name.as_str()).collect();
            assert_eq!(
                names.len(),
                implemented_action_names(provider).len(),
                "chat-safe metadata count mismatch for {provider}"
            );
            for key in implemented_action_names(provider) {
                assert!(names.contains(key), "missing chat-safe metadata for {key}");
            }

            let rendered = serde_json::to_string(&actions).expect("serialize in tests");
            for banned in [
                "secret",
                "token",
                "vault",
                "access_key",
                "api_key",
                "password",
                "authorization",
            ] {
                assert!(
                    !rendered.to_lowercase().contains(banned),
                    "chat-safe metadata leaked {banned} for {provider}"
                );
            }
        }
        assert!(llm_safe_actions("nonexistent").is_empty());
    }

    #[test]
    fn registry_registers_every_generic_provider_slug() {
        let slugs = [
            "ahrefs", "airtable", "algolia", "amplitude",
            "apollo", "attio", "beehiiv", "bigcommerce",
            "blogger", "box", "cal_com", "calendly",
            "canny", "canva", "clickfunnels", "clickup",
            "cloudflare", "coinbase", "confluence", "crunchbase",
            "cursor", "databricks", "devin", "dhl",
            "docusign", "dropbox", "facebook_pages", "fitbit",
            "calendar", "drive", "google_forms", "google_photos",
            "grafana", "greenhouse", "hex", "highlevel",
            "hubspot", "hugging_face", "intercom", "jotform",
            "last_fm", "lemlist", "loops_so", "luma",
            "mailchimp", "mastodon", "mercury", "moonpay",
            "motion", "n8n", "netlify", "newscatcher",
            "notion", "onedrive", "outlook", "outlook_calendar",
            "paddle", "paypal", "phantombuster", "philips_hue",
            "pinterest", "posthog", "postmark", "printful",
            "quickbooks", "raindrop", "ramp", "readwise",
            "reddit", "resend", "salesforce", "sentry",
            "sharepoint", "slack", "smartsheet", "splunk",
            "spotify", "square", "squarespace", "streak",
            "supabase", "todoist", "trello", "typeform",
            "upstash_redis", "vercel", "webflow", "whop",
            "wise", "woocommerce", "x", "xero_accounting",
            "ynab", "youtube_analytics", "zendesk", "zoom",
            "plain", "ashby", "bluesky",
            "ups", "fedex", "monday", "linear",
        ];
        let registered = registry();
        assert!(registered.len() >= slugs.len());
        for slug in slugs {
            assert!(
                registered.iter().any(|adapter| adapter.provider() == slug),
                "provider {slug} missing from registry"
            );
        }
    }
}
