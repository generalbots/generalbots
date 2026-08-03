//! Application catalog and registration.
//!
//! `register` mounts UI-fragment routers and the app catalog endpoint that
//! drives every launcher (start menu, apps menu) in the frontend.

pub mod registry;

pub use botuifragments::*;

use axum::{routing::get, Json, Router};
use serde_json::json;

/// Core apps that are always present regardless of feature flags.
const CORE_APPS: &[&str] = &["settings", "auth", "admin"];

pub fn register<S: Clone + Send + Sync + 'static>(r: Router<S>) -> Router<S> {
    r.merge(botuifragments::register(Router::new()))
        .route("/api/apps/catalog", get(catalog_handler))
}

/// Returns the complete application catalog with per-app `enabled` and
/// `compiled` flags so the frontend can render exactly what this build offers.
///
/// `enabled` is driven strictly by the `.product` file's `apps=` list (or the
/// default all-apps set when no `apps=` line is present). An app not listed
/// there is never surfaced.
pub async fn catalog_handler() -> Json<serde_json::Value> {
    let apps = registry::all_apps();

    let enabled: std::collections::HashSet<String> = botcore::product::PRODUCT_CONFIG
        .read()
        .map(|c| c.get_enabled_apps().into_iter().collect())
        .unwrap_or_default();

    let items: Vec<serde_json::Value> = apps
        .iter()
        .map(|a| {
            let id = a.id.as_str();
            let compiled = botcore::features::is_feature_compiled(id) || CORE_APPS.contains(&id);
            let is_enabled = enabled.contains(id) || CORE_APPS.contains(&id);
            json!({
                "id": a.id,
                "title": a.title,
                "category": a.category,
                "color": a.color,
                "url": a.url,
                "description": a.description,
                "keywords": a.keywords,
                "icon": a.icon,
                "enabled": is_enabled,
                "compiled": compiled,
            })
        })
        .collect();

    Json(json!({
        "apps": items,
        "categories": registry::CATEGORIES
            .iter()
            .map(|(k, l)| json!({"id": k, "label": l}))
            .collect::<Vec<_>>(),
        "labels": registry::category_labels(),
    }))
}
