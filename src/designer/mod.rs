use crate::auto_task::get_designer_error_context;
use crate::core::shared::get_content_type;
use crate::core::urls::ApiUrls;
use crate::shared::state::AppState;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub name: Option<String>,
    pub content: Option<String>,
    pub nodes: Option<serde_json::Value>,
    pub connections: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub content: Option<String>,
    pub nodes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileQuery {
    pub path: Option<String>,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DialogRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub content: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub line: usize,
    pub message: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicRequest {
    pub nodes: Vec<MagicNode>,
    pub connections: i32,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMagicRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMagicResponse {
    pub improved_code: Option<String>,
    pub explanation: Option<String>,
    pub suggestions: Option<Vec<MagicSuggestion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicSuggestion {
    #[serde(rename = "type")]
    pub suggestion_type: String,
    pub title: String,
    pub description: String,
}

pub fn configure_designer_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::DESIGNER_FILES, get(handle_list_files))
        .route(ApiUrls::DESIGNER_LOAD, get(handle_load_file))
        .route(ApiUrls::DESIGNER_SAVE, post(handle_save))
        .route(ApiUrls::DESIGNER_VALIDATE, post(handle_validate))
        .route(ApiUrls::DESIGNER_EXPORT, get(handle_export))
        .route(
            "/api/designer/dialogs",
            get(handle_list_dialogs).post(handle_create_dialog),
        )
        .route("/api/designer/dialogs/{id}", get(handle_get_dialog))
        .route(ApiUrls::DESIGNER_MODIFY, post(handle_designer_modify))
        .route("/api/v1/designer/magic", post(handle_magic_suggestions))
        .route("/api/v1/editor/magic", post(handle_editor_magic))
}

pub async fn handle_editor_magic(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EditorMagicRequest>,
) -> impl IntoResponse {
    let code = request.code;

    if code.trim().is_empty() {
        return Json(EditorMagicResponse {
            improved_code: None,
            explanation: Some("No code provided".to_string()),
            suggestions: None,
        });
    }

    let prompt = format!(
        r#"You are reviewing this HTMX application code. Analyze and improve it.

Focus on:
- Better HTMX patterns (reduce JS, use hx-* attributes properly)
- Accessibility (ARIA labels, keyboard navigation, semantic HTML)
- Performance (lazy loading, efficient selectors)
- UX (loading states, error handling, user feedback)
- Code organization (clean structure, no comments needed)

Current code:
```
{code}
```

Respond with JSON only:
{{
    "improved_code": "the improved code here",
    "explanation": "brief explanation of changes made"
}}

If the code is already good, respond with:
{{
    "improved_code": null,
    "explanation": "Code looks good, no improvements needed"
}}"#
    );

    #[cfg(feature = "llm")]
    {
        let config = serde_json::json!({
            "temperature": 0.3,
            "max_tokens": 4000
        });

        match state
            .llm_provider
            .generate(&prompt, &config, "gpt-4", "")
            .await
        {
            Ok(response) => {
                if let Ok(result) = serde_json::from_str::<EditorMagicResponse>(&response) {
                    return Json(result);
                }
                return Json(EditorMagicResponse {
                    improved_code: Some(response),
                    explanation: Some("AI suggestions".to_string()),
                    suggestions: None,
                });
            }
            Err(e) => {
                log::warn!("LLM call failed: {e}");
            }
        }
    }

    let _ = state;
    let mut suggestions = Vec::new();

    if !code.contains("hx-") {
        suggestions.push(MagicSuggestion {
            suggestion_type: "ux".to_string(),
            title: "Use HTMX attributes".to_string(),
            description: "Consider using hx-get, hx-post instead of JavaScript fetch calls."
                .to_string(),
        });
    }

    if !code.contains("hx-indicator") {
        suggestions.push(MagicSuggestion {
            suggestion_type: "ux".to_string(),
            title: "Add loading indicators".to_string(),
            description: "Use hx-indicator to show loading state during requests.".to_string(),
        });
    }

    if !code.contains("aria-") && !code.contains("role=") {
        suggestions.push(MagicSuggestion {
            suggestion_type: "a11y".to_string(),
            title: "Improve accessibility".to_string(),
            description: "Add ARIA labels and roles for screen reader support.".to_string(),
        });
    }

    if code.contains("onclick=") || code.contains("addEventListener") {
        suggestions.push(MagicSuggestion {
            suggestion_type: "perf".to_string(),
            title: "Replace JS with HTMX".to_string(),
            description: "HTMX can handle most interactions without custom JavaScript.".to_string(),
        });
    }

    Json(EditorMagicResponse {
        improved_code: None,
        explanation: None,
        suggestions: if suggestions.is_empty() {
            None
        } else {
            Some(suggestions)
        },
    })
}

pub async fn handle_magic_suggestions(
    State(state): State<Arc<AppState>>,
    Json(request): Json<MagicRequest>,
) -> impl IntoResponse {
    let mut suggestions = Vec::new();
    let nodes = &request.nodes;

    let has_hear = nodes.iter().any(|n| n.node_type == "HEAR");
    let has_talk = nodes.iter().any(|n| n.node_type == "TALK");
    let has_if = nodes
        .iter()
        .any(|n| n.node_type == "IF" || n.node_type == "SWITCH");
    let talk_count = nodes.iter().filter(|n| n.node_type == "TALK").count();

    if !has_hear && has_talk {
        suggestions.push(MagicSuggestion {
            suggestion_type: "ux".to_string(),
            title: "Add User Input".to_string(),
            description:
                "Your dialog has no HEAR nodes. Consider adding user input to make it interactive."
                    .to_string(),
        });
    }

    if talk_count > 5 {
        suggestions.push(MagicSuggestion {
            suggestion_type: "ux".to_string(),
            title: "Break Up Long Responses".to_string(),
            description:
                "You have many TALK nodes. Consider grouping related messages or using a menu."
                    .to_string(),
        });
    }

    if !has_if && nodes.len() > 3 {
        suggestions.push(MagicSuggestion {
            suggestion_type: "feature".to_string(),
            title: "Add Decision Logic".to_string(),
            description: "Add IF or SWITCH nodes to handle different user responses dynamically."
                .to_string(),
        });
    }

    if request.connections < (nodes.len() as i32 - 1) && nodes.len() > 1 {
        suggestions.push(MagicSuggestion {
            suggestion_type: "perf".to_string(),
            title: "Check Connections".to_string(),
            description: "Some nodes may not be connected. Ensure all nodes flow properly."
                .to_string(),
        });
    }

    if nodes.is_empty() {
        suggestions.push(MagicSuggestion {
            suggestion_type: "feature".to_string(),
            title: "Start with TALK".to_string(),
            description: "Begin your dialog with a TALK node to greet the user.".to_string(),
        });
    }

    suggestions.push(MagicSuggestion {
        suggestion_type: "a11y".to_string(),
        title: "Use Clear Language".to_string(),
        description: "Keep messages short and clear. Avoid jargon for better accessibility."
            .to_string(),
    });

    let _ = state;

    Json(suggestions)
}

pub async fn handle_list_files(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let files = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return get_default_files();
            }
        };

        let result: Result<Vec<DialogRow>, _> = diesel::sql_query(
            "SELECT id, name, content, updated_at FROM designer_dialogs ORDER BY updated_at DESC LIMIT 50",
        )
        .load(&mut db_conn);

        match result {
            Ok(dialogs) if !dialogs.is_empty() => dialogs
                .into_iter()
                .map(|d| (d.id, d.name, d.updated_at))
                .collect(),
            _ => get_default_files(),
        }
    })
    .await
    .unwrap_or_else(|_| get_default_files());

    let mut html = String::new();
    html.push_str("<div class=\"file-list\">");

    for (id, name, updated_at) in &files {
        let time_str = format_relative_time(*updated_at);
        html.push_str("<div class=\"file-item\" data-id=\"");
        html.push_str(&html_escape(id));
        html.push_str("\" onclick=\"selectFile(this)\">");
        html.push_str("<div class=\"file-icon\">");
        html.push_str("<svg width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\">");
        html.push_str(
            "<path d=\"M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z\"></path>",
        );
        html.push_str("<polyline points=\"14 2 14 8 20 8\"></polyline>");
        html.push_str("</svg>");
        html.push_str("</div>");
        html.push_str("<div class=\"file-info\">");
        html.push_str("<span class=\"file-name\">");
        html.push_str(&html_escape(name));
        html.push_str("</span>");
        html.push_str("<span class=\"file-time\">");
        html.push_str(&html_escape(&time_str));
        html.push_str("</span>");
        html.push_str("</div>");
        html.push_str("</div>");
    }

    if files.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No dialog files found</p>");
        html.push_str("<p class=\"hint\">Create a new dialog to get started</p>");
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}

