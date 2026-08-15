use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    Form, Router,
};
use diesel::RunQueryDsl;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn pool_conn(
    state: &Arc<AppState>,
) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, (StatusCode, String)>
{
    state
        .conn
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))
}

/// GET /api/bots/list — all bots as JSON for the "New Bot" parent dropdown.
pub async fn handle_bots_list(
    State(state): State<Arc<AppState>>,
) -> axum::response::Response {
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let rows: Vec<BotRow> = diesel::sql_query(
        "SELECT id, name FROM bots ORDER BY name ASC LIMIT 100",
    )
    .load::<BotRow>(&mut conn)
    .unwrap_or_default();

    let bots: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|b| serde_json::json!({ "id": b.id, "name": b.name }))
        .collect();

    Json(serde_json::json!(bots)).into_response()
}

/// GET /api/bots/tree — HTML bot hierarchy (roots + sub-bots via parent_bot_id).
pub async fn handle_bots_tree(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(_) => return Html(String::new()),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        parent_bot_id: Option<Uuid>,
    }

    let rows: Vec<BotRow> = diesel::sql_query(
        "SELECT id, name, description, parent_bot_id FROM bots ORDER BY name ASC LIMIT 200",
    )
    .load::<BotRow>(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(
            r##"<div class="empty-state"><p>No bots found. Create your first bot to get started.</p></div>"##
                .to_string(),
        );
    }

    let roots: Vec<&BotRow> = rows
        .iter()
        .filter(|b| b.parent_bot_id.is_none())
        .collect();

    fn render_bot(
        bot: &BotRow,
        rows: &[BotRow],
        depth: usize,
    ) -> String {
        let children: Vec<&BotRow> = rows
            .iter()
            .filter(|b| b.parent_bot_id == Some(bot.id))
            .collect();

        let desc = bot
            .description
            .as_deref()
            .map(|d| format!("<span class=\"bot-desc\">{}</span>", html_escape(d)))
            .unwrap_or_default();

        let toggle = if children.is_empty() {
            String::new()
        } else {
            format!(
                r##"<span class="bot-toggle" data-bot-id="{id}" onclick="toggleBotChildren('{id}')">▸</span>"##,
                id = bot.id
            )
        };

        let children_html: String = children
            .iter()
            .map(|c| render_bot(c, rows, depth + 1))
            .collect();

        let children_block = if children.is_empty() {
            String::new()
        } else {
            format!(
                r##"<div class="bot-children" id="bot-children-{id}">{children}</div>"##,
                id = bot.id,
                children = children_html
            )
        };

        format!(
            r##"<div class="bot-tree-item" data-bot-id="{id}" onclick="selectBot('{id}', '{name}', {is_root})">
    {toggle}
    <span class="bot-tree-icon">🤖</span>
    <span class="bot-tree-label">{name}</span>
    {desc}
    {children_block}
</div>"##,
            id = bot.id,
            name = html_escape(&bot.name),
            is_root = if depth == 0 { "true" } else { "false" },
        )
    }

    let mut tree = String::new();
    for root in roots {
        tree.push_str(&render_bot(root, &rows, 0));
    }

    Html(format!(
        r##"<div class="bot-tree-view">{tree}</div>"##
    ))
}

/// GET /api/bots/{bot_id}/config — bot settings as JSON for the config form.
pub async fn handle_bot_config_get(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
) -> impl IntoResponse {
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        llm_provider: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        enabled_tabs_json: Option<String>,
    }

    let bot: Option<BotRow> = diesel::sql_query(
        "SELECT name, description, llm_provider, enabled_tabs_json FROM bots WHERE id = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result::<BotRow>(&mut conn)
    .ok();

    let Some(bot) = bot else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Bot not found" })),
        )
            .into_response();
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct CfgRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_key: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_value: String,
    }

    let cfgs: Vec<CfgRow> = diesel::sql_query(
        "SELECT config_key, config_value FROM bot_configuration WHERE bot_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .load::<CfgRow>(&mut conn)
    .unwrap_or_default();

    let mut config = serde_json::Map::new();
    for c in cfgs {
        config.insert(c.config_key, serde_json::Value::String(c.config_value));
    }

    let enabled_tabs: serde_json::Value = bot
        .enabled_tabs_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| serde_json::json!(["chat"]));

    Json(serde_json::json!({
        "id": bot_id,
        "name": bot.name,
        "description": bot.description,
        "llm_provider": bot.llm_provider,
        "llm_model": config.get("llm-model").cloned().unwrap_or(serde_json::Value::Null),
        "llm_url": config.get("llm-url").cloned().unwrap_or(serde_json::Value::Null),
        "llm_temperature": config.get("llm-temperature").cloned().unwrap_or(serde_json::Value::from(0.7)),
        "enabled_tabs": enabled_tabs,
    }))
    .into_response()
}

