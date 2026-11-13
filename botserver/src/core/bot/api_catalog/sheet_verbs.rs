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

use serde_json::{Map, Value, json};

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
    // #2260 — a request for a NEW formatted sheet reaches this dispatcher as
    // `sheet.cells.set` carrying `title` + `values` (no `file_id`, no `range`),
    // because that is the verb whose description mentions styling. The cells
    // endpoint then answers 422 for the missing fields and nothing is created.
    // Route that shape to `sheet.create` instead of failing the turn; an
    // existing sheet must still identify itself with `file_id`.
    let has_target = obj.get("file_id").and_then(Value::as_str).is_some()
        || obj.get("sheet_id").and_then(Value::as_str).is_some();
    let values_instead_of_rows = !obj.contains_key("rows") && obj.contains_key("values");
    let (name, params) = if name == "sheet.cells.set" && !has_target {
        if !obj.contains_key("title") && !obj.contains_key("values") {
            return Err(
                "sheet.cells.set requires `file_id` (open or create a spreadsheet first)"
                    .to_string(),
            );
        }
        let mut params = obj.clone();
        if values_instead_of_rows {
            if let Some(values) = params.remove("values") {
                params.insert("rows".to_string(), values);
            }
        }
        ("sheet.create", params)
    } else {
        let mut params = obj.clone();
        // `sheet.create` and `sheet.cells.set` were conflated by the model in
        // both directions: accept `values` where the endpoint expects `rows`.
        if name == "sheet.create" && values_instead_of_rows {
            if let Some(values) = params.remove("values") {
                params.insert("rows".to_string(), values);
            }
        }
        // A single-cell write may say `cell` instead of an A1 `range`.
        if name == "sheet.cells.set"
            && !params.contains_key("range")
            && params.get("cell").and_then(Value::as_str).is_some()
        {
            if let Some(cell) = params.remove("cell") {
                params.insert("range".to_string(), cell);
            }
        }
        (name, params)
    };
    let binding = verb_endpoint(name).ok_or_else(|| format!("no endpoint for '{name}'"))?;
    let mapped = binding.map_params(&params);
    // The command itself was gated upstream (handle.rs / api.exec); a second
    // prompt here would ask for permission to write the very cells the user
    // just approved.
    let result = crate::core::bot::api_exec::exec_endpoint_trusted(
        state,
        user_id,
        Some(bot_uuid),
        binding.method,
        binding.path,
        &mapped,
    )
    .await?;

    let sheet_id = params
        .get("file_id")
        .and_then(Value::as_str)
        .or_else(|| params.get("sheet_id").and_then(Value::as_str))
        .or_else(|| result.get("id").and_then(Value::as_str))
        .unwrap_or("")
        .to_string();
    if sheet_id.is_empty() {
        return Ok(result);
    }

    let styles = params.get("styles");
    let formats = params.get("formats");
    if styles.is_some() || formats.is_some() {
        apply_styles(state, user_id, bot_uuid, &sheet_id, styles, formats).await?;
    }
    if let Some(rules) = params.get("conditional").and_then(Value::as_array) {
        apply_conditional(state, user_id, bot_uuid, &sheet_id, rules).await?;
    }
    apply_charts(state, user_id, bot_uuid, &sheet_id, &params).await?;

    Ok(crate::apps::commands::with_navigation(
        name, &params, result,
    ))
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

async fn apply_charts(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_uuid: Uuid,
    sheet_id: &str,
    params: &Map<String, Value>,
) -> Result<(), String> {
    let Some(charts) = chart_specs(params) else {
        return Ok(());
    };
    for chart in charts {
        let Some(obj) = chart.as_object() else {
            continue;
        };
        let mut body = obj.clone();
        body.insert("id".into(), Value::String(sheet_id.to_string()));
        if !body.contains_key("data_range") {
            if let Some(range) = body.remove("range").or_else(|| body.remove("dataRange")) {
                body.insert("data_range".into(), range);
            }
        }
        if !body.contains_key("label_range") {
            if let Some(labels) = body.remove("labels").or_else(|| body.remove("labelRange")) {
                body.insert("label_range".into(), labels);
            }
        }
        if !body.contains_key("chart_type") {
            if let Some(kind) = body.remove("type").or_else(|| body.remove("chartType")) {
                body.insert("chart_type".into(), kind);
            }
        }
        crate::core::bot::api_exec::exec_endpoint_trusted(
            state,
            user_id,
            Some(bot_uuid),
            "POST",
            "/api/sheet/chart",
            &body,
        )
        .await
        .map(|_| ())?;
    }
    Ok(())
}

fn chart_specs(params: &Map<String, Value>) -> Option<Vec<Value>> {
    if let Some(charts) = params.get("charts").and_then(Value::as_array) {
        return Some(charts.clone());
    }
    params.get("chart").and_then(|v| {
        if v.is_array() {
            v.as_array().cloned()
        } else if v.is_object() {
            Some(vec![v.clone()])
        } else {
            None
        }
    })
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
    crate::core::bot::api_exec::exec_endpoint_trusted(
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
