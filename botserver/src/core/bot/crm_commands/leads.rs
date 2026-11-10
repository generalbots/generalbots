//! Lead capture from chat (`crm.leads.create`).

use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use crate::core::bot::api_catalog::branch_scope;
use serde_json::{json, Value};
use super::rows::ContactIdRow;

/// #1441 P3 — capture a lead from chat: creates the contact (dedupe by email)
/// and the lead row, mirroring the suite lead-form linkage (A1).
pub(crate) async fn crm_create_lead_command(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let str_p = |k: &str| {
        params
            .get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };
    let title = str_p("title").unwrap_or_default();
    let email = str_p("email").filter(|e| !e.trim().is_empty());
    let first = str_p("first_name");
    let last = str_p("last_name");
    if title.trim().is_empty() && first.is_none() && last.is_none() {
        return Err("params.title or params.first_name/last_name is required".to_string());
    }
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let now = chrono::Utc::now();
    let contact_id: Option<Uuid> = match &email {
        Some(mail) => {
            let existing: Option<Uuid> = diesel::sql_query(
                "SELECT id AS contact_id FROM crm_contacts WHERE branch_id = $1 AND email ILIKE $2 LIMIT 1",
            )
            .bind::<diesel::sql_types::Uuid, _>(branch)
            .bind::<diesel::sql_types::Text, _>(mail)
            .get_result::<ContactIdRow>(&mut conn)
            .map(|r| r.contact_id)
            .ok();
            match existing {
                Some(id) => Some(id),
                None => {
                    let cid = Uuid::new_v4();
                    diesel::sql_query(
                        "INSERT INTO crm_contacts (id, org_id, bot_id, branch_id, first_name, last_name, email, company, job_title, status, created_at, updated_at) \
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'lead', $10, $10)",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(cid)
                    .bind::<diesel::sql_types::Uuid, _>(branch)
                    .bind::<diesel::sql_types::Uuid, _>(*bot_uuid)
                    .bind::<diesel::sql_types::Uuid, _>(branch)
                    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(first.clone())
                    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(last.clone())
                    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(email.clone())
                    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(str_p("company"))
                    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(str_p("job_title"))
                    .bind::<diesel::sql_types::Timestamptz, _>(now)
                    .execute(&mut conn)
                    .map_err(|e| format!("insert contact: {e}"))?;
                    Some(cid)
                }
            }
        }
        None => None,
    };
    let lead_id = Uuid::new_v4();
    let value: Option<f64> = params.get("value").and_then(|v| v.as_f64());
    let currency = str_p("currency").unwrap_or_else(|| "USD".to_string());
    let display_title = if title.is_empty() {
        let name = [first.as_deref(), last.as_deref()]
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<&str>>()
            .join(" ");
        if name.is_empty() {
            "Chat lead".to_string()
        } else {
            format!("Lead — {name}")
        }
    } else {
        title
    };
    diesel::sql_query(
        "INSERT INTO crm_deals (id, org_id, bot_id, branch_id, contact_id, name, title, value, currency, stage, probability, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, '', $6, $7, $8, 'new', 10, $9, $9)",
    )
    .bind::<diesel::sql_types::Uuid, _>(lead_id)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Uuid, _>(*bot_uuid)
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(contact_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(display_title.clone()))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Float8>, _>(value)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Some(currency.clone()))
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("insert lead: {e}"))?;
    Ok(json!({
        "summary": format!("Lead '{display_title}' created (stage: new)."),
        "lead_id": lead_id,
        "contact_id": contact_id,
        "value": value,
        "currency": currency,
    }))
}
