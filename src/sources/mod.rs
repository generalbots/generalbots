use crate::shared::state::AppState;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

pub fn configure_sources_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Tab endpoints - match frontend hx-get endpoints
        .route("/api/sources/prompts", get(handle_prompts))
        .route("/api/sources/templates", get(handle_templates))
        .route("/api/sources/news", get(handle_news))
        .route("/api/sources/mcp-servers", get(handle_mcp_servers))
        .route("/api/sources/llm-tools", get(handle_llm_tools))
        .route("/api/sources/models", get(handle_models))
        // Search
        .route("/api/sources/search", get(handle_search))
}

/// GET /api/sources/prompts - Prompts tab content
pub async fn handle_prompts(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let category = params.category.unwrap_or_else(|| "all".to_string());

    let prompts = get_prompts_data(&category);

    let mut html = String::new();
    html.push_str("<div class=\"panel-layout\">");

    // Categories sidebar
    html.push_str("<aside class=\"categories-sidebar\">");
    html.push_str("<h3>Categories</h3>");
    html.push_str("<div class=\"category-list\">");

    let categories = vec![
        ("all", "All Prompts", ""),
        ("writing", "Writing", ""),
        ("coding", "Coding", ""),
        ("analysis", "Analysis", ""),
        ("creative", "Creative", ""),
        ("business", "Business", ""),
        ("education", "Education", ""),
    ];

    for (id, name, icon) in &categories {
        let active = if *id == category { " active" } else { "" };
        html.push_str("<button class=\"category-item");
        html.push_str(active);
        html.push_str("\" hx-get=\"/api/sources/prompts?category=");
        html.push_str(id);
        html.push_str("\" hx-target=\"#content-area\" hx-swap=\"innerHTML\">");
        html.push_str("<span class=\"category-icon\">");
        html.push_str(icon);
        html.push_str("</span>");
        html.push_str("<span class=\"category-name\">");
        html.push_str(name);
        html.push_str("</span>");
        html.push_str("</button>");
    }

    html.push_str("</div>");
    html.push_str("</aside>");

    // Prompts grid
    html.push_str("<div class=\"content-main\">");
    html.push_str("<div class=\"prompts-grid\" id=\"prompts-grid\">");

    for prompt in &prompts {
        html.push_str("<div class=\"prompt-card\">");
        html.push_str("<div class=\"prompt-header\">");
        html.push_str("<span class=\"prompt-icon\">");
        html.push_str(&prompt.icon);
        html.push_str("</span>");
        html.push_str("<h4>");
        html.push_str(&html_escape(&prompt.title));
        html.push_str("</h4>");
        html.push_str("</div>");
        html.push_str("<p class=\"prompt-description\">");
        html.push_str(&html_escape(&prompt.description));
        html.push_str("</p>");
        html.push_str("<div class=\"prompt-footer\">");
        html.push_str("<span class=\"prompt-category\">");
        html.push_str(&html_escape(&prompt.category));
        html.push_str("</span>");
        html.push_str("<button class=\"btn-use\" onclick=\"usePrompt('");
        html.push_str(&html_escape(&prompt.id));
        html.push_str("')\">Use</button>");
        html.push_str("</div>");
        html.push_str("</div>");
    }

    if prompts.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No prompts found in this category</p>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

/// GET /api/sources/templates - Templates tab content
pub async fn handle_templates(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let templates = get_templates_data();

    let mut html = String::new();
    html.push_str("<div class=\"templates-container\">");
    html.push_str("<div class=\"templates-header\">");
    html.push_str("<h3>Bot Templates</h3>");
    html.push_str("<p>Pre-built bot configurations ready to deploy</p>");
    html.push_str("</div>");
    html.push_str("<div class=\"templates-grid\">");

    for template in &templates {
        html.push_str("<div class=\"template-card\">");
        html.push_str("<div class=\"template-icon\">");
        html.push_str(&template.icon);
        html.push_str("</div>");
        html.push_str("<div class=\"template-info\">");
        html.push_str("<h4>");
        html.push_str(&html_escape(&template.name));
        html.push_str("</h4>");
        html.push_str("<p>");
        html.push_str(&html_escape(&template.description));
        html.push_str("</p>");
        html.push_str("<div class=\"template-meta\">");
        html.push_str("<span class=\"template-category\">");
        html.push_str(&html_escape(&template.category));
        html.push_str("</span>");
        html.push_str("</div>");
        html.push_str("</div>");
        html.push_str("<div class=\"template-actions\">");
        html.push_str("<button class=\"btn-preview\">Preview</button>");
        html.push_str("<button class=\"btn-use-template\">Use Template</button>");
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

/// GET /api/sources/news - News tab content
pub async fn handle_news(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let news_items = vec![
        ("", "General Bots 6.0 Released", "Major update with improved performance and new features", "2 hours ago"),
        ("", "New MCP Server Integration", "Connect to external tools more easily with our new MCP support", "1 day ago"),
        ("", "Analytics Dashboard Update", "Real-time metrics and improved visualizations", "3 days ago"),
        ("", "Security Enhancement", "Enhanced encryption and authentication options", "1 week ago"),
        ("", "Multi-language Support", "Now supporting 15+ languages for bot conversations", "2 weeks ago"),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"news-container\">");
    html.push_str("<div class=\"news-header\">");
    html.push_str("<h3>Latest News</h3>");
    html.push_str("<p>Updates and announcements from the General Bots team</p>");
    html.push_str("</div>");
    html.push_str("<div class=\"news-list\">");

    for (icon, title, description, time) in &news_items {
        html.push_str("<div class=\"news-item\">");
        html.push_str("<div class=\"news-icon\">");
        html.push_str(icon);
        html.push_str("</div>");
        html.push_str("<div class=\"news-content\">");
        html.push_str("<h4>");
        html.push_str(&html_escape(title));
        html.push_str("</h4>");
        html.push_str("<p>");
        html.push_str(&html_escape(description));
        html.push_str("</p>");
        html.push_str("<span class=\"news-time\">");
        html.push_str(time);
        html.push_str("</span>");
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

/// GET /api/sources/mcp-servers - MCP Servers tab content
pub async fn handle_mcp_servers(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let servers = vec![
        ("", "Database Server", "PostgreSQL, MySQL, SQLite connections", "Active", true),
        ("", "Filesystem Server", "Local and cloud file access", "Active", true),
        ("", "Web Server", "HTTP/REST API integrations", "Active", true),
        ("", "Email Server", "SMTP/IMAP email handling", "Inactive", false),
        ("", "Slack Server", "Slack workspace integration", "Active", true),
        ("", "Analytics Server", "Data processing and reporting", "Active", true),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"mcp-container\">");
    html.push_str("<div class=\"mcp-header\">");
    html.push_str("<h3>MCP Servers</h3>");
    html.push_str("<p>Model Context Protocol servers for extended capabilities</p>");
    html.push_str("<button class=\"btn-add-server\">+ Add Server</button>");
    html.push_str("</div>");
    html.push_str("<div class=\"mcp-grid\">");

    for (icon, name, description, status, is_active) in &servers {
        let status_class = if *is_active { "status-active" } else { "status-inactive" };
        html.push_str("<div class=\"mcp-card\">");
        html.push_str("<div class=\"mcp-icon\">");
        html.push_str(icon);
        html.push_str("</div>");
        html.push_str("<div class=\"mcp-info\">");
        html.push_str("<h4>");
        html.push_str(&html_escape(name));
        html.push_str("</h4>");
        html.push_str("<p>");
        html.push_str(&html_escape(description));
        html.push_str("</p>");
        html.push_str("</div>");
        html.push_str("<div class=\"mcp-status ");
        html.push_str(status_class);
        html.push_str("\">");
        html.push_str(status);
        html.push_str("</div>");
        html.push_str("<div class=\"mcp-actions\">");
        html.push_str("<button class=\"btn-configure\">Configure</button>");
        if *is_active {
            html.push_str("<button class=\"btn-disable\">Disable</button>");
        } else {
            html.push_str("<button class=\"btn-enable\">Enable</button>");
        }
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

/// GET /api/sources/llm-tools - LLM Tools tab content
pub async fn handle_llm_tools(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let tools = vec![
        ("", "Web Search", "Search the web for real-time information", true),
        ("", "Calculator", "Perform mathematical calculations", true),
        ("", "Calendar", "Manage calendar events and schedules", true),
        ("", "Note Taking", "Create and manage notes", true),
        ("", "Weather", "Get weather forecasts and conditions", false),
        ("", "News Reader", "Fetch and summarize news articles", false),
        ("", "URL Fetcher", "Retrieve and parse web content", true),
        ("", "Code Executor", "Run code snippets safely", false),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"tools-container\">");
    html.push_str("<div class=\"tools-header\">");
    html.push_str("<h3>LLM Tools</h3>");
    html.push_str("<p>Extend your bot's capabilities with these tools</p>");
    html.push_str("</div>");
    html.push_str("<div class=\"tools-grid\">");

    for (icon, name, description, enabled) in &tools {
        let enabled_class = if *enabled { "enabled" } else { "disabled" };
        html.push_str("<div class=\"tool-card ");
        html.push_str(enabled_class);
        html.push_str("\">");
        html.push_str("<div class=\"tool-icon\">");
        html.push_str(icon);
        html.push_str("</div>");
        html.push_str("<div class=\"tool-info\">");
        html.push_str("<h4>");
        html.push_str(&html_escape(name));
        html.push_str("</h4>");
        html.push_str("<p>");
        html.push_str(&html_escape(description));
        html.push_str("</p>");
        html.push_str("</div>");
        html.push_str("<label class=\"toggle-switch\">");
        if *enabled {
            html.push_str("<input type=\"checkbox\" checked>");
        } else {
            html.push_str("<input type=\"checkbox\">");
        }
        html.push_str("<span class=\"toggle-slider\"></span>");
        html.push_str("</label>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

/// GET /api/sources/models - Models tab content
pub async fn handle_models(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let models = vec![
        ("🧠", "GPT-4o", "OpenAI", "Latest multimodal model with vision capabilities", "Active"),
        ("🧠", "GPT-4o-mini", "OpenAI", "Fast and efficient for most tasks", "Active"),
        ("🦙", "Llama 3.1 70B", "Meta", "Open source large language model", "Available"),
        ("🔷", "Claude 3.5 Sonnet", "Anthropic", "Advanced reasoning and analysis", "Available"),
        ("💎", "Gemini Pro", "Google", "Multimodal AI with long context", "Available"),
        ("", "Mistral Large", "Mistral AI", "European AI model with strong performance", "Available"),
    ];

    let mut html = String::new();
    html.push_str("<div class=\"models-container\">");
    html.push_str("<div class=\"models-header\">");
    html.push_str("<h3>AI Models</h3>");
    html.push_str("<p>Available language models for your bots</p>");
    html.push_str("</div>");
    html.push_str("<div class=\"models-grid\">");

    for (icon, name, provider, description, status) in &models {
        let status_class = if *status == "Active" { "model-active" } else { "model-available" };
        html.push_str("<div class=\"model-card ");
        html.push_str(status_class);
        html.push_str("\">");
        html.push_str("<div class=\"model-icon\">");
        html.push_str(icon);
        html.push_str("</div>");
        html.push_str("<div class=\"model-info\">");
        html.push_str("<div class=\"model-header\">");
        html.push_str("<h4>");
        html.push_str(&html_escape(name));
        html.push_str("</h4>");
        html.push_str("<span class=\"model-provider\">");
        html.push_str(&html_escape(provider));
        html.push_str("</span>");
        html.push_str("</div>");
        html.push_str("<p>");
        html.push_str(&html_escape(description));
        html.push_str("</p>");
        html.push_str("<div class=\"model-footer\">");
        html.push_str("<span class=\"model-status\">");
        html.push_str(status);
        html.push_str("</span>");
        if *status == "Active" {
            html.push_str("<button class=\"btn-configure\">Configure</button>");
        } else {
            html.push_str("<button class=\"btn-activate\">Activate</button>");
        }
        html.push_str("</div>");
        html.push_str("</div>");
        html.push_str("</div>");
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

/// GET /api/sources/search - Search across all sources
pub async fn handle_search(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default();

    if query.is_empty() {
        return Html("<div class=\"search-prompt\"><p>Enter a search term</p></div>".to_string());
    }

    let query_lower = query.to_lowercase();

    // Search across prompts
    let prompts = get_prompts_data("all");
    let matching_prompts: Vec<_> = prompts
        .iter()
        .filter(|p| {
            p.title.to_lowercase().contains(&query_lower)
                || p.description.to_lowercase().contains(&query_lower)
        })
        .collect();

    // Search across templates
    let templates = get_templates_data();
    let matching_templates: Vec<_> = templates
        .iter()
        .filter(|t| {
            t.name.to_lowercase().contains(&query_lower)
                || t.description.to_lowercase().contains(&query_lower)
        })
        .collect();

    let mut html = String::new();
    html.push_str("<div class=\"search-results\">");
    html.push_str("<div class=\"search-header\">");
    html.push_str("<h3>Search Results for \"");
    html.push_str(&html_escape(&query));
    html.push_str("\"</h3>");
    html.push_str("</div>");

    if matching_prompts.is_empty() && matching_templates.is_empty() {
        html.push_str("<div class=\"no-results\">");
        html.push_str("<p>No results found</p>");
        html.push_str("<p class=\"hint\">Try different keywords</p>");
        html.push_str("</div>");
    } else {
        if !matching_prompts.is_empty() {
            html.push_str("<div class=\"result-section\">");
            html.push_str("<h4>Prompts (");
            html.push_str(&matching_prompts.len().to_string());
            html.push_str(")</h4>");
            html.push_str("<div class=\"results-grid\">");
            for prompt in matching_prompts {
                html.push_str("<div class=\"result-item prompt-result\">");
                html.push_str("<span class=\"result-icon\">");
                html.push_str(&prompt.icon);
                html.push_str("</span>");
                html.push_str("<div class=\"result-info\">");
                html.push_str("<strong>");
                html.push_str(&html_escape(&prompt.title));
                html.push_str("</strong>");
                html.push_str("<p>");
                html.push_str(&html_escape(&prompt.description));
                html.push_str("</p>");
                html.push_str("</div>");
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html.push_str("</div>");
        }

        if !matching_templates.is_empty() {
            html.push_str("<div class=\"result-section\">");
            html.push_str("<h4>Templates (");
            html.push_str(&matching_templates.len().to_string());
            html.push_str(")</h4>");
            html.push_str("<div class=\"results-grid\">");
            for template in matching_templates {
                html.push_str("<div class=\"result-item template-result\">");
                html.push_str("<span class=\"result-icon\">");
                html.push_str(&template.icon);
                html.push_str("</span>");
                html.push_str("<div class=\"result-info\">");
                html.push_str("<strong>");
                html.push_str(&html_escape(&template.name));
                html.push_str("</strong>");
                html.push_str("<p>");
                html.push_str(&html_escape(&template.description));
                html.push_str("</p>");
                html.push_str("</div>");
                html.push_str("</div>");
            }
            html.push_str("</div>");
            html.push_str("</div>");
        }
    }

    html.push_str("</div>");

    Html(html)
}

// Data structures

struct PromptData {
    id: String,
    title: String,
    description: String,
    category: String,
    icon: String,
}

struct TemplateData {
    name: String,
    description: String,
    category: String,
    icon: String,
}

fn get_prompts_data(category: &str) -> Vec<PromptData> {
    let all_prompts = vec![
        PromptData {
            id: "summarize".to_string(),
            title: "Summarize Text".to_string(),
            description: "Create concise summaries of long documents or articles".to_string(),
            category: "writing".to_string(),
            icon: "".to_string(),
        },
        PromptData {
            id: "code-review".to_string(),
            title: "Code Review".to_string(),
            description: "Analyze code for bugs, improvements, and best practices".to_string(),
            category: "coding".to_string(),
            icon: "".to_string(),
        },
        PromptData {
            id: "data-analysis".to_string(),
            title: "Data Analysis".to_string(),
            description: "Extract insights and patterns from data sets".to_string(),
            category: "analysis".to_string(),
            icon: "".to_string(),
        },
        PromptData {
            id: "creative-writing".to_string(),
            title: "Creative Writing".to_string(),
            description: "Generate stories, poems, and creative content".to_string(),
            category: "creative".to_string(),
            icon: "".to_string(),
        },
        PromptData {
            id: "email-draft".to_string(),
            title: "Email Draft".to_string(),
            description: "Compose professional emails quickly".to_string(),
            category: "business".to_string(),
            icon: "".to_string(),
        },
        PromptData {
            id: "explain-concept".to_string(),
            title: "Explain Concept".to_string(),
            description: "Break down complex topics into simple explanations".to_string(),
            category: "education".to_string(),
            icon: "".to_string(),
        },
        PromptData {
            id: "debug-code".to_string(),
            title: "Debug Code".to_string(),
            description: "Find and fix issues in your code".to_string(),
            category: "coding".to_string(),
            icon: "🐛".to_string(),
        },
        PromptData {
            id: "meeting-notes".to_string(),
            title: "Meeting Notes".to_string(),
            description: "Organize and format meeting discussions".to_string(),
            category: "business".to_string(),
            icon: "".to_string(),
        },
    ];

    if category == "all" {
        all_prompts
    } else {
        all_prompts
            .into_iter()
            .filter(|p| p.category == category)
            .collect()
    }
}

fn get_templates_data() -> Vec<TemplateData> {
    vec![
        TemplateData {
            name: "Customer Support Bot".to_string(),
            description: "Handle customer inquiries and support tickets automatically".to_string(),
            category: "Support".to_string(),
            icon: "🎧".to_string(),
        },
        TemplateData {
            name: "FAQ Bot".to_string(),
            description: "Answer frequently asked questions from your knowledge base".to_string(),
            category: "Support".to_string(),
            icon: "".to_string(),
        },
        TemplateData {
            name: "Lead Generation Bot".to_string(),
            description: "Qualify leads and collect prospect information".to_string(),
            category: "Sales".to_string(),
            icon: "".to_string(),
        },
        TemplateData {
            name: "Onboarding Bot".to_string(),
            description: "Guide new users through your product or service".to_string(),
            category: "HR".to_string(),
            icon: "👋".to_string(),
        },
        TemplateData {
            name: "Survey Bot".to_string(),
            description: "Collect feedback through conversational surveys".to_string(),
            category: "Research".to_string(),
            icon: "".to_string(),
        },
        TemplateData {
            name: "Appointment Scheduler".to_string(),
            description: "Book and manage appointments automatically".to_string(),
            category: "Productivity".to_string(),
            icon: "".to_string(),
        },
    ]
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
