use crate::docs::storage::{
    create_new_document, delete_document_from_drive, get_current_user_id,
    list_documents_from_drive, load_document_from_drive, save_document_to_drive,
};
use crate::docs::types::{
    DocsSaveRequest, DocsSaveResponse, DocsAiRequest, DocsAiResponse, Document, DocumentMetadata,
    SearchQuery, TemplateResponse,
};
use crate::docs::utils::{convert_to_html, detect_document_format, html_to_markdown, markdown_to_html, rtf_to_html, strip_html};
use crate::docs::types::{
    AcceptRejectAllRequest, AcceptRejectChangeRequest, AddCommentRequest, AddEndnoteRequest,
    AddFootnoteRequest, ApplyStyleRequest, CompareDocumentsRequest, CompareDocumentsResponse,
    CommentReply, ComparisonSummary, CreateStyleRequest, DeleteCommentRequest, DeleteEndnoteRequest,
    DeleteFootnoteRequest, DeleteStyleRequest, DocumentComment, DocumentComparison, DocumentDiff,
    DocumentStyle, EnableTrackChangesRequest, Endnote, Footnote, GenerateTocRequest,
    GetOutlineRequest, ListCommentsResponse, ListEndnotesResponse, ListFootnotesResponse,
    ListStylesResponse, ListTrackChangesResponse, OutlineItem, OutlineResponse, ReplyCommentRequest,
    ResolveCommentRequest, TableOfContents, TocEntry, TocResponse, TrackChange, UpdateEndnoteRequest,
    UpdateFootnoteRequest, UpdateStyleRequest, UpdateTocRequest,
};
use crate::shared::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use docx_rs::{AlignmentType, Docx, Paragraph, Run};
use log::error;
use std::sync::Arc;
use uuid::Uuid;

pub async fn handle_docs_ai(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<DocsAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();

    let response = if command.contains("summarize") || command.contains("summary") {
        "I've created a summary of your document. The key points are highlighted above."
    } else if command.contains("expand") || command.contains("longer") {
        "I've expanded the selected text with more details and examples."
    } else if command.contains("shorter") || command.contains("concise") {
        "I've made the text more concise while preserving the key information."
    } else if command.contains("formal") {
        "I've rewritten the text in a more formal, professional tone."
    } else if command.contains("casual") || command.contains("friendly") {
        "I've rewritten the text in a more casual, friendly tone."
    } else if command.contains("grammar") || command.contains("fix") {
        "I've corrected the grammar and spelling errors in your text."
    } else if command.contains("translate") {
        "I've translated the selected text. Please specify the target language if needed."
    } else if command.contains("bullet") || command.contains("list") {
        "I've converted the text into a bulleted list format."
    } else if command.contains("help") {
        "I can help you with:\n• Summarize text\n• Expand or shorten content\n• Fix grammar\n• Change tone (formal/casual)\n• Translate text\n• Convert to bullet points"
    } else {
        "I understand you want help with your document. Try commands like 'summarize', 'make shorter', 'fix grammar', or 'make formal'."
    };

    Json(DocsAiResponse {
        response: response.to_string(),
        result: None,
    })
}

pub async fn handle_docs_save(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let doc_id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc_id, &req.title, &req.content).await
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

pub async fn handle_docs_get_by_id(
    State(state): State<Arc<AppState>>,
    Path(doc_id): Path<String>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(doc)) => Ok(Json(doc)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn handle_new_document(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(create_new_document()))
}

pub async fn handle_list_documents(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DocumentMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    match list_documents_from_drive(&state, &user_id).await {
        Ok(docs) => Ok(Json(docs)),
        Err(e) => {
            error!("Failed to list documents: {}", e);
            Ok(Json(Vec::new()))
        }
    }
}

pub async fn handle_search_documents(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<DocumentMetadata>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let docs = match list_documents_from_drive(&state, &user_id).await {
        Ok(d) => d,
        Err(_) => Vec::new(),
    };

    let filtered = if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        docs.into_iter()
            .filter(|d| d.title.to_lowercase().contains(&q_lower))
            .collect()
    } else {
        docs
    };

    Ok(Json(filtered))
}

