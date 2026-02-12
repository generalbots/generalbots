use crate::core::shared::state::AppState;
use crate::docs::storage::{get_current_user_id, load_document_from_drive};
use crate::docs::utils::{html_to_markdown, strip_html};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use docx_rs::{AlignmentType, Docx, Paragraph, Run};
use std::sync::Arc;

pub async fn handle_export_pdf(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Ok((
        [(axum::http::header::CONTENT_TYPE, "application/pdf")],
        "PDF export not yet implemented".to_string(),
    ))
}

pub async fn handle_export_docx(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc = match load_document_from_drive(&state, &user_id, &query.id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let docx_bytes = html_to_docx(&doc.content, &doc.title);

    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )],
        docx_bytes,
    ))
}

fn html_to_docx(html: &str, title: &str) -> Vec<u8> {
    let plain_text = strip_html(html);
    let paragraphs: Vec<&str> = plain_text.split("\n\n").collect();

    let mut docx = Docx::new();

    let title_para = Paragraph::new()
        .add_run(Run::new().add_text(title).bold())
        .align(AlignmentType::Center);
    docx = docx.add_paragraph(title_para);

    for para_text in paragraphs {
        if !para_text.trim().is_empty() {
            let para = Paragraph::new().add_run(Run::new().add_text(para_text.trim()));
            docx = docx.add_paragraph(para);
        }
    }

    let mut buffer = Vec::new();
    if let Ok(_) = docx.build().pack(&mut std::io::Cursor::new(&mut buffer)) {
        buffer
    } else {
        Vec::new()
    }
}

pub async fn handle_export_md(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc = match load_document_from_drive(&state, &user_id, &query.id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let markdown = html_to_markdown(&doc.content);

    Ok(([(axum::http::header::CONTENT_TYPE, "text/markdown")], markdown))
}

pub async fn handle_export_html(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc = match load_document_from_drive(&state, &user_id, &query.id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
        doc.title, doc.content
    );

    Ok(([(axum::http::header::CONTENT_TYPE, "text/html")], full_html))
}

pub async fn handle_export_txt(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::ExportQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc = match load_document_from_drive(&state, &user_id, &query.id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let plain_text = strip_html(&doc.content);

    Ok(([(axum::http::header::CONTENT_TYPE, "text/plain")], plain_text))
}