fn get_default_files() -> Vec<(String, String, DateTime<Utc>)> {
    vec![
        (
            "welcome".to_string(),
            "Welcome Dialog".to_string(),
            Utc::now(),
        ),
        ("faq".to_string(), "FAQ Bot".to_string(), Utc::now()),
        (
            "support".to_string(),
            "Customer Support".to_string(),
            Utc::now(),
        ),
    ]
}

pub async fn handle_load_file(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FileQuery>,
) -> impl IntoResponse {
    let file_id = params.path.unwrap_or_else(|| "welcome".to_string());
    let conn = state.conn.clone();

    let dialog = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return None;
            }
        };

        diesel::sql_query(
            "SELECT id, name, content, updated_at FROM designer_dialogs WHERE id = $1",
        )
        .bind::<diesel::sql_types::Text, _>(&file_id)
        .get_result::<DialogRow>(&mut db_conn)
        .ok()
    })
    .await
    .unwrap_or(None);

    let content = match dialog {
        Some(d) => d.content,
        None => get_default_dialog_content(),
    };

    let mut html = String::new();
    html.push_str("<div class=\"canvas-loaded\" data-content=\"");
    html.push_str(&html_escape(&content));
    html.push_str("\">");

    let nodes = parse_basic_to_nodes(&content);
    for node in &nodes {
        html.push_str(&format_node_html(node));
    }

    html.push_str("</div>");
    html.push_str("<script>initializeCanvas();</script>");

    Html(html)
}

