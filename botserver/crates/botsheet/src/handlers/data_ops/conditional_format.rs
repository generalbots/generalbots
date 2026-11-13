//! Conditional formatting mutations.

use crate::auth::{resolve_user_id, SheetUser};
use crate::state::SheetState;
use crate::types::{ConditionalFormatRequest, ConditionalFormatRule, SaveResponse};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_conditional_format(
    State(state): State<Arc<SheetState>>,
    user: Option<Extension<SheetUser>>,
    Json(req): Json<ConditionalFormatRequest>,
) -> Result<Json<SaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = resolve_user_id(user.as_deref())?;
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

    let mut sheet = session.sheet.write().await;
    if let Err(e) = botsheet_core::state::ensure_write_allowed(&user_id, &sheet) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": e }))));
    }
    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    let (start_row, start_col, end_row, end_col) = botsheet_core::types::bounds_from_request(
        req.range.as_deref(),
        req.start_row,
        req.start_col,
        req.end_row,
        req.end_col,
    );
    // The toolbar posts a comparison `value` and a fill colour instead of the
    // API's `condition` and `style.background`; merge both shapes here. The
    // renderer parses conditions as `>100`, `between:a:b`, `text:contains:x`
    // or the literal `duplicates`, so map the preset to that grammar.
    let condition = if req.condition.trim().is_empty() {
        let value = req.value.clone().unwrap_or_default();
        match req.rule_type.as_str() {
            "greater_than" => format!(">{value}"),
            "less_than" => format!("<{value}"),
            "equal_to" => format!("={value}"),
            "greater_equal" | "greater_or_equal" => format!(">={value}"),
            "less_equal" | "less_or_equal" => format!("<={value}"),
            "not_equal" => format!("<>{value}"),
            "between" => {
                if value.contains(':') {
                    format!("between:{value}")
                } else {
                    format!("between:{value}:{value}")
                }
            }
            "text_contains" => format!("text:contains:{value}"),
            "duplicate" | "duplicates" => "duplicates".to_string(),
            _ => value,
        }
    } else {
        req.condition.clone()
    };
    // Chat plans often say "negative" / "positive" instead of a comparison.
    let condition = match condition.trim().to_ascii_lowercase().as_str() {
        "negative" | "negatives" => "<0".to_string(),
        "positive" | "positives" => ">=0".to_string(),
        _ => condition,
    };
    let rule_type = if req.rule_type == "duplicate" {
        "duplicates".to_string()
    } else {
        req.rule_type.clone()
    };
    let mut style = req.style.clone();
    if style.background.is_none() {
        style.background = req.bg_color.clone();
    }
    let rule = ConditionalFormatRule {
        id: Uuid::new_v4().to_string(),
        start_row,
        start_col,
        end_row,
        end_col,
        rule_type,
        condition,
        style,
        priority: 1,
    };

    let formats = worksheet.conditional_formats.get_or_insert_with(Vec::new);
    formats.push(rule);

    sheet.updated_at = Utc::now();
    state.sessions.record(
        &session,
        &state,
        &user_id,
        "sheet_mutation",
        serde_json::json!({ "sheet_id": req.sheet_id }),
    );

    Ok(Json(SaveResponse {
        id: req.sheet_id,
        success: true,
        message: Some("Conditional format applied".to_string()),
    }))
}
