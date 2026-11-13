use super::*;

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
pub(super) fn build_plan(goal: &str, app_hint: Option<&str>) -> Vec<serde_json::Value> {
    let g = goal.to_lowercase();

    // Explicit app hint wins: single step opening that app.
    if let Some(hint) = app_hint {
        if let Some(app) = registry::all_apps().into_iter().find(|a| a.id == hint) {
            return vec![step(
                &app.id,
                &app.title,
                &format!("Open {}", app.title),
                None,
            )];
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
            steps.push(step(app_id, label, label, *params));
        }
    }

    if steps.is_empty() {
        // Default: route to the Concierge so the user can refine the goal.
        steps.push(step(
            "concierge",
            "Concierge",
            "Refine goal in Concierge",
            None,
        ));
    }

    // Deduplicate by app id, keeping first occurrence order.
    let mut seen: Vec<String> = Vec::new();
    steps.retain(|s| {
        let id = s
            .get("app")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if seen.contains(&id) {
            false
        } else {
            seen.push(id);
            true
        }
    });

    steps
}
pub(super) fn step(
    app: &str,
    title: &str,
    action: &str,
    params: Option<&str>,
) -> serde_json::Value {
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
pub(super) fn is_app_compiled(id: &str) -> bool {
    match id {
        // Core surfaces are compiled unconditionally.
        "about" | "admin" | "settings" => true,
        "analytics" => cfg!(feature = "analytics"),
        "attendant" => cfg!(feature = "attendant"),
        "automations" => cfg!(feature = "automations"),
        "banking" => cfg!(feature = "banking"),
        "billing" => cfg!(feature = "billing"),
        "browser" => cfg!(feature = "browser"),
        "calendar" => cfg!(feature = "calendar"),
        "campaigns" | "lists" | "marketing" => cfg!(feature = "marketing"),
        "canvas" => cfg!(feature = "canvas"),
        "chat" => cfg!(feature = "chat"),
        "compliance" => cfg!(feature = "compliance"),
        "crm" => cfg!(any(feature = "people", feature = "contacts")),
        "database" => cfg!(feature = "database"),
        "designer" => cfg!(feature = "designer"),
        "desktop" => cfg!(feature = "desktop"),
        "docs" => cfg!(feature = "docs"),
        "drive" => cfg!(feature = "drive"),
        "editor" => cfg!(feature = "editor"),
        "fraud" => cfg!(feature = "fraud"),
        "goals" => cfg!(feature = "goals"),
        "handoff" => cfg!(feature = "handoff"),
        "hr" => cfg!(feature = "hr"),
        "integrations" => cfg!(feature = "integrations"),
        "kyc" => cfg!(feature = "kyc"),
        "learn" => cfg!(feature = "learn"),
        "mail" => cfg!(feature = "mail"),
        "meet" => cfg!(feature = "meet"),
        "memory" => cfg!(feature = "memory-os"),
        "minutes" => cfg!(feature = "minutes"),
        "monitoring" => cfg!(feature = "monitoring"),
        "o365" => cfg!(feature = "m365"),
        "paper" => cfg!(feature = "paper"),
        "people" => cfg!(feature = "people"),
        "plan" => cfg!(feature = "plan"),
        "player" => cfg!(feature = "player"),
        "pos" => cfg!(feature = "pos"),
        "products" => cfg!(feature = "products"),
        "project" => cfg!(feature = "project"),
        "research" => cfg!(feature = "research"),
        "retail" => cfg!(feature = "retail"),
        "sales" => cfg!(feature = "sales"),
        "sheet" => cfg!(feature = "sheet"),
        "slides" => cfg!(feature = "slides"),
        "social" => cfg!(feature = "social"),
        "sources" => cfg!(feature = "sources"),
        "tasks" => cfg!(feature = "tasks"),
        "tax" => cfg!(feature = "tax"),
        "templates" => cfg!(feature = "templates"),
        // #1841 — a terminal without the agent VM has no workspace to attach
        // to, so the app is not compiled for launcher purposes unless both
        // halves are present.
        "terminal" => cfg!(feature = "terminal") && cfg!(feature = "agent-vm"),
        "tickets" | "itsm" => cfg!(feature = "tickets"),
        "timeclock" => cfg!(feature = "timeclock"),
        "vibe" => cfg!(feature = "vibe"),
        "video" => cfg!(feature = "video"),
        "vision" => cfg!(feature = "vision"),
        "workspace" => cfg!(feature = "workspaces"),
        // Apps with no dedicated botserver feature. They have UI and routes in
        // every build and were previously hidden because the map answered
        // `false` for anything it did not recognise (#1381).
        "calculator" | "clock" | "clock-widget" | "concierge" | "jukebox" | "notes" | "photos"
        | "plugins" | "recycle" | "store" | "timer" | "weather" | "notepad" | "preview" => true,
        _ => false,
    }
}
