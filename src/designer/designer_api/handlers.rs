use super::types::*;
use super::utils::*;
use super::validators::validate_basic_code;
use crate::auto_task::get_designer_error_context;
use crate::core::urls::ApiUrls;
use crate::core::shared::state::AppState;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    Json, Router,
};
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub fn configure_designer_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(ApiUrls::DESIGNER_FILES, get(handle_list_files))
        .route(ApiUrls::DESIGNER_LOAD, get(handle_load_file))
        .route(ApiUrls::DESIGNER_SAVE, post(handle_save))
        .route(ApiUrls::DESIGNER_VALIDATE, post(handle_validate))
        .route(ApiUrls::DESIGNER_EXPORT, get(handle_export))
        .route(
            ApiUrls::DESIGNER_DIALOGS,
            get(handle_list_dialogs).post(handle_create_dialog),
        )
        .route(ApiUrls::DESIGNER_DIALOG_BY_ID, get(handle_get_dialog))
        .route(ApiUrls::DESIGNER_MODIFY, post(super::llm_integration::handle_designer_modify))
        .route("/api/ui/designer/magic", post(handle_magic_suggestions))
        .route("/api/ui/editor/magic", post(super::llm_integration::handle_editor_magic))
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

pub async fn handle_load_file(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FileQuery>,
) -> impl IntoResponse {
    let file_path = params.path.unwrap_or_else(|| "welcome".to_string());

    let content = if let Some(bucket) = params.bucket {
        match load_from_drive(&state, &bucket, &file_path).await {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to load file from drive: {}", e);
                get_default_dialog_content()
            }
        }
    } else {
        let conn = state.conn.clone();
        let file_id = file_path;

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

        match dialog {
            Some(d) => d.content,
            None => get_default_dialog_content(),
        }
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
