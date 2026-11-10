//! LLM cost panel of the Analytics operational view (#2116): tokens, spend,
//! cost per conversation, spend per tenant and daily quota burn, read from
//! `llm_metrics` and `llm_budget`.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use diesel::RunQueryDsl;

use crate::analytics_types::{format_number, html_escape};
use crate::handlers_ops_traffic::{window_minutes, OpsWindowQuery};
use crate::DbPool;

// ---------------------------------------------------------------------------
// Cost: tokens, spend per conversation and per tenant, quota burn
// ---------------------------------------------------------------------------

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CostTotalsRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    requests: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    input_tokens: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    output_tokens: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    conversations: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    failures: i64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    cost: f64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TenantCostRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    tenant: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    tokens: i64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    cost: f64,
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct QuotaRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    bot: String,
    #[diesel(sql_type = diesel::sql_types::Double)]
    daily_spend: f64,
    #[diesel(sql_type = diesel::sql_types::Double)]
    daily_limit: f64,
}

struct CostData {
    totals: CostTotalsRow,
    tenants: Vec<TenantCostRow>,
    quotas: Vec<QuotaRow>,
}

fn load_cost(pool: &DbPool, minutes: i32) -> Option<CostData> {
    let Ok(mut conn) = pool.get() else {
        log::error!("ops cost panel: no DB connection");
        return None;
    };

    let totals = match diesel::sql_query(
        "SELECT
            COUNT(*)::bigint AS requests,
            COALESCE(SUM(input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens,
            COUNT(DISTINCT session_id)::bigint AS conversations,
            COUNT(*) FILTER (WHERE NOT success)::bigint AS failures,
            COALESCE(SUM(estimated_cost), 0)::float8 AS cost
         FROM llm_metrics
         WHERE timestamp > NOW() - ($1::int * INTERVAL '1 minute')",
    )
    .bind::<diesel::sql_types::Integer, _>(minutes)
    .get_result::<CostTotalsRow>(&mut conn)
    {
        Ok(row) => row,
        Err(error) => {
            log::error!("ops cost totals failed: {error}");
            return None;
        }
    };

    let tenants = diesel::sql_query(
        "SELECT COALESCE(o.name::text, 'unassigned') AS tenant,
                COALESCE(SUM(m.total_tokens), 0)::bigint AS tokens,
                COALESCE(SUM(m.estimated_cost), 0)::float8 AS cost
         FROM llm_metrics m
         LEFT JOIN bots b ON b.id = m.bot_id
         LEFT JOIN organizations o ON o.org_id = b.org_id
         WHERE m.timestamp > NOW() - ($1::int * INTERVAL '1 minute')
         GROUP BY 1
         ORDER BY cost DESC
         LIMIT 5",
    )
    .bind::<diesel::sql_types::Integer, _>(minutes)
    .load::<TenantCostRow>(&mut conn)
    .unwrap_or_else(|error| {
        log::error!("ops cost tenants failed: {error}");
        Vec::new()
    });

    let quotas = diesel::sql_query(
        "SELECT b.name::text AS bot,
                lb.daily_spend::float8 AS daily_spend,
                lb.daily_limit::float8 AS daily_limit
         FROM llm_budget lb
         JOIN bots b ON b.id = lb.bot_id
         WHERE lb.daily_limit > 0
         ORDER BY lb.daily_spend / NULLIF(lb.daily_limit, 0) DESC
         LIMIT 5",
    )
    .load::<QuotaRow>(&mut conn)
    .unwrap_or_else(|error| {
        log::error!("ops cost quota query failed: {error}");
        Vec::new()
    });

    Some(CostData {
        totals,
        tenants,
        quotas,
    })
}

fn render_cost(data: &CostData, minutes: i32) -> String {
    let totals = &data.totals;
    let per_conversation = if totals.conversations == 0 {
        None
    } else {
        Some(totals.cost / totals.conversations as f64)
    };
    let per_conversation_label = match per_conversation {
        Some(value) => format!("${value:.4}"),
        None => "—".to_string(),
    };

    let mut tenant_rows = String::new();
    for tenant in &data.tenants {
        tenant_rows.push_str(&format!(
            "<tr><td>{name}</td><td>{tokens}</td><td>${cost:.4}</td></tr>",
            name = html_escape(&tenant.tenant),
            tokens = format_number(tenant.tokens),
            cost = tenant.cost,
        ));
    }
    if tenant_rows.is_empty() {
        tenant_rows.push_str("<tr><td colspan=\"3\" class=\"ops-empty-cell\">No tenant spend recorded in this window.</td></tr>");
    }

    let mut quota_rows = String::new();
    for quota in &data.quotas {
        let burn = if quota.daily_limit > 0.0 {
            quota.daily_spend / quota.daily_limit
        } else {
            0.0
        };
        let class = if burn >= 1.0 {
            "ops-page"
        } else if burn >= 0.8 {
            "ops-warn"
        } else {
            "ops-ok"
        };
        quota_rows.push_str(&format!(
            "<tr><td>{bot}</td><td>${spend:.2} / ${limit:.2}</td><td class=\"{class}\">{pct:.0}%</td></tr>",
            bot = html_escape(&quota.bot),
            spend = quota.daily_spend,
            limit = quota.daily_limit,
            class = class,
            pct = burn * 100.0,
        ));
    }
    if quota_rows.is_empty() {
        quota_rows.push_str("<tr><td colspan=\"3\" class=\"ops-empty-cell\">No LLM budget rows configured.</td></tr>");
    }

    format!(
        "<div class=\"ops-panel\" id=\"ops-cost\">
    <div class=\"chart-header\"><h3>LLM cost &amp; tokens</h3><span class=\"ops-meta\">last {minutes} min · source: llm_metrics</span></div>
    <div class=\"ops-stats\">
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{requests}</span><span class=\"ops-stat-label\">LLM calls</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{input}</span><span class=\"ops-stat-label\">Input tokens</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{output}</span><span class=\"ops-stat-label\">Output tokens</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">${cost:.4}</span><span class=\"ops-stat-label\">Cost</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{per_conversation}</span><span class=\"ops-stat-label\">Cost / conversation</span></div>
        <div class=\"ops-stat\"><span class=\"ops-stat-value\">{failures}</span><span class=\"ops-stat-label\">Failed calls</span></div>
    </div>
    <div class=\"ops-columns\">
        <div>
            <h4 class=\"ops-subtitle\">Cost per tenant</h4>
            <table class=\"ops-table\"><thead><tr><th>Tenant</th><th>Tokens</th><th>Cost</th></tr></thead><tbody>{tenant_rows}</tbody></table>
        </div>
        <div>
            <h4 class=\"ops-subtitle\">Daily quota burn</h4>
            <table class=\"ops-table\"><thead><tr><th>Bot</th><th>Spend</th><th>Burn</th></tr></thead><tbody>{quota_rows}</tbody></table>
        </div>
    </div>
</div>",
        minutes = minutes,
        requests = format_number(totals.requests),
        input = format_number(totals.input_tokens),
        output = format_number(totals.output_tokens),
        cost = totals.cost,
        per_conversation = per_conversation_label,
        failures = format_number(totals.failures),
        tenant_rows = tenant_rows,
        quota_rows = quota_rows,
    )
}

pub async fn handle_ops_cost(
    State(pool): State<Arc<DbPool>>,
    Query(query): Query<OpsWindowQuery>,
) -> impl IntoResponse {
    let minutes = window_minutes(query.window);
    let conn = pool.clone();
    let loaded = tokio::task::spawn_blocking(move || load_cost(&conn, minutes))
        .await
        .unwrap_or_else(|error| {
            log::error!("ops cost panel task failed: {error}");
            None
        });

    match loaded {
        Some(data) => Html(render_cost(&data, minutes)),
        None => Html(
            "<div class=\"ops-panel\" id=\"ops-cost\"><p class=\"ops-empty\">Cost data unavailable: the database pool is not reachable.</p></div>"
                .to_string(),
        ),
    }
}

#[cfg(test)]
#[path = "handlers_ops_cost_tests.rs"]
mod tests;
