use crate::basic::keywords::mcp_directory::McpCsvLoader;
use crate::basic::keywords::get_all_keywords;
use crate::core::shared::state::AppState;
use super::types::{ApiResponse, SearchQuery, BotQuery, RepositoryInfo, AppInfo};

use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Json,
};
use std::fmt::Write;
use std::sync::Arc;

pub async fn handle_list_repositories(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    use super::html_renderers::html_escape;

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
                        {}
                    </span>
                    <span class="repo-meta-item">⭐ {}</span>
                    <span class="repo-meta-item">🍴 {}</span>
                    <span class="repo-meta-item">Last sync: {}</span>
                </div>
                <div class="repo-actions">
                    <button class="btn-browse" onclick="window.open('{}', '_blank')">Browse</button>
                </div>
            </div>"#,
                html_escape(&repo.name),
                html_escape(&repo.owner),
                status_class,
                status_text,
                html_escape(&repo.description),
                language,
                repo.stars,
                repo.forks,
                last_sync,
                html_escape(&repo.url)
        );
    }

    if repos.is_empty() {
        html.push_str(r#"<div class="empty-state">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
            </svg>
            <h3>No Repositories</h3>
            <p>Connect your GitHub repositories to get started</p>
        </div>"#);
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
        "Repository {} disconnected",
        id
    )))
}

pub async fn handle_list_apps(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    use super::html_renderers::html_escape;

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
                    </div>
                </div>
                <p class="app-description">{}</p>
                <div class="app-actions">
                    <button class="btn-open" onclick="window.location.href='{}'">Open</button>
                    <button class="btn-edit">Edit</button>
                </div>
            </div>"#,
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
            <h3>No Apps</h3>
            <p>Create your first app to get started</p>
        </div>"#);
    }

    html.push_str("</div>");
    Html(html)
}

