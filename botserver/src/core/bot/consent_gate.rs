//! Consent gate for agent-initiated app commands (issue #1176).
//! Holds the process-wide `ConsentService` and offers a single decision
//! entry point used by the chat command path and the api/ui loopback executor.
use std::sync::{Arc, OnceLock};
// #2448 criterion 4 — scoped, single-use agent-batch undo data model.
#[path = "undo/mod.rs"]
pub mod undo;

static SERVICE: OnceLock<Arc<botconsent::ConsentService>> = OnceLock::new();

pub fn init(pool: botconsent::DbPool) {
    let service = Arc::new(botconsent::ConsentService::new(pool));
    service.ensure_sweeper();
    let _ = SERVICE.set(service);
}

fn service() -> Option<&'static Arc<botconsent::ConsentService>> {
    SERVICE.get()
}

/// The one process-wide service. The HTTP routes MUST mount this instance:
/// creating a second `ConsentService` gives it an empty pending map, so every
/// prompt the gate raised resolved to "Consent request not found" (#2194).
pub fn shared_service() -> Option<Arc<botconsent::ConsentService>> {
    SERVICE.get().cloned()
}

pub struct GateOutcome {
    pub allowed: bool,
    pub request_id: Option<String>,
    pub pending_request: Option<botconsent::PendingRequest>,
    pub card_html: Option<String>,
    /// Set when this decision consumed an "Allow once" approval. The payload
    /// the user approved travels with it so the executor can recover the
    /// parameters the model dropped on its retry (#2194: the retry text is
    /// "continue", which carries no amount, so `service.tax` ran without a
    /// `value` and failed after the user had already approved it).
    pub approved_detail: Option<serde_json::Value>,
}

/// Decide whether `user_id` may run `action_class` on `app_id`.
/// When the consent service is not initialized (feature disabled at boot)
/// the gate is open, preserving legacy behavior.
pub async fn check(
    user_id: uuid::Uuid,
    app_id: &str,
    action_class: &str,
    detail: serde_json::Value,
) -> GateOutcome {
    check_with_context(user_id, app_id, action_class, detail, None, None).await
}

/// Same decision with the run/thread that originated it, so "this run" grants
/// and the audit trail can be correlated.
pub async fn check_with_context(
    user_id: uuid::Uuid,
    app_id: &str,
    action_class: &str,
    detail: serde_json::Value,
    thread_id: Option<String>,
    run_id: Option<String>,
) -> GateOutcome {
    let Some(service) = service() else {
        return GateOutcome {
            allowed: true,
            request_id: None,
            pending_request: None,
            card_html: None,
            approved_detail: None,
        };
    };
    // An "Allow once" click is a one-shot handoff: consume it before the
    // grant lookup, otherwise the model's retry raised a fresh prompt and the
    // approved call never ran (#2203).
    if let Some(approved) =
        botconsent::enforce::take_approved_match(service, user_id, app_id, action_class).await
    {
        return GateOutcome {
            allowed: true,
            request_id: None,
            pending_request: None,
            card_html: None,
            approved_detail: Some(approved.detail),
        };
    }
    let context = botconsent::enforce::ConsentContext {
        thread_id,
        run_id,
        workspace_id: None,
    };
    match botconsent::enforce::authorize_with_context(
        service,
        user_id,
        app_id,
        action_class,
        detail,
        &context,
    )
    .await
    {
        botconsent::enforce::ConsentDecision::Granted(_) => GateOutcome {
            allowed: true,
            request_id: None,
            pending_request: None,
            card_html: None,
            approved_detail: None,
        },
        botconsent::enforce::ConsentDecision::Pending(request) => {
            let card = botconsent::cards::prompt_card_html(&request);
            GateOutcome {
                allowed: false,
                request_id: Some(request.request_id.clone()),
                pending_request: Some(request),
                card_html: Some(card),
                approved_detail: None,
            }
        }
        botconsent::enforce::ConsentDecision::Denied => GateOutcome {
            allowed: false,
            request_id: None,
            pending_request: None,
            card_html: None,
            approved_detail: None,
        },
    }
}

pub fn deny_message(app_id: &str) -> String {
    format!(
        "This action on '{app_id}' requires your permission. Approve it in Settings > App permissions."
    )
}

/// Pure predicate: does this class require the explicit confirm/cancel gate
/// instead of the permission-grant buttons (#2448 criterion 3)? Destructive
/// `delete` is consent-per-call: `authorize_with_context` never satisfies it
/// from a stored grant, so only the Confirm button's staged one-shot approval
/// can resume it. Every other class keeps the legacy grant decisions.
pub fn destructive_confirm_required(action_class: &str) -> bool {
    action_class == "delete"
}

/// Upper bound on `{label, before, after}` entries in a consent preview.
const PREVIEW_MAX_ENTRIES: usize = 10;
/// Upper bound on characters kept for any single preview value.
const PREVIEW_VALUE_CHARS: usize = 80;

