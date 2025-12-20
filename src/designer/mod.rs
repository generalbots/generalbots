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

pub fn configure_designer_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Match frontend /api/v1/designer/* endpoints
        .route("/api/v1/designer/files", get(handle_list_files))
        .route("/api/v1/designer/load", get(handle_load_file))
        .route("/api/v1/designer/save", post(handle_save))
        .route("/api/v1/designer/validate", post(handle_validate))
        .route("/api/v1/designer/export", get(handle_export))
        // Legacy endpoints for compatibility
        .route(
            "/api/designer/dialogs",
            get(handle_list_dialogs).post(handle_create_dialog),
        )
        .route("/api/designer/dialogs/{id}", get(handle_get_dialog))
}

/// GET /api/v1/designer/files - List available dialog files
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

/// GET /api/v1/designer/load - Load a specific dialog file
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

    // Return the canvas nodes as HTML for HTMX to swap
    let mut html = String::new();
    html.push_str("<div class=\"canvas-loaded\" data-content=\"");
    html.push_str(&html_escape(&content));
    html.push_str("\">");

    // Parse content and generate node HTML
    let nodes = parse_basic_to_nodes(&content);
    for node in &nodes {
        html.push_str(&format_node_html(node));
    }

    html.push_str("</div>");
    html.push_str("<script>initializeCanvas();</script>");

    Html(html)
}

/// POST /api/v1/designer/save - Save dialog
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

/// POST /api/v1/designer/validate - Validate dialog code
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
        html.push_str("<div class=\"validation-errors\">");
        html.push_str("<div class=\"validation-header\">");
        html.push_str("<span class=\"validation-icon\">x</span>");
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
        html.push_str("</ul>");
        html.push_str("</div>");
    }

    if !validation.warnings.is_empty() {
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
        html.push_str("</ul>");
        html.push_str("</div>");
    }

    html.push_str("</div>");

    Html(html)
}

/// GET /api/v1/designer/export - Export dialog as .bas file
pub async fn handle_export(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<FileQuery>,
) -> impl IntoResponse {
    let _file_id = params.path.unwrap_or_else(|| "dialog".to_string());

    // In production, this would generate and download the file
    Html("<script>alert('Export started. File will download shortly.');</script>".to_string())
}

/// GET /api/designer/dialogs - List dialogs (legacy endpoint)
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

/// POST /api/designer/dialogs - Create new dialog (legacy endpoint)
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

/// GET /api/designer/dialogs/{id} - Get specific dialog (legacy endpoint)
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

        // Check for common syntax issues
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

        // Check for unclosed strings
        let quote_count = trimmed.chars().filter(|c| *c == '"').count();
        if quote_count % 2 != 0 {
            errors.push(ValidationError {
                line: line_num,
                column: trimmed.find('"').unwrap_or(0) + 1,
                message: "Unclosed string literal".to_string(),
                node_id: None,
            });
        }

        // Warnings
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

    // Check block structures
    let mut if_count = 0i32;
    let mut for_count = 0i32;
    let mut sub_count = 0i32;

    for line in &lines {
        let upper = line.to_uppercase();
        let trimmed = upper.trim();

        if trimmed.starts_with("IF ") && !trimmed.ends_with(" THEN") && trimmed.contains(" THEN") {
            // Single-line IF
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