pub async fn handle_save(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let now = Utc::now();
    let name = payload.name.unwrap_or_else(|| "Untitled".to_string());
    let content = payload.content.unwrap_or_default();
    let dialog_id = Uuid::new_v4().to_string();

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Err(format!("Database error: {}", e));
            }
        };

        diesel::sql_query(
            "INSERT INTO designer_dialogs (id, name, description, bot_id, content, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (id) DO UPDATE SET content = $5, updated_at = $8",
        )
        .bind::<diesel::sql_types::Text, _>(&dialog_id)
        .bind::<diesel::sql_types::Text, _>(&name)
        .bind::<diesel::sql_types::Text, _>("")
        .bind::<diesel::sql_types::Text, _>("default")
        .bind::<diesel::sql_types::Text, _>(&content)
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut db_conn)
        .map_err(|e| format!("Save failed: {}", e))?;

        Ok(())
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task error: {}", e)));

    match result {
        Ok(_) => {
            let mut html = String::new();
            html.push_str("<div class=\"save-result success\">");
            html.push_str("<span class=\"save-icon\">*</span>");
            html.push_str("<span class=\"save-message\">Saved successfully</span>");
            html.push_str("</div>");
            Html(html)
        }
        Err(e) => {
            let mut html = String::new();
            html.push_str("<div class=\"save-result error\">");
            html.push_str("<span class=\"save-icon\">x</span>");
            html.push_str("<span class=\"save-message\">Save failed: ");
            html.push_str(&html_escape(&e));
            html.push_str("</span>");
            html.push_str("</div>");
            Html(html)
        }
    }
}

