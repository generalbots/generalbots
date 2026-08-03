use axum::{extract::State, response::Html};
use chrono::Utc;
use std::sync::Arc;

use botcore::shared::state::AppState;

use super::get_conn;

pub async fn dashboard_health(State(state): State<Arc<AppState>>) -> Html<String> {
    let db_ok = get_conn(&state).is_some();
    let cache_ok = state
        .cache
        .as_ref()
        .map(|c| {
            c.get_connection_with_timeout(std::time::Duration::from_millis(750))
                .and_then(|mut conn| redis::cmd("PING").query::<String>(&mut conn))
                .map(|reply| reply.to_uppercase() == "PONG")
                .unwrap_or(false)
        })
        .unwrap_or(false);
    let drive_ok = state.drive.is_some();
    let llm_url = std::env::var("LLM_URL")
        .or_else(|_| std::env::var("OLLAMA_HOST"))
        .ok();
    let llm_ok = llm_url.is_some();

    let mut items = String::new();
    items.push_str(&health_item("API Server", db_ok, "Operational", "HTTP API responding"));
    items.push_str(&health_item(
        "Database",
        db_ok,
        if db_ok { "Operational" } else { "Unreachable" },
        if db_ok { "Connection pool healthy" } else { "Connection failed" },
    ));
    items.push_str(&health_item(
        "Cache",
        cache_ok,
        if cache_ok { "Operational" } else { "Unreachable" },
        if cache_ok { "PING responded" } else { "No PONG" },
    ));
    items.push_str(&health_item(
        "Storage",
        drive_ok,
        if drive_ok { "Operational" } else { "Not configured" },
        if drive_ok { "Drive repository active" } else { "No drive client" },
    ));
    items.push_str(&health_item(
        "LLM Service",
        llm_ok,
        if llm_ok { "Configured" } else { "Not configured" },
        if llm_ok { "Provider URL present" } else { "LLM_URL unset" },
    ));

    Html(items)
}

fn health_item(name: &str, ok: bool, status: &str, detail: &str) -> String {
    let (indicator, cls) = if ok { ("healthy", "") } else { ("unhealthy", " warning") };
    format!(
        r##"<div class="health-item{cls}">
    <div class="health-indicator {indicator}"></div>
    <div class="health-info">
        <span class="health-name">{name}</span>
        <span class="health-status">{status}</span>
    </div>
    <div class="health-metrics">
        <span class="metric">{detail}</span>
    </div>
</div>"##
    )
}

pub async fn admin_health(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    let db_ok = get_conn(&state).is_some();
    let cache_ok = state.cache.as_ref().map(|_| true).unwrap_or(false);
    axum::Json(serde_json::json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "services": {
            "api": { "status": if db_ok { "up" } else { "down" } },
            "database": { "status": if db_ok { "up" } else { "down" } },
            "cache": { "status": if cache_ok { "up" } else { "down" } },
        },
        "timestamp": Utc::now().to_rfc3339(),
    }))
}
