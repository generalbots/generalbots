//! Find & replace across the cells of one worksheet.
//!
//! The toolbar modal posts `{id, find, replace}` to a JSON endpoint and swaps
//! the HTML result into `#find-replace-results`; before this handler the form
//! posted to `GET /api/sheet/load` and answered 405.

use crate::auth::{resolve_user_id, SheetUser};
use crate::state::SheetState;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Html,
    Json,
};
use botsheet_core::engine::value::CellValue;
use botsheet_core::types::FindReplaceRequest;
use chrono::Utc;
use std::sync::Arc;

fn replace_ascii_case(haystack: &str, needle: &str, replacement: &str) -> (String, usize) {
    if needle.is_empty() {
        return (haystack.to_string(), 0);
    }
    let lowered = haystack.to_ascii_lowercase();
    let lowered_needle = needle.to_ascii_lowercase();
    let mut output = String::with_capacity(haystack.len());
    let mut cursor = 0usize;
    let mut count = 0usize;
    while let Some(position) = lowered[cursor..].find(&lowered_needle) {
        let start = cursor + position;
        output.push_str(&haystack[cursor..start]);
        output.push_str(replacement);
        cursor = start + needle.len();
        count += 1;
    }
    output.push_str(&haystack[cursor..]);
    (output, count)
}

pub async fn handle_find_replace(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<FindReplaceRequest>,
) -> Result<Html<String>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref());
    if req.find.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": "find is required" })),
        ));
    }
    let session = state
        .sessions
        .get_or_load(&state, &user_id, &req.sheet_id)
        .await
        .map_err(|error| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": error })),
            )
        })?;
    let mut sheet = session.sheet.write().await;
    if let Err(error) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": error })),
        ));
    }
    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let mut replacements = 0usize;
    for cell in worksheet.data.values_mut() {
        if cell.formula.is_some() {
            continue;
        }
        if matches!(
            cell.typed,
            Some(CellValue::Number(_)) | Some(CellValue::Bool(_)) | Some(CellValue::Error(_))
        ) {
            continue;
        }
        let Some(value) = cell.value.clone() else {
            continue;
        };
        let (replaced, count) = if req.match_case {
            let count = value.matches(&req.find).count();
            (value.replace(&req.find, &req.replace), count)
        } else {
            replace_ascii_case(&value, &req.find, &req.replace)
        };
        if count == 0 {
            continue;
        }
        replacements += count;
        cell.value = Some(replaced.clone());
        if cell.typed.is_some() {
            cell.typed = Some(CellValue::Text(replaced));
        }
    }
    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "find_replace",
        serde_json::json!({ "sheet_id": req.sheet_id, "replacements": replacements }),
    );

    Ok(Html(format!(
        r##"<div class="ss-find-result" style="padding:8px;color:#4ade80;font-size:12px;">{replacements} replacement(s) applied.</div>"##
    )))
}
