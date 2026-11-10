//! Deal capture and kanban stage moves (`crm.deal.create`,
//! `crm.deal.move-stage`).

use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use crate::core::bot::api_catalog::branch_scope;
use serde_json::{json, Value};
use super::contacts::{ensure_account, ensure_contact};
use super::rows::{stage_probability, StageRow};

/// #1441 C8 — `crm.deal.create`: capture a deal from chat with optional
/// contact/account linkage (dedupe by email, by company name).
pub(crate) async fn crm_create_deal_command(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let str_p = |k: &str| {
        params
            .get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
    };
    let title = str_p("title")
        .or_else(|| str_p("name"))
        .filter(|t| !t.is_empty())
        .ok_or_else(|| "params.title is required".to_string())?;
    let value: Option<f64> = params.get("value").and_then(|v| v.as_f64());
    let currency = str_p("currency").unwrap_or_else(|| "USD".to_string());
    let stage = str_p("stage").unwrap_or_else(|| "new".to_string());

    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let now = chrono::Utc::now();

    let contact_id = match str_p("email").filter(|e| !e.is_empty()) {
        Some(mail) => Some(ensure_contact(
            state,
            &mut conn,
            branch,
            *bot_uuid,
            &Some(mail),
            params,
        )?),
        None => None,
    };
    let account_id = match str_p("company").filter(|c| !c.is_empty()) {
        Some(company) => Some(ensure_account(&mut conn, branch, &company)?),
        None => None,
    };

    let deal_id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO crm_deals (id, org_id, bot_id, branch_id, contact_id, account_id, name, title, value, currency, stage, probability, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, '', $7, $8, $9, $10, $11, $12, $12)",
    )
    .bind::<diesel::sql_types::Uuid, _>(deal_id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Uuid, _>(*bot_uuid)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(contact_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(account_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(title.clone()))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Float8>, _>(value)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(currency.clone()))
    .bind::<diesel::sql_types::Text, _>(stage.clone())
    .bind::<diesel::sql_types::Int4, _>(stage_probability(&stage))
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("insert deal: {e}"))?;

    Ok(json!({
        "summary": format!("Deal '{title}' created at stage '{stage}'."),
        "deal_id": deal_id,
        "contact_id": contact_id,
        "account_id": account_id,
        "value": value,
        "currency": currency,
    }))
}

/// #1441 C8 — `crm.deal.move-stage`: the kanban drag-and-drop, from chat.
pub(crate) async fn crm_move_deal_stage_command(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let deal_id = params
        .get("deal_id")
        .and_then(|v| v.as_str())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or_else(|| "params.deal_id (uuid) is required".to_string())?;
    let stage = params
        .get("stage")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "params.stage is required".to_string())?;

    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;

    let previous: Option<Option<String>> =
        diesel::sql_query("SELECT stage FROM crm_deals WHERE id = $1 AND branch_id = $2")
            .bind::<diesel::sql_types::Uuid, _>(deal_id)
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .load::<StageRow>(&mut conn)
            .ok()
            .and_then(|rows| rows.into_iter().next())
            .map(|r| r.stage);

    diesel::sql_query(
        "UPDATE crm_deals SET stage = $1, probability = $2, updated_at = NOW() \
         WHERE id = $3 AND branch_id = $4",
    )
    .bind::<diesel::sql_types::Text, _>(stage.clone())
    .bind::<diesel::sql_types::Int4, _>(stage_probability(&stage))
    .bind::<diesel::sql_types::Uuid, _>(deal_id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .execute(&mut conn)
    .map_err(|e| format!("update stage: {e}"))?;

    let previous_stage = previous.flatten();
    Ok(json!({
        "summary": format!(
            "Deal moved from '{}' to '{stage}'.",
            previous_stage.clone().unwrap_or_else(|| "unset".to_string())
        ),
        "deal_id": deal_id,
        "previous_stage": previous_stage,
        "stage": stage,
    }))
}
