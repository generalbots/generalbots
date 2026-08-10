use crate::auth::{resolve_user_id, SheetUser};
use crate::state::SheetState;
use crate::types::{
    NamedRange, NamedRangesExportResponse, NamedRangesImportResponse,
};
use axum::{
    body::Bytes,
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub async fn handle_export_named_ranges_csv(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<NamedRangesExportResponse>, (StatusCode, Json<Value>)> {
    let sheet_id = params.get("sheet_id").cloned().unwrap_or_default();
    let user_id = resolve_user_id(user.as_deref());
    let session = state
        .sessions
        .get_or_load(&state, &user_id, &sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": e })),
            )
        })?;
    let sheet = session.sheet.read().await.clone();

    let mut csv = String::from("name,range,description\n");
    if let Some(ranges) = sheet.named_ranges.as_ref() {
        for r in ranges {
            let range = format!(
                "R{}C{}:R{}C{}",
                r.start_row + 1,
                r.start_col + 1,
                r.end_row + 1,
                r.end_col + 1
            );
            let desc = r.comment.clone().unwrap_or_default();
            csv.push_str(&format!(
                "{},{},{}\n",
                csv_escape(&r.name),
                csv_escape(&range),
                csv_escape(&desc)
            ));
        }
    }

    Ok(Json(NamedRangesExportResponse { csv }))
}

pub async fn handle_import_named_ranges_csv(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Query(params): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<Json<NamedRangesImportResponse>, (StatusCode, Json<Value>)> {
    let sheet_id = params.get("sheet_id").cloned().unwrap_or_default();
    let user_id = resolve_user_id(user.as_deref());
    let session = state
        .sessions
        .get_or_load(&state, &user_id, &sheet_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": e })),
            )
        })?;
    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(json!({ "error": e }))));
    }

    let csv = String::from_utf8_lossy(&body).to_string();

    let mut added: u32 = 0;
    let mut updated: u32 = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut entries: Vec<NamedRange> = Vec::new();
    let mut existing: HashMap<String, usize> = HashMap::new();
    if let Some(ranges) = sheet.named_ranges.as_ref() {
        for (i, r) in ranges.iter().enumerate() {
            existing.insert(r.name.clone(), i);
        }
    }

    for (line_no, raw) in csv.lines().enumerate() {
        if line_no == 0 {
            continue;
        }
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<String> = line
            .split(',')
            .map(|p| p.trim().to_string())
            .collect();
        if parts.is_empty() {
            continue;
        }
        let name = parts.first().cloned().unwrap_or_default();
        let range = parts.get(1).cloned().unwrap_or_default();
        let desc = parts.get(2).cloned().unwrap_or_default();
        if name.is_empty() || range.is_empty() {
            errors.push(format!("Line {}: missing name or range", line_no + 1));
            continue;
        }
        if let Some(&idx) = existing.get(&name) {
            if let Some(ranges) = sheet.named_ranges.as_mut() {
                ranges[idx].comment = if desc.is_empty() { None } else { Some(desc.clone()) };
                updated += 1;
            }
        } else {
            let entry = NamedRange {
                id: Uuid::new_v4().to_string(),
                name: name.clone(),
                scope: "workbook".to_string(),
                worksheet_index: Some(0),
                start_row: 0,
                start_col: 0,
                end_row: 0,
                end_col: 0,
                comment: if desc.is_empty() { None } else { Some(desc.clone()) },
            };
            let ranges = sheet.named_ranges.get_or_insert_with(Vec::new);
            ranges.push(entry.clone());
            added += 1;
            entries.push(entry);
        }
    }

    sheet.updated_at = Utc::now();
    state.sessions.record(&session, &state, &user_id, "named_ranges_import", json!({ "added": added, "updated": updated }));

    Ok(Json(NamedRangesImportResponse {
        added,
        updated,
        errors,
        entries,
    }))
}
