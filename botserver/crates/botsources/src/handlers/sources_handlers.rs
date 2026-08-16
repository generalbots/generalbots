use crate::renderers::{get_prompts_data, get_templates_data, html_escape};
use crate::state::{AppState, get_work_path_or_default, get_keywords_or_default, make_mcp_loader};
use crate::types::{ApiResponse, BotQuery, SearchQuery, RepositoryInfo, AppInfo};

use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse},
    Json,
};
use std::fmt::Write;
use std::sync::Arc;

pub async fn handle_list_repositories(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let repos: Vec<RepositoryInfo> = vec![RepositoryInfo {
        id: "1".to_string(),
        name: "botserver".to_string(),
        owner: "generalbots".to_string(),
        description: "General Bots server implementation".to_string(),
        url: "https://github.com/generalbots/botserver".to_string(),
        language: Some("Rust".to_string()),
        stars: 150,
        forks: 45,
        status: "connected".to_string(),
        last_sync: Some("2024-01-15T10:30:00Z".to_string()),
    }];

    let mut html = String::new();
    html.push_str("<div class=\"repos-grid\">");

    for repo in &repos {
        let status_class = if repo.status == "connected" { "connected" } else { "disconnected" };
        let status_text = if repo.status == "connected" { "Connected" } else { "Disconnected" };
        let language = repo.language.as_deref().unwrap_or("Unknown");
        let last_sync = repo.last_sync.as_deref().unwrap_or("Never");

        let _ = write!(
            html,
            r#"<div class="repo-card">
            <div class="repo-header">
            <div class="repo-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
            </svg>
            </div>
            <div class="repo-info">
            <h4 class="repo-name">{}</h4>
            <span class="repo-owner">{}</span>
            </div>
            <span class="repo-status {}">{}</span>
            </div>
            <p class="repo-description">{}</p>
            <div class="repo-meta">
            <span class="repo-meta-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            </svg>
            {}</span>
            <span class="repo-meta-item">⭐ {}</span>
            <span class="repo-meta-item">🍴 {}</span>
            <span class="repo-meta-item">Last sync: {}</span>
            </div>
            <div class="repo-actions">
            <button class="btn-browse" onclick="window.open('{}', '_blank')">Browse</button>
            </div></div>"#,
            html_escape(&repo.name),
            html_escape(&repo.owner),
            status_class, status_text,
            html_escape(&repo.description),
            language, repo.stars, repo.forks, last_sync,
            html_escape(&repo.url)
        );
    }

    if repos.is_empty() {
        html.push_str(r#"<div class="empty-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
        </svg>
        <h3>No Repositories</h3><p>Connect your GitHub repositories to get started</p></div>"#);
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_connect_repository(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(ApiResponse::success(format!("Repository {} connected", id)))
}

pub async fn handle_disconnect_repository(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(ApiResponse::success(format!(
        "Repository {} disconnected", id
    )))
}

pub async fn handle_list_apps(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let apps: Vec<AppInfo> = vec![AppInfo {
        id: "1".to_string(),
        name: "crm".to_string(),
        app_type: "htmx".to_string(),
        description: "Customer relationship management".to_string(),
        url: "/crm".to_string(),
        created_at: "2024-01-10T14:00:00Z".to_string(),
        status: "active".to_string(),
    }];

    let mut html = String::new();
    html.push_str("<div class=\"apps-grid\">");

    for app in &apps {
        let app_icon = match app.app_type.as_str() {
            "htmx" => "📱",
            "react" => "⚛️",
            "vue" => "💚",
            _ => "🔷",
        };

        let _ = write!(
            html,
            r#"<div class="app-card">
            <div class="app-header">
            <div class="app-icon">{}</div>
            <div class="app-info">
            <h4 class="app-name">{}</h4>
            <span class="app-type">{}</span>
            </div></div>
            <p class="app-description">{}</p>
            <div class="app-actions">
            <button class="btn-open" onclick="window.location.href='{}'">Open</button>
            <button class="btn-edit">Edit</button>
            </div></div>"#,
            app_icon,
            html_escape(&app.name),
            html_escape(&app.app_type),
            html_escape(&app.description),
            html_escape(&app.url)
        );
    }

    if apps.is_empty() {
        html.push_str(r#"<div class="empty-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <rect x="3" y="3" width="7" height="7"></rect>
        <rect x="14" y="3" width="7" height="7"></rect>
        <rect x="14" y="14" width="7" height="7"></rect>
        <rect x="3" y="14" width="7" height="7"></rect>
        </svg>
        <h3>No Apps</h3><p>Create your first app to get started</p></div>"#);
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_prompts(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let category = params.category.unwrap_or_else(|| "all".to_string());
    let prompts = get_prompts_data(&category);

    let mut html = String::new();
    html.push_str("<div class=\"panel-layout\">");
    html.push_str("<aside class=\"categories-sidebar\">");
    html.push_str("<h3>Categories</h3>");
    html.push_str("<div class=\"category-list\">");

    let categories = vec![
        ("all", "All Prompts", "📋"),
        ("writing", "Writing", "✍️"),
        ("coding", "Coding", "💻"),
        ("analysis", "Analysis", "📊"),
        ("creative", "Creative", "🎨"),
        ("business", "Business", "💼"),
        ("education", "Education", "📚"),
    ];

    for (id, name, icon) in &categories {
        let active = if *id == category { " active" } else { "" };
        let _ = write!(
            html,
            "<button class=\"category-item{}\" hx-get=\"/api/sources/prompts?category={}\" hx-target=\"#content-area\" hx-swap=\"innerHTML\"><span class=\"category-icon\">{}</span><span class=\"category-name\">{}</span></button>",
            active, id, icon, name
        );
    }

    html.push_str("</div></aside>");
    html.push_str("<div class=\"content-main\"><div class=\"prompts-grid\" id=\"prompts-grid\">");

    for prompt in &prompts {
        let _ = write!(
            html,
            "<div class=\"prompt-card\"><div class=\"prompt-header\"><span class=\"prompt-icon\">{}</span><h4>{}</h4></div><p class=\"prompt-description\">{}</p><div class=\"prompt-footer\"><span class=\"prompt-category\">{}</span><button class=\"btn-use\" onclick=\"usePrompt('{}')\">Use</button></div></div>",
            prompt.icon, html_escape(&prompt.title), html_escape(&prompt.description), html_escape(&prompt.category), html_escape(&prompt.id)
        );
    }

    if prompts.is_empty() {
        html.push_str("<div class=\"empty-state\"><p>No prompts found in this category</p></div>");
    }

    html.push_str("</div></div></div>");
    Html(html)
}

pub async fn handle_templates(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let templates = get_templates_data();

    let mut html = String::new();
    html.push_str("<div class=\"templates-container\">");
    html.push_str("<div class=\"templates-header\"><h3>Bot Templates</h3><p>Pre-built bot configurations ready to deploy</p></div>");
    html.push_str("<div class=\"templates-grid\">");

    for template in &templates {
        let _ = write!(
            html,
            "<div class=\"template-card\"><div class=\"template-icon\">{}</div><div class=\"template-info\"><h4>{}</h4><p>{}</p><div class=\"template-meta\"><span class=\"template-category\">{}</span></div></div><div class=\"template-actions\"><button class=\"btn-preview\">Preview</button><button class=\"btn-use-template\">Use Template</button></div></div>",
            template.icon, html_escape(&template.name), html_escape(&template.description), html_escape(&template.category)
        );
    }

    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_news(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let news_items = vec![
        ("📢", "General Bots 6.0 Released", "Major update with improved performance and new features", "2 hours ago"),
        ("🔌", "New MCP Server Integration", "Connect to external tools more easily with our new MCP support", "1 day ago"),
        ("📊", "Analytics Dashboard Update", "Real-time metrics and improved visualizations", "3 days ago"),
        ("🔒", "Security Enhancement", "Enhanced encryption and authentication options", "1 week ago"),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"news-container\">");
    html.push_str("<div class=\"news-header\"><h3>Latest News</h3><p>Updates and announcements from the General Bots team</p></div>");
    html.push_str("<div class=\"news-list\">");

    for (icon, title, description, time) in &news_items {
        let _ = write!(
            html,
            "<div class=\"news-item\"><div class=\"news-icon\">{}</div><div class=\"news-content\"><h4>{}</h4><p>{}</p><span class=\"news-time\">{}</span></div></div>",
            icon, html_escape(title), html_escape(description), time
        );
    }

    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_llm_tools(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = get_work_path_or_default(&state.get_work_path);

    let keywords = get_keywords_or_default(&state.get_keywords);
    let loader = make_mcp_loader(&state.mcp_loader, &work_path, &bot_id);
    let scan_result = loader.load();
    let mcp_tools_count: usize = scan_result.servers.iter().map(|s| s.tools.len()).sum();

    let mut html = String::new();
    html.push_str("<div class=\"tools-container\">");
    let _ = write!(
        html,
        "<div class=\"tools-header\"><h3>LLM Tools</h3><p>All tools available for Tasks and LLM invocation</p><div class=\"tools-stats\"><span class=\"stat\"><strong>{}</strong> BASIC keywords</span><span class=\"stat\"><strong>{}</strong> MCP tools</span></div></div>",
        keywords.len(), mcp_tools_count
    );

    html.push_str("<div class=\"tools-grid\">");
    for keyword in keywords.iter().take(20) {
        let _ = write!(
            html,
            "<span class=\"keyword-tag\">{}</span>",
            html_escape(keyword)
        );
    }
    if keywords.len() > 20 {
        let _ = write!(
            html,
            "<span class=\"keyword-more\">+{} more...</span>",
            keywords.len() - 20
        );
    }
    html.push_str("</div></div>");

    Html(html)
}

pub async fn handle_models(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Pull the bot's real LLM configuration instead of a hardcoded catalog.
    let config_manager = state.config_manager.clone();
    let default_bot_id = uuid::Uuid::nil();
    let configured_model = config_manager
        .get_config(&default_bot_id, "llm-model", None)
        .unwrap_or_default();
    let configured_provider = config_manager
        .get_config(&default_bot_id, "llm-provider", None)
        .unwrap_or_default();
    let llm_url = config_manager
        .get_config(&default_bot_id, "llm-url", None)
        .unwrap_or_default();

    // Known provider families the runtime can drive (see botllm::LLMProviderType).
    // `match_url` is a lowercase substring tested against the configured llm-url;
    // an empty string means the provider is never auto-detected from URL.
    let catalog: Vec<(&str, &str, &str, &str)> = vec![
        ("🧠", "OpenAI", "GPT, GPT-4o and compatible endpoints", "openai.com"),
        ("🔷", "Anthropic", "Claude 3 and later family", "anthropic.com"),
        ("🦙", "Meta Llama", "Open-weight Llama models (via OpenAI-compatible servers)", ""),
        ("⚡", "Groq", "Fast inference on open models", "groq.com"),
        ("⛰️", "Vertex / Gemini", "Google AI Studio and Vertex endpoints", "googleapis.com"),
        ("☁️", "Amazon Bedrock", "Foundation models via Bedrock", "bedrock"),
    ];

    let provider_lower = configured_provider.to_lowercase();
    let url_lower = llm_url.to_lowercase();

    let mut html = String::new();
    html.push_str("<div class=\"models-container\">");
    html.push_str("<div class=\"models-header\"><h3>AI Models</h3><p>Language models available to your bots</p></div>");

    if !configured_model.is_empty() {
        let _ = write!(
            html,
            "<div class=\"active-model-banner\"><strong>Configured model:</strong> {} <span class=\"model-status active\">Active</span></div>",
            html_escape(&configured_model)
        );
    }

    html.push_str("<div class=\"models-grid\">");

    for (icon, name, description, match_url) in &catalog {
        // The provider actually wired up in config.csv (llm-provider or the
        // llm-url host) is the live one; everything else is listed as available.
        let is_configured = (!provider_lower.is_empty() && name.to_lowercase().contains(&provider_lower))
            || (!match_url.is_empty() && !url_lower.is_empty() && url_lower.contains(match_url));
        let status_class = if is_configured { "model-active" } else { "model-available" };
        let status = if is_configured { "Active" } else { "Available" };
        let _ = write!(
            html,
            "<div class=\"model-card {}\"><div class=\"model-icon\">{}</div><div class=\"model-info\"><div class=\"model-header\"><h4>{}</h4><span class=\"model-provider\">{}</span></div><p>{}</p><div class=\"model-footer\"><span class=\"model-status\">{}</span></div></div></div>",
            status_class, icon, html_escape(name), html_escape(name), html_escape(description), status
        );
    }

    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_search(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default();

    if query.is_empty() {
        return Html("<div class=\"search-prompt\"><p>Enter a search term</p></div>".to_string());
    }

    let query_lower = query.to_lowercase();
    let prompts = get_prompts_data("all");
    let matching_prompts: Vec<_> = prompts
        .iter()
        .filter(|p| {
            p.title.to_lowercase().contains(&query_lower)
            || p.description.to_lowercase().contains(&query_lower)
        })
        .collect();

    let mut html = String::new();
    let _ = write!(html, "<div class=\"search-results\"><div class=\"search-header\"><h3>Search Results for \"{}\"</h3></div>", html_escape(&query));

    if matching_prompts.is_empty() {
        html.push_str("<div class=\"no-results\"><p>No results found</p></div>");
    } else {
        let _ = write!(
            html,
            "<div class=\"result-section\"><h4>Prompts ({})</h4><div class=\"results-grid\">",
            matching_prompts.len()
        );
        for prompt in matching_prompts {
            let _ = write!(
                html,
                "<div class=\"result-item\"><span class=\"result-icon\">{}</span><div class=\"result-info\"><strong>{}</strong><p>{}</p></div></div>",
                prompt.icon, html_escape(&prompt.title), html_escape(&prompt.description)
            );
        }
        html.push_str("</div></div>");
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_mentions_autocomplete(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default().to_lowercase();

    #[derive(serde::Serialize)]
    struct MentionItem {
        name: String,
        display: String,
        #[serde(rename = "type")]
        item_type: String,
        icon: String,
        description: String,
    }

    let mut mentions: Vec<MentionItem> = Vec::new();

    let repos = vec![
        ("botserver", "Main bot server", "repo"),
        ("botui", "User interface", "repo"),
        ("botbook", "Documentation", "repo"),
        ("botlib", "Core library", "repo"),
    ];

    for (name, desc, _) in repos {
        if query.is_empty() || name.contains(&query) {
            mentions.push(MentionItem {
                name: name.to_string(),
                display: format!("@{}", name),
                item_type: "repository".to_string(),
                icon: "📁".to_string(),
                description: desc.to_string(),
            });
        }
    }

    let apps = vec![
        ("crm", "Customer management app", "app"),
        ("dashboard", "Analytics dashboard", "app"),
    ];

    for (name, desc, _) in apps {
        if query.is_empty() || name.contains(&query) {
            mentions.push(MentionItem {
                name: name.to_string(),
                display: format!("@{}", name),
                item_type: "app".to_string(),
                icon: "📱".to_string(),
                description: desc.to_string(),
            });
        }
    }

    let bot_id = "default".to_string();
    let work_path = get_work_path_or_default(&state.get_work_path);
    let loader = make_mcp_loader(&state.mcp_loader, &work_path, &bot_id);
    let scan_result = loader.load();

    for server in scan_result.servers {
        if query.is_empty() || server.name.to_lowercase().contains(&query) {
            mentions.push(MentionItem {
                name: server.name.clone(),
                display: format!("@{}", server.name),
                item_type: "mcp".to_string(),
                icon: "🔌".to_string(),
                description: server.description,
            });
        }
    }

    mentions.truncate(10);
    Json(mentions)
}

use serde::Deserialize as SerdeDeserialize;

#[derive(SerdeDeserialize)]
pub struct SavePromptRequest {
    pub prompt_id: Option<String>,
    pub collection: Option<String>,
    pub prompt: Option<String>,
}pub async fn handle_prompts_save(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SavePromptRequest>,
) -> impl IntoResponse {
    let _ = state;
    let prompt_id = form.prompt_id.unwrap_or_default();
    let collection = form.collection.unwrap_or_else(|| "default".to_string());
    let prompt = form.prompt.unwrap_or_default();

    Json(serde_json::json!({
        "ok": true,
        "prompt_id": prompt_id,
        "collection": collection,
        "prompt": prompt,
        "message": "Prompt saved to collection",
    }))
}

#[derive(SerdeDeserialize)]
pub struct InstallSkillRequest {
    pub name: Option<String>,
    pub bot_id: Option<String>,
}

pub async fn handle_install_skill(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallSkillRequest>,
) -> impl IntoResponse {
    use diesel::prelude::*;

    let skill_id = payload.name.unwrap_or_default();
    let bot_id = payload
        .bot_id
        .unwrap_or_else(|| "default".to_string());

    if skill_id.trim().is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "ok": false, "error": "skill name required" })),
        );
    }

    let pool = state.conn.clone();
    let skill_id_c = skill_id.clone();
    let bot_id_c = bot_id.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {e}");
                return Err("database unavailable".to_string());
            }
        };
        let _ = diesel::sql_query(
            "INSERT INTO bot_skills (bot_id, skill_id, name, installed_at) VALUES ($1, $2, $3, NOW()) ON CONFLICT (bot_id, skill_id) DO NOTHING",
        )
        .bind::<diesel::sql_types::Text, _>(&bot_id_c)
        .bind::<diesel::sql_types::Text, _>(&skill_id_c)
        .bind::<diesel::sql_types::Text, _>(&skill_id_c)
        .execute(&mut conn);
        Ok::<(), String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({
                "ok": true,
                "skill": skill_id,
                "bot_id": bot_id,
                "message": "Skill installed successfully",
            })),
        ),
        _ => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "ok": false, "error": "skill install failed" })),
        ),
    }
}