pub async fn handle_validate(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<ValidateRequest>,
) -> impl IntoResponse {
    let content = payload.content.unwrap_or_default();
    let validation = validate_basic_code(&content);

    let mut html = String::new();
    html.push_str("<div class=\"validation-result\">");

    if validation.valid {
        html.push_str("<div class=\"validation-success\">");
        html.push_str("<span class=\"validation-icon\">*</span>");
        html.push_str("<span class=\"validation-text\">Dialog is valid</span>");
        html.push_str("</div>");
    } else {
        if !validation.errors.is_empty() {
            html.push_str("<div class=\"validation-errors\">");
            html.push_str("<div class=\"validation-header\">");
            html.push_str("<span class=\"validation-icon\">✗</span>");
            html.push_str("<span class=\"validation-text\">");
            html.push_str(&validation.errors.len().to_string());
            html.push_str(" error(s) found</span>");
            html.push_str("</div>");
            html.push_str("<ul class=\"error-list\">");
            for error in &validation.errors {
                html.push_str("<li class=\"error-item\" data-line=\"");
                html.push_str(&error.line.to_string());
                html.push_str("\">");
                html.push_str("<span class=\"error-line\">Line ");
                html.push_str(&error.line.to_string());
                html.push_str(":</span> ");
                html.push_str(&html_escape(&error.message));
                html.push_str("</li>");
            }
        } else if !validation.warnings.is_empty() {
            html.push_str("<div class=\"validation-warnings\">");
            html.push_str("<div class=\"validation-header\">");
            html.push_str("<span class=\"validation-icon\">!</span>");
            html.push_str("<span class=\"validation-text\">");
            html.push_str(&validation.warnings.len().to_string());
            html.push_str(" warning(s)</span>");
            html.push_str("</div>");
            html.push_str("<ul class=\"warning-list\">");
            for warning in &validation.warnings {
                html.push_str("<li class=\"warning-item\">");
                html.push_str("<span class=\"warning-line\">Line ");
                html.push_str(&warning.line.to_string());
                html.push_str(":</span> ");
                html.push_str(&html_escape(&warning.message));
                html.push_str("</li>");
            }
        }

        if !validation.errors.is_empty() || !validation.warnings.is_empty() {
            html.push_str("</ul>");
            html.push_str("</div>");
        }
    }

    html.push_str("</div>");

    Html(html)
}

pub async fn handle_export(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<FileQuery>,
) -> impl IntoResponse {
    let _file_id = params.path.unwrap_or_else(|| "dialog".to_string());

    Html("<script>alert('Export started. File will download shortly.');</script>".to_string())
}

pub async fn handle_list_dialogs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let conn = state.conn.clone();

    let dialogs = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        diesel::sql_query(
            "SELECT id, name, content, updated_at FROM designer_dialogs ORDER BY updated_at DESC LIMIT 50",
        )
        .load::<DialogRow>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<div class=\"dialogs-list\">");

    for dialog in &dialogs {
        html.push_str("<div class=\"dialog-card\" data-id=\"");
        html.push_str(&html_escape(&dialog.id));
        html.push_str("\">");
        html.push_str("<h4>");
        html.push_str(&html_escape(&dialog.name));
        html.push_str("</h4>");
        html.push_str("<span class=\"dialog-time\">");
        html.push_str(&format_relative_time(dialog.updated_at));
        html.push_str("</span>");
        html.push_str("</div>");
    }

    if dialogs.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No dialogs yet</p>");
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}

