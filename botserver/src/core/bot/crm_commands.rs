//! #1441 P3 — CRM chat-command executors (`crm.pipeline.forecast`,
//! `crm.leads.create`, `crm.leads.report`), split out of `api_catalog.rs` to
//! respect the 450-line budget. The dispatch arms live in `api_catalog.rs`;
//! every helper here is branch-scoped through `branch_scope`.

use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use crate::core::bot::api_catalog::branch_scope;
use serde_json::{json, Value};

/// #1441 P3 — weighted pipeline forecast for the chat surface. Reuses the
/// branch's open opportunities with the same stage weights the suite forecast
/// endpoint applies; closed/stalled stages contribute nothing.
pub(crate) async fn crm_forecast_command(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    periods: i64,
) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<ForecastDealRow> = diesel::sql_query(
        "SELECT stage, value FROM crm_deals \
         WHERE branch_id = $1 AND (closed_at IS NULL) ",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(|e| format!("Query error: {e}"))?;

    let weight = |stage: Option<&str>| match stage {
        Some("new") => 0.10,
        Some("qualified") => 0.25,
        Some("proposal") => 0.50,
        Some("negotiation") => 0.75,
        _ => 0.0,
    };
    let mut weighted_total = 0.0f64;
    let mut open_count = 0usize;
    let mut pipeline_value = 0.0f64;
    for row in &rows {
        let v = row.value.unwrap_or(0.0);
        pipeline_value += v;
        let w = weight(row.stage.as_deref());
        if w > 0.0 {
            open_count += 1;
            weighted_total += v * w;
        }
    }
    let per_period = if periods > 0 { weighted_total / periods as f64 } else { 0.0 };
    Ok(json!({
        "summary": format!(
            "Pipeline {open_count} open opportunities worth {pipeline_value:.2}; weighted forecast {weighted_total:.2} over {periods} month(s) ({per_period:.2}/month)."
        ),
        "pipeline_value": pipeline_value,
        "weighted_forecast": weighted_total,
        "open_count": open_count,
        "periods": periods,
        "per_period": per_period,
    }))
}

/// #1441 P3 — capture a lead from chat: creates the contact (dedupe by email)
/// and the lead row, mirroring the suite lead-form linkage (A1).
pub(crate) async fn crm_create_lead_command(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let str_p = |k: &str| params.get(k).and_then(|v| v.as_str()).map(|s| s.to_string());
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
        let name = [first.as_deref(), last.as_deref()].iter().flatten().copied().collect::<Vec<&str>>().join(" ");
        if name.is_empty() { "Chat lead".to_string() } else { format!("Lead — {name}") }
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

#[derive(diesel::QueryableByName)]
struct ContactIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    contact_id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct ForecastDealRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    stage: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
    value: Option<f64>,
}

#[derive(diesel::QueryableByName)]
struct StageRollupRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    stage: Option<String>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total: i64,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
    value: Option<f64>,
}

/// #1441 P3 — leads-by-stage rollup for the chat surface.
pub(crate) async fn crm_pipeline_report_command(state: &Arc<AppState>, bot_uuid: &Uuid) -> Result<Value, String> {
    let branch = branch_scope(state, bot_uuid)?;
    let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
    let rows: Vec<StageRollupRow> = diesel::sql_query(
        "SELECT stage, COUNT(*) AS total, SUM(value) AS value FROM crm_deals \
         WHERE branch_id = $1 GROUP BY stage ORDER BY stage",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load(&mut conn)
    .map_err(|e| format!("Query error: {e}"))?;
    let stages: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "stage": r.stage.unwrap_or_else(|| "unknown".to_string()),
                "count": r.total,
                "value": r.value.unwrap_or(0.0),
            })
        })
        .collect();
    Ok(json!({ "summary": "Leads by stage", "stages": stages }))
}

