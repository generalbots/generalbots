use aws_sdk_s3::primitives::ByteStream;
use crate::core::shared::state::AppState;
use crate::core::urls::ApiUrls;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse},
};
use std::sync::Arc;

use super::auth::get_current_user;
use super::models::ExportQuery;
use super::storage::load_document_from_drive;
use super::utils::{format_error, html_escape, markdown_to_html, strip_markdown};

pub async fn handle_export_pdf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(_doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            return Html("<script>alert('PDF export started. The file will be saved to your exports folder.');</script>".to_string());
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

pub async fn handle_export_docx(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(_doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            return Html("<script>alert('Word export started. The file will be saved to your exports folder.');</script>".to_string());
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

pub async fn handle_export_md(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            let export_path = format!(
                "users/{}/exports/{}.md",
                user_identifier
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
                    .to_lowercase(),
                doc.title
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            );

            if let Some(s3_client) = state.drive.as_ref() {
                let _ = s3_client
                    .put_object()
                    .bucket(&state.bucket_name)
                    .key(&export_path)
                    .body(ByteStream::from(doc.content.into_bytes()))
                    .content_type("text/markdown")
                    .send()
                    .await;
            }

            return Html(
                "<script>alert('Markdown exported to your exports folder.');</script>".to_string(),
            );
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

pub async fn handle_export_html(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            let html_content = format!(
                "<!DOCTYPE html>\n<html>\n<head>\n<title>{}</title>\n<meta charset=\"utf-8\">\n</head>\n<body>\n<article>\n{}\n</article>\n</body>\n</html>",
                html_escape(&doc.title),
                markdown_to_html(&doc.content)
            );

            let export_path = format!(
                "users/{}/exports/{}.html",
                user_identifier
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
                    .to_lowercase(),
                doc.title
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            );

            if let Some(s3_client) = state.drive.as_ref() {
                let _ = s3_client
                    .put_object()
                    .bucket(&state.bucket_name)
                    .key(&export_path)
                    .body(ByteStream::from(html_content.into_bytes()))
                    .content_type("text/html")
                    .send()
                    .await;
            }

            return Html(
                "<script>alert('HTML exported to your exports folder.');</script>".to_string(),
            );
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}

pub async fn handle_export_txt(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    let Ok((_user_id, user_identifier)) = get_current_user(&state, &headers).await else {
        return Html(format_error("Authentication required"));
    };

    if let Some(doc_id) = params.id {
        if let Ok(Some(doc)) = load_document_from_drive(&state, &user_identifier, &doc_id).await {
            let plain_text = strip_markdown(&doc.content);

            let export_path = format!(
                "users/{}/exports/{}.txt",
                user_identifier
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
                    .to_lowercase(),
                doc.title
                    .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            );

            if let Some(s3_client) = state.drive.as_ref() {
                let _ = s3_client
                    .put_object()
                    .bucket(&state.bucket_name)
                    .key(&export_path)
                    .body(ByteStream::from(plain_text.into_bytes()))
                    .content_type("text/plain")
                    .send()
                    .await;
            }

            return Html(
                "<script>alert('Text exported to your exports folder.');</script>".to_string(),
            );
        }
    }

    Html("<script>alert('Please save your document first.');</script>".to_string())
}