pub async fn handle_create_dialog(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let now = Utc::now();
    let dialog_id = Uuid::new_v4().to_string();
    let name = payload.name.unwrap_or_else(|| "New Dialog".to_string());
    let content = payload.content.unwrap_or_else(get_default_dialog_content);

    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Err(format!("Database error: {}", e));
            }
        };

        diesel::sql_query(
            "INSERT INTO designer_dialogs (id, name, description, bot_id, content, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind::<diesel::sql_types::Text, _>(&dialog_id)
        .bind::<diesel::sql_types::Text, _>(&name)
        .bind::<diesel::sql_types::Text, _>("")
        .bind::<diesel::sql_types::Text, _>("default")
        .bind::<diesel::sql_types::Text, _>(&content)
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut db_conn)
        .map_err(|e| format!("Create failed: {}", e))?;

        Ok(dialog_id)
    })
    .await
    .unwrap_or_else(|e| Err(format!("Task error: {}", e)));

    match result {
        Ok(id) => {
            let mut html = String::new();
            html.push_str("<div class=\"dialog-created\" data-id=\"");
            html.push_str(&html_escape(&id));
            html.push_str("\">");
            html.push_str("<span class=\"success\">Dialog created</span>");
            html.push_str("</div>");
            Html(html)
        }
        Err(e) => {
            let mut html = String::new();
            html.push_str("<div class=\"error\">");
            html.push_str(&html_escape(&e));
            html.push_str("</div>");
            Html(html)
        }
    }
}

pub async fn handle_get_dialog(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let conn = state.conn.clone();

    let dialog = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return None;
            }
        };

        diesel::sql_query(
            "SELECT id, name, content, updated_at FROM designer_dialogs WHERE id = $1",
        )
        .bind::<diesel::sql_types::Text, _>(&id)
        .get_result::<DialogRow>(&mut db_conn)
        .ok()
    })
    .await
    .unwrap_or(None);

    match dialog {
        Some(d) => {
            let mut html = String::new();
            html.push_str("<div class=\"dialog-content\" data-id=\"");
            html.push_str(&html_escape(&d.id));
            html.push_str("\">");
            html.push_str("<div class=\"dialog-header\">");
            html.push_str("<h3>");
            html.push_str(&html_escape(&d.name));
            html.push_str("</h3>");
            html.push_str("</div>");
            html.push_str("<div class=\"dialog-code\">");
            html.push_str("<pre>");
            html.push_str(&html_escape(&d.content));
            html.push_str("</pre>");
            html.push_str("</div>");
            html.push_str("</div>");
            Html(html)
        }
        None => Html("<div class=\"error\">Dialog not found</div>".to_string()),
    }
}

