//! `show_app`: publishes a small interactive HTML app for the conversation.

use serde_json::{json, Value};

use super::super::native_tools::persist;
use super::super::types::SessionIdentity;
use super::{emit_custom, invalid};

pub(super) async fn run_show_app(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = match super::super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            let message = format!("Could not read the app: {error}");
            super::super::native_tools::send(identity, &message).await;
            return;
        }
    };
    let html = parsed
        .get("html")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if html.is_empty() {
        invalid(identity, "show_app", &["data.html is required".to_string()]).await;
        return;
    }
    if html.len() > 512 * 1024 {
        invalid(
            identity,
            "show_app",
            &["data.html exceeds 512 KiB; split the app or return a surface".to_string()],
        )
        .await;
        return;
    }
    let title = parsed
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Result");
    let permissions: Vec<String> = parsed
        .get("permissions")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|value| matches!(*value, "camera" | "microphone" | "geolocation" | "clipboard-read" | "clipboard-write"))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let csp_value = parsed.get("csp").cloned().unwrap_or(Value::Null);
    let csp = crate::core::bot::sandbox::UiCsp {
        connect_domains: string_domains(csp_value.get("connectDomains")),
        resource_domains: string_domains(csp_value.get("resourceDomains")),
        permissions: permissions.clone(),
        domain: None,
    };
    let SessionIdentity {
        state,
        session_id,
        user_id,
        ..
    } = *identity;
    let thread_id = session_id.to_string();
    let origin = crate::core::bot::sandbox::routes::sandbox_origin();
    let hash = crate::core::bot::sandbox::sign::hash_of(&html);
    let (url, html_base64) = if origin.is_empty() {
        use base64::Engine as _;
        (
            None,
            Some(base64::engine::general_purpose::STANDARD.encode(html.as_bytes())),
        )
    } else {
        let path = crate::core::bot::sandbox::publish_resource(
            state,
            &state.bucket_name,
            &thread_id,
            &html,
            csp,
            900,
            Some(title),
        )
        .await;
        (Some(format!("{origin}{path}")), None)
    };
    let fallback = format!("{title}: an interactive app is ready in the conversation.");
    persist(state, session_id, user_id, &fallback).await;
    emit_custom(
        identity,
        "gb.app",
        json!({
            "title": title,
            "surface_id": format!("app-{hash}"),
            "url": url,
            "html_base64": html_base64,
            "permissions": permissions,
            "pin_hash": hash,
            "fallback": fallback,
        }),
    )
    .await;
}

fn string_domains(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|value| {
                    let lower = value.to_ascii_lowercase();
                    lower.starts_with("https://")
                        || lower.starts_with("wss://")
                        || lower.starts_with("http://localhost")
                        || lower.starts_with("http://127.0.0.1")
                })
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
