//! `show_surface`: renders a validated component surface.

use serde_json::{json, Value};

use super::super::native_tools::{persist, send};
use super::super::types::SessionIdentity;
use super::{emit_custom, invalid, SURFACE_COMPONENTS};

pub(super) async fn run_show_surface(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = match super::super::native_args::parse_args(args) {
        Ok(value) => value,
        Err(error) => {
            let message = format!("Could not read the surface: {error}");
            send(identity, &message).await;
            return;
        }
    };
    let surface = match parsed.get("surface") {
        Some(Value::Object(_)) => parsed.get("surface").cloned().unwrap_or(Value::Null),
        _ => parsed.clone(),
    };
    let mut issues = Vec::new();
    let components = match surface.get("components") {
        Some(Value::Array(items)) if !items.is_empty() => items.clone(),
        Some(Value::Array(_)) => {
            issues.push("data.components must contain at least one component".to_string());
            Vec::new()
        }
        Some(_) => {
            issues.push("data.components must be an array".to_string());
            Vec::new()
        }
        None => {
            issues.push("data.components is required".to_string());
            Vec::new()
        }
    };
    for (index, component) in components.iter().enumerate() {
        let Some(object) = component.as_object() else {
            issues.push(format!("components[{index}] must be an object"));
            continue;
        };
        let kind = object.get("type").and_then(Value::as_str).unwrap_or("");
        if kind.is_empty() {
            issues.push(format!("components[{index}].type is required"));
        } else if !SURFACE_COMPONENTS.contains(&kind) {
            issues.push(format!(
                "components[{index}].type '{kind}' is not in the catalog"
            ));
        }
        if object.get("id").and_then(Value::as_str).unwrap_or("").is_empty() {
            issues.push(format!("components[{index}].id is required"));
        }
    }
    if !issues.is_empty() {
        invalid(identity, "show_surface", &issues).await;
        return;
    }
    let fallback = surface
        .get("fallback")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            parsed
                .get("fallback")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "A result is available in the client.".to_string());
    let title = surface
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Result");
    {
        let SessionIdentity {
            state,
            session_id,
            user_id,
            ..
        } = *identity;
        persist(state, session_id, user_id, &fallback).await;
    }
    emit_custom(
        identity,
        "gb.surface",
        json!({
            "surface": {
                "version": "gb.basic.v1",
                "surface_id": surface.get("surface_id").cloned().unwrap_or(Value::Null),
                "title": title,
                "components": components,
                "data": surface.get("data").cloned().unwrap_or(Value::Null),
                "fallback": fallback,
            },
            "fallback": fallback,
        }),
    )
    .await;
}