fn validate_basic_code(code: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let lines: Vec<&str> = code.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("REM ") {
            continue;
        }

        let upper = trimmed.to_uppercase();

        if upper.starts_with("IF ") && !upper.contains(" THEN") {
            errors.push(ValidationError {
                line: line_num,
                column: 1,
                message: "IF statement missing THEN keyword".to_string(),
                node_id: None,
            });
        }

        if upper.starts_with("FOR ") && !upper.contains(" TO ") {
            errors.push(ValidationError {
                line: line_num,
                column: 1,
                message: "FOR statement missing TO keyword".to_string(),
                node_id: None,
            });
        }

        let quote_count = trimmed.chars().filter(|c| *c == '"').count();
        if quote_count % 2 != 0 {
            errors.push(ValidationError {
                line: line_num,
                column: trimmed.find('"').unwrap_or(0) + 1,
                message: "Unclosed string literal".to_string(),
                node_id: None,
            });
        }

        if upper.starts_with("GOTO ") {
            warnings.push(ValidationWarning {
                line: line_num,
                message: "GOTO statements can make code harder to maintain".to_string(),
                node_id: None,
            });
        }

        if trimmed.len() > 120 {
            warnings.push(ValidationWarning {
                line: line_num,
                message: "Line exceeds recommended length of 120 characters".to_string(),
                node_id: None,
            });
        }
    }

    let mut if_count = 0i32;
    let mut for_count = 0i32;
    let mut sub_count = 0i32;

    for line in &lines {
        let upper = line.to_uppercase();
        let trimmed = upper.trim();

        if trimmed.starts_with("IF ") && !trimmed.ends_with(" THEN") && trimmed.contains(" THEN") {
        } else if trimmed.starts_with("IF ") {
            if_count += 1;
        } else if trimmed == "END IF" || trimmed == "ENDIF" {
            if_count -= 1;
        }

        if trimmed.starts_with("FOR ") {
            for_count += 1;
        } else if trimmed == "NEXT" || trimmed.starts_with("NEXT ") {
            for_count -= 1;
        }

        if trimmed.starts_with("SUB ") {
            sub_count += 1;
        } else if trimmed == "END SUB" {
            sub_count -= 1;
        }
    }

    if if_count > 0 {
        errors.push(ValidationError {
            line: lines.len(),
            column: 1,
            message: format!("{} unclosed IF statement(s)", if_count),
            node_id: None,
        });
    }

    if for_count > 0 {
        errors.push(ValidationError {
            line: lines.len(),
            column: 1,
            message: format!("{} unclosed FOR loop(s)", for_count),
            node_id: None,
        });
    }

    if sub_count > 0 {
        errors.push(ValidationError {
            line: lines.len(),
            column: 1,
            message: format!("{} unclosed SUB definition(s)", sub_count),
            node_id: None,
        });
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

fn get_default_dialog_content() -> String {
    "' Welcome Dialog\n\
     ' Created with Dialog Designer\n\
     \n\
     SUB Main()\n\
         TALK \"Hello! How can I help you today?\"\n\
         \n\
         answer = HEAR\n\
         \n\
         IF answer LIKE \"*help*\" THEN\n\
             TALK \"I'm here to assist you.\"\n\
         ELSE IF answer LIKE \"*bye*\" THEN\n\
             TALK \"Goodbye!\"\n\
         ELSE\n\
             TALK \"I understand: \" + answer\n\
         END IF\n\
     END SUB\n"
        .to_string()
}

struct DialogNode {
    id: String,
    node_type: String,
    content: String,
    x: i32,
    y: i32,
}

fn parse_basic_to_nodes(content: &str) -> Vec<DialogNode> {
    let mut nodes = Vec::new();
    let mut y_pos = 100;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        let upper = trimmed.to_uppercase();
        let node_type = if upper.starts_with("TALK ") {
            "talk"
        } else if upper.starts_with("HEAR") {
            "hear"
        } else if upper.starts_with("IF ") {
            "if"
        } else if upper.starts_with("FOR ") {
            "for"
        } else if upper.starts_with("SET ") || upper.contains(" = ") {
            "set"
        } else if upper.starts_with("CALL ") {
            "call"
        } else if upper.starts_with("SUB ") {
            "sub"
        } else {
            continue;
        };

        nodes.push(DialogNode {
            id: format!("node-{}", i),
            node_type: node_type.to_string(),
            content: trimmed.to_string(),
            x: 400,
            y: y_pos,
        });

        y_pos += 80;
    }

    nodes
}

fn format_node_html(node: &DialogNode) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"canvas-node node-");
    html.push_str(&node.node_type);
    html.push_str("\" id=\"");
    html.push_str(&html_escape(&node.id));
    html.push_str("\" style=\"left: ");
    html.push_str(&node.x.to_string());
    html.push_str("px; top: ");
    html.push_str(&node.y.to_string());
    html.push_str("px;\" draggable=\"true\">");
    html.push_str("<div class=\"node-header\">");
    html.push_str("<span class=\"node-type\">");
    html.push_str(&node.node_type.to_uppercase());
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"node-content\">");
    html.push_str(&html_escape(&node.content));
    html.push_str("</div>");
    html.push_str("<div class=\"node-ports\">");
    html.push_str("<div class=\"port port-in\"></div>");
    html.push_str("<div class=\"port port-out\"></div>");
    html.push_str("</div>");
    html.push_str("</div>");
    html
}

fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(time);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        time.format("%b %d").to_string()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesignerModifyRequest {
    pub app_name: String,
    pub current_page: Option<String>,
    pub message: String,
    pub context: Option<DesignerContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesignerContext {
    pub page_html: Option<String>,
    pub tables: Option<Vec<String>>,
    pub recent_changes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignerModifyResponse {
    pub success: bool,
    pub message: String,
    pub changes: Vec<DesignerChange>,
    pub suggestions: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignerChange {
    pub change_type: String,
    pub file_path: String,
    pub description: String,
    pub preview: Option<String>,
}

pub async fn handle_designer_modify(
    State(state): State<Arc<AppState>>,
    Json(request): Json<DesignerModifyRequest>,
) -> impl IntoResponse {
    let app = &request.app_name;
    let msg_preview = &request.message[..request.message.len().min(100)];
    log::info!("Designer modify request for app '{app}': {msg_preview}");

    let session = match get_designer_session(&state) {
        Ok(s) => s,
        Err(e) => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(DesignerModifyResponse {
                    success: false,
                    message: "Authentication required".to_string(),
                    changes: Vec::new(),
                    suggestions: Vec::new(),
                    error: Some(e.to_string()),
                }),
            );
        }
    };

    match process_designer_modification(&state, &request, &session).await {
        Ok(response) => (axum::http::StatusCode::OK, Json(response)),
        Err(e) => {
            log::error!("Designer modification failed: {e}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(DesignerModifyResponse {
                    success: false,
                    message: "Failed to process modification".to_string(),
                    changes: Vec::new(),
                    suggestions: Vec::new(),
                    error: Some(e.to_string()),
                }),
            )
        }
    }
}

fn get_designer_session(
    state: &AppState,
) -> Result<crate::shared::models::UserSession, Box<dyn std::error::Error + Send + Sync>> {
    use crate::shared::models::schema::bots::dsl::*;
    use crate::shared::models::UserSession;

    let mut conn = state.conn.get()?;

    let bot_result: Result<(Uuid, String), _> = bots.select((id, name)).first(&mut conn);

    match bot_result {
        Ok((bot_id_val, _bot_name_val)) => Ok(UserSession {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            bot_id: bot_id_val,
            title: "designer".to_string(),
            context_data: serde_json::json!({}),
            current_tool: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }),
        Err(_) => Err("No bot found for designer session".into()),
    }
}

async fn process_designer_modification(
    state: &AppState,
    request: &DesignerModifyRequest,
    session: &crate::shared::models::UserSession,
) -> Result<DesignerModifyResponse, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = build_designer_prompt(request);
    let llm_response = call_designer_llm(state, &prompt).await?;
    let (changes, message, suggestions) =
        parse_and_apply_changes(state, request, &llm_response, session).await?;

    Ok(DesignerModifyResponse {
        success: true,
        message,
        changes,
        suggestions,
        error: None,
    })
}

fn build_designer_prompt(request: &DesignerModifyRequest) -> String {
    let context_info = request
        .context
        .as_ref()
        .map(|ctx| {
            let mut info = String::new();
            if let Some(ref html) = ctx.page_html {
                let _ = writeln!(
                    info,
                    "\nCurrent page HTML (first 500 chars):\n{}",
                    &html[..html.len().min(500)]
                );
            }
            if let Some(ref tables) = ctx.tables {
                let _ = writeln!(info, "\nAvailable tables: {}", tables.join(", "));
            }
            info
        })
        .unwrap_or_default();

    let error_context = get_designer_error_context(&request.app_name).unwrap_or_default();

    format!(
        r#"You are a Designer AI assistant helping modify an HTMX-based application.

App Name: {}
Current Page: {}
{}
{}
User Request: "{}"

Analyze the request and respond with JSON describing the changes needed:
{{
    "understanding": "brief description of what user wants",
    "changes": [
        {{
            "type": "modify_html|add_field|remove_field|add_table|modify_style|add_page",
            "file": "filename.html or styles.css",
            "description": "what this change does",
            "code": "the new/modified code snippet"
        }}
    ],
    "message": "friendly response to user explaining what was done",
    "suggestions": ["optional follow-up suggestions"]
}}

Guidelines:
- Use HTMX attributes (hx-get, hx-post, hx-target, hx-swap, hx-trigger)
- Keep styling minimal and consistent
- API endpoints follow pattern: /api/db/{{table_name}}
- Forms should use hx-post for submissions
- Lists should use hx-get with pagination

Respond with valid JSON only."#,
        request.app_name,
        request.current_page.as_deref().unwrap_or("index.html"),
        context_info,
        error_context,
        request.message
    )
}

