//! Leads-by-stage rollup (`crm.leads.report`).

use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use crate::core::bot::api_catalog::branch_scope;
use serde_json::{json, Value};
use super::rows::StageRollupRow;

/// #1441 P3 — leads-by-stage rollup for the chat surface.
pub(crate) async fn crm_pipeline_report_command(
    state: &Arc<AppState>,
    bot_uuid: &Uuid,
) -> Result<Value, String> {
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
