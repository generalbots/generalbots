//! Service-tax calculation in the products/services registry (issue #722).
//!
//! `POST /api/products/services/:id/tax` computes the Brazilian service-tax
//! breakdown for a registered service using the shared bottax engine. Rates
//! come from `billing_tax_rates` (branch-scoped); a per-service `tax_rate`
//! override (effective total %) is split proportionally across
//! IRPJ/CSLL/PIS-COFINS/ISS. The calculation is returned immediately — no
//! record is persisted per call.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::ProductsState;

#[derive(Debug, Deserialize)]
pub struct ServiceTaxRequest {
    /// Optional explicit service value (overrides the service price).
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceTaxQuery {
    /// Resolve the service by name when the path id is not a UUID.
    pub name: Option<String>,
}

#[derive(diesel::QueryableByName)]
struct ServiceRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    service_type: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
    fixed_price: Option<Decimal>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
    hourly_rate: Option<Decimal>,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    tax_rate: Decimal,
}

pub async fn calculate_service_tax(
    State(state): State<Arc<ProductsState>>,
    Path(id): Path<String>,
    Query(query): Query<ServiceTaxQuery>,
    Json(req): Json<ServiceTaxRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;
    let branch_id = crate::get_bot_context(&state.pool, &state.get_default_bot);
    let result = calculate_service_tax_inner(
        &mut conn,
        branch_id,
        &id,
        query.name.as_deref(),
        req.value.as_deref(),
    )
    .await;
    match result {
        Ok(value) => Ok(Json(value)),
        Err((code, msg)) => Err((code, msg)),
    }
}

/// Shared business logic for the service-tax calculation (#722). Resolves the
/// service by id or name, applies the branch tax rates and returns the
/// breakdown JSON. Used by the REST handler and by the LLM api-command
/// catalog so chat can answer fiscal questions without a BASIC keyword.
pub async fn calculate_service_tax_inner(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    service_id_or_name: &str,
    name: Option<&str>,
    value: Option<&str>,
) -> Result<serde_json::Value, (StatusCode, String)> {
    use diesel::prelude::*;
    let id = service_id_or_name;

    let by_id = if let Ok(uid) = Uuid::parse_str(id) {
        diesel::sql_query(
            "SELECT id, name, service_type, fixed_price, hourly_rate, tax_rate \
             FROM services WHERE id = $1 AND ($2 = '00000000-0000-0000-0000-000000000000' OR branch_id = $2) LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(uid)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<ServiceRow>(conn)
        .ok()
    } else {
        None
    };
    let by_name = by_id.or_else(|| {
        let lookup = if Uuid::parse_str(id).is_ok() {
            name.unwrap_or_default().to_string()
        } else {
            id.to_string()
        };
        if lookup.is_empty() {
            return None;
        }
        diesel::sql_query(
            "SELECT id, name, service_type, fixed_price, hourly_rate, tax_rate \
             FROM services WHERE name ILIKE $1 AND ($2 = '00000000-0000-0000-0000-000000000000' OR branch_id = $2) LIMIT 1",
        )
        .bind::<diesel::sql_types::Text, _>(format!("%{lookup}%"))
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<ServiceRow>(conn)
        .ok()
    });
    let service = by_name.ok_or((StatusCode::NOT_FOUND, format!("Service '{id}' not found")))?;

    let service_value = value
        .map(Decimal::from_str)
        .transpose()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid value: {e}")))?
        .or(service.fixed_price)
        .or(service.hourly_rate)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Service has no price; provide 'value'".to_string()))?;

    let loaded = bottax::storage::load_rates_from_billing(conn, &branch_id);
    let mut rates = loaded.unwrap_or_default();
    let source = if loaded.is_some() {
        "billing_tax_rates"
    } else {
        "default"
    };

    // Per-service effective total override: scale proportionally so the four
    // components sum to the configured percentage.
    if service.tax_rate > Decimal::ZERO {
        let total_default = rates.irpj_pct + rates.csll_pct + rates.pis_cofins_pct + rates.iss_pct;
        if total_default > Decimal::ZERO {
            let factor = service.tax_rate / total_default;
            rates.irpj_pct = (rates.irpj_pct * factor).round_dp(2);
            rates.csll_pct = (rates.csll_pct * factor).round_dp(2);
            rates.pis_cofins_pct = (rates.pis_cofins_pct * factor).round_dp(2);
            rates.iss_pct = (rates.iss_pct * factor).round_dp(2);
        }
    }

    let breakdown = bottax::calculator::calculate_service_tax(service_value, &rates);

    if let Err((_code, msg)) = bottax::handlers::persist_calculation(
        conn,
        &branch_id,
        Some(service.id),
        Some(&service.name),
        &breakdown,
        source,
    ) {
        log::warn!("service.tax audit insert skipped: {msg}");
    }

    Ok(json!({
        "service": {
            "id": service.id,
            "name": service.name,
            "service_type": service.service_type,
        },
        "service_value": breakdown.service_value.to_string(),
        "breakdown": breakdown,
        "rates": {
            "irpj_pct": rates.irpj_pct.to_string(),
            "csll_pct": rates.csll_pct.to_string(),
            "pis_cofins_pct": rates.pis_cofins_pct.to_string(),
            "iss_pct": rates.iss_pct.to_string(),
        },
        "rate_source": source,
    }))
}