/// Derives an OPTIONAL structured preview from data already present in the
/// consent detail (`{ command, params }`). Only a fixed whitelist of metadata
/// keys and before/after-shaped params is surfaced — never document content —
/// values are truncated and the entry list is bounded (#2448 criterion 4).
fn consent_preview(detail: &serde_json::Value) -> Option<serde_json::Value> {
    let params = detail.get("params")?.as_object()?;
    let count = preview_affected_count(params);
    let entries = preview_entries(params);
    if count.is_none() && entries.is_empty() {
        return None;
    }
    let mut preview = serde_json::Map::new();
    if let Some(count) = count {
        preview.insert("affected_count".to_string(), serde_json::json!(count));
    }
    if !entries.is_empty() {
        preview.insert("entries".to_string(), serde_json::json!(entries));
    }
    Some(serde_json::Value::Object(preview))
}

/// Count from count-like params, falling back to the length of a target list.
fn preview_affected_count(params: &serde_json::Map<String, serde_json::Value>) -> Option<usize> {
    for key in [
        "affected_count",
        "affected",
        "count",
        "total",
        "quantity",
        "n",
        "size",
        "limit",
    ] {
        if let Some(value) = params.get(key).and_then(serde_json::Value::as_u64) {
            return usize::try_from(value).ok();
        }
    }
    for key in ["ids", "items", "records", "targets", "selection"] {
        if let Some(len) = params
            .get(key)
            .and_then(serde_json::Value::as_array)
            .map(Vec::len)
        {
            return Some(len);
        }
    }
    None
}

/// Bounded, content-free diff entries. Only explicit before/after param pairs
/// and record identifiers are used; no free-form or document fields.
fn preview_entries(params: &serde_json::Map<String, serde_json::Value>) -> Vec<serde_json::Value> {
    let pairs: &[(&str, &str, &str)] = &[
        ("before", "after", "change"),
        ("from", "to", "transition"),
        ("old", "new", "update"),
        ("find", "replace", "replace"),
        ("source", "target", "move"),
        ("previous", "next", "change"),
    ];
    let mut entries = Vec::new();
    for (before_key, after_key, label) in pairs {
        if entries.len() >= PREVIEW_MAX_ENTRIES {
            break;
        }
        let before = params.get(*before_key);
        let after = params.get(*after_key);
        if before.is_none() && after.is_none() {
            continue;
        }
        entries.push(serde_json::json!({
            "label": *label,
            "before": before.map(preview_value).unwrap_or(serde_json::Value::Null),
            "after": after.map(preview_value).unwrap_or(serde_json::Value::Null),
        }));
    }
    // Target identifiers are metadata, not document content.
    for key in ["ids", "items", "records", "targets", "selection"] {
        if entries.len() >= PREVIEW_MAX_ENTRIES {
            break;
        }
        let Some(list) = params.get(key).and_then(serde_json::Value::as_array) else {
            continue;
        };
        for value in list {
            if entries.len() >= PREVIEW_MAX_ENTRIES {
                break;
            }
            entries.push(serde_json::json!({
                "label": "target",
                "before": preview_value(value),
                "after": serde_json::Value::Null,
            }));
        }
    }
    entries
}

