//! Application catalog and registration.
//!
//! `register` mounts UI-fragment routers and the app catalog endpoint that
//! drives every launcher (start menu, apps menu) in the frontend.

pub mod commands;
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
/// Returns whether the app's backend routes are compiled into this build.
///
/// The app registry ids map to botserver cargo features (marketing provides
/// campaigns/lists, m365 provides o365, and so on). botcore's feature list
/// only covers library features, so this function checks botserver's own
/// features to avoid hiding apps whose routes are actually registered.
fn is_app_compiled(id: &str) -> bool {
    match id {
        "chat" => cfg!(feature = "chat"),
        "vibe" => cfg!(feature = "vibe"),
        "research" => cfg!(feature = "research"),
        "video" => cfg!(feature = "video"),
        "vision" => cfg!(feature = "vision"),
        "learn" => cfg!(feature = "learn"),
        "mail" => cfg!(feature = "mail"),
        "calendar" => cfg!(feature = "calendar"),
        "meet" => cfg!(feature = "meet"),
        "docs" => cfg!(feature = "docs"),
        "sheet" => cfg!(feature = "sheet"),
        "slides" => cfg!(feature = "slides"),
        "paper" => cfg!(feature = "paper"),
        "tasks" => cfg!(feature = "tasks"),
        "plan" => cfg!(feature = "plan"),
        "goals" => cfg!(feature = "goals"),
        "minutes" => cfg!(feature = "minutes"),
        "timeclock" => cfg!(feature = "timeclock"),
        "o365" => cfg!(feature = "m365"),
        "templates" => cfg!(feature = "templates"),
        "designer" => cfg!(feature = "designer"),
        "crm" => cfg!(any(feature = "people", feature = "contacts")),
        "people" => cfg!(feature = "people"),
        "campaigns" | "lists" | "marketing" => cfg!(feature = "marketing"),
        "billing" => cfg!(feature = "billing"),
        "products" => cfg!(feature = "billing"),
        "tickets" | "itsm" => cfg!(feature = "tickets"),
        "hr" => cfg!(feature = "hr"),
        "banking" => cfg!(feature = "banking"),
        "sales" => cfg!(feature = "sales"),
        "pos" => cfg!(feature = "pos"),
        "retail" => cfg!(feature = "retail"),
        "handoff" => cfg!(feature = "handoff"),
        "kyc" => cfg!(feature = "kyc"),
        "fraud" => cfg!(feature = "fraud"),
        "compliance" => cfg!(feature = "compliance"),
        "tax" => cfg!(feature = "tax"),
        "social" => cfg!(feature = "social"),
        "attendant" => cfg!(feature = "attendant"),
        "editor" | "bas-editor" => true,
        "database" => cfg!(feature = "database"),
        "browser" => cfg!(feature = "browser"),
        "versions" => true,
        "integrations" => cfg!(feature = "integrations"),
        "sources" => cfg!(feature = "sources"),
        "tools" => cfg!(feature = "automation"),
        "terminal" => cfg!(feature = "terminal"),
        "canvas" => cfg!(feature = "canvas"),
        "workspace" => cfg!(feature = "workspaces"),
        "project" => cfg!(feature = "project"),
        "analytics" => cfg!(feature = "analytics"),
        "monitoring" => cfg!(feature = "monitoring"),
        "admin" => true,
        "settings" => true,
        "drive" => cfg!(feature = "drive"),
        "vdi" => true,
        "biometry" => true,
        "player" => cfg!(feature = "player"),
        _ => false,
    }
}

pub async fn catalog_handler() -> Json<serde_json::Value> {
    let apps = registry::all_apps();

    let enabled: std::collections::HashSet<String> = botcore::product::PRODUCT_CONFIG
        .read()
        .map(|c| c.get_enabled_apps().into_iter().collect())
        .unwrap_or_default();

    let derived: Vec<crate::core::bot::commands_derived::DerivedCommand> =
        crate::core::bot::commands_derived::derived_commands();

    let items: Vec<serde_json::Value> = apps
        .iter()
        .map(|a| {
            let id = a.id.as_str();
            let compiled = is_app_compiled(id) || CORE_APPS.contains(&id);
            let is_enabled = enabled.contains(id) || CORE_APPS.contains(&id);
            // Merge curated commands with the harvested on-the-fly surface.
            let app_derived: Vec<serde_json::Value> = derived
                .iter()
                .filter(|c| c.name.starts_with(&format!("{id}.")))
                .take(40)
                .map(|c| {
                    json!({
                        "name": c.name,
                        "method": c.method,
                        "path": c.path,
                        "summary": c.summary,
                        "derived": true,
                    })
                })
                .collect();
            let mut commands: Vec<serde_json::Value> = commands::commands_for_app(id)
                .iter()
                .map(|c| {
                    json!({ "name": c.name, "label": c.label, "summary": c.summary, "derived": false })
                })
                .collect();
            commands.extend(app_derived);
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
                "commands": commands,
                "deep_link_params": commands::deep_link_params_for_app(id),
            })
        })
        .collect();

    let _derived_count = derived.len();
    Json(json!({
        "apps": items,
        "categories": registry::CATEGORIES
            .iter()
            .map(|(k, l)| json!({"id": k, "label": l}))
            .collect::<Vec<_>>(),
        "labels": registry::category_labels(),
        "surface": json!({ "curated_commands": commands::all_commands().len(), "harvested_endpoints": _derived_count }),
    }))
}
