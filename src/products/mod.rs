pub mod api;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{products, services, price_lists};
use crate::shared::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub category: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

fn bd_to_f64(bd: &BigDecimal) -> f64 {
    bd.to_f64().unwrap_or(0.0)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn format_currency(amount: f64, currency: &str) -> String {
    match currency.to_uppercase().as_str() {
        "USD" => format!("${:.2}", amount),
        "EUR" => format!("€{:.2}", amount),
        "GBP" => format!("£{:.2}", amount),
        "BRL" => format!("R${:.2}", amount),
        _ => format!("{:.2} {}", amount, currency),
    }
}

pub fn configure_products_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ui/products/items", get(handle_products_items))
        .route("/api/ui/products/services", get(handle_products_services))
        .route("/api/ui/products/pricelists", get(handle_products_pricelists))
        .route("/api/ui/products/stats/total-products", get(handle_total_products))
        .route("/api/ui/products/stats/total-services", get(handle_total_services))
        .route("/api/ui/products/stats/pricelists", get(handle_total_pricelists))
        .route("/api/ui/products/stats/active", get(handle_active_products))
        .route("/api/ui/products/search", get(handle_products_search))
}

async fn handle_products_items(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProductQuery>,
) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = products::table
            .filter(products::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref category) = query.category {
            db_query = db_query.filter(products::category.eq(category));
        }

        if let Some(ref status) = query.status {
            let is_active = status == "active";
            db_query = db_query.filter(products::is_active.eq(is_active));
        }

        db_query = db_query.order(products::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                products::id,
                products::sku,
                products::name,
                products::description,
                products::category,
                products::price,
                products::currency,
                products::stock_quantity,
                products::is_active,
            ))
            .load::<(Uuid, Option<String>, String, Option<String>, Option<String>, BigDecimal, String, i32, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, sku, name, desc, category, price, currency, stock, is_active) in items {
                let sku_str = sku.unwrap_or_else(|| "-".to_string());
                let desc_str = desc.unwrap_or_default();
                let cat_str = category.unwrap_or_else(|| "Uncategorized".to_string());
                let price_str = format_currency(bd_to_f64(&price), &currency);
                let stock_str = stock.to_string();
                let status_class = if is_active { "status-active" } else { "status-inactive" };
                let status_text = if is_active { "Active" } else { "Inactive" };

                html.push_str(&format!(
                    r##"<div class="product-card" data-id="{id}">
                        <div class="product-header">
                            <span class="product-name">{}</span>
                            <span class="product-sku">{}</span>
                        </div>
                        <div class="product-body">
                            <p class="product-desc">{}</p>
                            <div class="product-meta">
                                <span class="product-category">{}</span>
                                <span class="product-price">{}</span>
                                <span class="product-stock">Stock: {}</span>
                                <span class="{}">{}</span>
                            </div>
                        </div>
                        <div class="product-actions">
                            <button class="btn-sm" hx-get="/api/products/{id}" hx-target="#product-detail">View</button>
                            <button class="btn-sm btn-secondary" hx-get="/api/products/{id}/edit" hx-target="#modal-content">Edit</button>
                        </div>
                    </div>"##,
                    html_escape(&name),
                    html_escape(&sku_str),
                    html_escape(&desc_str),
                    html_escape(&cat_str),
                    price_str,
                    stock_str,
                    status_class,
                    status_text
                ));
            }
            Html(format!(r##"<div class="products-grid">{html}</div>"##))
        }
        _ => Html(
            r##"<div class="products-empty">
                <div class="empty-icon">📦</div>
                <p>No products yet</p>
                <p class="empty-hint">Add your first product to get started</p>
            </div>"##.to_string(),
        ),
    }
}

async fn handle_products_services(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProductQuery>,
) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = services::table
            .filter(services::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref category) = query.category {
            db_query = db_query.filter(services::category.eq(category));
        }

        if let Some(ref status) = query.status {
            let is_active = status == "active";
            db_query = db_query.filter(services::is_active.eq(is_active));
        }

        db_query = db_query.order(services::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                services::id,
                services::name,
                services::description,
                services::category,
                services::service_type,
                services::hourly_rate,
                services::fixed_price,
                services::currency,
                services::duration_minutes,
                services::is_active,
            ))
            .load::<(Uuid, String, Option<String>, Option<String>, String, Option<BigDecimal>, Option<BigDecimal>, String, Option<i32>, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, name, desc, category, svc_type, hourly, fixed, currency, duration, is_active) in items {
                let _desc_str = desc.unwrap_or_default();
                let cat_str = category.unwrap_or_else(|| "General".to_string());
                let type_str = svc_type;
                let price_str = if let Some(ref h) = hourly {
                    format!("{}/hr", format_currency(bd_to_f64(h), &currency))
                } else if let Some(ref f) = fixed {
                    format_currency(bd_to_f64(f), &currency)
                } else {
                    "-".to_string()
                };
                let duration_str = duration.map(|d| format!("{} min", d)).unwrap_or_else(|| "-".to_string());
                let status_class = if is_active { "status-active" } else { "status-inactive" };
                let status_text = if is_active { "Active" } else { "Inactive" };

                html.push_str(&format!(
                    r##"<tr class="service-row" data-id="{id}">
                        <td class="service-name">{}</td>
                        <td class="service-category">{}</td>
                        <td class="service-type">{}</td>
                        <td class="service-price">{}</td>
                        <td class="service-duration">{}</td>
                        <td class="service-status"><span class="{}">{}</span></td>
                        <td class="service-actions">
                            <button class="btn-sm" hx-get="/api/products/services/{id}" hx-target="#service-detail">View</button>
                        </td>
                    </tr>"##,
                    html_escape(&name),
                    html_escape(&cat_str),
                    html_escape(&type_str),
                    price_str,
                    duration_str,
                    status_class,
                    status_text
                ));
            }
            Html(html)
        }
        _ => Html(
            r##"<tr class="empty-row">
                <td colspan="7" class="empty-state">
                    <div class="empty-icon">🔧</div>
                    <p>No services yet</p>
                    <p class="empty-hint">Add services to your catalog</p>
                </td>
            </tr>"##.to_string(),
        ),
    }
}

