//! Composite spreadsheet commands: values **and** their styling in one call.
//!
//! `sheet.cells.set` / `sheet.create` used to forward only values through the
//! generic verb loopback, so every fill, bold header, number format and
//! negative-red rule the model sent alongside them was dropped. The sheet then
//! looked unstyled and the UAT scenarios that ask for a formatted cash-flow
//! (#2260) or a colour-coded timetable (#2263) could never pass: the app has
//! the format and conditional-format endpoints, but no chat path reached them.
//!
//! This module runs the value write first, then applies `styles` / `formats`
//! (A1 -> style / number format) and `conditional` (list of rules) against the
//! same sheet, so a single model call produces a styled artifact.

use super::*;

use serde_json::{json, Map, Value};

use crate::apps::commands::verb_endpoint;

const FORMAT_PATH: &str = "/api/sheet/format";
const CONDITIONAL_PATH: &str = "/api/sheet/conditional-format";

pub(super) fn handles(name: &str) -> bool {
    matches!(name, "sheet.cells.set" | "sheet.create")
}

pub(super) async fn execute(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_uuid: Uuid,
    name: &str,
    obj: &Map<String, Value>,
) -> Result<Value, String> {
    let binding = verb_endpoint(name).ok_or_else(|| format!("no endpoint for '{name}'"))?;
    let mapped = binding.map_params(obj);
    let result = crate::core::bot::api_exec::exec_endpoint(
        state,
        user_id,
        Some(bot_uuid),
        binding.method,
        binding.path,
        &mapped,
    )
    .await?;

    let sheet_id = obj
        .get("file_id")
        .and_then(Value::as_str)
        .or_else(|| obj.get("sheet_id").and_then(Value::as_str))
        .or_else(|| result.get("id").and_then(Value::as_str))
        .unwrap_or("")
        .to_string();
    if sheet_id.is_empty() {
        return Ok(result);
    }

    let styles = obj.get("styles");
    let formats = obj.get("formats");
    if styles.is_some() || formats.is_some() {
        apply_styles(state, user_id, bot_uuid, &sheet_id, styles, formats).await?;
    }
    if let Some(rules) = obj.get("conditional").and_then(Value::as_array) {
        apply_conditional(state, user_id, bot_uuid, &sheet_id, rules).await?;
    }

    Ok(crate::apps::commands::with_navigation(name, obj, result))
}

/// Groups the A1 -> style / format maps into one request per identical pair, so
/// a 12-column balance row costs one round trip instead of twelve.
async fn apply_styles(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_uuid: Uuid,
    sheet_id: &str,
    styles: Option<&Value>,
    formats: Option<&Value>,
) -> Result<(), String> {
    let mut ranges: Vec<(String, Value, Option<String>)> = Vec::new();
    let style_by_ref = styles.and_then(Value::as_object);
    let format_by_ref = formats.and_then(Value::as_object);

    let mut refs: Vec<String> = Vec::new();
    for source in [style_by_ref, format_by_ref].into_iter().flatten() {
        for reference in source.keys() {
            if !refs.contains(reference) {
                refs.push(reference.clone());
            }
        }
    }

    for reference in refs {
        let style = style_by_ref
            .and_then(|m| m.get(&reference))
            .cloned()
            .unwrap_or_else(|| json!({}));
        let format = format_by_ref
            .and_then(|m| m.get(&reference))
            .and_then(Value::as_str)
            .map(str::to_string);
        if style.as_object().map(|m| m.is_empty()).unwrap_or(true) && format.is_none() {
            continue;
        }
        ranges.push((reference, style, format));
    }

    for (range, style, format) in ranges {
        let mut params = Map::new();
        params.insert("range".into(), Value::String(range));
        params.insert("style".into(), style);
        if let Some(format) = format {
            params.insert("format".into(), Value::String(format));
        }
        post(state, user_id, bot_uuid, sheet_id, FORMAT_PATH, &params).await?;
    }
    Ok(())
}

async fn apply_conditional(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_uuid: Uuid,
    sheet_id: &str,
    rules: &[Value],
) -> Result<(), String> {
    for rule in rules {
        let Some(rule) = rule.as_object() else {
            continue;
        };
        let Some(range) = rule.get("range").and_then(Value::as_str) else {
            continue;
        };
        let mut params = rule.clone();
        params.insert("range".into(), Value::String(range.to_string()));
        // Chat plans say "fill"; the endpoint curates `bg_color`.
        if !params.contains_key("bg_color") {
            if let Some(fill) = rule.get("fill").and_then(Value::as_str) {
                params.insert("bg_color".into(), Value::String(fill.to_string()));
            }
        }
        post(
            state,
            user_id,
            bot_uuid,
            sheet_id,
            CONDITIONAL_PATH,
            &params,
        )
        .await?;
    }
    Ok(())
}

async fn post(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_uuid: Uuid,
    sheet_id: &str,
    path: &str,
    params: &Map<String, Value>,
) -> Result<(), String> {
    let mut params = params.clone();
    params.insert("sheet_id".into(), Value::String(sheet_id.to_string()));
    crate::core::bot::api_exec::exec_endpoint(
        state,
        user_id,
        Some(bot_uuid),
        "POST",
        path,
        &params,
    )
    .await
    .map(|_| ())
}
