use super::*;
use std::collections::HashMap;

pub(super) fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub(super) fn form_or_json_bool(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(serde_json::Value::Bool(v)) => *v,
        Some(serde_json::Value::String(v)) => matches!(v.as_str(), "on" | "true" | "1"),
        Some(serde_json::Value::Number(v)) => v.as_i64().map(|n| n != 0).unwrap_or(false),
        _ => false,
    }
}

/// Accept both the HTMX form payload (`application/x-www-form-urlencoded`) and
/// the JSON payload sent by the settings JavaScript helpers. Unchecked
/// checkboxes are absent from form payloads, which `form_or_json_bool` treats
/// as `false`.
fn parse_notification_body(
    content_type: &str,
    body: &[u8],
) -> Result<HashMap<String, serde_json::Value>, String> {
    if content_type.contains("application/json") {
        let value: serde_json::Value =
            serde_json::from_slice(body).map_err(|e| format!("Invalid JSON: {e}"))?;
        match value {
            serde_json::Value::Object(map) => Ok(map.into_iter().collect()),
            _ => Err("Expected a JSON object".to_string()),
        }
    } else {
        let form: HashMap<String, String> =
            serde_urlencoded::from_bytes(body).map_err(|e| format!("Invalid form: {e}"))?;
        Ok(form
            .into_iter()
            .map(|(key, value)| (key, serde_json::Value::String(value)))
            .collect())
    }
}

pub async fn notif_prefs_get(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Query(query): axum::extract::Query<HashMap<String, String>>,
) -> axum::response::Response {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html(String::new())).into_response(),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let prefs = read_pref(&mut conn, user_id, "notifications");
    // #1813 — the digest job records its last run under a separate preference
    // key; surface it here so Settings can show delivery health instead of a
    // fire-and-forget checkbox.
    let digest_status = read_pref(
        &mut conn,
        user_id,
        crate::settings_digest::DIGEST_STATUS_KEY,
    );
    let email_mentions = prefs
        .get("email_mentions")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let email_digest = prefs
        .get("email_digest")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let digest_last_sent = digest_status
        .get("last_sent_at")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let digest_last_error = digest_status
        .get("last_error")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let digest_next_run = if email_digest {
        let timezone = read_pref(&mut conn, user_id, "timezone")
            .as_str()
            .unwrap_or("UTC")
            .to_string();
        crate::settings_digest::next_run_at(&digest_status, &timezone)
            .map(|next| serde_json::Value::String(next.to_rfc3339()))
            .unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    let email_recipient = prefs
        .get("email_recipient")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let email_marketing = prefs
        .get("email_marketing")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let push_enabled = prefs
        .get("push_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let push_sound = prefs
        .get("push_sound")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let desktop_notifications = prefs
        .get("desktop_notifications")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let badge_count = prefs
        .get("badge_count")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let wants_json = query.get("format").map(|v| v == "json").unwrap_or(false)
        || headers
            .get(axum::http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("application/json"))
            .unwrap_or(false);
    if wants_json {
        return axum::Json(serde_json::json!({
            "email_mentions": email_mentions,
            "email_digest": email_digest,
            "email_recipient": email_recipient,
            "digest_last_sent": digest_last_sent,
            "digest_last_error": digest_last_error,
            "digest_next_run": digest_next_run,
            "email_marketing": email_marketing,
            "push_enabled": push_enabled,
            "push_sound": push_sound,
            "desktop_notifications": desktop_notifications,
            "badge_count": badge_count,
        }))
        .into_response();
    }

    (StatusCode::OK, Html(format!(
        r#"<div class="preference-list">
    <label class="checkbox-label"><input type="checkbox" name="email_mentions" {email_mentions_ck}> <span data-i18n="settings-mentions">Mentions and replies</span></label>
    <label class="checkbox-label"><input type="checkbox" name="email_digest" {email_digest_ck}> <span data-i18n="settings-weekly-digest">Weekly digest</span></label>
    <label class="checkbox-label"><input type="checkbox" name="email_marketing" {email_marketing_ck}> <span data-i18n="settings-marketing">Marketing</span></label>
    <label class="checkbox-label"><input type="checkbox" name="push_enabled" {push_enabled_ck}> <span data-i18n="settings-push-notifications">Push notifications</span></label>
    <label class="checkbox-label"><input type="checkbox" name="push_sound" {push_sound_ck}> <span data-i18n="settings-push-sound">Sound</span></label>
    <label class="checkbox-label"><input type="checkbox" name="desktop_notifications" {desktop_ck}> <span data-i18n="settings-desktop-notifications">Desktop notifications</span></label>
    <label class="checkbox-label"><input type="checkbox" name="badge_count" {badge_ck}> <span data-i18n="settings-badge-count">Badge count</span></label>
</div>"#,
        email_mentions_ck = if email_mentions { "checked" } else { "" },
        email_digest_ck = if email_digest { "checked" } else { "" },
        email_marketing_ck = if email_marketing { "checked" } else { "" },
        push_enabled_ck = if push_enabled { "checked" } else { "" },
        push_sound_ck = if push_sound { "checked" } else { "" },
        desktop_ck = if desktop_notifications { "checked" } else { "" },
        badge_ck = if badge_count { "checked" } else { "" },
    )))
    .into_response()
}

pub async fn notif_prefs_put(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("Database unavailable".to_string()),
            )
        }
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let fields = match parse_notification_body(content_type, &body) {
        Ok(fields) => fields,
        Err(e) => return (StatusCode::BAD_REQUEST, Html(e)),
    };

    let email_recipient = fields
        .get("email_recipient")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .unwrap_or("");
    let prefs = serde_json::json!({
        "email_mentions": form_or_json_bool(fields.get("email_mentions")),
        "email_digest": form_or_json_bool(fields.get("email_digest")),
        "email_recipient": email_recipient,
        "email_marketing": form_or_json_bool(fields.get("email_marketing")),
        "push_enabled": form_or_json_bool(fields.get("push_enabled")),
        "push_sound": form_or_json_bool(fields.get("push_sound")),
        "desktop_notifications": form_or_json_bool(fields.get("desktop_notifications")),
        "badge_count": form_or_json_bool(fields.get("badge_count")),
    });
    write_pref(&mut conn, user_id, "notifications", &prefs);

    (
        StatusCode::OK,
        Html("Notification preferences saved".to_string()),
    )
}
/// POST /api/oauth/{provider}/connect — starts the account-linking flow.
/// Delegates to [`crate::settings_oauth::oauth_connect`], which builds a
/// user-bound OAuth state (PKCE) and returns the provider redirect URL.
pub(super) async fn oauth_connect_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(provider_name): axum::extract::Path<String>,
) -> axum::response::Response {
    crate::settings_oauth::oauth_connect(State(state), headers, axum::extract::Path(provider_name))
        .await
}
