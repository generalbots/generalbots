//! #1444 — Skills Marketplace split from `skills.rs` (450-line rule).
//!
//! #1169 — Skills Marketplace: a curated catalog of installable skill
//! packages. `GET /api/vibe/skills/marketplace` lists catalog entries with
//! an `installed` flag; `POST /api/vibe/skills/marketplace/install` pulls a
//! catalog skill into the local SkillStore (register/install).

use crate::skills::{SkillResponse, SkillStore};use axum::{Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

const MARKETPLACE: &[(&str, &str, &str, &[&str], &str)] = &[
    (
        "sql-expert",
        "SQL expert",
        "Writes precise SQL for the project's database schema: SELECT/INSERT/UPDATE with \
         correct quoting, joins, and indexing advice.",
        &["sql", "query", "database"],
        "1.0.0",
    ),
    (
        "data-viz",
        "Data visualization",
        "Chooses the right chart type, computes aggregations, and produces chart-ready \
         JSON for the suite's chart components.",
        &["chart", "graph", "visualization", "dashboard"],
        "1.0.0",
    ),
    (
        "email-writer",
        "Email writer",
        "Drafts clear, professional emails (plain or HTML) with subject lines, tone \
         matching, and call-to-action.",
        &["email", "mail", "draft"],
        "1.0.0",
    ),
    (
        "code-reviewer",
        "Code reviewer",
        "Reviews Rust/JS code for panics, unsafe patterns, and the project's security \
         directives; suggests minimal fixes.",
        &["review", "code review", "security"],
        "1.0.0",
    ),
    (
        "meeting-notes",
        "Meeting notes",
        "Turns a meeting transcript into structured minutes: decisions, action items, \
         owners, and next steps.",
        &["meeting", "minutes", "notes", "summary"],
        "1.0.0",
    ),
    (
        "incident-runbook",
        "Incident runbook",
        "Guides triage of botserver outages: log checks, health probes, rollback \
         decision tree, and postmortem template.",
        &["incident", "outage", "runbook", "triage"],
        "1.0.0",
    ),
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketplaceItem {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub version: String,
    pub installed: bool,
}

pub async fn list_marketplace(Extension(store): Extension<Arc<SkillStore>>) -> Json<Value> {
    let installed = store.list().await;
    let items: Vec<MarketplaceItem> = MARKETPLACE
        .iter()
        .map(|(name, description, _, triggers, version)| MarketplaceItem {
            name: name.to_string(),
            description: description.to_string(),
            triggers: triggers.iter().map(|t| t.to_string()).collect(),
            version: version.to_string(),
            installed: installed.iter().any(|s| &s.name == name),
        })
        .collect();
    Json(json!({ "success": true, "items": items }))
}

#[derive(Debug, Deserialize)]
pub struct InstallRequest {
    pub name: String,
}

pub async fn install_marketplace(
    Extension(store): Extension<Arc<SkillStore>>,
    Json(req): Json<InstallRequest>,
) -> Json<SkillResponse> {
    for (name, description, content, triggers, _version) in MARKETPLACE {
        if *name == req.name {
            let skill = store
                .register(
                    name.to_string(),
                    description.to_string(),
                    content.to_string(),
                    triggers.iter().map(|t| t.to_string()).collect(),
                )
                .await;
            return Json(SkillResponse {
                success: true,
                skill: Some(skill),
                error: None,
            });
        }
    }
    Json(SkillResponse {
        success: false,
        skill: None,
        error: Some(format!("skill '{}' not in marketplace", req.name)),
    })
}