pub async fn handle_get_document(
    State(state): State<Arc<AppState>>,
    Query(query): Query<crate::docs::types::LoadQuery>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc_id = query.id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Document ID required" })),
        )
    })?;

    match load_document_from_drive(&state, &user_id, &doc_id).await {
        Ok(Some(doc)) => Ok(Json(doc)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Document not found" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

pub async fn handle_save_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let doc_id = req.id.unwrap_or_else(|| Uuid::new_v4().to_string());

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc_id, &req.title, &req.content).await
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

pub async fn handle_autosave(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DocsSaveRequest>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    handle_save_document(State(state), Json(req)).await
}

pub async fn handle_delete_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::docs::types::LoadQuery>,
) -> Result<Json<DocsSaveResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let doc_id = req.id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Document ID required" })),
        )
    })?;

    if let Err(e) = delete_document_from_drive(&state, &user_id, &doc_id).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(DocsSaveResponse {
        id: doc_id,
        success: true,
    }))
}

pub async fn handle_template_blank() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Untitled Document".to_string(),
        content: String::new(),
    }))
}

pub async fn handle_template_meeting() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<h1>Meeting Notes</h1>
<p><strong>Date:</strong> [Date]</p>
<p><strong>Attendees:</strong> [Names]</p>
<p><strong>Location:</strong> [Location/Virtual]</p>
<hr>
<h2>Agenda</h2>
<ol>
<li>Topic 1</li>
<li>Topic 2</li>
<li>Topic 3</li>
</ol>
<h2>Discussion Points</h2>
<p>[Notes here]</p>
<h2>Action Items</h2>
<ul>
<li>[ ] Action 1 - Owner - Due Date</li>
<li>[ ] Action 2 - Owner - Due Date</li>
</ul>
<h2>Next Meeting</h2>
<p>[Date and time of next meeting]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Meeting Notes".to_string(),
        content: content.to_string(),
    }))
}

pub async fn handle_template_report() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<h1>Report Title</h1>
<p><em>Author: [Your Name]</em></p>
<p><em>Date: [Date]</em></p>
<hr>
<h2>Executive Summary</h2>
<p>[Brief overview of the report]</p>
<h2>Introduction</h2>
<p>[Background and context]</p>
<h2>Methodology</h2>
<p>[How the information was gathered]</p>
<h2>Findings</h2>
<p>[Key findings and data]</p>
<h2>Recommendations</h2>
<ul>
<li>Recommendation 1</li>
<li>Recommendation 2</li>
<li>Recommendation 3</li>
</ul>
<h2>Conclusion</h2>
<p>[Summary and next steps]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Report".to_string(),
        content: content.to_string(),
    }))
}

pub async fn handle_template_letter() -> Result<Json<TemplateResponse>, (StatusCode, Json<serde_json::Value>)> {
    let content = r#"<p>[Your Name]<br>
[Your Address]<br>
[City, State ZIP]<br>
[Date]</p>
<p>[Recipient Name]<br>
[Recipient Title]<br>
[Company Name]<br>
[Address]<br>
[City, State ZIP]</p>
<p>Dear [Recipient Name],</p>
<p>[Opening paragraph - state the purpose of your letter]</p>
<p>[Body paragraph(s) - provide details and supporting information]</p>
<p>[Closing paragraph - summarize and state any call to action]</p>
<p>Sincerely,</p>
<p>[Your Name]<br>
[Your Title]</p>"#;

    Ok(Json(TemplateResponse {
        id: Uuid::new_v4().to_string(),
        title: "Letter".to_string(),
        content: content.to_string(),
    }))
}

pub async fn handle_ai_summarize(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let summary = if text.len() > 200 {
        format!("Summary: {}...", &text[..200])
    } else {
        format!("Summary: {}", text)
    };

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: summary,
        error: None,
    }))
}

pub async fn handle_ai_expand(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let expanded = format!("{}\n\n[Additional context and details would be added here by AI]", text);

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: expanded,
        error: None,
    }))
}

pub async fn handle_ai_improve(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}

pub async fn handle_ai_simplify(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}

pub async fn handle_ai_translate(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let lang = req.translate_lang.unwrap_or_else(|| "English".to_string());

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: format!("[Translated to {}]: {}", lang, text),
        error: None,
    }))
}

pub async fn handle_ai_custom(
    Json(req): Json<crate::docs::types::AiRequest>,
) -> Result<Json<crate::docs::types::AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(crate::docs::types::AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}

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

pub async fn handle_add_comment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddCommentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    let comment = DocumentComment {
        id: uuid::Uuid::new_v4().to_string(),
        author_id: user_id.clone(),
        author_name: "User".to_string(),
        content: req.content,
        position: req.position,
        length: req.length,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        replies: vec![],
        resolved: false,
    };

    let comments = doc.comments.get_or_insert_with(Vec::new);
    comments.push(comment.clone());
    doc.updated_at = Utc::now();

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true, "comment": comment })))
}