async fn handle_products_pricelists(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProductQuery>,
) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let mut db_query = price_lists::table
            .filter(price_lists::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(ref status) = query.status {
            let is_active = status == "active";
            db_query = db_query.filter(price_lists::is_active.eq(is_active));
        }

        db_query = db_query.order(price_lists::created_at.desc());

        let limit = query.limit.unwrap_or(50);
        db_query = db_query.limit(limit);

        db_query
            .select((
                price_lists::id,
                price_lists::name,
                price_lists::description,
                price_lists::currency,
                price_lists::is_default,
                price_lists::discount_percent,
                price_lists::customer_group,
                price_lists::is_active,
            ))
            .load::<(Uuid, String, Option<String>, String, bool, BigDecimal, Option<String>, bool)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, name, _desc, currency, is_default, discount, customer_group, is_active) in items {
                let discount_pct = bd_to_f64(&discount);
                let discount_str = if discount_pct > 0.0 { format!("{:.1}%", discount_pct) } else { "-".to_string() };
                let group_str = customer_group.unwrap_or_else(|| "All".to_string());
                let default_badge = if is_default { r##"<span class="badge badge-primary">Default</span>"## } else { "" };
                let status_class = if is_active { "status-active" } else { "status-inactive" };
                let status_text = if is_active { "Active" } else { "Inactive" };

                html.push_str(&format!(
                    r##"<tr class="pricelist-row" data-id="{id}">
                        <td class="pricelist-name">{} {}</td>
                        <td class="pricelist-currency">{}</td>
                        <td class="pricelist-discount">{}</td>
                        <td class="pricelist-group">{}</td>
                        <td class="pricelist-status"><span class="{}">{}</span></td>
                        <td class="pricelist-actions">
                            <button class="btn-sm" hx-get="/api/products/pricelists/{id}" hx-target="#pricelist-detail">View</button>
                        </td>
                    </tr>"##,
                    html_escape(&name),
                    default_badge,
                    currency,
                    discount_str,
                    html_escape(&group_str),
                    status_class,
                    status_text
                ));
            }
            Html(html)
        }
        _ => Html(
            r##"<tr class="empty-row">
                <td colspan="6" class="empty-state">
                    <div class="empty-icon">💰</div>
                    <p>No price lists yet</p>
                    <p class="empty-hint">Create price lists for different customer segments</p>
                </td>
            </tr>"##.to_string(),
        ),
    }
}

async fn handle_total_products(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        products::table
            .filter(products::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_total_services(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        services::table
            .filter(services::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_total_pricelists(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        price_lists::table
            .filter(price_lists::bot_id.eq(bot_id))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_active_products(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        products::table
            .filter(products::bot_id.eq(bot_id))
            .filter(products::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    Html(format!("{}", result.unwrap_or(0)))
}

async fn handle_products_search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = query.q.clone().unwrap_or_default();
    if q.is_empty() {
        return Html(String::new());
    }

    let pool = state.conn.clone();
    let search_term = format!("%{}%", q);

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let (bot_id, _) = get_default_bot(&mut conn);

        products::table
            .filter(products::bot_id.eq(bot_id))
            .filter(
                products::name.ilike(&search_term)
                    .or(products::sku.ilike(&search_term))
                    .or(products::description.ilike(&search_term))
            )
            .order(products::name.asc())
            .limit(20)
            .select((
                products::id,
                products::sku,
                products::name,
                products::category,
                products::price,
                products::currency,
            ))
            .load::<(Uuid, Option<String>, String, Option<String>, BigDecimal, String)>(&mut conn)
            .ok()
    })
    .await
    .ok()
    .flatten();

    match result {
        Some(items) if !items.is_empty() => {
            let mut html = String::new();
            for (id, sku, name, category, price, currency) in items {
                let sku_str = sku.unwrap_or_else(|| "-".to_string());
                let cat_str = category.unwrap_or_else(|| "Uncategorized".to_string());
                let price_str = format_currency(bd_to_f64(&price), &currency);

                html.push_str(&format!(
                    r##"<div class="search-result-item" hx-get="/api/products/{id}" hx-target="#product-detail">
                        <span class="result-name">{}</span>
                        <span class="result-sku">{}</span>
                        <span class="result-category">{}</span>
                        <span class="result-price">{}</span>
                    </div>"##,
                    html_escape(&name),
                    html_escape(&sku_str),
                    html_escape(&cat_str),
                    price_str
                ));
            }
            Html(format!(r##"<div class="search-results">{html}</div>"##))
        }
        _ => Html(format!(
            r##"<div class="search-results-empty">
                <p>No results for "{}"</p>
            </div>"##,
            html_escape(&q)
        )),
    }
}