async fn call_designer_llm(
    _state: &AppState,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let llm_url = std::env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let llm_model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/api/generate", llm_url))
        .json(&serde_json::json!({
            "model": llm_model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.3,
                "num_predict": 2000
            }
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(format!("LLM request failed: {status}").into());
    }

    let result: serde_json::Value = response.json().await?;
    let response_text = result["response"].as_str().unwrap_or("{}").to_string();

    let json_text = if response_text.contains("```json") {
        response_text
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(&response_text)
            .trim()
            .to_string()
    } else if response_text.contains("```") {
        response_text
            .split("```")
            .nth(1)
            .unwrap_or(&response_text)
            .trim()
            .to_string()
    } else {
        response_text
    };

    Ok(json_text)
}

async fn parse_and_apply_changes(
    state: &AppState,
    request: &DesignerModifyRequest,
    llm_response: &str,
    session: &crate::shared::models::UserSession,
) -> Result<(Vec<DesignerChange>, String, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct LlmChangeResponse {
        _understanding: Option<String>,
        changes: Option<Vec<LlmChange>>,
        message: Option<String>,
        suggestions: Option<Vec<String>>,
    }

    #[derive(Deserialize)]
    struct LlmChange {
        #[serde(rename = "type")]
        change_type: String,
        file: String,
        description: String,
        code: Option<String>,
    }

    let parsed: LlmChangeResponse = serde_json::from_str(llm_response).unwrap_or_else(|_| LlmChangeResponse {
        _understanding: Some("Could not parse LLM response".to_string()),
        changes: None,
        message: Some("I understood your request but encountered an issue processing it. Could you try rephrasing?".to_string()),
        suggestions: Some(vec!["Try being more specific".to_string()]),
    });

    let mut applied_changes = Vec::new();

    if let Some(changes) = parsed.changes {
        for change in changes {
            if let Some(ref code) = change.code {
                match apply_file_change(state, &request.app_name, &change.file, code, session).await
                {
                    Ok(()) => {
                        applied_changes.push(DesignerChange {
                            change_type: change.change_type,
                            file_path: change.file,
                            description: change.description,
                            preview: Some(code[..code.len().min(200)].to_string()),
                        });
                    }
                    Err(e) => {
                        let file = &change.file;
                        log::warn!("Failed to apply change to {file}: {e}");
                    }
                }
            }
        }
    }

    let message = parsed.message.unwrap_or_else(|| {
        if applied_changes.is_empty() {
            "I couldn't make any changes. Could you provide more details?".to_string()
        } else {
            format!(
                "Done! I made {} change(s) to your app.",
                applied_changes.len()
            )
        }
    });

    let suggestions = parsed.suggestions.unwrap_or_default();

    Ok((applied_changes, message, suggestions))
}

async fn apply_file_change(
    state: &AppState,
    app_name: &str,
    file_name: &str,
    content: &str,
    session: &crate::shared::models::UserSession,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::shared::models::schema::bots::dsl::*;

    let mut conn = state.conn.get()?;
    let bot_name_val: String = bots
        .filter(id.eq(session.bot_id))
        .select(name)
        .first(&mut conn)?;

    let bucket_name = format!("{}.gbai", bot_name_val.to_lowercase());
    let file_path = format!(".gbdrive/apps/{app_name}/{file_name}");

    if let Some(ref s3_client) = state.drive {
        use aws_sdk_s3::primitives::ByteStream;

        s3_client
            .put_object()
            .bucket(&bucket_name)
            .key(&file_path)
            .body(ByteStream::from(content.as_bytes().to_vec()))
            .content_type(get_content_type(file_name))
            .send()
            .await?;

        log::info!("Designer updated file: s3://{bucket_name}/{file_path}");

        let site_path = state
            .config
            .as_ref()
            .map(|c| c.site_path.clone())
            .unwrap_or_else(|| "./botserver-stack/sites".to_string());

        let local_path = format!("{site_path}/{app_name}/{file_name}");
        if let Some(parent) = std::path::Path::new(&local_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&local_path, content)?;

        log::info!("Designer synced to local: {local_path}");
    }

    Ok(())
}
