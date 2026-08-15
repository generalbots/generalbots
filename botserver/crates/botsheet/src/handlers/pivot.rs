use crate::auth::{resolve_user_id, SheetUser};
use crate::state::SheetState;
use crate::types::{PivotRequest, PivotResult};
use axum::{extract::{Extension, State}, http::StatusCode, Json};
use botsheet_core::engine::value::CellValue;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::sync::Arc;

fn col_letter_to_index(s: &str) -> Option<u32> {
    let mut n: u32 = 0;
    for c in s.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        let up = c.to_ascii_uppercase() as u32 - 'A' as u32 + 1;
        n = n.saturating_mul(26).saturating_add(up);
    }
    if n == 0 {
        None
    } else {
        Some(n - 1)
    }
}

fn parse_range(range: &str) -> Option<(u32, u32, u32, u32)> {
    let s = range.replace('$', "");
    let parts: Vec<&str> = s.split(':').collect();
    if parts.is_empty() {
        return None;
    }
    let (start_ref, end_ref) = if parts.len() == 1 {
        (parts[0], parts[0])
    } else {
        (parts[0], parts[1])
    };
    let split_ref = |r: &str| -> Option<(u32, u32)> {
        let p = r.find(|c: char| c.is_ascii_digit())?;
        if p == 0 || p == r.len() {
            return None;
        }
        let col = col_letter_to_index(&r[..p])?;
        let row: u32 = r[p..].parse().ok()?;
        Some((row, col))
    };
    let (r1, c1) = split_ref(start_ref)?;
    let (r2, c2) = split_ref(end_ref)?;
    let (rr1, rr2) = if r1 <= r2 { (r1, r2) } else { (r2, r1) };
    let (cc1, cc2) = if c1 <= c2 { (c1, c2) } else { (c2, c1) };
    Some((rr1, cc1, rr2, cc2))
}

fn cell_to_string(v: Option<&crate::types::CellData>) -> String {
    match v.and_then(|c| c.value.as_ref()) {
        Some(val) => val.clone(),
        None => String::new(),
    }
}

fn cell_to_number(v: Option<&crate::types::CellData>) -> Option<f64> {
    // Only typed Numbers aggregate; text that merely looks numeric (`0123`)
    // must not be summed. Untyped cells (CSV/ODS) keep the parse fallback.
    match v.and_then(|c| c.typed.as_ref()) {
        Some(CellValue::Number(n)) => Some(*n),
        Some(_) => None,
        None => cell_to_string(v).trim().parse::<f64>().ok(),
    }
}

fn collect_field_index(header_row: u32, start_col: u32, end_col: u32, data: &std::collections::HashMap<String, crate::types::CellData>) -> std::collections::HashMap<String, u32> {
    let mut idx = std::collections::HashMap::new();
    for col in start_col..=end_col {
        let key = format!("{},{}", header_row, col);
        let name = cell_to_string(data.get(&key)).trim().to_string();
        if !name.is_empty() {
            idx.insert(name, col);
        }
    }
    idx
}

pub async fn handle_pivot(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<PivotRequest>,
) -> Result<Json<PivotResult>, (StatusCode, Json<Value>)> {
    let user_id = resolve_user_id(user.as_deref());
    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": e })),
            )
        })?;
    let sheet = session.sheet.read().await.clone();

    let worksheet = match sheet.worksheets.first() {
        Some(w) => w,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Sheet has no worksheets" })),
            ))
        }
    };

    let (r1, c1, r2, c2) = match req.source_range.as_deref().and_then(parse_range) {
        Some(b) => b,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid or missing source_range" })),
            ))
        }
    };

    if r2 - r1 < 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "source_range must include at least a header row and one data row" })),
        ));
    }

    let field_index = collect_field_index(r1, c1, c2, &worksheet.data);
    let resolve = |name: &str| -> Result<u32, (StatusCode, Json<Value>)> {
        field_index
            .get(name)
            .copied()
            .ok_or_else(|| {
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
                .map(|c| cell_to_string(worksheet.data.get(&format!("{},{}", r, c))))
                .collect();
            parts.join("\x00")
        };
        let col_key = if col_cols.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = col_cols
                .iter()
                .map(|c| cell_to_string(worksheet.data.get(&format!("{},{}", r, c))))
                .collect();
            parts.join("\x00")
        };
        row_keys.insert(row_key.clone());
        col_keys.insert(col_key.clone());

        let bucket = group_values
            .entry((row_key, col_key))
            .or_insert_with(|| vec![Vec::new(); value_specs.len()]);
        for (i, (col, _)) in value_specs.iter().enumerate() {
            if let Some(n) = cell_to_number(worksheet.data.get(&format!("{},{}", r, col))) {
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
            "MIN" => vals.iter().cloned().reduce(f64::min),
            "MAX" => vals.iter().cloned().reduce(f64::max),
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
                if let Some(v) = cells.get(&key).and_then(|v| v.as_f64()) {
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
                if let Some(v) = cells.get(&key).and_then(|v| v.as_f64()) {
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

    Ok(Json(PivotResult { result }))
}