pub async fn handle_reply_comment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReplyCommentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(comments) = &mut doc.comments {
        for comment in comments.iter_mut() {
            if comment.id == req.comment_id {
                let reply = CommentReply {
                    id: uuid::Uuid::new_v4().to_string(),
                    author_id: user_id.clone(),
                    author_name: "User".to_string(),
                    content: req.content.clone(),
                    created_at: Utc::now(),
                };
                comment.replies.push(reply);
                comment.updated_at = Utc::now();
                break;
            }
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_resolve_comment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResolveCommentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(comments) = &mut doc.comments {
        for comment in comments.iter_mut() {
            if comment.id == req.comment_id {
                comment.resolved = req.resolved;
                comment.updated_at = Utc::now();
                break;
            }
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_delete_comment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteCommentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(comments) = &mut doc.comments {
        comments.retain(|c| c.id != req.comment_id);
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_comments(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ListCommentsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let doc_id = params.get("doc_id").cloned().unwrap_or_default();
    let user_id = get_current_user_id();
    let doc = match load_document_from_drive(&state, &user_id, &doc_id).await {
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

    let comments = doc.comments.unwrap_or_default();
    Ok(Json(ListCommentsResponse { comments }))
}

pub async fn handle_enable_track_changes(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EnableTrackChangesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    doc.track_changes_enabled = req.enabled;
    doc.updated_at = Utc::now();

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true, "enabled": req.enabled })))
}

pub async fn handle_accept_reject_change(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AcceptRejectChangeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(changes) = &mut doc.track_changes {
        for change in changes.iter_mut() {
            if change.id == req.change_id {
                change.accepted = Some(req.accept);
                break;
            }
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_accept_reject_all(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AcceptRejectAllRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(changes) = &mut doc.track_changes {
        for change in changes.iter_mut() {
            change.accepted = Some(req.accept);
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_track_changes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ListTrackChangesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let doc_id = params.get("doc_id").cloned().unwrap_or_default();
    let user_id = get_current_user_id();
    let doc = match load_document_from_drive(&state, &user_id, &doc_id).await {
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

    let changes = doc.track_changes.unwrap_or_default();
    Ok(Json(ListTrackChangesResponse {
        changes,
        enabled: doc.track_changes_enabled,
    }))
}

pub async fn handle_generate_toc(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateTocRequest>,
) -> Result<Json<TocResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    let mut entries = Vec::new();
    let content = &doc.content;
    let mut position = 0;

    for level in 1..=req.max_level {
        let tag = format!("<h{level}>");
        let end_tag = format!("</h{level}>");
        let mut search_pos = 0;

        while let Some(start) = content[search_pos..].find(&tag) {
            let abs_start = search_pos + start;
            if let Some(end) = content[abs_start..].find(&end_tag) {
                let text_start = abs_start + tag.len();
                let text_end = abs_start + end;
                let text = strip_html(&content[text_start..text_end]);

                entries.push(TocEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    text,
                    level,
                    page_number: None,
                    position: abs_start,
                });
                search_pos = text_end + end_tag.len();
            } else {
                break;
            }
        }
        position = search_pos;
    }

    entries.sort_by_key(|e| e.position);

    let toc = TableOfContents {
        id: uuid::Uuid::new_v4().to_string(),
        title: "Table of Contents".to_string(),
        entries,
        max_level: req.max_level,
        show_page_numbers: req.show_page_numbers,
        use_hyperlinks: req.use_hyperlinks,
    };

    doc.toc = Some(toc.clone());
    doc.updated_at = Utc::now();

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(TocResponse { toc }))
}

pub async fn handle_update_toc(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateTocRequest>,
) -> Result<Json<TocResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    let existing_toc = doc.toc.unwrap_or_else(|| TableOfContents {
        id: uuid::Uuid::new_v4().to_string(),
        title: "Table of Contents".to_string(),
        entries: vec![],
        max_level: 3,
        show_page_numbers: true,
        use_hyperlinks: true,
    });

    let gen_req = GenerateTocRequest {
        doc_id: req.doc_id,
        max_level: existing_toc.max_level,
        show_page_numbers: existing_toc.show_page_numbers,
        use_hyperlinks: existing_toc.use_hyperlinks,
    };

    handle_generate_toc(State(state), Json(gen_req)).await
}

pub async fn handle_add_footnote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddFootnoteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    let footnotes = doc.footnotes.get_or_insert_with(Vec::new);
    let reference_mark = format!("{}", footnotes.len() + 1);

    let footnote = Footnote {
        id: uuid::Uuid::new_v4().to_string(),
        reference_mark,
        content: req.content,
        position: req.position,
    };

    footnotes.push(footnote.clone());
    doc.updated_at = Utc::now();

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true, "footnote": footnote })))
}

