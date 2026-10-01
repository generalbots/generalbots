//! #1441 B — the Campaigns grid fragment.
//!
//! Cards expose the actions the marketing API already serves: Send
//! (`POST /api/crm/campaigns/:id/send`), metrics
//! (`GET /api/crm/metrics/campaign/:id`) and delete. #1453 wired the Send
//! button in the suite but no fragment ever rendered it, so the campaign view
//! was read-only in practice.

use axum::{extract::{Query, State}, http::HeaderMap, response::Html};
use diesel::prelude::*;
use std::sync::Arc;

use super::lists::or_dash;
use super::{branch_ctx, format_money, GridQuery};
use crate::models::html_escape;
use crate::CrateState;

/// `marketing_campaigns.budget` is NUMERIC in the live schema — `botmarketing`
/// models it as BigDecimal and owns the writes. The grid only needs a number to
/// display, so the cast happens in SQL instead of a column migration that would
/// break the owning crate (same approach as `crm_leads_compat`).
#[derive(Debug, diesel::QueryableByName)]
struct CampaignRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    campaign_type: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    status: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Float8>)]
    budget: Option<f64>,
}

/// `/api/ui/crm/campaigns` — campaign cards with create/send/metrics/delete.
pub async fn handle_crm_campaigns(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Query(query): Query<GridQuery>,
) -> Html<String> {
    let Ok(mut conn) = state.db_pool.get() else {
        return Html(r#"<div class="campaigns-empty">Service unavailable</div>"#.to_string());
    };
    let branch_id = branch_ctx(&state, &headers, &mut conn);

    let mut campaigns: Vec<CampaignRow> = diesel::sql_query(
        "SELECT id, name, campaign_type, status, budget::float8 AS budget \
         FROM marketing_campaigns WHERE branch_id = $1 \
         ORDER BY created_at DESC LIMIT 200",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .load(&mut conn)
    .unwrap_or_else(|e| {
        log::warn!("[crm] campaigns grid query failed: {e}");
        Vec::new()
    });

    if let Some(term) = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let needle = term.to_lowercase();
        campaigns.retain(|c| {
            [c.name.as_str(), c.campaign_type.as_str(), c.status.as_deref().unwrap_or("")]
                .iter()
                .any(|v| v.to_lowercase().contains(&needle))
        });
    }

    if campaigns.is_empty() {
        return Html(r#"<div class="campaigns-empty">No campaigns yet</div>"#.to_string());
    }

    let mut html = String::new();
    for campaign in campaigns {
        let status = campaign.status.clone().unwrap_or_else(|| "draft".to_string());
        let budget = campaign
            .budget
            .map(|b| html_escape(&format_money(b, None)))
            .unwrap_or_else(|| "-".to_string());
        let sendable = status == "draft" || status == "scheduled";
        html.push_str(&format!(
            r#"<div class="campaign-card" data-id="{id}" data-entity="campaign">
<div class="campaign-card-header"><span class="campaign-card-title">{name}</span><span class="campaign-card-type">{ctype}</span></div>
<div class="campaign-card-status {status}">{status_label}</div>
<div class="campaign-channels"><span class="campaign-channel-tag">{ctype}</span><span class="campaign-metric"><span class="campaign-metric-value">{budget}</span><span class="campaign-metric-label">budget</span></span></div>
<div class="campaign-card-actions">{send}<button class="campaign-send-btn row-action" data-action="send" data-entity="campaign" data-id="{id}">Send</button><button class="campaign-metrics-btn row-action" data-action="metrics" data-entity="campaign" data-id="{id}">Metrics</button><button class="row-action" data-action="edit" data-entity="campaign" data-id="{id}">Edit</button><button class="row-action danger" data-action="delete" data-entity="campaign" data-id="{id}">Delete</button></div>
</div>"#,
            id = campaign.id,
            name = html_escape(&campaign.name),
            ctype = or_dash(Some(campaign.campaign_type.as_str())),
            status = html_escape(&status),
            status_label = html_escape(&status),
            budget = budget,
            send = if sendable { "" } else { r#"<span class="campaign-locked" title="Already dispatched">dispatched</span>"# },
        ));
    }
    Html(html)
}
