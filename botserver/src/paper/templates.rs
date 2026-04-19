use crate::core::shared::state::AppState;
use axum::{
    extract::State,
    http::HeaderMap,
    response::{Html, IntoResponse},
};
use chrono::Utc;
use std::fmt::Write;
use std::sync::Arc;
use uuid::Uuid;

use super::auth::get_current_user;
use super::handlers::handle_new_document;
use super::storage::save_document_to_drive;
use super::utils::format_document_content;

pub async fn handle_template_blank(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    handle_new_document(State(state), headers).await
}

pub async fn handle_template_meeting(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(super::utils::format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Meeting Notes".to_string();
    let now = Utc::now();

    let mut content = String::new();
    content.push_str("# Meeting Notes\n\n");
    let _ = writeln!(content, "**Date:** {}\n", now.format("%Y-%m-%d"));
    content.push_str("**Attendees:**\n- \n\n");
    content.push_str("## Agenda\n\n1. \n\n");
    content.push_str("## Discussion\n\n\n\n");
    content.push_str("## Action Items\n\n- [ ] \n\n");
    content.push_str("## Next Steps\n\n");

    let _ =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await;

    Html(format_document_content(&title, &content))
}

pub async fn handle_template_todo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(super::utils::format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "To-Do List".to_string();

    let mut content = String::new();
    content.push_str("# To-Do List\n\n");
    content.push_str("## High Priority\n\n- [ ] \n\n");
    content.push_str("## Medium Priority\n\n- [ ] \n\n");
    content.push_str("## Low Priority\n\n- [ ] \n\n");
    content.push_str("## Completed\n\n- [x] Example completed task\n");

    let _ =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await;

    Html(format_document_content(&title, &content))
}

pub async fn handle_template_research(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(super::utils::format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Research Notes".to_string();

    let mut content = String::new();
    content.push_str("# Research Notes\n\n");
    content.push_str("## Topic\n\n\n\n");
    content.push_str("## Research Questions\n\n1. \n\n");
    content.push_str("## Sources\n\n- \n\n");
    content.push_str("## Key Findings\n\n\n\n");
    content.push_str("## Analysis\n\n\n\n");
    content.push_str("## Conclusions\n\n\n\n");
    content.push_str("## References\n\n");

    let _ =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await;

    Html(format_document_content(&title, &content))
}

pub async fn handle_template_report(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(super::utils::format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Report".to_string();
    let now = Utc::now();

    let mut content = String::new();
    content.push_str("# Report\n\n");
    let _ = writeln!(content, "**Date:** {}\n", now.format("%Y-%m-%d"));
    content.push_str("**Author:**\n\n");
    content.push_str("---\n\n");
    content.push_str("## Executive Summary\n\n\n\n");
    content.push_str("## Introduction\n\n\n\n");
    content.push_str("## Background\n\n\n\n");
    content.push_str("## Findings\n\n### Key Finding 1\n\n\n\n### Key Finding 2\n\n\n\n");
    content.push_str("## Analysis\n\n\n\n");
    content.push_str("## Recommendations\n\n1. \n2. \n3. \n\n");
    content.push_str("## Conclusion\n\n\n\n");
    content.push_str("## Appendix\n\n");

    let _ =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await;

    Html(format_document_content(&title, &content))
}

pub async fn handle_template_letter(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let (_user_id, user_identifier) = match get_current_user(&state, &headers).await {
        Ok(u) => u,
        Err(e) => {
            log::error!("Auth error: {}", e);
            return Html(super::utils::format_error("Authentication required"));
        }
    };

    let doc_id = Uuid::new_v4().to_string();
    let title = "Letter".to_string();
    let now = Utc::now();

    let mut content = String::new();
    content.push_str("[Your Name]\n");
    content.push_str("[Your Address]\n");
    content.push_str("[City, State ZIP]\n");
    content.push_str("[Your Email]\n\n");
    let _ = writeln!(content, "{}\n", now.format("%B %d, %Y"));
    content.push_str("[Recipient Name]\n");
    content.push_str("[Recipient Title]\n");
    content.push_str("[Company/Organization]\n");
    content.push_str("[Address]\n");
    content.push_str("[City, State ZIP]\n\n");
    content.push_str("Dear [Recipient Name],\n\n");
    content.push_str("[Opening paragraph - State the purpose of your letter]\n\n");
    content.push_str("[Body paragraph(s) - Provide details, explanations, or supporting information]\n\n");
    content.push_str("[Closing paragraph - Summarize, request action, or express appreciation]\n\n");
    content.push_str("Sincerely,\n\n\n");
    content.push_str("[Your Signature]\n");
    content.push_str("[Your Typed Name]\n");

    let _ =
        save_document_to_drive(&state, &user_identifier, &doc_id, &title, &content, false).await;

    Html(format_document_content(&title, &content))
}
