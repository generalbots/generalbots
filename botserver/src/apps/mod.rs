//! Application catalog and registration.
//!
//! `register` mounts UI-fragment routers and the app catalog endpoint that
//! drives every launcher (start menu, apps menu) in the frontend.

pub mod commands;
pub mod integration_catalog;
pub mod registry;

pub use botuifragments::*;

use axum::routing::{get, post, Router};
use axum::Json;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Core apps that are always present regardless of feature flags.
const CORE_APPS: &[&str] = &["settings", "auth", "admin"];

/// In-memory app-install counters for the App Store (#1156). The frontend
/// overlays these with per-user localStorage state; this endpoint provides a
/// server-side popularity ranking that survives navigation.
#[derive(Default)]
struct AppInstallStats {
    counts: Mutex<HashMap<String, u64>>,
}

static INSTALLS: OnceLock<Arc<AppInstallStats>> = OnceLock::new();

fn installs() -> Arc<AppInstallStats> {
    INSTALLS
        .get_or_init(|| Arc::new(AppInstallStats::default()))
        .clone()
}

pub fn register<S: Clone + Send + Sync + 'static>(r: Router<S>) -> Router<S> {
    let router = r
        .merge(botuifragments::register(Router::new()))
        .route("/api/apps/catalog", get(catalog_handler))
        .route("/api/apps/install", post(install_handler))
        .route("/api/apps/install/stats", get(install_stats_handler))
        .route("/api/agent/plan", post(agent_plan_handler));
    integration_catalog::register(router)
}

#[derive(serde::Deserialize)]
struct InstallBody {
    app_id: Option<String>,
}

/// Records an app-store install and returns the resolved app metadata.
pub async fn install_handler(Json(body): Json<InstallBody>) -> Json<serde_json::Value> {
    let app_id = body.app_id.unwrap_or_default();
    let found = registry::all_apps().into_iter().find(|a| a.id == app_id);
    match found {
        Some(app) => {
            let stats = installs();
            let mut counts = match stats.counts.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            let count = counts.entry(app.id.clone()).or_insert(0);
            *count += 1;
            Json(json!({
                "ok": true,
                "app_id": app.id,
                "title": app.title,
                "category": app.category,
                "url": app.url,
                "installs": *count,
            }))
        }
        None => Json(json!({ "ok": false, "error": "unknown_app", "app_id": app_id })),
    }
}

/// Returns the server-side install popularity ranking.
pub async fn install_stats_handler() -> Json<serde_json::Value> {
    let stats = installs();
    let counts = match stats.counts.lock() {
        Ok(g) => g.clone(),
        Err(p) => p.into_inner().clone(),
    };
    let total: u64 = counts.values().sum();
    Json(json!({ "installs": counts, "total": total }))
}

#[derive(serde::Deserialize)]
struct AgentPlanBody {
    goal: Option<String>,
    app: Option<String>,
}

/// Concierge planner (#1157): turns a natural-language goal into an ordered
/// list of app actions the frontend `agent-executor.js` can run. Pure
/// heuristic (no LLM round-trip) so it is instant and offline; the executor
/// may enrich steps with the LLM later.
pub async fn agent_plan_handler(Json(body): Json<AgentPlanBody>) -> Json<serde_json::Value> {
    let goal = body.goal.unwrap_or_default();
    let steps = build_plan(&goal, body.app.as_deref());
    Json(json!({
        "goal": goal,
        "steps": steps,
    }))
}

