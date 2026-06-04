use axum::{
    extract::State,
    Json,
};
use diesel::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::AppState;
use crate::models::extract_user_from_session;

#[derive(Debug, Deserialize)]
pub struct DraftRequest {
    pub account_id: String,
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: Option<String>,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub draft_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DraftResponse {
    pub id: String,
    pub updated_at: String,
}

#[derive(QueryableByName)]
struct TimestampRow {
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    ts: chrono::DateTime<chrono::Utc>,
}

pub async fn upsert_draft(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DraftRequest>,
) -> Result<Json<DraftResponse>, DraftError> {
    let user_id = extract_user_from_session()
        .map_err(|_| DraftError("Authentication required".into()))?;

    let account_uuid = Uuid::parse_str(&req.account_id)
        .map_err(|_| DraftError("Invalid account ID".into()))?;

    let body_combined = match (req.body_html.as_deref(), req.body_text.as_deref()) {
        (Some(html), Some(text)) => format!("<!--html-->{html}\n{text}"),
        (Some(html), None) => html.to_string(),
        (None, Some(text)) => text.to_string(),
        (None, None) => String::new(),
    };

    let pool = state.pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db = pool.get().map_err(|e| format!("DB pool error: {e}"))?;

        if let Some(ref draft_id_str) = req.draft_id {
            let draft_uuid = Uuid::parse_str(draft_id_str)
                .map_err(|_| "Invalid draft ID".to_string())?;

            let affected = diesel::sql_query(
                "UPDATE email_drafts SET \
                 to_address = $2, cc_address = $3, bcc_address = $4, \
                 subject = $5, body = $6, updated_at = NOW() \
                 WHERE id = $1 AND user_id = $7",
            )
            .bind::<diesel::sql_types::Uuid, _>(draft_uuid)
            .bind::<diesel::sql_types::Text, _>(&req.to)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.cc.as_deref())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.bcc.as_deref())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.subject.as_deref())
            .bind::<diesel::sql_types::Text, _>(&body_combined)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .execute(&mut db)
            .map_err(|e| format!("Draft update failed: {e}"))?;

            if affected == 0 {
                return Err("Draft not found or access denied".to_string());
            }

            let row: TimestampRow = diesel::sql_query(
                "SELECT updated_at FROM email_drafts WHERE id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(draft_uuid)
            .get_result(&mut db)
            .map_err(|e| format!("Failed to read updated_at: {e}"))?;

            Ok::<_, String>((draft_uuid, row.ts))
        } else {
            let new_id = Uuid::new_v4();

            diesel::sql_query(
                "INSERT INTO email_drafts \
                 (id, user_id, account_id, to_address, cc_address, bcc_address, subject, body) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind::<diesel::sql_types::Uuid, _>(new_id)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .bind::<diesel::sql_types::Uuid, _>(account_uuid)
            .bind::<diesel::sql_types::Text, _>(&req.to)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.cc.as_deref())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.bcc.as_deref())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(req.subject.as_deref())
            .bind::<diesel::sql_types::Text, _>(&body_combined)
            .execute(&mut db)
            .map_err(|e| format!("Draft insert failed: {e}"))?;

            let row: TimestampRow = diesel::sql_query(
                "SELECT created_at FROM email_drafts WHERE id = $1",
            )
            .bind::<diesel::sql_types::Uuid, _>(new_id)
            .get_result(&mut db)
            .map_err(|e| format!("Failed to read created_at: {e}"))?;

            Ok::<_, String>((new_id, row.ts))
        }
    })
    .await;

    let (id, ts) = result
        .map_err(|e| DraftError(format!("Task error: {e}")))?
        .map_err(DraftError)?;

    info!("Draft saved: id={id}");
    Ok(Json(DraftResponse {
        id: id.to_string(),
        updated_at: ts.to_rfc3339(),
    }))
}

pub struct DraftError(pub String);

impl axum::response::IntoResponse for DraftError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draft_request_defaults() {
        let req = DraftRequest {
            account_id: "00000000-0000-0000-0000-000000000001".into(),
            to: "user@example.com".into(),
            cc: None,
            bcc: None,
            subject: None,
            body_html: None,
            body_text: None,
            draft_id: None,
        };
        assert!(req.cc.is_none());
        assert!(req.draft_id.is_none());
    }

    #[test]
    fn test_body_combined_html_only() {
        let html = Some("<p>Hello</p>".to_string());
        let text: Option<String> = None;
        let combined = match (html.as_deref(), text.as_deref()) {
            (Some(h), _) => h.to_string(),
            _ => String::new(),
        };
        assert_eq!(combined, "<p>Hello</p>");
    }
}
