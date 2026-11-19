mod build;
mod helpers;

use crate::auth::{resolve_user_id, SheetUser};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use botsheet_core::state::SheetState;
use botsheet_core::types::{PivotRequest, PivotResult};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

use build::build_pivot_worksheet;
use helpers::{cell_to_number, cell_to_string, collect_field_index, parse_range};

pub async fn handle_pivot(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<PivotRequest>,
) -> Result<Json<PivotResult>, (StatusCode, Json<Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;
    if req.sheet_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing sheet_id/file_id for pivot" })),
        ));
    }
    let session = state
        .sessions
        .get_or_load(&state, &user_id, req.sheet_id.trim())
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": e })),
            )
        })?;
    let sheet = session.sheet.read().await.clone();

    let worksheet = match sheet.worksheets.first() {
        Some(w) => w,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Sheet has no worksheets" })),
            ));
        }
    };

    let (r1, c1, r2, c2) = match req
        .source_range
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(parse_range)
    {
        Some(b) => b,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid or missing source_range (e.g. A1:D100)" })),
            ));
        }
    };

    if r2.saturating_sub(r1) < 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({ "error": "source_range must include at least a header row and one data row" }),
            ),
        ));
    }

    let field_index = collect_field_index(r1, c1, c2, &worksheet.data);
    let resolve = |name: &str| -> Result<u32, (StatusCode, Json<Value>)> {
        field_index.get(name).copied().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Unknown field: {name}") })),
            )
        })
    };

    let row_cols: Vec<u32> = req
        .rows
        .iter()
        .map(|f| resolve(f))
        .collect::<Result<Vec<_>, _>>()?;
    let col_cols: Vec<u32> = req
        .cols
        .iter()
        .map(|f| resolve(f))
        .collect::<Result<Vec<_>, _>>()?;
    let value_specs: Vec<(u32, String)> = req
        .values
        .iter()
        .map(|p| Ok((resolve(&p.field)?, p.agg.to_uppercase())))
        .collect::<Result<Vec<_>, _>>()?;
    if value_specs.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({ "error": "Pivot requires at least one value field (e.g. values:[{\"field\":\"Amount\",\"agg\":\"SUM\"}])" }),
            ),
        ));
    }

    let mut row_keys: BTreeSet<String> = BTreeSet::new();
    let mut col_keys: BTreeSet<String> = BTreeSet::new();
    let mut group_values: std::collections::HashMap<(String, String), Vec<Vec<f64>>> =
        std::collections::HashMap::new();

    for r in (r1 + 1)..=r2 {
        let row_key = if row_cols.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = row_cols
                .iter()
                .map(|c| cell_to_string(worksheet.data.get(&format!("{r},{c}"))))
                .collect();
            parts.join("\x00")
        };
        let col_key = if col_cols.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = col_cols
                .iter()
                .map(|c| cell_to_string(worksheet.data.get(&format!("{r},{c}"))))
                .collect();
            parts.join("\x00")
        };
        row_keys.insert(row_key.clone());
        col_keys.insert(col_key.clone());

        let bucket = group_values
            .entry((row_key, col_key))
            .or_insert_with(|| vec![Vec::new(); value_specs.len()]);
        for (i, (col, _)) in value_specs.iter().enumerate() {
            if let Some(n) = cell_to_number(worksheet.data.get(&format!("{r},{col}"))) {
                bucket[i].push(n);
            }
        }
    }

    let row_keys_v: Vec<String> = row_keys.into_iter().collect();
    let col_keys_v: Vec<String> = col_keys.into_iter().collect();

    let aggregate = |vals: &[f64], agg: &str| -> Option<f64> {
        if vals.is_empty() {
            return None;
        }
        match agg {
            "SUM" => Some(vals.iter().sum()),
            "AVG" | "AVERAGE" | "MEAN" => {
                let sum: f64 = vals.iter().sum();
                Some(sum / vals.len() as f64)
            }
            "COUNT" => Some(vals.len() as f64),
            "MIN" => vals.iter().copied().reduce(f64::min),
            "MAX" => vals.iter().copied().reduce(f64::max),
            _ => Some(vals.iter().sum()),
        }
    };

    let mut cells: Map<String, Value> = Map::new();
    for ((rk, ck), values) in &group_values {
        for (i, (_, agg)) in value_specs.iter().enumerate() {
            let agg_value = aggregate(&values[i], agg);
            let key = format!("{rk}\x00{ck}\x00{i}");
            if let Some(v) = agg_value {
                cells.insert(key, json!(v));
            } else {
                cells.insert(key, Value::Null);
            }
        }
    }

    let mut row_totals: Map<String, Value> = Map::new();
    let mut col_totals: Map<String, Value> = Map::new();
    let mut grand_total: f64 = 0.0;
    for rk in &row_keys_v {
        let mut sum: f64 = 0.0;
        let mut any = false;
        for ck in &col_keys_v {
            for i in 0..value_specs.len() {
                let key = format!("{rk}\x00{ck}\x00{i}");
                if let Some(v) = cells.get(&key).and_then(Value::as_f64) {
                    sum += v;
                    any = true;
                }
            }
        }
        if any {
            row_totals.insert(rk.clone(), json!(sum));
            grand_total += sum;
        }
    }
    for ck in &col_keys_v {
        let mut sum: f64 = 0.0;
        let mut any = false;
        for rk in &row_keys_v {
            for i in 0..value_specs.len() {
                let key = format!("{rk}\x00{ck}\x00{i}");
                if let Some(v) = cells.get(&key).and_then(Value::as_f64) {
                    sum += v;
                    any = true;
                }
            }
        }
        if any {
            col_totals.insert(ck.clone(), json!(sum));
        }
    }

    let result = json!({
        "rowKeys": row_keys_v,
        "colKeys": col_keys_v,
        "cells": cells,
        "rowTotals": row_totals,
        "colTotals": col_totals,
        "grandTotal": grand_total,
    });

    // Materialize the pivot as a second worksheet so it appears as its own tab
    // in the UI (UAT SHEET-05: "pivot table is created on a second tab").
    let pivot_ws = build_pivot_worksheet(
        &row_keys_v,
        &col_keys_v,
        &value_specs,
        &cells,
        &row_totals,
        &col_totals,
        grand_total,
    );
    {
        let mut sheet_w = session.sheet.write().await;
        let mut name = "Pivot".to_string();
        if sheet_w.worksheets.iter().any(|w| w.name == name) {
            name = format!("Pivot {}", sheet_w.worksheets.len() + 1);
        }
        let mut ws = pivot_ws;
        ws.name = name;
        sheet_w.worksheets.push(ws);
        sheet_w.updated_at = chrono::Utc::now();
        let snapshot = sheet_w.clone();
        drop(sheet_w);
        if let Err(e) = crate::state::save_sheet_to_drive(&state, &user_id, &snapshot).await {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    json!({ "error": format!("pivot saved result but failed to persist worksheet: {e}") }),
                ),
            ));
        }
    }
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "pivot_created",
        json!({ "sheet_id": req.sheet_id }),
    );

    Ok(Json(PivotResult { result }))
}