/// Maps goal keywords to concrete app actions. Each step carries the app id,
/// a human action label, and optional deep-link params for the launcher.
fn build_plan(goal: &str, app_hint: Option<&str>) -> Vec<serde_json::Value> {
    let g = goal.to_lowercase();

    // Explicit app hint wins: single step opening that app.
    if let Some(hint) = app_hint {
        if let Some(app) = registry::all_apps().into_iter().find(|a| a.id == hint) {
            return vec![step(&app.id, &app.title, &format!("Open {}", app.title), None)];
        }
    }

    let mut steps: Vec<serde_json::Value> = Vec::new();

    let rules: &[(&str, &str, &str, Option<&str>)] = &[
        ("email", "mail", "Open Mail", None),
        ("calendar", "calendar", "Open Calendar", None),
        ("schedule", "calendar", "Open Calendar", Some("view=month")),
        ("meeting", "meet", "Open Meet", None),
        ("task", "tasks", "Open Tasks", None),
        ("todo", "tasks", "Open Tasks", None),
        ("crm", "crm", "Open CRM", None),
        ("customer", "crm", "Open CRM", None),
        ("client", "crm", "Open CRM", None),
        ("lead", "crm", "Open CRM", None),
        ("doc", "docs", "Open Docs", None),
        ("write", "docs", "Open Docs", None),
        ("sheet", "sheet", "Open Sheets", None),
        ("spreadsheet", "sheet", "Open Sheets", None),
        ("slide", "slides", "Open Slides", None),
        ("presentation", "slides", "Open Slides", None),
        ("search", "research", "Open Research", None),
        ("research", "research", "Open Research", None),
        ("find", "research", "Open Research", None),
        ("chat", "chat", "Open Chat", None),
        ("message", "chat", "Open Chat", None),
        ("drive", "drive", "Open Explorer", None),
        ("file", "drive", "Open Explorer", None),
        ("upload", "drive", "Open Explorer", None),
        ("product", "products", "Open Products", None),
        ("price", "products", "Open Products", None),
        ("invoice", "billing", "Open Billing", None),
        ("payment", "billing", "Open Billing", None),
        ("billing", "billing", "Open Billing", None),
        ("ticket", "tickets", "Open Tickets", None),
        ("support", "tickets", "Open Tickets", None),
        ("video", "meet", "Open Meet", None),
        ("call", "meet", "Open Meet", None),
        ("note", "notes", "Open Sticky Notes", None),
        ("memo", "notes", "Open Sticky Notes", None),
        ("timer", "timer", "Open Timer", None),
        ("pomodoro", "timer", "Open Timer", None),
        ("focus", "timer", "Open Timer", None),
        ("weather", "weather", "Open Weather", None),
        ("photo", "photos", "Open Photos", None),
        ("image", "photos", "Open Photos", None),
        ("trash", "recycle", "Open Recycle Bin", None),
        ("recycle", "recycle", "Open Recycle Bin", None),
        ("store", "store", "Open App Store", None),
        ("install", "store", "Open App Store", None),
        ("app", "store", "Open App Store", None),
        ("concierge", "concierge", "Open Concierge", None),
        ("assistant", "concierge", "Open Concierge", None),
    ];

    for (keyword, app_id, label, params) in rules {
        if g.contains(keyword) {
            steps.push(step(app_id, label, label, params));
        }
    }

    if steps.is_empty() {
        // Default: route to the Concierge so the user can refine the goal.
        steps.push(step("concierge", "Concierge", "Refine goal in Concierge", None));
    }

    // Deduplicate by app id, keeping first occurrence order.
    let mut seen: Vec<String> = Vec::new();
    steps.retain(|s| {
        let id = s.get("app").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        if seen.contains(&id) {
            false
        } else {
            seen.push(id);
            true
        }
    });

    steps
}

fn step(app: &str, title: &str, action: &str, params: Option<&str>) -> serde_json::Value {
    json!({
        "app": app,
        "title": title,
        "action": action,
        "params": params.map(|p| p.to_string()).unwrap_or_default(),
    })
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
        "jukebox" => true,
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
        "store" | "concierge" | "notes" | "photos" | "timer" | "weather" | "recycle" => true,
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
                "kind": a.kind,
                "widget": a.widget,
                "launcher_default": a.launcher_default,
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
