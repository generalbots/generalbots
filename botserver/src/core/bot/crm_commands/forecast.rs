//! Weighted pipeline forecast (`crm.pipeline.forecast`).

use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

use crate::core::bot::api_catalog::branch_scope;
use serde_json::{json, Value};
use super::rows::ForecastDealRow;

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
    let per_period = if periods > 0 {
        weighted_total / periods as f64
    } else {
        0.0
    };
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
