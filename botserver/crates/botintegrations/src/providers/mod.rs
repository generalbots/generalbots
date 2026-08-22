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
        "password",
        "authorization",
    ]
    .iter()
    .any(|fragment| lower.contains(fragment))
}

/// Recursively strips keys whose names look like credential material so a
/// hostile provider response can never smuggle secrets into chat or API
/// payloads. S3 object keys are legitimate outcome data and are preserved.
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
        other => other.clone(),
    }
}

/// Adapters available in this build. Slice 1 ships AWS only.
pub fn registry() -> Vec<Arc<dyn ProviderAdapter>> {
    vec![Arc::new(aws::AwsAdapter)]
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
        let actions = llm_safe_actions("aws");
        let names: Vec<&str> = actions.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names.len(), implemented_action_names("aws").len());
        for key in implemented_action_names("aws") {
            assert!(names.contains(key), "missing chat-safe metadata for {key}");
        }
        assert!(names.contains(&"s3.objects.list"));

        let rendered = serde_json::to_string(&actions).expect("serialize in tests");
        for banned in [
            "secret",
            "token",
            "vault",
            "access_key",
            "password",
            "authorization",
        ] {
            assert!(
                !rendered.to_lowercase().contains(banned),
                "chat-safe metadata leaked {banned}"
            );
        }

        assert!(llm_safe_actions("nonexistent").is_empty());
    }
}