/// PUT /api/bots/config — persist bot name/description/tabs/LLM settings.
pub async fn handle_bot_config_put(
    State(state): State<Arc<AppState>>,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    let Some(bot_id_str) = form.get("bot_id") else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "bot_id is required" })),
        )
            .into_response();
    };
    let Ok(bot_id) = Uuid::parse_str(bot_id_str) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invalid bot_id" })),
        )
            .into_response();
    };

    let name = form.get("name").cloned().unwrap_or_default();
    let description = form.get("description").cloned();
    let llm_provider = form.get("llm_provider").cloned().unwrap_or_else(|| "openai".to_string());
    let llm_model = form.get("llm_model").cloned();
    let llm_url = form.get("llm_url").cloned();
    let llm_temperature = form.get("llm_temperature").cloned();

    let tabs: Vec<String> = form
        .iter()
        .filter(|(k, _)| k.ends_with("[]") || *k == "tabs")
        .map(|(_, v)| v.clone())
        .collect();
    let tabs_json = serde_json::to_string(&tabs).unwrap_or_else(|_| "[\"chat\"]".to_string());

    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Bot name is required" })),
        )
            .into_response();
    }

    let result = diesel::sql_query(
        "UPDATE bots SET name = $2, description = $3, llm_provider = $4, \
                enabled_tabs_json = $5, updated_at = NOW() WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(description)
    .bind::<diesel::sql_types::Text, _>(&llm_provider)
    .bind::<diesel::sql_types::Text, _>(&tabs_json)
    .execute(&mut conn);

    if let Err(e) = result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Update failed: {e}") })),
        )
            .into_response();
    }

    // Persist LLM settings in bot_configuration (upsert per key). The table
    // requires a NOT NULL branch_id (migration 6.5.23) and has no config_type
    // column anymore — the global fallback scope is the nil branch.
    let branch_id = uuid::Uuid::nil();
    let mut llm_settings: Vec<(&str, Option<String>)> = vec![
        ("llm-model", llm_model),
        ("llm-url", llm_url),
        ("llm-temperature", llm_temperature),
    ];
    llm_settings.retain(|(_, v)| v.is_some());
    for (key, value) in llm_settings {
        let _ = diesel::sql_query(
            "INSERT INTO bot_configuration (id, bot_id, branch_id, config_key, config_value, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) \
             ON CONFLICT (branch_id, bot_id, config_key) DO UPDATE SET config_value = EXCLUDED.config_value, updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Text, _>(key)
        .bind::<diesel::sql_types::Text, _>(value.unwrap_or_default())
        .execute(&mut conn);
    }

    Json(serde_json::json!({ "success": true })).into_response()
}

/// POST /api/bots — create a new bot from the modal form.
pub async fn handle_bot_create(
    State(state): State<Arc<AppState>>,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    let name = form.get("name").cloned().unwrap_or_default();
    if name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Bot name is required" })),
        )
            .into_response();
    }

    let description = form.get("description").cloned();
    let parent_bot_id = form
        .get("parent_bot_id")
        .and_then(|v| if v.is_empty() { None } else { Uuid::parse_str(v).ok() });

    let bot_id = Uuid::new_v4();
    let branch_id = uuid::Uuid::nil();
    let result = diesel::sql_query(
        "INSERT INTO bots (id, name, description, parent_bot_id, branch_id, llm_provider, llm_config, \
                context_provider, context_config, enabled_tabs_json, is_active, is_public, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, 'openai', '{}', 'openai', '{}', '[\"chat\"]', true, false, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(description)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(parent_bot_id)
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .execute(&mut conn);

    if let Err(e) = result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Create failed: {e}") })),
        )
            .into_response();
    }

    Json(serde_json::json!({ "success": true, "id": bot_id })).into_response()
}

pub fn configure_bot_tree_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/bots/list", axum::routing::get(handle_bots_list))
        .route("/api/bots/tree", axum::routing::get(handle_bots_tree))
        .route(
            "/api/bots/:bot_id/config",
            axum::routing::get(handle_bot_config_get),
        )
        .route(
            "/api/bots/config",
            axum::routing::put(handle_bot_config_put),
        )
        .route("/api/bots", axum::routing::post(handle_bot_create))
}
