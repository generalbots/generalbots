//! Usage and cost transparency (GB6-25).
//!
//! The suite used to estimate tokens from character counts and price them with
//! a hardcoded table, so a reply streamed in several chunks was counted several
//! times. This endpoint reports authoritative per-day, per-model token counts
//! from `message_history` and prices them only when the deployment publishes a
//! price list (`GB_LLM_PRICING`). No price list means the cost is reported as
//! unavailable instead of invented.

use std::collections::BTreeMap;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Json;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::settings_api::{get_conn, resolve_user_id};

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    #[serde(default)]
    pub days: Option<i64>,
}

#[derive(diesel::QueryableByName)]
struct UsageRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    day: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    model: Option<String>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    tokens: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    messages: i64,
}

fn parse_price_list(raw: &str) -> BTreeMap<String, f64> {
    if raw.trim().is_empty() {
        return BTreeMap::new();
    }
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw) else {
        log::warn!("usage price list is not valid JSON; costs will be unavailable");
        return BTreeMap::new();
    };
    let mut prices = BTreeMap::new();
    if let Some(map) = parsed.as_object() {
        for (model, value) in map {
            let price = value
                .get("input_per_million")
                .or_else(|| value.get("per_million"))
                .and_then(serde_json::Value::as_f64);
            if let Some(price) = price {
                prices.insert(model.to_ascii_lowercase(), price);
            }
        }
    }
    prices
}

/// Price per million tokens, keyed by model id (#1648). Order of preference:
/// the bot's own Vault configuration (`llm-pricing`, or `llm-price-<model>`),
/// then the deployment-wide `GB_LLM_PRICING`. No price means "unavailable",
/// never an invented number.
fn prices_for(state: &Arc<AppState>, models: &[String]) -> BTreeMap<String, f64> {
    let cfg = botcore::config::ConfigManager::new(state.conn.clone());
    let mut prices = cfg
        .get_config(&Uuid::nil(), "llm-pricing", Some(""))
        .map(|raw| parse_price_list(&raw))
        .unwrap_or_default();
    if prices.is_empty() {
        if let Ok(raw) = std::env::var("GB_LLM_PRICING") {
            prices = parse_price_list(&raw);
        }
    }
    for model in models {
        let key = format!("llm-price-{}", model.to_ascii_lowercase());
        if prices.contains_key(&model.to_ascii_lowercase()) {
            continue;
        }
        if let Some(price) = cfg
            .get_config(&Uuid::nil(), &key, Some(""))
            .ok()
            .and_then(|value| value.trim().parse::<f64>().ok())
        {
            prices.insert(model.to_ascii_lowercase(), price);
        }
    }
    prices
}

/// GET /api/user/usage/summary?days=30
pub async fn usage_summary(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<UsageQuery>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(conn) => conn,
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Database unavailable" })),
            )
        }
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err((status, _html)) => {
            return (
                status,
                Json(json!({ "error": "A valid session token is required" })),
            )
        }
    };
    let days = query.days.unwrap_or(30).clamp(1, 365);
    let rows = diesel::sql_query(
        "SELECT ((created_at AT TIME ZONE 'UTC')::date)::text AS day, \
         llm_model AS model, \
         COALESCE(SUM(token_count), 0)::bigint AS tokens, \
         COUNT(*)::bigint AS messages \
         FROM message_history \
         WHERE user_id = $1 AND created_at >= NOW() - ($2::text || ' days')::interval \
         GROUP BY 1, 2 ORDER BY 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(days.to_string())
    .load::<UsageRow>(&mut conn);
    let rows = match rows {
        Ok(rows) => rows,
        Err(error) => {
            log::warn!("usage summary query failed: {error}");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Usage could not be read" })),
            );
        }
    };

    let models: Vec<String> = rows
        .iter()
        .filter_map(|row| row.model.clone())
        .collect();
    let prices = prices_for(&state, &models);
    let mut per_day: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut total_tokens: i64 = 0;
    let mut total_messages: i64 = 0;
    let mut total_cost = 0.0_f64;
    let mut priced = 0_usize;
    let mut per_model: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for row in &rows {
        let model = row.model.clone().unwrap_or_else(|| "unknown".to_string());
        total_tokens += row.tokens;
        total_messages += row.messages;
        let day_entry = per_day
            .entry(row.day.clone())
            .or_insert_with(|| json!({ "day": row.day.clone(), "tokens": 0, "messages": 0 }));
        if let Some(object) = day_entry.as_object_mut() {
            let tokens = object.get("tokens").and_then(serde_json::Value::as_i64).unwrap_or(0);
            let messages = object.get("messages").and_then(serde_json::Value::as_i64).unwrap_or(0);
            object.insert("tokens".to_string(), json!(tokens + row.tokens));
            object.insert("messages".to_string(), json!(messages + row.messages));
        }
        let price = prices.get(&model.to_ascii_lowercase()).copied();
        if let Some(price) = price {
            priced += 1;
            total_cost += (row.tokens as f64 / 1_000_000.0) * price;
        }
        let entry = per_model.entry(model).or_insert_with(|| {
            json!({ "tokens": 0, "messages": 0, "cost": null })
        });
        if let Some(object) = entry.as_object_mut() {
            let tokens = object.get("tokens").and_then(serde_json::Value::as_i64).unwrap_or(0);
            let messages = object
                .get("messages")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(0);
            object.insert("tokens".to_string(), json!(tokens + row.tokens));
            object.insert("messages".to_string(), json!(messages + row.messages));
            if let Some(price) = price {
                let cost = (row.tokens as f64 / 1_000_000.0) * price;
                object.insert("cost".to_string(), json!(cost));
            }
        }
    }

    let pricing_available = !prices.is_empty();
    (
        axum::http::StatusCode::OK,
        Json(json!({
            "days": days,
            "source": "message_history",
            "pricing_available": pricing_available,
            "totals": {
                "tokens": total_tokens,
                "messages": total_messages,
                "cost": if pricing_available && priced > 0 { json!(total_cost) } else { json!(null) },
            },
            "models": per_model,
            "per_day": per_day.into_values().collect::<Vec<_>>(),
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::parse_price_list;
    use std::collections::BTreeMap;

    #[test]
    fn empty_price_list_means_cost_is_unavailable() {
        assert!(parse_price_list("").is_empty());
        assert!(parse_price_list("   ").is_empty());
    }

    #[test]
    fn invalid_json_is_ignored_instead_of_inventing_prices() {
        assert!(parse_price_list("{not json").is_empty());
        assert!(parse_price_list("[1, 2, 3]").is_empty());
    }

    #[test]
    fn per_model_prices_are_lowercased() {
        let prices: BTreeMap<String, f64> =
            parse_price_list(r#"{"GPT-4o-mini": {"input_per_million": 0.15}}"#);
        assert_eq!(prices.get("gpt-4o-mini").copied(), Some(0.15));
    }

    #[test]
    fn the_shorthand_key_is_accepted() {
        let prices = parse_price_list(r#"{"llama-3.1": {"per_million": 2.0}}"#);
        assert_eq!(prices.get("llama-3.1").copied(), Some(2.0));
    }
}