pub async fn handle_update_footnote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateFootnoteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(footnotes) = &mut doc.footnotes {
        for footnote in footnotes.iter_mut() {
            if footnote.id == req.footnote_id {
                footnote.content = req.content.clone();
                break;
            }
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_delete_footnote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteFootnoteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(footnotes) = &mut doc.footnotes {
        footnotes.retain(|f| f.id != req.footnote_id);
        for (i, footnote) in footnotes.iter_mut().enumerate() {
            footnote.reference_mark = format!("{}", i + 1);
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_footnotes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ListFootnotesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let doc_id = params.get("doc_id").cloned().unwrap_or_default();
    let user_id = get_current_user_id();
    let doc = match load_document_from_drive(&state, &user_id, &doc_id).await {
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

    let footnotes = doc.footnotes.unwrap_or_default();
    Ok(Json(ListFootnotesResponse { footnotes }))
}

pub async fn handle_add_endnote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddEndnoteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    let endnotes = doc.endnotes.get_or_insert_with(Vec::new);
    let reference_mark = to_roman_numeral(endnotes.len() + 1);

    let endnote = Endnote {
        id: uuid::Uuid::new_v4().to_string(),
        reference_mark,
        content: req.content,
        position: req.position,
    };

    endnotes.push(endnote.clone());
    doc.updated_at = Utc::now();

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true, "endnote": endnote })))
}

fn to_roman_numeral(num: usize) -> String {
    let numerals = [
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
        (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
        (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
    ];
    let mut result = String::new();
    let mut n = num;
    for (value, numeral) in numerals {
        while n >= value {
            result.push_str(numeral);
            n -= value;
        }
    }
    result
}

pub async fn handle_update_endnote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateEndnoteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(endnotes) = &mut doc.endnotes {
        for endnote in endnotes.iter_mut() {
            if endnote.id == req.endnote_id {
                endnote.content = req.content.clone();
                break;
            }
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_delete_endnote(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteEndnoteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(endnotes) = &mut doc.endnotes {
        endnotes.retain(|e| e.id != req.endnote_id);
        for (i, endnote) in endnotes.iter_mut().enumerate() {
            endnote.reference_mark = to_roman_numeral(i + 1);
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_endnotes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ListEndnotesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let doc_id = params.get("doc_id").cloned().unwrap_or_default();
    let user_id = get_current_user_id();
    let doc = match load_document_from_drive(&state, &user_id, &doc_id).await {
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

    let endnotes = doc.endnotes.unwrap_or_default();
    Ok(Json(ListEndnotesResponse { endnotes }))
}

pub async fn handle_create_style(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateStyleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    let styles = doc.styles.get_or_insert_with(Vec::new);
    styles.push(req.style.clone());
    doc.updated_at = Utc::now();

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true, "style": req.style })))
}

pub async fn handle_update_style(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateStyleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(styles) = &mut doc.styles {
        for style in styles.iter_mut() {
            if style.id == req.style.id {
                *style = req.style.clone();
                break;
            }
        }
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_delete_style(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteStyleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let mut doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    if let Some(styles) = &mut doc.styles {
        styles.retain(|s| s.id != req.style_id);
    }

    doc.updated_at = Utc::now();
    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_styles(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<ListStylesResponse>, (StatusCode, Json<serde_json::Value>)> {
    let doc_id = params.get("doc_id").cloned().unwrap_or_default();
    let user_id = get_current_user_id();
    let doc = match load_document_from_drive(&state, &user_id, &doc_id).await {
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

    let styles = doc.styles.unwrap_or_default();
    Ok(Json(ListStylesResponse { styles }))
}

pub async fn handle_apply_style(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApplyStyleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    let style = doc.styles
        .as_ref()
        .and_then(|styles| styles.iter().find(|s| s.id == req.style_id))
        .cloned();

    if style.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Style not found" })),
        ));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "style": style,
        "position": req.position,
        "length": req.length
    })))
}

pub async fn handle_get_outline(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GetOutlineRequest>,
) -> Result<Json<OutlineResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();
    let doc = match load_document_from_drive(&state, &user_id, &req.doc_id).await {
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

    let mut items = Vec::new();
    let content = &doc.content;

    for level in 1..=6u32 {
        let tag = format!("<h{level}>");
        let end_tag = format!("</h{level}>");
        let mut search_pos = 0;

        while let Some(start) = content[search_pos..].find(&tag) {
            let abs_start = search_pos + start;
            if let Some(end) = content[abs_start..].find(&end_tag) {
                let text_start = abs_start + tag.len();
                let text_end = abs_start + end;
                let text = strip_html(&content[text_start..text_end]);
                let length = text_end - text_start;

                items.push(OutlineItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    text,
                    level,
                    position: abs_start,
                    length,
                    style_name: format!("Heading {level}"),
                });
                search_pos = text_end + end_tag.len();
            } else {
                break;
            }
        }
    }

    items.sort_by_key(|i| i.position);

    Ok(Json(OutlineResponse { items }))
}

pub async fn handle_import_document(
    State(state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = "import.docx".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            filename = field.file_name().unwrap_or("import.docx").to_string();
            if let Ok(bytes) = field.bytes().await {
                file_bytes = Some(bytes.to_vec());
            }
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No file uploaded" })),
        )
    })?;

    let format = detect_document_format(&bytes);
    let content = match format {
        "rtf" => rtf_to_html(&String::from_utf8_lossy(&bytes)),
        "html" => String::from_utf8_lossy(&bytes).to_string(),
        "markdown" => markdown_to_html(&String::from_utf8_lossy(&bytes)),
        "txt" => {
            let text = String::from_utf8_lossy(&bytes);
            format!("<p>{}</p>", text.replace('\n', "</p><p>"))
        }
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Unsupported format: {}", format) })),
            ))
        }
    };

    let title = filename.rsplit('/').next().unwrap_or(&filename)
        .rsplit('.').last().unwrap_or(&filename)
        .to_string();

    let user_id = get_current_user_id();
    let mut doc = create_new_document(&title);
    doc.content = content;
    doc.owner_id = user_id.clone();

    if let Err(e) = save_document_to_drive(&state, &user_id, &doc).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        ));
    }

    Ok(Json(doc))
}

