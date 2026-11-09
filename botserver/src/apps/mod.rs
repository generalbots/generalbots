//! Application catalog and registration.
//!
//! `register` mounts UI-fragment routers and the app catalog endpoint that
//! drives every launcher (start menu, apps menu) in the frontend.

pub mod commands;
pub mod integration_catalog;
pub mod registry;

#[cfg(feature = "uifragments")]
pub use botuifragments::*;

use axum::routing::{get, post, Router};
use axum::Json;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// Core surfaces that are always present regardless of feature flags or the
/// `.product` files. Everything else is classified by the product profile as
/// always (`apps`), preview (`preview_apps`) or hidden (`hidden_apps`); #1811
/// removed the hardcoded always-visible list that used to override it.
const CORE_APPS: &[&str] = &["about", "auth", "settings"];

/// In-memory app-install counters for the App Store (#1156). The frontend
/// overlays these with per-user localStorage state; this endpoint provides a
/// server-side popularity ranking that survives navigation.
#[derive(Default)]
struct AppInstallStats {
    counts: Mutex<HashMap<String, u64>>,
}

static INSTALLS: OnceLock<Arc<AppInstallStats>> = OnceLock::new();

#[derive(serde::Deserialize)]
pub struct InstallBody {
    app_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct AgentPlanBody {
    goal: Option<String>,
    app: Option<String>,
}

mod agent_plan_handler;
mod bot_launcher_apps;
mod catalog_handler;
mod install_handler;
mod installs;

pub use installs::*;
// Explicit import: the `botuifragments::*` glob above also exports a
// `register`, so the app-level one must shadow it by name.
pub use agent_plan_handler::*;
use bot_launcher_apps::*;
pub use catalog_handler::*;
pub use install_handler::*;
pub use installs::register;