pub async fn handle_prompts(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    use super::html_renderers::{html_escape, get_prompts_data};

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
    use super::html_renderers::{html_escape, get_templates_data};

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
    use super::html_renderers::html_escape;

    let news_items = vec![
        (
            "📢",
            "General Bots 6.0 Released",
            "Major update with improved performance and new features",
            "2 hours ago",
        ),
        (
            "🔌",
            "New MCP Server Integration",
            "Connect to external tools more easily with our new MCP support",
            "1 day ago",
        ),
        (
            "📊",
            "Analytics Dashboard Update",
            "Real-time metrics and improved visualizations",
            "3 days ago",
        ),
        (
            "🔒",
            "Security Enhancement",
            "Enhanced encryption and authentication options",
            "1 week ago",
        ),
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
    State(_state): State<Arc<AppState>>,
    Query(params): Query<BotQuery>,
) -> impl IntoResponse {
    use super::html_renderers::html_escape;

    let bot_id = params.bot_id.unwrap_or_else(|| "default".to_string());
    let work_path = crate::core::shared::utils::get_work_path();

    let keywords = get_all_keywords();
    let loader = McpCsvLoader::new(&work_path, &bot_id);
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

pub async fn handle_models(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    use super::html_renderers::html_escape;

    let models = vec![
        (
            "🧠",
            "GPT-4o",
            "OpenAI",
            "Latest multimodal model",
            "Active",
        ),
        (
            "🧠",
            "GPT-4o-mini",
            "OpenAI",
            "Fast and efficient",
            "Active",
        ),
        (
            "🦙",
            "Llama 3.1 70B",
            "Meta",
            "Open source LLM",
            "Available",
        ),
        (
            "🔷",
            "Claude 3.5 Sonnet",
            "Anthropic",
            "Advanced reasoning",
            "Available",
        ),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"models-container\">");
    html.push_str("<div class=\"models-header\"><h3>AI Models</h3><p>Available language models for your bots</p></div>");
    html.push_str("<div class=\"models-grid\">");

    for (icon, name, provider, description, status) in &models {
        let status_class = if *status == "Active" {
            "model-active"
        } else {
            "model-available"
        };
        let _ = write!(
            html,
                "<div class=\"model-card {}\"><div class=\"model-icon\">{}</div><div class=\"model-info\"><div class=\"model-header\"><h4>{}</h4><span class=\"model-provider\">{}</span></div><p>{}</p><div class=\"model-footer\"><span class=\"model-status\">{}</span></div></div></div>",
                status_class, icon, html_escape(name), html_escape(provider), html_escape(description), status
        );
    }

    html.push_str("</div></div>");
    Html(html)
}

pub async fn handle_search(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    use super::html_renderers::{html_escape, get_prompts_data};

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
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    use super::html_renderers::html_escape;

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
    let work_path = crate::core::shared::utils::get_work_path();
    let loader = McpCsvLoader::new(&work_path, &bot_id);
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub key_preview: String,
    pub created_at: String,
    pub last_used: Option<String>,
}

pub async fn handle_list_api_keys(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    use crate::basic::keywords::config::get_config;
    use crate::core::config::manager::ConfigManager;
    use std::sync::Arc as StdArc;

    let config_manager = state.config_manager.clone();
    let default_bot_id = uuid::Uuid::nil();

    let mut keys = Vec::new();

    let llm_key = config_manager.get_config(&default_bot_id, "llm-key", None).unwrap_or_default();
    if !llm_key.is_empty() {
        keys.push(ApiKeyInfo {
            id: "llm-key".to_string(),
            name: "LLM API Key".to_string(),
            provider: "OpenAI Compatible".to_string(),
            key_preview: format!("sk-...{}", &llm_key.chars().take(4).collect::<String>()),
            created_at: "2024-01-01".to_string(),
            last_used: None,
        });
    }

    let openai_key = config_manager.get_config(&default_bot_id, "openai-key", None).unwrap_or_default();
    if !openai_key.is_empty() {
        keys.push(ApiKeyInfo {
            id: "openai-key".to_string(),
            name: "OpenAI API Key".to_string(),
            provider: "OpenAI".to_string(),
            key_preview: format!("sk-...{}", &openai_key.chars().take(4).collect::<String>()),
            created_at: "2024-01-01".to_string(),
            last_used: None,
        });
    }

    let anthropic_key = config_manager.get_config(&default_bot_id, "anthropic-key", None).unwrap_or_default();
    if !anthropic_key.is_empty() {
        keys.push(ApiKeyInfo {
            id: "anthropic-key".to_string(),
            name: "Anthropic API Key".to_string(),
            provider: "Anthropic (Claude)".to_string(),
            key_preview: format!("sk-ant-...{}", &anthropic_key.chars().take(4).collect::<String>()),
            created_at: "2024-01-01".to_string(),
            last_used: None,
        });
    }

    let html = render_api_keys_html(&keys);
    Html(html)
}

fn render_api_keys_html(keys: &[ApiKeyInfo]) -> String {
    let mut html = String::new();
    html.push_str(r#"<div class="section-header">
        <h2>API Keys (BYOK)</h2>
        <button class="btn btn-primary" onclick="showAddKeyModal()">+ Add API Key</button>
    </div>
    <p style="margin-bottom: 24px; color: var(--text-secondary);">
        Manage your own LLM API keys for BYOK (Bring Your Own Key) mode. These keys are stored securely and used for AI features.
    </p>"#);

    if keys.is_empty() {
        html.push_str(r#"<div class="empty-state">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"></path>
            </svg>
            <h3>No API Keys Configured</h3>
            <p>Add your own LLM API keys to enable BYOK mode</p>
        </div>"#);
    } else {
        html.push_str(r#"<div class="servers-grid">"#);
        for key in keys {
            let _ = write!(html, r#"
                <div class="server-card">
                    <div class="server-header">
                        <div class="server-icon">🔑</div>
                        <div class="server-info">
                            <div class="server-name">{}</div>
                            <span class="server-type">{}</span>
                        </div>
                    </div>
                    <div class="server-description">{}</div>
                    <div class="server-meta">
                        <span class="server-meta-item">Created: {}</span>
                    </div>
                    <div class="server-actions" style="margin-top: 12px; display: flex; gap: 8px;">
                        <button class="btn btn-sm btn-outline" onclick="deleteApiKey('{}')">Delete</button>
                    </div>
                </div>"#,
                key.name,
                key.provider,
                key.key_preview,
                key.created_at,
                key.id
            );
        }
        html.push_str("</div>");
    }

    html.push_str(r#"<script>
        function showAddKeyModal() {
            const modal = document.createElement('div');
            modal.className = 'modal-overlay';
            modal.innerHTML = `
                <div class="modal-content" style="background: white; padding: 24px; border-radius: 12px; max-width: 500px; margin: 100px auto;">
                    <h3 style="margin-bottom: 16px;">Add API Key</h3>
                    <div class="form-group" style="margin-bottom: 16px;">
                        <label style="display: block; margin-bottom: 8px; font-weight: 500;">Name</label>
                        <input type="text" id="keyName" placeholder="e.g., My OpenAI Key" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 8px;">
                    </div>
                    <div class="form-group" style="margin-bottom: 16px;">
                        <label style="display: block; margin-bottom: 8px; font-weight: 500;">Provider</label>
                        <select id="keyProvider" style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 8px;">
                            <option value="llm-key">OpenAI Compatible</option>
                            <option value="openai-key">OpenAI</option>
                            <option value="anthropic-key">Anthropic (Claude)</option>
                            <option value="google-key">Google (Gemini)</option>
                            <option value="azure-key">Azure OpenAI</option>
                        </select>
                    </div>
                    <div class="form-group" style="margin-bottom: 16px;">
                        <label style="display: block; margin-bottom: 8px; font-weight: 500;">API Key</label>
                        <input type="password" id="keyValue" placeholder="sk-..." style="width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 8px;">
                    </div>
                    <div style="display: flex; gap: 12px;">
                        <button class="btn btn-primary" onclick="saveApiKey()">Save</button>
                        <button class="btn btn-outline" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                    </div>
                </div>
            `;
            document.body.appendChild(modal);
        }

        async function saveApiKey() {
            const provider = document.getElementById('keyProvider').value;
            const keyValue = document.getElementById('keyValue').value;
            if (!keyValue) {
                alert('Please enter an API key');
                return;
            }
            try {
                const response = await fetch('/api/ui/sources/api-keys', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ provider, key: keyValue })
                });
                if (response.ok) {
                    location.reload();
                } else {
                    alert('Failed to save API key');
                }
            } catch (e) {
                alert('Error: ' + e.message);
            }
        }

        async function deleteApiKey(id) {
            if (confirm('Delete this API key?')) {
                try {
                    const response = await fetch('/api/ui/sources/api-keys/' + id, { method: 'DELETE' });
                    if (response.ok) {
                        location.reload();
                    }
                } catch (e) {
                    alert('Error: ' + e.message);
                }
            }
        }
    </script>"#);

    html
}

pub async fn handle_add_api_key(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    use crate::core::config::manager::ConfigManager;

    let provider = payload.get("provider").and_then(|v| v.as_str()).unwrap_or("llm-key");
    let key = payload.get("key").and_then(|v| v.as_str()).unwrap_or("");

    if key.is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": "Key is required" })));
    }

    let config_manager = state.config_manager.clone();
    let default_bot_id = uuid::Uuid::nil();

    if let Err(e) = config_manager.set_config(&default_bot_id, provider, key) {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })));
    }

    Json(serde_json::json!({ "success": true }))
}