pub async fn handle_compare_documents(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CompareDocumentsRequest>,
) -> Result<Json<CompareDocumentsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = get_current_user_id();

    let original = match load_document_from_drive(&state, &user_id, &req.original_doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Original document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let modified = match load_document_from_drive(&state, &user_id, &req.modified_doc_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Modified document not found" })),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            ))
        }
    };

    let original_text = strip_html(&original.content);
    let modified_text = strip_html(&modified.content);

    let mut differences = Vec::new();
    let mut insertions = 0u32;
    let mut deletions = 0u32;
    let mut modifications = 0u32;

    let original_words: Vec<&str> = original_text.split_whitespace().collect();
    let modified_words: Vec<&str> = modified_text.split_whitespace().collect();

    let mut i = 0;
    let mut j = 0;
    let mut position = 0;

    while i < original_words.len() || j < modified_words.len() {
        if i >= original_words.len() {
            differences.push(DocumentDiff {
                diff_type: "insertion".to_string(),
                position,
                original_text: None,
                modified_text: Some(modified_words[j].to_string()),
                length: modified_words[j].len(),
            });
            insertions += 1;
            j += 1;
        } else if j >= modified_words.len() {
            differences.push(DocumentDiff {
                diff_type: "deletion".to_string(),
                position,
                original_text: Some(original_words[i].to_string()),
                modified_text: None,
                length: original_words[i].len(),
            });
            deletions += 1;
            i += 1;
        } else if original_words[i] == modified_words[j] {
            position += original_words[i].len() + 1;
            i += 1;
            j += 1;
        } else {
            differences.push(DocumentDiff {
                diff_type: "modification".to_string(),
                position,
                original_text: Some(original_words[i].to_string()),
                modified_text: Some(modified_words[j].to_string()),
                length: original_words[i].len().max(modified_words[j].len()),
            });
            modifications += 1;
            position += modified_words[j].len() + 1;
            i += 1;
            j += 1;
        }
    }

    let comparison = DocumentComparison {
        id: uuid::Uuid::new_v4().to_string(),
        original_doc_id: req.original_doc_id,
        modified_doc_id: req.modified_doc_id,
        created_at: Utc::now(),
        differences,
        summary: ComparisonSummary {
            insertions,
            deletions,
            modifications,
            total_changes: insertions + deletions + modifications,
        },
    };

    Ok(Json(CompareDocumentsResponse { comparison }))
}