/// Truncate a param value to a bounded, stringified form.
fn preview_value(value: &serde_json::Value) -> serde_json::Value {
    let text = match value {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    let truncated: String = text.chars().take(PREVIEW_VALUE_CHARS).collect();
    serde_json::Value::String(truncated)
}

/// Marker protocol consumed by `chat/modules/40_consent_cards.js`: a plain
/// sentence for legacy clients plus a base64url JSON payload the suite
/// renderer swaps for an interactive card.
pub fn marker_content(request: &botconsent::PendingRequest) -> String {
    use base64::Engine as _;
    let mut payload = serde_json::json!({
        "request_id": request.request_id,
        "app_id": request.app_id,
        "action_class": request.action_class,
        "detail": &request.detail,
    });
    // #2448 criterion 3/4 — destructive markers carry `confirm_required` so
    // the suite renders Confirm/Cancel (not grants), plus an OPTIONAL bounded,
    // content-free preview. Legacy clients ignore both extra keys.
    if destructive_confirm_required(&request.action_class) {
        payload["confirm_required"] = serde_json::json!(true);
        if let Some(preview) = consent_preview(&request.detail) {
            payload["preview"] = preview;
        }
        // #2448 criterion 4 — carry only an explicit batch id; a consent id is not one.
        let supplied = request.detail.pointer("/params/batch_id").and_then(serde_json::Value::as_str);
        if let Some(id) = supplied.filter(|id| !id.trim().is_empty()) {
            payload["batch_id"] = serde_json::json!(id);
        }
    }
    let encoded =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
    // Requests raised by the api loopback carry an empty app id; printing
    // "on ''" was meaningless to the user.
    let app = if request.app_id.trim().is_empty() {
        "workspace"
    } else {
        request.app_id.as_str()
    };
    format!(
        "Permission needed: the assistant wants to run '{class}' on '{app}'. Reply via the buttons or Settings.\nGB-CONSENT-REQUEST:{encoded}",
        class = request.action_class,
        app = app,
        encoded = encoded,
    )
}

#[cfg(test)]
mod tests {
    use super::{consent_preview, destructive_confirm_required, marker_content};

    #[test]
    fn destructive_confirm_gate_is_class_scoped() {
        assert!(destructive_confirm_required("delete"));
        // Every non-delete class keeps the legacy permission-grant buttons.
        for class in [
            "read", "create", "update", "share", "publish", "payment", "", "unknown",
        ] {
            assert!(
                !destructive_confirm_required(class),
                "'{class}' must not require the confirm gate"
            );
        }
    }

    #[test]
    fn marker_flags_confirm_gate_only_for_destructive_class() {
        use base64::Engine as _;
        let make = |class: &str| botconsent::PendingRequest {
            request_id: "req-1".to_string(),
            user_id: uuid::Uuid::nil(),
            app_id: String::new(),
            action_class: class.to_string(),
            detail: detail(
                "database.rows.delete",
                serde_json::json!({ "ids": [1, 2], "count": 2 }),
            ),
            thread_id: None,
            run_id: None,
            risk_tier: String::new(),
        };
        let decode = |marker: &str| -> serde_json::Value {
            let encoded = marker
                .split("GB-CONSENT-REQUEST:")
                .nth(1)
                .unwrap_or_default()
                .trim();
            let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(encoded)
                .unwrap_or_default();
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
        };

        let destructive = decode(&marker_content(&make("delete")));
        assert_eq!(destructive["confirm_required"], serde_json::json!(true));
        assert!(
            destructive["preview"].is_object(),
            "destructive marker must carry the bounded preview"
        );

        let read = decode(&marker_content(&make("read")));
        assert!(
            read.get("confirm_required").is_none(),
            "read class must not expose a confirm gate"
        );
        assert!(
            read.get("preview").is_none(),
            "read class must not expose a destructive preview"
        );
    }

    fn detail(command: &str, params: serde_json::Value) -> serde_json::Value {
        serde_json::json!({ "command": command, "params": params })
    }

    #[test]
    fn preview_is_bounded_truncated_and_omitted_without_a_diff() {
        let ids: Vec<usize> = (0..50).collect();
        let params = serde_json::json!({
            "ids": ids,
            "count": 50,
            "find": "x".repeat(200),
            "replace": "y"
        });
        let preview = consent_preview(&detail("database.rows.bulk_delete", params));
        assert!(preview.is_some(), "a bulk delete must yield a preview");
        let preview = match preview {
            Some(value) => value,
            None => serde_json::Value::Null,
        };
        assert_eq!(preview["affected_count"], serde_json::json!(50));
        let empty: Vec<serde_json::Value> = Vec::new();
        let entries = preview["entries"].as_array().unwrap_or(&empty);
        assert_eq!(entries.len(), 10, "entries must be capped at 10");
        for entry in entries {
            assert!(entry["after"].is_null() || entry["after"].as_str().is_some());
        }
        let first_before = entries[0]["before"].as_str().unwrap_or("");
        assert!(
            first_before.chars().count() <= 80,
            "preview values must be truncated"
        );
        // No structured diff means no field at all (backward compatible).
        let empty = consent_preview(&detail("api.find", serde_json::json!({ "q": "x" })));
        assert!(empty.is_none());
    }

    #[test]
    fn preview_never_includes_document_content() {
        let secret = "CONFIDENTIAL-DOCUMENT-BODY-XYZ";
        let params = serde_json::json!({
            "content": secret,
            "body": secret,
            "text": secret,
            "scope": "document-42",
            "from": "draft",
            "to": "final"
        });
        let preview = consent_preview(&detail("docs.track_changes.accept_all", params));
        assert!(
            preview.is_some(),
            "a before/after pair must yield a preview"
        );
        let rendered = match preview {
            Some(value) => value.to_string(),
            None => String::new(),
        };
        assert!(
            !rendered.contains(secret),
            "preview must not carry document content"
        );
        // Marker exposes the bounded preview only on the destructive class.
        let request = botconsent::PendingRequest {
            request_id: "req-1".to_string(),
            user_id: uuid::Uuid::nil(),
            app_id: String::new(),
            action_class: "delete".to_string(),
            detail: detail(
                "docs.track_changes.accept_all",
                serde_json::json!({ "from": "draft", "to": "final" }),
            ),
            thread_id: None,
            run_id: None,
            risk_tier: String::new(),
        };
        let marker = marker_content(&request);
        assert!(marker.contains("GB-CONSENT-REQUEST:"));
        // The marker is base64url-encoded, so assert on the structured preview
        // itself rather than on a plaintext substring of the encoded string.
        let derived = consent_preview(&request.detail);
        assert!(derived.is_some(), "destructive request must yield a preview");
        assert!(
            derived
                .and_then(|value| value["entries"].as_array().cloned())
                .is_some_and(|entries| !entries.is_empty()),
            "before/after params must produce a preview entry"
        );
    }
}
