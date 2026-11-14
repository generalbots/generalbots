//! Conditional formatting mutations.

use crate::auth::{SheetUser, resolve_user_id};
use crate::state::SheetState;
use crate::types::{ConditionalFormatRequest, ConditionalFormatRule, SaveResponse, Worksheet};
use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Builds and appends one conditional format rule from a request-shaped payload.
pub fn apply_conditional_request(
    worksheet: &mut Worksheet,
    req: &ConditionalFormatRequest,
) -> bool {
    let (start_row, start_col, end_row, end_col) = botsheet_core::types::bounds_from_request(
        req.range.as_deref(),
        req.start_row,
        req.start_col,
        req.end_row,
        req.end_col,
    );
    let condition = if req.condition.trim().is_empty() {
        let value = req.value.clone().unwrap_or_default();
        match req.rule_type.as_str() {
            "greater_than" => format!(">{value}"),
            "less_than" => format!("<{value}"),
            "equal_to" => format!("={value}"),
            "greater_equal" | "greater_or_equal" => format!(">={value}"),
            "less_equal" | "less_or_equal" => format!("<={value}"),
            "not_equal" => format!("<>{value}"),
            "between" if value.contains(':') => format!("between:{value}"),
            "between" => format!("between:{value}:{value}"),
            "text_contains" => format!("text:contains:{value}"),
            "duplicate" | "duplicates" => "duplicates".to_string(),
            _ => value,
        }
    } else {
        let raw = req.condition.trim();
        let lowered = raw.to_ascii_lowercase().replace(['-', ' '], "_");
        let value = req.value.clone().unwrap_or_default();
        let comparison = matches!(
            lowered.as_str(),
            "lessthan"
                | "less_than"
                | "greaterthan"
                | "greater_than"
                | "equal"
                | "equalto"
                | "equal_to"
                | "notequal"
                | "not_equal"
        );
        if comparison && value.is_empty() {
            raw.to_string()
        } else {
            match lowered.as_str() {
                "lessthan" | "less_than" => format!("<{value}"),
                "greaterthan" | "greater_than" => format!(">{value}"),
                "equal" | "equalto" | "equal_to" => format!("={value}"),
                "notequal" | "not_equal" => format!("<>{value}"),
                _ => raw.to_string(),
            }
        }
    };
    let condition = match condition.trim().to_ascii_lowercase().as_str() {
        "negative" | "negatives" => "<0".to_string(),
        "positive" | "positives" => ">=0".to_string(),
        _ => condition,
    };
    let mut style = req.style.clone();
    if style.background.is_none() {
        style.background = req.bg_color.clone();
    }
    worksheet
        .conditional_formats
        .get_or_insert_with(Vec::new)
        .push(ConditionalFormatRule {
            id: Uuid::new_v4().to_string(),
            start_row,
            start_col,
            end_row,
            end_col,
            rule_type: if req.rule_type == "duplicate" {
                "duplicates".to_string()
            } else {
                req.rule_type.clone()
            },
            condition,
            style,
            priority: 1,
        });
    true
}

/// Applies inline conditional rules (JSON shape of /api/sheet/conditional-format).
pub fn apply_inline_conditional_rules(
    worksheet: &mut Worksheet,
    sheet_id: &str,
    worksheet_index: usize,
    rules: &Option<Vec<serde_json::Value>>,
) {
    let Some(rules) = rules.as_ref() else {
        return;
    };
    for rule in rules {
        let Some(map) = rule.as_object() else {
            continue;
        };
        let mut payload = map.clone();
        payload
            .entry("sheet_id".to_string())
            .or_insert_with(|| serde_json::Value::String(sheet_id.to_string()));
        payload
            .entry("worksheet_index".to_string())
            .or_insert_with(|| serde_json::Value::Number(worksheet_index.into()));
        if let Ok(req) =
            serde_json::from_value::<ConditionalFormatRequest>(serde_json::Value::Object(payload))
        {
            let _ = apply_conditional_request(worksheet, &req);
        }
    }
}

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
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": e })),
        ));
    }
    if req.worksheet_index >= sheet.worksheets.len() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid worksheet index" })),
        ));
    }

    let worksheet = &mut sheet.worksheets[req.worksheet_index];
    apply_conditional_request(worksheet, &req);

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