pub async fn handle_delete_api_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use crate::core::config::manager::ConfigManager;

    let config_manager = state.config_manager.clone();
    let default_bot_id = uuid::Uuid::nil();

    if let Err(e) = config_manager.set_config(&default_bot_id, &id, "") {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() })));
    }

    Json(serde_json::json!({ "success": true }))
}

pub fn configure_sources_routes() -> axum::Router<Arc<AppState>> {
    use crate::core::urls::ApiUrls;
    use super::mcp_handlers::*;
    use super::handlers::*;

    axum::Router::new()
        .route(ApiUrls::SOURCES_PROMPTS, get(handle_prompts))
        .route(ApiUrls::SOURCES_TEMPLATES, get(handle_templates))
        .route(ApiUrls::SOURCES_NEWS, get(handle_news))
        .route(ApiUrls::SOURCES_MCP_SERVERS, get(handle_mcp_servers))
        .route(ApiUrls::SOURCES_LLM_TOOLS, get(handle_llm_tools))
        .route(ApiUrls::SOURCES_MODELS, get(handle_models))
        .route(ApiUrls::SOURCES_SEARCH, get(handle_search))
        .route(ApiUrls::SOURCES_REPOSITORIES, get(handle_list_repositories))
        .route(
            ApiUrls::SOURCES_REPOSITORIES_CONNECT,
            post(handle_connect_repository),
        )
        .route(
            ApiUrls::SOURCES_REPOSITORIES_DISCONNECT,
            post(handle_disconnect_repository),
        )
        .route(ApiUrls::SOURCES_APPS, get(handle_list_apps))
        .route(ApiUrls::SOURCES_MCP, get(handle_list_mcp_servers_json))
        .route(ApiUrls::SOURCES_MCP, post(handle_add_mcp_server))
        .route(ApiUrls::SOURCES_MCP_BY_NAME, get(handle_get_mcp_server).put(handle_update_mcp_server).delete(handle_delete_mcp_server))
        .route(ApiUrls::SOURCES_MCP_ENABLE, post(handle_enable_mcp_server))
        .route(ApiUrls::SOURCES_MCP_DISABLE, post(handle_disable_mcp_server))
        .route(ApiUrls::SOURCES_MCP_TOOLS, get(handle_list_mcp_server_tools))
        .route(ApiUrls::SOURCES_MCP_TEST, post(handle_test_mcp_server))
        .route(ApiUrls::SOURCES_MCP_SCAN, post(handle_scan_mcp_directory))
        .route(ApiUrls::SOURCES_MCP_EXAMPLES, get(handle_get_mcp_examples))
        .route(ApiUrls::SOURCES_API_KEYS, get(handle_list_api_keys).post(handle_add_api_key))
        .route(ApiUrls::SOURCES_API_KEYS_BY_ID, delete(handle_delete_api_key))
        .route(ApiUrls::SOURCES_MENTIONS, get(handle_mentions_autocomplete))
        .route(ApiUrls::SOURCES_TOOLS, get(handle_list_all_tools))
}
